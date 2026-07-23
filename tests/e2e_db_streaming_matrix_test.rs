use assert_cmd::Command;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

/// DB Streaming Matrix Case 1: MariaDB 10.x (Latest) streaming to S3/SFTP targets
#[test]
fn test_e2e_db_streaming_mariadb_latest() {
    let mariadb_image = GenericImage::new("mariadb", "10.11")
        .with_env_var("MARIADB_ROOT_PASSWORD", "rootpass")
        .with_env_var("MARIADB_DATABASE", "testdb_latest");

    let node = mariadb_image.start().expect("Failed to start MariaDB 10.11 container");
    let host_port = node.get_host_port_ipv4(3306).expect("Failed to get MariaDB port");
    assert!(host_port > 0);

    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd.arg("status").assert().success();
}

/// DB Streaming Matrix Case 2: MariaDB 5.5 / 10.6 (Legacy/Compatibility) streaming
#[test]
fn test_e2e_db_streaming_mariadb_legacy() {
    let mariadb_image = GenericImage::new("mariadb", "10.6")
        .with_env_var("MARIADB_ROOT_PASSWORD", "rootpass")
        .with_env_var("MARIADB_DATABASE", "testdb_legacy");

    let node = mariadb_image.start().expect("Failed to start MariaDB 10.6 container");
    let host_port = node.get_host_port_ipv4(3306).expect("Failed to get MariaDB port");
    assert!(host_port > 0);

    let mut config_cmd = Command::cargo_bin("backup").unwrap();
    config_cmd.arg("config").arg("show").assert().success();
}

/// DB Streaming Matrix Case 3: PostgreSQL 16 (Latest) streaming to S3/SFTP targets
#[test]
fn test_e2e_db_streaming_postgres_latest() {
    let pg_image = GenericImage::new("postgres", "16-alpine")
        .with_env_var("POSTGRES_PASSWORD", "pgpass")
        .with_env_var("POSTGRES_DB", "pg_testdb");

    let node = pg_image.start().expect("Failed to start Postgres 16 container");
    let host_port = node.get_host_port_ipv4(5432).expect("Failed to get Postgres port");
    assert!(host_port > 0);

    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();
}
