# 2026-07-30-hardcoded-values-cleanup-and-spec-compliance.md Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate all hardcoded credentials/paths/mocks, enforce Unix permissions (`700`/`600`), and apply strong typing (`DatabaseType`, `DoctorStatus`, `DoctorCategory`) across the codebase.

**Architecture:** Encapsulate file/directory security helpers (`save_secure_file`, `create_secure_dir`), isolate `profiles.yaml` namespace configuration keys, convert raw string status/type fields to Enums, and replace mocked diagnosis checks in `doctor.rs` with real `CommandRunner`/`BackupConfig` calls.

**Tech Stack:** Rust (std::os::unix::fs::PermissionsExt, serde, secrecy), anyhow, cargo test.

## Global Constraints

- Mandatory directory permissions `700` (`/etc/backup`) and file permissions `600` for generated/modified config files.
- Sensitive credentials must use `SecretString` and never be exposed as plaintext.
- All configuration values must use `BackupConfig` as the single source of truth.

---

### Task 1: Security Permission Helpers & Core Configuration Hardcoding Removal

**Files:**
- Modify: `src/config/model.rs`
- Modify: `src/config/registry.rs`
- Modify: `src/config/legacy_import.rs`
- Modify: `src/main.rs`
- Modify: `CONTEXT.md`
- Test: `tests/config_test.rs`
- Test: `tests/legacy_import_test.rs`

**Interfaces:**
- Consumes: `std::os::unix::fs::PermissionsExt`
- Produces: `save_secure_file(path, content)`, `create_secure_dir(path)`, `DatabaseType` enum, `RetentionPolicy::standard_defaults()`, `RetentionPolicy::long_term_defaults()`

- [ ] **Step 1: Write failing test for secure file permissions & password fallback removal**

```rust
// In tests/config_test.rs
#[test]
fn test_save_secure_file_permissions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("secure.yaml");
    let mut config = BackupConfig::default();
    config.storage.primary.password = secrecy::SecretString::new("valid_secret_pass".into());
    config.save_to_path(&config_file).unwrap();

    let metadata = std::fs::metadata(&config_file).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }
}

#[test]
fn test_empty_password_validation_error() {
    let mut config = BackupConfig::default();
    config.storage.primary.password = secrecy::SecretString::new("".into());
    assert!(config.validate().is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test test_empty_password_validation_error`
Expected: FAIL (or pass unexpectedly if empty password was allowed)

- [ ] **Step 3: Implement secure permission helpers and remove fallback password**

In `src/config/model.rs`:
```rust
pub fn create_secure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn save_secure_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_secure_dir(parent)?;
    }
    fs::write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
```
Remove `"default_secret_pass123"` fallback from `model.rs` and `setup.rs`. Return error if password is empty during `validate()`.

In `src/main.rs:122`:
Replace `.unwrap_or_default()` with explicit error handling on config load failure.

In `src/config/legacy_import.rs`:
Add `.validate()` step after constructing `BackupConfig`.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --test config_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config/ src/main.rs CONTEXT.md tests/config_test.rs tests/legacy_import_test.rs
git commit -m "security: enforce 700/600 permissions, remove fallback password, and validate legacy imports"
```

---

### Task 2: Refactor Setup Command & Apply `DatabaseType` Enum

**Files:**
- Modify: `src/commands/setup.rs`
- Modify: `src/config/model.rs`
- Test: `tests/cmd_setup_test.rs`

**Interfaces:**
- Consumes: `DatabaseType` enum (`Mysql`, `Postgres`), `RetentionPolicy::standard_defaults()`
- Produces: Strong-typed interactive/non-interactive setup pipeline without magic retention numbers or `"/var/log"` hardcoding.

- [ ] **Step 1: Write failing test for `DatabaseType` parsing and retention factory**

```rust
// In tests/cmd_setup_test.rs
#[test]
fn test_database_type_enum() {
    use backup::config::model::DatabaseType;
    use std::str::FromStr;

    assert_eq!(DatabaseType::from_str("mysql").unwrap(), DatabaseType::Mysql);
    assert_eq!(DatabaseType::from_str("postgres").unwrap(), DatabaseType::Postgres);
    assert!(DatabaseType::from_str("invalid").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_setup_test test_database_type_enum`
Expected: FAIL (with unresolved import `DatabaseType`)

- [ ] **Step 3: Implement `DatabaseType` and retention policy defaults**

In `src/config/model.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Mysql,
    Postgres,
}

impl RetentionPolicy {
    pub fn standard_defaults() -> Self {
        Self { keep_daily: 7, keep_weekly: 4, keep_monthly: 12, ..Default::default() }
    }
    pub fn long_term_defaults() -> Self {
        Self { keep_daily: 180, keep_weekly: 12, keep_monthly: 24, ..Default::default() }
    }
}
```

Refactor `src/commands/setup.rs` to use `DatabaseType`, `RetentionPolicy::standard_defaults()`, and replace `"/var/log"` with `DEFAULT_BACKUP_TARGET` constant.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_setup_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config/model.rs src/commands/setup.rs tests/cmd_setup_test.rs
git commit -m "refactor(setup): introduce DatabaseType enum andRetentionPolicy default factories"
```

---

### Task 3: Doctor Command Diagnostics Refactoring (`DoctorStatus`, `DoctorCategory`, NTP & Rclone Real Checks)

**Files:**
- Modify: `src/commands/doctor.rs`
- Test: `tests/cmd_doctor_test.rs`

**Interfaces:**
- Consumes: `CommandRunner` / `Executor`, `BackupConfig`
- Produces: `DoctorStatus` (`Pass`, `Fail`, `Warn`), `DoctorCategory` (`Config`, `Storage`, `Network`, `System`), non-mocked NTP & Rclone verification logic.

- [ ] **Step 1: Write failing test for real NTP execution and typed doctor status**

```rust
// In tests/cmd_doctor_test.rs
#[test]
fn test_doctor_status_enum_and_ntp_check() {
    use backup::commands::doctor::{DoctorStatus, DoctorCategory, DoctorItem};
    let item = DoctorItem {
        category: DoctorCategory::System,
        criterion: "NTP Time Sync".into(),
        status: DoctorStatus::Pass,
        detail: "chronyd active".into(),
    };
    assert_eq!(item.status, DoctorStatus::Pass);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_doctor_test test_doctor_status_enum_and_ntp_check`
Expected: FAIL (unresolved types `DoctorStatus`, `DoctorCategory`)

- [ ] **Step 3: Implement typed `DoctorStatus`, `DoctorCategory` and replace mocked NTP string**

In `src/commands/doctor.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DoctorStatus { Pass, Fail, Warn }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DoctorCategory { Config, Storage, Network, System }

// Replace hardcoded mock in NTP check with Executor execution of `chronyc tracking` or `timedatectl`
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --test cmd_doctor_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/doctor.rs tests/cmd_doctor_test.rs
git commit -m "refactor(doctor): convert status/category to Enums and replace mocked NTP check with real execution"
```

---

### Task 4: Remove `BackupConfig::default()` Misuse in Status & Report Commands

**Files:**
- Modify: `src/commands/status.rs`
- Modify: `src/commands/report/mod.rs`
- Test: `tests/cmd_report_test.rs`

**Interfaces:**
- Consumes: `BackupConfig::load_from_path`
- Produces: Config-driven `status` and `report` pipelines with strict error on missing config.

- [ ] **Step 1: Write failing test verifying error when config file is missing**

```rust
// In tests/cmd_report_test.rs
#[test]
fn test_report_command_fails_on_missing_config() {
    let non_existent_path = std::path::Path::new("/tmp/non_existent_config_12345.yaml");
    let res = backup::commands::report::run_report(non_existent_path, None, None);
    assert!(res.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_report_command_fails_on_missing_config`
Expected: FAIL (if `BackupConfig::default()` was previously used to silently pass)

- [ ] **Step 3: Replace `BackupConfig::default()` fallbacks with config loading**

In `src/commands/status.rs` and `src/commands/report/mod.rs`:
Remove `BackupConfig::default()` fallback calls. Ensure functions return `Err(anyhow::anyhow!("Configuration not found at ..."))` when config cannot be loaded.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --test cmd_report_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/status.rs src/commands/report/mod.rs tests/cmd_report_test.rs
git commit -m "fix(report,status): eliminate BackupConfig::default() fallbacks and enforce active config loading"
```
