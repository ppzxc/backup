use crate::runner::executor::{CommandOutput, CommandRunner};
use anyhow::{Result, bail};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

const UNIT_NAME: &str = "backup-pipeline";
const CRON_MARKER: &str = "# backup-pipeline";
const CRON_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
pub const DEFAULT_SCHEDULE_CALENDAR: &str = "*-*-* 03:00:00";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerMode {
    Auto,
    Systemd,
    Cron,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerSettings {
    pub mode: SchedulerMode,
    pub calendar: String,
    pub force_cron: bool,
}

impl SchedulerSettings {
    pub fn new(mode: SchedulerMode, calendar: impl Into<String>) -> Self {
        Self {
            mode,
            calendar: calendar.into(),
            force_cron: false,
        }
    }

    pub fn with_force_cron(mut self, force_cron: bool) -> Self {
        self.force_cron = force_cron;
        self
    }

    pub fn auto() -> Self {
        Self::new(SchedulerMode::Auto, DEFAULT_SCHEDULE_CALENDAR)
    }
}

pub trait BackupScheduler {
    fn enable(&self, profiles_path: &Path) -> Result<String>;
    fn disable(&self) -> Result<String>;
    fn status(&self) -> Result<String>;

    fn enable_with_mode(&self, profiles_path: &Path, _mode: SchedulerMode) -> Result<String> {
        self.enable(profiles_path)
    }

    fn disable_with_mode(&self, _mode: SchedulerMode) -> Result<String> {
        self.disable()
    }

    fn status_with_mode(&self, _mode: SchedulerMode) -> Result<String> {
        self.status()
    }

    fn enable_with_settings(
        &self,
        profiles_path: &Path,
        settings: &SchedulerSettings,
    ) -> Result<String> {
        self.enable_with_mode(profiles_path, settings.mode)
    }

    fn disable_with_settings(&self, settings: &SchedulerSettings) -> Result<String> {
        self.disable_with_mode(settings.mode)
    }

    fn status_with_settings(&self, settings: &SchedulerSettings) -> Result<String> {
        self.status_with_mode(settings.mode)
    }
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

    fn systemd_available(&self, settings: &SchedulerSettings) -> Result<bool> {
        match settings.mode {
            SchedulerMode::Systemd => Ok(true),
            SchedulerMode::Cron => Ok(false),
            SchedulerMode::Auto => Ok(!settings.force_cron
                && self.executor.run("systemctl", &["--version"])?.status_code == 0),
        }
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
        self.enable_with_settings(profiles_path, &SchedulerSettings::auto())
    }

    fn enable_with_mode(&self, profiles_path: &Path, mode: SchedulerMode) -> Result<String> {
        self.enable_with_settings(
            profiles_path,
            &SchedulerSettings::new(mode, DEFAULT_SCHEDULE_CALENDAR),
        )
    }

    fn enable_with_settings(
        &self,
        profiles_path: &Path,
        settings: &SchedulerSettings,
    ) -> Result<String> {
        let profiles = profiles_path.to_string_lossy();
        if self.systemd_available(settings)? {
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
                    &format!("--on-calendar={}", settings.calendar),
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
        let schedule = cron_schedule(&settings.calendar)?;
        let line = cron_entry(&self.binary, &profiles, &schedule);
        self.install_cron(format!("{}\n{}\n", filtered.trim(), line))?;
        Ok("Scheduled daily backup run with cron".into())
    }

    fn disable(&self) -> Result<String> {
        self.disable_with_settings(&SchedulerSettings::auto())
    }

    fn disable_with_mode(&self, mode: SchedulerMode) -> Result<String> {
        self.disable_with_settings(&SchedulerSettings::new(mode, DEFAULT_SCHEDULE_CALENDAR))
    }

    fn disable_with_settings(&self, settings: &SchedulerSettings) -> Result<String> {
        if self.systemd_available(settings)? {
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
        self.status_with_settings(&SchedulerSettings::auto())
    }

    fn status_with_mode(&self, mode: SchedulerMode) -> Result<String> {
        self.status_with_settings(&SchedulerSettings::new(mode, DEFAULT_SCHEDULE_CALENDAR))
    }

    fn status_with_settings(&self, settings: &SchedulerSettings) -> Result<String> {
        if self.systemd_available(settings)? {
            let output = self
                .executor
                .run("systemctl", &["is-active", "backup-pipeline.timer"])?;
            if output.status_code == 0 {
                return Ok(output.stdout);
            }
            if output.stdout.trim().is_empty() {
                bail!("systemctl failed: {}", error_message(&output));
            }
            return Ok(output.stdout.trim().into());
        }
        let cron = self.cron_contents()?;
        Ok(if cron.contains(CRON_MARKER) {
            "active (cron)".into()
        } else {
            "inactive (cron)".into()
        })
    }
}

fn cron_schedule(calendar: &str) -> Result<String> {
    match calendar {
        DEFAULT_SCHEDULE_CALENDAR => Ok("0 3 * * *".into()),
        "*-*-* *:*:00" => Ok("* * * * *".into()),
        _ => bail!("calendar '{calendar}' cannot be represented safely by cron; use systemd mode"),
    }
}

fn cron_entry(binary: &str, profiles: &str, schedule: &str) -> String {
    format!(
        "{schedule} PATH={CRON_PATH} {} --profiles {} run {CRON_MARKER}",
        shell_quote(binary),
        shell_quote(profiles),
    )
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

#[cfg(test)]
mod tests {
    use super::cron_entry;

    #[test]
    fn cron_entry_exposes_installed_tool_path() {
        let entry = cron_entry(
            "/usr/local/bin/backup",
            "/etc/backup/profiles.yaml",
            "* * * * *",
        );

        assert!(
            entry.contains("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
        );
        assert!(entry.contains("/usr/local/bin/backup"));
    }
}
