use anyhow::Result;

pub fn execute_restore(snapshot_id: &str, target_path: &str) -> Result<String> {
    Ok(format!("Restored snapshot {} to {}", snapshot_id, target_path))
}
