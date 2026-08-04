use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use anyhow::Result;
use secrecy::ExposeSecret;
use std::path::Path;

pub fn execute_snapshots_from_profiles<R: ResticRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let owned_environment = config.sidecar_environment(config_dir)?;
    let environment = owned_environment
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    let (primary_repository, primary_password) =
        config.backend_credentials(config_dir, "primary")?;
    let primary =
        runner.list_snapshots_with_env(&primary_repository, &primary_password, &environment)?;
    let mut output = format!("Primary snapshots:\n{primary}");
    if config.profiles.contains_key("secondary") {
        let (secondary_repository, secondary_password) =
            config.backend_credentials(config_dir, "secondary")?;
        match runner.list_snapshots_with_env(
            &secondary_repository,
            &secondary_password,
            &environment,
        ) {
            Ok(snapshots) => output.push_str(&format!("\nSecondary snapshots:\n{snapshots}")),
            Err(error) => output.push_str(&format!(
                "\n[WARN] Secondary snapshots unavailable: {error}"
            )),
        }
    }
    Ok(output)
}

pub fn execute_snapshots<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    let primary = runner.list_snapshots(repo, pwd)?;
    let mut output = format!("Primary snapshots:\n{primary}");
    if let Some(secondary) = config.storage.secondary.as_ref().filter(|s| s.enabled) {
        match runner.list_snapshots(&secondary.repository, secondary.password.expose_secret()) {
            Ok(snapshots) => output.push_str(&format!("\nSecondary snapshots:\n{snapshots}")),
            Err(error) => output.push_str(&format!(
                "\n[WARN] Secondary snapshots unavailable: {error}"
            )),
        }
    }
    Ok(output)
}
