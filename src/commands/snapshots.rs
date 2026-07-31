use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use anyhow::Result;
use secrecy::ExposeSecret;

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
