use crate::config::model::{BackupConfig, borrowed_environment};
use crate::runner::restic::ResticRunner;
use anyhow::Result;
use secrecy::ExposeSecret;
use std::path::Path;

pub use crate::runner::snapshot::{
    SnapshotInfo, SnapshotJsonError, SnapshotSelection, SnapshotSelectionReason,
    SnapshotSelectionStatus, parse_snapshot_json, select_latest_tagged_snapshot,
    select_latest_tagged_snapshot_from_infos, select_latest_tagged_snapshot_from_json,
    select_latest_tagged_snapshot_with_env,
};

pub fn execute_snapshots_from_profiles<R: ResticRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let owned_environment = config.sidecar_environment(config_dir)?;
    let environment = borrowed_environment(&owned_environment);
    let (primary_repository, primary_password) = config
        .backend_credentials(config_dir, "primary")
        .map_err(|error| {
            anyhow::anyhow!(
                "primary snapshots unavailable: {}",
                redact_snapshot_diagnostic(&error.to_string(), "", "")
            )
        })?;
    let primary_sftp_args = config
        .profiles
        .get("primary")
        .and_then(|profile| profile.option.as_ref())
        .and_then(|options| options.get("sftp.args"))
        .map(String::as_str);
    let primary = runner
        .list_snapshots_with_env_and_sftp_args(
            &primary_repository,
            &primary_password,
            &environment,
            primary_sftp_args,
        )
        .map_err(|error| {
            anyhow::anyhow!(
                "primary snapshots unavailable: {}",
                redact_snapshot_diagnostic(
                    &error.to_string(),
                    &primary_repository,
                    &primary_password
                )
            )
        })?;
    let mut output = format!(
        "Primary snapshots:\n{}",
        redact_snapshot_diagnostic(&primary, &primary_repository, &primary_password)
    );
    if config.profiles.contains_key("secondary") {
        match config.backend_credentials(config_dir, "secondary") {
            Ok((secondary_repository, secondary_password)) => {
                let secondary_sftp_args = config
                    .profiles
                    .get("secondary")
                    .and_then(|profile| profile.option.as_ref())
                    .and_then(|options| options.get("sftp.args"))
                    .map(String::as_str);
                match runner.list_snapshots_with_env_and_sftp_args(
                    &secondary_repository,
                    &secondary_password,
                    &environment,
                    secondary_sftp_args,
                ) {
                    Ok(snapshots) => output.push_str(&format!(
                        "\nSecondary snapshots:\n{}",
                        redact_snapshot_diagnostic(
                            &snapshots,
                            &secondary_repository,
                            &secondary_password,
                        )
                    )),
                    Err(error) => output.push_str(&format!(
                        "\n[WARN] Secondary snapshots unavailable: {}",
                        redact_snapshot_diagnostic(
                            &error.to_string(),
                            &secondary_repository,
                            &secondary_password,
                        )
                    )),
                }
            }
            Err(error) => output.push_str(&format!(
                "\n[WARN] Secondary snapshots unavailable: {}",
                redact_snapshot_diagnostic(&error.to_string(), "", "")
            )),
        }
    }
    Ok(output)
}

fn redact_snapshot_diagnostic(value: &str, repository: &str, password: &str) -> String {
    crate::commands::redact_diagnostic(value, &[password, repository])
}

pub fn execute_snapshots<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    let primary = runner.list_snapshots(repo, pwd).map_err(|error| {
        anyhow::anyhow!(
            "primary snapshots unavailable: {}",
            redact_snapshot_diagnostic(&error.to_string(), repo, pwd)
        )
    })?;
    let mut output = format!(
        "Primary snapshots:\n{}",
        redact_snapshot_diagnostic(&primary, repo, pwd)
    );
    if let Some(secondary) = config.storage.secondary.as_ref().filter(|s| s.enabled) {
        match runner.list_snapshots(&secondary.repository, secondary.password.expose_secret()) {
            Ok(snapshots) => output.push_str(&format!(
                "\nSecondary snapshots:\n{}",
                redact_snapshot_diagnostic(
                    &snapshots,
                    &secondary.repository,
                    secondary.password.expose_secret(),
                )
            )),
            Err(error) => output.push_str(&format!(
                "\n[WARN] Secondary snapshots unavailable: {}",
                redact_snapshot_diagnostic(
                    &error.to_string(),
                    &secondary.repository,
                    secondary.password.expose_secret(),
                )
            )),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::redact_snapshot_diagnostic;

    #[test]
    fn snapshot_diagnostics_mask_credentials_and_repository() {
        let redacted = redact_snapshot_diagnostic(
            "repository=s3:https://user:password@example/backup token=abc",
            "s3:https://user:password@example/backup",
            "password",
        );
        assert!(!redacted.contains("password"));
        assert!(!redacted.contains("s3:https://user:password@example/backup"));
        assert!(redacted.contains("<redacted>"));
    }
}
