use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use crate::error::LauncherError;
use crate::game_library::{self, CompatTool, Game, GameConfig, scan_compat_tools, validate_config};
use crate::launcher::{self, LaunchEnv, LaunchResult};
use crate::runtime::{self, all_runtime_info, ensure_shim, RuntimeInfo, RuntimeVariant, install_runtime};
use crate::steam_import::{self, SteamGame};

#[tauri::command]
pub fn get_games() -> Result<Vec<Game>, String> {
    game_library::load_library().map(|l| l.games).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_game(name: String, exe_path: String) -> Result<Game, String> {
    let path = PathBuf::from(&exe_path);
    if !path.is_file() {
        return Err(format!("Executable not found: {exe_path}"));
    }
    game_library::add_game(name, path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_game(game_id: String) -> Result<(), String> {
    game_library::remove_game(&game_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_game_config(game_id: String, config: GameConfig) -> Result<(), String> {
    game_library::update_game_config(&game_id, config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_game_meta(
    game_id: String,
    name: Option<String>,
    cover_art: Option<String>,
    notes: Option<String>,
    exe_path: Option<String>,
) -> Result<(), String> {
    game_library::update_game_meta(
        &game_id,
        name,
        cover_art.map(PathBuf::from),
        notes,
        exe_path.map(PathBuf::from),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_compat_tools() -> Vec<CompatTool> {
    scan_compat_tools()
}

#[tauri::command]
pub fn get_runtime_status() -> Vec<RuntimeInfo> {
    all_runtime_info()
}

#[tauri::command]
pub async fn install_runtime_cmd(variant: String, app: AppHandle) -> Result<(), String> {
    let rv = variant_from_str(&variant).map_err(|e| e.to_string())?;
    install_runtime(rv, app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_runtime_update(variant: String) -> Result<String, String> {
    let rv = variant_from_str(&variant).map_err(|e| e.to_string())?;
    let status = runtime::check_for_update(rv).await;
    serde_json::to_string(&status).map_err(|e| e.to_string())
}

fn variant_from_str(s: &str) -> Result<RuntimeVariant, LauncherError> {
    match s {
        "steamrt3" | "sniper"  => Ok(RuntimeVariant::Sniper),
        "steamrt2" | "soldier" => Ok(RuntimeVariant::Soldier),
        "steamrt4"             => Ok(RuntimeVariant::SteamRT4),
        _ => Err(LauncherError::InvalidConfig(format!("Unknown variant: {s}"))),
    }
}

#[tauri::command]
pub fn launch_game(game_id: String, app: AppHandle) -> Result<LaunchResult, String> {
    use std::io::Write;
    let mut log = std::fs::OpenOptions::new().create(true).append(true)
        .open("/tmp/mythix_launch.log").ok();
    macro_rules! llog {
        ($($arg:tt)*) => {{
            let msg = format!($($arg)*);
            eprintln!("{}", msg);
            if let Some(ref mut f) = log { let _ = writeln!(f, "{}", msg); }
        }};
    }

    llog!("\n[mythix] ──────────────────────────────────────");
    llog!("[mythix] launch_game called with id: {}", game_id);

    let lib = game_library::load_library().map_err(|e| {
        llog!("[mythix] LIBRARY LOAD FAILED: {}", e);
        e.to_string()
    })?;
    let game = lib.games.iter().find(|g| g.id == game_id)
        .ok_or_else(|| {
            let msg = format!("Game not found: {game_id}");
            llog!("[mythix] {}", msg);
            msg
        })?;

    llog!("[mythix] Launching: {}", game.name);
    llog!("[mythix] Exe: {}", game.exe_path.display());

    validate_config(game).map_err(|e| {
        llog!("[mythix] VALIDATION FAILED: {}", e);
        e.to_string()
    })?;

    let proton_path = game.config.proton_path.as_ref()
        .ok_or_else(|| {
            llog!("[mythix] ERROR: No Proton/Neutron/Wine path configured");
            "No Proton/Wine path configured".to_string()
        })?;

    llog!("[mythix] Tool path: {}", proton_path.display());

    let tool = scan_compat_tools().into_iter().find(|t| &t.path == proton_path)
        .unwrap_or_else(|| {
            let kind = if proton_path.join("neutron").is_file() { "neutron" }
                else if proton_path.join("proton").is_file() { "proton" }
                else { "wine" };
            llog!("[mythix] Tool not in scan results, using fallback kind: {}", kind);
            CompatTool {
                name: proton_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").into(),
                path: proton_path.clone(),
                kind: kind.into(),
            }
        });

    llog!("[mythix] Tool: {} (kind: {})", tool.name, tool.kind);

    ensure_shim().ok();
    let env = launcher::build_launch_env(game, &tool).map_err(|e| {
        llog!("[mythix] ENV BUILD FAILED: {}", e);
        e.to_string()
    })?;

    llog!("[mythix] Command: {:?}", env.command);
    llog!("[mythix] Spawning...");

    launcher::spawn_game(game, &env, &app).map_err(|e| {
        llog!("[mythix] SPAWN FAILED: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub fn kill_game(pid: u32, game_id: Option<String>) -> Result<(), String> {
    eprintln!("[mythix] kill_game called for PID {} game {:?}", pid, game_id);

    // Look up the game's prefix so we can kill the wineserver
    let prefix_path = game_id.and_then(|gid| {
        game_library::load_library().ok()
            .and_then(|lib| lib.games.into_iter().find(|g| g.id == gid))
            .and_then(|g| g.config.prefix_path)
    });

    // 1. Try killing the PID and its tree (may already be dead from setsid --fork)
    let ps = pid.to_string();
    let tree = collect_tree(pid);
    for &child in &tree {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &child.to_string()])
            .status();
    }
    let _ = std::process::Command::new("kill")
        .args(["-TERM", &ps])
        .status();
    let _ = std::process::Command::new("kill")
        .args(["-TERM", "--", &format!("-{}", ps)])
        .status();

    // 2. Kill the wineserver for this prefix — this is the reliable way to
    //    kill Wine/Proton games since setsid --fork orphans the process tree
    if let Some(ref pfx) = prefix_path {
        let wineprefix = pfx.join("pfx");
        eprintln!("[mythix] Killing wineserver for prefix: {}", wineprefix.display());

        // Find wineserver — check common locations
        let proton_path = game_library::load_library().ok()
            .and_then(|lib| lib.games.iter()
                .find(|g| g.config.prefix_path.as_ref() == Some(pfx))
                .and_then(|g| g.config.proton_path.clone()));

        let wineserver_paths: Vec<PathBuf> = [
            proton_path.as_ref().map(|p| p.join("files").join("bin").join("wineserver")),
            proton_path.as_ref().map(|p| p.join("dist").join("bin").join("wineserver")),
            proton_path.as_ref().map(|p| p.join("bin").join("wineserver")),
            Some(PathBuf::from("/usr/bin/wineserver")),
        ].into_iter().flatten().filter(|p| p.exists()).collect();

        for ws in &wineserver_paths {
            eprintln!("[mythix] Trying wineserver -k via {}", ws.display());
            let _ = std::process::Command::new(ws)
                .arg("-k")
                .env("WINEPREFIX", &wineprefix)
                .status();
            break;
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    // 3. SIGKILL anything that survived — find by prefix env
    if let Some(ref pfx) = prefix_path {
        let pfx_str = pfx.join("pfx").to_string_lossy().to_string();
        // Find all processes with this WINEPREFIX
        if let Ok(out) = std::process::Command::new("bash")
            .args(["-c", &format!(
                "grep -rl '{}' /proc/*/environ 2>/dev/null | grep -oP '/proc/\\K[0-9]+' | sort -u",
                pfx_str
            )])
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Ok(p) = line.trim().parse::<u32>() {
                    if p != std::process::id() {
                        eprintln!("[mythix] Killing leftover PID {} (matched prefix)", p);
                        let _ = std::process::Command::new("kill")
                            .args(["-KILL", &p.to_string()])
                            .status();
                    }
                }
            }
        }

        // Also kill wineserver with SIGKILL
        for ws_path in [
            prefix_path.as_ref().and_then(|_| Some(PathBuf::from("/usr/bin/wineserver"))),
        ].into_iter().flatten().filter(|p| p.exists()) {
            let _ = std::process::Command::new(&ws_path)
                .arg("-k9")
                .env("WINEPREFIX", pfx.join("pfx"))
                .status();
        }
    }

    // 4. Last resort: SIGKILL the original PID tree
    let survivors = collect_tree(pid);
    for &p in &survivors {
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &p.to_string()])
            .status();
    }
    let _ = std::process::Command::new("kill")
        .args(["-KILL", "--", &format!("-{}", ps)])
        .status();

    eprintln!("[mythix] Kill sequence complete");
    Ok(())
}

fn collect_tree(pid: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let mut stack = vec![pid];
    while let Some(parent) = stack.pop() {
        if let Ok(out) = std::process::Command::new("pgrep")
            .args(["-P", &parent.to_string()])
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Ok(child_pid) = line.trim().parse::<u32>() {
                    result.push(child_pid);
                    stack.push(child_pid);
                }
            }
        }
    }
    result
}

#[tauri::command]
pub fn dry_run_launch(game_id: String) -> Result<LaunchEnv, String> {
    let lib = game_library::load_library().map_err(|e| e.to_string())?;
    let game = lib.games.iter().find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game not found: {game_id}"))?;
    let proton_path = game.config.proton_path.as_ref()
        .ok_or_else(|| "No Proton/Wine path configured".to_string())?;
    let tool = CompatTool {
        name: proton_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").into(),
        path: proton_path.clone(),
        kind: "proton".into(),
    };
    launcher::build_launch_env(game, &tool).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setup_prefix(path: String) -> Result<(), String> {
    launcher::setup_pfx(std::path::Path::new(&path)).map_err(|e| e.to_string())
}

// ── Steam Import ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn scan_steam() -> Result<Vec<SteamGame>, String> {
    let lib = game_library::load_library().map_err(|e| e.to_string())?;
    let existing_ids: HashSet<String> = lib.games.iter()
        .map(|g| g.config.game_id.clone())
        .collect();
    steam_import::scan_steam_games(&existing_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_steam_game(
    appid: String,
    name: String,
    install_path: String,
    steam_prefix: Option<String>,
    proton_path: Option<String>,
    app: AppHandle,
) -> Result<Game, String> {
    // Run the heavy work (cp -a of potentially multi-GB prefix) on a blocking thread
    // so we don't freeze the UI
    tokio::task::spawn_blocking(move || {
        let game_id = format!("mythix-{}", appid);

        // Auto-detect exe
        let exe_path = steam_import::find_game_exe(&install_path)
            .unwrap_or_else(|| format!("{}/game.exe", install_path));

        // Clone prefix if one exists
        let prefix_path = if let Some(ref sp) = steam_prefix {
            let clean_name = name.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect::<String>()
                .replace("__", "_")
                .trim_matches('_')
                .to_string();

            eprintln!("[mythix] Cloning prefix for {}...", name);
            let _ = app.emit("import:status", format!("Cloning prefix for {}…", name));

            match steam_import::clone_steam_prefix(sp, &clean_name) {
                Ok(result) => {
                    eprintln!("[mythix] Cloned prefix to {} ({} bytes)", result.prefix_path, result.size_bytes);
                    let _ = app.emit("import:status", format!("Prefix cloned ({})", format_bytes(result.size_bytes)));
                    Some(PathBuf::from(result.prefix_path))
                }
                Err(e) => {
                    eprintln!("[mythix] Prefix clone failed: {}, creating fresh", e);
                    let _ = app.emit("import:status", "Prefix clone failed, creating fresh…".to_string());
                    let fallback = crate::paths::default_prefix_dir(&game_id)
                        .unwrap_or_else(|| PathBuf::from("/tmp").join("gamez-pfx_data").join(&game_id));
                    Some(fallback)
                }
            }
        } else {
            let p = crate::paths::default_prefix_dir(&game_id)
                .unwrap_or_else(|| PathBuf::from("/tmp").join("gamez-pfx_data").join(&game_id));
            Some(p)
        };

        let _ = app.emit("import:status", "Saving to library…".to_string());

        // Add to library
        let mut lib = game_library::load_library().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();

        let config = GameConfig {
            prefix_path,
            proton_path: proton_path.map(PathBuf::from),
            store: "steam".into(),
            game_id,
            env_overrides: std::collections::HashMap::new(),
            launch_args: Vec::new(),
            proton_verb: "waitforexitandrun".to_string(),
            prefix_type: game_library::PrefixType::Proton,
            use_runtime: None,
            runtime_variant: None,
        };

        let game = Game {
            id: id.clone(),
            name,
            exe_path: PathBuf::from(exe_path),
            cover_art: None,
            config,
            added_at: format!("unix:{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs()),
            playtime_secs: 0,
            notes: String::new(),
        };

        lib.games.push(game.clone());
        game_library::save_library(&lib).map_err(|e| e.to_string())?;

        let _ = app.emit("import:status", "Done!".to_string());
        Ok(game)
    })
    .await
    .map_err(|e| format!("Import thread panicked: {e}"))?
}

fn format_bytes(bytes: u64) -> String {
    let mut val = bytes as f64;
    for unit in &["B", "KB", "MB", "GB", "TB"] {
        if val < 1024.0 { return format!("{:.1} {}", val, unit); }
        val /= 1024.0;
    }
    format!("{:.1} PB", val)
}

#[tauri::command]
pub fn find_exe(install_path: String) -> Option<String> {
    steam_import::find_game_exe(&install_path)
}

// ── Cover art ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_cover(game_id: String) -> Result<Game, String> {
    let lib = game_library::load_library().map_err(|e| e.to_string())?;
    let game = lib.games.iter().find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game {game_id} not found"))?
        .clone();

    let path = crate::coverart::fetch_cover(
        &game.config.game_id,
        &game.id,
    ).await.map_err(|e| e.to_string())?;

    let path_str = path.to_string_lossy().to_string();
    game_library::update_game_meta(&game.id, None, Some(path), None, None)
        .map_err(|e| e.to_string())?;

    let lib2 = game_library::load_library().map_err(|e| e.to_string())?;
    lib2.games.into_iter().find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game vanished after update: {path_str}"))
}

#[tauri::command]
pub fn import_cover(game_id: String, image_path: String) -> Result<Game, String> {
    let lib = game_library::load_library().map_err(|e| e.to_string())?;
    let game = lib.games.iter().find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game {game_id} not found"))?
        .clone();

    let path = crate::coverart::import_cover(
        std::path::Path::new(&image_path),
        &game.id,
    ).map_err(|e| e.to_string())?;

    game_library::update_game_meta(&game.id, None, Some(path), None, None)
        .map_err(|e| e.to_string())?;

    let lib2 = game_library::load_library().map_err(|e| e.to_string())?;
    lib2.games.into_iter().find(|g| g.id == game_id)
        .ok_or_else(|| "Game vanished after update".to_string())
}

#[tauri::command]
pub fn clear_cover(game_id: String) -> Result<(), String> {
    crate::coverart::remove_cover(&game_id).map_err(|e| e.to_string())?;
    let mut lib = game_library::load_library().map_err(|e| e.to_string())?;
    let g = lib.games.iter_mut().find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game {game_id} not found"))?;
    g.cover_art = None;
    game_library::save_library(&lib).map_err(|e| e.to_string())
}

// ── Global settings ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_settings() -> crate::settings::GlobalSettings {
    crate::settings::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: crate::settings::GlobalSettings) -> Result<(), String> {
    crate::settings::save_settings(&settings).map_err(|e| e.to_string())
}

// ── Window controls ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn window_minimize(window: tauri::Window) { window.minimize().ok(); }

#[tauri::command]
pub fn window_maximize(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) { window.unmaximize().ok(); }
    else { window.maximize().ok(); }
}

#[tauri::command]
pub fn window_close(window: tauri::Window) { window.close().ok(); }

#[tauri::command]
pub fn window_hide(window: tauri::Window) {
    crate::gamepad::notify_hidden();
    window.hide().ok();
}

#[tauri::command]
pub fn window_show(window: tauri::Window) {
    window.show().ok();
    window.unminimize().ok();
    window.set_focus().ok();
}
