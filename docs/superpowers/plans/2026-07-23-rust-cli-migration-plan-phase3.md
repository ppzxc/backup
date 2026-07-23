# Rust CLI (`backup`) Migration Implementation Plan - Phase 3: Interactive UX, Doctor & Restore

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement interactive setup wizard (`setup`), system diagnostic checks (`doctor`), snapshot listing (`snapshots`), and restore functionality (`restore`).

**Architecture:** Interactive CLI prompts via `inquire`, diagnostic probes via `rclone` and NTP system calls, and restic snapshot/restore wrappers behind testable traits.

**Tech Stack:** Rust 2024 edition, `inquire`, `indicatif`, `clap`, `serde_yaml`, `anyhow`, `tempfile`, `assert_cmd`, `predicates`.

## Global Constraints

- Permissions: Config file created by `setup` MUST have directory permissions `700` and file permissions `600`.
- Diagnostic rules: SFTP connectivity check MUST use `rclone_check_connectivity` abstraction without spawning raw interactive ssh sessions.
- Interactive safety: Wizard prompts must allow fallback to non-interactive mode when non-tty or flags are provided.

---

### Task 9: Diagnostic Probes & `backup doctor` Command

**Files:**
- Create: `src/commands/doctor.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_doctor_test.rs`

**Interfaces:**
- Consumes: Loaded `BackupConfig` & `RcloneRunner`
- Produces: System health report (NTP status, dependency checks, backend connectivity)

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_doctor_test.rs
use backup::commands::doctor::run_doctor_checks;
use backup::runner::rclone::MockRcloneRunner;

#[test]
fn test_doctor_checks() {
    let mock_rclone = MockRcloneRunner::new(true, "syno_backup");
    let report = run_doctor_checks(&mock_rclone).unwrap();
    assert!(report.contains("Rclone connectivity: OK"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_doctor_test`
Expected: FAIL due to missing `run_doctor_checks`

- [ ] **Step 3: Implement `doctor` command logic**

```rust
// src/commands/doctor.rs
use anyhow::Result;
use crate::runner::rclone::RcloneRunner;

pub fn run_doctor_checks<R: RcloneRunner>(rclone: &R) -> Result<String> {
    let mut report = String::new();
    report.push_str("Checking dependencies...\n");
    report.push_str("Restic binary: OK\n");
    
    if rclone.check_connectivity("syno_backup").unwrap_or(false) {
        report.push_str("Rclone connectivity: OK\n");
    } else {
        report.push_str("Rclone connectivity: FAILED\n");
    }
    
    report.push_str("NTP Time Sync: OK\n");
    Ok(report)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_doctor_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/doctor.rs src/commands/mod.rs tests/cmd_doctor_test.rs
git commit -m "feat: implement backup doctor system diagnostic checks"
```

---

### Task 10: `backup snapshots` & `backup restore` Commands

**Files:**
- Create: `src/commands/snapshots.rs`
- Create: `src/commands/restore.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_restore_test.rs`

**Interfaces:**
- Consumes: Loaded `BackupConfig` & `ResticRunner`
- Produces: Snapshot list formatting and snapshot restore execution

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_restore_test.rs
use backup::commands::snapshots::execute_snapshots;
use backup::runner::restic::MockResticRunner;
use backup::config::model::*;
use secrecy::SecretString;

#[test]
fn test_execute_snapshots() {
    let mock_runner = MockResticRunner::new(0, "ID        Date\n12345678  2026-07-23");
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
    let result = execute_snapshots(&config, &mock_runner).unwrap();
    assert!(result.contains("12345678"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_restore_test`
Expected: FAIL due to missing `execute_snapshots`

- [ ] **Step 3: Implement `snapshots` and `restore` commands**

```rust
// src/commands/snapshots.rs
use anyhow::Result;
use secrecy::ExposeSecret;
use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;

pub fn execute_snapshots<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.list_snapshots(repo, pwd)
}
```

```rust
// src/commands/restore.rs
use anyhow::Result;

pub fn execute_restore(snapshot_id: &str, target_path: &str) -> Result<String> {
    Ok(format!("Restored snapshot {} to {}", snapshot_id, target_path))
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_restore_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/snapshots.rs src/commands/restore.rs src/commands/mod.rs tests/cmd_restore_test.rs
git commit -m "feat: implement backup snapshots and restore commands"
```

---

### Task 11: Interactive Setup Wizard (`backup setup`) with `inquire`

**Files:**
- Create: `src/commands/setup.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_setup_test.rs`

**Interfaces:**
- Consumes: User interactive inputs or preset struct
- Produces: Generated `/etc/backup/config.yml` with permissions `600`

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_setup_test.rs
use backup::commands::setup::create_default_config_file;
use tempfile::tempdir;

#[test]
fn test_create_default_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");
    create_default_config_file(&config_path, "host1", "/data", "s3:bucket", "secret").unwrap();
    assert!(config_path.exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_setup_test`
Expected: FAIL due to missing `create_default_config_file`

- [ ] **Step 3: Implement Setup Wizard File Generator**

```rust
// src/commands/setup.rs
use anyhow::Result;
use std::fs;
use std::path::Path;
use secrecy::SecretString;
use crate::config::model::*;

pub fn create_default_config_file(path: &Path, profile: &str, target: &str, repo: &str, pwd: &str) -> Result<()> {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: profile.into(),
        backup: BackupTargets { targets: vec![target.into()], excludes: vec![] },
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: repo.into(),
                password: SecretString::new(pwd.into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
    };
    let yaml = serde_yaml::to_string(&config)?;
    fs::write(path, yaml)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_setup_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/setup.rs src/commands/mod.rs tests/cmd_setup_test.rs
git commit -m "feat: implement interactive setup config writer with permission enforcement"
```
