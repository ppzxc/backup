use crate::runner::executor::{CommandOutput, CommandRunner};
use anyhow::{Error, Result, bail};
use std::io::Write;
use std::io::{self, ErrorKind};
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
    platform_capabilities: Option<crate::platform::PlatformCapabilities>,
}

impl SchedulerSettings {
    pub fn new(mode: SchedulerMode, calendar: impl Into<String>) -> Self {
        Self {
            mode,
            calendar: calendar.into(),
            force_cron: false,
            platform_capabilities: None,
        }
    }

    pub fn with_force_cron(mut self, force_cron: bool) -> Self {
        self.force_cron = force_cron;
        self
    }

    pub fn with_platform_capabilities(
        mut self,
        capabilities: crate::platform::PlatformCapabilities,
    ) -> Self {
        self.platform_capabilities = Some(capabilities);
        self
    }

    pub fn platform_capabilities(&self) -> Option<&crate::platform::PlatformCapabilities> {
        self.platform_capabilities.as_ref()
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

    /// Registers a schedule while giving a concrete scheduler a chance to restore its
    /// previously active state if replacement fails.
    fn enable_preserving_state(
        &self,
        profiles_path: &Path,
        settings: &SchedulerSettings,
    ) -> Result<String> {
        self.enable_with_settings(profiles_path, settings)
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
            SchedulerMode::Systemd => {
                if let Some(capabilities) = settings.platform_capabilities() {
                    if !capabilities.systemd_available {
                        bail!("systemd is unavailable according to the platform capability probe");
                    }
                }
                Ok(true)
            }
            SchedulerMode::Cron => Ok(false),
            SchedulerMode::Auto => {
                if settings.force_cron {
                    return Ok(false);
                }
                if let Some(capabilities) = settings.platform_capabilities() {
                    return Ok(capabilities.systemd_available);
                }
                match self.executor.run("systemctl", &["--version"]) {
                    Ok(output) if output.status_code == 0 => Ok(true),
                    Ok(output) if systemd_is_unavailable(&output) => Ok(false),
                    Ok(output) => bail!(
                        "systemd capability probe failed: {}",
                        error_message(&output)
                    ),
                    Err(error) if command_is_unavailable(&error) => Ok(false),
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn checked(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = self.executor.run(program, args)?;
        if output.status_code != 0 {
            bail!("{program} failed: {}", error_message(&output));
        }
        Ok(output.stdout)
    }

    fn checked_or_missing(&self, program: &str, args: &[&str]) -> Result<()> {
        let output = self.executor.run(program, args)?;
        if output.status_code == 0 {
            return Ok(());
        }
        if missing_systemd_unit(&output) {
            return Ok(());
        }
        bail!("{program} failed: {}", error_message(&output));
    }

    fn cron_contents(&self) -> Result<String> {
        let output = self.executor.run("crontab", &["-l"])?;
        if output.status_code == 0 {
            Ok(output.stdout)
        } else if output.status_code == 1
            && output.stderr.to_ascii_lowercase().contains("no crontab")
        {
            Ok(String::new())
        } else {
            bail!("crontab failed: {}", error_message(&output));
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

    fn ensure_cron_capability(&self, settings: &SchedulerSettings) -> Result<()> {
        if let Some(capabilities) = settings.platform_capabilities() {
            if !capabilities.cron_available {
                bail!("cron is unavailable according to the platform capability probe");
            }
        }
        Ok(())
    }

    fn ensure_cron_registration_capability(&self, settings: &SchedulerSettings) -> Result<()> {
        self.ensure_cron_capability(settings)?;
        if let Some(capabilities) = settings.platform_capabilities() {
            if !capabilities.crond_running {
                bail!("crond is not running; refusing to register a cron schedule");
            }
        }
        Ok(())
    }

    fn systemd_timer_active(&self) -> Result<bool> {
        let output = self
            .executor
            .run("systemctl", &["is-active", "backup-pipeline.timer"])?;
        match output.status_code {
            0 => Ok(true),
            3 => Ok(false),
            _ => bail!("systemctl failed: {}", error_message(&output)),
        }
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

        self.ensure_cron_registration_capability(settings)?;
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

    fn enable_preserving_state(
        &self,
        profiles_path: &Path,
        settings: &SchedulerSettings,
    ) -> Result<String> {
        if !self.systemd_available(settings)? {
            return self.enable_with_settings(profiles_path, settings);
        }

        let was_active = self.systemd_timer_active()?;
        match self.enable_with_settings(profiles_path, settings) {
            Ok(output) => Ok(output),
            Err(error) if was_active => {
                if let Err(restore_error) =
                    self.checked("systemctl", &["start", "backup-pipeline.timer"])
                {
                    bail!(
                        "{error}; failed to restore the previous scheduled backup: {restore_error}"
                    );
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    fn disable(&self) -> Result<String> {
        self.disable_with_settings(&SchedulerSettings::auto())
    }

    fn disable_with_mode(&self, mode: SchedulerMode) -> Result<String> {
        self.disable_with_settings(&SchedulerSettings::new(mode, DEFAULT_SCHEDULE_CALENDAR))
    }

    fn disable_with_settings(&self, settings: &SchedulerSettings) -> Result<String> {
        if self.systemd_available(settings)? {
            self.checked_or_missing("systemctl", &["stop", "backup-pipeline.timer"])?;
            self.checked_or_missing("systemctl", &["reset-failed", "backup-pipeline.timer"])?;
            self.checked_or_missing("systemctl", &["stop", "backup-pipeline.service"])?;
            self.checked_or_missing("systemctl", &["reset-failed", "backup-pipeline.service"])?;
            return Ok("Disabled scheduled backup run with systemd".into());
        }
        self.ensure_cron_capability(settings)?;
        let existing = self.cron_contents()?;
        let filtered = existing
            .lines()
            .filter(|line| !line.contains(CRON_MARKER))
            .collect::<Vec<_>>()
            .join("\n");
        if !existing.contains(CRON_MARKER) {
            return Ok("No scheduled backup run found with cron".into());
        }
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
            if matches!(output.status_code, 1 | 3 | 4) {
                if systemd_state_text(output.stdout.trim()) {
                    return Ok(output.stdout.trim().into());
                }
                if systemd_state_text(output.stderr.trim()) {
                    return Ok(output.stderr.trim().into());
                }
            }
            bail!("systemctl failed: {}", error_message(&output));
        }
        self.ensure_cron_capability(settings)?;
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

fn command_is_unavailable(error: &Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == ErrorKind::NotFound)
    })
}

fn systemd_is_unavailable(output: &CommandOutput) -> bool {
    let message = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    message.contains("no systemd")
        || message.contains("systemd unavailable")
        || message.contains("systemd is not running")
        || message.contains("not been booted with systemd")
        || message.contains("command not found")
}

fn missing_systemd_unit(output: &CommandOutput) -> bool {
    let message = output.stderr.to_ascii_lowercase();
    message.contains("unit ")
        && (message.contains("not loaded")
            || message.contains("not found")
            || message.contains("does not exist"))
}

fn systemd_state_text(value: &str) -> bool {
    matches!(
        value,
        "active"
            | "inactive"
            | "failed"
            | "activating"
            | "deactivating"
            | "reloading"
            | "maintenance"
            | "unknown"
    )
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
