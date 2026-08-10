use anyhow::Result;
use backup::commands::report::restore_drill::{
    RestoreDrillClock, RestoreDrillStatus, RestoreDrillTimestamp,
};
use backup::commands::report::{
    ReportConfig, RestoreDrillEvidence, RestoreDrillPolicy, RestoreDrillProfileConfig,
    RestoreDrillStorageConfig, execute_restore_drill_evidence_export,
    execute_restore_drill_with_runner_and_clock,
};
use backup::config::model::{DatabaseType, ResticProfileConfig};
use backup::runner::restic::ResticRunner;
use backup::runner::snapshot::{SnapshotInfo, SnapshotJsonError};
use secrecy::SecretString;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tempfile::tempdir;

struct FixedClock {
    samples: Mutex<VecDeque<RestoreDrillTimestamp>>,
}

impl FixedClock {
    fn new(samples: impl IntoIterator<Item = RestoreDrillTimestamp>) -> Self {
        Self {
            samples: Mutex::new(samples.into_iter().collect()),
        }
    }
}

impl RestoreDrillClock for FixedClock {
    fn now(&self) -> RestoreDrillTimestamp {
        self.samples
            .lock()
            .unwrap()
            .pop_front()
            .expect("restore drill clock sample")
    }
}

struct StrictRestoreRunner {
    calls: Mutex<Vec<String>>,
    snapshots: Vec<SnapshotInfo>,
    snapshot_error: Option<SnapshotJsonError>,
    restore_error: Option<String>,
}

impl StrictRestoreRunner {
    fn new(snapshots: Vec<SnapshotInfo>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            snapshots,
            snapshot_error: None,
            restore_error: None,
        }
    }

    fn failing_restore(snapshots: Vec<SnapshotInfo>, error: &str) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            snapshots,
            snapshot_error: None,
            restore_error: Some(error.into()),
        }
    }

    fn malformed_snapshots() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            snapshots: Vec::new(),
            snapshot_error: Some(SnapshotJsonError::Malformed),
            restore_error: None,
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn unsupported(&self, operation: &str) -> Result<String> {
        anyhow::bail!("unexpected restore drill operation: {operation}")
    }
}

impl ResticRunner for StrictRestoreRunner {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        self.unsupported("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        self.unsupported("backup")
    }

    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        self.unsupported("list_snapshots")
    }

    fn list_snapshot_infos_with_env(
        &self,
        repository: &str,
        password: &str,
        environment: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        assert_eq!(repository, "s3:primary-repository");
        assert_eq!(password, "primary-secret");
        assert_eq!(environment, [("AWS_ACCESS_KEY_ID", "access")]);
        self.calls.lock().unwrap().push("list-snapshots".into());
        if let Some(error) = self.snapshot_error {
            return Err(anyhow::Error::new(error));
        }
        Ok(self.snapshots.clone())
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        self.unsupported("restore without environment")
    }

    fn restore_with_env_and_timeout(
        &self,
        repository: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        environment: &[(&str, &str)],
        timeout: std::time::Duration,
    ) -> Result<String> {
        assert_eq!(repository, "s3:primary-repository");
        assert_eq!(password, "primary-secret");
        assert!(
            snapshot == "full-snapshot-001" || snapshot == "full-snapshot-timeout",
            "unexpected snapshot {snapshot}"
        );
        assert_eq!(environment, [("AWS_ACCESS_KEY_ID", "access")]);
        assert_eq!(timeout, std::time::Duration::from_secs(14_400));
        self.calls.lock().unwrap().push("restore".into());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(target).unwrap().permissions().mode() & 0o777,
                0o700,
                "each isolated restore target must be private"
            );
        }
        if let Some(error) = &self.restore_error {
            anyhow::bail!("{error}");
        }
        fs::write(Path::new(target).join("restored.txt"), "restored")?;
        Ok("restored".into())
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        self.unsupported("backup_command")
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
        self.unsupported("backup_command_with_env")
    }
}

fn timestamp(wall_clock: &str, monotonic_milliseconds: u64) -> RestoreDrillTimestamp {
    RestoreDrillTimestamp {
        wall_clock: wall_clock.into(),
        monotonic_milliseconds,
    }
}

fn config(root: &Path) -> ReportConfig {
    let mut config = ReportConfig::default();
    config.profile = "daily-files".into();
    config.primary_repository = "s3:primary-repository".into();
    config.primary_password = SecretString::new("primary-secret".into());
    config.primary_environment = vec![(
        "AWS_ACCESS_KEY_ID".into(),
        SecretString::new("access".into()),
    )];
    config.restore_drill_work_dir = root.join("restore-drill");
    config
}

struct DatabaseProfileRunner {
    calls: Mutex<Vec<String>>,
    snapshots: Vec<SnapshotInfo>,
    restore_filename: Option<String>,
    restore_content: String,
}

impl DatabaseProfileRunner {
    fn new(snapshots: Vec<SnapshotInfo>) -> Self {
        Self::with_restore_output(
            snapshots,
            Some("database.sql"),
            "-- PostgreSQL database dump\nCREATE TABLE items (id integer);",
        )
    }

    fn with_restore_output(
        snapshots: Vec<SnapshotInfo>,
        restore_filename: Option<&str>,
        restore_content: &str,
    ) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            snapshots,
            restore_filename: restore_filename.map(str::to_owned),
            restore_content: restore_content.into(),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn unsupported(&self, operation: &str) -> Result<String> {
        anyhow::bail!("unexpected database restore drill operation: {operation}")
    }
}

impl ResticRunner for DatabaseProfileRunner {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        self.unsupported("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        self.unsupported("backup")
    }

    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        self.unsupported("list_snapshots")
    }

    fn list_snapshot_infos(&self, repository: &str, password: &str) -> Result<Vec<SnapshotInfo>> {
        self.list_snapshot_infos_with_env(repository, password, &[])
    }

    fn list_snapshot_infos_with_env(
        &self,
        repository: &str,
        password: &str,
        environment: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        assert_eq!(repository, "/primary");
        assert_eq!(password, "primary-secret");
        assert!(environment.is_empty());
        self.calls.lock().unwrap().push("list".into());
        Ok(self.snapshots.clone())
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        self.unsupported("restore without environment")
    }

    fn restore_with_env_and_timeout(
        &self,
        repository: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        environment: &[(&str, &str)],
        timeout: std::time::Duration,
    ) -> Result<String> {
        assert_eq!(repository, "/primary");
        assert_eq!(password, "primary-secret");
        assert_eq!(snapshot, "database-snapshot-001");
        assert!(environment.is_empty());
        assert_eq!(timeout, std::time::Duration::from_secs(14_400));
        self.calls.lock().unwrap().push("restore".into());
        if let Some(filename) = &self.restore_filename {
            fs::write(Path::new(target).join(filename), &self.restore_content)?;
        }
        Ok("restored".into())
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        self.unsupported("backup_command")
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
        self.unsupported("backup_command_with_env")
    }
}

#[test]
fn restore_drill_policy_is_loaded_from_audit_metadata_with_documented_defaults() {
    let temp = tempdir().unwrap();
    let password = temp.path().join("primary-password");
    fs::write(&password, "primary-secret").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&password, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  audit:\n    restore-drill-rto-minutes: 7\n    restore-drill-timeout-minutes: 14\n    restore-drill-work-dir: {}\nprofiles:\n  primary:\n    repository: /tmp/primary\n    password-file: {}\n  daily-files:\n    inherit: primary\n    backup:\n      source: [/data]\n",
            temp.path().join("drill-root").display(),
            password.display()
        ),
    )
    .unwrap();
    let profiles = ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let report_config = ReportConfig::from_profiles(&profiles, &profiles_path).unwrap();
    assert_eq!(report_config.restore_drill_policy.rto_minutes, 7);
    assert_eq!(report_config.restore_drill_policy.timeout_minutes, 14);
    assert_eq!(
        report_config.restore_drill_work_dir,
        temp.path().join("drill-root")
    );

    let default_profiles_path = temp.path().join("profiles-defaults.yaml");
    fs::write(
        &default_profiles_path,
        format!(
            "version: '2'\nprofiles:\n  primary:\n    repository: /tmp/primary\n    password-file: {}\n  daily-files:\n    inherit: primary\n    backup:\n      source: [/data]\n",
            password.display()
        ),
    )
    .unwrap();
    let default_profiles = ResticProfileConfig::load_from_path(&default_profiles_path).unwrap();
    let defaults = ReportConfig::from_profiles(&default_profiles, &default_profiles_path).unwrap();
    assert_eq!(defaults.restore_drill_policy.rto_minutes, 120);
    assert_eq!(defaults.restore_drill_policy.timeout_minutes, 240);
    assert_eq!(
        defaults.restore_drill_work_dir,
        Path::new("/var/lib/backup/restore-drill")
    );

    for audit in [
        "restore-drill-rto-minutes: 0",
        "restore-drill-timeout-minutes: 6\n    restore-drill-rto-minutes: 7",
    ] {
        let invalid = temp.path().join("invalid.yaml");
        fs::write(
            &invalid,
            format!(
                "version: '2'\napplication:\n  audit:\n    {audit}\nprofiles:\n  daily-files:\n    backup: {{source: [/data]}}\n"
            ),
        )
        .unwrap();
        assert!(ResticProfileConfig::load_from_path(&invalid).is_err());
    }
}

#[test]
fn restore_drill_restores_database_profile_by_reserved_tag_and_validates_signature() {
    let temp = tempdir().unwrap();
    let password = temp.path().join("primary-password");
    fs::write(&password, "primary-secret").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&password, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  database:\n    profile: database\n    type: postgres\n    connection-url: ${{BACKUP_DATABASE_CONNECTION_URL}}\nprofiles:\n  primary:\n    repository: /primary\n    password-file: {}\n  database:\n    inherit: primary\n",
            password.display()
        ),
    )
    .unwrap();
    fs::write(
        temp.path().join("database-connection-url"),
        "postgres://backup-user:db-secret@db:5432/app",
    )
    .unwrap();
    let profiles = ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let mut config = ReportConfig::from_profiles(&profiles, &profiles_path).unwrap();
    config.restore_drill_work_dir = temp.path().join("restore-drill");

    assert_eq!(config.database_type, Some(DatabaseType::Postgres));
    assert_eq!(config.restore_drill_profiles.len(), 1);
    assert_eq!(
        config.restore_drill_profiles[0].database_type,
        Some(DatabaseType::Postgres)
    );

    let runner = DatabaseProfileRunner::with_restore_output(
        vec![SnapshotInfo {
            id: "database-snapshot-001".into(),
            timestamp: "2026-08-07T09:00:00Z".into(),
            tags: vec!["backup-profile:database".into()],
        }],
        Some("database.sql"),
        "-- PostgreSQL database dump\nCREATE TABLE items (id integer);\n-- db-secret",
    );
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00+09:00", 10_000),
        timestamp("2026-08-07T10:00:01+09:00", 11_000),
        timestamp("2026-08-07T10:00:04+09:00", 14_421),
        timestamp("2026-08-07T10:00:05+09:00", 15_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();

    assert_eq!(runner.calls(), ["list", "restore"]);
    assert_eq!(evidence.overall_status, RestoreDrillStatus::Pass);
    let result = &evidence.storage_results[0];
    assert_eq!(result.profile, "database");
    assert_eq!(result.snapshot_id.as_deref(), Some("database-snapshot-001"));
    let verification = result.database_verification.as_ref().unwrap();
    assert_eq!(verification.db_type, DatabaseType::Postgres);
    assert!(verification.signature_verified);
    assert!(!verification.import_performed);
    assert!(!verification.record_validation_performed);

    let json = backup::commands::report::render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["target_snapshot_id"], "database-snapshot-001");
    assert_eq!(
        value["recovery_results"]["database_verification"]["signature_verified"],
        true
    );
    assert!(!json.contains("BACKUP_DATABASE_CONNECTION_URL"));
    assert!(!json.contains("primary-secret"));
    assert!(!json.contains("postgres://backup-user:db-secret@db:5432/app"));
    assert!(!json.contains("db-secret"));
    let html = backup::commands::report::render_restore_drill_evidence_html(&evidence);
    assert!(!html.contains("postgres://backup-user:db-secret@db:5432/app"));
    assert!(!html.contains("db-secret"));
}

#[test]
fn restore_drill_marks_database_validation_not_performed_when_backend_is_unconfigured() {
    let temp = tempdir().unwrap();
    let mut report_config = config(temp.path());
    report_config.profile = "database".into();
    report_config.database_type = Some(DatabaseType::Mysql);
    report_config.restore_drill_profiles = vec![RestoreDrillProfileConfig {
        profile: "database".into(),
        database_type: Some(DatabaseType::Mysql),
        primary: Some(RestoreDrillStorageConfig {
            backend: "primary".into(),
            repository: String::new(),
            password: SecretString::new(String::new()),
            environment: Vec::new(),
        }),
        primary_diagnostic: None,
        secondary: None,
        secondary_diagnostic: None,
    }];
    let runner = DatabaseProfileRunner::new(Vec::new());
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00+09:00", 10_000),
        timestamp("2026-08-07T10:00:01+09:00", 11_000),
        timestamp("2026-08-07T10:00:02+09:00", 12_000),
        timestamp("2026-08-07T10:00:03+09:00", 13_000),
    ]);

    let evidence =
        execute_restore_drill_with_runner_and_clock(&report_config, &runner, &clock).unwrap();
    assert_eq!(runner.calls(), Vec::<String>::new());
    let primary = &evidence.storage_results[0];
    let verification = primary.database_verification.as_ref().unwrap();
    assert_eq!(primary.status, RestoreDrillStatus::NotPerformed);
    assert_eq!(verification.db_type, DatabaseType::Mysql);
    assert_eq!(
        verification.signature_status,
        RestoreDrillStatus::NotPerformed
    );
    let json = backup::commands::report::render_restore_drill_evidence_json(&evidence).unwrap();
    assert_ne!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap()["recovery_results"]["database_verification"],
        serde_json::Value::Null
    );
}

#[test]
fn restore_drill_reports_each_database_signature_failure_for_both_supported_types() {
    let failure_cases = [
        ("missing dump", None, ""),
        ("empty dump", Some("database.sql"), ""),
        ("wrong extension", Some("database.txt"), "valid dump"),
        (
            "invalid content",
            Some("database.sql"),
            "not a database dump",
        ),
    ];

    for database_type in [DatabaseType::Mysql, DatabaseType::Postgres] {
        let mismatched_dump = match database_type {
            DatabaseType::Mysql => "-- PostgreSQL database dump\nCREATE TABLE items (id int);",
            DatabaseType::Postgres => "-- MySQL dump 10.11\nCREATE TABLE items (id int);",
        };
        let cases = failure_cases.into_iter().chain(std::iter::once((
            "database type mismatch",
            Some("database.sql"),
            mismatched_dump,
        )));

        for (name, filename, content) in cases {
            let temp = tempdir().unwrap();
            let mut report_config = config(temp.path());
            report_config.profile = "database".into();
            report_config.database_type = Some(database_type);
            report_config.restore_drill_profiles = vec![RestoreDrillProfileConfig {
                profile: "database".into(),
                database_type: Some(database_type),
                primary: Some(RestoreDrillStorageConfig {
                    backend: "primary".into(),
                    repository: "/primary".into(),
                    password: SecretString::new("primary-secret".into()),
                    environment: Vec::new(),
                }),
                primary_diagnostic: None,
                secondary: None,
                secondary_diagnostic: None,
            }];
            let runner = DatabaseProfileRunner::with_restore_output(
                vec![SnapshotInfo {
                    id: "database-snapshot-001".into(),
                    timestamp: "2026-08-07T09:00:00Z".into(),
                    tags: vec!["backup-profile:database".into()],
                }],
                filename,
                content,
            );
            let clock = FixedClock::new([
                timestamp("2026-08-07T10:00:00+09:00", 10_000),
                timestamp("2026-08-07T10:00:01+09:00", 11_000),
                timestamp("2026-08-07T10:00:04+09:00", 14_000),
                timestamp("2026-08-07T10:00:05+09:00", 15_000),
            ]);

            let evidence =
                execute_restore_drill_with_runner_and_clock(&report_config, &runner, &clock)
                    .unwrap();
            let result = &evidence.storage_results[0];
            let verification = result.database_verification.as_ref().unwrap();
            assert_eq!(
                evidence.overall_status,
                RestoreDrillStatus::Fail,
                "case={name}, type={database_type}"
            );
            assert_eq!(result.status, RestoreDrillStatus::Fail, "case={name}");
            assert_eq!(result.validation_status, RestoreDrillStatus::Fail);
            assert_eq!(verification.db_type, database_type);
            assert!(!verification.signature_verified, "case={name}");
            assert_eq!(
                verification.signature_status,
                RestoreDrillStatus::Fail,
                "case={name}"
            );
            assert!(!verification.import_performed);
            assert!(!verification.record_validation_performed);
        }
    }
}

#[test]
fn restore_drill_selects_concrete_snapshot_and_records_measured_primary_evidence() {
    let temp = tempdir().unwrap();
    let config = config(temp.path());
    let runner = StrictRestoreRunner::new(vec![SnapshotInfo {
        id: "full-snapshot-001".into(),
        timestamp: "2026-08-07T09:00:00Z".into(),
        tags: vec!["backup-profile:daily-files".into()],
    }]);
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00+09:00", 10_000),
        timestamp("2026-08-07T10:00:01+09:00", 11_000),
        timestamp("2026-08-07T10:00:04+09:00", 14_421),
        timestamp("2026-08-07T10:00:05+09:00", 15_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    assert_eq!(runner.calls(), ["list-snapshots", "restore"]);
    assert_eq!(evidence.overall_status, RestoreDrillStatus::Pass);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&config.restore_drill_work_dir)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    let result = &evidence.storage_results[0];
    assert_eq!(result.snapshot_id.as_deref(), Some("full-snapshot-001"));
    assert_eq!(
        result.snapshot_time.as_deref(),
        Some("2026-08-07T09:00:00Z")
    );
    assert_eq!(result.elapsed_milliseconds, Some(3_421));
    assert_eq!(result.elapsed_seconds, Some(3));
    assert_eq!(result.file_count, Some(1));
    assert_eq!(result.total_bytes, Some(8));
    assert_eq!(result.rto_satisfied, Some(true));

    let json_file = temp.path().join("restore-drill.json");
    let html_file = temp.path().join("restore-drill.html");
    execute_restore_drill_evidence_export(
        &evidence,
        Some(&json_file),
        Some(backup::commands::report::ReportFormat::Json),
        temp.path(),
    )
    .unwrap();
    execute_restore_drill_evidence_export(
        &evidence,
        Some(&html_file),
        Some(backup::commands::report::ReportFormat::Html),
        temp.path(),
    )
    .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(json_file).unwrap()).unwrap();
    let html = fs::read_to_string(html_file).unwrap();
    assert_eq!(json["execution_id"], evidence.execution_id);
    assert_eq!(json["target_snapshot_id"], "full-snapshot-001");
    assert_eq!(json["recovery_results"]["elapsed_milliseconds"], 3_421);
    assert_eq!(json["recovery_results"]["data_integrity_verified"], true);
    assert!(html.contains("full-snapshot-001"));
    assert!(html.contains("3421 ms"));
}

#[test]
fn restore_drill_missing_tag_is_not_performed_and_never_restores_latest() {
    let temp = tempdir().unwrap();
    let config = config(temp.path());
    let runner = StrictRestoreRunner::new(vec![SnapshotInfo {
        id: "legacy-snapshot".into(),
        timestamp: "2026-08-07T09:00:00Z".into(),
        tags: Vec::new(),
    }]);
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00+09:00", 10_000),
        timestamp("2026-08-07T10:00:01+09:00", 11_000),
        timestamp("2026-08-07T10:00:02+09:00", 12_000),
        timestamp("2026-08-07T10:00:03+09:00", 13_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    assert_eq!(evidence.overall_status, RestoreDrillStatus::NotPerformed);
    assert_eq!(runner.calls(), ["list-snapshots"]);
    assert!(evidence.storage_results[0].snapshot_id.is_none());
}

#[test]
fn restore_drill_malformed_snapshot_json_is_not_performed() {
    let temp = tempdir().unwrap();
    let config = config(temp.path());
    let runner = StrictRestoreRunner::malformed_snapshots();
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00Z", 10_000),
        timestamp("2026-08-07T10:00:01Z", 11_000),
        timestamp("2026-08-07T10:00:02Z", 12_000),
        timestamp("2026-08-07T10:00:03Z", 13_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    assert_eq!(evidence.overall_status, RestoreDrillStatus::NotPerformed);
    assert_eq!(
        evidence.storage_results[0].status,
        RestoreDrillStatus::NotPerformed
    );
    assert_eq!(
        evidence.storage_results[0].failure_stage.as_deref(),
        Some("snapshot-selection")
    );
    assert_eq!(runner.calls(), ["list-snapshots"]);
}

#[test]
fn restore_drill_rejects_a_non_directory_workspace_before_adapter_calls() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("restore-work-file");
    fs::write(&workspace, "not a directory").unwrap();
    let mut config = config(temp.path());
    config.restore_drill_work_dir = workspace;
    let runner = StrictRestoreRunner::new(vec![SnapshotInfo {
        id: "full-snapshot-001".into(),
        timestamp: "2026-08-07T09:00:00Z".into(),
        tags: vec!["backup-profile:daily-files".into()],
    }]);
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00Z", 10_000),
        timestamp("2026-08-07T10:00:01Z", 11_000),
        timestamp("2026-08-07T10:00:02Z", 12_000),
        timestamp("2026-08-07T10:00:03Z", 13_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    let result = &evidence.storage_results[0];
    assert_eq!(result.status, RestoreDrillStatus::Fail);
    assert_eq!(result.failure_stage.as_deref(), Some("workspace"));
    assert!(result.diagnostic.as_deref().unwrap().contains("directory"));
    assert!(runner.calls().is_empty());
}

#[cfg(unix)]
#[test]
fn restore_drill_rejects_a_symlinked_workspace_before_adapter_calls() {
    let temp = tempdir().unwrap();
    let real_workspace = temp.path().join("real-restore-work");
    let workspace = temp.path().join("restore-work-symlink");
    fs::create_dir(&real_workspace).unwrap();
    std::os::unix::fs::symlink(&real_workspace, &workspace).unwrap();
    let mut config = config(temp.path());
    config.restore_drill_work_dir = workspace;
    let runner = StrictRestoreRunner::new(vec![SnapshotInfo {
        id: "full-snapshot-001".into(),
        timestamp: "2026-08-07T09:00:00Z".into(),
        tags: vec!["backup-profile:daily-files".into()],
    }]);
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00Z", 10_000),
        timestamp("2026-08-07T10:00:01Z", 11_000),
        timestamp("2026-08-07T10:00:02Z", 12_000),
        timestamp("2026-08-07T10:00:03Z", 13_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    let result = &evidence.storage_results[0];
    assert_eq!(result.status, RestoreDrillStatus::Fail);
    assert_eq!(result.failure_stage.as_deref(), Some("workspace"));
    assert!(result.diagnostic.as_deref().unwrap().contains("symlink"));
    assert!(runner.calls().is_empty());
}

#[cfg(unix)]
#[test]
fn restore_drill_hardens_an_existing_workspace_before_restore() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let workspace = temp.path().join("restore-work-insecure");
    fs::create_dir(&workspace).unwrap();
    fs::set_permissions(&workspace, fs::Permissions::from_mode(0o755)).unwrap();
    let mut config = config(temp.path());
    config.restore_drill_work_dir = workspace.clone();
    let runner = StrictRestoreRunner::new(vec![SnapshotInfo {
        id: "full-snapshot-001".into(),
        timestamp: "2026-08-07T09:00:00Z".into(),
        tags: vec!["backup-profile:daily-files".into()],
    }]);
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00Z", 10_000),
        timestamp("2026-08-07T10:00:01Z", 11_000),
        timestamp("2026-08-07T10:00:04Z", 14_000),
        timestamp("2026-08-07T10:00:05Z", 15_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    assert_eq!(evidence.overall_status, RestoreDrillStatus::Pass);
    assert_eq!(
        fs::metadata(workspace).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn restore_drill_timeout_failure_records_stage_elapsed_time_and_cleans_target() {
    let temp = tempdir().unwrap();
    let config = config(temp.path());
    let runner = StrictRestoreRunner::failing_restore(
        vec![SnapshotInfo {
            id: "full-snapshot-timeout".into(),
            timestamp: "2026-08-07T09:00:00Z".into(),
            tags: vec!["backup-profile:daily-files".into()],
        }],
        "Process execution timed out after 240 minutes; process group terminated",
    );
    let clock = FixedClock::new([
        timestamp("2026-08-07T10:00:00Z", 1_000),
        timestamp("2026-08-07T10:00:01Z", 2_000),
        timestamp("2026-08-07T10:04:01Z", 242_000),
        timestamp("2026-08-07T10:04:02Z", 243_000),
    ]);

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();
    let result = &evidence.storage_results[0];
    assert_eq!(evidence.overall_status, RestoreDrillStatus::Fail);
    assert_eq!(result.failure_stage.as_deref(), Some("restore"));
    assert!(result.timed_out);
    assert_eq!(result.elapsed_milliseconds, Some(240_000));
    assert!(
        result
            .diagnostic
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("interrupted_stage=restore"))
    );
    assert_eq!(
        fs::read_dir(config.restore_drill_work_dir).unwrap().count(),
        0
    );
}

#[test]
fn restore_drill_export_preserves_first_artifact_when_second_format_fails() {
    let temp = tempdir().unwrap();
    let base = temp.path().join("restore-drill-report");
    fs::create_dir(base.with_extension("json")).unwrap();
    let evidence = RestoreDrillEvidence::new(
        "restore-drill-export-001",
        "2026-08-07T10:00:00Z",
        "2026-08-07T10:00:01Z",
        RestoreDrillPolicy::default(),
        vec![
            backup::commands::report::RestoreDrillStorageResult::not_performed(
                "daily-files",
                "primary",
                "tagged snapshot is unavailable",
            ),
        ],
    );

    let error = execute_restore_drill_evidence_export(&evidence, Some(&base), None, temp.path())
        .expect_err("the JSON directory should make the second format fail");

    let html = base.with_extension("html");
    assert!(html.exists(), "the successful HTML artifact must be kept");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(temp.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&html).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert!(
        error.to_string().contains(html.to_string_lossy().as_ref()),
        "the partial artifact must be reported alongside the export failure: {error}"
    );
}

#[cfg(unix)]
#[test]
fn restore_drill_export_rejects_a_symlinked_report_parent() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let real_output = temp.path().join("real-output");
    fs::create_dir(&real_output).unwrap();
    let symlinked_output = temp.path().join("symlinked-output");
    symlink(&real_output, &symlinked_output).unwrap();
    let evidence = RestoreDrillEvidence::new(
        "restore-drill-export-002",
        "2026-08-07T10:00:00Z",
        "2026-08-07T10:00:01Z",
        RestoreDrillPolicy::default(),
        vec![
            backup::commands::report::RestoreDrillStorageResult::not_performed(
                "daily-files",
                "primary",
                "tagged snapshot is unavailable",
            ),
        ],
    );

    let error = execute_restore_drill_evidence_export(
        &evidence,
        None,
        Some(backup::commands::report::ReportFormat::Json),
        &symlinked_output,
    )
    .expect_err("a symlinked report parent must be rejected");

    assert!(
        error.to_string().contains("symlink"),
        "expected a symlink diagnostic, got: {error}"
    );
    assert_eq!(fs::read_dir(real_output).unwrap().count(), 0);
}

struct MultiStorageRunner {
    calls: Mutex<Vec<String>>,
    fail_snapshot: Option<String>,
}

impl MultiStorageRunner {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail_snapshot: None,
        }
    }

    fn failing_restore(snapshot: &str) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail_snapshot: Some(snapshot.into()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn unsupported(&self, operation: &str) -> Result<String> {
        anyhow::bail!("unexpected restore drill operation: {operation}")
    }
}

impl ResticRunner for MultiStorageRunner {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        self.unsupported("init")
    }

    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        self.unsupported("backup")
    }

    fn list_snapshots(&self, repository: &str, password: &str) -> Result<String> {
        self.assert_storage_credentials(repository, password);
        self.list_snapshot_infos(repository, password)
            .map(|_| String::new())
    }

    fn list_snapshot_infos(&self, repository: &str, password: &str) -> Result<Vec<SnapshotInfo>> {
        self.assert_storage_credentials(repository, password);
        self.list_snapshot_infos_for(repository)
    }

    fn list_snapshot_infos_with_env(
        &self,
        repository: &str,
        password: &str,
        environment: &[(&str, &str)],
    ) -> Result<Vec<SnapshotInfo>> {
        self.assert_storage_credentials(repository, password);
        assert!(environment.is_empty(), "unexpected storage environment");
        self.list_snapshot_infos_for(repository)
    }

    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        self.unsupported("restore without environment")
    }

    fn restore_with_env_and_timeout(
        &self,
        repository: &str,
        password: &str,
        snapshot: &str,
        target: &str,
        environment: &[(&str, &str)],
        timeout: std::time::Duration,
    ) -> Result<String> {
        self.assert_storage_credentials(repository, password);
        assert!(environment.is_empty(), "unexpected storage environment");
        assert_eq!(timeout, std::time::Duration::from_secs(14_400));
        self.calls
            .lock()
            .unwrap()
            .push(format!("restore:{repository}:{snapshot}"));
        if self.fail_snapshot.as_deref() == Some(snapshot) {
            anyhow::bail!("restore failed for {snapshot}");
        }
        fs::write(Path::new(target).join("restored.txt"), "restored")?;
        Ok("restored".into())
    }

    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        self.unsupported("backup_command")
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
        self.unsupported("backup_command_with_env")
    }
}

impl MultiStorageRunner {
    fn list_snapshot_infos_for(&self, repository: &str) -> Result<Vec<SnapshotInfo>> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("list:{repository}"));
        let snapshots = [
            ("alpha", "alpha-snapshot"),
            ("solo", "solo-snapshot"),
            ("zeta", "zeta-snapshot"),
        ];
        Ok(snapshots
            .into_iter()
            .map(|(profile, id)| SnapshotInfo {
                id: format!("{repository}-{id}"),
                timestamp: "2026-08-07T09:00:00Z".into(),
                tags: vec![format!("backup-profile:{profile}")],
            })
            .collect())
    }

    fn assert_storage_credentials(&self, repository: &str, password: &str) {
        let expected = match repository {
            "/primary" => "primary-secret",
            "/secondary" => "secondary-secret",
            other => panic!("unexpected repository {other}"),
        };
        assert_eq!(password, expected);
    }
}

#[test]
fn restore_drill_verifies_sorted_profiles_on_primary_then_secondary_independently() {
    let temp = tempdir().unwrap();
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        "version: '2'\nprofiles:\n  secondary:\n    repository: /secondary\n    password-file: secondary-password\n  zeta:\n    inherit: primary\n    backup:\n      source: [/zeta]\n    copy:\n      profile: secondary\n  primary:\n    repository: /primary\n    password-file: primary-password\n  alpha:\n    inherit: primary\n    backup:\n      source: [/alpha]\n    copy:\n      profile: secondary\n  solo:\n    inherit: primary\n    backup:\n      source: [/solo]\n  default: {}\n",
    )
    .unwrap();
    for (name, value) in [
        ("primary-password", "primary-secret"),
        ("secondary-password", "secondary-secret"),
    ] {
        let path = temp.path().join(name);
        fs::write(&path, value).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let profiles = ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let mut config = ReportConfig::from_profiles(&profiles, &profiles_path).unwrap();
    config.restore_drill_work_dir = temp.path().join("restore-drill");
    let runner = MultiStorageRunner::new();
    let clock = FixedClock::new(
        (0..12).map(|value| timestamp(&format!("2026-08-07T10:00:{value:02}Z"), value * 100)),
    );

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();

    assert_eq!(
        runner.calls(),
        [
            "list:/primary",
            "restore:/primary:/primary-alpha-snapshot",
            "list:/secondary",
            "restore:/secondary:/secondary-alpha-snapshot",
            "list:/primary",
            "restore:/primary:/primary-solo-snapshot",
            "list:/primary",
            "restore:/primary:/primary-zeta-snapshot",
            "list:/secondary",
            "restore:/secondary:/secondary-zeta-snapshot",
        ]
    );
    assert_eq!(evidence.overall_status, RestoreDrillStatus::Pass);
    assert_eq!(
        evidence
            .storage_results
            .iter()
            .map(|result| (result.profile.as_str(), result.backend.as_str()))
            .collect::<Vec<_>>(),
        [
            ("alpha", "primary"),
            ("alpha", "secondary"),
            ("solo", "primary"),
            ("solo", "secondary"),
            ("zeta", "primary"),
            ("zeta", "secondary")
        ]
    );
    for index in [0, 1, 2, 4, 5] {
        assert!(evidence.storage_results[index].snapshot_id.is_some());
    }
    assert_eq!(
        evidence.storage_results[3].status,
        RestoreDrillStatus::NotApplicable
    );

    let failing_runner = MultiStorageRunner::failing_restore("/secondary-alpha-snapshot");
    let failure_clock = FixedClock::new(
        (0..12).map(|value| timestamp(&format!("2026-08-07T10:01:{value:02}Z"), value * 100)),
    );
    let failure =
        execute_restore_drill_with_runner_and_clock(&config, &failing_runner, &failure_clock)
            .unwrap();
    assert_eq!(failure.overall_status, RestoreDrillStatus::Fail);
    assert_eq!(failing_runner.calls().len(), 10);
    assert_eq!(failure.storage_results[1].status, RestoreDrillStatus::Fail);
    assert_eq!(failure.storage_results[2].status, RestoreDrillStatus::Pass);
    assert_eq!(
        failure.storage_results[3].status,
        RestoreDrillStatus::NotApplicable
    );
    assert_eq!(failure.storage_results[4].status, RestoreDrillStatus::Pass);
    assert_eq!(failure.storage_results[5].status, RestoreDrillStatus::Pass);
}

#[test]
fn restore_drill_keeps_primary_evidence_when_secondary_configuration_is_inactive() {
    let temp = tempdir().unwrap();
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: /primary\n    password-file: primary-password\n  secondary: {}\n  alpha:\n    inherit: primary\n    backup:\n      source: [/alpha]\n    copy:\n      profile: secondary\n",
    )
    .unwrap();
    let password = temp.path().join("primary-password");
    fs::write(&password, "primary-secret").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&password, fs::Permissions::from_mode(0o600)).unwrap();
    }

    let profiles = ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let mut config = ReportConfig::from_profiles(&profiles, &profiles_path).unwrap();
    config.restore_drill_work_dir = temp.path().join("restore-drill");
    let runner = MultiStorageRunner::new();
    let clock = FixedClock::new(
        (0..6).map(|value| timestamp(&format!("2026-08-07T10:02:{value:02}Z"), value * 100)),
    );

    let evidence = execute_restore_drill_with_runner_and_clock(&config, &runner, &clock).unwrap();

    assert_eq!(
        runner.calls(),
        ["list:/primary", "restore:/primary:/primary-alpha-snapshot"]
    );
    assert_eq!(evidence.storage_results[0].status, RestoreDrillStatus::Pass);
    assert_eq!(
        evidence.storage_results[1].status,
        RestoreDrillStatus::NotApplicable
    );
    assert!(
        evidence.storage_results[1]
            .diagnostic
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("secondary"))
    );
}
