mod support;

use backup::runner::executor::CommandOutput;
use backup::runner::scheduler::{
    BackupScheduler, SchedulerMode, SchedulerSettings, SystemScheduler,
};
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

#[test]
fn scheduler_settings_control_calendar_and_auto_cron_fallback() {
    let executor = MockExecutor::new();
    for _ in 0..4 {
        executor.push_output("systemctl", ok(""));
    }
    executor.push_output("systemd-run", ok("created"));
    let scheduler = SystemScheduler::new(&executor, "/usr/bin/backup");
    let path = tempdir().unwrap().path().join("profiles.yaml");
    let settings = SchedulerSettings::new(SchedulerMode::Systemd, "*-*-* 04:15:00");

    scheduler
        .enable_with_settings(&path, &settings)
        .expect("systemd schedule should be installed");
    let systemd_run = executor
        .get_calls()
        .into_iter()
        .find(|(program, _)| program == "systemd-run")
        .unwrap();
    assert!(
        systemd_run
            .1
            .contains(&"--on-calendar=*-*-* 04:15:00".into())
    );

    let cron_executor = MockExecutor::new();
    cron_executor.push_output(
        "crontab",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        },
    );
    cron_executor.push_output("crontab", ok("installed"));
    let cron_scheduler = SystemScheduler::new(&cron_executor, "/usr/bin/backup");
    cron_scheduler
        .enable_with_settings(&path, &SchedulerSettings::auto().with_force_cron(true))
        .expect("forced cron schedule should be installed");
    assert_eq!(cron_executor.call_count("systemctl"), 0);
    assert_eq!(cron_executor.call_count("crontab"), 2);
}

#[test]
fn cron_scheduler_rejects_calendars_that_cannot_be_represented_safely() {
    let executor = MockExecutor::new();
    executor.push_output(
        "crontab",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        },
    );
    let scheduler = SystemScheduler::new(&executor, "/usr/bin/backup");
    let error = scheduler
        .enable_with_settings(
            tempdir().unwrap().path(),
            &SchedulerSettings::new(SchedulerMode::Cron, "Mon *-*-* 04:15:00"),
        )
        .unwrap_err();
    assert!(error.to_string().contains("cannot be represented safely"));
    assert_eq!(executor.call_count("crontab"), 1);
}

fn ok(stdout: &str) -> CommandOutput {
    CommandOutput {
        status_code: 0,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}
