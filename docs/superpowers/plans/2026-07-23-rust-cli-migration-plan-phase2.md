# Rust CLI (`backup`) Migration Implementation Plan - Phase 2: Core Business Logic

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement full business logic for all `backup` subcommands in Rust (`run`, `status`, `doctor`, `schedule`, `setup`, `config`, `restore`, `snapshots`).

**Architecture:** Imperative process execution wrapper traits (`ResticRunner`, `RcloneRunner`, `SystemdRunner`) for isolated testing, backed by pure domain logic using `--password-file` and tempfiles for credential safety.

**Tech Stack:** Rust 2024 edition, `clap`, `inquire`, `indicatif`, `config`, `serde`, `secrecy`, `tempfile`, `tracing`, `anyhow`, `assert_cmd`, `predicates`.

## Global Constraints

- Configuration file: `/etc/backup/config.yml` (Permissions: dir `700`, file `600`).
- Process runner abstraction: Use `std::process::Command` wrapped behind traits for test stubbing.
- Credential safety: MUST use `--password-file` with `tempfile::NamedTempFile` (mode 600) instead of passing passwords via `RESTIC_PASSWORD` environment variables to prevent `/proc/<pid>/environ` leakage.
- Masked logging: Never expose plain passwords in logs or terminal outputs.

---

### Task 6: External Process Runner Abstraction (`ResticRunner` & `RcloneRunner` with NamedTempFile)

**Files:**
- Create: `src/runner/mod.rs`
- Create: `src/runner/restic.rs`
- Create: `src/runner/rclone.rs`
- Test: `tests/runner_test.rs`

**Interfaces:**
- Consumes: `BackupConfig` credentials & options
- Produces: `ResticRunner` and `RcloneRunner` traits for command invocation

- [ ] **Step 1: Write the failing test**

```rust
// tests/runner_test.rs
use backup::runner::restic::MockResticRunner;

#[test]
fn test_mock_restic_runner() {
    let runner = MockResticRunner::new(0, "repository initialized");
    let output = runner.init_repo("s3:bucket", "secret").unwrap();
    assert!(output.contains("repository initialized"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runner_test`
Expected: FAIL due to missing `MockResticRunner`

- [ ] **Step 3: Implement Process Runner Traits with TempFile Password Delivery**

```rust
// src/runner/restic.rs
use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

pub trait ResticRunner {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String>;
    fn backup_paths(&self, repo: &str, password: &str, targets: &[String], excludes: &[String]) -> Result<String>;
    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String>;
}

pub struct SystemResticRunner;

fn create_temp_password_file(password: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(password.as_bytes())?;
    file.flush()?;
    Ok(file)
}

impl ResticRunner for SystemResticRunner {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let output = Command::new("restic")
            .arg("-r").arg(repo)
            .arg("--password-file").arg(pass_file.path())
            .arg("init")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn backup_paths(&self, repo: &str, password: &str, targets: &[String], excludes: &[String]) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let mut cmd = Command::new("restic");
        cmd.arg("-r").arg(repo)
           .arg("--password-file").arg(pass_file.path())
           .arg("backup");
        for t in targets { cmd.arg(t); }
        for e in excludes { cmd.arg("--exclude").arg(e); }
        let output = cmd.output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let output = Command::new("restic")
            .arg("-r").arg(repo)
            .arg("--password-file").arg(pass_file.path())
            .arg("snapshots")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub struct MockResticRunner {
    pub exit_code: i32,
    pub response: String,
}

impl MockResticRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self { exit_code, response: response.to_string() }
    }
}

impl ResticRunner for MockResticRunner {
    fn init_repo(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn backup_paths(&self, _repo: &str, _password: &str, _targets: &[String], _excludes: &[String]) -> Result<String> {
        Ok(self.response.clone())
    }
    fn list_snapshots(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}
```

```rust
// src/runner/mod.rs
pub mod restic;
pub mod rclone;
```

```rust
// src/lib.rs (update)
pub mod config;
pub mod runner;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test runner_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/runner/ src/lib.rs tests/runner_test.rs
git commit -m "feat: add restic and rclone process runner abstractions with NamedTempFile password security"
```

---

### Task 7: Implementation of `backup run` and `backup status` Commands

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/run.rs`
- Create: `src/commands/status.rs`
- Modify: `src/main.rs`
- Test: `tests/cmd_run_test.rs`

**Interfaces:**
- Consumes: Loaded `BackupConfig` & `ResticRunner`
- Produces: Execution logic for `backup run` & `backup status`

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_run_test.rs
use backup::commands::run::execute_run;
use backup::runner::restic::MockResticRunner;
use backup::config::model::*;
use secrecy::SecretString;

#[test]
fn test_execute_run() {
    let mock_runner = MockResticRunner::new(0, "backup complete");
    let config = BackupConfig {
        version: "1.0".into(),
        profile: "test".into(),
        backup: BackupTargets { targets: vec!["/tmp".into()], excludes: vec![] },
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: "rclone:syno:/backup".into(),
                password: SecretString::new("secret".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
    };
    let result = execute_run(&config, &mock_runner).unwrap();
    assert!(result.contains("backup complete"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_run_test`
Expected: FAIL due to missing `execute_run`

- [ ] **Step 3: Implement `execute_run` and `execute_status`**

```rust
// src/commands/run.rs
use anyhow::Result;
use secrecy::ExposeSecret;
use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;

pub fn execute_run<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.backup_paths(repo, pwd, &config.backup.targets, &config.backup.excludes)
}
```

```rust
// src/commands/status.rs
use anyhow::Result;
use crate::config::model::BackupConfig;

pub fn execute_status(config: &BackupConfig) -> Result<String> {
    Ok(format!(
        "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
        config.profile,
        config.storage.primary.backend,
        config.storage.primary.repository,
        config.backup.targets
    ))
}
```

```rust
// src/commands/mod.rs
pub mod run;
pub mod status;
```

```rust
// src/lib.rs (update)
pub mod config;
pub mod runner;
pub mod commands;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_run_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/ src/lib.rs tests/cmd_run_test.rs
git commit -m "feat: implement backup run and backup status business logic"
```

---

### Task 8: Systemd Schedule Management (`backup schedule <enable|disable|show>`)

**Files:**
- Create: `src/commands/schedule.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_schedule_test.rs`

**Interfaces:**
- Consumes: Systemd unit templates
- Produces: Service & timer file management under `/etc/systemd/system/`

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_schedule_test.rs
use backup::commands::schedule::generate_systemd_timer;

#[test]
fn test_generate_systemd_timer() {
    let timer = generate_systemd_timer("02:00:00");
    assert!(timer.contains("OnCalendar=*-*-* 02:00:00"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_schedule_test`
Expected: FAIL due to missing `generate_systemd_timer`

- [ ] **Step 3: Implement Schedule Generator**

```rust
// src/commands/schedule.rs
pub fn generate_systemd_service(binary_path: &str) -> String {
    format!(r#"[Unit]
Description=Restic Backup Service
After=network.target

[Service]
Type=oneshot
ExecStart={} run
"#; binary_path)
}

pub fn generate_systemd_timer(on_calendar: &str) -> String {
    format!(r#"[Unit]
Description=Restic Backup Timer

[Timer]
OnCalendar=*-*-* {}
Persistent=true

[Install]
WantedBy=timers.target
"#, on_calendar)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_schedule_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/schedule.rs tests/cmd_schedule_test.rs
git commit -m "feat: implement systemd service and timer generator"
```
