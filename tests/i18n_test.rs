use backup::i18n::{CliHelp, I18nMessages, Language};

#[test]
fn test_language_parse_and_detection() {
    assert_eq!(Language::from_str("ko"), Language::Ko);
    assert_eq!(Language::from_str("en"), Language::En);
    assert_eq!(Language::from_str("invalid"), Language::En);
}

#[test]
fn test_i18n_messages_lookup() {
    let msg_ko = I18nMessages::get(Language::Ko);
    let msg_en = I18nMessages::get(Language::En);

    assert!(msg_ko.enter_profile_name.contains("프로필"));
    assert!(msg_en.enter_profile_name.contains("Profile"));
}

#[test]
fn test_cli_help_korean_contains_only_korean() {
    let h = CliHelp::get(Language::Ko);
    // 핵심 서브커맨드 설명이 한국어만 포함하는지 검증
    assert!(h.cmd_setup.contains("마법사"), "setup 도움말이 한국어여야 합니다");
    assert!(h.cmd_config.contains("레지스트리"), "config 도움말이 한국어여야 합니다");
    assert!(h.cmd_run.contains("파이프라인"), "run 도움말이 한국어여야 합니다");
    assert!(h.cmd_doctor.contains("진단"), "doctor 도움말이 한국어여야 합니다");
    assert!(h.cmd_schedule.contains("타이머"), "schedule 도움말이 한국어여야 합니다");
    assert!(h.cmd_restore.contains("복구"), "restore 도움말이 한국어여야 합니다");
    assert!(h.cmd_snapshots.contains("스냅샷"), "snapshots 도움말이 한국어여야 합니다");
    assert!(h.cmd_status.contains("운영 상태"), "status 도움말이 한국어여야 합니다");
    assert!(h.cmd_update.contains("업데이트"), "update 도움말이 한국어여야 합니다");
    assert!(h.cmd_uninstall.contains("삭제"), "uninstall 도움말이 한국어여야 합니다");
}

#[test]
fn test_cli_help_english_contains_only_english() {
    let h = CliHelp::get(Language::En);
    // 핵심 서브커맨드 설명이 영어만 포함하는지 검증
    assert!(h.cmd_setup.contains("wizard"), "setup help must be English");
    assert!(h.cmd_config.contains("registry"), "config help must be English");
    assert!(h.cmd_run.contains("pipeline"), "run help must be English");
    assert!(h.cmd_doctor.contains("diagnostics"), "doctor help must be English");
    assert!(h.cmd_schedule.contains("scheduler"), "schedule help must be English");
    assert!(h.cmd_restore.contains("Restore"), "restore help must be English");
    assert!(h.cmd_snapshots.contains("snapshots"), "snapshots help must be English");
    assert!(h.cmd_status.contains("operational"), "status help must be English");
    assert!(h.cmd_update.contains("Self-update"), "update help must be English");
    assert!(h.cmd_uninstall.contains("Uninstall"), "uninstall help must be English");
}

#[test]
fn test_cli_help_no_bilingual_in_korean() {
    let h = CliHelp::get(Language::Ko);
    // 한국어 모드에서 영어 혼용 문자열이 없는지 확인 (about 제외)
    assert!(!h.cmd_setup.contains("wizard"), "한국어 모드에서 'wizard'가 노출되면 안 됩니다");
    assert!(!h.cmd_config.contains("registry"), "한국어 모드에서 'registry'가 노출되면 안 됩니다");
}

#[test]
fn test_cli_help_no_bilingual_in_english() {
    let h = CliHelp::get(Language::En);
    // 영어 모드에서 한국어 혼용 문자열이 없는지 확인
    assert!(!h.cmd_setup.contains("마법사"), "영어 모드에서 '마법사'가 노출되면 안 됩니다");
    assert!(!h.cmd_config.contains("레지스트리"), "영어 모드에서 '레지스트리'가 노출되면 안 됩니다");
}
