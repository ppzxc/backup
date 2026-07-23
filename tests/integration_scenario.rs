use assert_cmd::Command;

#[test]
fn test_e2e_full_workflow_cli() {
    // 1. Verify binary CLI help and version
    let mut help_cmd = Command::cargo_bin("backup").unwrap();
    help_cmd.arg("--help").assert().success();

    let mut ver_cmd = Command::cargo_bin("backup").unwrap();
    ver_cmd.arg("--version").assert().success();

    // 2. Status subcommand
    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd.arg("status").assert().success();

    // 3. Setup subcommand
    let mut setup_cmd = Command::cargo_bin("backup").unwrap();
    setup_cmd.arg("setup").assert().success();

    // 4. Schedule subcommand
    let mut schedule_cmd = Command::cargo_bin("backup").unwrap();
    schedule_cmd.arg("schedule").arg("status").assert().success();

    // 5. Config subcommands
    let mut config_show_cmd = Command::cargo_bin("backup").unwrap();
    config_show_cmd.arg("config").arg("show").assert().success();

    let mut config_export_cmd = Command::cargo_bin("backup").unwrap();
    config_export_cmd
        .arg("config")
        .arg("export")
        .assert()
        .success();

    // 6. Doctor subcommand
    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();

    // 7. Update subcommand
    let mut update_cmd = Command::cargo_bin("backup").unwrap();
    update_cmd.arg("update").assert().success();
}
