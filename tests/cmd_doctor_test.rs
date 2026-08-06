use backup::commands::doctor::run_doctor_checks;
mod support;
use support::MockRcloneRunner;

fn write_mode_600(path: &std::path::Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn set_mode_700(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
}

#[test]
fn test_doctor_checks() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("profiles.yaml");
    write_mode_600(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: rclone:primary-remote:/backup\n",
    );
    let mock_rclone = MockRcloneRunner::new(0, "syno_backup");
    let report = run_doctor_checks(&mock_rclone, Some(&config_path)).unwrap();
    assert!(report.contains("Storage connectivity: Pass"));
    assert!(report.contains("Restic binary: Pass"));
    assert!(report.contains("NTP Time Sync: Pass"));
}

#[test]
fn doctor_reports_missing_explicit_temporary_config_path() {
    use crate::support::MockExecutor;
    use backup::commands::doctor::{DoctorCategory, SystemHealthDiagnoser};

    let temp = tempfile::tempdir().unwrap();
    let missing_config = temp.path().join("missing/profiles.yaml");
    let snapshot = SystemHealthDiagnoser::diagnose_with_runner(
        &MockRcloneRunner::new(0, "syno_backup"),
        &MockExecutor::new(),
        Some(&missing_config),
    );

    let config_item = snapshot
        .items
        .iter()
        .find(|item| item.category == DoctorCategory::Config)
        .unwrap();
    assert_eq!(
        config_item.detail,
        "Unified profiles configuration is missing"
    );
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
    use crate::support::MockExecutor;
    use backup::commands::doctor::{DoctorStatus, check_ntp_sync_with_runner};
    use backup::runner::executor::CommandOutput;

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

#[test]
fn doctor_runs_both_storage_checks_even_when_the_first_one_fails() {
    use anyhow::Result;
    use backup::commands::doctor::run_doctor_contract_with_runner;
    use backup::runner::executor::{CommandOutput, CommandRunner};
    use std::path::Path;
    use std::sync::Mutex;

    struct RecordingRclone {
        calls: Mutex<Vec<String>>,
    }

    impl backup::runner::rclone::RcloneRunner for RecordingRclone {
        fn check_connectivity(&self, remote: &str) -> Result<String> {
            self.calls.lock().unwrap().push(remote.into());
            if remote == "primary-remote" {
                anyhow::bail!("primary remote unavailable")
            }
            Ok("secondary reachable".into())
        }

        fn list_remotes(&self) -> Result<String> {
            Ok(String::new())
        }

        fn sync(&self, _source: &str, _destination: &str) -> Result<String> {
            Ok(String::new())
        }
    }

    struct SuccessfulCommands;

    impl CommandRunner for SuccessfulCommands {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<CommandOutput> {
            Ok(CommandOutput {
                status_code: 0,
                stdout: "Reference ID: test".into(),
                stderr: String::new(),
            })
        }

        fn run_with_env(
            &self,
            _program: &str,
            args: &[&str],
            _environment: &[(&str, &str)],
        ) -> Result<CommandOutput> {
            if let Some(target) = args
                .windows(2)
                .find_map(|pair| (pair[0] == "--target").then_some(pair[1]))
            {
                std::fs::write(Path::new(target).join("restored.txt"), "restored").unwrap();
            }
            Ok(CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    let config = tempfile::tempdir().unwrap();
    let config_path = config.path().join("profiles.yaml");
    write_mode_600(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: rclone:primary-remote:/tmp/repo\n    password-file: password\n  secondary:\n    repository: rclone:secondary-remote:/tmp/repo\n    password-file: password\n",
    );
    write_mode_600(&config.path().join("password"), "doctor-password");

    let rclone = RecordingRclone {
        calls: Mutex::new(Vec::new()),
    };
    let (report, _passed) = run_doctor_contract_with_runner(
        &rclone,
        &SuccessfulCommands,
        Some(Path::new(&config_path)),
        "contract-host",
    )
    .unwrap();

    assert_eq!(
        rclone.calls.lock().unwrap().as_slice(),
        ["primary-remote", "secondary-remote"]
    );
    assert!(report.contains("Storage connectivity: Fail"));
}

#[test]
fn doctor_probes_non_rclone_backend_through_restic() {
    use anyhow::Result;
    use backup::runner::executor::{CommandOutput, CommandRunner};
    use std::path::Path;
    use std::sync::Mutex;

    struct RecordingRclone {
        calls: Mutex<Vec<String>>,
    }

    impl backup::runner::rclone::RcloneRunner for RecordingRclone {
        fn check_connectivity(&self, remote: &str) -> Result<String> {
            self.calls.lock().unwrap().push(remote.into());
            Ok("unexpected rclone probe".into())
        }

        fn list_remotes(&self) -> Result<String> {
            Ok(String::new())
        }

        fn sync(&self, _source: &str, _destination: &str) -> Result<String> {
            Ok(String::new())
        }
    }

    struct RecordingCommands {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl CommandRunner for RecordingCommands {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<CommandOutput> {
            Ok(CommandOutput {
                status_code: 0,
                stdout: "Reference ID: test".into(),
                stderr: String::new(),
            })
        }

        fn run_with_env(
            &self,
            program: &str,
            args: &[&str],
            _environment: &[(&str, &str)],
        ) -> Result<CommandOutput> {
            self.calls.lock().unwrap().push(
                std::iter::once(program.to_string())
                    .chain(args.iter().map(|arg| (*arg).to_string()))
                    .collect(),
            );
            if let Some(target) = args
                .windows(2)
                .find_map(|pair| (pair[0] == "--target").then_some(pair[1]))
            {
                std::fs::write(Path::new(target).join("restored.txt"), "restored").unwrap();
            }
            Ok(CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    let config = tempfile::tempdir().unwrap();
    set_mode_700(config.path());
    let config_path = config.path().join("profiles.yaml");
    write_mode_600(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: s3:s3.example/bucket\n    password-file: password\n",
    );
    write_mode_600(&config.path().join("password"), "doctor-password");
    let rclone = RecordingRclone {
        calls: Mutex::new(Vec::new()),
    };
    let commands = RecordingCommands {
        calls: Mutex::new(Vec::new()),
    };

    let (report, passed, diagnostics) =
        backup::commands::doctor::run_doctor_contract_with_runner_and_diagnostics(
            &rclone,
            &commands,
            Some(&config_path),
            "contract-host",
        )
        .unwrap();

    assert!(passed, "{report}\n{diagnostics}");
    assert!(report.contains("Storage connectivity: Pass"));
    assert!(rclone.calls.lock().unwrap().is_empty());
    assert!(commands.calls.lock().unwrap().iter().any(|call| {
        call.first().is_some_and(|program| program == "restic")
            && call.iter().any(|argument| argument == "snapshots")
    }));
}
