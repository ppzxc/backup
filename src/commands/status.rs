use anyhow::Result;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::config::model::BackupConfig;
use crate::runner::executor::{CommandRunner, SystemExecutor};

#[derive(Debug, Deserialize)]
pub struct ResticSnapshotInfo {
    pub id: String,
    pub time: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub hostname: String,
}

pub fn execute_status(config: &BackupConfig) -> Result<String> {
    let executor = SystemExecutor;
    execute_status_with_runner(config, &executor, None)
}

pub fn execute_status_from_profiles_config<R: crate::runner::resticprofile::ResticProfileRunner>(
    config_path: &std::path::Path,
    profile_filter: Option<&str>,
    runner: &R,
) -> Result<String> {
    let restic_config = match crate::config::model::ResticProfileConfig::load_from_path(config_path) {
        Ok(c) => c,
        Err(_) => {
            let default_config = crate::config::model::BackupConfig::default();
            let executor = crate::runner::executor::SystemExecutor;
            return execute_status_with_runner(&default_config, &executor, profile_filter);
        }
    };
    let target_profile_name = profile_filter.unwrap_or("default");

    let profile_section = restic_config
        .profiles
        .get(target_profile_name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found in config", target_profile_name))?;

    let repository = profile_section
        .repository
        .as_deref()
        .or_else(|| {
            profile_section.inherit.as_ref().and_then(|parents| {
                parents.iter().find_map(|p| restic_config.profiles.get(p).and_then(|sec| sec.repository.as_deref()))
            })
        })
        .or_else(|| restic_config.profiles.get("primary").and_then(|p| p.repository.as_deref()))
        .or_else(|| restic_config.profiles.get("default").and_then(|p| p.repository.as_deref()))
        .unwrap_or("unknown");

    let backend = if repository.starts_with("s3:") {
        "s3"
    } else if repository.starts_with("rclone:") {
        "rclone"
    } else if repository.starts_with("sftp:") {
        "sftp"
    } else {
        "local"
    };

    let targets = profile_section
        .backup
        .as_ref()
        .and_then(|b| b.source.as_ref())
        .or_else(|| {
            profile_section.inherit.as_ref().and_then(|parents| {
                parents.iter().find_map(|p| restic_config.profiles.get(p).and_then(|sec| sec.backup.as_ref().and_then(|b| b.source.as_ref())))
            })
        })
        .or_else(|| restic_config.profiles.get("default").and_then(|p| p.backup.as_ref().and_then(|b| b.source.as_ref())))
        .cloned()
        .unwrap_or_default();

    let mut output_str = format!(
        "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
        target_profile_name, backend, repository, targets
    );

    match runner.list_snapshots(config_path, target_profile_name) {
        Ok(raw_output) => {
            let trimmed = raw_output.trim();
            if trimmed.is_empty() {
                output_str.push_str("\nSnapshots: None");
            } else {
                output_str.push_str(&format!("\nSnapshots:\n{}", trimmed));
            }
        }
        Err(err) => {
            output_str.push_str(&format!("\n[WARN] Failed to fetch snapshots: {}", err));
        }
    }

    Ok(output_str)
}

pub fn execute_status_with_runner<E: CommandRunner>(
    config: &BackupConfig,
    runner: &E,
    profile_filter: Option<&str>,
) -> Result<String> {
    let target_profile = profile_filter.unwrap_or(&config.profile);

    let mut output_str = format!(
        "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
        target_profile,
        config.storage.primary.backend,
        config.storage.primary.repository,
        config.backup.targets
    );

    let password = config.storage.primary.password.expose_secret();
    let repo = &config.storage.primary.repository;

    match query_snapshots(runner, repo, password) {
        Ok(snapshots) => {
            if let Some(latest) = snapshots.first() {
                output_str.push_str(&format!(
                    "\nLatest Snapshot: {}\nSnapshot Time: {}\nTotal Snapshots: {}",
                    latest.id,
                    latest.time,
                    snapshots.len()
                ));
            } else {
                output_str.push_str("\nSnapshots: None");
            }
        }
        Err(err) => {
            output_str.push_str(&format!(
                "\n[WARN] Failed to fetch snapshots: {}",
                err
            ));
        }
    }

    Ok(output_str)
}

fn query_snapshots<E: CommandRunner>(
    runner: &E,
    repo: &str,
    password: &str,
) -> Result<Vec<ResticSnapshotInfo>> {
    let pass_file = create_temp_password_file(password)?;
    let pass_path = pass_file.path().to_string_lossy();

    let output = runner.run("restic", &["-r", repo, "--password-file", &pass_path, "snapshots", "--json"])?;

    if output.status_code != 0 {
        let err_msg = if !output.stderr.trim().is_empty() {
            output.stderr.trim().to_string()
        } else {
            format!("restic snapshots failed with exit code {}", output.status_code)
        };
        anyhow::bail!("{}", err_msg);
    }

    let snapshots: Vec<ResticSnapshotInfo> = serde_json::from_str(&output.stdout)?;
    Ok(snapshots)
}

fn create_temp_password_file(password: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(password.as_bytes())?;
    file.flush()?;
    Ok(file)
}
