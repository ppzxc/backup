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
    use backup::commands::doctor::{DoctorCategory, DoctorItem, DoctorStatus};
    let item = DoctorItem {
        category: DoctorCategory::System,
        criterion: "NTP Time Sync".into(),
        status: DoctorStatus::Pass,
        detail: "chronyd active".into(),
    };
    assert_eq!(item.status, DoctorStatus::Pass);
}

#[test]
fn test_ntp_sync_with_mock_executor() {
    use backup::commands::doctor::{DoctorStatus, check_ntp_sync_with_runner};
    use backup::runner::executor::{CommandOutput, MockExecutor};

    let mock = MockExecutor::new();
    mock.push_output(
        "chronyc",
        CommandOutput {
            status_code: 0,
            stdout: "Reference ID    : 192.168.1.1\nSystem time     : 0.0001 sec fast".into(),
            stderr: "".into(),
        },
    );

    let (status, detail) = check_ntp_sync_with_runner(&mock);
    assert_eq!(status, DoctorStatus::Pass);
    assert!(detail.contains("chronyd active"));

    let mock_fail = MockExecutor::new();
    mock_fail.push_output(
        "chronyc",
        CommandOutput {
            status_code: 1,
            stdout: "".into(),
            stderr: "506 Cannot talk to daemon".into(),
        },
    );
    mock_fail.push_output(
        "timedatectl",
        CommandOutput {
            status_code: 1,
            stdout: "".into(),
            stderr: "Failed to query server".into(),
        },
    );

    let (status_fail, detail_fail) = check_ntp_sync_with_runner(&mock_fail);
    assert_eq!(status_fail, DoctorStatus::Warn);
    assert!(detail_fail.contains("NTP synchronization status unknown or inactive"));
}
