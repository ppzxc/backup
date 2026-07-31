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
