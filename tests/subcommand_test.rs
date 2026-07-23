use assert_cmd::Command;

fn assert_cmd_args_success(args: &[&str]) {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(args).assert().success();
}

#[test]
fn test_subcommands_help() {
    assert_cmd_args_success(&["--help"]);
}

#[test]
fn test_setup_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["setup", "dependencies"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("dependencies"), "Expected dependency verification output");
}

#[test]
fn test_config_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["config", "show"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("version") || stdout.contains("profile"), "Expected config show output");
}

#[test]
fn test_backend_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["backend", "migrate"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("migration"), "Expected backend migration output");
}

#[test]
fn test_doctor_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["doctor"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Checking dependencies"), "Expected doctor check output");
}

#[test]
fn test_schedule_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["schedule", "status"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Schedule status") || stdout.contains("Active"), "Expected schedule status output");
}

#[test]
fn test_uninstall_flags_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["uninstall", "--yes"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Uninstalled"), "Expected uninstall output");
}



