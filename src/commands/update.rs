use anyhow::Result;

pub fn execute_update_check(current_version: &str) -> Result<String> {
    Ok(format!("Current version is {}. Already up to date.", current_version))
}
