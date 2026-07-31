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
    assert!(
        stdout.contains("마법사"),
        "Missing Korean setup docstring in help output"
    );
    assert!(
        stdout.contains("동기화"),
        "Missing Korean copy docstring in help output"
    );
    assert!(
        stdout.contains("파이프라인"),
        "Missing Korean run docstring in help output"
    );
    assert!(
        stdout.contains("진단"),
        "Missing Korean doctor docstring in help output"
    );
}

#[test]
fn test_setup_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["setup", "dependencies"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("dependencies"),
        "Expected dependency verification output"
    );
}

#[test]
fn test_copy_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["copy", "--dry-run"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Snapshot copy") || stdout.contains("Copy completed"),
        "Expected copy command output"
    );
}

#[test]
fn test_doctor_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["doctor"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Checking dependencies"),
        "Expected doctor check output"
    );
}

#[test]
fn test_report_subcommands_output() {
    let lang = backup::i18n::Language::Ko;
    let help = backup::i18n::CliHelp::get(lang);
    assert_eq!(help.cmd_report, "ISMS-P 감사 증적 및 레포트 생성");
    assert_eq!(
        help.cmd_report_environment,
        "백업 환경 디렉터리/파일 권한 및 비밀값 마스킹 검사 보고서 생성"
    );
    assert_eq!(
        help.cmd_report_time_sync,
        "NTP/Chrony 시간 동기화 상태 점검 보고서 생성"
    );
    assert_eq!(
        help.cmd_report_restore_drill,
        "복구 드릴 실행, RTO 측정 및 DB 헤더 무결성 확인 보고서 생성"
    );
}

#[test]
fn test_report_cli_standalone_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_file = temp_dir.path().join("report_out");

    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd
        .args(&[
            "report",
            "environment",
            "--file",
            out_file.to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("ISMS report saved to"));
    assert!(temp_dir.path().join("report_out.html").exists());
    assert!(temp_dir.path().join("report_out.json").exists());
}

#[test]
fn test_report_cli_format_json_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_file = temp_dir.path().join("report_env.json");

    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd
        .args(&[
            "report",
            "environment",
            "--file",
            out_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("ISMS report saved to"));
    assert!(temp_dir.path().join("report_env.json").exists());
    assert!(!temp_dir.path().join("report_env.html").exists());
}

#[test]
fn test_schedule_subcommands_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["schedule", "status"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Schedule status") || stdout.contains("Active"),
        "Expected schedule status output"
    );
}

#[test]
fn test_uninstall_flags_output() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.args(&["uninstall", "--yes"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Uninstalled"), "Expected uninstall output");
}
