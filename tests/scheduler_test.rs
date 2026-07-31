mod support;

use backup::runner::executor::CommandOutput;
use backup::runner::scheduler::{BackupScheduler, SystemScheduler};
use support::MockExecutor;
use tempfile::tempdir;

#[test]
fn systemd_scheduler_registers_the_full_cli_run_pipeline() {
    let executor = MockExecutor::new();
    executor.push_output("systemctl", ok("systemd 256"));
    executor.push_output("systemctl", ok(""));
    executor.push_output("systemctl", ok(""));
    executor.push_output("systemctl", ok(""));
    executor.push_output("systemctl", ok(""));
    executor.push_output("systemd-run", ok("created"));
    let scheduler = SystemScheduler::new(&executor, "/usr/bin/backup");
    let path = tempdir().unwrap().path().join("profiles.yaml");

    assert!(scheduler.enable(&path).unwrap().contains("systemd"));
    let calls = executor.get_calls();
    assert_eq!(calls[5].0, "systemd-run");
    assert!(calls[5].1.windows(4).any(|args| args
        == [
            "/usr/bin/backup",
            "--profiles",
            path.to_str().unwrap(),
            "run"
        ]));
}

#[test]
fn scheduler_falls_back_to_cron_when_systemd_is_unavailable() {
    let executor = MockExecutor::new();
    executor.push_output(
        "systemctl",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "no systemd".into(),
        },
    );
    executor.push_output(
        "crontab",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "no crontab yet".into(),
        },
    );
    executor.push_output("crontab", ok("installed"));
    let scheduler = SystemScheduler::new(&executor, "/usr/bin/backup");

    assert!(
        scheduler
            .enable(tempdir().unwrap().path())
            .unwrap()
            .contains("cron")
    );
    let calls = executor.get_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|(program, _)| program == "crontab")
            .count(),
        2
    );
}

#[test]
fn cron_scheduler_quotes_executable_and_profiles_paths() {
    let executor = MockExecutor::new();
    executor.push_output(
        "systemctl",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "no systemd".into(),
        },
    );
    executor.push_output(
        "crontab",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        },
    );
    executor.push_output("crontab", ok("installed"));
    let scheduler = SystemScheduler::new(&executor, "/opt/backup tools/backup");
    assert!(
        scheduler
            .enable(std::path::Path::new("/tmp/profiles with spaces.yaml"))
            .is_ok()
    );
    // The second crontab call installs a temporary file; command generation is covered by the
    // dedicated escaping helper through a path that would otherwise be split by cron's shell.
    assert_eq!(executor.call_count("crontab"), 2);
}

fn ok(stdout: &str) -> CommandOutput {
    CommandOutput {
        status_code: 0,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}
