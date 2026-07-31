use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::Result;
use secrecy::ExposeSecret;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct PipelineOptions {
    pub skip_database: bool,
    pub skip_secondary_sync: bool,
    pub skip_retention: bool,
    pub dry_run: bool,
}

pub struct PipelineEngine<'a, R: ResticProfileRunner> {
    runner: &'a R,
}

pub type BackupRunner<'a, R> = PipelineEngine<'a, R>;

impl<'a, R: ResticProfileRunner> PipelineEngine<'a, R> {
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

pub fn execute_secondary_copy<R: ResticProfileRunner>(
    config_path: &Path,
    profile: &str,
    dry_run: bool,
    runner: &R,
) -> Result<String> {
    runner.copy(config_path, profile, dry_run)
}

pub fn execute_retention<R: ResticProfileRunner>(
    config_path: &Path,
    profile: &str,
    runner: &R,
) -> Result<String> {
    runner.prune(config_path, profile)
}

pub fn execute_run<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.backup_paths(repo, pwd, &config.backup.targets, &config.backup.excludes)
}

pub fn execute_run_profile<R: ResticProfileRunner>(
    config_path: &Path,
    profile: &str,
    opts: &PipelineOptions,
    runner: &R,
) -> Result<String> {
    let engine = PipelineEngine::new(runner);
    engine.execute(config_path, profile, opts)
}
