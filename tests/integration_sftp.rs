use testcontainers::runners::SyncRunner;
use testcontainers::GenericImage;

#[test]
fn test_sftp_container_lifecycle() {
    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["foo:pass:::upload"]);

    let node = sftp_image.start().expect("Failed to start SFTP container");
    let host_port = node.get_host_port_ipv4(22).expect("Failed to get port");
    assert!(host_port > 0);
}
