use anyhow::Result;
use std::path::Path;
use crate::runner::executor::CommandRunner;

pub trait ResticProfileRunner {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String>;
    fn schedule_enable(&self, config_path: &Path) -> Result<String>;
    fn schedule_disable(&self, config_path: &Path) -> Result<String>;
    fn schedule_status(&self, config_path: &Path) -> Result<String>;
    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn prune(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn check(&self, config_path: &Path, profile: &str) -> Result<String>;
}

pub struct ResticProfileTool<'a, E: CommandRunner> {
    executor: &'a E,
}

impl<'a, E: CommandRunner> ResticProfileTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }
}

impl<'a, E: CommandRunner> ResticProfileRunner for ResticProfileTool<'a, E> {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let mut args = vec!["--config", &config_str, "--name", profile, "backup"];
        if dry_run {
            args.push("--dry-run");
        }
        let output = self.executor.run("resticprofile", &args)?;
        Ok(output.stdout)
    }

    fn schedule_enable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "schedule", "--all"])?;
        Ok(output.stdout)
    }

    fn schedule_disable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "unschedule", "--all"])?;
        Ok(output.stdout)
    }

    fn schedule_status(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "status"])?;
        Ok(output.stdout)
    }

    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "snapshots"])?;
        Ok(output.stdout)
    }

    fn prune(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "prune"])?;
        Ok(output.stdout)
    }

    fn check(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "check"])?;
        Ok(output.stdout)
    }
}

pub struct MockResticProfileRunner {
    pub exit_code: i32,
    pub response: String,
}

impl MockResticProfileRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.to_string(),
        }
    }
}

impl ResticProfileRunner for MockResticProfileRunner {
    fn backup(&self, _config_path: &Path, _profile: &str, _dry_run: bool) -> Result<String> {
        Ok(self.response.clone())
    }
    fn schedule_enable(&self, _config_path: &Path) -> Result<String> {
        Ok(self.response.clone())
    }
    fn schedule_disable(&self, _config_path: &Path) -> Result<String> {
        Ok(self.response.clone())
    }
    fn schedule_status(&self, _config_path: &Path) -> Result<String> {
        Ok(self.response.clone())
    }
    fn list_snapshots(&self, _config_path: &Path, _profile: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn prune(&self, _config_path: &Path, _profile: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn check(&self, _config_path: &Path, _profile: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}
