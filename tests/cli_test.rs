use assert_cmd::Command;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn test_subcommands_not_placeholder() {
    let subcommands = vec![
        vec!["setup", "--help"],
        vec!["config", "--help"],
        vec!["run", "--help"],
        vec!["doctor"],
        vec!["schedule", "--help"],
        vec!["restore", "--help"],
        vec!["snapshots", "--help"],
        vec!["status"],
        vec!["update"],
        vec!["uninstall", "--help"],
    ];

    for args in subcommands {
        let mut cmd = Command::cargo_bin("backup").unwrap();
        let assert = cmd.args(&args).assert().success();
        let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        assert!(
            !output.contains("Command executed"),
            "Subcommand {:?} output placeholder 'Command executed'",
            args
        );
    }
}

