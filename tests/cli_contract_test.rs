use backup::cli::{
    AdapterSelection, AdapterSet, Cli, CliRuntimeContext, CommandOutcome, ContractCaseSpec,
    ContractDiagnostic, SchedulerMode, authoritative_cli_axes, authoritative_cli_command_paths,
    authoritative_cli_schema, dispatch, generate_cli_contract_matrix,
    generate_cli_contract_matrix_with_specs,
};
use backup::commands::report::ReportConfig;
use backup::i18n::Language;
use backup::runner::executor::{CommandOutput, CommandRunner, StrictCommandRunner};
use backup::runner::rclone::RcloneTool;
use backup::runner::restic::ResticTool;
use backup::runner::resticprofile::ResticProfileTool;
use backup::runner::scheduler::SystemScheduler;
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

const CONTRACT_COMMAND_PATHS: &[&str] = &[
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

const CONTRACT_OPTION_AXES: &[&str] = &[
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
    assert_eq!(
        actual,
        CONTRACT_OPTION_AXES
            .iter()
            .map(|axis| (*axis).into())
            .collect::<Vec<String>>()
    );

    let matrix_axes = generate_cli_contract_matrix()
        .into_iter()
        .filter_map(|case| case.option_axis)
        .collect::<Vec<_>>();
    assert_eq!(
        matrix_axes,
        CONTRACT_OPTION_AXES
            .iter()
            .map(|axis| (*axis).into())
            .collect::<Vec<String>>()
    );
}

#[test]
fn contract_matrix_coverage_guard_consumes_every_nested_command_path() {
    let expected = CONTRACT_COMMAND_PATHS
        .iter()
        .map(|path| (*path).into())
        .collect::<Vec<String>>();
    assert_eq!(authoritative_cli_command_paths(), expected);

    let matrix_paths = generate_cli_contract_matrix()
        .into_iter()
        .filter(|case| case.option_axis.is_none())
        .map(|case| case.command_path)
        .collect::<Vec<_>>();
    assert_eq!(
        matrix_paths,
        CONTRACT_COMMAND_PATHS
            .iter()
            .map(|path| (*path).into())
            .collect::<Vec<String>>()
    );
}

#[test]
fn matrix_cases_have_stable_ids_and_failure_diagnostics() {
    let cases = generate_cli_contract_matrix_with_specs(contract_matrix_specs()).unwrap();
    assert!(cases.iter().all(|case| case.expectation.is_some()));
    assert!(
        cases
            .iter()
            .any(|case| case.id == "option:backup.copy.dry_run:flag-enabled")
    );
    let ids = cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<Vec<_>>();
    let mut unique_ids = ids.clone();
    unique_ids.sort_unstable();
    unique_ids.dedup();
    assert_eq!(ids.len(), unique_ids.len());

    let failure = CommandOutcome::failure("copy", "adapter", "diagnostic");
    let diagnostic = ContractDiagnostic::from_outcome(
        cases[0].id.clone(),
        "profiles=/tmp/contract scheduler=auto",
        vec!["resticprofile copy".into()],
        vec!["resticprofile backup".into()],
        &failure,
    );
    let rendered = diagnostic.render();
    assert!(rendered.contains(&diagnostic.case_id));
    assert!(rendered.contains("expected_trace"));
    assert!(rendered.contains("actual_trace"));
    assert!(rendered.contains("exit_status=1"));
}

#[test]
fn every_matrix_input_is_accepted_by_the_authoritative_parser() {
    let cases = generate_cli_contract_matrix_with_specs(contract_matrix_specs()).unwrap();
    for case in cases {
        Cli::try_parse_from(&case.argv)
            .unwrap_or_else(|error| panic!("{} rejected: {error}", case.id));
    }
}

#[test]
fn configuration_gated_matrix_cases_use_shared_dispatch_and_strict_adapters() {
    let temp = tempfile::tempdir().unwrap();
    let cases = generate_cli_contract_matrix_with_specs(contract_matrix_specs()).unwrap();
    for case in cases {
        let cli = Cli::try_parse_from(&case.argv).unwrap();
        let mut context = CliRuntimeContext::from_cli(
            &cli,
            Language::En,
            None,
            SchedulerMode::Auto,
            AdapterSelection::StrictTest,
        )
        .unwrap();
        context = context.with_environment(
            temp.path().join("home"),
            "contract-host",
            backup::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR,
        );
        context.profiles_path = temp.path().join(format!(
            "missing-{}.yaml",
            case.id.replace([':', '/', '='], "-")
        ));
        if case.command_path == "backup.uninstall"
            && case.argv.iter().any(|argument| argument == "--yes")
        {
            std::fs::write(&context.profiles_path, "version: '2'\nprofiles: {}\n").unwrap();
        }

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

        let outcome = dispatch(&context, cli.command, &adapters);
        let expectation = case.expectation.as_ref().unwrap();
        let actual_trace = runner
            .calls()
            .into_iter()
            .map(|call| {
                let args = call
                    .args
                    .join(" ")
                    .replace(
                        context.profiles_path.to_string_lossy().as_ref(),
                        "<profiles>",
                    )
                    .replace(context.home_dir.to_string_lossy().as_ref(), "<home>");
                format!(
                    "{} {} env={:?} timeout={:?}",
                    call.program, args, call.environment, call.timeout
                )
            })
            .collect::<Vec<_>>();
        let diagnostic = ContractDiagnostic::from_outcome(
            case.id.clone(),
            format!(
                "argv={:?} profiles={}",
                case.argv,
                context.profiles_path.display()
            ),
            expectation.adapter_trace.clone(),
            actual_trace.clone(),
            &outcome,
        )
        .render();
        assert_eq!(outcome.exit_status, expectation.exit_status, "{diagnostic}");
        assert_eq!(
            outcome.external_state_changes, expectation.external_state_changes,
            "{diagnostic}"
        );
        let actual_artifact_kinds = outcome
            .artifacts
            .iter()
            .map(|artifact| artifact.kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_artifact_kinds, expectation.artifact_kinds,
            "{diagnostic}"
        );
        assert_eq!(
            outcome.stdout.is_empty(),
            expectation.stdout.is_empty(),
            "{diagnostic}"
        );
        if !expectation.stdout.is_empty() {
            assert!(
                outcome.stdout.starts_with(&expectation.stdout),
                "{diagnostic}"
            );
        }
        assert_eq!(
            outcome.stderr.is_empty(),
            expectation.stderr.is_empty(),
            "{diagnostic}"
        );
        if !expectation.stderr.is_empty() {
            assert!(outcome.stderr.contains(&expectation.stderr), "{diagnostic}");
        }
        assert_eq!(actual_trace, expectation.adapter_trace, "{diagnostic}");
        runner.assert_exhausted().unwrap();
    }
}

fn contract_matrix_specs() -> Vec<ContractCaseSpec> {
    let mut specs = CONTRACT_COMMAND_PATHS
        .into_iter()
        .map(|command_path| {
            let command_path = (*command_path).to_owned();
            let expectation = contract_expectation(&command_path, None);
            ContractCaseSpec {
                argv: command_path_args(&command_path),
                command_path,
                option_axis: None,
                behavior_class: "command-default".into(),
                values: Vec::new(),
                expectation,
            }
        })
        .collect::<Vec<_>>();
    specs.extend(CONTRACT_OPTION_AXES.into_iter().map(|option_axis| {
        let option_axis = (*option_axis).to_owned();
        let command_path = option_axis
            .rsplit_once('.')
            .map(|(path, _)| path.to_owned())
            .unwrap_or_else(|| option_axis.clone());
        let (behavior_class, values, option_args) = option_case(&option_axis);
        let mut argv = command_path_args(&command_path);
        if option_axis != "backup.setup.non_interactive" {
            argv.extend(option_args);
        }
        let expectation = contract_expectation(&command_path, Some(&option_axis));
        ContractCaseSpec {
            command_path,
            option_axis: Some(option_axis),
            behavior_class: behavior_class.into(),
            values,
            argv,
            expectation,
        }
    }));
    specs
}

fn contract_expectation(
    command_path: &str,
    option_axis: Option<&String>,
) -> backup::cli::ContractExpectation {
    let succeeds = command_path == "backup" || command_path == "backup.version";
    let adapter_trace = if command_path == "backup.doctor" {
        vec![
            "restic version env=[] timeout=None".into(),
            "rclone lsd default env=[] timeout=None".into(),
            "rclone lsd syno_backup env=[] timeout=None".into(),
            "chronyc tracking env=[] timeout=None".into(),
            "timedatectl status env=[] timeout=None".into(),
        ]
    } else if command_path == "backup.update" {
        vec![
            "curl -fsSL -H User-Agent: backup-cli -H Accept: application/vnd.github.v3+json https://api.github.com/repos/ppzxc/backup/releases/latest env=[] timeout=None".into(),
        ]
    } else if (command_path.starts_with("backup.schedule")
        && (command_path.ends_with(".disable") || command_path.ends_with(".status")))
        || command_path == "backup.schedule"
    {
        vec!["systemctl --version env=[] timeout=None".into()]
    } else if command_path == "backup.setup.dependencies" {
        vec![
            "which restic env=[] timeout=None".into(),
            "sh -c curl -fsSL https://github.com/restic/restic/releases/download/v0.16.4/restic_0.16.4_linux_amd64.bz2 | bunzip2 > <home>/.local/bin/restic && chmod +x <home>/.local/bin/restic env=[] timeout=None".into(),
            "which rclone env=[] timeout=None".into(),
            "sh -c curl -fsSL https://downloads.rclone.org/rclone-current-linux-amd64.zip -o /tmp/rclone.zip && unzip -q /tmp/rclone.zip -d /tmp && cp /tmp/rclone-*-linux-amd64/rclone <home>/.local/bin/rclone && chmod +x <home>/.local/bin/rclone && rm -rf /tmp/rclone* env=[] timeout=None".into(),
            "which resticprofile env=[] timeout=None".into(),
            "sh -c curl -fsSL https://github.com/creativeprojects/resticprofile/releases/download/v0.28.0/resticprofile_0.28.0_linux_amd64.tar.gz -o /tmp/rp.tar.gz && tar -xzf /tmp/rp.tar.gz -C /tmp && cp /tmp/resticprofile <home>/.local/bin/resticprofile && chmod +x <home>/.local/bin/resticprofile && rm -rf /tmp/rp* env=[] timeout=None".into(),
        ]
    } else if command_path == "backup.uninstall"
        && option_axis.is_some_and(|axis| axis.as_str() == "backup.uninstall.yes")
    {
        vec!["resticprofile --config <profiles> unschedule --all env=[] timeout=None".into()]
    } else {
        Vec::new()
    };
    backup::cli::ContractExpectation {
        exit_status: i32::from(!succeeds),
        stdout: if command_path == "backup.doctor" {
            "Checking dependencies...".into()
        } else if succeeds {
            "backup ".into()
        } else {
            String::new()
        },
        stderr: if command_path == "backup.doctor" {
            "doctor reported".into()
        } else if succeeds {
            String::new()
        } else {
            "failed at execution".into()
        },
        artifact_kinds: Vec::new(),
        external_state_changes: Vec::new(),
        adapter_trace,
    }
}

fn command_path_args(path: &str) -> Vec<String> {
    let mut args = path.split('.').map(str::to_owned).collect::<Vec<_>>();
    if args == ["backup"] {
        args.push("version".into());
    } else if args == ["backup", "setup"] {
        args.push("--non-interactive".into());
    } else if args.last().is_some_and(|value| value == "schedule") {
        args.push("status".into());
    }
    args
}

fn option_case(axis: &str) -> (&'static str, Vec<String>, Vec<String>) {
    let option = axis.rsplit('.').next().unwrap_or_default();
    let flag = format!("--{}", option.replace('_', "-"));
    match option {
        "dry_run"
        | "force"
        | "non_interactive"
        | "purge"
        | "quiet"
        | "skip_database"
        | "skip_retention"
        | "skip_secondary_sync"
        | "yes" => ("flag-enabled", Vec::new(), vec![flag]),
        "verbose" => ("flag-enabled", vec!["2".into()], vec!["-vv".into()]),
        "format" => ("enum-value", vec!["json".into()], vec![flag, "json".into()]),
        "lang" => ("enum-value", vec!["en".into()], vec![flag, "en".into()]),
        "profile" => (
            "enum-value",
            vec!["default".into()],
            vec![flag, "default".into()],
        ),
        "profiles" => (
            "path-or-text-value",
            vec!["/tmp/contract/profiles.yaml".into()],
            vec![flag, "/tmp/contract/profiles.yaml".into()],
        ),
        "storage" => (
            "enum-value",
            vec!["primary".into()],
            vec![flag, "primary".into()],
        ),
        "file" | "log_file" | "target" => (
            "path-or-text-value",
            vec!["/tmp/contract/output".into()],
            vec![flag, "/tmp/contract/output".into()],
        ),
        "snapshot" => (
            "path-or-text-value",
            vec!["snapshot-id".into()],
            vec![flag, "snapshot-id".into()],
        ),
        _ => (
            "value-supplied",
            vec!["value".into()],
            vec![flag, "value".into()],
        ),
    }
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
fn runtime_context_captures_environment_values_once_for_adapters() {
    let cli = Cli::try_parse_from(["backup", "schedule", "status"]).unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap()
    .with_environment("/tmp/backup-home", "contract-host", "*-*-* *:*:00");

    assert_eq!(context.home_dir, PathBuf::from("/tmp/backup-home"));
    assert_eq!(context.host_name, "contract-host");
    assert_eq!(context.scheduler_calendar, "*-*-* *:*:00");
}

#[test]
fn unified_report_config_is_derived_from_profiles_without_legacy_operational_config() {
    let temp = tempfile::tempdir().unwrap();
    let password = temp.path().join("primary-password");
    std::fs::write(&password, "report-password").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&password, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let profiles_path = temp.path().join("profiles.yaml");
    std::fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: /tmp/reports\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\n  audit:\n    system-manager: operator\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: {}\n  default:\n    backup:\n      exclude: ['/parent/cache']\n    retention:\n      keep-weekly: 5\n      keep-monthly: 13\n  files:\n    inherit: default\n    repository: /tmp/repo\n    backup:\n      source: ['/work/source']\n    retention:\n      keep-daily: 9\n",
            password.display()
        ),
    )
    .unwrap();
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_path).unwrap();

    let config = ReportConfig::from_profiles(&profiles, &profiles_path).unwrap();
    assert_eq!(config.output_dir, PathBuf::from("/tmp/reports"));
    assert_eq!(config.targets, vec!["/work/source"]);
    assert_eq!(config.excludes, vec!["/parent/cache"]);
    assert_eq!(config.retention.keep_daily, 9);
    assert_eq!(config.retention.keep_weekly, 5);
    assert_eq!(config.retention.keep_monthly, 13);
    assert_eq!(config.primary_repository, "/tmp/repo");
    assert_eq!(
        secrecy::ExposeSecret::expose_secret(&config.primary_password),
        "report-password"
    );
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
fn shared_dispatch_preserves_run_failure_report_artifacts_and_state() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    let reports = temp.path().join("reports");
    std::fs::write(
        &profiles,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: false\n    enableAnnualDrDrillReport: false\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: /tmp/password\n  default:\n    inherit: primary\n    backup:\n      source: ['/tmp']\n",
            reports.display()
        ),
    )
    .unwrap();
    backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();

    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "resticprofile",
        [
            "--config",
            profiles.to_str().unwrap(),
            "--name",
            "default",
            "backup",
        ],
        &[],
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "backend failed".into(),
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
    let cli =
        Cli::try_parse_from(["backup", "--profiles", profiles.to_str().unwrap(), "run"]).unwrap();
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
    assert!(outcome.stderr.contains("primary backup"), "{}", outcome.stderr);
    assert_eq!(outcome.artifacts.len(), 1);
    assert_eq!(
        outcome.external_state_changes,
        vec!["stage 'primary backup' attempted"]
    );
    assert!(outcome.artifacts[0].path.is_file());
    let report = std::fs::read_to_string(&outcome.artifacts[0].path).unwrap();
    assert!(report.contains("\"succeeded\": false"));
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
