use anyhow::Result;
use std::io::Write;
use tempfile::NamedTempFile;
use crate::runner::executor::CommandRunner;

pub trait ResticRunner {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String>;
    fn backup_paths(
        &self,
        repo: &str,
        password: &str,
        targets: &[String],
        excludes: &[String],
    ) -> Result<String>;
    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String>;
}

pub struct ResticTool<'a, E: CommandRunner> {
    executor: &'a E,
}

impl<'a, E: CommandRunner> ResticTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }
}

fn create_temp_password_file(password: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(password.as_bytes())?;
    file.flush()?;
    Ok(file)
}

impl<'a, E: CommandRunner> ResticRunner for ResticTool<'a, E> {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let output = self.executor.run("restic", &["-r", repo, "--password-file", &pass_path, "init"])?;
        Ok(output.stdout)
    }

    fn backup_paths(
        &self,
        repo: &str,
        password: &str,
        targets: &[String],
        excludes: &[String],
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let mut args = vec!["-r", repo, "--password-file", &pass_path, "backup"];
        for t in targets {
            args.push(t);
        }
        for e in excludes {
            args.push("--exclude");
            args.push(e);
        }
        let output = self.executor.run("restic", &args)?;
        Ok(output.stdout)
    }

    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let output = self.executor.run("restic", &["-r", repo, "--password-file", &pass_path, "snapshots"])?;
        Ok(output.stdout)
    }
}

pub struct MockResticRunner {
    pub exit_code: i32,
    pub response: String,
}

impl MockResticRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.to_string(),
        }
    }
}

impl ResticRunner for MockResticRunner {
    fn init_repo(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn backup_paths(
        &self,
        _repo: &str,
        _password: &str,
        _targets: &[String],
        _excludes: &[String],
    ) -> Result<String> {
        Ok(self.response.clone())
    }
    fn list_snapshots(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}

