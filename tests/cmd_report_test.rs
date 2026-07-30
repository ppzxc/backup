use backup::commands::report::{execute_report_file_export, render_html_isms_report, render_html_isms_report_with_type, ReportType};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_report_html_rendering() {
    let html = render_html_isms_report("test-host", "2026-07-23");
    assert!(html.contains("ISMS-P 백업 환경 및 보안 권한 점검 보고서"));
    assert!(html.contains("test-host"));
    assert!(html.contains("PASS"));
}

#[test]
fn test_report_types_rendering() {
    let html_env = render_html_isms_report_with_type(ReportType::Environment, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_env.contains("ISMS-P 백업 환경 및 보안 권한 점검 보고서"));

    let html_ts = render_html_isms_report_with_type(ReportType::TimeSync, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_ts.contains("ISMS-P 시각 동기화 검증 보고서"));

    let html_rd = render_html_isms_report_with_type(ReportType::RestoreDrill, &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"));
    assert!(html_rd.contains("ISMS-P 복구 모의훈련 및 RTO 측정 보고서"));
}

#[test]
fn test_report_file_export_with_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub").join("report.html");

    let msg = execute_report_file_export(ReportType::Environment, Some(&file_path)).unwrap();
    assert!(msg.contains("ISMS report saved to"));
    assert!(file_path.exists());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("ISMS-P 백업 환경 및 보안 권한 점검 보고서"));
}

#[test]
fn test_report_type_all_rendering() {
    let html_all = render_html_isms_report_with_type(ReportType::All, &backup::commands::report::AuditReportMeta::new("host-all", "2026-07-30"));
    assert!(html_all.contains("ISMS-P 종합 감사 보고서"));
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
    assert!(json_str.contains("report_type"));
    assert!(json_str.contains("all"));
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
    assert!(entries.iter().any(|p| p.extension().map_or(false, |ext| ext == "html")));
    assert!(entries.iter().any(|p| p.extension().map_or(false, |ext| ext == "json")));
}

