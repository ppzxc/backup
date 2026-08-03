use crate::runner::executor::{CommandOutput, CommandRunner};
use anyhow::Result;
use secrecy::ExposeSecret;
use std::path::Path;

pub trait ResticProfileRunner {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String>;
    fn init(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn schedule_enable(&self, config_path: &Path) -> Result<String>;
    fn schedule_disable(&self, config_path: &Path) -> Result<String>;
    fn schedule_status(&self, config_path: &Path) -> Result<String>;
    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn prune(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn check(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn copy(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String>;
}

pub struct ResticProfileTool<'a, E: CommandRunner> {
    executor: &'a E,
}

impl<'a, E: CommandRunner> ResticProfileTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }

    fn check_output(&self, output: CommandOutput) -> Result<String> {
        if output.status_code != 0 {
            let err_msg = if !output.stderr.trim().is_empty() {
                output.stderr.trim().to_string()
            } else if !output.stdout.trim().is_empty() {
                output.stdout.trim().to_string()
            } else {
                format!("command exited with status code {}", output.status_code)
            };
            anyhow::bail!(
                "resticprofile failed with exit code {}: {}",
                output.status_code,
                err_msg
            );
        }
        Ok(output.stdout)
    }

    fn run_profile_command(
        &self,
        config_path: &Path,
        profile: &str,
        args: &[&str],
    ) -> Result<CommandOutput> {
        let config = crate::config::model::BackupConfig::load_from_path(config_path).ok();
        let storage = config.as_ref().and_then(|config| {
            if profile == "secondary" {
                config
                    .storage
                    .secondary
                    .as_ref()
                    .filter(|storage| storage.enabled)
                    .map(|storage| (storage.s3.as_ref(), storage.password.expose_secret()))
            } else {
                Some((
                    config.storage.primary.s3.as_ref(),
                    config.storage.primary.password.expose_secret(),
                ))
            }
        });
        let mut env: Vec<(&str, &str)> = Vec::new();
        if let Some((Some(s3), _)) = storage {
            env.push(("AWS_ACCESS_KEY_ID", s3.access_key_id.expose_secret()));
            env.push((
                "AWS_SECRET_ACCESS_KEY",
                s3.secret_access_key.expose_secret(),
            ));
            env.push((
                "BACKUP_PRIMARY_AWS_ACCESS_KEY_ID",
                s3.access_key_id.expose_secret(),
            ));
            env.push((
                "BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY",
                s3.secret_access_key.expose_secret(),
            ));
            if profile == "secondary" {
                env.push((
                    "BACKUP_SECONDARY_AWS_ACCESS_KEY_ID",
                    s3.access_key_id.expose_secret(),
                ));
                env.push((
                    "BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY",
                    s3.secret_access_key.expose_secret(),
                ));
            }
        }
        self.executor.run_with_env("resticprofile", args, &env)
    }

    fn run_profile_command_with_timeout(
        &self,
        config_path: &Path,
        profile: &str,
        args: &[&str],
        timeout: std::time::Duration,
    ) -> Result<CommandOutput> {
        let config = crate::config::model::BackupConfig::load_from_path(config_path).ok();
        let storage = config.as_ref().and_then(|config| {
            if profile == "secondary" {
                config
                    .storage
                    .secondary
                    .as_ref()
                    .filter(|storage| storage.enabled)
                    .map(|storage| (storage.s3.as_ref(), storage.password.expose_secret()))
            } else {
                Some((
                    config.storage.primary.s3.as_ref(),
                    config.storage.primary.password.expose_secret(),
                ))
            }
        });
        let mut env: Vec<(&str, &str)> = Vec::new();
        if let Some((Some(s3), _)) = storage {
            env.push(("AWS_ACCESS_KEY_ID", s3.access_key_id.expose_secret()));
            env.push((
                "AWS_SECRET_ACCESS_KEY",
                s3.secret_access_key.expose_secret(),
            ));
            env.push((
                "BACKUP_PRIMARY_AWS_ACCESS_KEY_ID",
                s3.access_key_id.expose_secret(),
            ));
            env.push((
                "BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY",
                s3.secret_access_key.expose_secret(),
            ));
            if profile == "secondary" {
                env.push((
                    "BACKUP_SECONDARY_AWS_ACCESS_KEY_ID",
                    s3.access_key_id.expose_secret(),
                ));
                env.push((
                    "BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY",
                    s3.secret_access_key.expose_secret(),
                ));
            }
        }
        self.executor
            .run_with_timeout("resticprofile", args, &env, timeout)
    }
}

impl<'a, E: CommandRunner> ResticProfileRunner for ResticProfileTool<'a, E> {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let mut args = vec!["--config", &config_str, "--name", profile];
        if dry_run {
            args.push("--dry-run");
        }
        args.push("backup");
        let output = self.run_profile_command(config_path, profile, &args)?;
        self.check_output(output)
    }

    fn init(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.run_profile_command_with_timeout(
            config_path,
            profile,
            &["--config", &config_str, "--name", profile, "init"],
            std::time::Duration::from_secs(15),
        )?;
        self.check_output(output)
    }

    fn schedule_enable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run(
            "resticprofile",
            &["--config", &config_str, "schedule", "--all"],
        )?;
        self.check_output(output)
    }

    fn schedule_disable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run(
            "resticprofile",
            &["--config", &config_str, "unschedule", "--all"],
        )?;
        self.check_output(output)
    }

    fn schedule_status(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run(
            "resticprofile",
            &["--config", &config_str, "status", "--all"],
        )?;
        self.check_output(output)
    }

    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.run_profile_command(
            config_path,
            profile,
            &["--config", &config_str, "--name", profile, "snapshots"],
        )?;
        self.check_output(output)
    }

    fn prune(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.run_profile_command(
            config_path,
            profile,
            &["--config", &config_str, "--name", profile, "prune"],
        )?;
        self.check_output(output)
    }

    fn check(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.run_profile_command(
            config_path,
            profile,
            &["--config", &config_str, "--name", profile, "check"],
        )?;
        self.check_output(output)
    }

    fn copy(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let mut args = vec!["--config", &config_str, "--name", profile];
        if dry_run {
            args.push("--dry-run");
        }
        args.push("copy");
        let output = self.run_profile_command(config_path, profile, &args)?;
        self.check_output(output)
    }
}
