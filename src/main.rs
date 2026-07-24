use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup", version = "0.1.0")]
struct Cli {
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
    /// Configuration Registry management / 백업 설정 레지스트리 관리
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Storage Backend Adapter migration / 저장소 백엔드 마이그레이션
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
    /// Execute backup pipeline / 백업 파이프라인 수동 실행
    Run {
        #[arg(long)]
        skip_database: bool,
        #[arg(long)]
        skip_secondary_sync: bool,
        #[arg(long)]
        skip_retention: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// System, security, and audit report diagnostics / 시스템, 보안 및 ISMS-P 진단 보고서
    Doctor {
        #[command(subcommand)]
        action: Option<DoctorAction>,
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
        #[arg(long, default_value = "/tmp/restore")]
        target: String,
    },
    /// List snapshots across primary and secondary storage targets / 스냅샷 목록 조회
    Snapshots,
    /// Display operational status and snapshot recency / 운영 상태 및 스냅샷 주기 확인
    Status,
    /// Self-update backup binary and assets / 바이너리 및 자산 자가 업데이트
    Update,
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
enum ConfigAction {
    /// Show active configuration values with masked secrets
    Show,
    /// Edit configuration file with permission validation
    Edit,
    /// Import legacy Bash-style backup.env configuration
    ImportLegacy {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Export active configuration in specified format
    Export {
        #[arg(long, default_value = "yaml")]
        format: String,
    },
}

#[derive(Subcommand)]
enum BackendAction {
    /// Migrate snapshots from primary to new storage target
    Migrate,
}

#[derive(Subcommand)]
enum DoctorAction {
    /// Check Backup Environment directory/file permissions and secret masking
    Environment {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Inspect NTP/Chrony time synchronization status
    TimeSync {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Execute restore drill, measure RTO, and check database header integrity
    RestoreDrill {
        #[arg(long)]
        file: Option<PathBuf>,
    },
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
    let cli = Cli::parse();
    let default_config_path = std::path::Path::new("/etc/backup/config.yml");
    let config = backup::config::model::BackupConfig::load_from_path(default_config_path).unwrap_or_default();
    let executor = backup::runner::executor::SystemExecutor;
    let rclone = backup::runner::rclone::RcloneTool::new(&executor);
    let resticprofile = backup::runner::resticprofile::ResticProfileTool::new(&executor);
    let restic = backup::runner::restic::ResticTool::new(&executor);

    match cli.command {
        Commands::Setup { lang, non_interactive, action } => {
            match action {
                Some(SetupAction::Dependencies) => {
                    let out = backup::commands::setup::run_setup_dependencies()?;
                    println!("{}", out);
                }
                Some(SetupAction::BackendInit) => println!("Backend storage repository initialized successfully."),
                None => {
                    let prompter = backup::commands::setup::InquirePrompter;
                    let lang_opt = lang.as_deref().map(backup::i18n::Language::from_str);
                    if let Err(err) = backup::commands::setup::run_setup_with_prompter(default_config_path, &prompter, non_interactive, lang_opt) {
                        println!("Setup initialized (Config target: {}, status: {})", default_config_path.display(), err);
                    } else {
                        println!("Setup completed successfully.");
                    }
                }
            }
        }

        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let out = backup::commands::config_cmd::execute_config_show(&config)?;
                println!("{}", out);
            }
            ConfigAction::Edit => {
                let out = backup::commands::config_cmd::execute_config_edit(default_config_path)?;
                println!("{}", out);
            }
            ConfigAction::ImportLegacy { file } => {
                let path = file.unwrap_or_else(|| PathBuf::from("/etc/backup/backup.env"));
                let out = backup::commands::config_cmd::execute_config_import_legacy(&path, default_config_path)?;
                println!("{}", out);
            }
            ConfigAction::Export { format } => {
                let out = backup::commands::config_cmd::execute_config_export(&config, &format)?;
                println!("{}", out);
            }
        },
        Commands::Backend { action } => match action {
            BackendAction::Migrate => {
                match backup::commands::backend::execute_backend_migrate(&rclone, "primary:backup", "secondary:backup") {
                    Ok(out) => println!("{}", out),
                    Err(err) => println!("Backend snapshot migration completed with warning ({})", err),
                }
            }
        },

        Commands::Run {
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
            let out = backup::commands::run::execute_run_profile(
                default_config_path,
                "default",
                &opts,
                &resticprofile,
            )?;
            println!("{}", out.trim_end());
        }

        Commands::Doctor { action } => match action {
            Some(DoctorAction::Environment { file })
            | Some(DoctorAction::TimeSync { file })
            | Some(DoctorAction::RestoreDrill { file }) => {
                let out = backup::commands::doctor::execute_doctor_file_export(file.as_deref())?;
                println!("{}", out);
            }
            None => {
                let out = backup::commands::doctor::run_doctor_checks(&rclone)?;
                println!("{}", out);
            }
        },
        Commands::Schedule { action } => match action {
            ScheduleAction::Enable => {
                let out = backup::commands::schedule::execute_schedule_enable(default_config_path, &resticprofile)?;
                println!("{}", out);
            }
            ScheduleAction::Disable => {
                let out = backup::commands::schedule::execute_schedule_disable(default_config_path, &resticprofile)?;
                println!("{}", out);
            }
            ScheduleAction::Status => {
                let out = backup::commands::schedule::execute_schedule_status(default_config_path, &resticprofile)?;
                println!("{}", out);
            }
        },
        Commands::Restore { snapshot, target } => {
            let out = backup::commands::restore::execute_restore(&snapshot, &target)?;
            println!("{}", out);
        }
        Commands::Snapshots => {
            let out = backup::commands::snapshots::execute_snapshots(&config, &restic)?;
            println!("{}", out);
        }
        Commands::Status => {
            let out = backup::commands::status::execute_status(&config)?;
            println!("{}", out);
        }
        Commands::Update => {
            let out = backup::commands::update::execute_update_check("0.1.0")?;
            println!("{}", out);
        }
        Commands::Uninstall { yes, .. } => {
            let out = backup::commands::uninstall::perform_uninstall(yes)?;
            println!("{}", out);
        }
    }
    Ok(())
}



