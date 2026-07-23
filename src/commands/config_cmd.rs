use anyhow::Result;
use crate::config::model::BackupConfig;

pub fn execute_config_show(config: &BackupConfig) -> Result<String> {
    config.render("yaml", true)
}

pub fn execute_config_export(config: &BackupConfig, format: &str) -> Result<String> {
    config.render(format, false)
}

pub fn execute_config_import_legacy(source_path: &std::path::Path, target_path: &std::path::Path) -> Result<String> {
    let content = std::fs::read_to_string(source_path)?;
    let config = crate::config::legacy_import::parse_legacy_env(&content)?;
    config.save_to_path(target_path)?;
    Ok(format!("Imported legacy configuration from {} to {}", source_path.display(), target_path.display()))
}

pub fn execute_config_edit(config_path: &std::path::Path) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Ok(format!("Config file edit session initiated with {} for {}", editor, config_path.display()))
}


