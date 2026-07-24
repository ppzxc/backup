use anyhow::{anyhow, Result};
use std::process::Command;

/// 시맨틱 버전 수치 파싱 (예: "v0.1.5" -> Some((0, 1, 5)))
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let clean = v.trim().trim_start_matches('v');
    let base = clean.split('-').next()?;
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts[2].parse::<u32>().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

/// latest 버전이 current 버전보다 최신인지 비교합니다.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

/// GitHub Releases API를 조회하여 최신 태그명과 다운로드 URL을 가져옵니다.
pub fn fetch_latest_release_info() -> Result<(String, String)> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "User-Agent: backup-cli",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/ppzxc/backup/releases/latest",
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to fetch release info from GitHub Releases API"));
    }

    let body = String::from_utf8(output.stdout)?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("tag_name not found in release response"))?
        .to_string();

    let target_asset_name = format!("backup-{}-x86_64-unknown-linux-musl.tar.gz", tag_name);
    let mut download_url = String::new();

    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name == target_asset_name {
                    if let Some(url) = asset["browser_download_url"].as_str() {
                        download_url = url.to_string();
                        break;
                    }
                }
            }
        }
    }

    if download_url.is_empty() {
        // 백업 기본 URL 구조
        download_url = format!(
            "https://github.com/ppzxc/backup/releases/download/{}/{}",
            tag_name, target_asset_name
        );
    }

    Ok((tag_name, download_url))
}

/// 현재 실행 바이너리를 다운로드한 새 바이너리로 교체합니다.
pub fn perform_self_replace(download_url: &str) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let tmp_dir = tempfile::tempdir()?;
    let archive_path = tmp_dir.path().join("backup_update.tar.gz");

    // 1. 다운로드
    let status = Command::new("curl")
        .args(["-fsSL", download_url, "-o", archive_path.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to download update package from {}", download_url));
    }

    // 2. 압축 해제
    let status = Command::new("tar")
        .args([
            "-xzf",
            archive_path.to_str().unwrap(),
            "-C",
            tmp_dir.path().to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to extract update package"));
    }

    let new_binary = tmp_dir.path().join("backup");
    if !new_binary.exists() {
        return Err(anyhow!("Extracted binary 'backup' not found"));
    }

    // 3. 권한 설정 및 덮어쓰기
    Command::new("chmod").args(["+x", new_binary.to_str().unwrap()]).status()?;

    // Linux에서 실행 중인 바이너리 덮어쓰기 (rename / replace)
    std::fs::rename(&new_binary, &current_exe).or_else(|_| {
        let backup_exe = current_exe.with_extension("old");
        std::fs::rename(&current_exe, &backup_exe)?;
        std::fs::copy(&new_binary, &current_exe)?;
        let _ = std::fs::remove_file(backup_exe);
        Ok::<(), std::io::Error>(())
    })?;

    Ok(())
}

/// 자가 업데이트 실행 및 결과 메시지를 반환합니다.
pub fn execute_update_check(current_version: &str) -> Result<String> {
    match fetch_latest_release_info() {
        Ok((latest_tag, download_url)) => {
            if is_newer_version(current_version, &latest_tag) {
                println!("New version {} found. Updating from {}...", latest_tag, current_version);
                if let Err(e) = perform_self_replace(&download_url) {
                    Ok(format!(
                        "New version {} available at {}, but auto-update failed: {}",
                        latest_tag, download_url, e
                    ))
                } else {
                    Ok(format!("Successfully updated backup to version {}!", latest_tag))
                }
            } else {
                Ok(format!("Current version is {}. Already up to date.", current_version))
            }
        }
        Err(e) => Ok(format!(
            "Current version is {}. Failed to check latest release online: {}",
            current_version, e
        )),
    }
}
