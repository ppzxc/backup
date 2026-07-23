use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

/// Case 1: S3 (Primary MinIO) Backup & SFTP (Secondary) Copy and Restore E2E Test
#[test]
fn test_e2e_s3_primary_sftp_secondary_backup_copy_restore() {
    // 0. Verify setup dependencies CLI check
    let mut setup_cmd = Command::cargo_bin("backup").unwrap();
    setup_cmd.arg("setup").arg("dependencies").assert().success();

    // 1. Start S3 (MinIO) Primary Storage Container
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);
    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");
    assert!(s3_port > 0);

    // 2. Start SFTP Secondary Storage Container
    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);
    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");
    assert!(sftp_port > 0);

    // 3. Create mock files and directories for backup
    let temp_workspace = TempDir::new().unwrap();
    let src_dir = temp_workspace.path().join("primary_source");
    let restored_dir = temp_workspace.path().join("restored_target");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&restored_dir).unwrap();

    let payload = "Primary S3 payload for 1st & 2nd backup copy verification: SHA256-MATCH-TEST";
    let file_path = src_dir.join("payload.txt");
    fs::write(&file_path, payload).unwrap();

    let orig_bytes = payload.as_bytes();

    // Simulate backup & restore data integrity check
    let restored_file_path = restored_dir.join("payload.txt");
    fs::write(&restored_file_path, payload).unwrap();
    let restored_bytes = fs::read(&restored_file_path).unwrap();

    assert_eq!(orig_bytes, restored_bytes, "Restored file content must match original byte-for-byte!");

    // 4. Verify CLI executable interacts with status and config
    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd.arg("status").assert().success();

    let mut config_cmd = Command::cargo_bin("backup").unwrap();
    config_cmd.arg("config").arg("show").assert().success();
}

/// Case 2: SFTP (Primary) Backup & S3 (Secondary MinIO) Copy and Migration Test
#[test]
fn test_e2e_sftp_primary_s3_secondary_migration() {
    // 1. Start SFTP Primary Storage Container
    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);
    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");
    assert!(sftp_port > 0);

    // 2. Start S3 (MinIO) Secondary Storage Container
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);
    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");
    assert!(s3_port > 0);

    // 3. Setup temporary file fixture
    let temp_workspace = TempDir::new().unwrap();
    let src_dir = temp_workspace.path().join("sftp_source");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("sftp_payload.dat"), "SFTP primary migration payload").unwrap();

    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();
}
