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
