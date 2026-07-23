use testcontainers::runners::SyncRunner;
use testcontainers::GenericImage;

#[test]
fn test_s3_minio_container_lifecycle() {
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);

    let node = minio_image.start().expect("Failed to start MinIO container");
    let host_port = node.get_host_port_ipv4(9000).expect("Failed to get port");
    assert!(host_port > 0);
}
