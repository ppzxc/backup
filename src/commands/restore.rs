use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use anyhow::{Result, bail};
use secrecy::ExposeSecret;
use std::path::Path;

pub fn validate_restored_output(target: &Path, database_stream: bool) -> Result<()> {
    if !target.is_dir() {
        bail!("Restore target was not created");
    }
    let mut files = Vec::new();
    let mut directories = vec![target.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() && entry.metadata()?.len() > 0 {
                files.push(entry.path());
            }
        }
    }
    if files.is_empty() {
        bail!("Restore completed but produced no non-empty output");
    }
    if database_stream {
        let dump = files
            .iter()
            .find(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sql"))
            })
            .ok_or_else(|| anyhow::anyhow!("Database Stream restore did not produce a SQL dump"))?;
        let content = std::fs::read_to_string(dump)?;
        let looks_like_sql = content.starts_with("--")
            || content.contains("CREATE ")
            || content.contains("SET ")
            || content.contains("INSERT ");
        if !looks_like_sql {
            bail!("Database Stream restore produced an invalid SQL dump format");
        }
    }
    Ok(())
}

pub fn execute_restore<R: ResticRunner>(
    config: &BackupConfig,
    runner: &R,
    snapshot_id: &str,
    target_path: &str,
    force: bool,
) -> Result<String> {
    let target = Path::new(target_path);
    if target.exists() && target.read_dir()?.next().is_some() && !force {
        bail!("Restore target is not empty; pass --force to overwrite");
    }
    let result = runner.restore(
        &config.storage.primary.repository,
        config.storage.primary.password.expose_secret(),
        snapshot_id,
        target_path,
    )?;
    validate_restored_output(
        target,
        matches!(
            config.backup.backup_type,
            crate::config::model::BackupType::DbStream { .. }
        ),
    )?;
    Ok(result)
}
