use backup::commands::schedule::{
    execute_schedule_disable, execute_schedule_enable, execute_schedule_status,
    generate_systemd_service, generate_systemd_timer,
};
use backup::runner::resticprofile::MockResticProfileRunner;
use std::path::Path;

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

#[test]
fn test_execute_schedule_commands() {
    let mock = MockResticProfileRunner::new(0, "scheduled successfully");
    let path = Path::new("/etc/backup/profiles.yaml");

    let res_enable = execute_schedule_enable(path, &mock).unwrap();
    assert_eq!(res_enable, "scheduled successfully");

    let res_disable = execute_schedule_disable(path, &mock).unwrap();
    assert_eq!(res_disable, "scheduled successfully");

    let res_status = execute_schedule_status(path, &mock).unwrap();
    assert_eq!(res_status, "scheduled successfully");
}
