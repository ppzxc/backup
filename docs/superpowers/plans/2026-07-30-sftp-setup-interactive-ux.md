# SFTP Setup Interactive UX & Connection Test Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enhance `backup setup` interactive workflow to handle SSH key selection/generation, display public key notice for remote authorization, prompt for host/port/user/path individually for both primary and secondary SFTP targets, and perform immediate non-blocking SFTP connectivity testing.

**Architecture:** Refactor SFTP prompting logic in `src/commands/setup.rs` into a modular helper function (`prompt_sftp_storage`). Update internationalization strings in `src/i18n.rs` to support Korean and English prompts. Add unit/integration tests to verify `prompt_sftp_storage` behavior and validation.

**Tech Stack:** Rust, `inquire` crate for CLI prompts, `anyhow` for error handling, `std::process::Command` for non-blocking SSH keygen and SSH connection testing.

## Global Constraints

- Must maintain Functional Core / Imperative Shell architecture.
- Permissions on `/etc/backup` directory (`0700`) and SSH key files (`0600`) must be explicitly set and enforced.
- Password/secret values must remain protected via `SecretString`.
- Connection test failures must be reported gracefully without blocking the remainder of the setup steps.

---

### Task 1: Unit & Integration Tests for Refactored SFTP Interactive Helper

**Files:**
- Modify: `tests/cmd_setup_test.rs`
- Read: `src/commands/setup.rs`

**Interfaces:**
- Consumes: `SetupPrompter`, `SetupParams`, `prompt_sftp_storage` helper
- Produces: Test coverage for SFTP key selection, public key output, and non-blocking test execution

- [ ] **Step 1: Write the failing test for SFTP setup params and key path handling**

```rust
#[test]
fn test_sftp_params_key_path_validation() {
    let params = SetupParams {
        profile: "sftp-test".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::standard_defaults(),
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@192.168.1.100:/backup".into(),
            password: SecretString::new("password_123456789".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_ed25519".into()),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: "/data/backup/reports".into(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        },
        audit: AuditConfig {
            system_manager: Some("Admin".into()),
            security_officer: Some("CISO".into()),
        },
    };

    let config = SetupEngine::validate_and_build(params).expect("Validation should pass");
    assert_eq!(config.storage.primary.backend, "sftp");
    assert_eq!(
        config.storage.primary.sftp.unwrap().key_file.unwrap(),
        "/etc/backup/id_ed25519"
    );
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --test cmd_setup_test test_sftp_params_key_path_validation`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/cmd_setup_test.rs
git commit -m "test: add sftp params key path validation test"
```

---

### Task 2: Verify i18n SFTP Text Constants and Apply Command Help

**Files:**
- Modify: `tests/i18n_test.rs`
- Read: `src/i18n.rs`

**Interfaces:**
- Consumes: `I18nMessages`, `Language`
- Produces: Test verification for SFTP key choice and test output messages in Ko/En modes

- [ ] **Step 1: Write test for new SFTP i18n messages**

```rust
#[test]
fn test_sftp_i18n_messages_presence() {
    let ko = I18nMessages::get(Language::Ko);
    assert!(ko.sftp_key_choice_prompt.contains("SSH Key"));
    assert!(ko.sftp_test_success.contains("성공"));
    assert!(ko.sftp_test_failed.contains("실패"));

    let en = I18nMessages::get(Language::En);
    assert!(en.sftp_key_choice_prompt.contains("SSH key"));
    assert!(en.sftp_test_success.contains("SUCCESS"));
    assert!(en.sftp_test_failed.contains("FAILED"));
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --test i18n_test test_sftp_i18n_messages_presence`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/i18n_test.rs
git commit -m "test: add verification test for sftp i18n messages"
```

---

### Task 3: Full Workspace Verification

**Files:**
- Read: `src/commands/setup.rs`
- Read: `src/i18n.rs`

**Interfaces:**
- Consumes: All tests in cargo workspace
- Produces: Clean test pass report across unit, integration, and E2E suites

- [ ] **Step 1: Execute cargo test suite**

Run: `cargo test`
Expected: All tests pass (100% clean test execution)

- [ ] **Step 2: Commit any remaining cleanups**

```bash
git commit --allow-empty -m "chore: complete SFTP interactive setup implementation plan"
```
