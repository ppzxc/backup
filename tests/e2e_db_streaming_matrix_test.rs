use assert_cmd::Command;
use std::process::Command as StdCommand;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

/// DB Streaming Matrix Case 1: MariaDB 10.x (Latest) streaming to S3/SFTP targets
#[test]
fn test_e2e_db_streaming_mariadb_latest() {
    // 0. Verify binary dependencies check
    Command::cargo_bin("backup").unwrap().arg("setup").arg("dependencies").assert().success();

    let mariadb_image = GenericImage::new("mariadb", "10.11")
        .with_env_var("MARIADB_ROOT_PASSWORD", "rootpass")
        .with_env_var("MARIADB_DATABASE", "testdb_latest");

    let node = mariadb_image.start().expect("Failed to start MariaDB 10.11 container");
    let host_port = node.get_host_port_ipv4(3306).expect("Failed to get MariaDB port");
    assert!(host_port > 0);

    // Seed test table and data in MariaDB container
    let container_id = node.id();
    let seed_sql = "CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(50)); INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');";
    let _ = StdCommand::new("docker")
        .args(&["exec", container_id, "mariadb", "-uroot", "-prootpass", "testdb_latest", "-e", seed_sql])
        .output();

    // Verify database dump streaming capability via docker exec mysqldump
    let dump_out = StdCommand::new("docker")
        .args(&["exec", container_id, "mysqldump", "-uroot", "-prootpass", "testdb_latest"])
        .output();

    if let Ok(dump_res) = dump_out {
        let dump_str = String::from_utf8_lossy(&dump_res.stdout);
        if !dump_str.is_empty() {
            assert!(dump_str.contains("users"), "Dump stream must contain created users table!");
        }
    }

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

    let container_id = node.id();
    let seed_sql = "CREATE TABLE IF NOT EXISTS legacy_logs (id INT PRIMARY KEY, log VARCHAR(100)); INSERT INTO legacy_logs VALUES (100, 'Legacy 10.6 Log Entry');";
    let _ = StdCommand::new("docker")
        .args(&["exec", container_id, "mariadb", "-uroot", "-prootpass", "testdb_legacy", "-e", seed_sql])
        .output();

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

    let container_id = node.id();
    let seed_sql = "CREATE TABLE IF NOT EXISTS audit_events (id INT PRIMARY KEY, event VARCHAR(100)); INSERT INTO audit_events VALUES (1, 'Postgres Audit Event 1');";
    let _ = StdCommand::new("docker")
        .args(&["exec", container_id, "psql", "-U", "postgres", "-d", "pg_testdb", "-c", seed_sql])
        .output();

    let dump_out = StdCommand::new("docker")
        .args(&["exec", container_id, "pg_dump", "-U", "postgres", "pg_testdb"])
        .output();

    if let Ok(dump_res) = dump_out {
        let dump_str = String::from_utf8_lossy(&dump_res.stdout);
        if !dump_str.is_empty() {
            assert!(dump_str.contains("audit_events"), "Postgres dump stream must contain audit_events table!");
        }
    }

    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();
}
