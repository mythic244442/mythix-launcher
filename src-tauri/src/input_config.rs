use std::path::Path;
use std::fs;
use crate::gamepad::{ControllerInfo, ControllerType, PlayStationModel};

#[derive(Debug)]
struct GlyphRule {
    steam_app_id: &'static str,
    name: &'static str,
    apply: fn(&ControllerInfo, &Path) -> Result<(), String>,
}

static GLYPH_RULES: &[GlyphRule] = &[
    GlyphRule {
        steam_app_id: "1091500",
        name: "Cyberpunk 2077",
        apply: apply_cyberpunk2077,
    },
    GlyphRule {
        steam_app_id: "1174180",
        name: "Red Dead Redemption 2",
        apply: apply_rdr2,
    },
    GlyphRule {
        steam_app_id: "1245620",
        name: "Elden Ring",
        apply: apply_elden_ring,
    },
    GlyphRule {
        steam_app_id: "292030",
        name: "The Witcher 3",
        apply: apply_witcher3,
    },
];

pub fn configure_game_glyphs(
    steam_app_id: &str,
    prefix_path: &Path,
    controller: &ControllerInfo,
) {
    if controller.controller_type == ControllerType::Xbox
        || controller.controller_type == ControllerType::Unknown
    {
        return;
    }

    for rule in GLYPH_RULES {
        if rule.steam_app_id == steam_app_id {
            eprintln!("[mythix] Applying {} glyph config for {}",
                      controller.controller_type.as_str(), rule.name);
            match (rule.apply)(controller, prefix_path) {
                Ok(()) => eprintln!("[mythix] Glyph config applied successfully"),
                Err(e) => eprintln!("[mythix] Glyph config failed: {}", e),
            }
            return;
        }
    }
}

fn glyph_index(info: &ControllerInfo) -> u32 {
    match info.controller_type {
        ControllerType::PlayStation => match info.ps_model {
            Some(PlayStationModel::DualSense | PlayStationModel::DualSenseEdge) => 3,
            _ => 2,
        },
        ControllerType::Switch => 4, // if the game supports it
        _ => 0,
    }
}

fn patch_json_int_field(
    content: &str,
    field_name: &str,
    value: u32,
) -> Option<String> {
    let field_pos = content.find(&format!("\"name\": \"{}\"", field_name))
        .or_else(|| content.find(&format!("\"name\":\"{}\"", field_name)))?;

    let after_field = &content[field_pos..];

    let value_pat = "\"value\":";
    let value_alt = "\"value\": ";
    let val_offset = after_field.find(value_pat)
        .map(|p| (p, value_pat.len()))
        .or_else(|| after_field.find(value_alt).map(|p| (p, value_alt.len())))?;

    let val_start = field_pos + val_offset.0 + val_offset.1;
    let val_rest = &content[val_start..];
    let val_end = val_rest.find(|c: char| c == ',' || c == '}' || c == '\n')?;
    let abs_val_end = val_start + val_end;

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..val_start]);
    result.push_str(&format!(" {}", value));
    result.push_str(&content[abs_val_end..]);

    let idx_search_start = abs_val_end;
    let remaining = &result[idx_search_start..];
    if let Some(idx_pos) = remaining.find("\"index\":").or_else(|| remaining.find("\"index\": ")) {
        let pat = if remaining[idx_pos..].starts_with("\"index\": ") { "\"index\": " } else { "\"index\":" };
        let idx_val_start = idx_search_start + idx_pos + pat.len();
        let idx_rest = &result[idx_val_start..];
        if let Some(idx_val_end) = idx_rest.find(|c: char| c == ',' || c == '}' || c == '\n') {
            let abs_idx_end = idx_val_start + idx_val_end;
            let mut final_result = String::with_capacity(result.len());
            final_result.push_str(&result[..idx_val_start]);
            final_result.push_str(&format!(" {}", value));
            final_result.push_str(&result[abs_idx_end..]);
            return Some(final_result);
        }
    }

    Some(result)
}

// ── Per-game glyph configuration functions ──────────────────────────────────

fn apply_cyberpunk2077(info: &ControllerInfo, prefix: &Path) -> Result<(), String> {
    let settings_path = prefix.join("pfx/drive_c/users")
        .join(find_user_dir(prefix))
        .join("AppData/Local/CD Projekt Red/Cyberpunk 2077/UserSettings.json");

    if !settings_path.exists() {
        return Err("UserSettings.json not found (game not run yet?)".into());
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("read: {}", e))?;

    // GamepadLayout: 0=Auto, 1=Xbox, 2=PS4, 3=PS5
    let idx = glyph_index(info);
    if let Some(patched) = patch_json_int_field(&content, "GamepadLayout", idx) {
        if patched != content {
            fs::write(&settings_path, &patched)
                .map_err(|e| format!("write: {}", e))?;
            eprintln!("[mythix] Cyberpunk 2077: GamepadLayout → {}", idx);
        }
    }
    Ok(())
}

fn apply_rdr2(info: &ControllerInfo, prefix: &Path) -> Result<(), String> {
    // RDR2 uses system.xml: <InputDevice value="2" /> for PS4
    let docs = prefix.join("pfx/drive_c/users")
        .join(find_user_dir(prefix))
        .join("Documents/Rockstar Games/Red Dead Redemption 2/Settings");

    let settings_path = docs.join("system.xml");
    if !settings_path.exists() {
        return Err("system.xml not found (game not run yet?)".into());
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("read: {}", e))?;

    // value="0" = auto, "1" = xbox, "2" = PS
    let target = match info.controller_type {
        ControllerType::PlayStation => "2",
        ControllerType::Switch => "2", // RDR2 has no switch option
        _ => return Ok(()),
    };

    if let Some(start) = content.find("<InputDevice") {
        if let Some(val_start) = content[start..].find("value=\"") {
            let abs = start + val_start + 7;
            if let Some(val_end) = content[abs..].find('"') {
                let current = &content[abs..abs+val_end];
                if current != target {
                    let mut patched = String::with_capacity(content.len());
                    patched.push_str(&content[..abs]);
                    patched.push_str(target);
                    patched.push_str(&content[abs+val_end..]);
                    fs::write(&settings_path, &patched)
                        .map_err(|e| format!("write: {}", e))?;
                    eprintln!("[mythix] RDR2: InputDevice → {}", target);
                }
            }
        }
    }
    Ok(())
}

fn apply_elden_ring(info: &ControllerInfo, prefix: &Path) -> Result<(), String> {
    // Elden Ring: GraphicsConfig.xml, <PadType>1</PadType> for PS
    let settings_path = prefix.join("pfx/drive_c/users")
        .join(find_user_dir(prefix))
        .join("AppData/Roaming/EldenRing");

    let Ok(entries) = fs::read_dir(&settings_path) else {
        return Err("EldenRing config dir not found".into());
    };

    for entry in entries.flatten() {
        let config = entry.path().join("GraphicsConfig.xml");
        if !config.exists() { continue; }

        let content = fs::read_to_string(&config)
            .map_err(|e| format!("read: {}", e))?;

        // 0 = auto/xbox, 1 = PS
        let target = match info.controller_type {
            ControllerType::PlayStation => "1",
            _ => return Ok(()),
        };

        let tag = "<PadType>";
        let end_tag = "</PadType>";
        if let Some(start) = content.find(tag) {
            let val_start = start + tag.len();
            if let Some(end) = content[val_start..].find(end_tag) {
                let current = &content[val_start..val_start+end];
                if current.trim() != target {
                    let mut patched = String::with_capacity(content.len());
                    patched.push_str(&content[..val_start]);
                    patched.push_str(target);
                    patched.push_str(&content[val_start+end..]);
                    fs::write(&config, &patched)
                        .map_err(|e| format!("write: {}", e))?;
                    eprintln!("[mythix] Elden Ring: PadType → {}", target);
                }
            }
        }
    }
    Ok(())
}

fn apply_witcher3(info: &ControllerInfo, prefix: &Path) -> Result<(), String> {
    // Witcher 3: user.settings, [Gameplay] GamepadType= (0=auto, 1=xbox, 2=ps4, 3=ps5)
    let settings_path = prefix.join("pfx/drive_c/users")
        .join(find_user_dir(prefix))
        .join("Documents/The Witcher 3");

    let config = settings_path.join("user.settings");
    if !config.exists() {
        return Err("user.settings not found".into());
    }

    let content = fs::read_to_string(&config)
        .map_err(|e| format!("read: {}", e))?;

    let target = glyph_index(info).to_string();

    if let Some(pos) = content.find("GamepadType=") {
        let val_start = pos + "GamepadType=".len();
        let val_end = content[val_start..].find('\n')
            .map(|e| val_start + e)
            .unwrap_or(content.len());
        let current = content[val_start..val_end].trim();
        if current != target {
            let mut patched = String::with_capacity(content.len());
            patched.push_str(&content[..val_start]);
            patched.push_str(&target);
            patched.push_str(&content[val_end..]);
            fs::write(&config, &patched)
                .map_err(|e| format!("write: {}", e))?;
            eprintln!("[mythix] Witcher 3: GamepadType → {}", target);
        }
    }
    Ok(())
}

fn find_user_dir(prefix: &Path) -> String {
    let users_dir = prefix.join("pfx/drive_c/users");
    let real_user = std::env::var("USER").unwrap_or_default();

    // Try real username first, then steamuser
    for candidate in &[real_user.as_str(), "steamuser"] {
        if !candidate.is_empty() && users_dir.join(candidate).is_dir() {
            return candidate.to_string();
        }
    }

    // Fall back to first non-Public directory
    if let Ok(entries) = fs::read_dir(&users_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "Public" && entry.path().is_dir() {
                return name;
            }
        }
    }

    real_user
}
