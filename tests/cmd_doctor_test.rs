use backup::commands::doctor::run_doctor_checks;
use backup::runner::rclone::MockRcloneRunner;

#[test]
fn test_doctor_checks() {
    let mock_rclone = MockRcloneRunner::new(0, "syno_backup");
    let report = run_doctor_checks(&mock_rclone, None).unwrap();
    assert!(report.contains("Rclone connectivity: OK"));
    assert!(report.contains("Restic binary: OK"));
    assert!(report.contains("NTP Time Sync: OK"));
}

#[test]
fn test_doctor_status_enum_and_ntp_check() {
    use backup::commands::doctor::{DoctorStatus, DoctorCategory, DoctorItem};
    let item = DoctorItem {
        category: DoctorCategory::System,
        criterion: "NTP Time Sync".into(),
        status: DoctorStatus::Pass,
        detail: "chronyd active".into(),
    };
    assert_eq!(item.status, DoctorStatus::Pass);
}

