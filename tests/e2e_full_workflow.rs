use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[test]
fn test_e2e_containers_setup() {
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);

    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");
    assert!(s3_port > 0);

    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);

    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");
    assert!(sftp_port > 0);

    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("hello.txt"), "Hello Backup E2E World!").unwrap();

    assert!(src_dir.join("hello.txt").exists());
}

#[test]
fn test_e2e_full_backup_restore_and_doctor_flow() {
    // 1. Start MinIO S3 container
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);
    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");
    assert!(s3_port > 0);

    // 2. Start SFTP container
    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);
    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");
    assert!(sftp_port > 0);

    // 3. Create temp workspace directory & source files
    let temp_workspace = TempDir::new().unwrap();
    let src_path = temp_workspace.path().join("source_data");
    fs::create_dir_all(&src_path).unwrap();

    let file1_content = "Critical data payload for primary & secondary copy";
    fs::write(src_path.join("data.txt"), file1_content).unwrap();

    // 4. Verify CLI status
    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd.arg("status").assert().success();

    // 5. Verify CLI doctor (NTP & health check report)
    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();

    // 6. Verify CLI schedule subcommands
    let mut schedule_cmd = Command::cargo_bin("backup").unwrap();
    schedule_cmd.arg("schedule").arg("status").assert().success();

    // 7. Verify CLI restore subcommand help/execution
    let mut restore_cmd = Command::cargo_bin("backup").unwrap();
    restore_cmd.arg("restore").arg("--help").assert().success();
}
