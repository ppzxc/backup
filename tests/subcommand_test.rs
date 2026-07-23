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
fn test_setup_subcommands() {
    assert_cmd_args_success(&["setup", "--help"]);
    assert_cmd_args_success(&["setup", "dependencies", "--help"]);
    assert_cmd_args_success(&["setup", "backend-init", "--help"]);
}

#[test]
fn test_config_subcommands() {
    assert_cmd_args_success(&["config", "show", "--help"]);
    assert_cmd_args_success(&["config", "edit", "--help"]);
    assert_cmd_args_success(&["config", "import-legacy", "--help"]);
}

#[test]
fn test_backend_subcommands() {
    assert_cmd_args_success(&["backend", "migrate", "--help"]);
}

#[test]
fn test_run_flags() {
    assert_cmd_args_success(&["run", "--help"]);
}

#[test]
fn test_doctor_subcommands() {
    assert_cmd_args_success(&["doctor", "environment", "--help"]);
    assert_cmd_args_success(&["doctor", "time-sync", "--help"]);
    assert_cmd_args_success(&["doctor", "restore-drill", "--help"]);
}

#[test]
fn test_schedule_subcommands() {
    assert_cmd_args_success(&["schedule", "enable", "--help"]);
    assert_cmd_args_success(&["schedule", "disable", "--help"]);
    assert_cmd_args_success(&["schedule", "status", "--help"]);
}

#[test]
fn test_uninstall_flags() {
    assert_cmd_args_success(&["uninstall", "--help"]);
}


