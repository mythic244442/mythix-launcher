// game_library.rs
// Game library persistence and per-game configuration.
// Stores data in $XDG_DATA_HOME/mythix/library.json

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::LauncherError;

// ── Data model ───────────────────────────────────────────────────────────────

/// A Wine prefix type, mirroring Proton's conventions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrefixType {
    /// Standard Proton-style prefix ($WINEPREFIX/pfx)
    Proton,
    /// Vanilla Wine prefix (no pfx subdir)
    Wine,
}

impl Default for PrefixType {
    fn default() -> Self {
        Self::Proton
    }
}

/// Per-game launch configuration.
/// Analogous to mythix's env dict, but structured and persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Absolute path to the Wine/Proton prefix directory.
    /// Defaults to $HOME/Games/mythix/<game_id>
    pub prefix_path: Option<PathBuf>,

    /// Absolute path to the Proton or wine-proton hybrid directory.
    /// Must contain a `proton` or `wine` executable.
    pub proton_path: Option<PathBuf>,

    /// Store identifier (e.g. "gog", "egs", "itch", "heroic", "")
    pub store: String,

    /// Game ID used to build STEAM_COMPAT_APP_ID.
    /// Format: 8-character hex string, or any string for manual entries.
    pub game_id: String,

    /// Additional environment variables injected at launch time.
    /// These override any value set by the launcher's env setup.
    pub env_overrides: HashMap<String, String>,

    /// Extra arguments passed to the executable.
    pub launch_args: Vec<String>,

    /// Proton verb. Defaults to "waitforexitandrun".
    pub proton_verb: String,

    /// Prefix type. Proton creates a pfx/ symlink; vanilla Wine doesn't.
    #[serde(default)]
    pub prefix_type: PrefixType,

    /// Override Steam Runtime usage for this game.
    /// None = auto (use toolmanifest), Some(true) = force runtime,
    /// Some(false) = force no runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_runtime: Option<bool>,

    /// Which runtime container to use. None = auto (sniper for modern tools).
    /// Values: "sniper", "soldier", "steamrt4"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_variant: Option<String>,

}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            prefix_path: None,
            proton_path: None,
            store: String::new(),
            game_id: Uuid::new_v4().to_string().replace('-', "")[..8].to_string(),
            env_overrides: HashMap::new(),
            launch_args: Vec::new(),
            proton_verb: "waitforexitandrun".to_string(),
            prefix_type: PrefixType::Proton,
            use_runtime: None,
            runtime_variant: None,
        }
    }
}

/// A game entry in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// UUID, stable across renames.
    pub id: String,
    /// Display name shown in the library.
    pub name: String,
    /// Absolute path to the .exe or launch script.
    pub exe_path: PathBuf,
    /// Optional path to a cover image (JPEG/PNG).
    pub cover_art: Option<PathBuf>,
    /// Per-game launch configuration.
    pub config: GameConfig,
    /// ISO 8601 timestamp of when the game was added.
    pub added_at: String,
    /// Total playtime in seconds.
    pub playtime_secs: u64,
    /// Optional free-form notes.
    pub notes: String,
}

/// The root structure serialised to library.json.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Library {
    pub version: u32,
    pub games: Vec<Game>,
}

// ── Persistence ──────────────────────────────────────────────────────────────

fn library_path() -> Result<PathBuf, LauncherError> {
    let data_dir = crate::paths::data_dir()
        .ok_or_else(|| LauncherError::Io("Cannot resolve XDG_DATA_HOME".into()))?;
    Ok(data_dir.join("library.json"))
}

pub fn load_library() -> Result<Library, LauncherError> {
    let path = library_path()?;
    if !path.exists() {
        return Ok(Library {
            version: 1,
            games: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| LauncherError::Io(e.to_string()))?;
    if raw.trim().is_empty() {
        return Ok(Library {
            version: 1,
            games: Vec::new(),
        });
    }
    serde_json::from_str(&raw)
        .map_err(|e| LauncherError::Serialization(e.to_string()))
}

pub fn save_library(lib: &Library) -> Result<(), LauncherError> {
    let path = library_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| LauncherError::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(lib)
        .map_err(|e| LauncherError::Serialization(e.to_string()))?;
    fs::write(&path, json)
        .map_err(|e| LauncherError::Io(e.to_string()))
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

pub fn add_game(name: String, exe_path: PathBuf) -> Result<Game, LauncherError> {
    let mut lib = load_library()?;
    let settings = crate::settings::load_settings();

    let id = Uuid::new_v4().to_string();
    let game_id = id.replace('-', "")[..8].to_string();

    // Use settings default prefix root if set, otherwise fall back to the standard path
    let default_prefix = if !settings.default_prefix_root.is_empty() {
        Some(PathBuf::from(&settings.default_prefix_root).join(&game_id))
    } else {
        crate::paths::default_prefix_dir(&game_id)
    };

    // Use settings default proton path if set
    let default_proton = if !settings.default_proton_path.is_empty() {
        Some(PathBuf::from(&settings.default_proton_path))
    } else {
        None
    };

    let config = GameConfig {
        prefix_path: default_prefix,
        proton_path: default_proton,
        game_id: game_id.clone(),
        proton_verb: settings.default_proton_verb.clone(),
        ..Default::default()
    };

    let game = Game {
        id: id.clone(),
        name,
        exe_path,
        cover_art: None,
        config,
        added_at: chrono_now(),
        playtime_secs: 0,
        notes: String::new(),
    };

    lib.games.push(game.clone());
    save_library(&lib)?;

    Ok(game)
}

pub fn remove_game(game_id: &str) -> Result<(), LauncherError> {
    let mut lib = load_library()?;
    lib.games.retain(|g| g.id != game_id);
    save_library(&lib)
}

pub fn update_game_config(game_id: &str, config: GameConfig) -> Result<(), LauncherError> {
    let mut lib = load_library()?;
    let game = lib
        .games
        .iter_mut()
        .find(|g| g.id == game_id)
        .ok_or_else(|| LauncherError::NotFound(game_id.to_string()))?;
    game.config = config;
    save_library(&lib)
}

pub fn update_game_meta(
    game_id: &str,
    name: Option<String>,
    cover_art: Option<PathBuf>,
    notes: Option<String>,
    exe_path: Option<PathBuf>,
) -> Result<(), LauncherError> {
    let mut lib = load_library()?;
    let game = lib
        .games
        .iter_mut()
        .find(|g| g.id == game_id)
        .ok_or_else(|| LauncherError::NotFound(game_id.to_string()))?;
    if let Some(n) = name { game.name = n; }
    if let Some(c) = cover_art { game.cover_art = Some(c); }
    if let Some(n) = notes { game.notes = n; }
    if let Some(e) = exe_path { game.exe_path = e; }
    save_library(&lib)
}

pub fn record_playtime(game_id: &str, secs: u64) -> Result<(), LauncherError> {
    let mut lib = load_library()?;
    let game = lib
        .games
        .iter_mut()
        .find(|g| g.id == game_id)
        .ok_or_else(|| LauncherError::NotFound(game_id.to_string()))?;
    game.playtime_secs += secs;
    save_library(&lib)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn chrono_now() -> String {
    // Simple RFC-3339-ish timestamp without pulling in chrono
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{secs}")
}

// ── Validation ───────────────────────────────────────────────────────────────

/// Locate the Neutron launcher script in a tool dir. Older builds ship a
/// `neutron` executable; the rewritten Proton-format launcher is `neutron.py`.
pub fn neutron_script(dir: &Path) -> Option<PathBuf> {
    ["neutron", "neutron.py"]
        .iter()
        .map(|n| dir.join(n))
        .find(|p| p.is_file())
}

/// Check a game's config for obvious errors before attempting a launch.
pub fn validate_config(game: &Game) -> Result<(), LauncherError> {
    if !game.exe_path.is_file() {
        return Err(LauncherError::InvalidConfig(format!(
            "Executable not found: {}",
            game.exe_path.display()
        )));
    }
    if let Some(pfx) = &game.config.prefix_path {
        if pfx.exists() && !pfx.is_dir() {
            return Err(LauncherError::InvalidConfig(format!(
                "Prefix path exists but is not a directory: {}",
                pfx.display()
            )));
        }
    }
    if let Some(proton) = &game.config.proton_path {
        if !proton.is_dir() {
            return Err(LauncherError::InvalidConfig(format!(
                "Proton path is not a directory: {}",
                proton.display()
            )));
        }
        // Must have a proton, neutron, or wine executable
        let has_proton = proton.join("proton").is_file();
        let has_neutron = neutron_script(proton).is_some();
        let has_wine = proton.join("bin").join("wine").is_file()
            || proton.join("wine").is_file()
            || proton.join("files").join("bin").join("wine").is_file();
        if !has_proton && !has_neutron && !has_wine {
            return Err(LauncherError::InvalidConfig(format!(
                "No 'proton', 'neutron', or 'wine' executable found in: {}",
                proton.display()
            )));
        }
    }
    Ok(())
}

// ── Proton version scanning ──────────────────────────────────────────────────

/// A detected Proton/Wine compatibility tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatTool {
    pub name: String,
    pub path: PathBuf,
    /// "proton", "wine-proton-hybrid", "wine", or "other"
    pub kind: String,
}

/// Scan the standard compatibility tool directories for installed versions.
///
/// Searches (in order):
///   - $XDG_DATA_HOME/Steam/compatibilitytools.d
///   - $HOME/.steam/root/compatibilitytools.d
///   - $XDG_DATA_HOME/mythix/compatibilitytools
pub fn scan_compat_tools() -> Vec<CompatTool> {
    let mut tools: Vec<CompatTool> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    let search_dirs: Vec<PathBuf> = {
        let mut dirs_list = Vec::new();

        // Custom tools directory from settings (checked first)
        let settings = crate::settings::load_settings();
        if !settings.custom_tools_dir.is_empty() {
            dirs_list.push(PathBuf::from(&settings.custom_tools_dir));
        }

        // XDG path
        if let Some(data) = dirs::data_dir() {
            dirs_list.push(data.join("Steam").join("compatibilitytools.d"));
            dirs_list.push(data.join("mythix-launcher").join("compatibilitytools.d"));
        }

        // Legacy ~/.steam path
        if let Some(home) = dirs::home_dir() {
            dirs_list.push(home.join(".steam").join("root").join("compatibilitytools.d"));
            dirs_list.push(home.join(".steam").join("steam").join("compatibilitytools.d"));
        }

        dirs_list
    };

    for dir in &search_dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(dir) else { continue };

        for entry in entries.flatten() {
            let mut path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Canonicalize to resolve symlinks and ensure uniqueness
            if let Ok(canonical) = fs::canonicalize(&path) {
                path = canonical;
            }

            if seen_paths.contains(&path) {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Detect kind from directory contents and name
            let kind = detect_tool_kind(&path, &name);

            // Only include tools we can actually use
            if kind != "unknown" {
                seen_paths.insert(path.clone());
                tools.push(CompatTool { name, path, kind: kind.to_string() });
            } else {
                // Container dir (e.g. compatibilitytools-files/) — check one
                // level deeper for nested tools.
                let Ok(sub_entries) = fs::read_dir(&path) else { continue };
                for sub in sub_entries.flatten() {
                    let mut sub_path = sub.path();
                    if !sub_path.is_dir() { continue; }
                    if let Ok(canonical) = fs::canonicalize(&sub_path) {
                        sub_path = canonical;
                    }
                    if seen_paths.contains(&sub_path) { continue; }
                    let sub_name = sub_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let sub_kind = detect_tool_kind(&sub_path, &sub_name);
                    if sub_kind != "unknown" {
                        seen_paths.insert(sub_path.clone());
                        tools.push(CompatTool { name: sub_name, path: sub_path, kind: sub_kind.to_string() });
                    }
                }
            }
        }
    }

    // Sort by name so they appear in a nice alphabetical order in the UI!
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

fn detect_tool_kind(path: &Path, name: &str) -> &'static str {
    let has_proton_script = path.join("proton").is_file();
    let has_neutron_script = neutron_script(path).is_some();
    let has_toolmanifest = path.join("toolmanifest.vdf").is_file();
    let has_wine_bin = path.join("bin").join("wine").is_file()
        || path.join("wine").is_file();
    let _has_files_dir = path.join("files").is_dir();

    let name_lower = name.to_lowercase();

    // Neutron: has a `neutron` launcher script (built by mythix-neutron_builder)
    let is_neutron = has_neutron_script
        || (name_lower.contains("neutron") && has_toolmanifest);

    // Detect hybrid tools by name. Accept both "mythix" and the legacy "looni"
    // so tools built before the rebrand are still recognised.
    let is_hybrid = name_lower.contains("hybrid")
        || name_lower.contains("mythix")
        || name_lower.contains("looni")
        || (has_proton_script && has_wine_bin);

    if is_neutron {
        "neutron"
    } else if is_hybrid {
        "wine-proton-hybrid"
    } else if has_proton_script && has_toolmanifest {
        "proton"
    } else if has_wine_bin {
        "wine"
    } else {
        "unknown"
    }
}
