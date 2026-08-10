use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(unix)]
const SIGKILL: i32 = 9;

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn setpgid(pid: i32, process_group: i32) -> i32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput>;
    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        _env: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        self.run(program, args)
    }
    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        _timeout: Duration,
    ) -> Result<CommandOutput> {
        self.run_with_env(program, args, env)
    }
}

/// A test-only-in-wiring command runner that consumes an exact, ordered call plan.
///
/// The runner deliberately has no default response: an unexpected program, argument list,
/// environment, or timeout is an error at the call site. This keeps command-contract tests from
/// passing when production accidentally omits or changes an external invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictCommandExpectation {
    pub program: String,
    pub args: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub timeout: Option<Duration>,
    pub output: CommandOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCall {
    pub program: String,
    pub args: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub timeout: Option<Duration>,
}

pub struct StrictCommandRunner {
    expectations: Mutex<VecDeque<StrictCommandExpectation>>,
    calls: Mutex<Vec<CommandCall>>,
}

impl StrictCommandRunner {
    pub fn new<I>(expectations: I) -> Self
    where
        I: IntoIterator<Item = StrictCommandExpectation>,
    {
        Self {
            expectations: Mutex::new(expectations.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn expectation<I>(
        program: &str,
        args: I,
        environment: &[(&str, &str)],
        output: CommandOutput,
    ) -> StrictCommandExpectation
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        StrictCommandExpectation {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            environment: environment
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
            timeout: None,
            output,
        }
    }

    pub fn expectation_with_timeout(
        mut expectation: StrictCommandExpectation,
        timeout: Duration,
    ) -> StrictCommandExpectation {
        expectation.timeout = Some(timeout);
        expectation
    }

    pub fn calls(&self) -> Vec<CommandCall> {
        self.calls
            .lock()
            .expect("strict runner calls poisoned")
            .clone()
    }

    pub fn assert_exhausted(&self) -> Result<()> {
        let remaining = self
            .expectations
            .lock()
            .expect("strict runner expectations poisoned");
        if remaining.is_empty() {
            Ok(())
        } else {
            anyhow::bail!(
                "strict command runner has {} unexpected remaining expectation(s): {}",
                remaining.len(),
                format_call(
                    &remaining[0].program,
                    &remaining[0]
                        .args
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    &remaining[0]
                        .environment
                        .iter()
                        .map(|(key, value)| (key.as_str(), value.as_str()))
                        .collect::<Vec<_>>()
                )
            )
        }
    }

    fn consume(
        &self,
        program: &str,
        args: &[&str],
        environment: &[(&str, &str)],
        timeout: Option<Duration>,
    ) -> Result<CommandOutput> {
        let call = CommandCall {
            program: program.into(),
            args: args.iter().map(|arg| (*arg).into()).collect(),
            environment: environment
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
            timeout,
        };
        self.calls
            .lock()
            .expect("strict runner calls poisoned")
            .push(call.clone());

        let mut expectations = self
            .expectations
            .lock()
            .expect("strict runner expectations poisoned");
        let Some(expected) = expectations.front() else {
            anyhow::bail!(
                "unexpected command: {}",
                format_call(program, args, environment)
            );
        };

        let actual = format_call(program, args, environment);
        let expected_args = expected.args.iter().map(String::as_str).collect::<Vec<_>>();
        let expected_environment = expected
            .environment
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect::<Vec<_>>();
        let expected_call = format_call(&expected.program, &expected_args, &expected_environment);
        if expected.program != call.program
            || expected.args != call.args
            || expected.environment != call.environment
            || expected.timeout != call.timeout
        {
            anyhow::bail!("unexpected command: expected {expected_call}, got {actual}");
        }
        Ok(expectations
            .pop_front()
            .expect("strict expectation disappeared while locked")
            .output)
    }
}

impl CommandRunner for StrictCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        self.consume(program, args, &[], None)
    }

    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        environment: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        self.consume(program, args, environment, None)
    }

    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        environment: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<CommandOutput> {
        self.consume(program, args, environment, Some(timeout))
    }
}

fn format_call(program: &str, args: &[&str], environment: &[(&str, &str)]) -> String {
    let env = environment
        .iter()
        .map(|(key, value)| {
            if is_sensitive_name(key) {
                format!("{key}=<redacted>")
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let args = args
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let previous = index.checked_sub(1).and_then(|index| args.get(index));
            if previous.is_some_and(|value| is_sensitive_name(value))
                || is_sensitive_value(argument)
            {
                "<redacted>"
            } else {
                argument
            }
        })
        .collect::<Vec<_>>();
    format!(
        "{program} [{}]{}",
        args.join(" "),
        if env.is_empty() {
            String::new()
        } else {
            format!(" env=[{env}]")
        }
    )
}

fn is_sensitive_name(value: &str) -> bool {
    let normalized = value.trim_start_matches('-').to_ascii_lowercase();
    [
        "password",
        "secret",
        "token",
        "credential",
        "access-key",
        "secret-access-key",
        "connection-url",
        "url",
        "uri",
        "repository",
        "aws_secret_access_key",
        "aws_access_key_id",
        "mysql_pwd",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn is_sensitive_value(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("://")
        || normalized.starts_with("s3:")
        || normalized.starts_with("sftp:")
        || normalized.starts_with("mysql:")
        || normalized.starts_with("postgres:")
        || normalized.starts_with("postgresql:")
}

pub struct SystemExecutor;

impl CommandRunner for SystemExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute process: {}", program))?;

        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        let output = Command::new(program)
            .args(args)
            .envs(env.iter().copied())
            .output()
            .with_context(|| format!("Failed to execute process: {}", program))?;
        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<CommandOutput> {
        let mut command = Command::new(program);
        command
            .args(args)
            .envs(env.iter().copied())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                if setpgid(0, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to execute process: {}", program))?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child.wait_with_output()?;
                    return Ok(CommandOutput {
                        status_code: status.code().unwrap_or(-1),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    });
                }
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        terminate_process_tree(&mut child);
                        return Ok(CommandOutput {
                            status_code: -1,
                            stdout: String::new(),
                            stderr: format!(
                                "Process execution timed out after {} seconds; process group terminated",
                                timeout.as_secs()
                            ),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    anyhow::bail!("Error waiting for process: {}", e);
                }
            }
        }
    }
}

fn terminate_process_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let process_group = -(child.id() as i32);
        // The timed command is placed in its own process group before exec. Killing the
        // negative process-group ID reaches descendants such as a shell-spawned restic helper.
        unsafe {
            let _ = kill(process_group, SIGKILL);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}
