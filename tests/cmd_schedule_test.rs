use backup::commands::schedule::{generate_systemd_service, generate_systemd_timer};

#[test]
fn test_generate_systemd_timer() {
    let timer = generate_systemd_timer("02:00:00");
    assert!(timer.contains("OnCalendar=*-*-* 02:00:00"));
}

#[test]
fn test_generate_systemd_service() {
    let service = generate_systemd_service("/usr/local/bin/backup");
    assert!(service.contains("ExecStart=/usr/local/bin/backup run"));
}
