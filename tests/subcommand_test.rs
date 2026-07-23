use assert_cmd::Command;

#[test]
fn test_subcommands_help() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_setup_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["setup", "--help"]).assert().success();

    let mut cmd_deps = Command::cargo_bin("backup").unwrap();
    cmd_deps.args(["setup", "dependencies", "--help"]).assert().success();

    let mut cmd_init = Command::cargo_bin("backup").unwrap();
    cmd_init.args(["setup", "backend-init", "--help"]).assert().success();
}

#[test]
fn test_config_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["config", "show", "--help"]).assert().success();

    let mut cmd_edit = Command::cargo_bin("backup").unwrap();
    cmd_edit.args(["config", "edit", "--help"]).assert().success();

    let mut cmd_import = Command::cargo_bin("backup").unwrap();
    cmd_import.args(["config", "import-legacy", "--help"]).assert().success();
}

#[test]
fn test_backend_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["backend", "migrate", "--help"]).assert().success();
}

#[test]
fn test_run_flags() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["run", "--help"]).assert().success();
}

#[test]
fn test_doctor_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["doctor", "environment", "--help"]).assert().success();

    let mut cmd_ts = Command::cargo_bin("backup").unwrap();
    cmd_ts.args(["doctor", "time-sync", "--help"]).assert().success();

    let mut cmd_rd = Command::cargo_bin("backup").unwrap();
    cmd_rd.args(["doctor", "restore-drill", "--help"]).assert().success();
}

#[test]
fn test_schedule_subcommands() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["schedule", "enable", "--help"]).assert().success();

    let mut cmd_dis = Command::cargo_bin("backup").unwrap();
    cmd_dis.args(["schedule", "disable", "--help"]).assert().success();

    let mut cmd_st = Command::cargo_bin("backup").unwrap();
    cmd_st.args(["schedule", "status", "--help"]).assert().success();
}

#[test]
fn test_uninstall_flags() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["uninstall", "--help"]).assert().success();
}

