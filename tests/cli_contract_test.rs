use backup::cli::{
    AdapterSelection, AdapterSet, Cli, CliRuntimeContext, CommandOutcome, ContractDiagnostic,
    SchedulerMode, authoritative_cli_axes, authoritative_cli_command_paths,
    authoritative_cli_schema, dispatch, generate_cli_contract_matrix,
};
use backup::i18n::Language;
use backup::runner::executor::{CommandOutput, CommandRunner, StrictCommandRunner};
use backup::runner::rclone::RcloneTool;
use backup::runner::restic::ResticTool;
use backup::runner::resticprofile::ResticProfileTool;
use backup::runner::scheduler::SystemScheduler;
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

#[test]
fn authoritative_schema_owns_every_top_level_command() {
    let schema = authoritative_cli_schema();
    let command_names = schema
        .get_subcommands()
        .map(|command| command.get_name())
        .collect::<Vec<_>>();

    assert_eq!(
        command_names,
        vec![
            "setup",
            "copy",
            "run",
            "database",
            "doctor",
            "report",
            "schedule",
            "restore",
            "snapshots",
            "status",
            "update",
            "version",
            "uninstall",
        ]
    );
    assert_eq!(Cli::command().get_name(), "backup");
}

#[test]
fn contract_matrix_coverage_guard_consumes_every_schema_option_axis() {
    let actual = authoritative_cli_axes();
    let expected = [
        "backup.copy.dry_run",
        "backup.copy.profile",
        "backup.database.dry_run",
        "backup.log_file",
        "backup.profiles",
        "backup.quiet",
        "backup.report.environment.file",
        "backup.report.environment.format",
        "backup.report.file",
        "backup.report.format",
        "backup.report.restore-drill.file",
        "backup.report.restore-drill.format",
        "backup.report.time-sync.file",
        "backup.report.time-sync.format",
        "backup.restore.force",
        "backup.restore.snapshot",
        "backup.restore.storage",
        "backup.restore.target",
        "backup.run.dry_run",
        "backup.run.profile",
        "backup.run.skip_database",
        "backup.run.skip_retention",
        "backup.run.skip_secondary_sync",
        "backup.setup.lang",
        "backup.setup.non_interactive",
        "backup.status.profile",
        "backup.uninstall.purge",
        "backup.uninstall.yes",
        "backup.verbose",
    ];
    assert_eq!(actual, expected);

    let matrix_axes = generate_cli_contract_matrix()
        .into_iter()
        .filter_map(|case| case.option_axis)
        .collect::<Vec<_>>();
    assert_eq!(matrix_axes, expected);
}

#[test]
fn contract_matrix_coverage_guard_consumes_every_nested_command_path() {
    let expected = vec![
        "backup",
        "backup.copy",
        "backup.database",
        "backup.doctor",
        "backup.report",
        "backup.report.environment",
        "backup.report.restore-drill",
        "backup.report.time-sync",
        "backup.restore",
        "backup.run",
        "backup.schedule",
        "backup.schedule.disable",
        "backup.schedule.enable",
        "backup.schedule.status",
        "backup.setup",
        "backup.setup.backend-init",
        "backup.setup.dependencies",
        "backup.snapshots",
        "backup.status",
        "backup.uninstall",
        "backup.update",
        "backup.version",
    ];
    assert_eq!(authoritative_cli_command_paths(), expected);

    let matrix_paths = generate_cli_contract_matrix()
        .into_iter()
        .filter(|case| case.option_axis.is_none())
        .map(|case| case.command_path)
        .collect::<Vec<_>>();
    assert_eq!(matrix_paths, expected);
}

#[test]
fn matrix_cases_have_stable_ids_and_failure_diagnostics() {
    let cases = generate_cli_contract_matrix();
    let ids = cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<Vec<_>>();
    let mut unique_ids = ids.clone();
    unique_ids.sort_unstable();
    unique_ids.dedup();
    assert_eq!(ids.len(), unique_ids.len());

    let diagnostic = ContractDiagnostic {
        case_id: cases[0].id.clone(),
        context: "profiles=/tmp/contract scheduler=auto".into(),
        expected_trace: vec!["resticprofile copy".into()],
        actual_trace: vec!["resticprofile backup".into()],
        stdout: "data".into(),
        stderr: "diagnostic".into(),
        exit_status: 1,
        artifacts: Vec::new(),
    };
    let rendered = diagnostic.render();
    assert!(rendered.contains(&diagnostic.case_id));
    assert!(rendered.contains("expected_trace"));
    assert!(rendered.contains("actual_trace"));
    assert!(rendered.contains("exit_status=1"));
}

#[test]
fn runtime_context_is_explicit_and_does_not_need_process_environment() {
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        "/tmp/contract/profiles.yaml",
        "--log-file",
        "/tmp/contract/backup.log",
        "-vv",
        "run",
        "--dry-run",
    ])
    .unwrap();

    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::Ko,
        Some("warn".into()),
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();

    assert_eq!(
        context.profiles_path,
        PathBuf::from("/tmp/contract/profiles.yaml")
    );
    assert_eq!(context.language, Language::Ko);
    assert_eq!(context.logging.level_filter, "trace");
    assert_eq!(
        context.logging.log_file,
        Some(PathBuf::from("/tmp/contract/backup.log"))
    );
    assert_eq!(context.scheduler_mode, SchedulerMode::Auto);
    assert_eq!(context.adapter_selection, AdapterSelection::StrictTest);
    assert_eq!(context.host_name, "localhost");
}

#[test]
fn command_outcome_separates_data_diagnostics_and_artifacts() {
    let outcome = CommandOutcome::success(
        "report data",
        "diagnostic warning",
        vec![PathBuf::from("/tmp/report.json")],
    );

    assert_eq!(outcome.stdout, "report data");
    assert_eq!(outcome.stderr, "diagnostic warning");
    assert_eq!(outcome.exit_status, 0);
    assert_eq!(outcome.artifacts[0].path, PathBuf::from("/tmp/report.json"));
    assert!(outcome.external_state_changes.is_empty());

    let failure = CommandOutcome::failure("run", "primary backup", "backend unavailable");
    assert_eq!(failure.exit_status, 1);
    assert!(failure.stderr.contains("run"));
    assert!(failure.stderr.contains("primary backup"));
    assert!(failure.stderr.contains("backend unavailable"));
}

#[test]
fn strict_command_runner_rejects_unexpected_calls_and_requires_exhaustion() {
    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "restic",
        ["version"],
        &[],
        CommandOutput {
            status_code: 0,
            stdout: "restic 0.17".into(),
            stderr: String::new(),
        },
    )]);

    let output =
        backup::runner::executor::CommandRunner::run(&runner, "restic", &["version"]).unwrap();
    assert_eq!(output.stdout, "restic 0.17");
    runner.assert_exhausted().unwrap();

    let unexpected = runner.run("rclone", &["listremotes"]).unwrap_err();
    assert!(unexpected.to_string().contains("unexpected command"));

    let secret_runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "restic",
        [
            "--password",
            "super-secret",
            "--repo",
            "s3:https://private.example/repo",
        ],
        &[("AWS_SECRET_ACCESS_KEY", "secret-value")],
        CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        },
    )]);
    let error = secret_runner
        .run_with_env(
            "restic",
            &[
                "--password",
                "other-secret",
                "--repo",
                "s3:https://other.example/repo",
            ],
            &[("AWS_SECRET_ACCESS_KEY", "other-value")],
        )
        .unwrap_err()
        .to_string();
    assert!(!error.contains("super-secret"));
    assert!(!error.contains("other-secret"));
    assert!(!error.contains("private.example"));
    assert!(error.contains("<redacted>"));
}

#[test]
fn shared_dispatch_reaches_the_strict_adapter_with_exact_profile_and_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    std::fs::write(
        &profiles,
        "version: '2'\nprofiles:\n  default:\n    repository: /tmp/repo\n    backup:\n      source: ['/tmp']\n",
    )
    .unwrap();

    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "resticprofile",
        [
            "--config",
            profiles.to_str().unwrap(),
            "--name",
            "default",
            "--dry-run",
            "copy",
        ],
        &[],
        CommandOutput {
            status_code: 0,
            stdout: "copy planned".into(),
            stderr: String::new(),
        },
    )]);
    let resticprofile = ResticProfileTool::new(&runner);
    let restic = ResticTool::new(&runner);
    let rclone = RcloneTool::new(&runner);
    let scheduler = SystemScheduler::new(&runner, "backup");
    let adapters = AdapterSet {
        command: &runner,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &resticprofile,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        profiles.to_str().unwrap(),
        "copy",
        "--dry-run",
    ])
    .unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();

    let outcome = dispatch(&context, cli.command, &adapters);
    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert!(outcome.stdout.contains("copy planned"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn invalid_profile_fails_before_any_strict_adapter_call() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    std::fs::write(
        &profiles,
        "version: '2'\nprofiles:\n  default:\n    repository: /tmp/repo\n    backup:\n      source: ['/tmp']\n",
    )
    .unwrap();
    let runner = StrictCommandRunner::new([]);
    let resticprofile = ResticProfileTool::new(&runner);
    let restic = ResticTool::new(&runner);
    let rclone = RcloneTool::new(&runner);
    let scheduler = SystemScheduler::new(&runner, "backup");
    let adapters = AdapterSet {
        command: &runner,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &resticprofile,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        profiles.to_str().unwrap(),
        "copy",
        "--profile",
        "unknown",
    ])
    .unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();

    let outcome = dispatch(&context, cli.command, &adapters);
    assert_eq!(outcome.exit_status, 1);
    assert!(outcome.stderr.contains("not configured"));
    runner.assert_exhausted().unwrap();
}
