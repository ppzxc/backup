use crate::runner::executor::CommandRunner;
use crate::runner::snapshot::{SnapshotInfo, parse_snapshot_json};
use anyhow::Result;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

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
    fn list_snapshot_infos(&self, repo: &str, password: &str) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshots(repo, password)?).map_err(anyhow::Error::new)
    }
    fn list_snapshots_with_env(
        &self,
        repo: &str,
        password: &str,
        _env: &[(&str, &str)],
    ) -> Result<String> {
        self.list_snapshots(repo, password)
    }
    fn list_snapshot_infos_with_env(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshots_with_env(repo, password, env)?)
            .map_err(anyhow::Error::new)
    }
    fn restore(&self, repo: &str, password: &str, snapshot: &str, target: &str) -> Result<String>;
    fn restore_with_env(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        _env: &[(&str, &str)],
    ) -> Result<String> {
        self.restore(repo, password, snapshot, target)
    }
    fn restore_with_env_and_timeout(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        _timeout: Duration,
    ) -> Result<String> {
        self.restore_with_env(repo, password, snapshot, target, env)
    }
    fn backup_command(
        &self,
        repo: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
    ) -> Result<String>;
    fn backup_command_with_env(
        &self,
        repo: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        env: &[(&str, &str)],
    ) -> Result<String>;
    fn backup_command_with_env_and_tag(
        &self,
        _repo: &str,
        _password: &str,
        _filename: &str,
        _program: &str,
        _args: &[String],
        tag: &str,
        _env: &[(&str, &str)],
    ) -> Result<String> {
        anyhow::bail!("ResticRunner does not support tagged Database Stream backups (tag '{tag}')")
    }
}

pub struct ResticTool<'a, E: CommandRunner + ?Sized> {
    executor: &'a E,
}

impl<'a, E: CommandRunner + ?Sized> ResticTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }

    fn list_snapshot_output(
        &self,
        repo: &str,
        password: &str,
        environment: Option<&[(&str, &str)]>,
        json: bool,
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let mut args = vec!["-r", repo, "--password-file", &pass_path, "snapshots"];
        if json {
            args.push("--json");
        }
        let output = match environment {
            Some(environment) => self.executor.run_with_env("restic", &args, environment)?,
            None => self.executor.run("restic", &args)?,
        };
        Self::checked(output)
    }

    fn checked(output: crate::runner::executor::CommandOutput) -> Result<String> {
        if output.status_code != 0 {
            anyhow::bail!(
                "restic failed with exit code {}: {}",
                output.status_code,
                output.stderr
            );
        }
        Ok(output.stdout)
    }

    fn restore_output(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        environment: &[(&str, &str)],
        timeout: Option<Duration>,
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let args = [
            "-r",
            repo,
            "--password-file",
            &pass_path,
            "restore",
            snapshot,
            "--target",
            target,
        ];
        let output = match timeout {
            Some(timeout) => {
                self.executor
                    .run_with_timeout("restic", &args, environment, timeout)?
            }
            None if environment.is_empty() => self.executor.run("restic", &args)?,
            None => self.executor.run_with_env("restic", &args, environment)?,
        };
        Self::checked(output)
    }
}

fn create_temp_password_file(password: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(password.as_bytes())?;
    file.flush()?;
    Ok(file)
}

impl<'a, E: CommandRunner + ?Sized> ResticRunner for ResticTool<'a, E> {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let output = self.executor.run(
            "restic",
            &["-r", repo, "--password-file", &pass_path, "init"],
        )?;
        Self::checked(output)
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
        Self::checked(output)
    }

    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String> {
        self.list_snapshot_output(repo, password, None, false)
    }
    fn list_snapshots_with_env(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.list_snapshot_output(repo, password, Some(env), false)
    }
    fn list_snapshot_infos(&self, repo: &str, password: &str) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshot_output(repo, password, None, true)?)
            .map_err(anyhow::Error::new)
    }
    fn list_snapshot_infos_with_env(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshot_output(repo, password, Some(env), true)?)
            .map_err(anyhow::Error::new)
    }
    fn restore(&self, repo: &str, password: &str, snapshot: &str, target: &str) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, &[], None)
    }
    fn restore_with_env(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, env, None)
    }
    fn restore_with_env_and_timeout(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, env, Some(timeout))
    }
    fn backup_command(
        &self,
        repo: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
    ) -> Result<String> {
        backup_command_output(
            self.executor,
            repo,
            password,
            filename,
            program,
            args,
            None,
            None,
        )
    }
    fn backup_command_with_env(
        &self,
        repo: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        env: &[(&str, &str)],
    ) -> Result<String> {
        backup_command_output(
            self.executor,
            repo,
            password,
            filename,
            program,
            args,
            None,
            Some(env),
        )
    }

    fn backup_command_with_env_and_tag(
        &self,
        repo: &str,
        password: &str,
        filename: &str,
        program: &str,
        args: &[String],
        tag: &str,
        env: &[(&str, &str)],
    ) -> Result<String> {
        backup_command_output(
            self.executor,
            repo,
            password,
            filename,
            program,
            args,
            Some(tag),
            Some(env),
        )
    }
}

fn backup_command_output<E: CommandRunner + ?Sized>(
    executor: &E,
    repo: &str,
    password: &str,
    filename: &str,
    program: &str,
    args: &[String],
    tag: Option<&str>,
    env: Option<&[(&str, &str)]>,
) -> Result<String> {
    let pass_file = create_temp_password_file(password)?;
    let pass_path = pass_file.path().to_string_lossy();
    let mut command = vec!["-r", repo, "--password-file", &pass_path, "backup"];
    if let Some(tag) = tag {
        command.extend(["--tag", tag]);
    }
    command.extend([
        "--stdin-from-command",
        "--stdin-filename",
        filename,
        "--",
        program,
    ]);
    command.extend(args.iter().map(String::as_str));
    let output = match env {
        Some(env) => executor.run_with_env("restic", &command, env)?,
        None => executor.run("restic", &command)?,
    };
    if output.status_code != 0 {
        anyhow::bail!(
            "restic failed with exit code {}: {}",
            output.status_code,
            output.stderr
        );
    }
    Ok(output.stdout)
}
