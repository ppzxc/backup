use assert_cmd::Command;

#[test]
fn test_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("status").assert().success();
}
