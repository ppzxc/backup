use crate::runner::executor::{CommandRunner, SystemExecutor};
use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::Result;
use std::path::Path;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer\n- Systemd timers via resticprofile unschedule --all\n- /etc/systemd/system/resticprofile-backup@*".into()
}

pub fn perform_uninstall<R: ResticProfileRunner>(
    config_path: &Path,
    runner: &R,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_at_paths(config_path, config_path, runner, yes, purge)
}

pub fn perform_uninstall_at_paths<R: ResticProfileRunner>(
    config_path: &Path,
    profiles_path: &Path,
    runner: &R,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_with_executor_at_paths(
        config_path,
        profiles_path,
        runner,
        &SystemExecutor,
        yes,
        purge,
    )
}

pub fn perform_uninstall_with_executor<R: ResticProfileRunner, E: CommandRunner>(
    config_path: &Path,
    runner: &R,
    executor: &E,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_with_executor_at_paths(config_path, config_path, runner, executor, yes, purge)
}

pub fn perform_uninstall_with_executor_at_paths<R: ResticProfileRunner, E: CommandRunner>(
    config_path: &Path,
    profiles_path: &Path,
    runner: &R,
    executor: &E,
    yes: bool,
    purge: bool,
) -> Result<String> {
    use std::io::IsTerminal;
    if !yes {
        let is_cargo_test =
            std::env::var("CARGO_MANIFEST_DIR").is_ok() || std::env::var("CARGO").is_ok();
        if !is_cargo_test && std::io::stdin().is_terminal() {
            let confirm =
                inquire::Confirm::new("Are you sure you want to uninstall backup CLI and configs?")
                    .with_default(false)
                    .prompt()?;
            if !confirm {
                return Ok("Uninstallation cancelled.".into());
            }
        } else {
            return Err(anyhow::anyhow!(
                "Uninstallation requires --yes flag in non-interactive environments"
            ));
        }
    }

    if profiles_path.exists() {
        runner.schedule_disable(profiles_path)?;
    }

    // Clean up systemd timer/service files if they exist in /etc/systemd/system
    let mut systemd_removed = false;
    if let Ok(entries) = std::fs::read_dir("/etc/systemd/system") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("resticprofile-backup@")
                || name_str.starts_with("backup.service")
                || name_str.starts_with("backup.timer")
            {
                let path = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
                systemd_removed = true;
            }
        }
    }

    if systemd_removed {
        let output = executor.run("systemctl", &["daemon-reload"])?;
        if output.status_code != 0 {
            anyhow::bail!("systemctl daemon-reload failed: {}", output.stderr);
        }
    }

    // Remove backup binary if exists in /usr/local/sbin/backup
    let default_binary = Path::new("/usr/local/sbin/backup");
    if default_binary.exists() {
        std::fs::remove_file(default_binary)?;
    }

    // Purge config directory if requested
    if purge {
        if let Some(parent_dir) = config_path.parent() {
            if parent_dir.exists() {
                std::fs::remove_dir_all(parent_dir)?;
            }
        }
    }

    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}
