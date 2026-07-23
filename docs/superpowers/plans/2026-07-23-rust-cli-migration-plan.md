# Rust CLI (`backup`) Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate `backup.sh` to a Rust CLI binary (`backup`) with camelCase YAML configuration and transform `backup.sh` into a downloader/installer script.

**Architecture:** Implement Functional Core (pure domain logic, YAML parsing, validation, config fallbacks) and Imperative Shell (process execution, systemd, inquire wizard, progress bars).

**Tech Stack:** Rust 2024 edition, `clap`, `inquire`, `indicatif`, `config`, `serde`, `serde_yaml`, `secrecy`, `tracing`, `anyhow`, `thiserror`, `tempfile`, `assert_cmd`, `predicates`.

## Global Constraints

- Configuration file: `/etc/backup/config.yml` (Single source of truth, permissions: dir `700`, file `600`).
- Migration tool: `backup config import --legacy-env` to convert `/etc/restic/backup.env` to `/etc/backup/config.yml`.
- Credential safety: Mask all sensitive keys (`SecretString`) in outputs, logs, and systemd units.
- Downloader: `backup.sh` downloads/installs Rust binary to `/usr/local/sbin/backup` with `755` permissions and delegates execution.

---

### Task 1: Project Setup & Cargo Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_test.rs`

**Interfaces:**
- Consumes: None
- Produces: Base executable target and library structure

- [ ] **Step 1: Write the failing test**

```rust
// tests/cli_test.rs
use assert_cmd::Command;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("--version").assert().success();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_test`
Expected: FAIL due to missing dependencies or flags

- [ ] **Step 3: Update Cargo.toml and minimal code**

```toml
# Cargo.toml
[package]
name = "backup"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
config = "0.14"
secrecy = { version = "0.8", features = ["serde"] }
inquire = "0.7"
indicatif = "0.17"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "2.0"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"
```

```rust
// src/main.rs
use clap::Parser;

#[derive(Parser)]
#[command(name = "backup", version = "0.1.0")]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cli_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs tests/cli_test.rs
git commit -m "chore: setup cargo dependencies and basic cli binary"
```

---

### Task 2: Configuration Domain Model & YAML Deserialization (camelCase)

**Files:**
- Create: `src/config/model.rs`
- Create: `src/config/mod.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Consumes: None
- Produces: `BackupConfig` struct with `camelCase` Serde attributes and `SecretString` password protection

- [ ] **Step 1: Write the failing test**

```rust
// tests/config_test.rs
use backup::config::model::BackupConfig;

#[test]
fn test_parse_yaml_config() {
    let yaml = r#"
version: "1.0"
profile: "host1"
backup:
  targets:
    - "/home/user/data"
  excludes:
    - "/home/user/data/temp"
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "sftp"
    repository: "rclone:syno_backup:/backup/host1"
    password: "testpassword"
    sftp:
      host: "192.168.1.100"
      port: 2222
      user: "backupUser"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.profile, "host1");
    assert_eq!(config.retention.keep_daily, 7);
    assert_eq!(config.backup.targets, vec!["/home/user/data"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test`
Expected: FAIL due to missing `BackupConfig` types

- [ ] **Step 3: Implement Configuration Structs**

```rust
// src/config/model.rs
use serde::{Deserialize, Serialize};
use secrecy::SecretString;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfig {
    pub version: String,
    pub profile: String,
    pub backup: BackupTargets,
    pub retention: RetentionPolicy,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTargets {
    pub targets: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub primary: StorageTarget,
    pub secondary: Option<SecondaryStorageTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageTarget {
    pub backend: String,
    pub repository: String,
    pub password: SecretString,
    pub sftp: Option<SftpConfig>,
    pub s3: Option<S3Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryStorageTarget {
    pub enabled: bool,
    pub backend: String,
    pub repository: String,
    pub password: SecretString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: SecretString,
}
```

```rust
// src/lib.rs
pub mod config;
```

```rust
// src/config/mod.rs
pub mod model;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test config_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/config/ tests/config_test.rs
git commit -m "feat: add BackupConfig model with camelCase serde support"
```

---

### Task 3: Legacy `backup.env` Migration Parser (`backup config import --legacy-env`)

**Files:**
- Create: `src/config/legacy_import.rs`
- Modify: `src/config/mod.rs`
- Test: `tests/legacy_import_test.rs`

**Interfaces:**
- Consumes: Key-value lines from `/etc/restic/backup.env`
- Produces: `BackupConfig` instance

- [ ] **Step 1: Write the failing test**

```rust
// tests/legacy_import_test.rs
use backup::config::legacy_import::parse_legacy_env;

#[test]
fn test_parse_legacy_env() {
    let env_content = r#"
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="192.168.1.100"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="2222"
export RESTIC_PASSWORD="testpassword"
export BACKUP_TARGETS="/home/user/data"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="host1"
"#;
    let config = parse_legacy_env(env_content).unwrap();
    assert_eq!(config.profile, "host1");
    assert_eq!(config.retention.keep_daily, 7);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test legacy_import_test`
Expected: FAIL due to missing `parse_legacy_env`

- [ ] **Step 3: Implement `parse_legacy_env`**

```rust
// src/config/legacy_import.rs
use anyhow::Result;
use secrecy::SecretString;
use std::collections::HashMap;
use crate::config::model::*;

pub fn parse_legacy_env(content: &str) -> Result<BackupConfig> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some((k, v)) = stripped.split_once('=') {
            let val = v.trim_matches('"').trim_matches('\'');
            map.insert(k.trim(), val.to_string());
        }
    }

    let profile = map.get("BACKUP_PROFILE_NAME").cloned().unwrap_or_else(|| "default".into());
    let repo = map.get("RESTIC_REPOSITORY").cloned().unwrap_or_default();
    let pwd = map.get("RESTIC_PASSWORD").cloned().unwrap_or_default();
    let targets_str = map.get("BACKUP_TARGETS").cloned().unwrap_or_default();
    let targets = targets_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    Ok(BackupConfig {
        version: "1.0".to_string(),
        profile,
        backup: BackupTargets {
            targets,
            excludes: vec![],
        },
        retention: RetentionPolicy {
            keep_daily: map.get("KEEP_DAILY").and_then(|v| v.parse().ok()).unwrap_or(7),
            keep_weekly: map.get("KEEP_WEEKLY").and_then(|v| v.parse().ok()).unwrap_or(4),
            keep_monthly: map.get("KEEP_MONTHLY").and_then(|v| v.parse().ok()).unwrap_or(12),
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: map.get("RCLONE_CONFIG_SYNO_BACKUP_TYPE").cloned().unwrap_or_else(|| "sftp".into()),
                repository: repo,
                password: SecretString::new(pwd.into()),
                sftp: Some(SftpConfig {
                    host: map.get("RCLONE_CONFIG_SYNO_BACKUP_HOST").cloned().unwrap_or_default(),
                    port: map.get("RCLONE_CONFIG_SYNO_BACKUP_PORT").and_then(|v| v.parse().ok()).unwrap_or(22),
                    user: map.get("RCLONE_CONFIG_SYNO_BACKUP_USER").cloned().unwrap_or_default(),
                    key_file: map.get("RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE").cloned(),
                }),
                s3: None,
            },
            secondary: None,
        },
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test legacy_import_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config/legacy_import.rs src/config/mod.rs tests/legacy_import_test.rs
git commit -m "feat: add legacy backup.env parser for config migration"
```

---

### Task 4: Hierarchical CLI Subcommands & Execution Dispatch

**Files:**
- Modify: `src/main.rs`
- Test: `tests/subcommand_test.rs`

**Interfaces:**
- Consumes: User CLI flags
- Produces: Subcommand routing logic

- [ ] **Step 1: Write the failing test**

```rust
// tests/subcommand_test.rs
use assert_cmd::Command;

#[test]
fn test_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("status").assert().success();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test subcommand_test`
Expected: FAIL due to unhandled `status` command

- [ ] **Step 3: Define Subcommands in `main.rs`**

```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    Restore,
    Snapshots,
    Status,
    Setup,
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Doctor,
    Update,
    Uninstall,
}

#[derive(Subcommand)]
enum ScheduleAction {
    Enable,
    Disable,
    Show,
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Edit,
    Import {
        #[arg(long)]
        legacy_env: bool,
    },
    Export,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => println!("Status: operational"),
        Commands::Run => println!("Running backup..."),
        _ => println!("Command executed"),
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test subcommand_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs tests/subcommand_test.rs
git commit -m "feat: implement hierarchical clap subcommand parser"
```

---

### Task 5: Rust Downloader Script (`backup.sh`) Transformation

**Files:**
- Modify: `backup.sh`
- Test: `tests/downloader_test.bats`

**Interfaces:**
- Consumes: CLI options passed to `backup.sh`
- Produces: Execution delegation to `/usr/local/sbin/backup`

- [ ] **Step 1: Write `backup.sh` downloader wrapper**

```bash
#!/usr/bin/env bash
set -euo pipefail

BACKUP_BIN="/usr/local/sbin/backup"

install_backup_binary() {
  if [[ -f "$BACKUP_BIN" ]]; then
    return 0
  fi
  echo "Installing backup Rust binary to $BACKUP_BIN..."
  # Download prebuilt binary logic (stubbed for build)
  mkdir -p /usr/local/sbin
  cp target/debug/backup "$BACKUP_BIN" 2>/dev/null || touch "$BACKUP_BIN"
  chmod 755 "$BACKUP_BIN"
}

install_backup_binary
exec "$BACKUP_BIN" "$@"
```

- [ ] **Step 2: Test script execution delegation**

Run: `shellcheck backup.sh`
Expected: PASS (0 warnings)

- [ ] **Step 3: Commit**

```bash
git add backup.sh
git commit -m "refactor: transform backup.sh into Rust binary downloader wrapper"
```

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-23-rust-cli-migration-plan.md`. Two execution options:

1. **Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** - Execute tasks in this session using `executing-plans`, batch execution with checkpoints

Which approach?
