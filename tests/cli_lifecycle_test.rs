use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn cli_lifecycle_contract_uses_explicit_temporary_paths() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("config.yml");
    let profiles = temp.path().join("profiles.yml");
    fs::write(
        &config,
        r#"version: "1.0"
profile: "test"
backup:
  backup_type: directory
  targets: ["/tmp"]
  excludes: []
retention: { keep_daily: 1, keep_weekly: 1, keep_monthly: 1 }
storage:
  primary: { backend: "local", repository: "/tmp/repo", password: "test-password" }
"#,
    )
    .unwrap();
    fs::write(
        &profiles,
        "version: '2'\nprofiles:\n  test:\n    repository: /tmp/repo\n",
    )
    .unwrap();

    let run_assert = Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--config",
            config.to_str().unwrap(),
            "--profiles",
            profiles.to_str().unwrap(),
            "run",
            "--dry-run",
            "--skip-database",
        ])
        .assert()
        .success();
    let run_stdout = String::from_utf8(run_assert.get_output().stdout.clone()).unwrap();
    assert!(!run_stdout.contains("Database streaming backup check"));

    Command::cargo_bin("backup")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "version"])
        .assert()
        .success();
}

#[test]
fn setup_non_interactive_writes_config_and_profiles_to_their_explicit_paths() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("environment/config.yml");
    let profiles = temp.path().join("profile-data/profiles.yml");

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--config",
            config.to_str().unwrap(),
            "--profiles",
            profiles.to_str().unwrap(),
            "setup",
            "--non-interactive",
        ])
        .assert()
        .success();

    assert!(config.exists(), "setup must honor --config");
    assert!(profiles.exists(), "setup must honor --profiles");
    assert!(
        !config.parent().unwrap().join("profiles.yaml").exists(),
        "setup must not substitute a config-derived path for --profiles"
    );
}

#[test]
fn copy_propagates_backend_failure() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("config.yml");
    let profiles = temp.path().join("profiles.yml");
    fs::write(&config, "version: '1.0'\nprofile: test\n").unwrap();
    fs::write(&profiles, "not valid: [profiles").unwrap();

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--config",
            config.to_str().unwrap(),
            "--profiles",
            profiles.to_str().unwrap(),
            "copy",
        ])
        .assert()
        .failure();
}

#[test]
fn database_dry_run_accepts_database_stream_configuration() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("database.yml");
    fs::write(
        &config,
        r#"version: '1.0'
profile: database
backup:
  backupType: !dbStream
    db_type: postgres
    connection_url: postgres://postgres:secret@db:5432/app
  targets: []
  excludes: []
retention: {keepDaily: 1, keepWeekly: 1, keepMonthly: 1}
storage:
  primary: {backend: s3, repository: 's3:http://minio:9000/database', password: test-password}
"#,
    )
    .unwrap();
    let parsed = backup::config::model::BackupConfig::load_from_path(&config);
    assert!(parsed.is_ok(), "configuration must parse: {parsed:?}");

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--config",
            config.to_str().unwrap(),
            "database",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("pg_dump -> app.sql"));
}
