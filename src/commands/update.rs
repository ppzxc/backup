use crate::runner::executor::{CommandRunner, SystemExecutor};
use anyhow::{Result, anyhow};
use std::path::Path;

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
pub fn fetch_latest_release_info_with_runner<R: CommandRunner + ?Sized>(
    runner: &R,
) -> Result<(String, String)> {
    let (tag, url, _) = fetch_latest_release_info_with_checksum(runner)?;
    Ok((tag, url))
}

fn fetch_latest_release_info_with_checksum<R: CommandRunner + ?Sized>(
    runner: &R,
) -> Result<(String, String, Option<String>)> {
    let output = runner.run(
        "curl",
        &[
            "-fsSL",
            "-H",
            "User-Agent: backup-cli",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/ppzxc/backup/releases/latest",
        ],
    )?;

    if output.status_code != 0 {
        return Err(anyhow!(
            "Failed to fetch release info from GitHub Releases API"
        ));
    }

    let json: serde_json::Value = serde_json::from_str(&output.stdout)?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("tag_name not found in release response"))?
        .to_string();

    let target_asset_name = format!("backup-{}-x86_64-unknown-linux-musl.tar.gz", tag_name);
    let mut download_url = String::new();
    let mut checksum = None;

    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name == target_asset_name {
                    if let Some(url) = asset["browser_download_url"].as_str() {
                        download_url = url.to_string();
                        checksum = asset["digest"]
                            .as_str()
                            .and_then(|digest| digest.strip_prefix("sha256:"))
                            .map(str::to_owned);
                        break;
                    }
                }
            }
        }
    }

    if download_url.is_empty() {
        download_url = format!(
            "https://github.com/ppzxc/backup/releases/download/{}/{}",
            tag_name, target_asset_name
        );
    }

    Ok((tag_name, download_url, checksum))
}

pub fn fetch_latest_release_info() -> Result<(String, String)> {
    let runner = SystemExecutor;
    fetch_latest_release_info_with_runner(&runner)
}

/// 현재 실행 바이너리를 다운로드한 새 바이너리로 교체합니다.
pub fn perform_self_replace_with_runner<R: CommandRunner + ?Sized>(
    download_url: &str,
    runner: &R,
) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let staging_parent = current_exe
        .parent()
        .ok_or_else(|| anyhow!("current executable has no parent directory"))?;
    perform_self_replace_at_path_with_runner(download_url, &current_exe, staging_parent, runner)
}

/// Download and atomically install an update at an explicit executable path.
///
/// All remote and archive work happens inside a private staging directory.  The current binary is
/// not touched until the extracted replacement exists, has been made executable, and passes the
/// final filesystem check.  This explicit path seam keeps lifecycle tests away from the running
/// test binary while preserving the production path through `current_exe`.
pub fn perform_self_replace_at_path_with_runner<R: CommandRunner + ?Sized>(
    download_url: &str,
    current_exe: &Path,
    staging_parent: &Path,
    runner: &R,
) -> Result<()> {
    perform_self_replace_at_path_with_runner_and_checksum(
        download_url,
        current_exe,
        staging_parent,
        runner,
        None,
    )
}

pub fn perform_self_replace_at_path_with_runner_and_checksum<R: CommandRunner + ?Sized>(
    download_url: &str,
    current_exe: &Path,
    staging_parent: &Path,
    runner: &R,
    expected_checksum: Option<&str>,
) -> Result<()> {
    if let Some(parent) = current_exe.parent() {
        if !parent.exists() {
            return Err(anyhow!(
                "current executable parent does not exist: {}",
                parent.display()
            ));
        }
    }
    std::fs::create_dir_all(staging_parent)?;
    let tmp_dir = tempfile::Builder::new()
        .prefix(".backup-update-")
        .tempdir_in(staging_parent)?;
    let archive_path = tmp_dir.path().join("backup_update.tar.gz");
    let archive_path_str = archive_path
        .to_str()
        .ok_or_else(|| anyhow!("update archive path is not valid UTF-8"))?;
    let tmp_dir_str = tmp_dir
        .path()
        .to_str()
        .ok_or_else(|| anyhow!("update staging path is not valid UTF-8"))?;

    // 1. 다운로드
    let out = runner.run("curl", &["-fsSL", download_url, "-o", archive_path_str])?;
    if out.status_code != 0 {
        return Err(anyhow!("Failed to download update package"));
    }
    if !archive_path.is_file() {
        return Err(anyhow!("Downloaded update package was not created"));
    }
    if let Some(expected_checksum) = expected_checksum {
        crate::commands::setup::verify_sha256_file(&archive_path, expected_checksum)?;
    }

    // 2. 압축 해제
    let out = runner.run("tar", &["-xzf", archive_path_str, "-C", tmp_dir_str])?;
    if out.status_code != 0 {
        return Err(anyhow!("Failed to extract update package"));
    }

    let new_binary = tmp_dir.path().join("backup");
    if !new_binary.exists() {
        return Err(anyhow!("Extracted binary 'backup' not found"));
    }
    if !new_binary.is_file() {
        return Err(anyhow!("Extracted binary 'backup' is not a regular file"));
    }

    // 3. 권한 설정 및 덮어쓰기
    let new_binary_str = new_binary
        .to_str()
        .ok_or_else(|| anyhow!("Extracted binary path is not valid UTF-8"))?;
    let chmod = runner.run("chmod", &["+x", new_binary_str])?;
    if chmod.status_code != 0 {
        return Err(anyhow!("Failed to mark extracted update binary executable"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&new_binary)?.permissions().mode();
        if mode & 0o111 == 0 {
            return Err(anyhow!("Extracted update binary is not executable"));
        }
    }
    #[cfg(not(unix))]
    if std::fs::metadata(&new_binary)?.permissions().readonly() {
        return Err(anyhow!("Extracted update binary is not executable"));
    }

    // Same-filesystem rename replaces the old executable atomically.  In particular, do not move
    // the old binary aside and copy over it: a copy failure would leave a partial installation.
    std::fs::rename(&new_binary, current_exe).map_err(|error| {
        anyhow!(
            "Failed to atomically replace {}: {error}",
            current_exe.display()
        )
    })?;

    Ok(())
}

pub fn perform_self_replace(download_url: &str) -> Result<()> {
    let runner = SystemExecutor;
    perform_self_replace_with_runner(download_url, &runner)
}

/// 자가 업데이트 실행 및 결과 메시지를 반환합니다.
pub fn execute_update_check_with_runner<R: CommandRunner + ?Sized>(
    current_version: &str,
    runner: &R,
) -> Result<String> {
    tracing::info!(current_version = %current_version, "Checking for software updates");
    let (latest_tag, download_url, checksum) = fetch_latest_release_info_with_checksum(runner)?;
    if is_newer_version(current_version, &latest_tag) {
        let checksum = checksum
            .ok_or_else(|| anyhow!("latest release does not provide a SHA-256 artifact digest"))?;
        let current_exe = std::env::current_exe()?;
        let staging_parent = current_exe
            .parent()
            .ok_or_else(|| anyhow!("current executable has no parent directory"))?;
        perform_self_replace_at_path_with_runner_and_checksum(
            &download_url,
            &current_exe,
            staging_parent,
            runner,
            Some(&checksum),
        )?;
        Ok(format!(
            "Updating from {} to {}...\nSuccessfully updated backup to version {}!",
            current_version, latest_tag, latest_tag
        ))
    } else {
        Ok(format!(
            "Current version is {}. Already up to date.",
            current_version
        ))
    }
}

pub fn execute_update_check(current_version: &str) -> Result<String> {
    let runner = SystemExecutor;
    execute_update_check_with_runner(current_version, &runner)
}
