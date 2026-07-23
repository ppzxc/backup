use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    Restore,
    Snapshots,
    Status,
    Setup,
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Doctor,
    Update,
    Uninstall,
}

#[derive(Subcommand)]
enum ScheduleAction {
    Enable,
    Disable,
    Show,
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Edit,
    Import {
        #[arg(long)]
        legacy_env: bool,
    },
    Export,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => println!("Status: operational"),
        Commands::Run => println!("Running backup..."),
        _ => println!("Command executed"),
    }
}
