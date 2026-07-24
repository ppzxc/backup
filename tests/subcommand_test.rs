use assert_cmd::Command;

#[test]
fn test_subcommands_help() {
    // LANG=ko_KR.UTF-8 환경에서 한국어 도움말이 출력되는지 검증합니다.
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd
        .env("LANG", "ko_KR.UTF-8")
        .env_remove("LC_ALL")
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("마법사"), "Missing Korean setup docstring in help output");
    assert!(stdout.contains("동기화"), "Missing Korean copy docstring in help output");
    assert!(stdout.contains("파이프라인"), "Missing Korean run docstring in help output");
    assert!(stdout.contains("ISMS-P 진단"), "Missing Korean doctor docstring in help output");
}

#[test]
fn test_setup_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["setup", "dependencies"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("dependencies"), "Expected dependency verification output");
}

#[test]
fn test_copy_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["copy", "--dry-run"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("copy"), "Expected copy command output");
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



