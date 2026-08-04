use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::Result;
use std::path::Path;

pub fn execute_copy<R: ResticProfileRunner + ?Sized>(
    runner: &R,
    config_path: &Path,
    profile: &str,
    dry_run: bool,
) -> Result<String> {
    tracing::info!(profile = %profile, dry_run = %dry_run, "Executing snapshot copy command");
    let out = runner.copy(config_path, profile, dry_run)?;
    Ok(format!(
        "Snapshot copy completed for profile [{}]:\n{}",
        profile, out
    ))
}
