use assert_cmd::Command;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn test_subcommands_not_placeholder() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.arg("doctor").assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!output.contains("Command executed"), "Doctor command output placeholder 'Command executed'");
}

