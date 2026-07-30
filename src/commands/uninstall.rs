use anyhow::Result;
use std::path::Path;
use crate::runner::resticprofile::ResticProfileRunner;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer\n- Systemd timers via resticprofile unschedule --all\n- /etc/systemd/system/resticprofile-backup@*".into()
}

pub fn perform_uninstall<R: ResticProfileRunner>(config_path: &Path, runner: &R, yes: bool, purge: bool) -> Result<String> {
    use std::io::IsTerminal;
    if !yes {
        let is_cargo_test = std::env::var("CARGO_MANIFEST_DIR").is_ok() || std::env::var("CARGO").is_ok();
        if !is_cargo_test && std::io::stdin().is_terminal() {
            let confirm = inquire::Confirm::new("Are you sure you want to uninstall backup CLI and configs?")
                .with_default(false)
                .prompt()?;
            if !confirm {
                return Ok("Uninstallation cancelled.".into());
            }
        } else {
            return Err(anyhow::anyhow!("Uninstallation requires --yes flag in non-interactive environments"));
        }
    }

    let _ = runner.schedule_disable(config_path);

    // Clean up systemd timer/service files if they exist in /etc/systemd/system
    let mut systemd_removed = false;
    if let Ok(entries) = std::fs::read_dir("/etc/systemd/system") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("resticprofile-backup@") || name_str.starts_with("backup.service") || name_str.starts_with("backup.timer") {
                let path = entry.path();
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(path);
                } else {
                    let _ = std::fs::remove_file(path);
                }
                systemd_removed = true;
            }
        }
    }

    if systemd_removed {
        let _ = std::process::Command::new("systemctl")
            .arg("daemon-reload")
            .status();
    }

    // Remove backup binary if exists in /usr/local/sbin/backup
    let default_binary = Path::new("/usr/local/sbin/backup");
    if default_binary.exists() {
        let _ = std::fs::remove_file(default_binary);
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if current_exe.exists() && current_exe != default_binary {
            let _ = std::fs::remove_file(current_exe);
        }
    }

    // Purge config directory if requested
    if purge {
        if let Some(parent_dir) = config_path.parent() {
            if parent_dir.exists() {
                let _ = std::fs::remove_dir_all(parent_dir);
            }
        }
    }

    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}


