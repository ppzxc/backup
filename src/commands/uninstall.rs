use crate::runner::executor::{CommandRunner, SystemExecutor};
use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallTargets {
    pub binary_path: PathBuf,
    pub systemd_dir: PathBuf,
}

impl Default for UninstallTargets {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from("/usr/local/sbin/backup"),
            systemd_dir: PathBuf::from("/etc/systemd/system"),
        }
    }
}

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/profiles.yaml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer\n- Systemd timers via resticprofile unschedule --all\n- /etc/systemd/system/resticprofile-backup@*".into()
}

pub fn perform_uninstall<R: ResticProfileRunner>(
    profiles_path: &Path,
    runner: &R,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_at_path(profiles_path, runner, yes, purge)
}

pub fn perform_uninstall_at_path<R: ResticProfileRunner + ?Sized>(
    profiles_path: &Path,
    runner: &R,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_with_executor_at_path(profiles_path, runner, &SystemExecutor, yes, purge)
}

pub fn perform_uninstall_with_executor<R: ResticProfileRunner, E: CommandRunner>(
    profiles_path: &Path,
    runner: &R,
    executor: &E,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_with_executor_at_path(profiles_path, runner, executor, yes, purge)
}

pub fn perform_uninstall_with_executor_at_path<
    R: ResticProfileRunner + ?Sized,
    E: CommandRunner + ?Sized,
>(
    profiles_path: &Path,
    runner: &R,
    executor: &E,
    yes: bool,
    purge: bool,
) -> Result<String> {
    perform_uninstall_with_executor_at_path_and_targets(
        profiles_path,
        runner,
        executor,
        yes,
        purge,
        &UninstallTargets::default(),
    )
}

pub fn perform_uninstall_with_executor_at_path_and_targets<
    R: ResticProfileRunner + ?Sized,
    E: CommandRunner + ?Sized,
>(
    profiles_path: &Path,
    runner: &R,
    executor: &E,
    yes: bool,
    purge: bool,
    targets: &UninstallTargets,
) -> Result<String> {
    tracing::info!(purge = %purge, "Executing backup CLI uninstallation");
    use std::io::IsTerminal;
    // Purge is deliberately non-interactive: allowing a positive prompt answer here would make
    // an omitted --yes indistinguishable from an accidental destructive invocation.
    if purge && !yes {
        return Err(anyhow::anyhow!(
            "uninstall --purge requires --yes and made no changes"
        ));
    }
    if !yes {
        if std::io::stdin().is_terminal() {
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

    // Scheduler cleanup is configuration-independent.  When the profiles file still exists, the
    // resticprofile adapter removes its configured schedules.  If it is already gone, remove the
    // CLI-owned Cron marker directly so stale schedules are not left behind.
    if profiles_path.exists() {
        runner.schedule_disable(profiles_path)?;
    } else {
        remove_owned_cron_entries(executor)?;
    }

    let systemd_removed = remove_owned_systemd_units(&targets.systemd_dir)?;

    if systemd_removed {
        let output = executor.run("systemctl", &["daemon-reload"])?;
        if output.status_code != 0 {
            anyhow::bail!("systemctl daemon-reload failed: {}", output.stderr);
        }
    }

    // Scheduler cleanup is complete before the executable is removed.
    if targets.binary_path.exists() {
        std::fs::remove_file(&targets.binary_path)?;
    }

    // Purge only files owned by this configuration scope.  In particular, never remove the
    // entire parent directory: callers may keep unrelated host data beside profiles.yaml.
    if purge {
        purge_configuration_scope(profiles_path)?;
    }

    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}

fn remove_owned_systemd_units(systemd_dir: &Path) -> Result<bool> {
    if !systemd_dir.exists() {
        return Ok(false);
    }
    let mut removed = false;
    for entry in std::fs::read_dir(systemd_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("resticprofile-backup@")
            || name == "backup.service"
            || name == "backup.timer"
        {
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
            removed = true;
        }
    }
    Ok(removed)
}

fn remove_owned_cron_entries<E: CommandRunner + ?Sized>(executor: &E) -> Result<()> {
    use std::io::Write;

    let output = executor.run("crontab", &["-l"])?;
    if output.status_code != 0 {
        if output.status_code == 1 && output.stderr.to_ascii_lowercase().contains("no crontab") {
            return Ok(());
        }
        anyhow::bail!("crontab listing failed: {}", output.stderr.trim());
    }
    if !output
        .stdout
        .lines()
        .any(|line| line.contains("# backup-pipeline"))
    {
        return Ok(());
    }

    let filtered = output
        .stdout
        .lines()
        .filter(|line| !line.contains("# backup-pipeline"))
        .collect::<Vec<_>>();
    let mut file = tempfile::NamedTempFile::new()?;
    writeln!(file, "{}", filtered.join("\n"))?;
    file.flush()?;
    let path = file.path().to_string_lossy();
    let installed = executor.run("crontab", &[&path])?;
    if installed.status_code != 0 {
        anyhow::bail!("crontab cleanup failed: {}", installed.stderr.trim());
    }
    Ok(())
}

fn purge_configuration_scope(profiles_path: &Path) -> Result<()> {
    let config_dir = profiles_path.parent().unwrap_or_else(|| Path::new("."));
    let config = crate::config::model::ResticProfileConfig::load_from_path(profiles_path).ok();
    let mut files = vec![profiles_path.to_path_buf()];

    // Setup-generated sidecars and the cache are deterministic names owned by this CLI.
    for name in [
        "enc",
        "database-connection-url",
        "primary-password",
        "secondary-password",
        "primary-aws-access-key-id",
        "primary-aws-secret-access-key",
        "secondary-aws-access-key-id",
        "secondary-aws-secret-access-key",
        "id_ed25519",
        "id_ed25519.pub",
        "id_ed25519_secondary",
        "id_ed25519_secondary.pub",
    ] {
        files.push(config_dir.join(name));
    }

    if let Some(config) = &config {
        for profile in config.profiles.values() {
            for password_file in [profile.password_file.as_deref()].into_iter().flatten() {
                if let Some(path) = resolve_scope_path(config_dir, password_file) {
                    files.push(path);
                }
            }
            if let Some(copy) = &profile.copy {
                if let Some(password_file) = copy.password_file.as_deref() {
                    if let Some(path) = resolve_scope_path(config_dir, password_file) {
                        files.push(path);
                    }
                }
            }
        }

        files.extend(config.environment_sidecar_paths(config_dir));

        let reports = PathBuf::from(config.application_config().reports.output_dir);
        if reports.starts_with(config_dir) && reports != *config_dir {
            remove_owned_directory(&reports)?;
        }
    }

    remove_owned_directory(&config_dir.join("cache"))?;
    for path in files {
        if path.is_file() || path.is_symlink() {
            std::fs::remove_file(path)?;
        }
    }
    // Removing an empty configuration directory is safe and keeps the existing uninstall UX,
    // while a directory containing unrelated files remains untouched.
    let _ = std::fs::remove_dir(config_dir);
    Ok(())
}

fn resolve_scope_path(config_dir: &Path, value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    };
    resolved.starts_with(config_dir).then_some(resolved)
}

fn remove_owned_directory(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
