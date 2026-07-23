use backup::commands::doctor::{execute_doctor_file_export, render_html_isms_report, run_doctor_checks};
use backup::runner::rclone::MockRcloneRunner;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_doctor_checks() {
    let mock_rclone = MockRcloneRunner::new(0, "syno_backup");
    let report = run_doctor_checks(&mock_rclone).unwrap();
    assert!(report.contains("Rclone connectivity: OK"));
}

#[test]
fn test_doctor_html_report_rendering() {
    let html = render_html_isms_report("test-host", "2026-07-23");
    assert!(html.contains("ISMS-P 인증 감사 증적 보고서"));
    assert!(html.contains("test-host"));
    assert!(html.contains("PASS"));
}

#[test]
fn test_diagnostic_collector_and_renderer_seam() {
    use backup::commands::doctor::{DiagnosticCollector, IsmsReportRenderer};
    let results = DiagnosticCollector::collect("host-abc", "2026-07-23 12:00:00");
    assert_eq!(results.host_name, "host-abc");
    assert!(results.overall_pass);
    assert_eq!(results.items.len(), 3);

    let renderer = IsmsReportRenderer;
    let html = renderer.render_html(&results);
    assert!(html.contains("host-abc"));
    assert!(html.contains("0700 / 0600"));
}

#[test]
fn test_doctor_file_export_with_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub").join("report.html");

    let msg = execute_doctor_file_export(Some(&file_path)).unwrap();
    assert!(msg.contains("ISMS report saved to"));
    assert!(file_path.exists());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("ISMS-P 인증 감사 증적 보고서"));
}
