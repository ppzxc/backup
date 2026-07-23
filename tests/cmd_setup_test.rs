use backup::commands::setup::create_default_config_file;
use tempfile::tempdir;

#[test]
fn test_create_default_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");
    create_default_config_file(&config_path, "host1", "/data", "s3:bucket", "secret").unwrap();
    assert!(config_path.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let parent_perms = std::fs::metadata(dir.path()).unwrap().permissions();
        assert_eq!(parent_perms.mode() & 0o777, 0o700);
        let file_perms = std::fs::metadata(&config_path).unwrap().permissions();
        assert_eq!(file_perms.mode() & 0o777, 0o600);
    }
}
