use anyhow::Result;
use backup::commands::schedule::{
    execute_schedule_disable, execute_schedule_enable, execute_schedule_enable_with_settings,
    execute_schedule_status,
};
use backup::runner::scheduler::{BackupScheduler, SchedulerSettings};
mod support;
use std::path::Path;
use std::sync::Mutex;
use support::MockScheduler;
use tempfile::tempdir;

fn write_profiles(path: &std::path::Path) {
    std::fs::write(path, "version: '2'\nprofiles: {}\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[test]
fn test_execute_schedule_commands() {
    let mock = MockScheduler::new(0, "scheduled successfully");
    let temp = tempdir().unwrap();
    let path = temp.path().join("profiles.yml");
    write_profiles(&path);

    let res_enable = execute_schedule_enable(&path, &mock).unwrap();
    assert_eq!(res_enable, "scheduled successfully");

    let res_disable = execute_schedule_disable(&path, &mock).unwrap();
    assert_eq!(res_disable, "scheduled successfully");

    let res_status = execute_schedule_status(&mock).unwrap();
    assert_eq!(res_status, "scheduled successfully");
    assert_eq!(
        mock.calls.lock().unwrap().as_slice(),
        ["enable", "disable", "status"]
    );
}

#[test]
fn schedule_enable_propagates_runner_failure() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("profiles.yml");
    write_profiles(&path);
    let runner = MockScheduler::new(1, "systemd unavailable");

    let error = execute_schedule_enable(&path, &runner).unwrap_err();

    assert!(error.to_string().contains("systemd unavailable"));
}

#[test]
fn schedule_status_reports_unavailable_without_unified_backup_configuration() {
    let runner = MockScheduler::new(1, "systemd unavailable");

    let error = execute_schedule_status(&runner).unwrap_err();
    assert!(error.to_string().contains("systemd unavailable"));
    assert_eq!(runner.calls.lock().unwrap().as_slice(), ["status"]);
}

struct PreservingScheduler {
    calls: Mutex<Vec<&'static str>>,
}

impl PreservingScheduler {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl BackupScheduler for PreservingScheduler {
    fn enable(&self, _profiles_path: &Path) -> Result<String> {
        self.calls.lock().unwrap().push("enable");
        Ok("enabled".into())
    }

    fn disable(&self) -> Result<String> {
        self.calls.lock().unwrap().push("disable");
        Ok("disabled".into())
    }

    fn status(&self) -> Result<String> {
        self.calls.lock().unwrap().push("status");
        Ok("status".into())
    }

    fn enable_with_settings(
        &self,
        _profiles_path: &Path,
        _settings: &SchedulerSettings,
    ) -> Result<String> {
        self.calls.lock().unwrap().push("enable_with_settings");
        anyhow::bail!("replacement failed")
    }

    fn enable_preserving_state(
        &self,
        _profiles_path: &Path,
        _settings: &SchedulerSettings,
    ) -> Result<String> {
        self.calls.lock().unwrap().push("enable_preserving_state");
        Ok("preserved".into())
    }
}

#[test]
fn schedule_enable_uses_state_preserving_registration() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("profiles.yaml");
    write_profiles(&path);
    let scheduler = PreservingScheduler::new();

    let result =
        execute_schedule_enable_with_settings(&path, &scheduler, &SchedulerSettings::auto())
            .unwrap();

    assert_eq!(result, "preserved");
    assert_eq!(
        scheduler.calls.lock().unwrap().as_slice(),
        ["enable_preserving_state"]
    );
}
