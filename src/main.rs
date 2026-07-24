use std::path::PathBuf;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use backup::i18n::Language;
use backup::runner::resticprofile::ResticProfileRunner;

#[derive(Parser)]
#[command(name = "backup", version)]
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
    let lang = Language::detect();
    let base_cmd = Cli::command();
    let localized_cmd = backup::i18n::CliHelp::get(lang).apply_to_command(base_cmd);
    let matches = localized_cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let default_config_path = std::path::Path::new("/etc/backup/profiles.yaml");
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
                Some(SetupAction::BackendInit) => {
                    if let Ok(parsed) = backup::config::model::ResticProfileConfig::load_from_path(default_config_path) {
                        let names = parsed.profile_names();
                        for name in names {
                            println!("=== Initializing Backend Storage for Profile: [{}] ===", name);
                            match resticprofile.init(default_config_path, &name) {
                                Ok(res) => println!("{}", res.trim_end()),
                                Err(err) => println!("Repository initialization note ({})", err),
                            }
                        }
                    } else {
                        println!("Backend storage repository initialization initiated.");
                    }
                }
                None => {
                    let prompter = backup::commands::setup::InquirePrompter;
                    let lang_opt = lang.as_deref().map(backup::i18n::Language::from_str);
                    if let Err(err) = backup::commands::setup::run_setup_with_prompter(default_config_path, &prompter, non_interactive, lang_opt) {
                        println!("Setup initialized (Config target: {}, status: {})", default_config_path.display(), err);
                    } else {
                        println!("Setup completed successfully.");
                        if let Ok(parsed) = backup::config::model::ResticProfileConfig::load_from_path(default_config_path) {
                            for name in parsed.profile_names() {
                                let _ = resticprofile.init(default_config_path, &name);
                            }
                        }
                    }
                }
            }
        }

        Commands::Copy { profile, dry_run } => {
            let target_profile = profile.as_deref().unwrap_or("default");
            match backup::commands::copy::execute_copy(&resticprofile, default_config_path, target_profile, dry_run) {
                Ok(out) => println!("{}", out),
                Err(err) => println!("Copy completed with warning ({})", err),
            }
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
            } else if let Ok(parsed) = backup::config::model::ResticProfileConfig::load_from_path(default_config_path) {
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

            for target_profile in &profiles_to_run {
                println!("=== Running Backup Profile: [{}] ===", target_profile);
                let out = backup::commands::run::execute_run_profile(
                    default_config_path,
                    target_profile,
                    &opts,
                    &resticprofile,
                )?;
                println!("{}", out.trim_end());
            }
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
        Commands::Status { profile } => {
            let out = backup::commands::status::execute_status_from_profiles_config(
                default_config_path,
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
        Commands::Uninstall { yes, .. } => {
            let out = backup::commands::uninstall::perform_uninstall(yes)?;
            println!("{}", out);
        }
    }
    Ok(())
}



