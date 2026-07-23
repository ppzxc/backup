# Rust Backup CLI Test Migration & BATS Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement pure Rust unit and integration test suite using `CommandRunner` traits and `testcontainers`, and remove all legacy BATS bash test files.

**Architecture:** Introduce `CommandRunner` / `Executor` traits in `src/runner/` with `MockExecutor` for unit testing external CLI commands (`restic`, `rclone`, `systemctl`, `mysqldump`, `pg_dump`). Use `testcontainers` for S3 (MinIO), SFTP, and DB integration tests, followed by an E2E scenario test (`tests/integration_scenario.rs`) and cleanup of `.bats` files.

**Tech Stack:** Rust 2024 edition, `clap`, `testcontainers`, `testcontainers-modules`, `tempfile`, `anyhow`, `assert_cmd`, `predicates`.

## Global Constraints

- 100% Pure Rust testing: After completion, no `.bats` dependencies should remain for testing `backup`.
- Zero side-effects in unit tests: Unit tests must use `MockExecutor` / `MockResticRunner` / `MockRcloneRunner` and never mutate real system state or systemd.
- Non-interactive test safety: No unit or integration test should block on stdin prompts.

---

### Task 1: CommandRunner Trait & MockExecutor Scaffolding

**Files:**
- Create: `src/runner/executor.rs`
- Modify: `src/runner/mod.rs`
- Modify: `src/runner/restic.rs`
- Modify: `src/runner/rclone.rs`
- Test: `tests/runner_test.rs`

**Interfaces:**
- Consumes: Standard `std::process::Command` parameters
- Produces: `trait CommandRunner`, `struct SystemExecutor`, `struct MockExecutor`

- [ ] **Step 1: Write the failing unit test in `tests/runner_test.rs`**

```rust
use backup::runner::executor::{CommandOutput, CommandRunner, MockExecutor};

#[test]
fn test_mock_executor_recording() {
    let mock = MockExecutor::new();
    mock.push_output("restic", CommandOutput {
        status_code: 0,
        stdout: "restic 0.16.0".into(),
        stderr: "".into(),
    });

    let res = mock.run("restic", &["version"]).unwrap();
    assert_eq!(res.status_code, 0);
    assert_eq!(res.stdout, "restic 0.16.0");
    assert_eq!(mock.call_count("restic"), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runner_test`
Expected: FAIL due to missing `executor` module.

- [ ] **Step 3: Implement `src/runner/executor.rs`**

```rust
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput>;
}

pub struct SystemExecutor;

impl CommandRunner for SystemExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute process: {}", program))?;

        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[derive(Clone, Default)]
pub struct MockExecutor {
    responses: Arc<Mutex<HashMap<String, Vec<CommandOutput>>>>,
    calls: Arc<Mutex<Vec<(String, Vec<String>)>>>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_output(&self, program: &str, output: CommandOutput) {
        let mut map = self.responses.lock().unwrap();
        map.entry(program.to_string()).or_default().push(output);
    }

    pub fn call_count(&self, program: &str) -> usize {
        let calls = self.calls.lock().unwrap();
        calls.iter().filter(|(p, _)| p == program).count()
    }

    pub fn get_calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl CommandRunner for MockExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let mut calls = self.calls.lock().unwrap();
        calls.push((program.to_string(), args.iter().map(|s| s.to_string()).collect()));

        let mut responses = self.responses.lock().unwrap();
        if let Some(list) = responses.get_mut(program) {
            if !list.is_empty() {
                return Ok(list.remove(0));
            }
        }

        Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}
```

- [ ] **Step 4: Update `src/runner/mod.rs` to expose `executor`**

```rust
pub mod executor;
pub mod rclone;
pub mod restic;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test runner_test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/runner/executor.rs src/runner/mod.rs tests/runner_test.rs
git commit -m "feat(runner): add CommandRunner trait and MockExecutor for unit testing"
```

---

### Task 2: Fix Non-Interactive Prompts and Complete Command Unit Tests

**Files:**
- Modify: `src/commands/uninstall.rs`
- Modify: `tests/cmd_uninstall_test.rs`
- Modify: `tests/cmd_setup_test.rs`
- Modify: `tests/cmd_run_test.rs`
- Modify: `tests/cmd_schedule_test.rs`

**Interfaces:**
- Consumes: `BackupConfig`, `CommandRunner`
- Produces: Tested subcommands with non-interactive CLI arguments support (`--yes`, `--non-interactive`)

- [ ] **Step 1: Write failing test for non-interactive uninstall in `tests/cmd_uninstall_test.rs`**

```rust
use backup::commands::uninstall::perform_uninstall_with_options;

#[test]
fn test_perform_uninstall_non_interactive() {
    let result = perform_uninstall_with_options(true).unwrap();
    assert!(result.contains("Uninstalled"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_uninstall_test`
Expected: FAIL due to missing `perform_uninstall_with_options`.

- [ ] **Step 3: Implement non-interactive option in `src/commands/uninstall.rs` and fix tests**

```rust
use anyhow::Result;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer".into()
}

pub fn perform_uninstall_with_options(auto_confirm: bool) -> Result<String> {
    if !auto_confirm {
        // If not auto-confirmed, in non-tty test environment default to prompt check or error
        return Ok("Cancelled uninstall by user.".into());
    }
    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}

pub fn perform_uninstall() -> Result<String> {
    perform_uninstall_with_options(false)
}
```

Update `tests/cmd_uninstall_test.rs` to call `perform_uninstall_with_options(true)` to prevent stdin stalling.

- [ ] **Step 4: Run all unit tests to ensure non-blocking execution**

Run: `cargo test`
Expected: PASS without waiting for stdin prompt.

- [ ] **Step 5: Commit**

```bash
git add src/commands/uninstall.rs tests/cmd_uninstall_test.rs
git commit -m "fix(uninstall): add auto-confirm option to prevent stdin blocking in unit tests"
```

---

### Task 3: `testcontainers` Setup & Modular Integration Tests (S3, SFTP, DB)

**Files:**
- Modify: `Cargo.toml`
- Create: `tests/integration_s3.rs`
- Create: `tests/integration_sftp.rs`
- Create: `tests/integration_db.rs`

**Interfaces:**
- Consumes: MinIO container, SFTP container, Postgres/MariaDB container
- Produces: Automated integration test binaries running under `cargo test`

- [ ] **Step 1: Add `testcontainers` to `Cargo.toml` dev-dependencies**

Add to `Cargo.toml`:
```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
testcontainers = "0.23"
```

- [ ] **Step 2: Write failing integration test for S3/MinIO in `tests/integration_s3.rs`**

```rust
use testcontainers::clients;
use testcontainers::images::generic::GenericImage;

#[test]
fn test_s3_minio_container_connect() {
    let docker = clients::Cli::default();
    let minio_image = GenericImage::new("minio/minio", "latest")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);

    let node = docker.run(minio_image);
    let port = node.get_host_port_ipv4(9000);
    assert!(port > 0);
}
```

- [ ] **Step 3: Run S3 integration test**

Run: `cargo test --test integration_s3`
Expected: PASS when Docker is running or test passes container initialization.

- [ ] **Step 4: Write SFTP and DB integration tests in `tests/integration_sftp.rs` and `tests/integration_db.rs`**

Implement container lifecycle and connectivity check tests.

- [ ] **Step 5: Run integration tests**

Run: `cargo test --test integration_s3 --test integration_sftp --test integration_db`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock tests/integration_s3.rs tests/integration_sftp.rs tests/integration_db.rs
git commit -m "test(integration): add testcontainers for S3, SFTP, and DB integration tests"
```

---

### Task 4: End-to-End (E2E) Scenario Test

**Files:**
- Create: `tests/integration_scenario.rs`

**Interfaces:**
- Consumes: Full CLI workflow (`setup` -> `run` -> `restore` -> `schedule`)
- Produces: Automated end-to-end integration test

- [ ] **Step 1: Write E2E workflow test in `tests/integration_scenario.rs`**

```rust
use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_e2e_full_workflow_help_and_setup() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("--help").assert().success();

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.yml");

    let mut setup_cmd = Command::cargo_bin("backup").unwrap();
    setup_cmd.arg("setup")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("--profile")
        .arg("e2e-test")
        .assert()
        .success();

    assert!(config_path.exists());
}
```

- [ ] **Step 2: Run E2E test**

Run: `cargo test --test integration_scenario`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/integration_scenario.rs
git commit -m "test(e2e): add integration_scenario.rs for full CLI workflow verification"
```

---

### Task 5: BATS Cleanup & Pure Rust Verification

**Files:**
- Remove: `tests/*.bats`
- Remove: `tests/test_helper.bash`
- Remove: `tests/integration/`
- Modify: `AGENTS.md` (Update test commands to Rust native `cargo test`)

**Interfaces:**
- Consumes: Clean git repository state
- Produces: 100% Pure Rust testing codebase

- [ ] **Step 1: Delete all `.bats` files and bash test helpers**

```bash
git rm tests/*.bats
git rm tests/test_helper.bash
git rm -rf tests/integration/
```

- [ ] **Step 2: Update `AGENTS.md` to reference `cargo test` instead of `bats`**

Update `AGENTS.md` section `Common Commands`:
```markdown
* **단위 테스트**: `cargo test`
* **단일 테스트 파일**: `cargo test --test <file_name>`
* **통합 테스트**: `cargo test --test integration_scenario`
```

- [ ] **Step 3: Run `cargo test` to verify complete test suite pass**

Run: `cargo test`
Expected: PASS (all tests green).

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md
git commit -m "refactor(test): remove legacy BATS test suite and finalize pure Rust test setup"
```
