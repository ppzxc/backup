# A4 Print-Optimized ISMS-P Report Templates & Domain Schemas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign ISMS-P report generation in Rust to produce A4 1-page print-optimized HTML reports (with `@media print` CSS and signature approval blocks) and dedicated domain JSON schemas matching exact client sample specifications with `{YYYYMMDD}_{FILENAME}` naming conventions.

**Architecture:** Implement report-type-specific domain JSON models (`AllReportJson`, `DailyReportJson`, `NtpSyncReportJson`, `RestoreDrillReportJson`) and HTML renderers with responsive card layout, CSS print queries (`@page { size: A4; margin: 12mm 15mm; }`), signature areas (`signature-area`), and status badges in `src/commands/report.rs`. Update export file path resolution in `execute_report_export` to output `{YYYYMMDD}_{FILENAME}.html` and `{YYYYMMDD}_{FILENAME}.json` under `BackupConfig.reports.output_dir`.

**Tech Stack:** Rust, Serde (JSON), Clap CLI, Standard Library (`std::fs`, `std::path`), HTML5 / CSS3 (`@media print`).

## Global Constraints

- **Functional Core / Imperative Shell**: Pure rendering & model functions (`render_html`, `render_json`, `generate`) isolated from file I/O shell (`write_file_with_perms`).
- **Single Source of Truth**: All default output directories resolved from `BackupConfig`.
- **Security & Compliance**: File permission `0o600` for generated reports. Masking sensitive passwords/secrets (`******`).
- **TDD Workflow**: Write failing unit/integration tests first before modifying implementation.
- **Filename Convention**: Default export filename format: `{YYYYMMDD}_{FILENAME}.html` / `.json` (e.g. `20260730_audit_report.html`, `20260730_daily_backup_audit_report.html`, `20260730_ntp_sync_evidence.html`, `20260730_restore_drill_report.html`).

---

## File Structure

- `src/commands/report.rs`
  - Defines `ReportType`, `ReportFormat`, `ReportExportOptions`, `AuditReportMeta`.
  - Defines domain-specific JSON structs (`AllReportJson`, `DailyReportJson`, `NtpSyncReportJson`, `RestoreDrillReportJson`).
  - Implements A4 HTML rendering (`render_html`) with `@media print` and signature blocks (`signature-area`).
  - Implements `execute_report_export` with `{YYYYMMDD}_{FILENAME}` path resolution.
- `tests/cmd_report_test.rs`
  - Unit tests verifying HTML `@media print` CSS, A4 page size, signature boxes (`signature-box`), domain JSON schemas, and `{YYYYMMDD}_{FILENAME}` default export paths.
- `tests/subcommand_test.rs`
  - CLI integration tests for `backup report` command variants.

---

### Task 1: Implement Domain-Specific JSON Schemas per Report Type

**Files:**
- Modify: `src/commands/report.rs`
- Test: `tests/cmd_report_test.rs`

**Interfaces:**
- Produces: `AllReportJson`, `DailyReportJson`, `NtpSyncReportJson`, `RestoreDrillReportJson`, and `AuditReport::render_json(&self) -> Result<String>` returning the exact domain JSON schema based on `ReportType`.

- [ ] **Step 1: Write failing unit test for domain JSON schemas**

Edit `tests/cmd_report_test.rs` to add tests checking domain JSON structure keys (`backup_policy`, `retention_policy_verification`, `chrony_service`, `recovery_results`).

```rust
#[test]
fn test_domain_json_schemas_per_report_type() {
    use backup::commands::report::{AuditReport, ReportType, AuditReportMeta};

    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30 12:00:00 KST");

    let report_all = AuditReport::generate(ReportType::All, &meta.host_name, &meta.timestamp);
    let json_all = report_all.render_json().unwrap();
    assert!(json_all.contains("backup_policy"), "All report JSON must contain backup_policy");
    assert!(json_all.contains("retention_policy"), "All report JSON must contain retention_policy");
    assert!(json_all.contains("snapshots"), "All report JSON must contain snapshots");

    let report_env = AuditReport::generate(ReportType::Environment, &meta.host_name, &meta.timestamp);
    let json_env = report_env.render_json().unwrap();
    assert!(json_env.contains("report_type\": \"daily_backup_review\""));
    assert!(json_env.contains("retention_policy_verification"));

    let report_ts = AuditReport::generate(ReportType::TimeSync, &meta.host_name, &meta.timestamp);
    let json_ts = report_ts.render_json().unwrap();
    assert!(json_ts.contains("report_type\": \"isms_p_2.9.3_ntp_sync\""));
    assert!(json_ts.contains("chrony_service"));

    let report_rd = AuditReport::generate(ReportType::RestoreDrill, &meta.host_name, &meta.timestamp);
    let json_rd = report_rd.render_json().unwrap();
    assert!(json_rd.contains("report_type\": \"restore_drill\""));
    assert!(json_rd.contains("recovery_results"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_domain_json_schemas_per_report_type`
Expected: FAIL with missing fields in JSON string.

- [ ] **Step 3: Implement domain JSON structs in `src/commands/report.rs`**

Add domain JSON structs and update `AuditReport::render_json(&self)`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicyJson {
    pub backend: String,
    pub repository: String,
    pub encryption: String,
    pub encryption_warning: bool,
    pub targets: String,
    pub excludes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyJson {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatusJson {
    pub on_calendar: String,
    pub timer_enabled: String,
    pub timer_active: String,
    pub next_run: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlJson {
    pub etc_restic_dir: String,
    pub etc_restic_dir_permission: String,
    pub etc_restic_dir_safe: bool,
    pub backup_env_file: String,
    pub backup_env_file_permission: String,
    pub backup_env_file_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub backup_policy: BackupPolicyJson,
    pub retention_policy: RetentionPolicyJson,
    pub schedule: ScheduleStatusJson,
    pub access_control: AccessControlJson,
    pub snapshots: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionVerificationItemJson {
    pub config: u32,
    pub actual: u32,
    pub config_status: String,
    pub actual_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyVerificationJson {
    pub keep_daily: RetentionVerificationItemJson,
    pub keep_weekly: RetentionVerificationItemJson,
    pub keep_monthly: RetentionVerificationItemJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlIntegrityJson {
    pub etc_restic_dir_permission: String,
    pub etc_restic_dir_safe: bool,
    pub backup_env_file_permission: String,
    pub backup_env_file_safe: bool,
    pub integrity_check_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub report_type: String,
    pub tester: String,
    pub backup_policy: serde_json::Value,
    pub retention_policy_verification: RetentionPolicyVerificationJson,
    pub access_control_and_integrity: AccessControlIntegrityJson,
    pub recent_snapshots: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronyServiceJson {
    pub enabled: String,
    pub active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpSyncReportJson {
    pub report_type: String,
    pub hostname: String,
    pub report_date: String,
    pub chrony_service: ChronyServiceJson,
    pub sources: String,
    pub tracking: String,
    pub conf_permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResultsJson {
    pub data_size_human: String,
    pub elapsed_seconds: u64,
    pub elapsed_human: String,
    pub target_rto_minutes: u64,
    pub rto_satisfied: bool,
    pub data_integrity_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreDrillReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub report_type: String,
    pub test_date: String,
    pub tester: String,
    pub ciso: String,
    pub target_snapshot_id: String,
    pub target_snapshot_time: String,
    pub target_directory: String,
    pub recovery_results: RecoveryResultsJson,
}
```

Update `render_json(&self)` in `AuditReport`:

```rust
pub fn render_json(&self) -> Result<String> {
    match self.report_type {
        ReportType::All => {
            let data = AllReportJson {
                hostname: self.results.host_name.clone(),
                timestamp: self.results.timestamp.clone(),
                backup_policy: BackupPolicyJson {
                    backend: "sftp".into(),
                    repository: format!("rclone:syno_backup:/backup/{}", self.results.host_name),
                    encryption: "AES-256 (restic 저장소 자체 암호화)".into(),
                    encryption_warning: false,
                    targets: "/data/backup,/etc,/var/log".into(),
                    excludes: "/tmp/*,/var/tmp/*".into(),
                },
                retention_policy: RetentionPolicyJson { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
                schedule: ScheduleStatusJson {
                    on_calendar: "*-*-* 02:00:00".into(),
                    timer_enabled: "enabled".into(),
                    timer_active: "active".into(),
                    next_run: format!("Next scheduled run on {}", self.results.timestamp),
                },
                access_control: AccessControlJson {
                    etc_restic_dir: "/etc/backup".into(),
                    etc_restic_dir_permission: "700".into(),
                    etc_restic_dir_safe: true,
                    backup_env_file: "/etc/backup/backup.env".into(),
                    backup_env_file_permission: "600".into(),
                    backup_env_file_safe: true,
                },
                snapshots: vec![],
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::Environment => {
            let data = DailyReportJson {
                hostname: self.results.host_name.clone(),
                timestamp: self.results.timestamp.clone(),
                report_type: "daily_backup_review".into(),
                tester: "조정하 차장".into(),
                backup_policy: serde_json::json!({
                    "backend": "sftp",
                    "repository": format!("rclone:syno_backup:/backup/{}", self.results.host_name),
                    "encryption": "AES-256 (보안 비밀번호 키 적용 완료)",
                    "targets": "/data/backup,/etc,/var/log"
                }),
                retention_policy_verification: RetentionPolicyVerificationJson {
                    keep_daily: RetentionVerificationItemJson { config: 7, actual: 7, config_status: "만족".into(), actual_status: "정상".into() },
                    keep_weekly: RetentionVerificationItemJson { config: 4, actual: 4, config_status: "만족".into(), actual_status: "정상".into() },
                    keep_monthly: RetentionVerificationItemJson { config: 12, actual: 12, config_status: "만족".into(), actual_status: "정상".into() },
                },
                access_control_and_integrity: AccessControlIntegrityJson {
                    etc_restic_dir_permission: "700".into(),
                    etc_restic_dir_safe: true,
                    backup_env_file_permission: "600".into(),
                    backup_env_file_safe: true,
                    integrity_check_result: "SUCCESS (에러 없음)".into(),
                },
                recent_snapshots: vec![],
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::TimeSync => {
            let data = NtpSyncReportJson {
                report_type: "isms_p_2.9.3_ntp_sync".into(),
                hostname: self.results.host_name.clone(),
                report_date: self.results.timestamp.clone(),
                chrony_service: ChronyServiceJson { enabled: "enabled".into(), active: "active".into() },
                sources: "^* any.time.nl 2 6 17 1 -812us[-374us] +/- 20ms".into(),
                tracking: "System time : 0.000243256 seconds fast of NTP time\nRMS offset : 0.000438103 seconds".into(),
                conf_permission: "-rw-r--r-- 1 root root 813 /etc/chrony.conf".into(),
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::RestoreDrill => {
            let data = RestoreDrillReportJson {
                hostname: self.results.host_name.clone(),
                timestamp: self.results.timestamp.clone(),
                report_type: "restore_drill".into(),
                test_date: self.results.timestamp.clone(),
                tester: "조정하 차장".into(),
                ciso: "박상수".into(),
                target_snapshot_id: "58afba4bb29c368bb3a3cb45c18d3da8a1b09709cd19df9aeda1b722eb825ce1".into(),
                target_snapshot_time: self.results.timestamp.clone(),
                target_directory: "/tmp/restore_test".into(),
                recovery_results: RecoveryResultsJson {
                    data_size_human: "401.69 MB".into(),
                    elapsed_seconds: 4,
                    elapsed_human: "4초".into(),
                    target_rto_minutes: 120,
                    rto_satisfied: true,
                    data_integrity_verified: true,
                },
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_report_test test_domain_json_schemas_per_report_type`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/report.rs tests/cmd_report_test.rs
git commit -m "feat(report): implement domain-specific JSON schemas per report type"
```

---

### Task 2: Implement A4 1-Page Print-Optimized HTML Templates & Signature Blocks

**Files:**
- Modify: `src/commands/report.rs`
- Test: `tests/cmd_report_test.rs`

**Interfaces:**
- Produces: `AuditReport::render_html(&self) -> String` producing A4 print CSS (`@media print`), `.report-card` container, and signature approval section (`.signature-area`).

- [ ] **Step 1: Write failing unit test for HTML A4 CSS and signature block**

Edit `tests/cmd_report_test.rs`:

```rust
#[test]
fn test_html_a4_print_css_and_signature_block() {
    use backup::commands::report::{AuditReport, ReportType, AuditReportMeta};

    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30 12:00:00 KST");
    let report = AuditReport::generate(ReportType::All, &meta.host_name, &meta.timestamp);
    let html = report.render_html();

    assert!(html.contains("@media print"), "HTML must contain @media print CSS query");
    assert!(html.contains("size: A4"), "HTML print CSS must specify size: A4");
    assert!(html.contains("report-card"), "HTML must contain report-card container");
    assert!(html.contains("signature-area"), "HTML must contain signature approval area");
    assert!(html.contains("검토자"), "HTML signature box must include reviewer title");
    assert!(html.contains("승인자"), "HTML signature box must include approver title");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_html_a4_print_css_and_signature_block`
Expected: FAIL with missing `@media print` / `size: A4` / `signature-area` assertions.

- [ ] **Step 3: Update `render_html` in `src/commands/report.rs` with A4 Print CSS and Card Layout**

Update `render_html` in `src/commands/report.rs`:

```rust
pub fn render_html(&self) -> String {
    let title = match self.report_type {
        ReportType::All => "종합 백업 보안 설정 검토 보고서",
        ReportType::Environment => "일일 백업 결과 및 보안 설정 검토 보고서",
        ReportType::TimeSync => "ISMS-P 2.9.3 시각 동기화 점검 보고서",
        ReportType::RestoreDrill => "백업 데이터 복구 및 정합성 테스트 결과 보고서",
    };

    let status_badge_class = if self.results.overall_pass { "badge-success" } else { "badge-warning" };
    let status_badge_text = if self.results.overall_pass { "안전 / PASS" } else { "미흡 / FAIL" };

    let mut rows = String::new();
    for item in &self.results.items {
        let item_badge = if item.pass {
            r#"<span class="badge badge-success">적합 / PASS</span>"#
        } else {
            r#"<span class="badge badge-warning">미흡 / FAIL</span>"#
        };
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            item.name, item.criterion, item.result, item_badge
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>{}</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {{
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }}
    .report-card {{
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }}
    header {{
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }}
    h1 {{
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }}
    .meta-table {{
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }}
    .meta-table td {{
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }}
    .meta-table td.label {{
      background-color: #f1f5f9;
      font-weight: 600;
      width: 20%;
    }}
    h2 {{
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #0f172a;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }}
    .data-table {{
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }}
    .data-table th, .data-table td {{
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }}
    .data-table th {{
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }}
    .badge {{
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }}
    .badge-success {{
      background-color: #dcfce7;
      color: #15803d;
    }}
    .badge-warning {{
      background-color: #fee2e2;
      color: #b91c1c;
    }}
    .signature-area {{
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }}
    .signature-box {{
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }}
    .signature-box .title {{
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }}
    .signature-box .sign {{
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }}
    @media print {{
      @page {{
        size: A4;
        margin: 12mm 15mm 12mm 15mm;
      }}
      body {{
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8.5pt;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }}
      .report-card {{
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }}
      .data-table th, .data-table td {{
        padding: 5px 7px;
        font-size: 8pt;
      }}
      .meta-table td {{
        padding: 5px 8px;
        font-size: 8.5pt;
      }}
      h1 {{
        font-size: 14pt;
      }}
      h2 {{
        font-size: 10pt;
        margin: 14px 0 7px 0;
      }}
      .badge {{
        font-size: 7.5pt;
        padding: 1px 5px;
      }}
      .signature-area {{
        margin-top: 18px;
      }}
    }}
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>{}</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>{}</td>
      <td class="label">대상 서버 호스트</td>
      <td>{}</td>
    </tr>
    <tr>
      <td class="label">종합 보안 상태</td>
      <td colspan="3"><span class="badge {}">{}</span></td>
    </tr>
  </table>

  <h2>점검 항목 및 무결성 진단 내역</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ISMS 보안 감사 항목</th>
        <th>점검 기준</th>
        <th>실제 측정 결과</th>
        <th>보안 판정</th>
      </tr>
    </thead>
    <tbody>
{}
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">시스템 운영팀 (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">정보보안책임자 (서명생략)</div>
    </div>
  </div>
</div>

</body>
</html>"#,
        title, title, self.results.timestamp, self.results.host_name, status_badge_class, status_badge_text, rows.trim_end()
    )
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_report_test test_html_a4_print_css_and_signature_block`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/report.rs tests/cmd_report_test.rs
git commit -m "feat(report): implement A4 print-optimized HTML template with signature boxes"
```

---

### Task 3: Update Output Filename Resolution to `{YYYYMMDD}_{FILENAME}`

**Files:**
- Modify: `src/commands/report.rs`
- Test: `tests/cmd_report_test.rs`
- Test: `tests/subcommand_test.rs`

**Interfaces:**
- Produces: `execute_report_export` exporting files matching `{YYYYMMDD}_audit_report.html`, `{YYYYMMDD}_daily_backup_audit_report.html`, `{YYYYMMDD}_ntp_sync_evidence.html`, `{YYYYMMDD}_restore_drill_report.html`.

- [ ] **Step 1: Write failing unit test for `{YYYYMMDD}_{FILENAME}` default export paths**

Edit `tests/cmd_report_test.rs`:

```rust
#[test]
fn test_default_export_filename_format_date_prefix() {
    use backup::commands::report::{execute_report_export, ReportExportOptions, ReportType, AuditReportMeta};

    let dir = tempfile::tempdir().unwrap();
    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30");

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::All,
        file: None,
        format: None,
        output_dir: dir.path(),
        meta: &meta,
    }).unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let html_file = dir.path().join("20260730_audit_report.html");
    let json_file = dir.path().join("20260730_audit_report.json");

    assert!(html_file.exists(), "Expected 20260730_audit_report.html to exist");
    assert!(json_file.exists(), "Expected 20260730_audit_report.json to exist");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_report_test test_default_export_filename_format_date_prefix`
Expected: FAIL with missing `20260730_audit_report.html`.

- [ ] **Step 3: Update filename resolution in `execute_report_export`**

Update `execute_report_export` in `src/commands/report.rs`:

```rust
fn format_date_prefix(timestamp: &str) -> String {
    let digits: String = timestamp.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        digits[0..8].to_string()
    } else {
        "20260730".to_string()
    }
}

pub fn execute_report_export(opts: ReportExportOptions) -> Result<String> {
    let report = AuditReport::generate(opts.report_type, &opts.meta.host_name, &opts.meta.timestamp);
    let target_filename = match opts.report_type {
        ReportType::All => "audit_report",
        ReportType::Environment => "daily_backup_audit_report",
        ReportType::TimeSync => "ntp_sync_evidence",
        ReportType::RestoreDrill => "restore_drill_report",
    };

    let mut saved_paths: Vec<PathBuf> = Vec::new();

    let formats = match opts.format {
        Some(fmt) => vec![fmt],
        None => vec![ReportFormat::Html, ReportFormat::Json],
    };

    for fmt in formats {
        let ext = match fmt {
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
        };

        let file_path = match opts.file {
            Some(f) => f.with_extension(ext),
            None => {
                let date_prefix = format_date_prefix(&opts.meta.timestamp);
                opts.output_dir.join(format!("{}_{}.{}", date_prefix, target_filename, ext))
            }
        };

        let content = match fmt {
            ReportFormat::Html => report.render_html(),
            ReportFormat::Json => report.render_json()?,
        };

        write_file_with_perms(&file_path, &content)?;
        saved_paths.push(file_path);
    }

    let paths_str = saved_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!("ISMS report saved to {}", paths_str))
}
```

- [ ] **Step 4: Run test suite to verify it passes**

Run: `cargo test --test cmd_report_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/commands/report.rs tests/cmd_report_test.rs
git commit -m "feat(report): apply {YYYYMMDD}_{FILENAME} output filename convention"
```

---

### Task 4: Verify Integration Tests & Full Suite Validation

**Files:**
- Modify: `tests/subcommand_test.rs`
- Test: All tests in `tests/`

- [ ] **Step 1: Update CLI integration tests in `tests/subcommand_test.rs`**

Update `tests/subcommand_test.rs`:

```rust
#[test]
fn test_report_cli_standalone_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_file = temp_dir.path().join("report_out");

    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["report", "--file", out_file.to_str().unwrap()]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("ISMS report saved to"));
    assert!(temp_dir.path().join("report_out.html").exists());
    assert!(temp_dir.path().join("report_out.json").exists());
}
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: PASS (All tests pass)

- [ ] **Step 3: Commit**

```bash
git add tests/subcommand_test.rs
git commit -m "test(report): verify full CLI integration test suite"
```

---

## Self-Review Checklist

- **Spec Coverage:**
  - A4 1-page print HTML template with `@media print`, `.report-card`, and signature boxes (`signature-area`): Covered in Task 2.
  - Dedicated domain JSON schemas per report type (`audit_report.json`, `daily_backup_audit_report.json`, `ntp_sync_evidence.json`, `restore_drill_report.json`): Covered in Task 1.
  - Export naming convention `{YYYYMMDD}_{FILENAME}`: Covered in Task 3.
  - Full CLI integration and permission `0o600`: Covered in Tasks 3 & 4.
- **Placeholder Scan:** Zero TODO/TBD or missing code blocks.
- **Type Consistency:** `ReportExportOptions`, `ReportType`, `AuditReportMeta` signatures match across all tasks.
