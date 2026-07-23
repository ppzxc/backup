use assert_cmd::Command;

#[test]
fn test_e2e_cli_lifecycle_full_subcommands() {
    // 1. Version & Help
    Command::cargo_bin("backup").unwrap().arg("--version").assert().success();
    Command::cargo_bin("backup").unwrap().arg("--help").assert().success();

    // 2. Status & Config
    Command::cargo_bin("backup").unwrap().arg("status").assert().success();
    Command::cargo_bin("backup").unwrap().arg("config").arg("show").assert().success();
    Command::cargo_bin("backup").unwrap().arg("config").arg("export").assert().success();

    // 3. Setup
    Command::cargo_bin("backup").unwrap().arg("setup").assert().success();

    // 4. Schedule & Doctor
    Command::cargo_bin("backup").unwrap().arg("schedule").arg("status").assert().success();
    Command::cargo_bin("backup").unwrap().arg("doctor").assert().success();

    // 5. Update
    Command::cargo_bin("backup").unwrap().arg("update").assert().success();
}
