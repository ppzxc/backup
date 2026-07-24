use backup::i18n::{I18nMessages, Language};

#[test]
fn test_language_parse_and_detection() {
    assert_eq!(Language::from_str("ko"), Language::Ko);
    assert_eq!(Language::from_str("korean"), Language::Ko);
    assert_eq!(Language::from_str("en"), Language::En);
    assert_eq!(Language::from_str("invalid"), Language::En);
}

#[test]
fn test_language_detect() {
    unsafe {
        std::env::set_var("LANG", "ko_KR.UTF-8");
    }
    assert_eq!(Language::detect(), Language::Ko);

    unsafe {
        std::env::set_var("LANG", "en_US.UTF-8");
    }
    assert_eq!(Language::detect(), Language::En);
}

#[test]
fn test_i18n_messages_lookup() {
    let msg_ko = I18nMessages::get(Language::Ko);
    let msg_en = I18nMessages::get(Language::En);

    assert!(msg_ko.enter_profile_name.contains("프로필"));
    assert!(msg_en.enter_profile_name.contains("Profile"));
}
