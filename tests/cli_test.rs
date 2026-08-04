use assert_cmd::Command;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.arg("--version").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "--version 출력에 CARGO_PKG_VERSION이 포함되어야 합니다"
    );
}

#[test]
fn test_cli_version_subcommand() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd.arg("version").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "backup version 출력에 CARGO_PKG_VERSION이 포함되어야 합니다"
    );
}

/// LANG=ko_KR.UTF-8 환경에서 --help 출력이 한국어만 포함하는지 검증
#[test]
fn test_help_korean_when_lang_ko() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.env("LANG", "ko_KR.UTF-8")
        .env_remove("LC_ALL")
        .arg("--help");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(
        stdout.contains("마법사"),
        "LANG=ko 환경에서 한국어 setup 설명이 있어야 합니다"
    );
    assert!(
        stdout.contains("파이프라인"),
        "LANG=ko 환경에서 한국어 run 설명이 있어야 합니다"
    );

    // 한국어 모드에서 영어 전용 텍스트가 섞이지 않는지 확인
    assert!(
        !stdout.contains("wizard"),
        "LANG=ko 환경에서 'wizard'가 노출되면 안 됩니다"
    );
}

/// LANG=en_US.UTF-8 환경에서 --help 출력이 영어만 포함하는지 검증
#[test]
fn test_help_english_when_lang_en() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .arg("--help");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(
        stdout.contains("wizard"),
        "LANG=en 환경에서 영어 setup 설명이 있어야 합니다"
    );
    assert!(
        stdout.contains("pipeline"),
        "LANG=en 환경에서 영어 run 설명이 있어야 합니다"
    );

    // 영어 모드에서 한국어 전용 텍스트가 섞이지 않는지 확인
    assert!(
        !stdout.contains("마법사"),
        "LANG=en 환경에서 '마법사'가 노출되면 안 됩니다"
    );
}

/// LANG=ko_KR.UTF-8 환경에서 setup --help 출력이 한국어인지 검증
#[test]
fn test_setup_subcommand_help_korean() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.env("LANG", "ko_KR.UTF-8")
        .env_remove("LC_ALL")
        .args(["setup", "--help"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("마법사"),
        "setup --help 한국어 도움말이 있어야 합니다"
    );
    assert!(
        !stdout.contains("wizard"),
        "setup --help에서 영어 'wizard'가 노출되면 안 됩니다"
    );
}

/// LANG=en_US.UTF-8 환경에서 setup --help 출력이 영어인지 검증
#[test]
fn test_setup_subcommand_help_english() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .args(["setup", "--help"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("wizard"),
        "setup --help English help text must be present"
    );
    assert!(
        !stdout.contains("마법사"),
        "setup --help must not show Korean text in en mode"
    );
}

#[test]
fn explicit_setup_language_overrides_environment_for_help() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let assert = cmd
        .env("LANG", "ko_KR.UTF-8")
        .env_remove("LC_ALL")
        .args(["setup", "--lang", "en", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("wizard"));
    assert!(!stdout.contains("마법사"));
}

#[test]
fn invalid_setup_language_fails_before_help_or_dispatch() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    cmd.args(["setup", "--lang", "fr", "--help"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("invalid language"));
}

#[test]
fn test_subcommands_not_placeholder() {
    let subcommands = vec![
        vec!["setup", "--help"],
        vec!["run", "--help"],
        vec!["schedule", "--help"],
        vec!["restore", "--help"],
        vec!["snapshots", "--help"],
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

    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    std::fs::write(
        &profiles,
        "version: '2'\nprofiles:\n  default:\n    repository: /tmp/repo\n    backup:\n      source: ['/tmp']\n",
    )
    .unwrap();
    let assert = Command::cargo_bin("backup")
        .unwrap()
        .args(["--profiles", profiles.to_str().unwrap(), "status"])
        .assert()
        .success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!output.contains("Command executed"));
}

#[test]
fn test_cli_logging_flags() {
    let mut cmd = Command::cargo_bin("backup").unwrap();
    let temp_log = tempfile::NamedTempFile::new().unwrap();
    let log_path = temp_log.path().to_str().unwrap();

    let _assert = cmd
        .args(["-v", "-q", "--log-file", log_path, "version"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains(
            "--quiet cannot be combined with --verbose",
        ));
}
