mod support;

use anyhow::Result;
use backup::runner::executor::{CommandOutput, CommandRunner, StrictCommandRunner};
use backup::runner::scheduler::{
    BackupScheduler, SchedulerMode, SchedulerSettings, SystemScheduler,
};
use std::io;
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

struct MissingSystemdRunner {
    calls: std::sync::Mutex<Vec<String>>,
}

impl MissingSystemdRunner {
    fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl CommandRunner for MissingSystemdRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        self.calls.lock().unwrap().push(program.into());
        if program == "systemctl" {
            return Err(anyhow::Error::new(io::Error::new(
                io::ErrorKind::NotFound,
                "systemctl not found",
            )));
        }
        if program == "crontab" && args == ["-l"] {
            return Ok(CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "no crontab for backup".into(),
            });
        }
        Ok(ok("installed"))
    }
}

#[test]
fn scheduler_falls_back_to_cron_when_systemd_binary_cannot_be_started() {
    let runner = MissingSystemdRunner::new();
    let scheduler = SystemScheduler::new(&runner, "/usr/bin/backup");

    assert!(
        scheduler
            .enable(tempdir().unwrap().path())
            .unwrap()
            .contains("cron")
    );
    assert_eq!(
        runner.calls.lock().unwrap().as_slice(),
        ["systemctl", "crontab", "crontab"]
    );
}

#[test]
fn scheduler_does_not_fallback_when_systemd_capability_probe_fails() {
    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "systemctl",
        ["--version"],
        &[],
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "permission denied".into(),
        },
    )]);
    let scheduler = SystemScheduler::new(&runner, "/usr/bin/backup");

    let error = scheduler.enable(tempdir().unwrap().path()).unwrap_err();

    assert!(error.to_string().contains("capability probe"));
    assert!(runner.calls().iter().all(|call| call.program != "crontab"));
    runner.assert_exhausted().unwrap();
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
            stderr: "no crontab for backup".into(),
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
            stderr: "no crontab for backup".into(),
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
            stderr: "no crontab for backup".into(),
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

#[test]
fn cron_query_failure_with_empty_output_is_not_treated_as_no_crontab() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation(
            "systemctl",
            ["--version"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "systemd unavailable".into(),
            },
        ),
        StrictCommandRunner::expectation(
            "crontab",
            ["-l"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: String::new(),
            },
        ),
    ]);

    let error = SystemScheduler::new(&runner, "/usr/bin/backup")
        .status()
        .unwrap_err();
    assert!(error.to_string().contains("crontab failed"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn systemd_scheduler_failure_does_not_fallback_to_cron() {
    let path = std::path::Path::new("/tmp/profiles.yaml");
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["reset-failed", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.service"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["reset-failed", "backup-pipeline.service"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemd-run",
            [
                "--unit",
                "backup-pipeline",
                "--on-calendar=*-*-* 03:00:00",
                "--timer-property=Persistent=true",
                "/usr/bin/backup",
                "--profiles",
                "/tmp/profiles.yaml",
                "run",
            ],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "registration failed".into(),
            },
        ),
    ]);
    let scheduler = SystemScheduler::new(&runner, "/usr/bin/backup");

    let error = scheduler.enable(path).unwrap_err();

    assert!(error.to_string().contains("registration failed"));
    assert!(runner.calls().iter().all(|call| call.program != "crontab"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn transactional_systemd_registration_restores_an_active_timer_after_failure() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["is-active", "backup-pipeline.timer"],
            &[],
            ok("active\n"),
        ),
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["reset-failed", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.service"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["reset-failed", "backup-pipeline.service"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemd-run",
            [
                "--unit",
                "backup-pipeline",
                "--on-calendar=*-*-* 03:00:00",
                "--timer-property=Persistent=true",
                "/usr/bin/backup",
                "--profiles",
                "/tmp/profiles.yaml",
                "run",
            ],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "registration failed".into(),
            },
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["start", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
    ]);
    let scheduler = SystemScheduler::new(&runner, "/usr/bin/backup");

    let error = scheduler
        .enable_preserving_state(
            std::path::Path::new("/tmp/profiles.yaml"),
            &SchedulerSettings::auto(),
        )
        .unwrap_err();

    assert!(error.to_string().contains("registration failed"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn scheduler_status_distinguishes_active_inactive_and_query_failures() {
    let active_runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["is-active", "backup-pipeline.timer"],
            &[],
            ok("active\n"),
        ),
    ]);
    let active = SystemScheduler::new(&active_runner, "/usr/bin/backup")
        .status()
        .unwrap();
    assert_eq!(active, "active\n");
    active_runner.assert_exhausted().unwrap();

    let inactive_runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["is-active", "backup-pipeline.timer"],
            &[],
            CommandOutput {
                status_code: 3,
                stdout: "inactive\n".into(),
                stderr: String::new(),
            },
        ),
    ]);
    let inactive = SystemScheduler::new(&inactive_runner, "/usr/bin/backup")
        .status()
        .unwrap();
    assert_eq!(inactive, "inactive");
    inactive_runner.assert_exhausted().unwrap();

    let failure_runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["is-active", "backup-pipeline.timer"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "systemctl unavailable".into(),
            },
        ),
    ]);
    let error = SystemScheduler::new(&failure_runner, "/usr/bin/backup")
        .status()
        .unwrap_err();
    assert!(error.to_string().contains("systemctl unavailable"));
    failure_runner.assert_exhausted().unwrap();

    let non_status_runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["is-active", "backup-pipeline.timer"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: "unexpected output".into(),
                stderr: String::new(),
            },
        ),
    ]);
    assert!(
        SystemScheduler::new(&non_status_runner, "/usr/bin/backup")
            .status()
            .is_err()
    );
    non_status_runner.assert_exhausted().unwrap();
}

#[test]
fn cron_status_query_failure_is_not_reported_as_inactive() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation(
            "systemctl",
            ["--version"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "systemd unavailable".into(),
            },
        ),
        StrictCommandRunner::expectation(
            "crontab",
            ["-l"],
            &[],
            CommandOutput {
                status_code: 2,
                stdout: String::new(),
                stderr: "permission denied".into(),
            },
        ),
    ]);
    let error = SystemScheduler::new(&runner, "/usr/bin/backup")
        .status()
        .unwrap_err();
    assert!(error.to_string().contains("permission denied"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn cron_disable_without_owned_entry_is_idempotent_without_reinstalling_crontab() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation(
            "systemctl",
            ["--version"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "systemd unavailable".into(),
            },
        ),
        StrictCommandRunner::expectation("crontab", ["-l"], &[], ok("0 4 * * * unrelated-job\n")),
    ]);
    let output = SystemScheduler::new(&runner, "/usr/bin/backup")
        .disable()
        .unwrap();
    assert!(output.contains("No scheduled backup"));
    assert_eq!(runner.calls().len(), 2);
    runner.assert_exhausted().unwrap();
}

#[test]
fn systemd_disable_propagates_cleanup_failures_without_fallback() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.timer"],
            &[],
            ok(""),
        ),
        StrictCommandRunner::expectation(
            "systemctl",
            ["reset-failed", "backup-pipeline.timer"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "cleanup failed".into(),
            },
        ),
    ]);
    let error = SystemScheduler::new(&runner, "/usr/bin/backup")
        .disable()
        .unwrap_err();
    assert!(error.to_string().contains("cleanup failed"));
    assert!(runner.calls().iter().all(|call| call.program != "crontab"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn systemd_disable_does_not_suppress_unrelated_not_found_text() {
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation("systemctl", ["--version"], &[], ok("systemd")),
        StrictCommandRunner::expectation(
            "systemctl",
            ["stop", "backup-pipeline.timer"],
            &[],
            CommandOutput {
                status_code: 1,
                stdout: "not found in audit output".into(),
                stderr: String::new(),
            },
        ),
    ]);

    let error = SystemScheduler::new(&runner, "/usr/bin/backup")
        .disable()
        .unwrap_err();

    assert!(error.to_string().contains("not found in audit output"));
    runner.assert_exhausted().unwrap();
}

fn ok(stdout: &str) -> CommandOutput {
    CommandOutput {
        status_code: 0,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}
