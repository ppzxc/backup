use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use serde::Serialize;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, info_span};

#[derive(Debug, Clone, Default)]
pub struct PipelineOptions {
    pub skip_database: bool,
    pub skip_secondary_sync: bool,
    pub skip_retention: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct RunCommandFailure {
    pub message: String,
    pub artifacts: Vec<PathBuf>,
    pub external_state_changes: Vec<String>,
}

impl fmt::Display for RunCommandFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RunCommandFailure {}

#[derive(Debug, Serialize)]
pub struct ExecutionReport {
    pub timestamp_unix_nanos: u128,
    pub profile: String,
    pub succeeded: bool,
    pub snapshot_id: Option<String>,
    pub primary_result: Option<String>,
    pub secondary_result: Option<String>,
    pub retention_result: Option<String>,
    pub failure_stage: Option<String>,
    pub error: Option<String>,
}

impl ExecutionReport {
    pub fn success(
        profile: &str,
        primary_result: String,
        secondary_result: Option<String>,
        retention_result: Option<String>,
    ) -> Self {
        Self::success_with_dry_run(
            profile,
            primary_result,
            secondary_result,
            retention_result,
            false,
        )
    }

    pub fn success_with_dry_run(
        profile: &str,
        primary_result: String,
        secondary_result: Option<String>,
        retention_result: Option<String>,
        dry_run: bool,
    ) -> Self {
        Self {
            timestamp_unix_nanos: now_nanos(),
            profile: profile.into(),
            succeeded: true,
            snapshot_id: (!dry_run)
                .then(|| snapshot_id_from(&primary_result))
                .flatten(),
            primary_result: Some(primary_result),
            secondary_result,
            retention_result,
            failure_stage: None,
            error: None,
        }
    }

    pub fn failure(profile: &str, stage: &str, error: impl ToString) -> Self {
        Self {
            timestamp_unix_nanos: now_nanos(),
            profile: profile.into(),
            succeeded: false,
            snapshot_id: None,
            primary_result: None,
            secondary_result: None,
            retention_result: None,
            failure_stage: Some(stage.into()),
            error: Some(error.to_string()),
        }
    }
}

fn snapshot_id_from(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.split_once("snapshot ")
            .and_then(|(_, rest)| rest.split_once(" saved"))
            .map(|(id, _)| id.trim().to_owned())
    })
}

pub fn write_execution_report(
    config: &BackupConfig,
    mut report: ExecutionReport,
) -> Result<std::path::PathBuf> {
    crate::config::model::create_secure_dir(Path::new(&config.reports.output_dir))?;
    redact_execution_report(config, &mut report);
    let path = Path::new(&config.reports.output_dir)
        .join(format!("execution-{}.json", report.timestamp_unix_nanos));
    crate::config::model::save_secure_file(
        &path,
        &String::from_utf8(serde_json::to_vec_pretty(&report)?)?,
    )?;
    Ok(path)
}

pub fn write_execution_report_from_profiles(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    mut report: ExecutionReport,
) -> Result<std::path::PathBuf> {
    let application = config.application_config();
    let output_dir = Path::new(&application.reports.output_dir);
    crate::config::model::create_secure_dir(output_dir)?;
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let mut secrets = config
        .sidecar_environment(config_dir)?
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    secrets.extend(
        config
            .profiles
            .values()
            .filter_map(|profile| profile.password.clone()),
    );
    for (profile_name, profile) in &config.profiles {
        if profile.repository.is_some()
            || profile.password_file.is_some()
            || profile.inherit.is_some()
        {
            let (_, password) = config
                .backend_credentials(config_dir, profile_name)
                .with_context(|| {
                    format!(
                        "cannot resolve credentials for execution report profile '{profile_name}'"
                    )
                })?;
            secrets.push(password);
        }
    }
    if config
        .application
        .as_ref()
        .and_then(|application| application.database.as_ref())
        .is_some()
    {
        let database_url = crate::config::model::ResticProfileConfig::secure_sidecar_value(
            config_dir,
            "database-connection-url",
        )?;
        secrets.push(database_url);
    }
    for secret in secrets
        .into_iter()
        .filter(|secret| !secret.trim().is_empty())
    {
        for field in [
            &mut report.primary_result,
            &mut report.secondary_result,
            &mut report.retention_result,
            &mut report.error,
        ] {
            if let Some(value) = field {
                *value = value.replace(&secret, "******");
            }
        }
    }
    let path = output_dir.join(format!("execution-{}.json", report.timestamp_unix_nanos));
    crate::config::model::save_secure_file(
        &path,
        &String::from_utf8(serde_json::to_vec_pretty(&report)?)?,
    )?;
    Ok(path)
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn redact_execution_report(config: &BackupConfig, report: &mut ExecutionReport) {
    let mut secrets = vec![config.storage.primary.password.expose_secret()];
    if let Some(secondary) = &config.storage.secondary {
        secrets.push(secondary.password.expose_secret());
        if let Some(s3) = &secondary.s3 {
            secrets.push(s3.access_key_id.expose_secret());
            secrets.push(s3.secret_access_key.expose_secret());
        }
    }
    if let Some(s3) = &config.storage.primary.s3 {
        secrets.push(s3.access_key_id.expose_secret());
        secrets.push(s3.secret_access_key.expose_secret());
    }
    if let crate::config::model::BackupType::DbStream {
        connection_url: Some(url),
        ..
    } = &config.backup.backup_type
    {
        secrets.push(url);
    }
    for secret in secrets
        .into_iter()
        .filter(|secret| !secret.trim().is_empty())
    {
        for field in [
            &mut report.primary_result,
            &mut report.secondary_result,
            &mut report.retention_result,
            &mut report.error,
        ] {
            if let Some(value) = field {
                *value = value.replace(secret, "******");
            }
        }
    }
}

pub struct PipelineEngine<'a, R: ResticProfileRunner + ?Sized> {
    runner: &'a R,
}

pub type BackupRunner<'a, R> = PipelineEngine<'a, R>;

impl<'a, R: ResticProfileRunner + ?Sized> PipelineEngine<'a, R> {
    pub fn new(runner: &'a R) -> Self {
        Self { runner }
    }

    pub fn execute(
        &self,
        config_path: &Path,
        profile: &str,
        opts: &PipelineOptions,
    ) -> Result<String> {
        let mut output = String::new();
        match self.runner.backup(config_path, profile, opts.dry_run) {
            Ok(profile_res) => output.push_str(&profile_res),
            Err(err) if opts.dry_run => {
                output.push_str(&format!(
                    "[Pipeline] [Dry-Run] resticprofile backup simulated ({})\n",
                    err
                ));
            }
            Err(err) => return Err(err),
        }

        Ok(output)
    }
}

pub fn resolve_profiles(
    config: &crate::config::model::ResticProfileConfig,
    profile: Option<&str>,
) -> Result<Vec<String>> {
    let _span = info_span!("profile resolution").entered();
    if let Some(p) = profile {
        if p.trim().is_empty() || p != p.trim() {
            anyhow::bail!("profile must be an exact, non-empty configured profile name");
        }
        if !config.profile_names().iter().any(|name| name == p) {
            anyhow::bail!("profile '{p}' is not configured");
        }
        info!(profile = %p, "Resolved target profile");
        Ok(vec![p.to_string()])
    } else {
        let names = config.profile_names();
        if names.is_empty() {
            anyhow::bail!("No Backup Profiles are configured for backup run");
        }
        info!(profiles = ?names, "Resolved configuration profiles");
        Ok(names)
    }
}

pub fn run_database_stage<R: ResticRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    runner: &R,
    dry_run: bool,
) -> Result<String> {
    let _span = info_span!("database").entered();
    info!("Executing database backup stage");
    crate::commands::database::execute_database_backup_from_profiles(
        config,
        config_path,
        runner,
        dry_run,
    )
}

pub fn execute_secondary_copy<R: ResticProfileRunner + ?Sized>(
    config_path: &Path,
    profile: &str,
    dry_run: bool,
    runner: &R,
) -> Result<String> {
    let _span = info_span!("secondary sync", profile = %profile).entered();
    info!(profile = %profile, "Executing secondary sync stage");
    runner.copy(config_path, profile, dry_run)
}

/// Executes the copy operation declared by every selected Backup Profile.
/// Profiles without `copy` are intentionally skipped: replication is profile-owned.
pub fn execute_secondary_copies<R: ResticProfileRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    profiles: &[String],
    dry_run: bool,
    runner: &R,
) -> Result<Vec<String>> {
    profiles
        .iter()
        .filter(|name| {
            config
                .profiles
                .get(*name)
                .and_then(|profile| profile.copy.as_ref())
                .is_some()
        })
        .map(|name| execute_secondary_copy(config_path, name, dry_run, runner))
        .collect()
}

pub fn execute_retention<R: ResticProfileRunner + ?Sized>(
    config_path: &Path,
    profile: &str,
    runner: &R,
) -> Result<String> {
    let _span = info_span!("retention", profile = %profile).entered();
    info!(profile = %profile, "Executing retention prune stage");
    runner.prune(config_path, profile)
}

pub fn execute_run<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.backup_paths(repo, pwd, &config.backup.targets, &config.backup.excludes)
}

pub fn execute_run_profile<R: ResticProfileRunner + ?Sized>(
    config_path: &Path,
    profile: &str,
    opts: &PipelineOptions,
    runner: &R,
) -> Result<String> {
    let _span = info_span!("primary backup", profile = %profile).entered();
    info!(profile = %profile, "Executing primary backup stage");
    let engine = PipelineEngine::new(runner);
    engine.execute(config_path, profile, opts)
}
