use backup::commands::update::{is_newer_version, parse_version};

#[test]
fn test_parse_version() {
    assert_eq!(parse_version("v0.1.5"), Some((0, 1, 5)));
    assert_eq!(parse_version("0.1.5"), Some((0, 1, 5)));
    assert_eq!(parse_version("v1.2.3-rc1"), Some((1, 2, 3)));
    assert_eq!(parse_version("invalid"), None);
}

#[test]
fn test_is_newer_version() {
    assert!(is_newer_version("0.1.5", "v0.1.6"));
    assert!(is_newer_version("0.1.5", "0.2.0"));
    assert!(is_newer_version("0.1.5", "1.0.0"));
    assert!(!is_newer_version("0.1.5", "v0.1.5"));
    assert!(!is_newer_version("0.1.5", "0.1.4"));
}
