// steam_import.rs
// Scans Steam library folders for installed games, reads appmanifest ACF files,
// detects existing compatdata prefixes, and provides prefix cloning.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::LauncherError;
use crate::vdf;

// ── Steam game metadata ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    pub appid: String,
    pub name: String,
    pub installdir: String,
    pub size_on_disk: u64,
    pub library_path: String,
    /// Full path to the compatdata prefix (if it exists)
    pub steam_prefix: Option<String>,
    /// Full path to the game's install directory
    pub install_path: String,
    /// Whether this game already exists in the mythix library
    pub already_imported: bool,
}

// Skip IDs: runtimes, redistributables, Proton builds — not real games
const SKIP_APPIDS: &[&str] = &[
    "228980",   // Steamworks Common Redistributables
    "1007",     // Steam Linux Runtime - soldier
    "1070560",  // Steam Linux Runtime - sniper
    "1391110",  // Steam Linux Runtime 3.0
    "1628350",  // Steam Linux Runtime 3.0 (sniper)
    "4183110",  // Proton Easy Anti-Cheat Runtime
    "1493710",  // Proton Experimental
    "1887720",  // Proton 8
    "2348590",  // Proton 9
    "2180100",  // Proton Hotfix
];

// ── Library discovery ────────────────────────────────────────────────────────

fn steam_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let debian = home.join(".steam").join("debian-installation");
    if debian.join("steamapps").is_dir() {
        return Some(debian);
    }
    let default = home.join(".steam").join("steam");
    if default.join("steamapps").is_dir() {
        return Some(default);
    }
    None
}

pub fn find_steam_libraries() -> Vec<PathBuf> {
    let Some(root) = steam_root() else { return Vec::new() };
    let vdf_path = root.join("steamapps").join("libraryfolders.vdf");

    if !vdf_path.is_file() {
        // Fallback: just use the root
        return vec![root];
    }

    let Ok(doc) = vdf::load(&vdf_path) else { return vec![root] };
    let Some(folders) = doc.get("libraryfolders") else { return vec![root] };
    let Some(map) = folders.as_map() else { return vec![root] };

    let mut libs = Vec::new();
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(999));

    for key in keys {
        if let Some(entry) = map.get(key) {
            if let Some(path_str) = entry.str_val("path") {
                let p = PathBuf::from(path_str);
                if p.join("steamapps").is_dir() {
                    libs.push(p);
                }
            }
        }
    }

    if libs.is_empty() {
        libs.push(root);
    }
    libs
}

// ── Game scanning ────────────────────────────────────────────────────────────

pub fn scan_steam_games(existing_game_ids: &HashSet<String>) -> Result<Vec<SteamGame>, LauncherError> {
    let libraries = find_steam_libraries();
    let mut games = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let skip: HashSet<&str> = SKIP_APPIDS.iter().copied().collect();

    for lib in &libraries {
        let steamapps = lib.join("steamapps");
        let Ok(entries) = fs::read_dir(&steamapps) else { continue };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }

            let Ok(doc) = vdf::load(&path) else { continue };
            let Some(app) = doc.get("appstate") else { continue };

            let appid = match app.str_val("appid") {
                Some(id) => id.to_string(),
                None => continue,
            };

            if seen.contains(&appid) || skip.contains(appid.as_str()) {
                continue;
            }
            seen.insert(appid.clone());

            let game_name = app.str_val("name").unwrap_or("Unknown").to_string();
            let installdir = app.str_val("installdir").unwrap_or("").to_string();
            let size: u64 = app.str_val("sizeondisk")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Check for compatdata prefix
            let compat_path = steamapps.join("compatdata").join(&appid).join("pfx");
            let steam_prefix = if compat_path.is_dir() {
                Some(compat_path.parent().unwrap().to_string_lossy().into_owned())
            } else {
                None
            };

            let install_path = steamapps.join("common").join(&installdir);

            // Check if already imported. Match both the new mythix-<appid> id
            // and the legacy looni-<appid> id so pre-rebrand imports still dedup.
            let mythix_id = format!("mythix-{}", appid);
            let legacy_id = format!("looni-{}", appid);
            let already_imported = existing_game_ids.contains(&mythix_id)
                || existing_game_ids.contains(&legacy_id);

            games.push(SteamGame {
                appid,
                name: game_name,
                installdir,
                size_on_disk: size,
                library_path: lib.to_string_lossy().into_owned(),
                steam_prefix,
                install_path: install_path.to_string_lossy().into_owned(),
                already_imported,
            });
        }
    }

    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(games)
}

// ── Prefix cloning ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    pub prefix_path: String,
    pub size_bytes: u64,
}

/// Clone a Steam compatdata prefix to a standalone mythix prefix.
/// Returns the path to the new prefix directory.
pub fn clone_steam_prefix(
    steam_prefix_path: &str,
    dest_name: &str,
) -> Result<CloneResult, LauncherError> {
    let src = PathBuf::from(steam_prefix_path);
    if !src.is_dir() {
        return Err(LauncherError::Prefix(format!(
            "Steam prefix not found: {}", steam_prefix_path
        )));
    }

    let pfx_src = src.join("pfx");
    if !pfx_src.is_dir() {
        return Err(LauncherError::Prefix(format!(
            "No pfx/ subdirectory in: {}", steam_prefix_path
        )));
    }

    let dest_base = crate::paths::dot_dir()
        .ok_or_else(|| LauncherError::Io("Cannot resolve HOME".into()))?
        .join("gamez-pfx_data")
        .join(dest_name);

    // If it already exists, remove it (user confirmed overwrite from frontend)
    if dest_base.exists() {
        fs::remove_dir_all(&dest_base)
            .map_err(|e| LauncherError::Prefix(format!("remove old prefix: {e}")))?;
    }

    fs::create_dir_all(&dest_base)
        .map_err(|e| LauncherError::Prefix(format!("create prefix dir: {e}")))?;

    // Use cp -a for a faithful copy (preserves symlinks, permissions, timestamps)
    let status = std::process::Command::new("cp")
        .args(["-a", &pfx_src.to_string_lossy(), &dest_base.join("pfx").to_string_lossy()])
        .status()
        .map_err(|e| LauncherError::Prefix(format!("cp: {e}")))?;

    if !status.success() {
        return Err(LauncherError::Prefix("cp -a failed".into()));
    }

    // Also copy shadercache if it exists
    let shader_src = src.join("shadercache");
    if shader_src.is_dir() {
        let _ = std::process::Command::new("cp")
            .args(["-a", &shader_src.to_string_lossy(), &dest_base.join("shadercache").to_string_lossy()])
            .status();
    }

    // Create tracked_files marker
    let tracked = dest_base.join("tracked_files");
    if !tracked.exists() {
        fs::File::create(&tracked).ok();
    }

    // Calculate size
    let size = dir_size(&dest_base);

    Ok(CloneResult {
        prefix_path: dest_base.to_string_lossy().into_owned(),
        size_bytes: size,
    })
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

// ── Exe detection ───────────────────────────────────────────────────────────

/// Try to find the main .exe for a game in its install directory.
pub fn find_game_exe(install_path: &str) -> Option<String> {
    let game_dir = PathBuf::from(install_path);
    if !game_dir.is_dir() {
        return None;
    }

    let skip_patterns = [
        "unins", "setup", "install", "redist", "vcredist",
        "dxsetup", "dotnet", "crash", "report", "update",
        "ue4prereq", "dxwebsetup", "easyanticheat",
    ];

    let mut candidates: Vec<(i64, PathBuf)> = Vec::new();

    fn scan_dir(dir: &Path, base: &Path, skip: &[&str], candidates: &mut Vec<(i64, PathBuf)>, depth: u32) {
        if depth > 4 { return; }
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                scan_dir(&p, base, skip, candidates, depth + 1);
            } else if let Some(ext) = p.extension() {
                if ext.to_str().map(|s| s.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                    let name_lower = p.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if skip.iter().any(|s| name_lower.contains(s)) {
                        continue;
                    }
                    let size = p.metadata().map(|m| m.len() as i64).unwrap_or(0);
                    let rel_depth = p.strip_prefix(base).map(|r| r.components().count()).unwrap_or(10) as i64;
                    // Score: larger size + shallower depth = better
                    let score = size - (rel_depth * 100_000_000);
                    candidates.push((score, p));
                }
            }
        }
    }

    scan_dir(&game_dir, &game_dir, &skip_patterns, &mut candidates, 0);
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.first().map(|(_, p)| p.to_string_lossy().into_owned())
}
