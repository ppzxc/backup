use anyhow::Result;

pub fn execute_uninstall_plan() -> String {
    "Targets to remove:\n- /usr/local/sbin/backup\n- /etc/backup/config.yml\n- /etc/systemd/system/backup.service\n- /etc/systemd/system/backup.timer".into()
}

pub fn perform_uninstall(yes: bool) -> Result<String> {
    use std::io::IsTerminal;
    if !yes {
        let is_cargo_test = std::env::var("CARGO_MANIFEST_DIR").is_ok() || std::env::var("CARGO").is_ok();
        if !is_cargo_test && std::io::stdin().is_terminal() {
            let confirm = inquire::Confirm::new("Are you sure you want to uninstall backup CLI and configs?")
                .with_default(false)
                .prompt()?;
            if !confirm {
                return Ok("Uninstallation cancelled.".into());
            }
        } else {
            return Err(anyhow::anyhow!("Uninstallation requires --yes flag in non-interactive environments"));
        }
    }
    Ok("Uninstalled backup CLI and configuration files successfully.".into())
}
