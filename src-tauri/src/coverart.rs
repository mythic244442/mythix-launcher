// Cover art — Steam CDN auto-fetch + custom image import.
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::LauncherError;

fn covers_dir() -> Result<PathBuf, LauncherError> {
    let dir = crate::paths::data_dir()
        .ok_or_else(|| LauncherError::Io("Cannot resolve XDG_DATA_HOME".into()))?
        .join("covers");
    fs::create_dir_all(&dir).map_err(|e| LauncherError::Io(e.to_string()))?;
    Ok(dir)
}

fn steam_appid_from_game_id(game_id: &str) -> Option<u64> {
    crate::paths::strip_game_id_prefix(game_id)?.parse::<u64>().ok()
}

/// Fetch cover art from Steam's public CDN (no API key needed).
pub async fn fetch_cover(
    game_id: &str,
    library_game_id: &str,
) -> Result<PathBuf, LauncherError> {
    let appid = steam_appid_from_game_id(game_id)
        .ok_or_else(|| LauncherError::Io(
            "No numeric Steam appid — use 'Choose Image' to set a custom cover".into()
        ))?;

    let client = reqwest::Client::builder()
        .user_agent("mythix-launcher/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    // Steam CDN portrait art (600x900)
    let urls = [
        format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg", appid),
        format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900_2x.jpg", appid),
        format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/header.jpg", appid),
    ];

    for url in &urls {
        let resp = match client.get(url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };
        let bytes = resp.bytes().await
            .map_err(|e| LauncherError::Io(format!("cover download failed: {e}")))?;
        if bytes.len() < 1000 { continue; } // placeholder/error page

        let dest = covers_dir()?.join(format!("{library_game_id}.jpg"));
        clean_old_covers(library_game_id, "jpg")?;
        fs::write(&dest, &bytes).map_err(|e| LauncherError::Io(e.to_string()))?;
        return Ok(dest);
    }

    Err(LauncherError::Io(format!("No cover art found on Steam CDN for appid {appid}")))
}

/// Import a custom image file as cover art.
pub fn import_cover(
    source_path: &Path,
    library_game_id: &str,
) -> Result<PathBuf, LauncherError> {
    if !source_path.is_file() {
        return Err(LauncherError::Io(format!("File not found: {}", source_path.display())));
    }

    let ext = source_path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "jpg".into());

    let ext = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" => ext,
        _ => return Err(LauncherError::Io(format!("Unsupported image format: .{ext}"))),
    };

    let dest = covers_dir()?.join(format!("{library_game_id}.{ext}"));
    clean_old_covers(library_game_id, &ext)?;
    fs::copy(source_path, &dest).map_err(|e| LauncherError::Io(e.to_string()))?;
    Ok(dest)
}

fn clean_old_covers(library_game_id: &str, keep_ext: &str) -> Result<(), LauncherError> {
    let dir = covers_dir()?;
    for ext in ["jpg", "jpeg", "png", "webp", "bmp", "gif"] {
        if ext == keep_ext { continue; }
        let stale = dir.join(format!("{library_game_id}.{ext}"));
        if stale.exists() { let _ = fs::remove_file(&stale); }
    }
    Ok(())
}

pub fn remove_cover(library_game_id: &str) -> Result<(), LauncherError> {
    let dir = covers_dir()?;
    for ext in ["jpg", "jpeg", "png", "webp", "bmp", "gif"] {
        let p = dir.join(format!("{library_game_id}.{ext}"));
        if p.exists() {
            fs::remove_file(&p).map_err(|e| LauncherError::Io(e.to_string()))?;
        }
    }
    Ok(())
}

pub fn cover_path_if_cached(library_game_id: &str) -> Option<PathBuf> {
    let dir = covers_dir().ok()?;
    for ext in ["jpg", "jpeg", "png", "webp", "bmp", "gif"] {
        let p = dir.join(format!("{library_game_id}.{ext}"));
        if p.exists() { return Some(p); }
    }
    None
}
