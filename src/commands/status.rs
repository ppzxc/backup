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
    if !config_path.exists() {
        return Ok(format!(
            "Configuration file not found at {}",
            config_path.display()
        ));
    }

    let restic_config = crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    let resolved_profiles =
        crate::config::profile_resolver::ProfileResolver::resolve_all_or_filtered(
            &restic_config,
            profile_filter,
        );

    if resolved_profiles.is_empty() {
        return Ok("No active backup profiles found in configuration.".to_string());
    }

    let mut full_output = Vec::new();

    for profile in &resolved_profiles {
        let mut output_str = format!(
            "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
            profile.name, profile.backend, profile.repository, profile.targets
        );

        match runner.list_snapshots(config_path, &profile.name) {
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

        full_output.push(output_str);
    }

    Ok(full_output.join("\n\n"))
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
            output_str.push_str(&format!("\n[WARN] Failed to fetch snapshots: {}", err));
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

    let output = runner.run(
        "restic",
        &[
            "-r",
            repo,
            "--password-file",
            &pass_path,
            "snapshots",
            "--json",
        ],
    )?;

    if output.status_code != 0 {
        let err_msg = if !output.stderr.trim().is_empty() {
            output.stderr.trim().to_string()
        } else {
            format!(
                "restic snapshots failed with exit code {}",
                output.status_code
            )
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
