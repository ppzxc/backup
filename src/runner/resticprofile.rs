use crate::config::model::{SecretEnvironment, borrowed_environment};
use crate::runner::executor::{CommandOutput, CommandRunner};
use anyhow::Result;
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
        let mut owned_env = profile_sidecar_environment(config_path, profile)?;
        if args.last() == Some(&"copy") {
            append_copy_s3_environment(config_path, profile, &mut owned_env)?;
        }
        let env = borrowed_environment(&owned_env);
        self.executor.run_with_env("resticprofile", args, &env)
    }

    fn run_profile_command_with_timeout(
        &self,
        config_path: &Path,
        profile: &str,
        args: &[&str],
        timeout: std::time::Duration,
    ) -> Result<CommandOutput> {
        let owned_env = profile_sidecar_environment(config_path, profile)?;
        let env = borrowed_environment(&owned_env);
        self.executor
            .run_with_timeout("resticprofile", args, &env, timeout)
    }
}

/// Builds the secret environment for one resticprofile invocation. Credentials
/// remain owned here until they are borrowed by `CommandRunner` at launch time.
fn profile_sidecar_environment(config_path: &Path, _profile: &str) -> Result<SecretEnvironment> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let config = crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    config.sidecar_environment(config_dir)
}

fn append_copy_s3_environment(
    config_path: &Path,
    profile: &str,
    environment: &mut SecretEnvironment,
) -> Result<()> {
    let config = crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    environment.extend(config.copy_sidecar_environment(config_dir, profile)?);
    Ok(())
}

impl<'a, E: CommandRunner> ResticProfileRunner for ResticProfileTool<'a, E> {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        crate::config::model::ResticProfileConfig::validate_reserved_backup_profile_tag_at_path(
            config_path,
            profile,
        )?;
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
        match self.check_output(output) {
            Ok(output) => Ok(output),
            Err(error) if existing_repository_error(&error) => {
                Ok("repository already initialized".into())
            }
            Err(error) => Err(error),
        }
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

fn existing_repository_error(error: &anyhow::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("config file already exists")
        || message.contains("repository master key and config already initialized")
        || message.contains("repository already initialized")
}
