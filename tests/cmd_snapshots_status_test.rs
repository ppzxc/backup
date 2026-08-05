use anyhow::Result;
use backup::commands::snapshots::execute_snapshots_from_profiles;
use backup::commands::status::execute_status_from_profiles_config;
use backup::runner::restic::ResticRunner;
use backup::runner::resticprofile::ResticProfileRunner;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::TempDir;

struct SnapshotRunner {
    responses: Mutex<Vec<Result<String, String>>>,
    calls: Mutex<Vec<String>>,
}

impl SnapshotRunner {
    fn new(responses: impl IntoIterator<Item = Result<String, String>>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn next(&self, operation: &str) -> Result<String> {
        self.calls.lock().unwrap().push(operation.into());
        match self.responses.lock().unwrap().remove(0) {
            Ok(output) => Ok(output),
            Err(error) => anyhow::bail!("{error}"),
        }
    }
}

impl ResticRunner for SnapshotRunner {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        self.next("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        self.next("backup")
    }

    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        self.next("list_snapshots")
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        self.next("restore")
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        self.next("backup_command")
    }

    fn backup_command_with_env(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &str,
        _: &[String],
        _: &[(&str, &str)],
    ) -> Result<String> {
        self.next("backup_command_with_env")
    }
}

fn write_password(path: &Path, value: &str) {
    std::fs::write(path, value).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn storage_config(temp: &TempDir, secondary: bool) -> PathBuf {
    write_password(&temp.path().join("primary-password"), "primary-secret");
    let mut yaml = String::from(
        "version: '2'\nprofiles:\n  primary:\n    repository: s3:https://primary.example/backup\n    password-file: primary-password\n",
    );
    if secondary {
        write_password(&temp.path().join("secondary-password"), "secondary-secret");
        yaml.push_str(
            "  secondary:\n    repository: s3:https://secondary.example/backup\n    password-file: secondary-password\n",
        );
    }
    let path = temp.path().join("profiles.yaml");
    std::fs::write(&path, yaml).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}

#[test]
fn snapshots_primary_failure_is_not_reported_as_success() {
    let temp = tempfile::tempdir().unwrap();
    let path = storage_config(&temp, false);
    let runner = SnapshotRunner::new([Err("primary-secret repository failed".into())]);

    let error = execute_snapshots_from_profiles(
        &backup::config::model::ResticProfileConfig::load_from_path(&path).unwrap(),
        &path,
        &runner,
    )
    .unwrap_err();

    assert!(error.to_string().contains("snapshots"));
    assert!(!error.to_string().contains("primary-secret"));
    assert_eq!(runner.calls.lock().unwrap().as_slice(), ["list_snapshots"]);
}

#[test]
fn snapshots_secondary_failure_keeps_primary_result_and_returns_warning() {
    let temp = tempfile::tempdir().unwrap();
    let path = storage_config(&temp, true);
    let runner = SnapshotRunner::new([
        Ok("primary snapshot abc123".into()),
        Err("secondary-secret repository failed".into()),
    ]);
    let config = backup::config::model::ResticProfileConfig::load_from_path(&path).unwrap();

    let output = execute_snapshots_from_profiles(&config, &path, &runner).unwrap();

    assert!(output.contains("primary snapshot abc123"));
    assert!(output.contains("[WARN] Secondary snapshots unavailable"));
    assert!(!output.contains("secondary-secret"));
    assert_eq!(
        runner.calls.lock().unwrap().as_slice(),
        ["list_snapshots", "list_snapshots"]
    );
}

#[test]
fn snapshots_without_secondary_only_query_primary() {
    let temp = tempfile::tempdir().unwrap();
    let path = storage_config(&temp, false);
    let runner = SnapshotRunner::new([Ok("primary only".into())]);
    let config = backup::config::model::ResticProfileConfig::load_from_path(&path).unwrap();

    let output = execute_snapshots_from_profiles(&config, &path, &runner).unwrap();

    assert!(output.contains("Primary snapshots"));
    assert!(!output.contains("Secondary snapshots"));
    assert_eq!(runner.calls.lock().unwrap().as_slice(), ["list_snapshots"]);
}

struct StatusRunner {
    responses: Mutex<Vec<Result<String, String>>>,
    calls: Mutex<Vec<String>>,
}

impl StatusRunner {
    fn new(responses: impl IntoIterator<Item = Result<String, String>>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl ResticProfileRunner for StatusRunner {
    fn backup(&self, _: &Path, _: &str, _: bool) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn init(&self, _: &Path, _: &str) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn schedule_enable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn schedule_disable(&self, _: &Path) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn schedule_status(&self, _: &Path) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn list_snapshots(&self, _: &Path, profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(profile.into());
        match self.responses.lock().unwrap().remove(0) {
            Ok(output) => Ok(output),
            Err(error) => anyhow::bail!("{error}"),
        }
    }
    fn prune(&self, _: &Path, _: &str) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn check(&self, _: &Path, _: &str) -> Result<String> {
        anyhow::bail!("not expected")
    }
    fn copy(&self, _: &Path, _: &str, _: bool) -> Result<String> {
        anyhow::bail!("not expected")
    }
}

fn status_config(temp: &TempDir, active: bool) -> PathBuf {
    let path = temp.path().join("profiles.yaml");
    let backup = if active {
        "\n    backup:\n      source: [/data]"
    } else {
        ""
    };
    std::fs::write(
        &path,
        format!(
            "version: '2'\nprofiles:\n  primary:\n    repository: s3:https://user:status-secret@example/backup\n    password-file: primary-password\n  daily:\n    inherit: primary{backup}\n"
        ),
    )
    .unwrap();
    write_password(&temp.path().join("primary-password"), "status-password");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}

#[test]
fn status_with_no_active_profile_is_a_warning_success() {
    let temp = tempfile::tempdir().unwrap();
    let path = status_config(&temp, false);
    let runner = StatusRunner::new([]);

    let output = execute_status_from_profiles_config(&path, None, &runner).unwrap();

    assert!(output.contains("[WARN] No active backup profiles found"));
    assert!(runner.calls.lock().unwrap().is_empty());
}

#[test]
fn status_masks_repository_credentials_and_preserves_partial_failure() {
    let temp = tempfile::tempdir().unwrap();
    let path = status_config(&temp, true);
    let runner = StatusRunner::new([Err("status-password and status-secret leaked".into())]);

    let error = execute_status_from_profiles_config(&path, None, &runner).unwrap_err();
    let failure = error
        .downcast_ref::<backup::commands::status::StatusCommandFailure>()
        .unwrap();

    assert!(failure.output.contains("Profile: daily"));
    assert!(!failure.output.contains("status-secret"));
    assert!(!failure.message.contains("status-password"));
    assert_eq!(runner.calls.lock().unwrap().as_slice(), ["daily"]);
}

#[test]
fn status_unknown_profile_fails_before_adapter_call() {
    let temp = tempfile::tempdir().unwrap();
    let path = status_config(&temp, true);
    let runner = StatusRunner::new([]);

    let error = execute_status_from_profiles_config(&path, Some("missing"), &runner).unwrap_err();

    assert!(error.to_string().contains("not configured"));
    assert!(runner.calls.lock().unwrap().is_empty());
}
