use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use anyhow::{Result, bail};
use secrecy::ExposeSecret;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum RestoreStorage {
    Primary,
    Secondary,
}

impl Default for RestoreStorage {
    fn default() -> Self {
        Self::Primary
    }
}

pub fn select_storage(config: &BackupConfig, storage: RestoreStorage) -> Result<(&str, &str)> {
    match storage {
        RestoreStorage::Primary => Ok((
            &config.storage.primary.repository,
            config.storage.primary.password.expose_secret(),
        )),
        RestoreStorage::Secondary => {
            let secondary = config
                .storage
                .secondary
                .as_ref()
                .filter(|storage| storage.enabled)
                .ok_or_else(|| anyhow::anyhow!("Secondary storage is not configured or enabled"))?;
            Ok((&secondary.repository, secondary.password.expose_secret()))
        }
    }
}

pub fn validate_restored_output(target: &Path, database_stream: bool) -> Result<()> {
    validate_restored_output_against(target, database_stream, &[])
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RestoreFileState {
    path: std::path::PathBuf,
    length: u64,
    modified: Option<std::time::SystemTime>,
}

fn restore_file_states(target: &Path) -> Result<Vec<RestoreFileState>> {
    let mut states = Vec::new();
    let mut directories = vec![target.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                let metadata = entry.metadata()?;
                states.push(RestoreFileState {
                    path: entry.path(),
                    length: metadata.len(),
                    modified: metadata.modified().ok(),
                });
            }
        }
    }
    Ok(states)
}

fn validate_restored_output_against(
    target: &Path,
    database_stream: bool,
    baseline: &[RestoreFileState],
) -> Result<()> {
    if !target
        .symlink_metadata()
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
    {
        bail!("Restore target was not created");
    }
    let states = restore_file_states(target)?;
    let files = states
        .iter()
        .filter(|state| state.length > 0)
        .map(|state| state.path.clone())
        .collect::<Vec<_>>();
    if files.is_empty() {
        bail!("Restore completed but produced no non-empty output");
    }
    if !baseline.is_empty()
        && states.iter().all(|state| {
            baseline.iter().any(|previous| {
                previous.path == state.path
                    && previous.length == state.length
                    && previous.modified == state.modified
            })
        })
    {
        bail!("Restore completed but produced no new output");
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

fn validate_restore_target(target: &Path, force: bool) -> Result<()> {
    match std::fs::symlink_metadata(target) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                bail!("Restore target cannot be a symlink");
            }
            if !file_type.is_dir() {
                bail!("Restore target must be a directory");
            }
            if target.read_dir()?.next().is_some() && !force {
                bail!("Restore target is not empty; pass --force to overwrite");
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = target
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let metadata = std::fs::symlink_metadata(parent)
                .map_err(|_| anyhow::anyhow!("Restore target parent does not exist"))?;
            if !metadata.file_type().is_dir() {
                bail!("Restore target parent must be a directory");
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                if mode & 0o222 == 0 || mode & 0o111 == 0 {
                    bail!("Restore target parent is not writable");
                }
            }
            #[cfg(not(unix))]
            if metadata.permissions().readonly() {
                bail!("Restore target parent is not writable");
            }
        }
        Err(error) => return Err(error.into()),
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
    execute_restore_from_storage(
        config,
        runner,
        snapshot_id,
        target_path,
        force,
        RestoreStorage::Primary,
    )
}

pub fn execute_restore_from_storage<R: ResticRunner>(
    config: &BackupConfig,
    runner: &R,
    snapshot_id: &str,
    target_path: &str,
    force: bool,
    storage: RestoreStorage,
) -> Result<String> {
    tracing::info!(
        snapshot_id = %snapshot_id,
        target_path = %target_path,
        storage = ?storage,
        force = %force,
        "Executing snapshot restore command"
    );
    let target = Path::new(target_path);
    validate_restore_target(target, force)?;
    let baseline = if force {
        restore_file_states(target)?
    } else {
        Vec::new()
    };
    let (repository, password) = select_storage(config, storage)?;
    let result = runner.restore(repository, password, snapshot_id, target_path)?;
    validate_restored_output_against(
        target,
        matches!(
            config.backup.backup_type,
            crate::config::model::BackupType::DbStream { .. }
        ),
        &baseline,
    )?;
    Ok(result)
}

pub fn execute_restore_from_profiles<R: ResticRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    runner: &R,
    snapshot_id: &str,
    target_path: &str,
    force: bool,
    storage: RestoreStorage,
) -> Result<String> {
    let target = Path::new(target_path);
    validate_restore_target(target, force)?;
    let baseline = if force {
        restore_file_states(target)?
    } else {
        Vec::new()
    };
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let owned_environment = config.sidecar_environment(config_dir)?;
    let environment = owned_environment
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    let backend = match storage {
        RestoreStorage::Primary => "primary",
        RestoreStorage::Secondary => "secondary",
    };
    let (repository, password) = config.backend_credentials(config_dir, backend)?;
    let result = runner.restore_with_env(
        &repository,
        &password,
        snapshot_id,
        target_path,
        &environment,
    )?;
    let database_stream = config
        .application
        .as_ref()
        .and_then(|application| application.database.as_ref())
        .is_some();
    validate_restored_output_against(target, database_stream, &baseline)?;
    Ok(result)
}
