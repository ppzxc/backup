use backup::commands::report::{execute_report_file_export, render_html_isms_report, render_html_isms_report_with_type, AuditReport, ReportType};
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
