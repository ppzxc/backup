use backup::i18n::Language;
use backup::runner::resticprofile::ResticProfileRunner;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "backup", version)]
struct Cli {
    /// Backup Environment configuration file path.
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,
    /// Backup Profile configuration file path.
    #[arg(long, global = true, value_name = "PATH")]
    profiles: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Backup Environment and Backup Profile setup / 백업 환경 및 프로필 설정 마법사
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
    let profiles_path = cli
        .profiles
        .unwrap_or_else(|| PathBuf::from(backup::config::model::DEFAULT_PROFILES_PATH));
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from(backup::config::model::DEFAULT_CONFIG_PATH));
    let config =
        backup::config::model::BackupConfig::load_from_path(&config_path).unwrap_or_default();
    configure_profile_environment(&config);
    let executor = backup::runner::executor::SystemExecutor;
    let rclone = backup::runner::rclone::RcloneTool::new(&executor);
    let resticprofile = backup::runner::resticprofile::ResticProfileTool::new(&executor);
    let restic = backup::runner::restic::ResticTool::new(&executor);

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
                if let Ok(parsed) =
                    backup::config::model::ResticProfileConfig::load_from_path(&profiles_path)
                {
                    let names = parsed.profile_names();
                    for name in names {
                        println!(
                            "=== Initializing Backend Storage for Profile: [{}] ===",
                            name
                        );
                        match resticprofile.init(&profiles_path, &name) {
                            Ok(res) => println!("{}", res.trim_end()),
                            Err(err) => println!("Repository initialization note ({})", err),
                        }
                    }
                    init_secondary_backend_if_present(
                        &profiles_path,
                        &config_path,
                        &resticprofile,
                        &executor,
                        true,
                    );
                } else {
                    println!("Backend storage repository initialization initiated.");
                }
            }
            None => {
                let prompter = backup::commands::setup::InquirePrompter;
                let lang_opt = lang.as_deref().map(backup::i18n::Language::from_str);
                if let Err(err) = backup::commands::setup::run_setup_with_prompter_at_paths(
                    &config_path,
                    &profiles_path,
                    &prompter,
                    non_interactive,
                    lang_opt,
                ) {
                    println!(
                        "Setup initialized (Config target: {}, status: {})",
                        config_path.display(),
                        err
                    );
                } else {
                    println!("Setup completed successfully.");
                }
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
            let opts = backup::commands::run::PipelineOptions {
                skip_database,
                skip_secondary_sync,
                skip_retention,
                dry_run,
            };

            let profiles_to_run = if let Some(p) = profile {
                vec![p]
            } else if let Ok(parsed) =
                backup::config::model::ResticProfileConfig::load_from_path(&profiles_path)
            {
                let names = parsed.profile_names();
                if names.is_empty() {
                    vec!["default".to_string()]
                } else {
                    names
                }
            } else if !config.profile.is_empty() {
                vec![config.profile.clone()]
            } else {
                vec!["default".to_string()]
            };

            if !skip_database
                && matches!(
                    config.backup.backup_type,
                    backup::config::model::BackupType::DbStream { .. }
                )
            {
                println!(
                    "{}",
                    backup::commands::database::execute_database_backup(&config, &restic, dry_run)?
                );
            }
            for target_profile in &profiles_to_run {
                println!("=== Running Backup Profile: [{}] ===", target_profile);
                let out = backup::commands::run::execute_run_profile(
                    &profiles_path,
                    target_profile,
                    &opts,
                    &resticprofile,
                )?;
                println!("{}", out.trim_end());
            }
        }
        Commands::Database { dry_run } => println!(
            "{}",
            backup::commands::database::execute_database_backup(&config, &restic, dry_run)?
        ),

        Commands::Doctor => {
            let out = backup::commands::doctor::run_doctor_checks(&rclone, Some(&config_path))?;
            println!("{}", out);
        }
        Commands::Report {
            action,
            file,
            format,
        } => {
            let out = backup::commands::report::ReportCommand::run(action, file, format, &config)?;
            println!("{}", out);
        }
        Commands::Schedule { action } => match action {
            ScheduleAction::Enable => {
                let out = backup::commands::schedule::execute_schedule_enable(
                    &profiles_path,
                    &resticprofile,
                )?;
                println!("{}", out);
            }
            ScheduleAction::Disable => {
                let out = backup::commands::schedule::execute_schedule_disable(
                    &profiles_path,
                    &resticprofile,
                )?;
                println!("{}", out);
            }
            ScheduleAction::Status => {
                let out = backup::commands::schedule::execute_schedule_status(
                    &profiles_path,
                    &resticprofile,
                )?;
                println!("{}", out);
            }
        },
        Commands::Restore {
            snapshot,
            target,
            force,
        } => {
            let out = backup::commands::restore::execute_restore(
                &config, &restic, &snapshot, &target, force,
            )?;
            println!("{}", out);
        }
        Commands::Snapshots => {
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
            let out = backup::commands::uninstall::perform_uninstall_at_paths(
                &config_path,
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
                std::env::set_var(format!("{prefix}_AWS_ACCESS_KEY_ID"), &s3.access_key_id);
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
    config_path: &std::path::Path,
    resticprofile: &R,
    executor: &E,
    verbose: bool,
) {
    if let Ok(parsed) = backup::config::model::ResticProfileConfig::load_from_path(profiles_path) {
        if let Some(sec_profile) = parsed.profiles.get("secondary") {
            let repo = sec_profile.repository.as_deref().unwrap_or("");
            if repo.starts_with("sftp:") {
                let backup_config =
                    backup::config::model::BackupConfig::load_from_path(config_path).ok();
                let sftp_conf = backup_config
                    .as_ref()
                    .and_then(|c| c.storage.secondary.as_ref())
                    .and_then(|s| s.sftp.as_ref());

                if let Some(sftp) = sftp_conf {
                    let key_path = sftp.key_file.as_deref().unwrap_or("");
                    if !key_path.is_empty() {
                        if let Err(reason) = backup::commands::setup::verify_sftp_connection(
                            &sftp.user, &sftp.host, sftp.port, key_path, executor,
                        ) {
                            println!(
                                "[Notice] Secondary SFTP storage connection verification skipped init: {}",
                                reason
                            );
                            println!(
                                "[Notice] Register public key ({}.pub) in {}@{}:~/.ssh/authorized_keys and run 'backup setup backend-init'",
                                key_path, sftp.user, sftp.host
                            );
                            return;
                        }
                    }
                }
            }

            if verbose {
                println!("=== Initializing Secondary Backend Storage for Profile: [secondary] ===");
            }
            match resticprofile.init(profiles_path, "secondary") {
                Ok(res) => {
                    if verbose && !res.trim().is_empty() {
                        println!("{}", res.trim_end());
                    }
                }
                Err(err) => println!(
                    "[Warning] Secondary storage repository initialization failed ({})",
                    err
                ),
            }
        }
    }
}
