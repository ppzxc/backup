use crate::commands::database::{DatabaseDumpValidation, validate_dump_signature};
use crate::config::model::{BackupConfig, DatabaseType};
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
    validate_restored_output_against(
        target,
        if database_stream {
            DatabaseValidationExpectation::Any
        } else {
            DatabaseValidationExpectation::None
        },
        &[],
    )
}

pub fn validate_restored_output_for_database(
    target: &Path,
    database_type: DatabaseType,
) -> Result<()> {
    validate_restored_output_against(
        target,
        DatabaseValidationExpectation::Typed(database_type),
        &[],
    )
}

/// Measurements captured from one restore target. The Restore Drill uses these values as its
/// evidence input instead of treating a successful restic exit status as proof of recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutputMetrics {
    pub file_count: u64,
    pub total_bytes: u64,
    pub validation_method: String,
    pub validation_error: Option<String>,
    pub database_validation: Option<DatabaseDumpValidation>,
}

impl RestoreOutputMetrics {
    pub fn validation_passed(&self) -> bool {
        self.validation_error.is_none() && self.file_count > 0 && self.total_bytes > 0
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseValidationExpectation {
    None,
    Any,
    Typed(DatabaseType),
}

/// Measures regular files and validates the optional Database Stream signature without importing
/// data into an operating database. Semantic validation failures remain in the returned metrics so
/// the Restore Drill can write a complete failure Evidence artifact.
pub fn measure_restore_output(
    target: &Path,
    database_stream: bool,
) -> Result<RestoreOutputMetrics> {
    measure_restore_output_with_expectation(
        target,
        if database_stream {
            DatabaseValidationExpectation::Any
        } else {
            DatabaseValidationExpectation::None
        },
    )
}

pub fn measure_restore_output_for_database(
    target: &Path,
    database_type: DatabaseType,
) -> Result<RestoreOutputMetrics> {
    measure_restore_output_with_expectation(
        target,
        DatabaseValidationExpectation::Typed(database_type),
    )
}

fn measure_restore_output_with_expectation(
    target: &Path,
    database_expectation: DatabaseValidationExpectation,
) -> Result<RestoreOutputMetrics> {
    if !target
        .symlink_metadata()
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
    {
        bail!("Restore target was not created");
    }
    let states = restore_file_states(target)?;
    let file_count = states.len() as u64;
    let total_bytes = states.iter().map(|state| state.length).sum();
    let validation_method = match database_expectation {
        DatabaseValidationExpectation::None => "regular file count and total bytes".to_string(),
        DatabaseValidationExpectation::Any => {
            "regular file count, total bytes, and supported SQL dump signature".to_string()
        }
        DatabaseValidationExpectation::Typed(database_type) => format!(
            "regular file count, total bytes, and {}",
            DatabaseDumpValidation::expected_signature(database_type)
        ),
    };
    let mut database_validation = match database_expectation {
        DatabaseValidationExpectation::Typed(database_type) => Some(DatabaseDumpValidation {
            database_type,
            expected_signature: DatabaseDumpValidation::expected_signature(database_type).into(),
            signature_verified: false,
        }),
        DatabaseValidationExpectation::None | DatabaseValidationExpectation::Any => None,
    };
    let mut validation_error = if file_count == 0 {
        Some("Restore Output Validation produced no regular files".into())
    } else if total_bytes == 0 {
        Some("Restore Output Validation produced zero total bytes".into())
    } else {
        None
    };

    if !matches!(database_expectation, DatabaseValidationExpectation::None)
        && validation_error.is_none()
    {
        let dumps = states
            .iter()
            .filter(|state| {
                state
                    .path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sql"))
            })
            .collect::<Vec<_>>();
        if dumps.is_empty() {
            validation_error = Some("Database Stream restore did not produce a SQL dump".into());
        } else if dumps.len() > 1 {
            validation_error =
                Some("Database Stream restore produced multiple SQL dump files".into());
        } else if dumps[0].length == 0 {
            validation_error = Some("Database Stream restore produced an empty SQL dump".into());
        } else {
            let dump_content = std::fs::read_to_string(&dumps[0].path).ok();
            let signature_matches =
                dump_content
                    .as_deref()
                    .is_some_and(|content| match database_expectation {
                        DatabaseValidationExpectation::Any => {
                            validate_dump_signature(content, DatabaseType::Mysql).signature_verified
                                || validate_dump_signature(content, DatabaseType::Postgres)
                                    .signature_verified
                        }
                        DatabaseValidationExpectation::Typed(expected) => {
                            validate_dump_signature(content, expected).signature_verified
                        }
                        DatabaseValidationExpectation::None => false,
                    });
            let any_signature_matches = dump_content.as_deref().is_some_and(|content| {
                validate_dump_signature(content, DatabaseType::Mysql).signature_verified
                    || validate_dump_signature(content, DatabaseType::Postgres).signature_verified
            });
            if let Some(validation) = database_validation.as_mut() {
                validation.signature_verified = signature_matches;
            }
            if !signature_matches {
                validation_error = Some(match database_expectation {
                    DatabaseValidationExpectation::Any => {
                        "Database Stream restore produced an invalid SQL dump signature".into()
                    }
                    DatabaseValidationExpectation::Typed(_) => {
                        if any_signature_matches {
                            "Database Stream dump signature does not match the configured database type".into()
                        } else {
                            "Database Stream restore produced an invalid SQL dump signature".into()
                        }
                    }
                    DatabaseValidationExpectation::None => unreachable!(),
                });
            }
        }
    }

    Ok(RestoreOutputMetrics {
        file_count,
        total_bytes,
        validation_method,
        validation_error,
        database_validation,
    })
}

fn validate_restored_output_against(
    target: &Path,
    database_expectation: DatabaseValidationExpectation,
    baseline: &[RestoreFileState],
) -> Result<()> {
    if !target
        .symlink_metadata()
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
    {
        bail!("Restore target was not created");
    }
    let metrics = measure_restore_output_with_expectation(target, database_expectation)?;
    if metrics.file_count == 0 || metrics.total_bytes == 0 {
        bail!("Restore completed but produced no non-empty output");
    }
    let states = restore_file_states(target)?;
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
    if let Some(error) = metrics.validation_error {
        bail!("{error}");
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
    // This compatibility API has no exact Backup Profile selector. Typed Database Stream
    // validation is intentionally reserved for the profile-scoped Restore Drill path.
    validate_restored_output_against(target, DatabaseValidationExpectation::None, &baseline)?;
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
    // The generic restore command has no exact Backup Profile selector. It must not infer a
    // Database Stream expectation from application metadata and apply it to every snapshot;
    // typed Database Stream validation belongs to the profile-scoped Restore Drill.
    validate_restored_output_against(target, DatabaseValidationExpectation::None, &baseline)?;
    Ok(result)
}
