use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn unified_profiles_yaml(application: &str, profile: &str) -> String {
    format!(
        "version: '2'\napplication:\n{application}\nprofiles:\n  {profile}:\n    repository: /tmp/repo\n    password: test-password\n    backup: {{source: ['/tmp']}}\n"
    )
}

#[test]
fn cli_lifecycle_contract_uses_one_explicit_profiles_path() {
    let temp = TempDir::new().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    fs::write(
        &profiles,
        unified_profiles_yaml(
            &format!(
                "  reports: {{outputDir: '{}', enableDailyReports: true, enableAnnualDrDrillReport: false}}",
                temp.path().join("reports").display()
            ),
            "test",
        ),
    )
    .unwrap();

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            profiles.to_str().unwrap(),
            "run",
            "--dry-run",
            "--skip-database",
        ])
        .assert()
        .success();

    Command::cargo_bin("backup")
        .unwrap()
        .arg("version")
        .assert()
        .success();
}

#[test]
fn setup_non_interactive_rejects_missing_profiles_file() {
    let temp = TempDir::new().unwrap();
    let profiles = temp.path().join("profile-data/profiles.yaml");

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            profiles.to_str().unwrap(),
            "setup",
            "--non-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "requires an existing unified profiles.yaml",
        ));

    assert!(!profiles.exists());
}

#[test]
fn copy_propagates_backend_failure() {
    let temp = TempDir::new().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    fs::write(&profiles, "not valid: [profiles").unwrap();

    Command::cargo_bin("backup")
        .unwrap()
        .args(["--profiles", profiles.to_str().unwrap(), "copy"])
        .assert()
        .failure();
}

#[test]
fn database_dry_run_accepts_unified_database_configuration() {
    let temp = TempDir::new().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    fs::write(
        &profiles,
        unified_profiles_yaml(
            "  database:\n    profile: database\n    type: postgres\n    connection-url: postgres://postgres:secret@db:5432/app",
            "database",
        ),
    )
    .unwrap();

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            profiles.to_str().unwrap(),
            "database",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("pg_dump -> app.sql"));
}
