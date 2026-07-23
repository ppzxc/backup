use testcontainers::runners::SyncRunner;
use testcontainers::GenericImage;

#[test]
fn test_mariadb_container_lifecycle() {
    let mariadb_image = GenericImage::new("mariadb", "10.6")
        .with_env_var("MARIADB_ROOT_PASSWORD", "rootpass")
        .with_env_var("MARIADB_DATABASE", "testdb");

    let node = mariadb_image.start().expect("Failed to start MariaDB container");
    let host_port = node.get_host_port_ipv4(3306).expect("Failed to get port");
    assert!(host_port > 0);
}

#[test]
fn test_postgres_container_lifecycle() {
    let pg_image = GenericImage::new("postgres", "15-alpine")
        .with_env_var("POSTGRES_PASSWORD", "pgpass")
        .with_env_var("POSTGRES_DB", "testdb");

    let node = pg_image.start().expect("Failed to start Postgres container");
    let host_port = node.get_host_port_ipv4(5432).expect("Failed to get port");
    assert!(host_port > 0);
}
