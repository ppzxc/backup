use assert_cmd::Command;
use backup::cli::Cli;
use clap::Parser;

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

#[test]
fn builtin_and_subcommand_version_are_the_same_canonical_output() {
    let builtin = Command::cargo_bin("backup")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    let subcommand = Command::cargo_bin("backup")
        .unwrap()
        .arg("version")
        .output()
        .unwrap();

    assert!(builtin.status.success());
    assert!(subcommand.status.success());
    assert_eq!(builtin.stdout, subcommand.stdout);
    assert!(builtin.stderr.is_empty());
    assert!(subcommand.stderr.is_empty());
}

#[test]
fn version_ignores_an_invalid_profiles_override_but_honors_explicit_logging() {
    let temp = tempfile::tempdir().unwrap();
    let missing_profiles = temp.path().join("missing/profiles.yaml");
    let log_file = temp.path().join("version.log");

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            missing_profiles.to_str().unwrap(),
            "--log-file",
            log_file.to_str().unwrap(),
            "version",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));

    assert!(
        log_file.exists(),
        "an explicit log target is a deliberate side effect"
    );

    let builtin_log_file = temp.path().join("builtin-version.log");
    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            missing_profiles.to_str().unwrap(),
            "--log-file",
            builtin_log_file.to_str().unwrap(),
            "--version",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));

    let invalid_log_target = temp.path().join("invalid-log-target");
    std::fs::create_dir(&invalid_log_target).unwrap();
    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            missing_profiles.to_str().unwrap(),
            "--log-file",
            invalid_log_target.to_str().unwrap(),
            "--version",
        ])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("logging initialization failed"));
}

#[test]
fn built_in_help_is_parser_only_even_with_invalid_runtime_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let missing_profiles = temp.path().join("missing/profiles.yaml");
    let log_file = temp.path().join("help.log");

    Command::cargo_bin("backup")
        .unwrap()
        .args([
            "--profiles",
            missing_profiles.to_str().unwrap(),
            "--log-file",
            log_file.to_str().unwrap(),
            "run",
            "--help",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Execute backup pipeline"));

    assert!(!log_file.exists());

    for (index, command) in [
        vec!["-h"],
        vec!["setup", "-h"],
        vec!["report", "environment", "-h"],
        vec!["schedule", "status", "-h"],
        vec!["restore", "-h"],
        vec!["uninstall", "-h"],
    ]
    .into_iter()
    .enumerate()
    {
        let command_log = temp.path().join(format!("help-{index}.log"));
        let mut args = vec![
            "--profiles".to_string(),
            missing_profiles.to_str().unwrap().to_string(),
            "--log-file".to_string(),
            command_log.to_str().unwrap().to_string(),
        ];
        args.extend(command.into_iter().map(String::from));
        Command::cargo_bin("backup")
            .unwrap()
            .args(&args)
            .assert()
            .success();
        assert!(!command_log.exists());
    }
}

#[test]
fn quiet_and_verbose_are_rejected_by_the_authoritative_parser() {
    let error = match Cli::try_parse_from(["backup", "--quiet", "-v", "version"]) {
        Ok(_) => panic!("quiet and verbose must be mutually exclusive"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn explicit_log_file_failure_is_reported_before_command_dispatch() {
    let temp = tempfile::tempdir().unwrap();
    let log_directory = temp.path().join("log-directory");
    std::fs::create_dir(&log_directory).unwrap();

    Command::cargo_bin("backup")
        .unwrap()
        .args(["--log-file", log_directory.to_str().unwrap(), "version"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("logging initialization failed"));
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
fn setup_language_values_are_case_sensitive_contract_values() {
    Command::cargo_bin("backup")
        .unwrap()
        .args(["setup", "--lang", "EN", "--help"])
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
        .failure();
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
        .stderr(predicates::str::contains("cannot be used with"));
}
