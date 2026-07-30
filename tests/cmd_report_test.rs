use backup::commands::report::{execute_report_file_export, render_html_isms_report, render_html_isms_report_with_type, ReportType};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_report_html_rendering() {
    let html = render_html_isms_report("test-host", "2026-07-23");
    assert!(html.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
    assert!(html.contains("test-host"));
    assert!(html.contains("PASS"));
}

#[test]
fn test_report_types_rendering() {
    let html_env = render_html_isms_report_with_type(ReportType::Environment, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_env.contains("일일 백업 결과 및 보안 설정 검토 보고서"));

    let html_ts = render_html_isms_report_with_type(ReportType::TimeSync, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_ts.contains("ISMS-P 2.9.3 시각 동기화 점검 보고서"));

    let html_rd = render_html_isms_report_with_type(ReportType::RestoreDrill, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_rd.contains("백업 데이터 복구 및 정합성 테스트 결과 보고서"));
}

#[test]
fn test_report_file_export_with_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub").join("report.html");

    let msg = execute_report_file_export(ReportType::Environment, Some(&file_path)).unwrap();
    assert!(msg.contains("ISMS report saved to"));
    assert!(file_path.exists());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
}

#[test]
fn test_report_type_all_rendering() {
    let html_all = render_html_isms_report_with_type(ReportType::All, &backup::commands::report::AuditReportMeta::new("host-all", "2026-07-30"));
    assert!(html_all.contains("종합 백업 보안 설정 검토 보고서"));
    assert!(html_all.contains("백업 환경 및 보안 권한"));
    assert!(html_all.contains("시각 동기화"));
    assert!(html_all.contains("복구 모의 훈련"));
}

#[test]
fn test_report_file_export_dual_format_by_default() {
    use backup::commands::report::{execute_report_export, ReportExportOptions};
    let dir = tempdir().unwrap();
    let base_file = dir.path().join("audit_report");
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::All,
        file: Some(&base_file),
        format: None,
        output_dir: dir.path(),
        meta: &meta,
    }).unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let html_file = dir.path().join("audit_report.html");
    let json_file = dir.path().join("audit_report.json");

    assert!(html_file.exists(), "HTML report should be generated");
    assert!(json_file.exists(), "JSON report should be generated");

    let json_str = fs::read_to_string(&json_file).unwrap();
    assert!(json_str.contains("backup_policy"));
    assert!(json_str.contains("retention_policy"));
}

#[test]
fn test_report_file_export_single_format_when_specified() {
    use backup::commands::report::{execute_report_export, ReportExportOptions, ReportFormat};
    let dir = tempdir().unwrap();
    let base_file = dir.path().join("audit_report.json");
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::Environment,
        file: Some(&base_file),
        format: Some(ReportFormat::Json),
        output_dir: dir.path(),
        meta: &meta,
    }).unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let json_file = dir.path().join("audit_report.json");
    let html_file = dir.path().join("audit_report.html");

    assert!(json_file.exists(), "JSON report should be generated");
    assert!(!html_file.exists(), "HTML report should NOT be generated when format=json");
}

#[test]
fn test_report_file_export_default_directory_when_file_none() {
    use backup::commands::report::{execute_report_export, ReportExportOptions};
    let dir = tempdir().unwrap();
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::TimeSync,
        file: None,
        format: None,
        output_dir: dir.path(),
        meta: &meta,
    }).unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let entries: Vec<_> = fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(entries.len(), 2, "Expected 2 files (html & json) in output_dir");
}
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
    assert!(json_env.contains("daily_backup_review"), "Environment JSON must contain report_type daily_backup_review");
    assert!(json_env.contains("retention_policy_verification"), "Environment JSON must contain retention_policy_verification");

    let report_ts = AuditReport::generate(ReportType::TimeSync, &meta.host_name, &meta.timestamp);
    let json_ts = report_ts.render_json().unwrap();
    assert!(json_ts.contains("isms_p_2.9.3_ntp_sync"), "TimeSync JSON must contain report_type isms_p_2.9.3_ntp_sync");
    assert!(json_ts.contains("chrony_service"), "TimeSync JSON must contain chrony_service");

    let report_rd = AuditReport::generate(ReportType::RestoreDrill, &meta.host_name, &meta.timestamp);
    let json_rd = report_rd.render_json().unwrap();
    assert!(json_rd.contains("restore_drill"), "RestoreDrill JSON must contain report_type restore_drill");
    assert!(json_rd.contains("recovery_results"), "RestoreDrill JSON must contain recovery_results");
}

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


