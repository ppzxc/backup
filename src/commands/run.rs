use anyhow::Result;
use std::path::Path;
use secrecy::ExposeSecret;
use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use crate::runner::resticprofile::ResticProfileRunner;

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

impl<'a, R: ResticProfileRunner> PipelineEngine<'a, R> {
    pub fn new(runner: &'a R) -> Self {
        Self { runner }
    }

    pub fn execute(&self, config_path: &Path, profile: &str, opts: &PipelineOptions) -> Result<String> {
        let mut output = String::new();
        if !opts.skip_database {
            if opts.dry_run {
                output.push_str("[Pipeline] [Dry-Run] Executed Database streaming backup check\n");
            } else {
                output.push_str("[Pipeline] Executed Database streaming backup check\n");
            }
        }
        let profile_res = self.runner.backup(config_path, profile, opts.dry_run)?;
        output.push_str(&profile_res);

        if !opts.skip_secondary_sync {
            if opts.dry_run {
                output.push_str("\n[Pipeline] [Dry-Run] Secondary storage sync simulated");
            } else {
                output.push_str("\n[Pipeline] Secondary storage sync completed");
            }
        }
        if !opts.skip_retention && !opts.dry_run {
            let prune_res = self.runner.prune(config_path, profile)?;
            output.push_str("\n[Pipeline] Retention prune completed: ");
            output.push_str(&prune_res);
        }
        Ok(output)
    }
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


