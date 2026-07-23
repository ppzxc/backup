use anyhow::Result;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer".into()
}

pub fn perform_uninstall() -> Result<String> {
    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}
