use anyhow::Result;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::fmt;
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

#[derive(Debug)]
pub struct StatusCommandFailure {
    pub message: String,
    pub output: String,
}

impl fmt::Display for StatusCommandFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StatusCommandFailure {}

pub fn execute_status(config: &BackupConfig) -> Result<String> {
    let executor = SystemExecutor;
    execute_status_with_runner(config, &executor, None)
}

pub fn execute_status_from_profiles_config<
    R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
>(
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
    let (resolved_profiles, mut warnings) = match profile_filter {
        Some(profile) => (
            crate::config::profile_resolver::ProfileResolver::resolve_for_status(
                &restic_config,
                Some(profile),
            )?,
            Vec::new(),
        ),
        None => crate::config::profile_resolver::ProfileResolver::resolve_all_active_with_failures(
            &restic_config,
        ),
    };

    if resolved_profiles.is_empty() {
        let output = "[WARN] No active backup profiles found in configuration.".to_string();
        if warnings.is_empty() {
            return Ok(output);
        }
        return Err(StatusCommandFailure {
            message: warnings.join("; "),
            output,
        }
        .into());
    }

    let mut full_output = Vec::new();

    for profile in &resolved_profiles {
        let mut output_str = format!(
            "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
            profile.name,
            profile.backend,
            redact_status_text(&profile.repository),
            profile.targets
        );

        match runner.list_snapshots(config_path, &profile.name) {
            Ok(raw_output) => {
                let trimmed = raw_output.trim();
                if trimmed.is_empty() {
                    output_str.push_str("\nSnapshots: None");
                } else {
                    output_str.push_str(&format!("\nSnapshots:\n{}", redact_status_text(trimmed)));
                }
            }
            Err(err) => {
                let diagnostic = redact_status_text(&err.to_string());
                tracing::warn!(profile = %profile.name, error = %diagnostic, "Failed to fetch snapshots for profile");
                warnings.push(format!(
                    "{}: failed to fetch snapshots: {diagnostic}",
                    profile.name
                ));
                output_str.push_str(&format!("\n[WARN] Failed to fetch snapshots: {diagnostic}"));
            }
        }

        full_output.push(output_str);
    }

    let output = full_output.join("\n\n");
    if warnings.is_empty() {
        Ok(output)
    } else {
        Err(StatusCommandFailure {
            message: warnings.join("; "),
            output,
        }
        .into())
    }
}

fn redact_status_text(value: &str) -> String {
    crate::commands::redact_diagnostic(value, &[])
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
        redact_status_text(&config.storage.primary.repository),
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
            let diagnostic = redact_status_text(&err.to_string());
            tracing::warn!(error = %diagnostic, "Failed to fetch snapshots");
            output_str.push_str(&format!("\n[WARN] Failed to fetch snapshots: {diagnostic}"));
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

#[cfg(test)]
mod tests {
    use super::redact_status_text;

    #[test]
    fn status_text_masks_url_credentials_and_secret_words() {
        let redacted =
            redact_status_text("s3://user:password@example/backup status-password token=abc");
        assert!(!redacted.contains("password"));
        assert!(!redacted.contains("user:"));
        assert!(redacted.contains("<redacted>"));
    }
}
