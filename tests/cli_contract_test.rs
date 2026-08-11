use anyhow::Result;
use assert_cmd::Command;
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
use backup::runner::restic::{ResticRunner, ResticTool};
use backup::runner::resticprofile::{ResticProfileRunner, ResticProfileTool};
use backup::runner::scheduler::{BackupScheduler, SystemScheduler};
use clap::{CommandFactory, Parser};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
fn value_option_missing_arguments_are_rejected_by_the_authoritative_parser() {
    for axis in CONTRACT_OPTION_AXES.iter().filter(|axis| {
        matches!(
            axis.rsplit('.').next().unwrap_or_default(),
            "file"
                | "format"
                | "lang"
                | "log_file"
                | "profile"
                | "profiles"
                | "snapshot"
                | "storage"
                | "target"
        )
    }) {
        let command_path = axis.rsplit_once('.').map(|(path, _)| path).unwrap_or(axis);
        let flag = format!("--{}", axis.rsplit('.').next().unwrap().replace('_', "-"));
        let mut argv = command_path_args(command_path);
        argv.push(flag);
        assert!(
            Cli::try_parse_from(&argv).is_err(),
            "{axis} accepted a missing value"
        );
    }
}

#[test]
fn option_behavior_classes_cover_empty_whitespace_unicode_and_invalid_values() {
    for axis in CONTRACT_OPTION_AXES {
        let option = axis.rsplit('.').next().unwrap_or_default();
        let command_path = axis.rsplit_once('.').map(|(path, _)| path).unwrap_or(axis);
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
            | "verbose"
            | "yes" => {}
            "format" | "storage" => {
                let mut invalid = command_path_args(command_path);
                invalid.extend([flag.clone(), "invalid".into()]);
                assert!(
                    Cli::try_parse_from(&invalid).is_err(),
                    "{axis} accepted an invalid format value"
                );
            }
            "lang" => {
                let mut invalid = command_path_args(command_path);
                invalid.extend([flag.clone(), "invalid".into()]);
                let cli = Cli::try_parse_from(&invalid).unwrap();
                assert!(
                    CliRuntimeContext::from_cli(
                        &cli,
                        Language::En,
                        None,
                        SchedulerMode::Auto,
                        AdapterSelection::StrictTest,
                    )
                    .is_err(),
                    "{axis} accepted an invalid language value"
                );
            }
            _ => {
                for value in ["", " ", "유니코드-값"] {
                    let mut argv = command_path_args(command_path);
                    argv.extend([flag.clone(), value.into()]);
                    let parsed = Cli::try_parse_from(&argv);
                    if value.is_empty() && matches!(option, "file" | "log_file" | "profiles") {
                        assert!(parsed.is_err(), "{axis} accepted an empty path");
                    } else {
                        parsed.unwrap_or_else(|error| panic!("{axis} rejected {value:?}: {error}"));
                    }
                }
            }
        }
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
            write_mode_600(&context.profiles_path, "version: '2'\nprofiles: {}\n");
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
    let default_specs = CONTRACT_OPTION_AXES
        .iter()
        .filter(|option_axis| **option_axis != "backup.setup.non_interactive")
        .map(|option_axis| {
            let option_axis = (*option_axis).to_owned();
            let command_path = option_axis
                .rsplit_once('.')
                .map(|(path, _)| path.to_owned())
                .unwrap_or_else(|| option_axis.clone());
            ContractCaseSpec {
                command_path: command_path.clone(),
                option_axis: Some(option_axis),
                behavior_class: "default-omitted".into(),
                values: vec!["absent".into()],
                argv: command_path_args(&command_path),
                expectation: contract_expectation(&command_path, None),
            }
        });
    specs.extend(default_specs);
    specs
}

fn contract_expectation(
    command_path: &str,
    option_axis: Option<&String>,
) -> backup::cli::ContractExpectation {
    let succeeds = command_path == "backup" || command_path == "backup.version";
    let adapter_trace = if command_path == "backup.doctor" {
        Vec::new()
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

#[test]
fn binary_contract_matrix_covers_every_command_and_option_case() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    let invalid_profiles = temp.path().join("invalid-profiles.yaml");
    write_mode_600(
        &profiles,
        "version: '2'\nprofiles:\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  default:\n    inherit: primary\n    backup:\n      source: ['/tmp']\n      tag: ['backup-profile:default']\n",
    );
    write_mode_600(&temp.path().join("primary-password"), "primary-secret");
    write_mode_600(&invalid_profiles, "version: [invalid\n");

    let cases = generate_cli_contract_matrix_with_specs(contract_matrix_specs()).unwrap();
    for case in &cases {
        let mut args = case.argv.iter().skip(1).cloned().collect::<Vec<_>>();
        if let Some(index) = args.iter().position(|argument| argument == "--profiles") {
            args[index + 1] = profiles.to_string_lossy().into_owned();
        } else {
            args.splice(
                0..0,
                [
                    "--profiles".to_string(),
                    profiles.to_string_lossy().into_owned(),
                ],
            );
        }
        args.push("--help".into());

        let output = Command::cargo_bin("backup")
            .unwrap()
            .args(&args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} rejected by the binary: {}",
            case.id,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.stdout.is_empty(),
            "{} did not emit help output",
            case.id
        );
    }

    // Help above proves parser wiring for every matrix case.  For commands whose first
    // operation is configuration loading, run the real binary through dispatch as well.  An
    // explicitly supplied invalid fixture makes this deterministic and prevents any external
    // adapter from being reached while still checking main's exit/error wiring.
    let dispatch_cases = cases.iter().filter(|case| {
        matches!(
            case.command_path.as_str(),
            "backup"
                | "backup.copy"
                | "backup.database"
                | "backup.report"
                | "backup.report.environment"
                | "backup.report.restore-drill"
                | "backup.report.time-sync"
                | "backup.restore"
                | "backup.run"
                | "backup.schedule.enable"
                | "backup.setup"
                | "backup.setup.backend-init"
                | "backup.snapshots"
                | "backup.status"
                | "backup.version"
        )
    });
    for case in dispatch_cases {
        let mut args = case.argv.iter().skip(1).cloned().collect::<Vec<_>>();
        if let Some(index) = args.iter().position(|argument| argument == "--profiles") {
            args[index + 1] = invalid_profiles.to_string_lossy().into_owned();
        } else {
            args.splice(
                0..0,
                [
                    "--profiles".to_string(),
                    invalid_profiles.to_string_lossy().into_owned(),
                ],
            );
        }
        let output = Command::cargo_bin("backup")
            .unwrap()
            .args(&args)
            .output()
            .unwrap();
        let is_version = matches!(case.command_path.as_str(), "backup" | "backup.version");
        assert_eq!(
            output.status.success(),
            is_version,
            "{} dispatch status mismatch: {}",
            case.id,
            String::from_utf8_lossy(&output.stderr)
        );
        if is_version {
            assert!(
                !output.stdout.is_empty(),
                "{} emitted no version data",
                case.id
            );
        } else {
            assert!(
                !output.stderr.is_empty(),
                "{} emitted no diagnostic",
                case.id
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn binary_contract_reaches_successful_dispatch_and_report_with_explicit_fixture() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    set_mode_700(temp.path());
    let bin_dir = temp.path().join("bin");
    let reports = temp.path().join("reports");
    std::fs::create_dir(&bin_dir).unwrap();
    let resticprofile = bin_dir.join("resticprofile");
    std::fs::write(
        &resticprofile,
        "#!/bin/sh\nprintf '%s\\n' 'snapshot binary-contract saved'\n",
    )
    .unwrap();
    std::fs::set_permissions(&resticprofile, std::fs::Permissions::from_mode(0o700)).unwrap();

    let profiles = temp.path().join("profiles.yaml");
    write_mode_600(
        &profiles,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: false\n    enableAnnualDrDrillReport: false\nprofiles:\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  default:\n    inherit: primary\n    backup:\n      source: ['/tmp']\n      tag: ['backup-profile:default']\n",
            reports.display()
        ),
    );
    write_mode_600(&temp.path().join("primary-password"), "primary-secret");
    let path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin_dir.display(), path.to_string_lossy());

    let output = Command::cargo_bin("backup")
        .unwrap()
        .env("PATH", path)
        .args([
            "--profiles",
            profiles.to_str().unwrap(),
            "run",
            "--skip-database",
            "--skip-secondary-sync",
            "--skip-retention",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("binary-contract"));
    let report = std::fs::read_dir(&reports)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("binary run should create an execution report");
    assert!(
        std::fs::read_to_string(&report)
            .unwrap()
            .contains("\"succeeded\": true")
    );
    assert_eq!(
        std::fs::metadata(report).unwrap().permissions().mode() & 0o777,
        0o600
    );
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
    write_mode_600(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: /tmp/reports\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\n  audit:\n    system-manager: operator\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: {}\n  default:\n    backup:\n      exclude: ['/parent/cache']\n    retention:\n      keep-weekly: 5\n      keep-monthly: 13\n  files:\n    inherit: default\n    repository: /tmp/repo\n    backup:\n      source: ['/work/source']\n    retention:\n      keep-daily: 9\n",
            password.display()
        ),
    );
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

    let mysql_runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "mysqldump",
        ["--host=db"],
        &[("MYSQL_PWD", "expected-db-secret")],
        CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        },
    )]);
    let mysql_error = mysql_runner
        .run_with_env(
            "mysqldump",
            &["--host=db"],
            &[("MYSQL_PWD", "actual-db-secret")],
        )
        .unwrap_err()
        .to_string();
    assert!(!mysql_error.contains("expected-db-secret"));
    assert!(!mysql_error.contains("actual-db-secret"));
    assert!(mysql_error.contains("MYSQL_PWD=<redacted>"));
}

#[test]
fn shared_dispatch_reaches_the_strict_adapter_with_exact_profile_and_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    write_mode_600(
        &profiles,
        "version: '2'\nprofiles:\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  secondary:\n    repository: /secondary-repository\n    password-file: secondary-password\n  default:\n    inherit: primary\n    backup:\n      source: ['/tmp']\n      tag: ['backup-profile:default']\n    copy:\n      profile: secondary\n",
    );
    write_mode_600(&temp.path().join("primary-password"), "primary-secret");
    write_mode_600(&temp.path().join("secondary-password"), "secondary-secret");
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
    write_mode_600(
        &profiles,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: false\n    enableAnnualDrDrillReport: false\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: /tmp/password\n  default:\n    inherit: primary\n    backup:\n      source: ['/tmp']\n      tag: ['backup-profile:default']\n",
            reports.display()
        ),
    );
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
    assert!(
        outcome.stderr.contains("primary backup"),
        "{}",
        outcome.stderr
    );
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
    write_mode_600(
        &profiles,
        "version: '2'\nprofiles:\n  default:\n    repository: /tmp/repo\n    backup:\n      source: ['/tmp']\n",
    );
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

#[test]
fn run_contract_resolves_all_runnable_profiles_or_one_exact_profile() {
    let fixture = run_contract_fixture();

    let (outcome, trace) = dispatch_run_contract(
        fixture.profiles_path(),
        &[],
        expected_full_pipeline(false, false, false, false),
    );
    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(
        trace,
        vec![
            "database",
            "primary:alpha:dry=false",
            "primary:beta:dry=false",
            "secondary:alpha:dry=false",
            "secondary:beta:dry=false",
            "retention:alpha",
            "retention:beta",
        ]
    );

    let (outcome, trace) = dispatch_run_contract(
        fixture.profiles_path(),
        &["--profile", "alpha"],
        RunContractExpectations {
            profile_calls: vec![
                profile_call(
                    "primary",
                    "alpha",
                    Some(false),
                    "snapshot alpha-id saved\n",
                    None,
                ),
                profile_call(
                    "secondary",
                    "alpha",
                    Some(false),
                    "copy alpha complete\n",
                    None,
                ),
                profile_call(
                    "retention",
                    "alpha",
                    None,
                    "retention alpha complete\n",
                    None,
                ),
            ],
            database_call: None,
        },
    );
    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(
        trace,
        vec![
            "primary:alpha:dry=false",
            "secondary:alpha:dry=false",
            "retention:alpha",
        ]
    );
    assert!(outcome.stdout.contains("Execution report:"));
}

#[test]
fn run_contract_executes_database_profile_once_and_excludes_it_from_file_stages() {
    let fixture = run_contract_fixture();
    let (outcome, trace) = dispatch_run_contract(
        fixture.profiles_path(),
        &["--profile", "database"],
        RunContractExpectations {
            profile_calls: Vec::new(),
            database_call: Some(database_call_expectation()),
        },
    );

    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(trace, vec!["database"]);
    assert!(!outcome.stdout.contains("primary:alpha"));
    assert!(!outcome.stdout.contains("secondary:alpha"));
}

#[test]
fn run_contract_covers_every_skip_combination_without_changing_stage_order() {
    for skip_database in [false, true] {
        for skip_secondary in [false, true] {
            for skip_retention in [false, true] {
                let fixture = run_contract_fixture();
                let mut args = Vec::new();
                if skip_database {
                    args.push("--skip-database");
                }
                if skip_secondary {
                    args.push("--skip-secondary-sync");
                }
                if skip_retention {
                    args.push("--skip-retention");
                }
                let (outcome, trace) = dispatch_run_contract(
                    fixture.profiles_path(),
                    &args,
                    expected_full_pipeline(false, skip_database, skip_secondary, skip_retention),
                );

                let mut expected: Vec<String> = Vec::new();
                if !skip_database {
                    expected.push("database".into());
                }
                expected.extend([
                    "primary:alpha:dry=false".into(),
                    "primary:beta:dry=false".into(),
                ]);
                if !skip_secondary {
                    expected.extend([
                        "secondary:alpha:dry=false".into(),
                        "secondary:beta:dry=false".into(),
                    ]);
                }
                if !skip_retention {
                    expected.extend(["retention:alpha".into(), "retention:beta".into()]);
                }
                let case_id = format!(
                    "run.skip-db={skip_database}.skip-secondary={skip_secondary}.skip-retention={skip_retention}"
                );
                let diagnostic = ContractDiagnostic::from_outcome(
                    case_id,
                    format!(
                        "argv={args:?} profiles={}",
                        fixture.profiles_path().display()
                    ),
                    expected.clone(),
                    trace.clone(),
                    &outcome,
                )
                .render();
                assert!(outcome.is_success(), "{diagnostic}");
                assert_eq!(trace, expected, "{diagnostic}");
                assert!(outcome.stdout.contains("Execution report:"), "{diagnostic}");
                let report = std::fs::read_to_string(&outcome.artifacts[0].path).unwrap();
                assert_eq!(
                    report.contains("database stream complete"),
                    !skip_database,
                    "{diagnostic}"
                );
                assert_eq!(
                    report.contains("copy alpha complete") && report.contains("copy beta complete"),
                    !skip_secondary,
                    "{diagnostic}"
                );
                assert_eq!(
                    report.contains("retention alpha complete")
                        && report.contains("retention beta complete"),
                    !skip_retention,
                    "{diagnostic}"
                );
            }
        }
    }
}

#[test]
fn run_contract_covers_profile_selection_and_flag_matrix() {
    for selection in [
        RunSelection::All,
        RunSelection::Alpha,
        RunSelection::Database,
    ] {
        for skip_database in [false, true] {
            for skip_secondary in [false, true] {
                for skip_retention in [false, true] {
                    for dry_run in [false, true] {
                        let fixture = run_contract_fixture();
                        let mut args = match selection {
                            RunSelection::All => Vec::new(),
                            RunSelection::Alpha => vec!["--profile", "alpha"],
                            RunSelection::Database => vec!["--profile", "database"],
                        };
                        if skip_database {
                            args.push("--skip-database");
                        }
                        if skip_secondary {
                            args.push("--skip-secondary-sync");
                        }
                        if skip_retention {
                            args.push("--skip-retention");
                        }
                        if dry_run {
                            args.push("--dry-run");
                        }
                        let (outcome, trace) = dispatch_run_contract(
                            fixture.profiles_path(),
                            &args,
                            expected_pipeline(
                                selection,
                                dry_run,
                                skip_database,
                                skip_secondary,
                                skip_retention,
                            ),
                        );
                        let expected_trace = expected_trace(
                            selection,
                            dry_run,
                            skip_database,
                            skip_secondary,
                            skip_retention,
                        );
                        let case_id = format!(
                            "run.selection={}.skip-db={skip_database}.skip-secondary={skip_secondary}.skip-retention={skip_retention}.dry-run={dry_run}",
                            selection.label()
                        );
                        let diagnostic = ContractDiagnostic::from_outcome(
                            case_id,
                            format!(
                                "argv={args:?} profiles={}",
                                fixture.profiles_path().display()
                            ),
                            expected_trace.clone(),
                            trace.clone(),
                            &outcome,
                        )
                        .render();
                        assert!(outcome.is_success(), "{diagnostic}");
                        assert_eq!(trace, expected_trace, "{diagnostic}");
                        assert_eq!(outcome.artifacts.len(), 1, "{diagnostic}");
                    }
                }
            }
        }
    }
}

#[test]
fn run_contract_dry_run_forwards_native_flags_and_creates_no_snapshot_or_mutation() {
    let fixture = run_contract_fixture();
    let (outcome, trace) = dispatch_run_contract(
        fixture.profiles_path(),
        &["--dry-run"],
        expected_full_pipeline(true, false, false, false),
    );

    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(
        trace,
        vec![
            "primary:alpha:dry=true",
            "primary:beta:dry=true",
            "secondary:alpha:dry=true",
            "secondary:beta:dry=true",
        ]
    );
    assert!(outcome.external_state_changes.is_empty());
    assert!(
        outcome
            .stdout
            .contains("[Dry-Run] Database Stream: pg_dump -> app.sql")
    );
    let report_path = &outcome.artifacts[0].path;
    let report = std::fs::read_to_string(report_path).unwrap();
    assert!(report.contains("\"succeeded\": true"));
    assert!(report.contains("\"snapshot_id\": null"));
    assert_eq!(file_mode(report_path), Some(0o600));
    assert_eq!(file_mode(report_path.parent().unwrap()), Some(0o700));
}

#[test]
fn run_contract_partial_failure_stops_later_stages_and_keeps_a_masked_report() {
    let fixture = run_contract_fixture();
    let mut expected = expected_full_pipeline(false, false, false, false);
    expected.profile_calls = vec![
        profile_call(
            "primary",
            "alpha",
            Some(false),
            "snapshot alpha-id saved\n",
            None,
        ),
        profile_call(
            "primary",
            "beta",
            Some(false),
            "snapshot beta-id saved\n",
            None,
        ),
        profile_call(
            "secondary",
            "alpha",
            Some(false),
            "",
            Some(
                "copy failed using primary-secret and archive-secret for postgres://backup-user:db-secret@db:5432/app",
            ),
        ),
    ];

    let (outcome, trace) = dispatch_run_contract(fixture.profiles_path(), &[], expected);
    assert_eq!(outcome.exit_status, 1);
    assert!(outcome.stdout.is_empty());
    assert!(outcome.stderr.contains("secondary sync"));
    assert_eq!(trace.len(), 4);
    assert_eq!(
        outcome.external_state_changes,
        vec!["stage 'secondary sync' attempted"]
    );
    assert_eq!(outcome.artifacts.len(), 1);
    let report_path = &outcome.artifacts[0].path;
    let report = std::fs::read_to_string(report_path).unwrap();
    assert!(report.contains("\"succeeded\": false"));
    assert!(report.contains("\"failure_stage\": \"secondary sync\""));
    assert!(!report.contains("primary-secret"));
    assert!(!report.contains("archive-secret"));
    assert!(!report.contains("postgres://backup-user:db-secret@db:5432/app"));
    assert_eq!(file_mode(report_path), Some(0o600));
}

#[test]
fn copy_and_sync_contract_share_exact_profile_and_native_dry_run_behavior() {
    for alias in ["copy", "sync"] {
        for dry_run in [false, true] {
            let fixture = copy_contract_fixture(true);
            let mut argv = vec![
                "backup".to_string(),
                "--profiles".to_string(),
                fixture.profiles.to_string_lossy().into_owned(),
                alias.to_string(),
                "--profile".to_string(),
                "default".to_string(),
            ];
            if dry_run {
                argv.push("--dry-run".into());
            }

            let native_args = if dry_run {
                vec![
                    "--config",
                    fixture.profiles.to_str().unwrap(),
                    "--name",
                    "default",
                    "--dry-run",
                    "copy",
                ]
            } else {
                vec![
                    "--config",
                    fixture.profiles.to_str().unwrap(),
                    "--name",
                    "default",
                    "copy",
                ]
            };
            let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
                "resticprofile",
                native_args,
                &[],
                CommandOutput {
                    status_code: 0,
                    stdout: if dry_run {
                        "copy planned\n".into()
                    } else {
                        "copy completed\n".into()
                    },
                    stderr: String::new(),
                },
            )]);
            let resticprofile = ResticProfileTool::new(&runner);
            let restic = ResticTool::new(&runner);
            let rclone = RcloneTool::new(&runner);
            let scheduler = SystemScheduler::new(&runner, "backup");
            let cli = Cli::try_parse_from(argv).unwrap();
            let context = CliRuntimeContext::from_cli(
                &cli,
                Language::En,
                None,
                SchedulerMode::Auto,
                AdapterSelection::StrictTest,
            )
            .unwrap();
            let adapters = AdapterSet {
                command: &runner,
                rclone: &rclone,
                restic: &restic,
                resticprofile: &resticprofile,
                scheduler: &scheduler,
                selection: AdapterSelection::StrictTest,
            };

            let outcome = dispatch(&context, cli.command, &adapters);
            assert!(outcome.is_success(), "{}", outcome.stderr);
            assert!(outcome.stdout.contains(if dry_run {
                "copy planned"
            } else {
                "copy completed"
            }));
            assert_eq!(
                outcome.external_state_changes,
                if dry_run {
                    Vec::<String>::new()
                } else {
                    vec!["snapshots copied".into()]
                },
                "alias={alias} dry_run={dry_run}"
            );
            runner.assert_exhausted().unwrap();
        }
    }
}

#[test]
fn copy_contract_rejects_invalid_profiles_and_missing_secondary_before_adapter_calls() {
    for invalid_profile in ["unknown", "", " ", " default "] {
        let fixture = copy_contract_fixture(true);
        let runner = StrictCommandRunner::new([]);
        let resticprofile = ResticProfileTool::new(&runner);
        let restic = ResticTool::new(&runner);
        let rclone = RcloneTool::new(&runner);
        let scheduler = SystemScheduler::new(&runner, "backup");
        let cli = Cli::try_parse_from([
            "backup",
            "--profiles",
            fixture.profiles.to_str().unwrap(),
            "copy",
            "--profile",
            invalid_profile,
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
        let adapters = AdapterSet {
            command: &runner,
            rclone: &rclone,
            restic: &restic,
            resticprofile: &resticprofile,
            scheduler: &scheduler,
            selection: AdapterSelection::StrictTest,
        };

        let outcome = dispatch(&context, cli.command, &adapters);
        assert_eq!(outcome.exit_status, 1, "profile={invalid_profile:?}");
        runner.assert_exhausted().unwrap();
    }

    let fixture = copy_contract_fixture(false);
    let runner = StrictCommandRunner::new([]);
    let resticprofile = ResticProfileTool::new(&runner);
    let restic = ResticTool::new(&runner);
    let rclone = RcloneTool::new(&runner);
    let scheduler = SystemScheduler::new(&runner, "backup");
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        fixture.profiles.to_str().unwrap(),
        "copy",
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
    let adapters = AdapterSet {
        command: &runner,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &resticprofile,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };

    let outcome = dispatch(&context, cli.command, &adapters);
    assert_eq!(outcome.exit_status, 1);
    assert!(
        runner.calls().is_empty(),
        "missing secondary must fail before copy"
    );
    runner.assert_exhausted().unwrap();
}

#[test]
fn database_contract_runs_one_stream_and_dry_run_performs_no_external_call() {
    let fixture = lifecycle_fixture(true);

    let runner = LifecycleResticAdapter::new(false, false);
    let outcome = dispatch_lifecycle(&fixture.profiles, ["database"], &runner);
    assert!(outcome.is_success(), "{}", outcome.stderr);
    let database_call = &runner.database_calls()[0];
    assert_eq!(database_call.repository, "/primary-repository");
    assert_eq!(database_call.filename, "app.sql");
    assert_eq!(database_call.program, "pg_dump");
    assert_eq!(
        database_call.tag.as_deref(),
        Some("backup-profile:database")
    );
    assert_eq!(
        database_call.args,
        vec![
            "--host=db",
            "--username=backup-user",
            "--dbname=app",
            "--port=5432"
        ]
    );
    assert_eq!(
        database_call.environment,
        vec![("PGPASSWORD".into(), "db-secret".into())]
    );
    assert_eq!(runner.restore_calls().len(), 0);
    assert!(outcome.stdout.contains("database stream complete"));

    let dry_runner = LifecycleResticAdapter::new(false, false);
    let dry_outcome = dispatch_lifecycle(&fixture.profiles, ["database", "--dry-run"], &dry_runner);
    assert!(dry_outcome.is_success(), "{}", dry_outcome.stderr);
    assert!(dry_runner.database_calls().is_empty());
    assert!(!dry_outcome.stdout.contains("postgres://"));
    assert!(!dry_outcome.stdout.contains("db-secret"));
    assert!(!dry_outcome.stdout.contains("backup-user"));
}

#[test]
fn database_contract_rejects_invalid_credentials_before_the_backup_adapter() {
    let fixture = lifecycle_fixture(true);
    std::fs::write(
        fixture.directory.path().join("database-connection-url"),
        "postgres://backup-user@db:5432/app",
    )
    .unwrap();
    set_mode_600(&fixture.directory.path().join("database-connection-url"));

    let runner = LifecycleResticAdapter::new(false, false);
    let outcome = dispatch_lifecycle(&fixture.profiles, ["database"], &runner);
    assert_eq!(outcome.exit_status, 1);
    assert!(runner.database_calls().is_empty());
}

#[test]
fn database_dry_run_rejects_an_unresolved_backend_before_rendering() {
    let fixture = lifecycle_fixture(true);
    let profiles = std::fs::read_to_string(&fixture.profiles).unwrap().replace(
        "  database:\n    inherit: primary\n",
        "  database:\n    inherit: primary\n    password-file: missing-password\n",
    );
    std::fs::write(&fixture.profiles, profiles).unwrap();

    let runner = LifecycleResticAdapter::new(false, false);
    let outcome = dispatch_lifecycle(&fixture.profiles, ["database", "--dry-run"], &runner);

    assert_eq!(outcome.exit_status, 1);
    assert!(!outcome.stderr.is_empty());
    assert!(runner.database_calls().is_empty());
}

#[test]
fn restore_contract_selects_one_storage_and_forwards_snapshot_behavior_classes() {
    for (storage, repository) in [
        ("primary", "/primary-repository"),
        ("secondary", "/secondary-repository"),
    ] {
        for snapshot in ["latest", "abcdef12", "abcdef"] {
            let fixture = lifecycle_fixture(false);
            let target = fixture
                .directory
                .path()
                .join(format!("restore-{storage}-{snapshot}"));
            let runner = LifecycleResticAdapter::new(false, false);
            let outcome = dispatch_lifecycle(
                &fixture.profiles,
                [
                    "restore",
                    "--storage",
                    storage,
                    "--snapshot",
                    snapshot,
                    "--target",
                    target.to_str().unwrap(),
                ],
                &runner,
            );
            assert!(outcome.is_success(), "{}", outcome.stderr);
            let calls = runner.restore_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].repository, repository);
            assert_eq!(calls[0].snapshot, snapshot);
            assert_eq!(calls[0].target, target.to_string_lossy());
        }
    }
}

#[test]
fn restore_contract_distinguishes_invalid_snapshot_values_from_native_resolution_failures() {
    let fixture = lifecycle_fixture(false);
    let target = fixture.directory.path().join("restore-failure");
    for snapshot in ["missing-id", "ambiguous-prefix"] {
        let runner = LifecycleResticAdapter::new(false, true);
        let outcome = dispatch_lifecycle(
            &fixture.profiles,
            [
                "restore",
                "--snapshot",
                snapshot,
                "--storage",
                "secondary",
                "--target",
                target.to_str().unwrap(),
            ],
            &runner,
        );
        assert_eq!(outcome.exit_status, 1);
        assert_eq!(runner.restore_calls().len(), 1);
    }

    for snapshot in ["", " ", " latest "] {
        let runner = LifecycleResticAdapter::new(false, false);
        let outcome = dispatch_lifecycle(
            &fixture.profiles,
            [
                "restore",
                "--snapshot",
                snapshot,
                "--target",
                target.to_str().unwrap(),
            ],
            &runner,
        );
        assert_eq!(outcome.exit_status, 1, "snapshot={snapshot:?}");
        assert!(runner.restore_calls().is_empty(), "snapshot={snapshot:?}");
    }
}

#[cfg(unix)]
#[test]
fn restore_contract_rejects_non_writable_missing_target_parent_before_adapter_call() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = lifecycle_fixture(false);
    let parent = fixture.directory.path().join("read-only-parent");
    std::fs::create_dir(&parent).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555)).unwrap();
    let target = parent.join("restore");
    let runner = LifecycleResticAdapter::new(false, false);
    let outcome = dispatch_lifecycle(
        &fixture.profiles,
        ["restore", "--target", target.to_str().unwrap()],
        &runner,
    );
    assert_eq!(outcome.exit_status, 1);
    assert!(runner.restore_calls().is_empty());
}

#[test]
fn restore_contract_rejects_invalid_targets_before_the_restore_adapter() {
    let fixture = lifecycle_fixture(false);
    let existing_file = fixture.directory.path().join("existing-file");
    std::fs::write(&existing_file, "not a directory").unwrap();
    let non_empty = fixture.directory.path().join("non-empty");
    std::fs::create_dir(&non_empty).unwrap();
    std::fs::write(non_empty.join("existing.txt"), "keep me").unwrap();
    let missing_parent = fixture.directory.path().join("missing-parent/restore");
    let mut invalid_targets = vec![
        ("missing target parent", missing_parent),
        ("regular file", existing_file),
        ("non-empty without force", non_empty),
    ];
    #[cfg(unix)]
    {
        let symlink_target = fixture.directory.path().join("symlink-target");
        std::fs::create_dir(&symlink_target).unwrap();
        let symlink = fixture.directory.path().join("restore-symlink");
        std::os::unix::fs::symlink(&symlink_target, &symlink).unwrap();
        invalid_targets.push(("symlink", symlink));
    }

    for (case, target) in invalid_targets {
        let runner = LifecycleResticAdapter::new(false, false);
        let outcome = dispatch_lifecycle(
            &fixture.profiles,
            ["restore", "--target", target.to_str().unwrap()],
            &runner,
        );
        assert_eq!(outcome.exit_status, 1, "case={case}: {}", outcome.stderr);
        assert!(runner.restore_calls().is_empty(), "case={case}");
    }
}

#[test]
fn restore_force_preserves_existing_target_and_sql_validation_controls_status() {
    let fixture = lifecycle_fixture(false);
    let target = fixture.directory.path().join("force-target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("existing.txt"), "keep me").unwrap();

    let runner = LifecycleResticAdapter::new(false, false);
    let outcome = dispatch_lifecycle(
        &fixture.profiles,
        ["restore", "--force", "--target", target.to_str().unwrap()],
        &runner,
    );
    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(
        std::fs::read_to_string(target.join("existing.txt")).unwrap(),
        "keep me"
    );

    let database_fixture = lifecycle_fixture(true);
    let database_target = database_fixture.directory.path().join("invalid-sql");
    let invalid_sql_runner = LifecycleResticAdapter::new(true, false);
    let invalid_sql_outcome = dispatch_lifecycle(
        &database_fixture.profiles,
        ["restore", "--target", database_target.to_str().unwrap()],
        &invalid_sql_runner,
    );
    assert_eq!(invalid_sql_outcome.exit_status, 0);
    assert_eq!(invalid_sql_runner.restore_calls().len(), 1);
}

#[test]
fn setup_backend_init_contract_attempts_every_target_in_deterministic_order() {
    let fixture = setup_contract_fixture();
    let before = std::fs::read(&fixture.profiles).unwrap();
    let runner = SetupTraceAdapter::new(Some("alpha"));
    let outcome = dispatch_setup(&fixture.profiles, ["setup", "backend-init"], &runner);

    assert_eq!(outcome.exit_status, 1);
    assert!(outcome.stderr.contains("alpha"));
    assert_eq!(
        runner.calls(),
        vec!["primary", "alpha", "zeta", "secondary"]
    );
    assert_eq!(std::fs::read(&fixture.profiles).unwrap(), before);
}

#[test]
fn setup_backend_init_promotes_a_retryable_pending_configuration_after_success() {
    let fixture = setup_contract_fixture();
    let pending_dir = backup::commands::setup::pending_setup_dir(&fixture.profiles);
    backup::config::model::create_secure_dir(&pending_dir).unwrap();
    let pending_password = pending_dir.join("primary-password");
    write_mode_600(&pending_password, "pending-secret");
    write_mode_600(
        &pending_dir.join("profiles.yaml"),
        &format!(
            "version: '2'\nprofiles:\n  primary:\n    repository: /pending-repository\n    password-file: {}\n  default: {{}}\n",
            pending_password.display()
        ),
    );

    let runner = SetupTraceAdapter::new(None);
    let outcome = dispatch_setup(&fixture.profiles, ["setup", "backend-init"], &runner);

    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert_eq!(runner.calls(), vec!["primary"]);
    assert!(!pending_dir.exists());
    let promoted = std::fs::read_to_string(&fixture.profiles).unwrap();
    assert!(promoted.contains("/pending-repository"));
    assert_eq!(
        std::fs::read_to_string(fixture.profiles.parent().unwrap().join("primary-password"))
            .unwrap(),
        "pending-secret"
    );
}

#[test]
fn setup_backend_init_keeps_pending_configuration_when_snapshots_rejects_the_repository_key() {
    let fixture = setup_contract_fixture();
    let pending_dir = backup::commands::setup::pending_setup_dir(&fixture.profiles);
    backup::config::model::create_secure_dir(&pending_dir).unwrap();
    write_mode_600(
        &pending_dir.join("profiles.yaml"),
        "version: '2'\nprofiles:\n  primary:\n    repository: /pending-repository\n    password-file: primary-password\n  default: {}\n",
    );
    write_mode_600(&pending_dir.join("primary-password"), "pending-secret");
    let runner = SetupTraceAdapter::with_snapshot_failure("No key found for configured password");

    let outcome = dispatch_setup(&fixture.profiles, ["setup", "backend-init"], &runner);

    assert_eq!(outcome.exit_status, 1);
    assert!(outcome.stderr.to_ascii_lowercase().contains("no key found"));
    assert!(pending_dir.exists());
    assert!(
        !std::fs::read_to_string(&fixture.profiles)
            .unwrap()
            .contains("/pending-repository")
    );
}

#[test]
fn setup_non_interactive_contract_initializes_existing_targets_without_prompts() {
    let fixture = setup_contract_fixture();
    let runner = SetupTraceAdapter::new(None);
    let scheduler = SetupTraceScheduler::new();
    let command_runner = StrictCommandRunner::new([]);
    let resticprofile = SetupResticProfileAdapter { inner: &runner };
    let rclone = RcloneTool::new(&command_runner);
    let restic = ResticTool::new(&command_runner);
    let system_scheduler = SetupSchedulerAdapter { inner: &scheduler };
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        fixture.profiles.to_str().unwrap(),
        "setup",
        "--non-interactive",
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
    let adapters = AdapterSet {
        command: &command_runner,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &resticprofile,
        scheduler: &system_scheduler,
        selection: AdapterSelection::StrictTest,
    };

    let outcome = dispatch(&context, cli.command, &adapters);

    assert!(outcome.is_success(), "{}", outcome.stderr);
    assert!(
        outcome
            .stdout
            .contains("Connecting to backend storage and initializing repository"),
        "setup progress notice must be part of CLI stdout: {}",
        outcome.stdout
    );
    assert_eq!(
        runner.calls(),
        vec!["primary", "alpha", "zeta", "secondary"]
    );
    assert_eq!(scheduler.enable_calls(), 1);
    command_runner.assert_exhausted().unwrap();
}

#[test]
fn setup_non_interactive_failure_preserves_progress_notice_on_stdout() {
    let fixture = setup_contract_fixture();
    let runner = SetupTraceAdapter::new(Some("alpha"));
    let outcome = dispatch_setup(&fixture.profiles, ["setup", "--non-interactive"], &runner);

    assert_eq!(outcome.exit_status, 1);
    assert!(
        outcome
            .stdout
            .contains("Connecting to backend storage and initializing repository")
    );
    assert!(outcome.stderr.contains("backend initialization"));
    assert!(!outcome.stderr.contains("message="));
}

#[test]
fn binary_structured_logs_are_written_to_the_system_sink_only() {
    let directory = tempfile::tempdir().unwrap();
    let log_file = directory.path().join("backup.log");

    let output = Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--log-file",
            log_file.to_str().unwrap(),
            "uninstall",
            "--purge",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uninstall failed at execution"));
    assert!(!stderr.contains("Executing backup CLI uninstallation"));
    assert!(!stderr.contains("message="));

    let system_log = std::fs::read_to_string(&log_file).unwrap();
    assert!(system_log.contains("Executing backup CLI uninstallation"));
}

#[derive(Debug, Clone)]
struct ProfileCallExpectation {
    operation: &'static str,
    profile: String,
    dry_run: Option<bool>,
    output: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct DatabaseCallExpectation {
    repository: String,
    password: String,
    filename: String,
    program: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    tag: Option<String>,
    output: String,
}

#[derive(Default)]
struct RunContractExpectations {
    profile_calls: Vec<ProfileCallExpectation>,
    database_call: Option<DatabaseCallExpectation>,
}

#[derive(Clone, Copy)]
enum RunSelection {
    All,
    Alpha,
    Database,
}

impl RunSelection {
    fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Alpha => "alpha",
            Self::Database => "database",
        }
    }
}

struct StrictRunProfileAdapter {
    calls: Mutex<VecDeque<ProfileCallExpectation>>,
    trace: Arc<Mutex<Vec<String>>>,
    expected_config_path: PathBuf,
}

impl StrictRunProfileAdapter {
    fn new(
        config_path: &Path,
        calls: Vec<ProfileCallExpectation>,
        trace: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            calls: Mutex::new(calls.into_iter().collect()),
            trace,
            expected_config_path: config_path.to_path_buf(),
        }
    }

    fn consume(
        &self,
        config_path: &Path,
        operation: &'static str,
        profile: &str,
        dry_run: Option<bool>,
    ) -> Result<String> {
        if config_path != self.expected_config_path {
            anyhow::bail!("unexpected profiles configuration path");
        }
        let trace_value = match dry_run {
            Some(dry_run) => format!("{operation}:{profile}:dry={dry_run}"),
            None => format!("{operation}:{profile}"),
        };
        self.trace.lock().unwrap().push(trace_value);
        let expected =
            self.calls.lock().unwrap().pop_front().ok_or_else(|| {
                anyhow::anyhow!("unexpected {operation} call for profile {profile}")
            })?;
        if expected.operation != operation
            || expected.profile != profile
            || expected.dry_run != dry_run
        {
            anyhow::bail!(
                "unexpected profile call: expected {}:{}:{:?}, got {}:{}:{:?}",
                expected.operation,
                expected.profile,
                expected.dry_run,
                operation,
                profile,
                dry_run
            );
        }
        if let Some(error) = expected.error {
            anyhow::bail!("{error}");
        }
        Ok(expected.output)
    }

    fn assert_exhausted(&self) {
        let remaining = self.calls.lock().unwrap().clone();
        assert!(
            remaining.is_empty(),
            "strict run profile adapter has unconsumed expectations: {remaining:?}; trace: {:?}",
            self.trace.lock().unwrap()
        );
    }
}

impl ResticProfileRunner for StrictRunProfileAdapter {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        self.consume(config_path, "primary", profile, Some(dry_run))
    }

    fn init(&self, _: &Path, profile: &str) -> Result<String> {
        anyhow::bail!("unexpected init call for profile {profile}")
    }

    fn schedule_enable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected schedule enable call")
    }

    fn schedule_disable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected schedule disable call")
    }

    fn schedule_status(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected schedule status call")
    }

    fn list_snapshots(&self, _: &Path, profile: &str) -> Result<String> {
        anyhow::bail!("unexpected snapshots call for profile {profile}")
    }

    fn prune(&self, config_path: &Path, profile: &str) -> Result<String> {
        self.consume(config_path, "retention", profile, None)
    }

    fn check(&self, _: &Path, profile: &str) -> Result<String> {
        anyhow::bail!("unexpected check call for profile {profile}")
    }

    fn copy(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        self.consume(config_path, "secondary", profile, Some(dry_run))
    }
}

struct StrictRunDatabaseAdapter {
    expected: Mutex<Option<DatabaseCallExpectation>>,
    trace: Arc<Mutex<Vec<String>>>,
}

impl StrictRunDatabaseAdapter {
    fn new(expected: Option<DatabaseCallExpectation>, trace: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            expected: Mutex::new(expected),
            trace,
        }
    }

    fn unexpected(operation: &str) -> Result<String> {
        anyhow::bail!("unexpected database {operation} call")
    }

    fn assert_exhausted(&self) {
        let remaining = self.expected.lock().unwrap().is_some();
        assert!(
            !remaining,
            "strict database adapter has an unconsumed expectation; trace: {:?}",
            self.trace.lock().unwrap()
        );
    }

    fn record_database_call(
        &self,
        repository: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        tag: Option<&str>,
        environment: &[(&str, &str)],
    ) -> Result<String> {
        self.trace.lock().unwrap().push("database".into());
        let expected = self
            .expected
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| anyhow::anyhow!("unexpected database backup call"))?;
        let actual_environment = environment
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect::<Vec<_>>();
        if repository != expected.repository
            || password != expected.password
            || filename != expected.filename
            || program != expected.program
            || args != expected.args
            || tag.map(str::to_owned) != expected.tag
            || actual_environment != expected.environment
        {
            anyhow::bail!("unexpected database adapter arguments or environment");
        }
        Ok(expected.output)
    }
}

impl ResticRunner for StrictRunDatabaseAdapter {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        Self::unexpected("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        Self::unexpected("backup paths")
    }

    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        Self::unexpected("list snapshots")
    }

    fn list_snapshots_with_env(&self, _: &str, _: &str, _: &[(&str, &str)]) -> Result<String> {
        Self::unexpected("list snapshots")
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        Self::unexpected("restore")
    }

    fn restore_with_env(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &str,
        _: &[(&str, &str)],
    ) -> Result<String> {
        Self::unexpected("restore")
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        Self::unexpected("backup command")
    }

    fn backup_command_with_env(
        &self,
        repository: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        environment: &[(&str, &str)],
    ) -> Result<String> {
        self.record_database_call(
            repository,
            password,
            filename,
            program,
            args,
            None,
            environment,
        )
    }

    fn backup_command_with_env_and_tag(
        &self,
        repository: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        tag: &str,
        environment: &[(&str, &str)],
    ) -> Result<String> {
        self.record_database_call(
            repository,
            password,
            filename,
            program,
            args,
            Some(tag),
            environment,
        )
    }
}

struct RunContractFixture {
    _directory: tempfile::TempDir,
    profiles: PathBuf,
}

struct CopyContractFixture {
    _directory: tempfile::TempDir,
    profiles: PathBuf,
}

struct LifecycleFixture {
    directory: tempfile::TempDir,
    profiles: PathBuf,
}

struct SetupFixture {
    _directory: tempfile::TempDir,
    profiles: PathBuf,
}

struct SetupTraceAdapter {
    calls: Mutex<Vec<String>>,
    failure: Option<String>,
    snapshot_failure: Option<String>,
}

impl SetupTraceAdapter {
    fn new(failure: Option<&str>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            failure: failure.map(Into::into),
            snapshot_failure: None,
        }
    }

    fn with_snapshot_failure(message: &str) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            failure: None,
            snapshot_failure: Some(message.into()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn init(&self, profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(profile.into());
        if self.failure.as_deref() == Some(profile) {
            anyhow::bail!("{profile} repository unavailable")
        }
        Ok(format!("{profile} initialized"))
    }

    fn snapshots(&self) -> Result<String> {
        if let Some(message) = &self.snapshot_failure {
            anyhow::bail!("{message}");
        }
        Ok("repository credentials verified".into())
    }
}

struct SetupResticProfileAdapter<'a> {
    inner: &'a SetupTraceAdapter,
}

impl ResticProfileRunner for SetupResticProfileAdapter<'_> {
    fn backup(&self, _: &Path, _: &str, _: bool) -> Result<String> {
        anyhow::bail!("unexpected setup backup call")
    }

    fn init(&self, _: &Path, profile: &str) -> Result<String> {
        self.inner.init(profile)
    }

    fn schedule_enable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected setup profile scheduler call")
    }

    fn schedule_disable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected setup profile scheduler call")
    }

    fn schedule_status(&self, _: &Path) -> Result<String> {
        anyhow::bail!("unexpected setup profile scheduler call")
    }

    fn list_snapshots(&self, _: &Path, _: &str) -> Result<String> {
        self.inner.snapshots()
    }

    fn prune(&self, _: &Path, _: &str) -> Result<String> {
        anyhow::bail!("unexpected setup prune call")
    }

    fn check(&self, _: &Path, _: &str) -> Result<String> {
        anyhow::bail!("unexpected setup check call")
    }

    fn copy(&self, _: &Path, _: &str, _: bool) -> Result<String> {
        anyhow::bail!("unexpected setup copy call")
    }
}

struct SetupTraceScheduler {
    enable_calls: Mutex<usize>,
}

impl SetupTraceScheduler {
    fn new() -> Self {
        Self {
            enable_calls: Mutex::new(0),
        }
    }

    fn enable_calls(&self) -> usize {
        *self.enable_calls.lock().unwrap()
    }
}

struct SetupSchedulerAdapter<'a> {
    inner: &'a SetupTraceScheduler,
}

impl BackupScheduler for SetupSchedulerAdapter<'_> {
    fn enable(&self, _: &Path) -> Result<String> {
        *self.inner.enable_calls.lock().unwrap() += 1;
        Ok("scheduled".into())
    }

    fn disable(&self) -> Result<String> {
        anyhow::bail!("unexpected setup scheduler disable call")
    }

    fn status(&self) -> Result<String> {
        anyhow::bail!("unexpected setup scheduler status call")
    }
}

#[derive(Debug, Clone)]
struct RestoreCall {
    repository: String,
    snapshot: String,
    target: String,
}

#[derive(Debug, Clone)]
struct DatabaseCall {
    repository: String,
    filename: String,
    program: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    tag: Option<String>,
}

struct LifecycleResticAdapter {
    invalid_sql: bool,
    fail_restore: bool,
    restore_calls: Mutex<Vec<RestoreCall>>,
    database_calls: Mutex<Vec<DatabaseCall>>,
}

impl LifecycleResticAdapter {
    fn new(invalid_sql: bool, fail_restore: bool) -> Self {
        Self {
            invalid_sql,
            fail_restore,
            restore_calls: Mutex::new(Vec::new()),
            database_calls: Mutex::new(Vec::new()),
        }
    }

    fn restore_calls(&self) -> Vec<RestoreCall> {
        self.restore_calls.lock().unwrap().clone()
    }

    fn database_calls(&self) -> Vec<DatabaseCall> {
        self.database_calls.lock().unwrap().clone()
    }

    fn unexpected(operation: &str) -> Result<String> {
        anyhow::bail!("unexpected lifecycle restic {operation} call")
    }

    fn record_database_call(
        &self,
        repository: &str,
        filename: &str,
        program: &str,
        args: &[String],
        tag: Option<&str>,
        environment: &[(&str, &str)],
    ) {
        self.database_calls.lock().unwrap().push(DatabaseCall {
            repository: repository.into(),
            filename: filename.into(),
            program: program.into(),
            args: args.to_vec(),
            environment: environment
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
            tag: tag.map(Into::into),
        });
    }
}

impl ResticRunner for LifecycleResticAdapter {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        Self::unexpected("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        Self::unexpected("backup paths")
    }

    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        Self::unexpected("snapshots")
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        Self::unexpected("restore")
    }

    fn restore_with_env(
        &self,
        repository: &str,
        _: &str,
        snapshot: &str,
        target: &str,
        _: &[(&str, &str)],
    ) -> Result<String> {
        self.restore_calls.lock().unwrap().push(RestoreCall {
            repository: repository.into(),
            snapshot: snapshot.into(),
            target: target.into(),
        });
        if self.fail_restore {
            anyhow::bail!("snapshot {snapshot} was not found or is ambiguous")
        }
        std::fs::create_dir_all(target)?;
        if self.invalid_sql {
            std::fs::write(std::path::Path::new(target).join("dump.sql"), "not sql")?;
        } else {
            std::fs::write(
                std::path::Path::new(target).join("restored.txt"),
                "restored",
            )?;
        }
        Ok("restore complete\n".into())
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        Self::unexpected("backup command")
    }

    fn backup_command_with_env(
        &self,
        repository: &str,
        _: &str,
        filename: &str,
        program: &str,
        args: &[String],
        environment: &[(&str, &str)],
    ) -> Result<String> {
        self.record_database_call(repository, filename, program, args, None, environment);
        Ok("database stream complete\n".into())
    }

    fn backup_command_with_env_and_tag(
        &self,
        repository: &str,
        _password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        tag: &str,
        environment: &[(&str, &str)],
    ) -> Result<String> {
        self.record_database_call(repository, filename, program, args, Some(tag), environment);
        Ok("database stream complete\n".into())
    }
}

impl RunContractFixture {
    fn profiles_path(&self) -> &Path {
        &self.profiles
    }
}

fn copy_contract_fixture(with_secondary: bool) -> CopyContractFixture {
    let directory = tempfile::tempdir().unwrap();
    let profiles = directory.path().join("profiles.yaml");
    let secondary_profile = if with_secondary {
        "  secondary:\n    repository: /secondary-repository\n    password-file: secondary-password\n"
    } else {
        ""
    };
    write_mode_600(
        &profiles,
        format!(
            "version: '2'\nprofiles:\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n{secondary_profile}  default:\n    inherit: primary\n    backup:\n      source: ['/data']\n    copy:\n      profile: secondary\n"
        ),
    );
    write_mode_600(&directory.path().join("primary-password"), "primary-secret");
    if with_secondary {
        write_mode_600(
            &directory.path().join("secondary-password"),
            "secondary-secret",
        );
    }
    backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();
    CopyContractFixture {
        _directory: directory,
        profiles,
    }
}

fn lifecycle_fixture(database: bool) -> LifecycleFixture {
    let directory = tempfile::tempdir().unwrap();
    let profiles = directory.path().join("profiles.yaml");
    let application = if database {
        "application:\n  database:\n    profile: database\n    type: postgres\n    connection-url: ${BACKUP_DATABASE_CONNECTION_URL}\n"
    } else {
        ""
    };
    let database_profile = if database {
        "  database:\n    inherit: primary\n"
    } else {
        ""
    };
    write_mode_600(
        &profiles,
        format!(
            "version: '2'\n{application}profiles:\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  secondary:\n    repository: /secondary-repository\n    password-file: secondary-password\n{database_profile}"
        ),
    );
    write_mode_600(&directory.path().join("primary-password"), "primary-secret");
    write_mode_600(
        &directory.path().join("secondary-password"),
        "secondary-secret",
    );
    if database {
        write_mode_600(
            &directory.path().join("database-connection-url"),
            "postgres://backup-user:db-secret@db:5432/app",
        );
    }
    backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();
    LifecycleFixture {
        directory,
        profiles,
    }
}

fn setup_contract_fixture() -> SetupFixture {
    let directory = tempfile::tempdir().unwrap();
    let profiles = directory.path().join("profiles.yaml");
    write_mode_600(
        &profiles,
        "version: '2'\nprofiles:\n  secondary:\n    repository: /secondary-repository\n    password-file: secondary-password\n  zeta:\n    inherit: primary\n    backup:\n      source: ['/zeta']\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  alpha:\n    inherit: primary\n    backup:\n      source: ['/alpha']\n  default: {}\n",
    );
    write_mode_600(&directory.path().join("primary-password"), "primary-secret");
    write_mode_600(
        &directory.path().join("secondary-password"),
        "secondary-secret",
    );
    backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();
    SetupFixture {
        _directory: directory,
        profiles,
    }
}

fn dispatch_setup<I>(profiles: &Path, args: I, resticprofile: &SetupTraceAdapter) -> CommandOutcome
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut argv = vec![
        "backup".to_string(),
        "--profiles".to_string(),
        profiles.to_string_lossy().into_owned(),
    ];
    argv.extend(args.into_iter().map(|arg| arg.as_ref().to_string()));
    let cli = Cli::try_parse_from(argv).unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();
    let command_runner = StrictCommandRunner::new([]);
    let adapter = SetupResticProfileAdapter {
        inner: resticprofile,
    };
    let restic = ResticTool::new(&command_runner);
    let rclone = RcloneTool::new(&command_runner);
    let scheduler = SystemScheduler::new(&command_runner, "backup");
    let adapters = AdapterSet {
        command: &command_runner,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &adapter,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };
    dispatch(&context, cli.command, &adapters)
}

fn dispatch_lifecycle<I>(
    profiles: &Path,
    args: I,
    restic: &LifecycleResticAdapter,
) -> CommandOutcome
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut argv = vec![
        "backup".to_string(),
        "--profiles".to_string(),
        profiles.to_string_lossy().into_owned(),
    ];
    argv.extend(args.into_iter().map(|arg| arg.as_ref().to_string()));
    let cli = Cli::try_parse_from(argv).unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();
    let command_runner = StrictCommandRunner::new([]);
    let resticprofile = ResticProfileTool::new(&command_runner);
    let rclone = RcloneTool::new(&command_runner);
    let scheduler = SystemScheduler::new(&command_runner, "backup");
    let adapters = AdapterSet {
        command: &command_runner,
        rclone: &rclone,
        restic,
        resticprofile: &resticprofile,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };
    dispatch(&context, cli.command, &adapters)
}

fn run_contract_fixture() -> RunContractFixture {
    let directory = tempfile::tempdir().unwrap();
    let profiles = directory.path().join("profiles.yaml");
    let reports = directory.path().join("reports");
    write_mode_600(
        &profiles,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: false\n    enableAnnualDrDrillReport: false\n  database:\n    profile: database\n    type: postgres\n    connection-url: ${{BACKUP_DATABASE_CONNECTION_URL}}\nprofiles:\n  default: {{}}\n  primary:\n    repository: /primary-repository\n    password-file: primary-password\n  secondary:\n    repository: /secondary-repository\n    password-file: secondary-password\n  alpha:\n    inherit: primary\n    backup:\n      source: ['/alpha']\n      tag: ['backup-profile:alpha']\n    copy:\n      profile: secondary\n  beta:\n    inherit: primary\n    backup:\n      source: ['/beta']\n      tag: ['backup-profile:beta']\n    copy:\n      profile: secondary\n  database:\n    inherit: primary\n    backup:\n      tag: ['backup-profile:database']\n",
            reports.display()
        ) + "  archive:\n    repository: /archive-repository\n    password-file: archive-password\n",
    );
    set_mode_600(&profiles);
    write_mode_600(&directory.path().join("primary-password"), "primary-secret");
    write_mode_600(
        &directory.path().join("secondary-password"),
        "secondary-secret",
    );
    write_mode_600(&directory.path().join("archive-password"), "archive-secret");
    write_mode_600(
        &directory.path().join("database-connection-url"),
        "postgres://backup-user:db-secret@db:5432/app",
    );
    backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();
    RunContractFixture {
        _directory: directory,
        profiles,
    }
}

fn expected_full_pipeline(
    dry_run: bool,
    skip_database: bool,
    skip_secondary: bool,
    skip_retention: bool,
) -> RunContractExpectations {
    expected_pipeline(
        RunSelection::All,
        dry_run,
        skip_database,
        skip_secondary,
        skip_retention,
    )
}

fn expected_pipeline(
    selection: RunSelection,
    dry_run: bool,
    skip_database: bool,
    skip_secondary: bool,
    skip_retention: bool,
) -> RunContractExpectations {
    let mut profile_calls = Vec::new();
    let database_call = (matches!(selection, RunSelection::All | RunSelection::Database)
        && !skip_database
        && !dry_run)
        .then(database_call_expectation);
    let profiles = match selection {
        RunSelection::All => vec!["alpha", "beta"],
        RunSelection::Alpha => vec!["alpha"],
        RunSelection::Database => Vec::new(),
    };
    for profile in &profiles {
        profile_calls.push(profile_call(
            "primary",
            profile,
            Some(dry_run),
            &format!("snapshot {profile}-id saved\n"),
            None,
        ));
    }
    if !skip_secondary {
        for profile in &profiles {
            profile_calls.push(profile_call(
                "secondary",
                profile,
                Some(dry_run),
                &format!("copy {profile} complete\n"),
                None,
            ));
        }
    }
    if !skip_retention && !dry_run {
        for profile in &profiles {
            profile_calls.push(profile_call(
                "retention",
                profile,
                None,
                &format!("retention {profile} complete\n"),
                None,
            ));
        }
    }
    RunContractExpectations {
        profile_calls,
        database_call,
    }
}

fn expected_trace(
    selection: RunSelection,
    dry_run: bool,
    skip_database: bool,
    skip_secondary: bool,
    skip_retention: bool,
) -> Vec<String> {
    let mut trace = Vec::new();
    if matches!(selection, RunSelection::All | RunSelection::Database) && !skip_database && !dry_run
    {
        trace.push("database".into());
    }
    let profiles = match selection {
        RunSelection::All => [Some("alpha"), Some("beta")].as_slice(),
        RunSelection::Alpha => [Some("alpha"), None].as_slice(),
        RunSelection::Database => [None, None].as_slice(),
    };
    for profile in profiles.iter().flatten() {
        trace.push(format!("primary:{profile}:dry={dry_run}"));
    }
    if !skip_secondary {
        for profile in profiles.iter().flatten() {
            trace.push(format!("secondary:{profile}:dry={dry_run}"));
        }
    }
    if !skip_retention && !dry_run {
        for profile in profiles.iter().flatten() {
            trace.push(format!("retention:{profile}"));
        }
    }
    trace
}

fn database_call_expectation() -> DatabaseCallExpectation {
    DatabaseCallExpectation {
        repository: "/primary-repository".into(),
        password: "primary-secret".into(),
        filename: "app.sql".into(),
        program: "pg_dump".into(),
        args: vec![
            "--host=db".into(),
            "--username=backup-user".into(),
            "--dbname=app".into(),
            "--port=5432".into(),
        ],
        environment: vec![("PGPASSWORD".into(), "db-secret".into())],
        tag: Some("backup-profile:database".into()),
        output: "database stream complete\n".into(),
    }
}

fn profile_call(
    operation: &'static str,
    profile: &str,
    dry_run: Option<bool>,
    output: &str,
    error: Option<&str>,
) -> ProfileCallExpectation {
    ProfileCallExpectation {
        operation,
        profile: profile.into(),
        dry_run,
        output: output.into(),
        error: error.map(Into::into),
    }
}

fn dispatch_run_contract(
    profiles: &Path,
    options: &[&str],
    expected: RunContractExpectations,
) -> (CommandOutcome, Vec<String>) {
    let mut argv = vec![
        "backup".to_string(),
        "--profiles".to_string(),
        profiles.to_string_lossy().into_owned(),
        "run".to_string(),
    ];
    argv.extend(options.iter().map(|option| (*option).into()));
    let cli = Cli::try_parse_from(argv).unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let profile_adapter =
        StrictRunProfileAdapter::new(profiles, expected.profile_calls, trace.clone());
    let database_adapter = StrictRunDatabaseAdapter::new(expected.database_call, trace.clone());
    let command_runner = StrictCommandRunner::new([]);
    let rclone = RcloneTool::new(&command_runner);
    let scheduler = SystemScheduler::new(&command_runner, "backup");
    let adapters = AdapterSet {
        command: &command_runner,
        rclone: &rclone,
        restic: &database_adapter,
        resticprofile: &profile_adapter,
        scheduler: &scheduler,
        selection: AdapterSelection::StrictTest,
    };

    let outcome = dispatch(&context, cli.command, &adapters);
    profile_adapter.assert_exhausted();
    database_adapter.assert_exhausted();
    command_runner.assert_exhausted().unwrap();
    (outcome, trace.lock().unwrap().clone())
}

fn write_mode_600(path: &Path, contents: impl AsRef<str>) {
    std::fs::write(path, contents.as_ref()).unwrap();
    set_mode_600(path);
}

fn set_mode_600(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn set_mode_700(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
}

fn file_mode(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return std::fs::metadata(path)
            .ok()
            .map(|metadata| metadata.permissions().mode() & 0o777);
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}
