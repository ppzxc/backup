use backup::cli::{
    AdapterSelection, Cli, CliRuntimeContext, RestoreDrillSetupOverrides, SchedulerMode,
};
use backup::i18n::Language;
use backup::runner::executor::SystemExecutor;
use backup::runner::rclone::RcloneTool;
use backup::runner::restic::ResticTool;
use backup::runner::resticprofile::ResticProfileTool;
use backup::runner::scheduler::SystemScheduler;
use clap::FromArgMatches;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() {
    let raw_args = std::env::args_os().collect::<Vec<_>>();
    let language = match parser_language() {
        Ok(language) => language,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let localized_schema = backup::i18n::CliHelp::get(language)
        .apply_to_command(backup::cli::authoritative_cli_schema());
    let matches = match localized_schema.try_get_matches_from(raw_args.clone()) {
        Ok(matches) => matches,
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayVersion => {
            if let Some(log_file) = explicit_log_file(&raw_args) {
                if let Err(error) = backup::logger::init_logging(backup::logger::LogConfig::new(
                    "info",
                    Some(log_file),
                )) {
                    print_startup_error(format!("logging initialization failed: {error}"), 1);
                }
            }
            let exit_code = error.exit_code();
            let _ = error.print();
            std::process::exit(exit_code);
        }
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
            let exit_code = error.exit_code();
            let _ = error.print();
            std::process::exit(exit_code);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let env_log_override = std::env::var("BACKUP_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .ok();
    let context = match CliRuntimeContext::from_cli(
        &cli,
        language,
        env_log_override,
        SchedulerMode::Auto,
        AdapterSelection::System,
    ) {
        Ok(context) => {
            let home_dir = std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            let host_name = std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "localhost".into());
            let scheduler_calendar = std::env::var("BACKUP_TEST_SCHEDULE_CALENDAR")
                .unwrap_or_else(|_| backup::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR.into());
            let force_cron = std::env::var_os("BACKUP_TEST_FORCE_CRON").is_some();
            let restore_drill_setup_overrides =
                match RestoreDrillSetupOverrides::from_process_environment() {
                    Ok(overrides) => overrides,
                    Err(error) => print_startup_error(
                        format!("invalid restore drill setup override: {error}"),
                        2,
                    ),
                };
            context
                .with_environment(home_dir, host_name, scheduler_calendar)
                .with_scheduler_force_cron(force_cron)
                .with_restore_drill_setup_overrides(restore_drill_setup_overrides)
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    if !matches!(&cli.command, backup::cli::Command::Version) || context.logging.log_file.is_some()
    {
        if let Err(error) = backup::logger::init_logging(backup::logger::LogConfig::new(
            context.logging.level_filter.clone(),
            context.logging.log_file.clone(),
        )) {
            print_startup_error(format!("logging initialization failed: {error}"), 1);
        }
    }

    let executor = SystemExecutor;
    let rclone = RcloneTool::new(&executor);
    let resticprofile = ResticProfileTool::new(&executor);
    let restic = ResticTool::new(&executor);
    let scheduler_binary = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("backup"))
        .to_string_lossy()
        .into_owned();
    let scheduler = SystemScheduler::new(&executor, scheduler_binary);
    let adapters = backup::cli::AdapterSet {
        command: &executor,
        rclone: &rclone,
        restic: &restic,
        resticprofile: &resticprofile,
        scheduler: &scheduler,
        selection: AdapterSelection::System,
    };

    let outcome = backup::cli::dispatch(&context, cli.command, &adapters);
    if !outcome.stdout.is_empty() {
        println!("{}", outcome.stdout);
    }
    if !outcome.stderr.is_empty() {
        eprintln!("{}", outcome.stderr);
    }
    if !outcome.is_success() {
        std::process::exit(outcome.exit_status);
    }
}

fn print_startup_error(message: impl Into<String>, exit_code: i32) -> ! {
    let error = clap::Error::raw(clap::error::ErrorKind::Io, message.into());
    let _ = error.print();
    std::process::exit(exit_code);
}

fn parser_language() -> anyhow::Result<Language> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(setup_index) = args.iter().position(|arg| arg == "setup") else {
        return Ok(Language::detect());
    };
    for arg in args.iter().skip(setup_index + 1) {
        if let Some(value) = arg.strip_prefix("--lang=") {
            return backup::cli::parse_language(value);
        }
    }
    if let Some(index) = args
        .iter()
        .skip(setup_index + 1)
        .position(|arg| arg == "--lang")
    {
        let value_index = setup_index + 1 + index + 1;
        if let Some(value) = args.get(value_index) {
            return backup::cli::parse_language(value);
        }
    }
    Ok(Language::detect())
}

fn explicit_log_file(args: &[OsString]) -> Option<PathBuf> {
    let mut path = None;
    let mut arguments = args.iter().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--log-file" {
            path = arguments.next().cloned().map(PathBuf::from);
        } else if let Some(value) = argument
            .to_str()
            .and_then(|value| value.strip_prefix("--log-file="))
        {
            path = Some(PathBuf::from(value));
        }
    }
    path
}
