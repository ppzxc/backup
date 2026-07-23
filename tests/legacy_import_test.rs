use backup::config::legacy_import::parse_legacy_env;

#[test]
fn test_parse_legacy_env() {
    let env_content = r#"
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="192.168.1.100"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="2222"
export RESTIC_PASSWORD="testpassword"
export BACKUP_TARGETS="/home/user/data"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="host1"
"#;
    let config = parse_legacy_env(env_content).unwrap();
    assert_eq!(config.profile, "host1");
    assert_eq!(config.retention.keep_daily, 7);
}
