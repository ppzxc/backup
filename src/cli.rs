//! The authoritative CLI schema, runtime context, and shared command dispatch seam.
//!
//! The binary is intentionally only a composition shell. Keeping parsing and dispatch here lets
//! the production process and contract tests observe the same command behavior.

use crate::commands::report::{ReportAction, ReportCommand, ReportFormat};
use crate::commands::restore::RestoreStorage;
use crate::config::model::{DEFAULT_PROFILES_PATH, ResticProfileConfig};
use crate::i18n::Language;
use crate::runner::executor::CommandRunner;
use crate::runner::rclone::RcloneRunner;
use crate::runner::restic::ResticRunner;
use crate::runner::resticprofile::ResticProfileRunner;
use crate::runner::scheduler::BackupScheduler;
use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use std::fmt;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "backup", version)]
pub struct Cli {
    /// Unified resticprofile v2 and application configuration file path.
    #[arg(long, global = true, value_name = "PATH")]
    pub profiles: Option<PathBuf>,

    /// Verbosity level (-v for debug, -vv for trace)
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (only warn/error logs)
    #[arg(long, short = 'q', global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Log file path
    #[arg(long, global = true, value_name = "PATH")]
    pub log_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Unified backup configuration setup wizard / 통합 백업 설정 마법사
    Setup {
        /// Select language (ko/en) / 언어 선택 (ko/en)
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        non_interactive: bool,
        #[command(subcommand)]
        action: Option<SetupAction>,
    },
    /// Sync snapshots from primary to secondary storage target / 저장소 간 스냅샷 동기화 및 복사
    #[command(alias = "sync")]
    Copy {
        /// Profile name to copy (default: "default")
        #[arg(long, short = 'p')]
        profile: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Execute backup pipeline / 백업 파이프라인 수동 실행
    Run {
        /// Profile name to run (default: "default")
        #[arg(long, short = 'p')]
        profile: Option<String>,
        #[arg(long)]
        skip_database: bool,
        #[arg(long)]
        skip_secondary_sync: bool,
        #[arg(long)]
        skip_retention: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Create a Database Stream snapshot / 데이터베이스 스트림 백업 실행
    Database {
        #[arg(long)]
        dry_run: bool,
    },
    /// Comprehensive system settings, dependencies, and health check diagnostics / 시스템 설정, 의존성 및 헬스체크 종합 진단
    Doctor,
    /// ISMS-P audit evidence and report generation / ISMS-P 감사 증적 및 레포트 생성
    Report {
        #[command(subcommand)]
        action: Option<ReportAction>,
        /// Report file path / 보고서 파일 저장 경로
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Report format (html, json) / 보고서 포맷
        #[arg(long)]
        format: Option<ReportFormat>,
    },
    /// Systemd timer / Cron scheduler management / 스케줄러 타이머 관리
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Restore files or database dumps from snapshot / 스냅샷 기반 파일 및 DB 복구
    Restore {
        #[arg(long, default_value = "latest")]
        snapshot: String,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        force: bool,
        /// Storage to restore from (primary or secondary)
        #[arg(long, value_enum, default_value_t = RestoreStorage::Primary)]
        storage: RestoreStorage,
    },
    /// List snapshots across primary and secondary storage targets / 스냅샷 목록 조회
    Snapshots,
    /// Display operational status and snapshot recency / 운영 상태 및 스냅샷 주기 확인
    Status {
        /// Profile name to query status for (optional)
        #[arg(long, short = 'p')]
        profile: Option<String>,
    },
    /// Self-update backup binary and assets / 바이너리 및 자산 자가 업데이트
    Update,
    /// Display CLI binary version / CLI 바이너리 버전 표시
    Version,
    /// Uninstall backup CLI and scheduled timers / 백업 CLI 및 스케줄러 삭제
    Uninstall {
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        purge: bool,
    },
}

#[derive(Subcommand)]
pub enum SetupAction {
    /// Verify and download required binary dependencies (restic, rclone, resticprofile)
    Dependencies,
    /// Initialize primary and secondary Backend Adapter repositories
    BackendInit,
}

#[derive(Subcommand)]
pub enum ScheduleAction {
    /// Enable systemd timers / cron fallback
    Enable,
    /// Disable scheduled timers
    Disable,
    /// Display timer/scheduler status
    Status,
}

/// The one parser schema consumed by production and the contract matrix.
pub fn authoritative_cli_schema() -> clap::Command {
    Cli::command()
}

/// Returns stable command.option axis identifiers for the contract matrix coverage guard.
pub fn authoritative_cli_axes() -> Vec<String> {
    fn visit(command: &clap::Command, prefix: &str, axes: &mut Vec<String>) {
        let name = if prefix.is_empty() {
            command.get_name().to_string()
        } else {
            format!("{prefix}.{}", command.get_name())
        };
        axes.extend(
            command
                .get_arguments()
                .map(|argument| format!("{name}.{}", argument.get_id().as_str())),
        );
        for subcommand in command.get_subcommands() {
            visit(subcommand, &name, axes);
        }
    }

    let mut axes = Vec::new();
    visit(&authoritative_cli_schema(), "", &mut axes);
    axes.sort();
    axes
}

pub fn authoritative_cli_command_paths() -> Vec<String> {
    fn visit(command: &clap::Command, prefix: &str, paths: &mut Vec<String>) {
        let name = if prefix.is_empty() {
            command.get_name().to_string()
        } else {
            format!("{prefix}.{}", command.get_name())
        };
        paths.push(name.clone());
        for subcommand in command.get_subcommands() {
            visit(subcommand, &name, paths);
        }
    }

    let mut paths = Vec::new();
    visit(&authoritative_cli_schema(), "", &mut paths);
    paths.sort();
    paths
}

pub use crate::runner::scheduler::SchedulerMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterSelection {
    System,
    StrictTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLogging {
    pub level_filter: String,
    pub log_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RestoreDrillSetupOverrides {
    pub rto_minutes: Option<u64>,
    pub timeout_minutes: Option<u64>,
    pub work_dir: Option<PathBuf>,
}

impl RestoreDrillSetupOverrides {
    pub fn from_process_environment() -> Result<Self> {
        fn minutes(name: &str) -> Result<Option<u64>> {
            let Some(value) = std::env::var_os(name) else {
                return Ok(None);
            };
            let value = value.to_string_lossy();
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|error| anyhow::anyhow!("{name} must be an unsigned integer: {error}"))
        }

        Ok(Self {
            rto_minutes: minutes("BACKUP_TEST_RESTORE_DRILL_RTO_MINUTES")?,
            timeout_minutes: minutes("BACKUP_TEST_RESTORE_DRILL_TIMEOUT_MINUTES")?,
            work_dir: std::env::var_os("BACKUP_TEST_RESTORE_DRILL_WORK_DIR").map(PathBuf::from),
        })
    }
}

/// All process-derived values needed by one command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRuntimeContext {
    pub profiles_path: PathBuf,
    pub language: Language,
    pub logging: RuntimeLogging,
    pub scheduler_mode: SchedulerMode,
    pub scheduler_calendar: String,
    pub scheduler_force_cron: bool,
    pub adapter_selection: AdapterSelection,
    pub home_dir: PathBuf,
    pub host_name: String,
    pub restore_drill_setup_overrides: RestoreDrillSetupOverrides,
}

impl CliRuntimeContext {
    pub fn from_cli(
        cli: &Cli,
        language: Language,
        env_log_override: Option<String>,
        scheduler_mode: SchedulerMode,
        adapter_selection: AdapterSelection,
    ) -> Result<Self> {
        if cli.quiet && cli.verbose > 0 {
            anyhow::bail!("--quiet cannot be combined with --verbose");
        }

        let language = match &cli.command {
            Command::Setup {
                lang: Some(value), ..
            } => parse_language(value)?,
            _ => language,
        };
        let level_filter = if cli.quiet {
            "warn".into()
        } else if cli.verbose > 0 {
            crate::logger::determine_level_filter(cli.verbose, false, None)
        } else {
            crate::logger::determine_level_filter(0, false, env_log_override.as_deref())
        };

        Ok(Self {
            profiles_path: cli
                .profiles
                .clone()
                .unwrap_or_else(|| PathBuf::from(DEFAULT_PROFILES_PATH)),
            language,
            logging: RuntimeLogging {
                level_filter,
                log_file: cli.log_file.clone(),
            },
            scheduler_mode,
            scheduler_calendar: crate::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR.into(),
            scheduler_force_cron: false,
            adapter_selection,
            home_dir: PathBuf::from("/tmp"),
            host_name: "localhost".into(),
            restore_drill_setup_overrides: RestoreDrillSetupOverrides::default(),
        })
    }

    pub fn with_environment(
        mut self,
        home_dir: impl Into<PathBuf>,
        host_name: impl Into<String>,
        scheduler_calendar: impl Into<String>,
    ) -> Self {
        self.home_dir = home_dir.into();
        self.host_name = host_name.into();
        self.scheduler_calendar = scheduler_calendar.into();
        self
    }

    pub fn with_scheduler_force_cron(mut self, force_cron: bool) -> Self {
        self.scheduler_force_cron = force_cron;
        self
    }

    pub fn with_restore_drill_setup_overrides(
        mut self,
        overrides: RestoreDrillSetupOverrides,
    ) -> Self {
        self.restore_drill_setup_overrides = overrides;
        self
    }

    pub fn scheduler_settings(&self) -> crate::runner::scheduler::SchedulerSettings {
        crate::runner::scheduler::SchedulerSettings::new(
            self.scheduler_mode,
            &self.scheduler_calendar,
        )
        .with_force_cron(self.scheduler_force_cron)
    }
}

pub fn parse_language(value: &str) -> Result<Language> {
    match value {
        "ko" => Ok(Language::Ko),
        "en" => Ok(Language::En),
        _ => anyhow::bail!("invalid language '{value}'; expected ko or en"),
    }
}

/// Production and strict test adapters are passed through this one composition point.
pub struct AdapterSet<'a> {
    pub command: &'a dyn CommandRunner,
    pub rclone: &'a dyn RcloneRunner,
    pub restic: &'a dyn ResticRunner,
    pub resticprofile: &'a dyn ResticProfileRunner,
    pub scheduler: &'a dyn BackupScheduler,
    pub selection: AdapterSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractExpectation {
    pub exit_status: i32,
    pub stdout: String,
    pub stderr: String,
    pub artifact_kinds: Vec<String>,
    pub external_state_changes: Vec<String>,
    pub adapter_trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractCaseSpec {
    pub command_path: String,
    pub option_axis: Option<String>,
    pub behavior_class: String,
    pub values: Vec<String>,
    pub argv: Vec<String>,
    pub expectation: ContractExpectation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliContractCase {
    pub id: String,
    pub command_path: String,
    pub option_axis: Option<String>,
    pub behavior_class: String,
    pub values: Vec<String>,
    pub argv: Vec<String>,
    pub expectation: Option<ContractExpectation>,
}

/// Generates stable schema cases independently of command implementation details.
///
/// Command-path cases ensure every parser branch exists in the contract inventory; option-axis
/// cases ensure every declared option is consumed by the matrix coverage guard.
pub fn generate_cli_contract_matrix() -> Vec<CliContractCase> {
    let mut cases = authoritative_cli_command_paths()
        .into_iter()
        .map(|command_path| CliContractCase {
            id: format!("command:{command_path}:default"),
            command_path,
            option_axis: None,
            behavior_class: "command-default".into(),
            values: Vec::new(),
            argv: Vec::new(),
            expectation: None,
        })
        .collect::<Vec<_>>();
    cases.extend(authoritative_cli_axes().into_iter().map(|option_axis| {
        let command_path = option_axis
            .rsplit_once('.')
            .map(|(path, _)| path.to_owned())
            .unwrap_or_else(|| option_axis.clone());
        let behavior_class = option_behavior_class(&option_axis);
        CliContractCase {
            id: format!("option:{option_axis}:default"),
            command_path,
            option_axis: Some(option_axis),
            behavior_class,
            values: Vec::new(),
            argv: Vec::new(),
            expectation: None,
        }
    }));
    cases
}

/// Builds executable contract cases from test-owned expectations.
///
/// The schema inventory above is intentionally not an expectation oracle.  This constructor is
/// the seam where an independent contract table supplies concrete values, parser input, and
/// adapter/output expectations while the authoritative schema still enforces completeness.
pub fn generate_cli_contract_matrix_with_specs(
    specs: impl IntoIterator<Item = ContractCaseSpec>,
) -> Result<Vec<CliContractCase>> {
    let specs = specs.into_iter().collect::<Vec<_>>();
    let command_paths = authoritative_cli_command_paths();
    let option_axes = authoritative_cli_axes();

    for spec in &specs {
        if let Some(option_axis) = &spec.option_axis {
            if !option_axes.contains(option_axis) {
                anyhow::bail!("contract matrix contains unknown option axis {option_axis}");
            }
            let expected_command_path = option_axis
                .rsplit_once('.')
                .map(|(path, _)| path)
                .unwrap_or(option_axis);
            if spec.command_path != expected_command_path {
                anyhow::bail!(
                    "contract matrix option {option_axis} belongs to {expected_command_path}, not {}",
                    spec.command_path
                );
            }
        } else if !command_paths.contains(&spec.command_path) {
            anyhow::bail!(
                "contract matrix contains unknown command path {}",
                spec.command_path
            );
        }
    }

    for command_path in &command_paths {
        if !specs
            .iter()
            .any(|spec| spec.option_axis.is_none() && spec.command_path == *command_path)
        {
            anyhow::bail!("contract matrix has no command case for {command_path}");
        }
    }
    for option_axis in &option_axes {
        if !specs
            .iter()
            .any(|spec| spec.option_axis.as_deref() == Some(option_axis.as_str()))
        {
            anyhow::bail!("contract matrix has no option case for {option_axis}");
        }
    }

    let mut cases = specs
        .into_iter()
        .map(|spec| {
            let prefix = if let Some(axis) = &spec.option_axis {
                format!("option:{axis}")
            } else {
                format!("command:{}", spec.command_path)
            };
            let suffix = if spec.values.is_empty() {
                spec.behavior_class.clone()
            } else {
                format!("{}={}", spec.behavior_class, spec.values.join(","))
            };
            CliContractCase {
                id: format!("{prefix}:{suffix}"),
                command_path: spec.command_path,
                option_axis: spec.option_axis,
                behavior_class: spec.behavior_class,
                values: spec.values,
                argv: spec.argv,
                expectation: Some(spec.expectation),
            }
        })
        .collect::<Vec<_>>();
    cases.sort_by(|left, right| left.id.cmp(&right.id));
    let mut ids = cases.iter().map(|case| &case.id).collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    if ids.len() != cases.len() {
        anyhow::bail!("contract matrix contains duplicate case IDs");
    }
    Ok(cases)
}

fn option_behavior_class(option_axis: &str) -> String {
    let option = option_axis.rsplit('.').next().unwrap_or_default();
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
        | "yes" => "flag-enabled",
        "format" | "lang" | "profile" | "profiles" | "storage" => "enum-value",
        "file" | "log_file" | "snapshot" | "target" => "path-or-text-value",
        _ => "value-supplied",
    }
    .into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDiagnostic {
    pub case_id: String,
    pub context: String,
    pub expected_trace: Vec<String>,
    pub actual_trace: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_status: i32,
    pub artifacts: Vec<Artifact>,
}

impl ContractDiagnostic {
    pub fn from_outcome(
        case_id: impl Into<String>,
        context: impl Into<String>,
        expected_trace: Vec<String>,
        actual_trace: Vec<String>,
        outcome: &CommandOutcome,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            context: context.into(),
            expected_trace,
            actual_trace,
            stdout: outcome.stdout.clone(),
            stderr: outcome.stderr.clone(),
            exit_status: outcome.exit_status,
            artifacts: outcome.artifacts.clone(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            "case={} context={} expected_trace={:?} actual_trace={:?} stdout={:?} stderr={:?} exit_status={} artifacts={:?}",
            self.case_id,
            self.context,
            self.expected_trace,
            self.actual_trace,
            self.stdout,
            self.stderr,
            self.exit_status,
            self.artifacts,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub path: PathBuf,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: i32,
    pub artifacts: Vec<Artifact>,
    pub external_state_changes: Vec<String>,
}

#[derive(Debug)]
struct SetupExecutionFailure {
    message: String,
    notices: String,
}

impl fmt::Display for SetupExecutionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for SetupExecutionFailure {}

impl CommandOutcome {
    pub fn success(
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        artifacts: Vec<PathBuf>,
    ) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            exit_status: 0,
            artifacts: artifacts
                .into_iter()
                .map(|path| Artifact {
                    path,
                    kind: "file".into(),
                })
                .collect(),
            external_state_changes: Vec::new(),
        }
    }

    pub fn success_with_changes(
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        artifacts: Vec<PathBuf>,
        external_state_changes: Vec<String>,
    ) -> Self {
        let mut outcome = Self::success(stdout, stderr, artifacts);
        outcome.external_state_changes = external_state_changes;
        outcome
    }

    pub fn failure(command: &str, stage: &str, error: impl Into<String>) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!(
                "{command} failed at {stage}: {}",
                redact_diagnostic(&error.into())
            ),
            exit_status: 1,
            artifacts: Vec::new(),
            external_state_changes: Vec::new(),
        }
    }

    pub fn failure_with_metadata(
        command: &str,
        stage: &str,
        error: impl Into<String>,
        artifacts: Vec<PathBuf>,
        external_state_changes: Vec<String>,
    ) -> Self {
        let mut outcome = Self::failure(command, stage, error);
        outcome.artifacts = artifacts
            .into_iter()
            .map(|path| Artifact {
                path,
                kind: "file".into(),
            })
            .collect();
        outcome.external_state_changes = external_state_changes;
        outcome
    }

    pub fn is_success(&self) -> bool {
        self.exit_status == 0
    }
}

fn redact_diagnostic(value: &str) -> String {
    let mut redact_next = false;
    value
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            let is_sensitive_name = [
                "password",
                "secret",
                "token",
                "credential",
                "connection-url",
                "connection_url",
                "repository",
                "access-key",
                "access_key",
            ]
            .iter()
            .any(|marker| lower.contains(marker));
            let sensitive_value = lower.contains("://")
                || lower.starts_with("s3:")
                || lower.starts_with("sftp:")
                || lower.starts_with("mysql:")
                || lower.starts_with("postgres:");
            if redact_next || sensitive_value {
                redact_next = false;
                "<redacted>".to_string()
            } else if is_sensitive_name && token.contains('=') {
                format!("{}<redacted>", token.split('=').next().unwrap_or(token))
            } else {
                redact_next = is_sensitive_name;
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn dispatch(
    context: &CliRuntimeContext,
    command: Command,
    adapters: &AdapterSet<'_>,
) -> CommandOutcome {
    let command_name = command_name(&command);
    if context.adapter_selection != adapters.selection {
        return CommandOutcome::failure(
            &command_name,
            "adapter selection",
            format!(
                "runtime selected {:?}, but composition supplied {:?}",
                context.adapter_selection, adapters.selection
            ),
        );
    }
    match dispatch_inner(context, command, adapters) {
        Ok(outcome) => outcome,
        Err(error) => {
            if let Some(setup_failure) = error.downcast_ref::<SetupExecutionFailure>() {
                let mut outcome = CommandOutcome::failure(
                    &command_name,
                    "execution",
                    setup_failure.message.clone(),
                );
                outcome.stdout = setup_failure.notices.clone();
                outcome
            } else if let Some(report_failure) =
                error.downcast_ref::<crate::commands::report::ReportCommandFailure>()
            {
                CommandOutcome::failure_with_metadata(
                    &command_name,
                    "execution",
                    report_failure.message.clone(),
                    report_failure.artifacts.clone(),
                    report_failure.external_state_changes.clone(),
                )
            } else if let Some(run_failure) =
                error.downcast_ref::<crate::commands::run::RunCommandFailure>()
            {
                CommandOutcome::failure_with_metadata(
                    &command_name,
                    "execution",
                    run_failure.message.clone(),
                    run_failure.artifacts.clone(),
                    run_failure.external_state_changes.clone(),
                )
            } else if let Some(status_failure) =
                error.downcast_ref::<crate::commands::status::StatusCommandFailure>()
            {
                let mut outcome = CommandOutcome::failure(
                    &command_name,
                    "status query",
                    status_failure.message.clone(),
                );
                outcome.stdout = status_failure.output.clone();
                outcome
            } else {
                CommandOutcome::failure(&command_name, "execution", error.to_string())
            }
        }
    }
}

fn finish_setup_dispatch(
    context: &CliRuntimeContext,
    result: Result<()>,
    notices: String,
) -> Result<CommandOutcome> {
    match result {
        Ok(()) => {
            let success = match context.language {
                Language::Ko => "설정이 성공적으로 완료되었습니다.",
                Language::En => "Setup completed successfully.",
            };
            let stdout = if notices.is_empty() {
                success.to_owned()
            } else {
                format!("{notices}\n{success}")
            };
            Ok(CommandOutcome::success_with_changes(
                stdout,
                "",
                Vec::new(),
                vec!["configuration and scheduler updated".into()],
            ))
        }
        Err(error) => Err(SetupExecutionFailure {
            message: error.to_string(),
            notices,
        }
        .into()),
    }
}

fn run_setup_with_notice_sink(
    context: &CliRuntimeContext,
    prompter: &crate::commands::setup::InquirePrompter,
    non_interactive: bool,
    lang: Option<Language>,
    adapters: &AdapterSet<'_>,
    notices: &mut dyn crate::commands::setup::SetupNoticeSink,
) -> Result<()> {
    let scheduler_settings = context.scheduler_settings();
    crate::commands::setup::SetupEngine::run_with_options(
        &context.profiles_path,
        prompter,
        non_interactive,
        lang,
        adapters.resticprofile,
        adapters.scheduler,
        crate::commands::setup::SetupRunOptions::new(&scheduler_settings, notices),
    )
}

fn dispatch_inner(
    context: &CliRuntimeContext,
    command: Command,
    adapters: &AdapterSet<'_>,
) -> Result<CommandOutcome> {
    match command {
        Command::Setup {
            lang,
            non_interactive,
            action,
        } => match action {
            Some(SetupAction::Dependencies) => {
                let install_dir =
                    crate::commands::setup::resolve_dependency_install_dir(&context.home_dir)?;
                let output = crate::commands::setup::run_setup_dependencies_with_runner_at_dir(
                    adapters.command,
                    &install_dir,
                    context.language,
                )?;
                Ok(CommandOutcome::success_with_changes(
                    output,
                    "",
                    Vec::new(),
                    vec!["dependencies verified or installed".into()],
                ))
            }
            Some(SetupAction::BackendInit) => {
                let has_pending =
                    crate::commands::setup::pending_setup_exists(&context.profiles_path)?;
                let init_profiles_path = if has_pending {
                    crate::commands::setup::pending_setup_profiles_path(&context.profiles_path)
                } else {
                    required_profiles_path(context)?
                };
                require_regular_profiles_file(&init_profiles_path)?;
                let config = ResticProfileConfig::load_from_path(&init_profiles_path)
                    .with_context(|| {
                        format!(
                            "failed to load unified profiles configuration at {}",
                            init_profiles_path.display()
                        )
                    })?;
                let targets = backend_initialization_targets(&config)?;
                let mut output = Vec::new();
                let mut failures = Vec::new();
                for profile in targets {
                    match adapters.resticprofile.init(&init_profiles_path, &profile) {
                        Ok(result) => {
                            let heading = match context.language {
                                Language::Ko => "=== 백엔드 저장소 초기화 프로필:",
                                Language::En => "=== Initializing Backend Storage for Profile:",
                            };
                            output
                                .push(format!("{heading} [{profile}] ===\n{}", result.trim_end()));
                        }
                        Err(error) => failures.push(format!(
                            "{profile}: {}",
                            crate::commands::setup::redact_backend_initialization_error(
                                error.to_string(),
                                &config,
                                &init_profiles_path,
                                &context.profiles_path,
                            )
                        )),
                    }
                }
                if failures.is_empty() {
                    crate::commands::setup::promote_pending_setup(&context.profiles_path)?;
                    Ok(CommandOutcome::success_with_changes(
                        output.join("\n"),
                        "",
                        Vec::new(),
                        vec!["backend repositories initialized".into()],
                    ))
                } else {
                    let prefix = match context.language {
                        Language::Ko => {
                            "모든 대상에 대한 시도 후 백엔드 저장소 초기화에 실패했습니다"
                        }
                        Language::En => {
                            "backend initialization failed after attempting every target"
                        }
                    };
                    anyhow::bail!("{prefix}: {}", failures.join("; "))
                }
            }
            None => {
                let prompter =
                    crate::commands::setup::InquirePrompter::with_restore_drill_overrides(
                        context.restore_drill_setup_overrides.clone(),
                    );
                let setup_language = Some(
                    lang.as_deref()
                        .map(parse_language)
                        .transpose()?
                        .unwrap_or(context.language),
                );
                if non_interactive {
                    let mut notices = crate::commands::setup::SetupNoticeCollector::new();
                    let result = run_setup_with_notice_sink(
                        context,
                        &prompter,
                        true,
                        setup_language,
                        adapters,
                        &mut notices,
                    );
                    finish_setup_dispatch(context, result, notices.into_output())
                } else {
                    let mut notices = crate::commands::setup::TuiSetupNoticeRenderer::stdout();
                    let result = run_setup_with_notice_sink(
                        context,
                        &prompter,
                        false,
                        setup_language,
                        adapters,
                        &mut notices,
                    );
                    finish_setup_dispatch(context, result, String::new())
                }
            }
        },
        Command::Copy { profile, dry_run } => {
            let config = load_profiles(context)?;
            let target_profile = profile.as_deref().unwrap_or("default");
            crate::config::profile_resolver::ProfileResolver::resolve_exact(
                &config,
                target_profile,
                "copy",
            )?;
            let copy_target = config
                .effective_copy_profile(target_profile)?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Backup Profile '{target_profile}' does not declare a copy target Backend Profile"
                    )
                })?;
            let config_dir = context
                .profiles_path
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| std::path::Path::new("."));
            config.backend_credentials(config_dir, target_profile)?;
            config.backend_credentials(config_dir, &copy_target)?;
            let output = crate::commands::copy::execute_copy(
                adapters.resticprofile,
                &context.profiles_path,
                target_profile,
                dry_run,
            )?;
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                if dry_run {
                    Vec::new()
                } else {
                    vec!["snapshots copied".into()]
                },
            ))
        }
        Command::Run {
            profile,
            skip_database,
            skip_secondary_sync,
            skip_retention,
            dry_run,
        } => dispatch_run(
            context,
            adapters,
            profile,
            crate::commands::run::PipelineOptions {
                skip_database,
                skip_secondary_sync,
                skip_retention,
                dry_run,
            },
        ),
        Command::Database { dry_run } => {
            let config = load_profiles(context)?;
            let output = crate::commands::database::execute_database_backup_from_profiles(
                &config,
                &context.profiles_path,
                adapters.restic,
                dry_run,
            )?;
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                vec![if dry_run {
                    "database backup plan rendered".into()
                } else {
                    "database snapshot created".into()
                }],
            ))
        }
        Command::Doctor => {
            let (output, passed, diagnostics) =
                crate::commands::doctor::run_doctor_contract_with_runner_and_diagnostics(
                    adapters.rclone,
                    adapters.command,
                    Some(&context.profiles_path),
                    &context.host_name,
                )?;
            let mut outcome = CommandOutcome::success(output, diagnostics, Vec::new());
            if !passed {
                outcome.exit_status = 1;
                outcome
                    .stderr
                    .push_str("\ndoctor reported one or more failed diagnostics");
            }
            Ok(outcome)
        }
        Command::Report {
            action,
            file,
            format,
        } => {
            let machine_readable = report_uses_machine_readable_output(action.as_ref(), format);
            let profiles_path = required_profiles_path(context)?;
            let profiles = load_profiles(context)?;
            let meta = crate::commands::report::AuditReportMeta::new(
                &context.host_name,
                crate::commands::report::get_formatted_time().0,
            )
            .with_profiles_path(&profiles_path);
            let output = ReportCommand::run_with_profile_adapters(
                action,
                file,
                format,
                &profiles,
                &profiles_path,
                adapters.command,
                adapters.restic,
                &meta,
            )?;
            let stdout = if machine_readable {
                ""
            } else {
                output.as_str()
            };
            Ok(CommandOutcome::success_with_changes(
                stdout,
                "",
                report_paths_from_output(&output),
                vec!["report artifacts committed".into()],
            ))
        }
        Command::Schedule { action } => {
            let output = match &action {
                ScheduleAction::Enable => {
                    crate::commands::schedule::execute_schedule_enable_with_settings(
                        &context.profiles_path,
                        adapters.scheduler,
                        &context.scheduler_settings(),
                    )?
                }
                ScheduleAction::Disable => {
                    crate::commands::schedule::execute_schedule_disable_with_settings(
                        &context.profiles_path,
                        adapters.scheduler,
                        &context.scheduler_settings(),
                    )?
                }
                ScheduleAction::Status => {
                    crate::commands::schedule::execute_schedule_status_with_settings(
                        adapters.scheduler,
                        &context.scheduler_settings(),
                    )?
                }
            };
            let changes = match action {
                ScheduleAction::Status => Vec::new(),
                ScheduleAction::Enable => vec!["scheduler registration updated".into()],
                ScheduleAction::Disable => vec!["scheduler registration removed".into()],
            };
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                changes,
            ))
        }
        Command::Restore {
            snapshot,
            target,
            force,
            storage,
        } => {
            let config = load_profiles(context)?;
            if snapshot.trim().is_empty() || snapshot != snapshot.trim() {
                anyhow::bail!("restore snapshot must be an exact, non-empty value");
            }
            let target = target.ok_or_else(|| anyhow::anyhow!("restore requires --target"))?;
            if target.trim().is_empty() || target != target.trim() {
                anyhow::bail!("restore target must be an exact, non-empty path");
            }
            let output = crate::commands::restore::execute_restore_from_profiles(
                &config,
                &context.profiles_path,
                adapters.restic,
                &snapshot,
                &target,
                force,
                storage,
            )?;
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                vec!["restore output created".into()],
            ))
        }
        Command::Snapshots => {
            let config = load_profiles(context)?;
            let output = crate::commands::snapshots::execute_snapshots_from_profiles(
                &config,
                &context.profiles_path,
                adapters.restic,
            )?;
            Ok(CommandOutcome::success(output, "", Vec::new()))
        }
        Command::Status { profile } => {
            let config = load_profiles(context)?;
            if let Some(profile) = profile.as_deref() {
                crate::config::profile_resolver::ProfileResolver::resolve_exact(
                    &config, profile, "status",
                )?;
            }
            let output = crate::commands::status::execute_status_from_profiles_config(
                &context.profiles_path,
                profile.as_deref(),
                adapters.resticprofile,
            )?;
            Ok(CommandOutcome::success(output, "", Vec::new()))
        }
        Command::Update => {
            let output = crate::commands::update::execute_update_check_with_runner(
                env!("CARGO_PKG_VERSION"),
                adapters.command,
            )?;
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                vec!["update installation attempted".into()],
            ))
        }
        Command::Version => Ok(CommandOutcome::success(
            format!("backup {}", env!("CARGO_PKG_VERSION")),
            "",
            Vec::new(),
        )),
        Command::Uninstall { yes, purge } => {
            let output = crate::commands::uninstall::perform_uninstall_with_executor_at_path(
                &context.profiles_path,
                adapters.resticprofile,
                adapters.command,
                yes,
                purge,
            )?;
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                Vec::new(),
                vec!["uninstall scope removed".into()],
            ))
        }
    }
}

fn dispatch_run(
    context: &CliRuntimeContext,
    adapters: &AdapterSet<'_>,
    profile: Option<String>,
    options: crate::commands::run::PipelineOptions,
) -> Result<CommandOutcome> {
    let config = load_profiles(context)?;
    let profiles = resolve_profiles(&config, profile.as_deref())?;
    let report_profile = profile.clone().unwrap_or_else(|| "all".into());
    let database_profile = config
        .application
        .as_ref()
        .and_then(|application| application.database.as_ref())
        .map(|database| database.profile.as_str());
    let mut stage = "profile resolution";

    let result = (|| -> Result<(String, Option<String>, Option<String>)> {
        let mut primary_results = Vec::new();
        if !options.skip_database
            && config
                .application
                .as_ref()
                .and_then(|application| application.database.as_ref())
                .is_some_and(|database| profiles.contains(&database.profile))
        {
            stage = "database";
            primary_results.push(crate::commands::run::run_database_stage(
                &config,
                &context.profiles_path,
                adapters.restic,
                options.dry_run,
            )?);
        }

        let ordinary_profiles = profiles
            .iter()
            .filter(|profile| Some(profile.as_str()) != database_profile)
            .cloned()
            .collect::<Vec<_>>();
        stage = "primary backup";
        for target_profile in &ordinary_profiles {
            primary_results.push(crate::commands::run::execute_run_profile(
                &context.profiles_path,
                target_profile,
                &options,
                adapters.resticprofile,
            )?);
        }

        let secondary_result = if !options.skip_secondary_sync {
            stage = "secondary sync";
            let copies = crate::commands::run::execute_secondary_copies(
                &config,
                &context.profiles_path,
                &ordinary_profiles,
                options.dry_run,
                adapters.resticprofile,
            )?;
            (!copies.is_empty()).then(|| copies.join("\n"))
        } else {
            None
        };
        let retention_result = if !options.skip_retention && !options.dry_run {
            stage = "retention";
            let results = ordinary_profiles
                .iter()
                .map(|profile| {
                    crate::commands::run::execute_retention(
                        &context.profiles_path,
                        profile,
                        adapters.resticprofile,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            Some(results.join("\n"))
        } else {
            None
        };
        Ok((
            primary_results.join("\n"),
            secondary_result,
            retention_result,
        ))
    })();

    match result {
        Ok((primary, secondary, retention)) => {
            let report = crate::commands::run::ExecutionReport::success_with_dry_run(
                &report_profile,
                primary.clone(),
                secondary.clone(),
                retention.clone(),
                options.dry_run,
            );
            let report_path = crate::commands::run::write_execution_report_from_profiles(
                &config,
                &context.profiles_path,
                report,
            )?;
            let mut output = primary;
            if let Some(result) = secondary {
                output.push_str(&format!("\n[Pipeline] Snapshot copy completed: {result}"));
            }
            if let Some(result) = retention {
                output.push_str(&format!("\n[Pipeline] Retention prune completed: {result}"));
            }
            output.push_str(&format!(
                "\n[Pipeline] Execution report: {}",
                report_path.display()
            ));
            Ok(CommandOutcome::success_with_changes(
                output,
                "",
                vec![report_path],
                if options.dry_run {
                    Vec::new()
                } else {
                    vec!["backup pipeline executed".into()]
                },
            ))
        }
        Err(error) => {
            let report_path = crate::commands::run::write_execution_report_from_profiles(
                &config,
                &context.profiles_path,
                crate::commands::run::ExecutionReport::failure(&report_profile, stage, &error),
            )
            .map_err(|report_error| {
                anyhow::anyhow!(
                    "stage '{stage}' failed: {error}; execution report failed: {report_error}"
                )
            })?;
            Err(anyhow::Error::new(
                crate::commands::run::RunCommandFailure {
                    message: format!("stage '{stage}' failed: {error}"),
                    artifacts: vec![report_path],
                    external_state_changes: vec![format!("stage '{stage}' attempted")],
                },
            ))
        }
    }
}

fn command_name(command: &Command) -> String {
    match command {
        Command::Setup { .. } => "setup",
        Command::Copy { .. } => "copy",
        Command::Run { .. } => "run",
        Command::Database { .. } => "database",
        Command::Doctor => "doctor",
        Command::Report { .. } => "report",
        Command::Schedule { .. } => "schedule",
        Command::Restore { .. } => "restore",
        Command::Snapshots => "snapshots",
        Command::Status { .. } => "status",
        Command::Update => "update",
        Command::Version => "version",
        Command::Uninstall { .. } => "uninstall",
    }
    .into()
}

fn required_profiles_path(context: &CliRuntimeContext) -> Result<PathBuf> {
    if context.profiles_path.as_os_str().is_empty()
        || context.profiles_path.to_string_lossy().trim().is_empty()
    {
        anyhow::bail!("--profiles path cannot be empty");
    }
    Ok(context.profiles_path.clone())
}

fn load_profiles(context: &CliRuntimeContext) -> Result<ResticProfileConfig> {
    let path = required_profiles_path(context)?;
    require_regular_profiles_file(&path)?;
    ResticProfileConfig::load_from_path(&path).with_context(|| {
        format!(
            "failed to load unified profiles configuration at {}",
            path.display()
        )
    })
}

fn require_regular_profiles_file(path: &std::path::Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        anyhow::anyhow!(
            "Unified profiles configuration not found at {}: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        anyhow::bail!(
            "Unified profiles configuration must be a regular file: {}",
            path.display()
        );
    }
    Ok(())
}

fn resolve_profiles(config: &ResticProfileConfig, profile: Option<&str>) -> Result<Vec<String>> {
    let profiles =
        crate::config::profile_resolver::ProfileResolver::resolve_for_run(config, profile)?
            .into_iter()
            .map(|profile| profile.name)
            .collect::<Vec<_>>();
    if profiles.is_empty() {
        anyhow::bail!("No Backup Profiles are configured for backup run");
    }
    Ok(profiles)
}

fn backend_initialization_targets(config: &ResticProfileConfig) -> Result<Vec<String>> {
    config.backend_initialization_targets()
}

fn report_paths_from_output(output: &str) -> Vec<PathBuf> {
    crate::commands::report::saved_report_paths(output)
}

fn report_uses_machine_readable_output(
    action: Option<&ReportAction>,
    format: Option<ReportFormat>,
) -> bool {
    matches!(format, Some(ReportFormat::Json))
        || action.is_some_and(|action| match action {
            ReportAction::Environment { format, .. }
            | ReportAction::TimeSync { format, .. }
            | ReportAction::RestoreDrill { format, .. } => {
                matches!(format, Some(ReportFormat::Json))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::report_paths_from_output;
    use std::path::PathBuf;

    #[test]
    fn report_artifact_parser_keeps_every_saved_path() {
        assert_eq!(
            report_paths_from_output("ISMS report saved to /tmp/report.html, /tmp/report.json"),
            vec![
                PathBuf::from("/tmp/report.html"),
                PathBuf::from("/tmp/report.json")
            ]
        );
        assert_eq!(
            report_paths_from_output(
                "All 3 sub-reports generated successfully:\nISMS report saved to /tmp/a.html, /tmp/a.json"
            ),
            vec![PathBuf::from("/tmp/a.html"), PathBuf::from("/tmp/a.json")]
        );
    }
}
