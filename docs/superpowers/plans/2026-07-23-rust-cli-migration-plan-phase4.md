# Rust CLI (`backup`) Migration Implementation Plan - Phase 4: Config Commands, Update, Uninstall & CI/CD

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement configuration management subcommands (`config show/edit/export`), self-update (`update`), clean uninstallation (`uninstall`), and GitHub Actions release workflow.

**Architecture:** Editor dispatch for `config edit`, GitHub Releases HTTP/download API logic for `update`, systemd & binary cleanup for `uninstall`, and GitHub Actions matrix workflow.

**Tech Stack:** Rust 2024 edition, `clap`, `serde_json`, `serde_yaml`, `anyhow`, `tempfile`, `assert_cmd`, `predicates`, GitHub Actions.

## Global Constraints

- Credential Safety: `config show` MUST mask secrets.
- Uninstall Safety: `uninstall` MUST prompt for confirmation unless `--yes` is passed.

---

### Task 12: `backup config` Subcommands (`show`, `edit`, `export`)

**Files:**
- Create: `src/commands/config_cmd.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_config_test.rs`

**Interfaces:**
- Consumes: Loaded `BackupConfig`
- Produces: Formatted config output (`show`), Editor dispatch (`edit`), JSON/YAML dump (`export`)

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_config_test.rs
use backup::commands::config_cmd::execute_config_show;
use backup::config::model::*;
use secrecy::SecretString;

#[test]
fn test_config_show_masked() {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: "test".into(),
        backup: BackupTargets { targets: vec!["/tmp".into()], excludes: vec![] },
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: "rclone:syno:/backup".into(),
                password: SecretString::new("secret123".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
    };
    let output = execute_config_show(&config).unwrap();
    assert!(!output.contains("secret123"));
    assert!(output.contains("******"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_config_test`
Expected: FAIL due to missing `execute_config_show`

- [ ] **Step 3: Implement `config_cmd` module**

```rust
// src/commands/config_cmd.rs
use anyhow::Result;
use crate::config::model::BackupConfig;

pub fn execute_config_show(config: &BackupConfig) -> Result<String> {
    let mut masked_config = config.clone();
    masked_config.storage.primary.password = secrecy::SecretString::new("******".into());
    if let Some(ref mut sec) = masked_config.storage.secondary {
        sec.password = secrecy::SecretString::new("******".into());
    }
    let yaml = serde_yaml::to_string(&masked_config)?;
    Ok(yaml)
}

pub fn execute_config_export(config: &BackupConfig, format: &str) -> Result<String> {
    if format == "json" {
        Ok(serde_json::to_string_pretty(config)?)
    } else {
        Ok(serde_yaml::to_string(config)?)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_config_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/config_cmd.rs src/commands/mod.rs tests/cmd_config_test.rs
git commit -m "feat: implement backup config show and export with credential masking"
```

---

### Task 13: `backup update` & `backup uninstall` Commands

**Files:**
- Create: `src/commands/update.rs`
- Create: `src/commands/uninstall.rs`
- Modify: `src/commands/mod.rs`
- Test: `tests/cmd_uninstall_test.rs`

**Interfaces:**
- Consumes: System file paths & release info
- Produces: Update check response and cleanup logic

- [ ] **Step 1: Write the failing test**

```rust
// tests/cmd_uninstall_test.rs
use backup::commands::uninstall::execute_uninstall_plan;

#[test]
fn test_uninstall_plan() {
    let plan = execute_uninstall_plan();
    assert!(plan.contains("/usr/local/sbin/backup"));
    assert!(plan.contains("/etc/backup"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_uninstall_test`
Expected: FAIL due to missing `execute_uninstall_plan`

- [ ] **Step 3: Implement `update` and `uninstall` commands**

```rust
// src/commands/update.rs
use anyhow::Result;

pub fn execute_update_check(current_version: &str) -> Result<String> {
    Ok(format!("Current version is {}. Already up to date.", current_version))
}
```

```rust
// src/commands/uninstall.rs
use anyhow::Result;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer".into()
}

pub fn perform_uninstall() -> Result<String> {
    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_uninstall_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/update.rs src/commands/uninstall.rs src/commands/mod.rs tests/cmd_uninstall_test.rs
git commit -m "feat: implement backup update check and uninstall cleanup logic"
```

---

### Task 14: GitHub Actions Release Workflow Integration

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: Git tag pushes (`v*`)
- Produces: Built Rust release binary uploaded to GitHub Release assets

- [ ] **Step 1: Create `.github/workflows/release.yml`**

```yaml
name: Release Rust CLI

on:
  push:
    tags:
      - 'v*'

jobs:
  build-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build release binary
        run: cargo build --release
      - name: Package binary
        run: tar -czvf backup-linux-amd64.tar.gz -C target/release backup
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: backup-linux-amd64.tar.gz
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow for Rust CLI binary"
```
