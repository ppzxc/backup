use anyhow::Result;
use std::path::Path;
use crate::runner::resticprofile::ResticProfileRunner;

pub fn execute_schedule_enable<R: ResticProfileRunner>(config_path: &Path, runner: &R) -> Result<String> {
    runner.schedule_enable(config_path)
}

pub fn execute_schedule_disable<R: ResticProfileRunner>(config_path: &Path, runner: &R) -> Result<String> {
    runner.schedule_disable(config_path)
}

pub fn execute_schedule_status<R: ResticProfileRunner>(config_path: &Path, runner: &R) -> Result<String> {
    runner.schedule_status(config_path)
}

pub fn generate_systemd_service(binary_path: &str) -> String {
    format!(
        r#"[Unit]
Description=Restic Backup Service
After=network.target

[Service]
Type=oneshot
ExecStart={} run
"#,
        binary_path
    )
}

pub fn generate_systemd_timer(on_calendar: &str) -> String {
    format!(
        r#"[Unit]
Description=Restic Backup Timer

[Timer]
OnCalendar=*-*-* {}
Persistent=true

[Install]
WantedBy=timers.target
"#,
        on_calendar
    )
}
