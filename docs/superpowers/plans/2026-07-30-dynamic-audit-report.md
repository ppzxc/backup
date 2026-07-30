# Dynamic Audit Report Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate dynamic system manager and security officer configuration into `profiles.yaml`, prompt for them during `backup setup`, collect server OS release info dynamically, and substitute all dynamic fields into HTML and JSON backup security reports.

**Architecture:** Extend `ResticProfileConfig` with an `audit` section (`AuditConfig`), update `backup setup` prompter with i18n prompts, collect OS release details in `RealReportData::collect_with_meta`, and dynamically bind these values across all report generators (`html_template.rs` and `json_schema.rs`).

**Tech Stack:** Rust, serde / serde_yaml, inquire (setup prompter), clap, std::fs / std::process.

## Global Constraints

- Preserve `resticprofile` schema compatibility (put custom metadata in top-level `audit:` key in `profiles.yaml`).
- Enforce strict POSIX permissions (`700` for `/etc/backup`, `600` for config files).
- Multi-language support (Korean & English) for interactive prompts.
- Test-driven development (TDD) for all config changes, setup flow, and report rendering.

---

### Task 1: Add `AuditConfig` to Configuration Model

**Files:**
- Modify: `src/config/model.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Consumes: `serde::Deserialize`, `serde::Serialize`
- Produces: `AuditConfig` struct, `ResticProfileConfig.audit: Option<AuditConfig>`

- [ ] **Step 1: Write failing test in `tests/config_test.rs`**

```rust
#[test]
fn test_restic_profile_config_audit_section() {
    let yaml = r#"
version: "2"
audit:
  system_manager: "홍길동 차장"
  security_officer: "김보안 이사"
global:
  min-memory: 1024
profiles: {}
"#;
    let config: ResticProfileConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.audit.is_some());
    let audit = config.audit.unwrap();
    assert_eq!(audit.system_manager, Some("홍길동 차장".to_string()));
    assert_eq!(audit.security_officer, Some("김보안 이사".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test test_restic_profile_config_audit_section`
Expected: FAIL (unknown field `audit` or struct missing)

- [ ] **Step 3: Implement `AuditConfig` in `src/config/model.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AuditConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_officer: Option<String>,
}

// In ResticProfileConfig:
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResticProfileConfig {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
    // ... remaining fields unchanged
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test config_test test_restic_profile_config_audit_section`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config/model.rs tests/config_test.rs
git commit -m "feat(config): add AuditConfig section to ResticProfileConfig"
```

---

### Task 2: Add Setup i18n Messages & Prompts for Audit Roles

**Files:**
- Modify: `src/i18n.rs`
- Modify: `src/commands/setup.rs`
- Test: `tests/cmd_setup_test.rs`

**Interfaces:**
- Consumes: `I18nMessages`, `SetupPrompter`
- Produces: `SetupParams.audit`, interactive prompts for `system_manager` & `security_officer`

- [ ] **Step 1: Write failing test in `tests/cmd_setup_test.rs`**

```rust
#[test]
fn test_setup_params_includes_audit_config() {
    let params = SetupParams {
        profile: "default".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::default(),
        primary_storage: StorageTarget {
            backend: StorageBackend::Local,
            repository: "/tmp/repo".into(),
            password: SecretString::new("password1234".into()),
            s3: None,
            sftp: None,
        },
        secondary_storage: None,
        reports: ReportsConfig::default(),
        audit: AuditConfig {
            system_manager: Some("홍길동".into()),
            security_officer: Some("김보안".into()),
        },
    };
    assert_eq!(params.audit.system_manager.unwrap(), "홍길동");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_setup_test test_setup_params_includes_audit_config`
Expected: FAIL (field `audit` missing on `SetupParams`)

- [ ] **Step 3: Update `src/i18n.rs` and `src/commands/setup.rs`**

In `src/i18n.rs`:
Add `prompt_system_manager` and `prompt_security_officer` to `I18nMessages`.
Korean defaults:
- `prompt_system_manager`: `"시스템 운영/백업 담당자 이름 (검토자):"`
- `prompt_security_officer`: `"정보보안 책임자 이름 (승인자):"`

In `src/commands/setup.rs`:
1. Add `pub audit: AuditConfig` to `SetupParams`.
2. In `InquirePrompter::prompt_setup_params`:
   ```rust
   let sys_mgr = prompt_text_with_default(msg.prompt_system_manager, "시스템 운영팀", lang)?;
   let sec_off = prompt_text_with_default(msg.prompt_security_officer, "정보보안책임자", lang)?;
   let audit = AuditConfig {
       system_manager: Some(sys_mgr),
       security_officer: Some(sec_off),
   };
   ```
3. Update `save_and_sync` to write `config.audit = Some(params.audit)` into `profiles.yaml`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_setup_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/i18n.rs src/commands/setup.rs tests/cmd_setup_test.rs
git commit -m "feat(setup): prompt system manager and security officer names during setup"
```

---

### Task 3: Dynamic OS Release Data & Audit Data Collection in Reports

**Files:**
- Modify: `src/commands/report/mod.rs`
- Test: `tests/cmd_report_test.rs`

**Interfaces:**
- Consumes: `/etc/os-release` / `uname`, `ResticProfileConfig.audit`
- Produces: `RealReportData.audit`, `RealReportData.os_info`

- [ ] **Step 1: Write failing test in `tests/cmd_report_test.rs`**

```rust
#[test]
fn test_real_report_data_collects_os_and_audit() {
    let config = crate::config::model::BackupConfig::default();
    let data = crate::commands::report::RealReportData::collect(&config);
    assert!(!data.os_info.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_real_report_data_collects_os_and_audit`
Expected: FAIL (`os_info` field missing)

- [ ] **Step 3: Implement OS detection and audit loading in `src/commands/report/mod.rs`**

1. Add helper function `collect_os_info() -> String`:
   Reads `/etc/os-release` (LINE `PRETTY_NAME="..."` or `NAME="..."`) or falls back to `uname -sr`.
2. Add `pub audit: AuditConfig` and `pub os_info: String` to `RealReportData`.
3. In `RealReportData::collect_with_meta`:
   - Load `profiles.yaml` if available to get `audit` config (fallback to default `"시스템 운영팀"` / `"정보보안책임자"`).
   - Set `os_info` using `collect_os_info()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_report_test test_real_report_data_collects_os_and_audit`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/report/mod.rs tests/cmd_report_test.rs
git commit -m "feat(report): collect OS release info and audit metadata for reports"
```

---

### Task 4: Dynamic Template Rendering (HTML & JSON)

**Files:**
- Modify: `src/commands/report/html_template.rs`
- Modify: `src/commands/report/json_schema.rs`
- Test: `tests/cmd_report_test.rs`

**Interfaces:**
- Consumes: `RealReportData`
- Produces: Dynamic HTML and JSON report strings containing configured manager names & OS info

- [ ] **Step 1: Write failing test in `tests/cmd_report_test.rs`**

```rust
#[test]
fn test_html_report_contains_custom_audit_names_and_os() {
    let mut data = crate::commands::report::RealReportData::default_for_test();
    data.audit.system_manager = Some("홍길동 차장".into());
    data.audit.security_officer = Some("김보안 이사".into());
    data.os_info = "Ubuntu 22.04 LTS".into();

    let html = crate::commands/report::html_template::render_html_real(
        crate::commands/report::ReportType::RestoreDrill,
        &data,
    );

    assert!(html.contains("홍길동 차장"));
    assert!(html.contains("김보안 이사"));
    assert!(html.contains("Ubuntu 22.04 LTS"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_html_report_contains_custom_audit_names_and_os`
Expected: FAIL (hardcoded strings still in template)

- [ ] **Step 3: Update `html_template.rs` and `json_schema.rs`**

1. In `html_template.rs`:
   - Replace hardcoded `"조정하 차장"` with `data.audit.system_manager.as_deref().unwrap_or("시스템 운영팀")`.
   - Replace hardcoded `"박상수 (인)"` / `"정보보안책임자 (서명생략)"` with `data.audit.security_officer.as_deref().unwrap_or("정보보안책임자")`.
   - Replace hardcoded `"Rocky Linux 9.8 (Blue Onyx)"` in `render_restore_drill_html` with `data.os_info`.
2. In `json_schema.rs`:
   - Include `system_manager`, `security_officer`, and `os_info` in JSON schemas.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_report_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/report/html_template.rs src/commands/report/json_schema.rs tests/cmd_report_test.rs
git commit -m "feat(report): dynamically render audit names and OS info in HTML/JSON reports"
```

---

### Task 5: Integration & Verification

**Files:**
- Test: `tests/integration_scenario.rs`

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: PASS (All unit and integration tests pass cleanly)

- [ ] **Step 2: Commit final changes**

```bash
git commit --allow-empty -m "chore: completed dynamic audit report metadata implementation"
```
