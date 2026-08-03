use backup::i18n::Language;
use backup::runner::resticprofile::ResticProfileRunner;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "backup", version)]
struct Cli {
    /// Unified resticprofile v2 and application configuration file path.
    #[arg(long, global = true, value_name = "PATH")]
    profiles: Option<PathBuf>,

    /// Verbosity level (-v for debug, -vv for trace)
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet mode (only warn/error logs)
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Log file path
    #[arg(long, global = true, value_name = "PATH")]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        action: Option<backup::commands::report::ReportAction>,
        /// Report file path / 보고서 파일 저장 경로
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Report format (html, json) / 보고서 포맷
        #[arg(long)]
        format: Option<backup::commands::report::ReportFormat>,
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
        target: String,
        #[arg(long)]
        force: bool,
        /// Storage to restore from (primary or secondary)
        #[arg(long, value_enum, default_value_t = backup::commands::restore::RestoreStorage::Primary)]
        storage: backup::commands::restore::RestoreStorage,
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
enum SetupAction {
    /// Verify and download required binary dependencies (restic, rclone, resticprofile)
    Dependencies,
    /// Initialize primary and secondary Backend Adapter repositories
    BackendInit,
}

#[derive(Subcommand)]
enum ScheduleAction {
    /// Enable systemd timers / cron fallback
    Enable,
    /// Disable scheduled timers
    Disable,
    /// Display timer/scheduler status
    Status,
}

fn main() -> anyhow::Result<()> {
    let lang = Language::detect();
    let base_cmd = Cli::command();
    let localized_cmd = backup::i18n::CliHelp::get(lang).apply_to_command(base_cmd);
    let matches = localized_cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let env_override = std::env::var("BACKUP_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .ok();
    let level_filter =
        backup::logger::determine_level_filter(cli.verbose, cli.quiet, env_override.as_deref());
    let log_config = backup::logger::LogConfig::new(level_filter, cli.log_file.clone());
    let _ = backup::logger::init_logging(log_config);

    let profiles_path = cli
        .profiles
        .unwrap_or_else(|| PathBuf::from(backup::config::model::DEFAULT_PROFILES_PATH));
    let executor = backup::runner::executor::SystemExecutor;
    let rclone = backup::runner::rclone::RcloneTool::new(&executor);
    let resticprofile = backup::runner::resticprofile::ResticProfileTool::new(&executor);
    let restic = backup::runner::restic::ResticTool::new(&executor);
    let scheduler_binary = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("backup"))
        .to_string_lossy()
        .into_owned();
    let scheduler = backup::runner::scheduler::SystemScheduler::new(&executor, scheduler_binary);

    match cli.command {
        Commands::Setup {
            lang,
            non_interactive,
            action,
        } => match action {
            Some(SetupAction::Dependencies) => {
                let out = backup::commands::setup::run_setup_dependencies()?;
                println!("{}", out);
            }
            Some(SetupAction::BackendInit) => {
                let parsed =
                    backup::config::model::ResticProfileConfig::load_from_path(&profiles_path)?;
                let names = parsed.profile_names();
                if names.is_empty() {
                    anyhow::bail!("No Backup Profiles are configured for backend initialization");
                }
                for name in names {
                    println!(
                        "=== Initializing Backend Storage for Profile: [{}] ===",
                        name
                    );
                    println!("{}", resticprofile.init(&profiles_path, &name)?.trim_end());
                }
                init_secondary_backend_if_present(&profiles_path, &resticprofile, &executor, true)?;
            }
            None => {
                let prompter = backup::commands::setup::InquirePrompter;
                let lang_opt = lang.as_deref().map(backup::i18n::Language::from_str);
                backup::commands::setup::run_setup_with_prompter_and_runners(
                    &profiles_path,
                    &prompter,
                    non_interactive,
                    lang_opt,
                    &resticprofile,
                    &scheduler,
                )?;
                println!("Setup completed successfully.");
            }
        },

        Commands::Copy { profile, dry_run } => {
            let target_profile = profile.as_deref().unwrap_or("default");
            let out = backup::commands::copy::execute_copy(
                &resticprofile,
                &profiles_path,
                target_profile,
                dry_run,
            )?;
            println!("{}", out);
        }

        Commands::Run {
            profile,
            skip_database,
            skip_secondary_sync,
            skip_retention,
            dry_run,
        } => {
            let config = backup::config::model::BackupConfig::load_from_path(&profiles_path)?;
            let profiles_config =
                backup::config::model::ResticProfileConfig::load_from_path(&profiles_path)?;
            configure_profile_environment(&config);
            let opts = backup::commands::run::PipelineOptions {
                skip_database,
                skip_secondary_sync,
                skip_retention,
                dry_run,
            };

            let report_profile = profile.clone().unwrap_or_else(|| "all".into());
            let mut stage = "profile resolution";
            let outcome = (|| -> anyhow::Result<(String, Option<String>, Option<String>)> {
                let profiles_to_run =
                    backup::commands::run::resolve_profiles(&profiles_path, profile.as_deref())?;
                let database_profile = profiles_config
                    .application
                    .as_ref()
                    .and_then(|application| application.database.as_ref())
                    .map(|database| database.profile.as_str());
                let mut primary_results = Vec::new();
                if !skip_database
                    && profiles_config
                        .application
                        .as_ref()
                        .and_then(|application| application.database.as_ref())
                        .is_some_and(|database| profiles_to_run.contains(&database.profile))
                    && matches!(
                        config.backup.backup_type,
                        backup::config::model::BackupType::DbStream { .. }
                    )
                {
                    stage = "database";
                    primary_results.push(backup::commands::run::run_database_stage(
                        &config, &restic, dry_run,
                    )?);
                }
                let ordinary_profiles: Vec<_> = profiles_to_run
                    .iter()
                    .filter(|profile| Some(profile.as_str()) != database_profile)
                    .collect();
                stage = "primary backup";
                for target_profile in &ordinary_profiles {
                    primary_results.push(backup::commands::run::execute_run_profile(
                        &profiles_path,
                        target_profile,
                        &opts,
                        &resticprofile,
                    )?);
                }
                let secondary_result = if !skip_secondary_sync {
                    stage = "secondary sync";
                    let copies = backup::commands::run::execute_secondary_copies(
                        &profiles_config,
                        &profiles_path,
                        &ordinary_profiles
                            .iter()
                            .map(|profile| (*profile).clone())
                            .collect::<Vec<_>>(),
                        dry_run,
                        &resticprofile,
                    )?;
                    (!copies.is_empty()).then(|| copies.join("\n"))
                } else {
                    None
                };
                let retention_result = if !skip_retention && !dry_run {
                    stage = "retention";
                    let mut results = Vec::new();
                    for target_profile in &ordinary_profiles {
                        results.push(backup::commands::run::execute_retention(
                            &profiles_path,
                            target_profile,
                            &resticprofile,
                        )?);
                    }
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
            match outcome {
                Ok((primary, secondary, retention)) => {
                    println!("{primary}");
                    if let Some(result) = &secondary {
                        println!("[Pipeline] Snapshot copy completed: {result}");
                    }
                    if let Some(result) = &retention {
                        println!("[Pipeline] Retention prune completed: {result}");
                    }
                    let path = backup::commands::run::write_execution_report(
                        &config,
                        backup::commands::run::ExecutionReport::success(
                            &report_profile,
                            primary,
                            secondary,
                            retention,
                        ),
                    )?;
                    println!("[Pipeline] Execution report: {}", path.display());
                }
                Err(error) => {
                    let path = backup::commands::run::write_execution_report(
                        &config,
                        backup::commands::run::ExecutionReport::failure(
                            &report_profile,
                            stage,
                            &error,
                        ),
                    )?;
                    tracing::error!("[Pipeline] Execution report: {}", path.display());
                    return Err(error);
                }
            }
        }
        Commands::Database { dry_run } => {
            let config = backup::config::model::BackupConfig::load_from_path(&profiles_path)?;
            configure_profile_environment(&config);
            println!(
                "{}",
                backup::commands::database::execute_database_backup(&config, &restic, dry_run)?
            );
        }

        Commands::Doctor => {
            let out = backup::commands::doctor::run_doctor_checks(&rclone, Some(&profiles_path))?;
            println!("{}", out);
        }
        Commands::Report {
            action,
            file,
            format,
        } => {
            let config = if profiles_path.exists() {
                backup::config::model::BackupConfig::load_from_path(&profiles_path)?
            } else {
                backup::config::model::BackupConfig::default()
            };
            configure_profile_environment(&config);
            let out = backup::commands::report::ReportCommand::run(action, file, format, &config)?;
            println!("{}", out);
        }
        Commands::Schedule { action } => match action {
            ScheduleAction::Enable => {
                let out = backup::commands::schedule::execute_schedule_enable(
                    &profiles_path,
                    &scheduler,
                )?;
                println!("{}", out);
            }
            ScheduleAction::Disable => {
                let out = backup::commands::schedule::execute_schedule_disable(
                    &profiles_path,
                    &scheduler,
                )?;
                println!("{}", out);
            }
            ScheduleAction::Status => {
                let out = backup::commands::schedule::execute_schedule_status(
                    &profiles_path,
                    &scheduler,
                )?;
                println!("{}", out);
            }
        },
        Commands::Restore {
            snapshot,
            target,
            force,
            storage,
        } => {
            let config = backup::config::model::BackupConfig::load_from_path(&profiles_path)?;
            configure_profile_environment(&config);
            let out = backup::commands::restore::execute_restore_from_storage(
                &config, &restic, &snapshot, &target, force, storage,
            )?;
            println!("{}", out);
        }
        Commands::Snapshots => {
            let config = backup::config::model::BackupConfig::load_from_path(&profiles_path)?;
            configure_profile_environment(&config);
            let out = backup::commands::snapshots::execute_snapshots(&config, &restic)?;
            println!("{}", out);
        }
        Commands::Status { profile } => {
            let out = backup::commands::status::execute_status_from_profiles_config(
                &profiles_path,
                profile.as_deref(),
                &resticprofile,
            )?;
            println!("{}", out);
        }
        Commands::Update => {
            let out = backup::commands::update::execute_update_check(env!("CARGO_PKG_VERSION"))?;
            println!("{}", out);
        }
        Commands::Version => {
            println!("backup {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Uninstall { yes, purge } => {
            let out = backup::commands::uninstall::perform_uninstall_at_path(
                &profiles_path,
                &resticprofile,
                yes,
                purge,
            )?;
            println!("{}", out);
        }
    }
    Ok(())
}

fn configure_profile_environment(config: &backup::config::model::BackupConfig) {
    let set_s3_environment = |prefix: &str, s3: Option<&backup::config::model::S3Config>| {
        if let Some(s3) = s3 {
            // SAFETY: this single-threaded CLI sets child-process inputs before starting commands.
            unsafe {
                std::env::set_var(
                    format!("{prefix}_AWS_ACCESS_KEY_ID"),
                    secrecy::ExposeSecret::expose_secret(&s3.access_key_id),
                );
                std::env::set_var(
                    format!("{prefix}_AWS_SECRET_ACCESS_KEY"),
                    secrecy::ExposeSecret::expose_secret(&s3.secret_access_key),
                );
            }
        }
    };
    set_s3_environment("BACKUP_PRIMARY", config.storage.primary.s3.as_ref());
    if let Some(secondary) = &config.storage.secondary {
        set_s3_environment("BACKUP_SECONDARY", secondary.s3.as_ref());
    }
}

fn init_secondary_backend_if_present<
    R: ResticProfileRunner,
    E: backup::runner::executor::CommandRunner,
>(
    profiles_path: &std::path::Path,
    resticprofile: &R,
    executor: &E,
    verbose: bool,
) -> anyhow::Result<()> {
    let parsed = backup::config::model::ResticProfileConfig::load_from_path(profiles_path)?;
    if let Some(sec_profile) = parsed.profiles.get("secondary") {
        let repo = sec_profile.repository.as_deref().unwrap_or("");
        if repo.starts_with("sftp:") {
            let backup_config = backup::config::model::BackupConfig::load_from_path(profiles_path)?;
            let sftp_conf = backup_config
                .storage
                .secondary
                .as_ref()
                .and_then(|s| s.sftp.as_ref());

            if let Some(sftp) = sftp_conf {
                let key_path = sftp.key_file.as_deref().unwrap_or("");
                if !key_path.is_empty() {
                    if let Err(reason) = backup::commands::setup::verify_sftp_connection(
                        &sftp.user, &sftp.host, sftp.port, key_path, executor,
                    ) {
                        anyhow::bail!(
                            "Secondary SFTP storage connection verification failed: {}",
                            reason
                        );
                    }
                }
            }
        }

        if verbose {
            println!("=== Initializing Secondary Backend Storage for Profile: [secondary] ===");
        }
        let res = resticprofile.init(profiles_path, "secondary")?;
        if verbose && !res.trim().is_empty() {
            println!("{}", res.trim_end());
        }
    }
    Ok(())
}
