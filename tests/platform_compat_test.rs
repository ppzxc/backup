use backup::commands::database::ensure_database_supported_on_platform;
use backup::commands::doctor::{DoctorStatus, check_time_sync_with_capabilities};
use backup::config::model::SftpAuthPolicy;
use backup::platform::{
    PlatformCapabilities, PlatformProfile, SchedulerSelection, SshKeyAlgorithm, TimeSyncMethod,
};
use backup::runner::executor::{CommandOutput, StrictCommandRunner};
use backup::runner::scheduler::SchedulerSettings;
use std::path::Path;

mod support;

#[test]
fn centos_6_10_x86_64_profile_describes_legacy_runtime_capabilities() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();

    assert_eq!(capabilities.profile, PlatformProfile::Centos6X86_64);
    assert_eq!(capabilities.scheduler_selection(), SchedulerSelection::Cron);
    assert!(!capabilities.systemd_available);
    assert!(capabilities.cron_available);
    assert!(capabilities.crond_running);
    assert_eq!(capabilities.time_sync_method(), TimeSyncMethod::Ntpd);
    assert!(!capabilities.chrony_available);
    assert!(capabilities.ntpd_available);
    assert_eq!(capabilities.ssh_key_algorithm(), SshKeyAlgorithm::Rsa);
    assert!(!capabilities.ssh_accept_new);
    assert!(capabilities.supports_database("mariadb", "5.5.56"));
    assert!(!capabilities.supports_database("postgres", "16"));
}

#[test]
fn platform_detection_recognizes_centos_release_without_os_release_file() {
    let capabilities =
        PlatformCapabilities::from_release_metadata("CentOS release 6.10 (Final)", "x86_64");

    assert_eq!(capabilities.profile, PlatformProfile::Centos6X86_64);
    assert_eq!(capabilities.ssh_key_algorithm(), SshKeyAlgorithm::Rsa);
}

#[test]
fn platform_detection_does_not_treat_other_centos_6_releases_as_6_10() {
    let capabilities =
        PlatformCapabilities::from_release_metadata("CentOS Linux release 6.9 (Final)", "x86_64");

    assert_ne!(capabilities.profile, PlatformProfile::Centos6X86_64);
}

#[test]
fn modern_profile_keeps_systemd_chrony_and_ed25519_defaults() {
    let capabilities = PlatformCapabilities::modern_linux_x86_64();

    assert_eq!(capabilities.profile, PlatformProfile::ModernLinux);
    assert_eq!(
        capabilities.scheduler_selection(),
        SchedulerSelection::Systemd
    );
    assert_eq!(capabilities.time_sync_method(), TimeSyncMethod::Chrony);
    assert_eq!(capabilities.ssh_key_algorithm(), SshKeyAlgorithm::Ed25519);
    assert!(capabilities.ssh_accept_new);
}

#[test]
fn legacy_sftp_policy_uses_rsa_and_strict_known_hosts() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let policy = SftpAuthPolicy::for_config_dir_with_capabilities(
        Path::new("/etc/backup/id_rsa"),
        Path::new("/etc/backup"),
        &capabilities,
    )
    .unwrap();

    assert!(policy.argument_tokens().unwrap().contains(&"-i".into()));
    assert!(
        policy
            .argument_tokens()
            .unwrap()
            .contains(&"StrictHostKeyChecking=yes".into())
    );
}

#[test]
fn legacy_sftp_connection_registers_host_key_before_strict_connection() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("id_rsa"), "private key").unwrap();
    let key = temp.path().join("id_rsa");
    let runner = StrictCommandRunner::new([
        StrictCommandRunner::expectation(
            "ssh-keyscan",
            ["-T", "5", "-p", "22", "sftp.example"],
            &[],
            CommandOutput {
                status_code: 0,
                stdout: "[sftp.example]:22 ssh-rsa AAAA".into(),
                stderr: String::new(),
            },
        ),
        StrictCommandRunner::expectation(
            "sftp",
            [
                "-i",
                key.to_str().unwrap(),
                "-o",
                "IdentitiesOnly=yes",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=yes",
                "-o",
                &format!(
                    "UserKnownHostsFile={}",
                    temp.path().join("known_hosts").display()
                ),
                "-P",
                "22",
                "-o",
                "ConnectTimeout=5",
                "-b",
                "/dev/null",
                "backup@sftp.example",
            ],
            &[],
            CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        ),
    ]);

    backup::commands::setup::verify_sftp_connection_with_config_dir_and_capabilities(
        "backup",
        "sftp.example",
        22,
        key.to_str().unwrap(),
        temp.path(),
        &runner,
        &capabilities,
    )
    .unwrap();
    assert!(
        std::fs::read_to_string(temp.path().join("known_hosts"))
            .unwrap()
            .contains("ssh-rsa")
    );
    runner.assert_exhausted().unwrap();
}

#[test]
fn legacy_sftp_reuse_accepts_the_strict_host_key_policy_it_generates() {
    use std::collections::BTreeMap;

    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let temp = tempfile::tempdir().unwrap();
    let key = temp.path().join("id_rsa");
    std::fs::write(&key, "private key").unwrap();
    std::fs::write(
        temp.path().join("known_hosts"),
        "[sftp.example]:22 ssh-rsa AAAA\n",
    )
    .unwrap();
    let mut options = BTreeMap::new();
    options.insert(
        "sftp.args".into(),
        format!(
            "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile={}",
            key.display(),
            temp.path().join("known_hosts").display()
        ),
    );

    let reused = backup::commands::setup::resolve_reused_sftp_config_with_capabilities(
        "sftp:backup@sftp.example:/backup",
        Some(&options),
        temp.path(),
        &capabilities,
    )
    .unwrap();

    assert_eq!(reused.key_file.as_deref(), Some(key.to_str().unwrap()));
}

#[test]
fn runtime_sftp_correction_requires_a_managed_known_hosts_file() {
    use std::collections::BTreeMap;

    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let temp = tempfile::tempdir().unwrap();
    let key = temp.path().join("id_rsa");
    std::fs::write(&key, "private key").unwrap();
    let mut options = BTreeMap::from([(
        "sftp.args".into(),
        format!(
            "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new",
            key.display()
        ),
    )]);

    let changed = backup::commands::setup::correct_managed_sftp_options_with_capabilities(
        "sftp:backup@sftp.example:/backup",
        &mut options,
        temp.path(),
        &capabilities,
    )
    .unwrap();

    assert!(!changed);
    assert!(options["sftp.args"].contains("accept-new"));

    std::fs::write(temp.path().join("known_hosts"), "").unwrap();
    let changed = backup::commands::setup::correct_managed_sftp_options_with_capabilities(
        "sftp:backup@sftp.example:/backup",
        &mut options,
        temp.path(),
        &capabilities,
    )
    .unwrap();

    assert!(changed);
    assert!(options["sftp.args"].contains("StrictHostKeyChecking=yes"));
}

#[test]
fn ntpd_diagnostic_is_used_when_chrony_is_not_available() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "ntpq",
        ["-pn"],
        &[],
        CommandOutput {
            status_code: 0,
            stdout: "*127.127.1.0 .LOCL. 0 l 64 0 0 0.000 0.000 0.000".into(),
            stderr: String::new(),
        },
    )]);

    let (status, detail) = check_time_sync_with_capabilities(&capabilities, &runner);

    assert_eq!(status, DoctorStatus::Pass);
    assert!(detail.contains("ntpd"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn ntpd_diagnostic_warns_when_no_peer_is_selected() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let runner = StrictCommandRunner::new([StrictCommandRunner::expectation(
        "ntpq",
        ["-pn"],
        &[],
        CommandOutput {
            status_code: 0,
            stdout: "     remote           refid      st t when poll reach   delay   offset  jitter\nserver.example .INIT. 16 u - 64 0 0.000 0.000 0.000".into(),
            stderr: String::new(),
        },
    )]);

    let (status, detail) = check_time_sync_with_capabilities(&capabilities, &runner);

    assert_eq!(status, DoctorStatus::Warn);
    assert!(detail.contains("synchronization unavailable"));
    runner.assert_exhausted().unwrap();
}

#[test]
fn centos_database_gate_rejects_postgres_before_dump_execution() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();

    assert!(ensure_database_supported_on_platform(&capabilities, "mariadb", "5.5.56").is_ok());
    let error = ensure_database_supported_on_platform(&capabilities, "postgres", "16").unwrap_err();
    assert!(error.to_string().contains("CentOS 6"));
}

#[test]
fn centos_database_gate_requires_the_supported_mariadb_client_version() {
    let mut capabilities = PlatformCapabilities::centos_6_10_x86_64();
    capabilities.mariadb_client_version = Some("10.11.0".into());

    let error = backup::commands::database::ensure_database_type_supported_on_platform(
        &capabilities,
        backup::config::model::DatabaseType::Mysql,
    )
    .unwrap_err();

    assert!(error.to_string().contains("5.5.56"));
}

#[test]
fn scheduler_settings_carry_one_platform_capability_snapshot() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let settings = SchedulerSettings::auto().with_platform_capabilities(capabilities.clone());

    assert_eq!(settings.platform_capabilities(), Some(&capabilities));
}

#[test]
fn report_exposes_generic_legacy_time_sync_and_scheduler_fields() {
    let capabilities = PlatformCapabilities::centos_6_10_x86_64();
    let meta = backup::commands::report::AuditReportMeta::new("host", "now")
        .with_platform_capabilities(capabilities);
    let data = backup::commands::report::RealReportData::collect_with_meta_with_runner(
        &backup::commands::report::ReportConfig::default(),
        &meta,
        &support::MockExecutor::new(),
    );

    assert_eq!(data.time_sync_method, "ntpd");
    assert_eq!(data.scheduler_backend, "cron");
}

#[test]
fn report_marks_failed_timedatectl_as_unavailable() {
    let mut capabilities = PlatformCapabilities::modern_linux_x86_64();
    capabilities.chrony_available = false;
    capabilities.ntpd_available = false;
    let meta = backup::commands::report::AuditReportMeta::new("host", "now")
        .with_platform_capabilities(capabilities);
    let runner = support::MockExecutor::new();
    runner.push_output(
        "timedatectl",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "timedatectl failed".into(),
        },
    );

    let data = backup::commands::report::RealReportData::collect_with_meta_with_runner(
        &backup::commands::report::ReportConfig::default(),
        &meta,
        &runner,
    );

    assert_eq!(data.time_sync_status, "unavailable");
}

#[test]
fn report_queries_the_scheduler_unit_registered_by_backup_scheduler() {
    let capabilities = PlatformCapabilities::modern_linux_x86_64();
    let meta = backup::commands::report::AuditReportMeta::new("host", "now")
        .with_platform_capabilities(capabilities);
    let runner = support::MockExecutor::new();

    let _ = backup::commands::report::RealReportData::collect_with_meta_with_runner(
        &backup::commands::report::ReportConfig::default(),
        &meta,
        &runner,
    );

    let calls = runner.get_calls();
    assert!(calls.iter().any(|(program, args)| {
        program == "systemctl"
            && args
                == &[
                    "is-enabled".to_string(),
                    "backup-pipeline.timer".to_string(),
                ]
    }));
    assert!(!calls.iter().any(|(program, args)| {
        program == "systemctl" && args.iter().any(|arg| arg == "backup.timer")
    }));
}

#[test]
fn cron_registration_fails_before_crontab_when_crond_capability_is_false() {
    let mut capabilities = PlatformCapabilities::centos_6_10_x86_64();
    capabilities.crond_running = false;
    let settings = SchedulerSettings::auto().with_platform_capabilities(capabilities);
    let runner = StrictCommandRunner::new([]);
    let scheduler = backup::runner::scheduler::SystemScheduler::new(&runner, "/usr/bin/backup");

    let error = backup::runner::scheduler::BackupScheduler::enable_with_settings(
        &scheduler,
        Path::new("/etc/backup/profiles.yaml"),
        &settings,
    )
    .unwrap_err();

    assert!(error.to_string().contains("crond is not running"));
    assert!(runner.calls().is_empty());
}

#[test]
fn cron_status_and_disable_can_inspect_state_when_crond_is_stopped() {
    let mut capabilities = PlatformCapabilities::centos_6_10_x86_64();
    capabilities.crond_running = false;
    let settings = SchedulerSettings::auto().with_platform_capabilities(capabilities);
    let runner = support::MockExecutor::new();
    runner.push_output(
        "crontab",
        CommandOutput {
            status_code: 0,
            stdout: "0 3 * * * /usr/bin/backup # backup-pipeline\n".into(),
            stderr: String::new(),
        },
    );
    let scheduler = backup::runner::scheduler::SystemScheduler::new(&runner, "/usr/bin/backup");

    assert!(
        backup::runner::scheduler::BackupScheduler::status_with_settings(&scheduler, &settings)
            .unwrap()
            .contains("active")
    );
    // The exact temporary path is intentionally not part of the public contract; a successful
    // disable proves the daemon state is not incorrectly required for removal operations.
    assert!(
        backup::runner::scheduler::BackupScheduler::disable_with_settings(&scheduler, &settings,)
            .is_ok()
    );
}

#[test]
fn offline_dependency_install_requires_and_verifies_sha256_manifest() {
    use backup::commands::setup::run_setup_dependencies_with_options;
    use tempfile::tempdir;

    let archive = tempdir().unwrap();
    for binary in ["restic", "rclone", "resticprofile"] {
        std::fs::write(archive.path().join(binary), b"hello").unwrap();
    }
    std::fs::write(
        archive.path().join("SHA256SUMS"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  restic\n2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  rclone\n2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  resticprofile\n",
    )
    .unwrap();
    let install_dir = archive.path().join("bin");

    let report = run_setup_dependencies_with_options(
        &StrictCommandRunner::new([]),
        &install_dir,
        backup::i18n::Language::En,
        Some(archive.path()),
    )
    .unwrap();

    assert!(report.contains("SHA-256"));
    assert!(install_dir.join("restic").is_file());
    assert!(install_dir.join("rclone").is_file());
    assert!(install_dir.join("resticprofile").is_file());
}

#[test]
fn verified_dependency_command_uses_the_current_artifact_url_without_legacy_fallback() {
    let command = backup::commands::setup::build_verified_download_command(
        "rclone",
        "https://downloads.rclone.org/v1.68.1/rclone-v1.68.1-linux-amd64.zip",
        "/usr/local/bin",
    );

    assert!(
        command.contains("https://downloads.rclone.org/v1.68.1/rclone-v1.68.1-linux-amd64.zip")
    );
    assert!(command.contains("sha256sum -c"));
    let rejected = backup::commands::setup::build_verified_download_command(
        "rclone",
        "https://downloads.example.test/rclone-current-linux-amd64.zip",
        "/usr/local/bin",
    );
    assert!(rejected.contains("Unsupported verified dependency artifact"));
    assert!(!rejected.contains("sha256sum -c"));
}
