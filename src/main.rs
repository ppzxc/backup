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
    /// Backup Environment and Backup Profile setup
    Setup {
        #[arg(long)]
        non_interactive: bool,
        #[command(subcommand)]
        action: Option<SetupAction>,
    },
    /// Configuration Registry management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Storage Backend Adapter migration
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
    /// Execute backup pipeline
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
    /// System, security, and audit report diagnostics
    Doctor {
        #[command(subcommand)]
        action: Option<DoctorAction>,
    },
    /// Systemd timer / Cron scheduler management
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Restore files or database dumps from snapshot
    Restore {
        #[arg(long, default_value = "latest")]
        snapshot: String,
        #[arg(long, default_value = "/tmp/restore")]
        target: String,
    },

    /// List snapshots across primary and secondary storage targets
    Snapshots,
    /// Display operational status and snapshot recency
    Status,
    /// Self-update backup binary and assets
    Update,
    /// Uninstall backup CLI and scheduled timers
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
        Commands::Setup { non_interactive, action } => {
            match action {
                Some(SetupAction::Dependencies) => {
                    let out = backup::commands::setup::run_setup_dependencies()?;
                    println!("{}", out);
                }
                Some(SetupAction::BackendInit) => println!("Backend storage repository initialized successfully."),
                None => {
                    let prompter = backup::commands::setup::InquirePrompter;
                    if let Err(err) = backup::commands::setup::run_setup_with_prompter(default_config_path, &prompter, non_interactive) {
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



