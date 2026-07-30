use backup::commands::report::{execute_report_file_export, render_html_isms_report_with_type, AuditReportMeta, ReportType};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_isms_environment_audit_report_format() {
    let meta = AuditReportMeta::new("prod-db-server-01", "2026-07-23 15:34:00");
    let html = render_html_isms_report_with_type(ReportType::Environment, &meta);
    assert!(html.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
    assert!(html.contains("700"));
    assert!(html.contains("600"));
}

#[test]
fn test_isms_time_sync_audit_report_format() {
    let meta = AuditReportMeta::new("prod-db-server-01", "2026-07-23 15:34:00");
    let html = render_html_isms_report_with_type(ReportType::TimeSync, &meta);
    assert!(html.contains("ISMS-P 2.9.3 시각 동기화 증적 보고서"));
}

#[test]
fn test_isms_restore_drill_rto_audit_report_format() {
    let meta = AuditReportMeta::new("prod-db-server-01", "2026-07-23 15:34:00");
    let html = render_html_isms_report_with_type(ReportType::RestoreDrill, &meta);
    assert!(html.contains("백업 데이터 복구 및 정합성 테스트 결과 보고서"));
}

#[test]
fn test_isms_export_creates_valid_file() {
    let dir = tempdir().unwrap();
    let export_path = dir.path().join("isms_report.html");

    let result = execute_report_file_export(ReportType::Environment, Some(&export_path)).unwrap();
    assert!(result.contains("ISMS report saved to"));
    assert!(export_path.exists());

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
}
