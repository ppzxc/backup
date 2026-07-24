use assert_cmd::Command;

#[test]
fn test_e2e_cli_lifecycle_full_subcommands() {
    // 1. Version & Help
    Command::cargo_bin("backup").unwrap().arg("--version").assert().success();
    Command::cargo_bin("backup").unwrap().arg("--help").assert().success();

    // 2. Status
    Command::cargo_bin("backup").unwrap().arg("status").assert().success();

    // 3. Setup
    Command::cargo_bin("backup").unwrap().args(&["setup", "--non-interactive"]).assert().success();

    // 4. Schedule & Doctor
    Command::cargo_bin("backup").unwrap().arg("schedule").arg("status").assert().success();
    Command::cargo_bin("backup").unwrap().arg("doctor").assert().success();

    // 5. Run & Restore flags wiring
    let run_assert = Command::cargo_bin("backup")
        .unwrap()
        .args(&["run", "--dry-run", "--skip-database"])
        .assert()
        .success();
    let run_stdout = String::from_utf8(run_assert.get_output().stdout.clone()).unwrap();
    assert!(!run_stdout.contains("Database streaming backup check"));

    let restore_assert = Command::cargo_bin("backup")
        .unwrap()
        .args(&["restore", "--snapshot", "snap-12345", "--target", "/tmp/restored-data"])
        .assert()
        .success();
    let restore_stdout = String::from_utf8(restore_assert.get_output().stdout.clone()).unwrap();
    assert!(restore_stdout.contains("snap-12345"));
    assert!(restore_stdout.contains("/tmp/restored-data"));

    // 6. Copy (Sync)
    Command::cargo_bin("backup").unwrap().args(&["copy", "--dry-run"]).assert().success();
    Command::cargo_bin("backup").unwrap().args(&["sync", "--dry-run"]).assert().success();
}


