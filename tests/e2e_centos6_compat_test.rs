//! CentOS 6 compatibility runner contract.
//!
//! The Docker execution is intentionally opt-in in local environments. This test
//! keeps the image contract visible in the default suite and catches accidental removal of the
//! legacy tools required by the Platform Support Profile.

use std::path::Path;
use std::process::Command;

#[test]
fn centos6_runner_declares_the_supported_x86_64_toolchain() {
    let dockerfile = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docker/Dockerfile.centos6"),
    )
    .unwrap();
    for required in [
        "FROM centos:6",
        "openssh-clients",
        "openssh-server",
        "cronie",
        "ntp",
        "x86_64",
    ] {
        assert!(
            dockerfile.contains(required),
            "CentOS 6 runner missing {required}"
        );
    }
    assert!(!dockerfile.contains("systemd PID 1"));

    let script = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/test_centos6.sh"),
    )
    .unwrap();
    for required in [
        "backup-centos6/profiles.yaml run",
        "backup-centos6/profiles.yaml restore",
        "backup-centos6/profiles.yaml doctor",
        "backup-centos6/profiles.yaml report",
        "export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/bin",
        "ssh-keyscan",
        "sshd",
        "StrictHostKeyChecking=yes",
    ] {
        assert!(script.contains(required), "CentOS smoke missing {required}");
    }
}

#[test]
#[ignore = "requires Docker, CentOS 6 Vault access, and the musl Rust target"]
fn centos6_runtime_smoke_executes_the_real_binary() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/test_centos6.sh");
    let status = Command::new("sh")
        .arg(script)
        .status()
        .expect("start CentOS 6 runtime smoke script");
    assert!(status.success(), "CentOS 6 runtime smoke script failed");
}
