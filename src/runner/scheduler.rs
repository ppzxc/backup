use crate::runner::executor::{CommandOutput, CommandRunner};
use anyhow::{Result, bail};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

const UNIT_NAME: &str = "backup-pipeline";
const CRON_MARKER: &str = "# backup-pipeline";
const DEFAULT_CALENDAR: &str = "*-*-* 03:00:00";

pub trait BackupScheduler {
    fn enable(&self, profiles_path: &Path) -> Result<String>;
    fn disable(&self) -> Result<String>;
    fn status(&self) -> Result<String>;
}

pub struct SystemScheduler<'a, E: CommandRunner> {
    executor: &'a E,
    binary: String,
}

impl<'a, E: CommandRunner> SystemScheduler<'a, E> {
    pub fn new(executor: &'a E, binary: impl Into<String>) -> Self {
        Self {
            executor,
            binary: binary.into(),
        }
    }

    fn systemd_available(&self) -> Result<bool> {
        if std::env::var_os("BACKUP_TEST_FORCE_CRON").is_some() {
            return Ok(false);
        }
        Ok(self.executor.run("systemctl", &["--version"])?.status_code == 0)
    }

    fn checked(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = self.executor.run(program, args)?;
        if output.status_code != 0 {
            bail!("{program} failed: {}", error_message(&output));
        }
        Ok(output.stdout)
    }

    fn cron_contents(&self) -> Result<String> {
        let output = self.executor.run("crontab", &["-l"])?;
        if output.status_code == 0 {
            Ok(output.stdout)
        } else {
            Ok(String::new())
        }
    }

    fn install_cron(&self, contents: String) -> Result<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(contents.as_bytes())?;
        file.flush()?;
        let path = file.path().to_string_lossy().into_owned();
        self.checked("crontab", &[&path])?;
        Ok(())
    }
}

impl<'a, E: CommandRunner> BackupScheduler for SystemScheduler<'a, E> {
    fn enable(&self, profiles_path: &Path) -> Result<String> {
        let profiles = profiles_path.to_string_lossy();
        if self.systemd_available()? {
            // A transient timer keeps its unit name until stopped.  Clearing an existing timer
            // makes repeated `backup schedule enable` registrations replace the prior run.
            let _ = self
                .executor
                .run("systemctl", &["stop", "backup-pipeline.timer"]);
            let _ = self
                .executor
                .run("systemctl", &["reset-failed", "backup-pipeline.timer"]);
            let _ = self
                .executor
                .run("systemctl", &["stop", "backup-pipeline.service"]);
            let _ = self
                .executor
                .run("systemctl", &["reset-failed", "backup-pipeline.service"]);
            self.checked(
                "systemd-run",
                &[
                    "--unit",
                    UNIT_NAME,
                    &format!("--on-calendar={}", schedule_calendar()),
                    "--timer-property=Persistent=true",
                    &self.binary,
                    "--profiles",
                    &profiles,
                    "run",
                ],
            )?;
            return Ok("Scheduled daily backup run with systemd".into());
        }

        let existing = self.cron_contents()?;
        let filtered = existing
            .lines()
            .filter(|line| !line.contains(CRON_MARKER))
            .collect::<Vec<_>>()
            .join("\n");
        let line = format!(
            "{} {} --profiles {} run {}",
            cron_schedule(),
            shell_quote(&self.binary),
            shell_quote(&profiles),
            CRON_MARKER
        );
        self.install_cron(format!("{}\n{}\n", filtered.trim(), line))?;
        Ok("Scheduled daily backup run with cron".into())
    }

    fn disable(&self) -> Result<String> {
        if self.systemd_available()? {
            let stop = self
                .executor
                .run("systemctl", &["stop", "backup-pipeline.timer"])?;
            if stop.status_code != 0 && !error_message(&stop).contains("not loaded") {
                bail!("systemctl failed: {}", error_message(&stop));
            }
            let _ = self
                .executor
                .run("systemctl", &["reset-failed", "backup-pipeline.timer"])?;
            let _ = self
                .executor
                .run("systemctl", &["stop", "backup-pipeline.service"]);
            let _ = self
                .executor
                .run("systemctl", &["reset-failed", "backup-pipeline.service"]);
            return Ok("Disabled scheduled backup run with systemd".into());
        }
        let existing = self.cron_contents()?;
        let filtered = existing
            .lines()
            .filter(|line| !line.contains(CRON_MARKER))
            .collect::<Vec<_>>()
            .join("\n");
        self.install_cron(format!("{}\n", filtered.trim()))?;
        Ok("Disabled scheduled backup run with cron".into())
    }

    fn status(&self) -> Result<String> {
        if self.systemd_available()? {
            return self.checked("systemctl", &["is-active", "backup-pipeline.timer"]);
        }
        let cron = self.cron_contents()?;
        Ok(if cron.contains(CRON_MARKER) {
            "active (cron)".into()
        } else {
            "inactive (cron)".into()
        })
    }
}

fn schedule_calendar() -> String {
    std::env::var("BACKUP_TEST_SCHEDULE_CALENDAR").unwrap_or_else(|_| DEFAULT_CALENDAR.into())
}

fn cron_schedule() -> String {
    if std::env::var_os("BACKUP_TEST_SCHEDULE_CALENDAR").is_some() {
        "* * * * *".into()
    } else {
        "0 3 * * *".into()
    }
}

fn error_message(output: &CommandOutput) -> String {
    if output.stderr.trim().is_empty() {
        output.stdout.trim().into()
    } else {
        output.stderr.trim().into()
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
