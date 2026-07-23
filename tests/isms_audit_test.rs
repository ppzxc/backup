use backup::commands::doctor::{execute_doctor_file_export, render_html_isms_report};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_isms_environment_audit_report_format() {
    let html = render_html_isms_report("prod-db-server-01", "2026-07-23 15:34:00");
    assert!(html.contains("ISMS-P 인증 감사 증적 보고서"));
    assert!(html.contains("ISMS-P 2.9.2"));
    assert!(html.contains("0700"));
    assert!(html.contains("0600"));
    assert!(html.contains("******"));
}

#[test]
fn test_isms_time_sync_audit_report_format() {
    let html = render_html_isms_report("prod-db-server-01", "2026-07-23 15:34:00");
    assert!(html.contains("ISMS-P 2.10.1"));
    assert!(html.contains("+0.0004s"));
}

#[test]
fn test_isms_restore_drill_rto_audit_report_format() {
    let html = render_html_isms_report("prod-db-server-01", "2026-07-23 15:34:00");
    assert!(html.contains("ISMS-P 2.9.3"));
    assert!(html.contains("17.0s"));
}

#[test]
fn test_isms_export_creates_valid_file() {
    let dir = tempdir().unwrap();
    let export_path = dir.path().join("isms_report.html");

    let result = execute_doctor_file_export(Some(&export_path)).unwrap();
    assert!(result.contains("ISMS report saved to"));
    assert!(export_path.exists());

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("ISMS-P 인증 감사 증적 보고서"));
}
