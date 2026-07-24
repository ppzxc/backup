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

/// POSIX 표준: LC_ALL이 설정되어 있으면 LANG보다 우선해야 합니다.
/// detect_from()은 순수 함수로 unsafe 환경변수 조작 없이 테스트할 수 있습니다.
#[test]
fn test_language_detect_from_lc_all_overrides_lang() {
    // LANG=ko 이지만 LC_ALL=en 이므로 영어여야 함 (POSIX: LC_ALL wins)
    let lang = Language::detect_from(Some("ko_KR.UTF-8"), Some("en_US.UTF-8"));
    assert_eq!(lang, Language::En, "LC_ALL=en은 LANG=ko보다 우선해야 합니다");
}

#[test]
fn test_language_detect_from_falls_back_to_lang_when_lc_all_absent() {
    // LC_ALL 없으면 LANG 사용
    let lang = Language::detect_from(Some("ko_KR.UTF-8"), None);
    assert_eq!(lang, Language::Ko, "LC_ALL 없을 때 LANG=ko면 Korean이어야 합니다");
}

#[test]
fn test_language_detect_from_lc_all_ko_wins_over_lang_en() {
    // LC_ALL=ko, LANG=en → Korean
    let lang = Language::detect_from(Some("en_US.UTF-8"), Some("ko_KR.UTF-8"));
    assert_eq!(lang, Language::Ko, "LC_ALL=ko이면 LANG=en이어도 Korean이어야 합니다");
}

#[test]
fn test_language_detect_from_empty_lc_all_falls_back_to_lang() {
    // LC_ALL이 빈 문자열이면 LANG으로 fallback
    let lang = Language::detect_from(Some("ko_KR.UTF-8"), Some(""));
    assert_eq!(lang, Language::Ko, "LC_ALL='' 일 때 LANG=ko면 Korean이어야 합니다");
}

#[test]
fn test_language_detect_from_both_absent() {
    // 둘 다 없으면 English (기본값)
    let lang = Language::detect_from(None, None);
    assert_eq!(lang, Language::En, "환경변수 없으면 기본값은 English여야 합니다");
}

#[test]
fn test_cli_help_korean_contains_only_korean() {
    let h = CliHelp::get(Language::Ko);
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

/// 한국어 모드의 모든 필드에 영어 문자열이 섞이지 않는지 검증합니다.
/// 한글 자모 범위(U+AC00~U+D7A3)를 포함하는 필드는 한국어로 간주합니다.
#[test]
fn test_cli_help_no_korean_mode_with_english_keywords_all_fields() {
    let h = CliHelp::get(Language::Ko);
    // 고유명사/약어 제외 목록 (양 언어에서 동일하게 사용)
    let proper_nouns = ["Systemd", "Cron", "NTP", "Chrony", "RTO",
                        "ISMS", "SFTP", "SSH", "URI", "restic", "rclone",
                        "resticprofile", "backup", "yaml", "json", "latest"];
    // 한국어 모드에서 나타나면 혼용을 의미하는 영어 전용 단어들
    let en_keywords = ["wizard", "registry", "pipeline", "diagnostics", "scheduler",
                       "operational", "Self-update", "Uninstall",
                       "masked secrets", "permission validation", "Verify"];
    let fields: &[(&str, &str)] = &[
        ("about",                  h.about),
        ("cmd_setup",              h.cmd_setup),
        ("cmd_config",             h.cmd_config),
        ("cmd_backend",            h.cmd_backend),
        ("cmd_run",                h.cmd_run),
        ("cmd_doctor",             h.cmd_doctor),
        ("cmd_schedule",           h.cmd_schedule),
        ("cmd_restore",            h.cmd_restore),
        ("cmd_snapshots",          h.cmd_snapshots),
        ("cmd_status",             h.cmd_status),
        ("cmd_update",             h.cmd_update),
        ("cmd_uninstall",          h.cmd_uninstall),
        ("cmd_setup_dependencies", h.cmd_setup_dependencies),
        ("cmd_setup_backend_init", h.cmd_setup_backend_init),
        ("opt_setup_lang",         h.opt_setup_lang),
        ("opt_setup_non_interactive", h.opt_setup_non_interactive),
        ("cmd_config_show",        h.cmd_config_show),
        ("cmd_config_edit",        h.cmd_config_edit),
        ("cmd_config_import_legacy", h.cmd_config_import_legacy),
        ("cmd_config_export",      h.cmd_config_export),
        ("opt_config_import_file", h.opt_config_import_file),
        ("opt_config_export_format", h.opt_config_export_format),
        ("cmd_backend_migrate",    h.cmd_backend_migrate),
        ("opt_run_skip_database",  h.opt_run_skip_database),
        ("opt_run_skip_secondary_sync", h.opt_run_skip_secondary_sync),
        ("opt_run_skip_retention", h.opt_run_skip_retention),
        ("opt_run_dry_run",        h.opt_run_dry_run),
        ("cmd_doctor_environment", h.cmd_doctor_environment),
        ("cmd_doctor_time_sync",   h.cmd_doctor_time_sync),
        ("cmd_doctor_restore_drill", h.cmd_doctor_restore_drill),
        ("opt_doctor_file",        h.opt_doctor_file),
        ("cmd_schedule_enable",    h.cmd_schedule_enable),
        ("cmd_schedule_disable",   h.cmd_schedule_disable),
        ("cmd_schedule_status",    h.cmd_schedule_status),
        ("opt_restore_snapshot",   h.opt_restore_snapshot),
        ("opt_restore_target",     h.opt_restore_target),
        ("opt_uninstall_yes",      h.opt_uninstall_yes),
        ("opt_uninstall_purge",    h.opt_uninstall_purge),
    ];
    for (field_name, value) in fields {
        for kw in &en_keywords {
            let is_proper = proper_nouns.iter().any(|n| kw.to_lowercase().starts_with(&n.to_lowercase()));
            if !is_proper && value.to_lowercase().contains(&kw.to_lowercase()) {
                panic!(
                    "[한국어 모드 혼용] 필드 '{}' = {:?} 에서 영어 키워드 '{}' 발견",
                    field_name, value, kw
                );
            }
        }
    }
}

/// 영어 모드의 모든 필드에 한국어 글자(유니코드 가나 범위)가 없는지 검증합니다.
#[test]
fn test_cli_help_no_english_mode_with_korean_chars_all_fields() {
    let h = CliHelp::get(Language::En);
    let fields: &[(&str, &str)] = &[
        ("about",                  h.about),
        ("cmd_setup",              h.cmd_setup),
        ("cmd_config",             h.cmd_config),
        ("cmd_backend",            h.cmd_backend),
        ("cmd_run",                h.cmd_run),
        ("cmd_doctor",             h.cmd_doctor),
        ("cmd_schedule",           h.cmd_schedule),
        ("cmd_restore",            h.cmd_restore),
        ("cmd_snapshots",          h.cmd_snapshots),
        ("cmd_status",             h.cmd_status),
        ("cmd_update",             h.cmd_update),
        ("cmd_uninstall",          h.cmd_uninstall),
        ("cmd_setup_dependencies", h.cmd_setup_dependencies),
        ("cmd_setup_backend_init", h.cmd_setup_backend_init),
        ("opt_setup_lang",         h.opt_setup_lang),
        ("opt_setup_non_interactive", h.opt_setup_non_interactive),
        ("cmd_config_show",        h.cmd_config_show),
        ("cmd_config_edit",        h.cmd_config_edit),
        ("cmd_config_import_legacy", h.cmd_config_import_legacy),
        ("cmd_config_export",      h.cmd_config_export),
        ("opt_config_import_file", h.opt_config_import_file),
        ("opt_config_export_format", h.opt_config_export_format),
        ("cmd_backend_migrate",    h.cmd_backend_migrate),
        ("opt_run_skip_database",  h.opt_run_skip_database),
        ("opt_run_skip_secondary_sync", h.opt_run_skip_secondary_sync),
        ("opt_run_skip_retention", h.opt_run_skip_retention),
        ("opt_run_dry_run",        h.opt_run_dry_run),
        ("cmd_doctor_environment", h.cmd_doctor_environment),
        ("cmd_doctor_time_sync",   h.cmd_doctor_time_sync),
        ("cmd_doctor_restore_drill", h.cmd_doctor_restore_drill),
        ("opt_doctor_file",        h.opt_doctor_file),
        ("cmd_schedule_enable",    h.cmd_schedule_enable),
        ("cmd_schedule_disable",   h.cmd_schedule_disable),
        ("cmd_schedule_status",    h.cmd_schedule_status),
        ("opt_restore_snapshot",   h.opt_restore_snapshot),
        ("opt_restore_target",     h.opt_restore_target),
        ("opt_uninstall_yes",      h.opt_uninstall_yes),
        ("opt_uninstall_purge",    h.opt_uninstall_purge),
    ];
    for (field_name, value) in fields {
        let has_korean = value.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c));
        if has_korean {
            panic!(
                "[영어 모드 혼용] 필드 '{}' = {:?} 에 한국어 문자 발견",
                field_name, value
            );
        }
    }
}
