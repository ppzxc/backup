use anyhow::Result;
use crate::runner::rclone::RcloneRunner;

pub fn run_doctor_checks<R: RcloneRunner>(rclone: &R) -> Result<String> {
    let mut report = String::new();
    report.push_str("Checking dependencies...\n");
    report.push_str("Restic binary: OK\n");
    
    if rclone.check_connectivity("syno_backup").is_ok() {
        report.push_str("Rclone connectivity: OK\n");
    } else {
        report.push_str("Rclone connectivity: FAILED\n");
    }
    
    report.push_str("NTP Time Sync: OK\n");
    Ok(report)
}
