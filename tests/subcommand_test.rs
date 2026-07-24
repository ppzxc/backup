use assert_cmd::Command;

#[test]
fn test_subcommands_help() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.arg("--help").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("백업 환경 및 프로필 설정 마법사"), "Missing Korean setup docstring in help output");
    assert!(stdout.contains("백업 설정 레지스트리 관리"), "Missing Korean config docstring in help output");
    assert!(stdout.contains("저장소 백엔드 마이그레이션"), "Missing Korean backend docstring in help output");
    assert!(stdout.contains("백업 파이프라인 수동 실행"), "Missing Korean run docstring in help output");
    assert!(stdout.contains("시스템, 보안 및 ISMS-P 진단 보고서"), "Missing Korean doctor docstring in help output");
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



