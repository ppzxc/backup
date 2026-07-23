use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_e2e_full_workflow_cli() {
    // 1. Verify binary CLI execution (--help and --version)
    let mut help_cmd = Command::cargo_bin("backup").unwrap();
    help_cmd.arg("--help").assert().success();

    let mut ver_cmd = Command::cargo_bin("backup").unwrap();
    ver_cmd.arg("--version").assert().success();

    // 2. Setup workflow: generate config in temp directory
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("backup.yml");

    let mut setup_cmd = Command::cargo_bin("backup").unwrap();
    setup_cmd
        .arg("setup")
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg("--profile")
        .arg("scenario-test")
        .assert()
        .success();

    assert!(config_file.exists());

    // 3. Status command with created config
    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg("status")
        .assert()
        .success();

    // 4. Schedule subcommand systemd generator check
    let mut schedule_cmd = Command::cargo_bin("backup").unwrap();
    schedule_cmd
        .arg("schedule")
        .arg("--show-units")
        .assert()
        .success();

    // 5. Config show & export subcommand check
    let mut config_show_cmd = Command::cargo_bin("backup").unwrap();
    config_show_cmd
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg("config")
        .arg("show")
        .assert()
        .success();

    let mut config_export_cmd = Command::cargo_bin("backup").unwrap();
    config_export_cmd
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg("config")
        .arg("export")
        .arg("--json")
        .assert()
        .success();
}
