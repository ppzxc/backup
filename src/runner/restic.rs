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
    fn list_snapshots_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<String> {
        if sftp_args.is_some() {
            anyhow::bail!("ResticRunner implementation does not support native SFTP arguments");
        }
        self.list_snapshots_with_env(repo, password, env)
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
    fn list_snapshot_infos_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(
            &self.list_snapshots_with_env_and_sftp_args(repo, password, env, sftp_args)?,
        )
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
    fn restore_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<String> {
        if sftp_args.is_some() {
            anyhow::bail!("ResticRunner implementation does not support native SFTP arguments");
        }
        self.restore_with_env(repo, password, snapshot, target, env)
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
    fn restore_with_env_and_sftp_args_and_timeout(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
        timeout: Duration,
    ) -> Result<String> {
        match sftp_args {
            Some(sftp_args) => self.restore_with_env_and_sftp_args(
                repo,
                password,
                snapshot,
                target,
                env,
                Some(sftp_args),
            ),
            None => {
                self.restore_with_env_and_timeout(repo, password, snapshot, target, env, timeout)
            }
        }
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
        sftp_args: Option<&str>,
        json: bool,
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let sftp_option = sftp_args.map(|args| format!("sftp.args={args}"));
        let mut args = vec!["-r".to_owned(), repo.to_owned()];
        if let Some(option) = &sftp_option {
            args.extend(["--option".into(), option.clone()]);
        }
        args.extend([
            "--password-file".into(),
            pass_path.into_owned(),
            "snapshots".into(),
        ]);
        if json {
            args.push("--json".into());
        }
        let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        let output = match environment {
            Some(environment) => self
                .executor
                .run_with_env("restic", &arg_refs, environment)?,
            None => self.executor.run("restic", &arg_refs)?,
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
        sftp_args: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let pass_path = pass_file.path().to_string_lossy();
        let sftp_option = sftp_args.map(|args| format!("sftp.args={args}"));
        let mut args = vec!["-r".to_owned(), repo.to_owned()];
        if let Some(option) = &sftp_option {
            args.extend(["--option".into(), option.clone()]);
        }
        args.extend([
            "--password-file".into(),
            pass_path.into_owned(),
            "restore".into(),
            snapshot.to_owned(),
            "--target".into(),
            target.to_owned(),
        ]);
        let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        let output = match timeout {
            Some(timeout) => {
                self.executor
                    .run_with_timeout("restic", &arg_refs, environment, timeout)?
            }
            None if environment.is_empty() => self.executor.run("restic", &arg_refs)?,
            None => self
                .executor
                .run_with_env("restic", &arg_refs, environment)?,
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
        self.list_snapshot_output(repo, password, None, None, false)
    }
    fn list_snapshots_with_env(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.list_snapshot_output(repo, password, Some(env), None, false)
    }
    fn list_snapshots_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<String> {
        self.list_snapshot_output(repo, password, Some(env), sftp_args, false)
    }
    fn list_snapshot_infos(&self, repo: &str, password: &str) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshot_output(repo, password, None, None, true)?)
            .map_err(anyhow::Error::new)
    }
    fn list_snapshot_infos_with_env(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshot_output(repo, password, Some(env), None, true)?)
            .map_err(anyhow::Error::new)
    }
    fn list_snapshot_infos_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<Vec<SnapshotInfo>> {
        parse_snapshot_json(&self.list_snapshot_output(
            repo,
            password,
            Some(env),
            sftp_args,
            true,
        )?)
        .map_err(anyhow::Error::new)
    }
    fn restore(&self, repo: &str, password: &str, snapshot: &str, target: &str) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, &[], None, None)
    }
    fn restore_with_env(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, env, None, None)
    }
    fn restore_with_env_and_sftp_args(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
    ) -> Result<String> {
        self.restore_output(repo, password, snapshot, target, env, sftp_args, None)
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
        self.restore_output(repo, password, snapshot, target, env, None, Some(timeout))
    }
    fn restore_with_env_and_sftp_args_and_timeout(
        &self,
        repo: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        env: &[(&str, &str)],
        sftp_args: Option<&str>,
        timeout: Duration,
    ) -> Result<String> {
        self.restore_output(
            repo,
            password,
            snapshot,
            target,
            env,
            sftp_args,
            Some(timeout),
        )
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
