//! Platform Support Profile and the capabilities discovered for one process run.
//!
//! The rest of the application consumes this value instead of branching on an OS name at each
//! call site.  The profile constructors are deterministic seams for tests; `detect` is the
//! imperative shell used by the production binary.

use crate::runner::executor::{CommandRunner, SystemExecutor};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformProfile {
    Centos6X86_64,
    ModernLinux,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerSelection {
    Systemd,
    Cron,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSyncMethod {
    Chrony,
    Ntpd,
    Timedatectl,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SshKeyAlgorithm {
    Ed25519,
    Rsa,
    Unavailable,
}

impl SshKeyAlgorithm {
    pub const fn key_name(self, secondary: bool) -> &'static str {
        match (self, secondary) {
            (Self::Rsa, false) => "id_rsa",
            (Self::Rsa, true) => "id_rsa_secondary",
            (_, false) => "id_ed25519",
            (_, true) => "id_ed25519_secondary",
        }
    }

    pub const fn ssh_keygen_type(self) -> &'static str {
        match self {
            Self::Rsa => "rsa",
            Self::Ed25519 => "ed25519",
            Self::Unavailable => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub profile: PlatformProfile,
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub systemd_available: bool,
    pub cron_available: bool,
    pub crond_running: bool,
    pub chrony_available: bool,
    pub ntpd_available: bool,
    pub ed25519_supported: bool,
    pub rsa_supported: bool,
    pub ssh_accept_new: bool,
    pub mariadb_client_version: Option<String>,
}

impl PlatformCapabilities {
    pub fn centos_6_10_x86_64() -> Self {
        Self {
            profile: PlatformProfile::Centos6X86_64,
            os_name: "CentOS".into(),
            os_version: "6.10".into(),
            architecture: "x86_64".into(),
            systemd_available: false,
            cron_available: true,
            crond_running: true,
            chrony_available: false,
            ntpd_available: true,
            ed25519_supported: false,
            rsa_supported: true,
            ssh_accept_new: false,
            mariadb_client_version: Some("5.5.56".into()),
        }
    }

    pub fn modern_linux_x86_64() -> Self {
        Self {
            profile: PlatformProfile::ModernLinux,
            os_name: "Linux".into(),
            os_version: String::new(),
            architecture: "x86_64".into(),
            systemd_available: true,
            cron_available: true,
            crond_running: true,
            chrony_available: true,
            ntpd_available: false,
            ed25519_supported: true,
            rsa_supported: true,
            ssh_accept_new: true,
            mariadb_client_version: None,
        }
    }

    /// Resolves the stable platform profile from release metadata.  CentOS 6 support is
    /// deliberately limited to x86_64; other Linux systems use the modern capability baseline
    /// until their probes say otherwise.
    pub fn from_release_metadata(release: &str, architecture: &str) -> Self {
        let release_lower = release.to_ascii_lowercase();
        if release_lower.contains("centos")
            && release_lower.contains("6.10")
            && architecture == "x86_64"
        {
            return Self::centos_6_10_x86_64();
        }

        let mut capabilities = Self::modern_linux_x86_64();
        capabilities.architecture = architecture.into();
        if architecture != "x86_64" {
            capabilities.profile = PlatformProfile::Unsupported;
        }
        capabilities.os_name = release
            .split_whitespace()
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or("Linux")
            .into();
        capabilities.os_version = release.into();
        capabilities
    }

    /// Detects the platform once through the command seam. Missing probes are represented as
    /// unavailable capabilities and never make the process panic.
    pub fn detect_with_runner<C: CommandRunner + ?Sized>(runner: &C) -> Self {
        let architecture = runner
            .run("uname", &["-m"])
            .ok()
            .filter(|output| output.status_code == 0)
            .map(|output| output.stdout.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| std::env::consts::ARCH.into());
        let release = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/etc/redhat-release"))
            .unwrap_or_else(|_| "Linux".into());
        let mut capabilities = Self::from_release_metadata(&release, &architecture);

        capabilities.systemd_available = command_succeeds(runner, "systemctl", &["--version"]);
        capabilities.cron_available = command_exists(runner, "crontab");
        // CentOS names the daemon `crond`; Debian-family systemd images expose the same
        // capability as the `cron` service.  Probe both names so the capability describes the
        // scheduler daemon rather than one distribution's service spelling.
        capabilities.crond_running = command_succeeds(runner, "service", &["crond", "status"])
            || command_succeeds(runner, "service", &["cron", "status"])
            || command_succeeds(runner, "systemctl", &["is-active", "crond"])
            || command_succeeds(runner, "systemctl", &["is-active", "cron"]);
        capabilities.chrony_available = command_succeeds(runner, "chronyc", &["tracking"]);
        capabilities.ntpd_available = command_exists(runner, "ntpq");
        capabilities.mariadb_client_version = runner
            .run("mysqldump", &["--version"])
            .ok()
            .filter(|output| output.status_code == 0)
            .and_then(|output| extract_version(&format!("{}\n{}", output.stdout, output.stderr)));
        // `ssh-keygen -t ... -f /dev/null` prompts before returning on modern OpenSSH. Inspect
        // its non-interactive usage output instead of generating a probe key or risking a hang.
        capabilities.ed25519_supported =
            command_output_contains(runner, "ssh-keygen", &["--help"], "ed25519");
        capabilities.rsa_supported = command_exists(runner, "ssh-keygen");
        capabilities.ssh_accept_new =
            !matches!(capabilities.profile, PlatformProfile::Centos6X86_64)
                && command_succeeds(
                    runner,
                    "ssh",
                    &[
                        "-G",
                        "-o",
                        "StrictHostKeyChecking=accept-new",
                        "-o",
                        "BatchMode=yes",
                        "localhost",
                    ],
                );
        capabilities
    }

    pub fn detect() -> Self {
        Self::detect_with_runner(&SystemExecutor)
    }

    pub fn scheduler_selection(&self) -> SchedulerSelection {
        if self.systemd_available {
            SchedulerSelection::Systemd
        } else if self.cron_available && self.crond_running {
            SchedulerSelection::Cron
        } else {
            SchedulerSelection::Unavailable
        }
    }

    pub fn time_sync_method(&self) -> TimeSyncMethod {
        if self.chrony_available {
            TimeSyncMethod::Chrony
        } else if self.ntpd_available {
            TimeSyncMethod::Ntpd
        } else if self.systemd_available {
            TimeSyncMethod::Timedatectl
        } else {
            TimeSyncMethod::Unavailable
        }
    }

    pub fn ssh_key_algorithm(&self) -> SshKeyAlgorithm {
        if self.ed25519_supported {
            SshKeyAlgorithm::Ed25519
        } else if self.rsa_supported {
            SshKeyAlgorithm::Rsa
        } else {
            SshKeyAlgorithm::Unavailable
        }
    }

    pub fn ssh_ed25519_supported(&self) -> bool {
        self.ed25519_supported
    }

    pub fn supports_database(&self, database: &str, version: &str) -> bool {
        let database = database.to_ascii_lowercase();
        if self.profile == PlatformProfile::Centos6X86_64 {
            return matches!(database.as_str(), "mariadb" | "mysql") && version == "5.5.56";
        }
        matches!(
            database.as_str(),
            "mariadb" | "mysql" | "postgres" | "postgresql"
        )
    }

    pub fn is_centos_6(&self) -> bool {
        self.profile == PlatformProfile::Centos6X86_64
    }
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self::modern_linux_x86_64()
    }
}

fn command_exists<C: CommandRunner + ?Sized>(runner: &C, program: &str) -> bool {
    runner
        .run("which", &[program])
        .is_ok_and(|output| output.status_code == 0 && !output.stdout.trim().is_empty())
}

fn command_succeeds<C: CommandRunner + ?Sized>(runner: &C, program: &str, args: &[&str]) -> bool {
    runner
        .run(program, args)
        .is_ok_and(|output| output.status_code == 0)
}

fn command_output_contains<C: CommandRunner + ?Sized>(
    runner: &C,
    program: &str,
    args: &[&str],
    needle: &str,
) -> bool {
    runner.run(program, args).is_ok_and(|output| {
        format!("{}\n{}", output.stdout, output.stderr)
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
    })
}

fn extract_version(value: &str) -> Option<String> {
    value
        .split(|character: char| !character.is_ascii_digit() && character != '.')
        .find(|candidate| {
            candidate.split('.').count() >= 3
                && candidate
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        })
        .map(str::to_owned)
}

/// Returns whether `ntpq -pn` selected a peer for synchronization. A successful command with
/// only an unselected peer list is not evidence that the clock is synchronized.
pub fn ntpq_output_is_synchronized(output: &str) -> bool {
    output.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with('*')
            || trimmed
                .split_whitespace()
                .any(|field| field.starts_with('*'))
    })
}
