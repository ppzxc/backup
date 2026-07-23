use backup::commands::doctor::run_doctor_checks;
use backup::runner::rclone::MockRcloneRunner;

#[test]
fn test_doctor_checks() {
    let mock_rclone = MockRcloneRunner::new(0, "syno_backup");
    let report = run_doctor_checks(&mock_rclone).unwrap();
    assert!(report.contains("Rclone connectivity: OK"));
}
