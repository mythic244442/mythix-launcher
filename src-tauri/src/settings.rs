use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    #[serde(default = "default_prefix_root")]
    pub default_prefix_root: String,

    #[serde(default = "default_proton_path")]
    pub default_proton_path: String,

    #[serde(default = "default_proton_verb")]
    pub default_proton_verb: String,

    #[serde(default = "default_library_view")]
    pub library_view: String,

    #[serde(default)]
    pub close_to_tray: bool,

    #[serde(default = "default_true")]
    pub auto_setup_prefix: bool,

    #[serde(default)]
    pub global_env: HashMap<String, String>,

    #[serde(default)]
    pub custom_tools_dir: String,
}

fn default_prefix_root() -> String {
    dirs::home_dir()
        .map(|h| h.join(".mythix").join("gamedata").to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn default_proton_path() -> String { String::new() }
fn default_proton_verb() -> String { "waitforexitandrun".into() }
fn default_library_view() -> String { "grid".into() }
fn default_true() -> bool { true }

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_prefix_root: default_prefix_root(),
            default_proton_path: default_proton_path(),
            default_proton_verb: default_proton_verb(),
            library_view: default_library_view(),
            close_to_tray: false,
            auto_setup_prefix: true,
            global_env: HashMap::new(),
            custom_tools_dir: String::new(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    crate::paths::data_dir().map(|d| d.join("settings.json"))
}

pub fn load_settings() -> GlobalSettings {
    let Some(path) = settings_path() else { return GlobalSettings::default() };
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => GlobalSettings::default(),
    }
}

pub fn save_settings(settings: &GlobalSettings) -> Result<(), crate::error::LauncherError> {
    let Some(path) = settings_path() else {
        return Err(crate::error::LauncherError::InvalidConfig("Cannot resolve data dir".into()));
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| crate::error::LauncherError::InvalidConfig(e.to_string()))?;
    std::fs::write(&path, json)?;
    Ok(())
}
