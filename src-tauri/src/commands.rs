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
    let mut games = steam_import::scan_steam_games(&existing_ids).map_err(|e| e.to_string())?;
    // Library state per game, so the Import tab can offer Prefix and Loader on
    // their own: does the entry launch through a loader profile, and does its
    // prefix exist yet?
    for sg in games.iter_mut().filter(|g| g.already_imported) {
        if let Some(idx) = library_index(&lib, &sg.appid) {
            sg.uses_loader = crate::loader::is_loader_stub(&lib.games[idx].exe_path);
            sg.prefix_ready = prefix_exists(&lib.games[idx]);
        }
    }
    Ok(games)
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

        // Already in the library (e.g. added through the loader first)? Then this
        // just gives that entry Steam's prefix instead of adding a duplicate.
        if game_library::load_library()
            .map(|lib| library_index(&lib, &appid).is_some())
            .unwrap_or(false)
        {
            return attach_steam_prefix(&appid, &name, steam_prefix.as_deref(), &app);
        }

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
                        .unwrap_or_else(|| PathBuf::from("/tmp").join("gamedata").join(&game_id));
                    Some(fallback)
                }
            }
        } else {
            let p = crate::paths::default_prefix_dir(&game_id)
                .unwrap_or_else(|| PathBuf::from("/tmp").join("gamedata").join(&game_id));
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
            wrappers: Vec::new(),
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

// ── Custom Import (any folder, not just Steam) ────────────────────────────────

/// Import a game from an arbitrary install folder. Resolves the exe (the one the
/// user picked, else auto-detected), creates a fresh default prefix (made on
/// first launch, like a Steam import with no prefix), and attaches the chosen
/// compat tool. No files are moved — the game launches from where it lives.
#[tauri::command]
pub async fn import_custom_game(
    name: String,
    install_path: String,
    exe_path: Option<String>,
    proton_path: Option<String>,
    app: AppHandle,
) -> Result<Game, String> {
    tokio::task::spawn_blocking(move || {
        let _ = app.emit("import:status", format!("Importing {name}…"));

        // Use the exe the user picked, otherwise auto-detect one under the folder.
        let exe = match exe_path {
            Some(e) if !e.trim().is_empty() => e,
            _ => steam_import::find_game_exe(&install_path)
                .ok_or_else(|| format!("No .exe found under {install_path}"))?,
        };
        if !PathBuf::from(&exe).is_file() {
            return Err(format!("Executable not found: {exe}"));
        }

        let settings = crate::settings::load_settings();
        let id = uuid::Uuid::new_v4().to_string();
        let game_id = id.replace('-', "")[..8].to_string();

        // Fresh prefix — created on first launch, mirroring a prefix-less import.
        let prefix_path = if !settings.default_prefix_root.is_empty() {
            Some(PathBuf::from(&settings.default_prefix_root).join(&game_id))
        } else {
            crate::paths::default_prefix_dir(&game_id)
        };

        // Compat tool: the one chosen in the UI, else the configured default.
        let proton = proton_path
            .filter(|p| !p.trim().is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                if settings.default_proton_path.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&settings.default_proton_path))
                }
            });

        let config = GameConfig {
            prefix_path,
            proton_path: proton,
            store: "custom".into(),
            game_id,
            env_overrides: std::collections::HashMap::new(),
            launch_args: Vec::new(),
            wrappers: Vec::new(),
            proton_verb: settings.default_proton_verb.clone(),
            prefix_type: game_library::PrefixType::Proton,
            use_runtime: None,
            runtime_variant: None,
        };

        let mut lib = game_library::load_library().map_err(|e| e.to_string())?;
        let game = Game {
            id: id.clone(),
            name,
            exe_path: PathBuf::from(exe),
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

// ── Loader import (Steam games via mythix-loader profiles) ───────────────────
//
// Prefix and Loader are separate, composable actions in the Import tab:
//  • Prefix (import_steam_game) clones Steam's compatdata prefix. It adds the game
//    when it isn't in the library yet, or gives the existing entry Steam's prefix.
//  • Loader (import_loader_game) builds the mythix-loader profile and never touches
//    prefixes: an existing entry keeps its prefix and starts launching through the
//    loader; a new entry gets a fresh prefix.

/// Index of the library entry for a Steam appid (new `mythix-` or legacy `looni-` id).
fn library_index(lib: &game_library::Library, appid: &str) -> Option<usize> {
    let (a, b) = (format!("mythix-{appid}"), format!("looni-{appid}"));
    lib.games.iter().position(|g| g.config.game_id == a || g.config.game_id == b)
}

/// Whether a library entry's prefix already exists on disk.
fn prefix_exists(game: &Game) -> bool {
    game.config.prefix_path.as_ref().map(|p| p.join("pfx").is_dir()).unwrap_or(false)
}

fn clean_prefix_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>()
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

/// Give an existing library entry a clone of Steam's compatdata prefix. Refuses to
/// replace a prefix that already exists on disk.
fn attach_steam_prefix(
    appid: &str,
    name: &str,
    steam_prefix: Option<&str>,
    app: &AppHandle,
) -> Result<Game, String> {
    let sp = steam_prefix
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Steam has no prefix for {name} to import."))?;
    {
        let lib = game_library::load_library().map_err(|e| e.to_string())?;
        let idx = library_index(&lib, appid)
            .ok_or_else(|| format!("{name} isn't in your library."))?;
        if prefix_exists(&lib.games[idx]) {
            return Err(format!(
                "{} already has a prefix — remove the game first if you want Steam's instead.",
                lib.games[idx].name
            ));
        }
    }

    let _ = app.emit("import:status", format!("Cloning prefix for {name}…"));
    let cloned = steam_import::clone_steam_prefix(sp, &clean_prefix_name(name))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("import:status", format!("Prefix cloned ({})", format_bytes(cloned.size_bytes)));

    // Re-read: the clone can take a while.
    let mut lib = game_library::load_library().map_err(|e| e.to_string())?;
    let idx = library_index(&lib, appid)
        .ok_or_else(|| format!("{name} left the library during the import."))?;
    lib.games[idx].config.prefix_path = Some(PathBuf::from(cloned.prefix_path));
    let game = lib.games[idx].clone();
    game_library::save_library(&lib).map_err(|e| e.to_string())?;
    let _ = app.emit("import:status", "Done!".to_string());
    Ok(game)
}

/// Plan a loader import without writing anything: appid, name, the exe from
/// Steam's own config (plus its launch options, for information), and whether the
/// game is already in the library.
#[tauri::command]
pub async fn preview_loader_import(
    install_path: String,
    appid: Option<String>,
) -> Result<crate::loader::LoaderPlan, String> {
    tokio::task::spawn_blocking(move || {
        let mut plan = crate::loader::run_loader(&install_path, appid.as_deref(), true)
            .map_err(|e| e.to_string())?;
        if let Ok(lib) = game_library::load_library() {
            if let Some(idx) = library_index(&lib, &plan.appid) {
                plan.in_library = true;
                plan.uses_loader = crate::loader::is_loader_stub(&lib.games[idx].exe_path);
            }
        }
        Ok(plan)
    })
    .await
    .map_err(|e| format!("Loader preview panicked: {e}"))?
}

/// Build (or rebuild) a mythix-loader profile in ~/.mythix/loader_profiles/ and
/// point the library at its stub. Never imports or clones a prefix: a game that's
/// already in the library keeps its prefix, tool, launch args and name; a new one
/// gets a fresh prefix. The stub starts the real exe from its install folder with
/// the real steamclient, so Steam must be running.
#[tauri::command]
pub async fn import_loader_game(
    install_path: String,
    appid: Option<String>,
    name: Option<String>,
    proton_path: Option<String>,
    app: AppHandle,
) -> Result<Game, String> {
    tokio::task::spawn_blocking(move || {
        let _ = app.emit("import:status", "Building loader profile…".to_string());
        let plan = crate::loader::run_loader(&install_path, appid.as_deref(), false)
            .map_err(|e| e.to_string())?;
        if plan.appid.is_empty()
            || plan.appid == "0"
            || !plan.appid.chars().all(|c| c.is_ascii_digit())
        {
            return Err("Couldn't find this game's Steam appid — enter it manually.".into());
        }

        let settings = crate::settings::load_settings();
        let chosen_tool = proton_path
            .filter(|p| !p.trim().is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                if settings.default_proton_path.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&settings.default_proton_path))
                }
            });

        let mut lib = game_library::load_library().map_err(|e| e.to_string())?;
        let game = apply_loader_profile(&mut lib, &plan, name, chosen_tool, &settings.default_proton_verb);
        game_library::save_library(&lib).map_err(|e| e.to_string())?;

        let _ = app.emit("import:status", "Done!".to_string());
        Ok(game)
    })
    .await
    .map_err(|e| format!("Import thread panicked: {e}"))?
}

/// Point the library at a loader profile (the pure part of `import_loader_game`).
/// A game that's already in the library switches to the stub and keeps everything
/// else — prefix, launch args, name, notes, playtime; only a missing tool is filled
/// in. Otherwise a new entry with a fresh prefix is added. Returns the entry.
fn apply_loader_profile(
    lib: &mut game_library::Library,
    plan: &crate::loader::LoaderPlan,
    name: Option<String>,
    chosen_tool: Option<PathBuf>,
    proton_verb: &str,
) -> Game {
    if let Some(idx) = library_index(lib, &plan.appid) {
        let entry = &mut lib.games[idx];
        entry.exe_path = PathBuf::from(&plan.stub_path);
        // The stub lives in the profile dir, so point Neutron at the game's real
        // install folder (this also shares it into runtime containers).
        entry.config.env_overrides
            .insert("STEAM_COMPAT_INSTALL_PATH".to_string(), plan.game_dir.clone());
        if entry.config.proton_path.is_none() {
            entry.config.proton_path = chosen_tool;
        }
        return entry.clone();
    }

    // New entry with a fresh prefix (made on first launch). Prefix in the Steam
    // list can give it a copy of Steam's instead, before it's first launched.
    let game_id = format!("mythix-{}", plan.appid);
    let prefix_path = crate::paths::default_prefix_dir(&game_id)
        .unwrap_or_else(|| PathBuf::from("/tmp").join("gamedata").join(&game_id));
    let display_name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| plan.name.clone());
    let mut env_overrides = std::collections::HashMap::new();
    env_overrides.insert("STEAM_COMPAT_INSTALL_PATH".to_string(), plan.game_dir.clone());

    let game = Game {
        id: uuid::Uuid::new_v4().to_string(),
        name: display_name,
        exe_path: PathBuf::from(&plan.stub_path),
        cover_art: None,
        config: GameConfig {
            prefix_path: Some(prefix_path),
            proton_path: chosen_tool,
            store: "steam".into(),
            game_id,
            env_overrides,
            launch_args: Vec::new(),
            wrappers: Vec::new(),
            proton_verb: proton_verb.to_string(),
            prefix_type: game_library::PrefixType::Proton,
            use_runtime: None,
            runtime_variant: None,
        },
        added_at: format!("unix:{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs()),
        playtime_secs: 0,
        notes: format!("Loader profile: {}\nLaunches: {}", plan.profile_dir, plan.exe),
    };
    lib.games.push(game.clone());
    game
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
pub fn get_controller_type() -> String {
    crate::gamepad::current_type().as_str().to_string()
}

#[derive(serde::Serialize)]
pub struct ControllerStatus {
    pub connected: usize,
    #[serde(rename = "type")]
    pub controller_type: String,
}

/// Connection count + glyph family, for the titlebar indicator (WebKitGTK's
/// Gamepad API only reports pads after a button press, so it can't be trusted).
#[tauri::command]
pub fn get_controller_status() -> ControllerStatus {
    ControllerStatus {
        connected: crate::gamepad::connected_count(),
        controller_type: crate::gamepad::current_type().as_str().to_string(),
    }
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

#[tauri::command]
pub fn check_hidraw_access() -> bool {
    use std::fs;
    // Check if hidraw devices have uaccess ACLs or group permissions
    // (the Tauri process may run as root, so check the actual file mode + ACL)
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("hidraw") {
                let path = entry.path();
                if let Ok(meta) = fs::metadata(&path) {
                    use std::os::unix::fs::MetadataExt;
                    let mode = meta.mode();
                    // Check if group or other has read access, or if uaccess ACL is set
                    if mode & 0o044 != 0 {
                        return true;
                    }
                    // Check for ACL (getfacl presence = uaccess rule applied)
                    if let Ok(out) = std::process::Command::new("getfacl")
                        .arg("-p").arg(&path)
                        .output()
                    {
                        let acl = String::from_utf8_lossy(&out.stdout);
                        if acl.contains("user:") && acl.lines().any(|l|
                            l.starts_with("user:") && !l.starts_with("user::") && l.contains("rw"))
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[tauri::command]
pub fn install_udev_rules() -> Result<String, String> {
    let rules_src = include_str!("../udev/71-mythix-controllers.rules");
    let dest = "/etc/udev/rules.d/71-mythix-controllers.rules";

    let status = std::process::Command::new("pkexec")
        .args(["bash", "-c", &format!(
            "echo '{}' > {} && udevadm control --reload-rules && udevadm trigger",
            rules_src.replace('\'', "'\\''"), dest
        )])
        .status()
        .map_err(|e| format!("Failed to run pkexec: {}", e))?;

    if status.success() {
        Ok("Udev rules installed. Reconnect your controller.".into())
    } else {
        Err("Failed to install udev rules (authentication cancelled?)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::LoaderPlan;

    fn entry(game_id: &str, exe: &str, prefix: Option<&str>) -> Game {
        Game {
            id: "id-1".into(),
            name: "My Ready or Not".into(),
            exe_path: PathBuf::from(exe),
            cover_art: None,
            config: GameConfig {
                prefix_path: prefix.map(PathBuf::from),
                proton_path: Some(PathBuf::from("/tools/Neutron")),
                game_id: game_id.into(),
                launch_args: vec!["-dx11".into()],
                ..GameConfig::default()
            },
            added_at: "unix:0".into(),
            playtime_secs: 42,
            notes: "my notes".into(),
        }
    }

    fn plan() -> LoaderPlan {
        LoaderPlan {
            appid: "1144200".into(),
            name: "Ready or Not".into(),
            game_dir: "/lib/steamapps/common/Ready Or Not".into(),
            exe: "/lib/steamapps/common/Ready Or Not/ReadyOrNot.exe".into(),
            profile_dir: "/home/u/.mythix/loader_profiles/ready-or-not".into(),
            stub_path: "/home/u/.mythix/loader_profiles/ready-or-not/readyornot.exe".into(),
            ..Default::default()
        }
    }

    fn library(games: Vec<Game>) -> game_library::Library {
        game_library::Library { version: 1, games }
    }

    #[test]
    fn loader_keeps_an_existing_entrys_prefix_and_settings() {
        let pfx = "/home/u/.mythix/gamedata/ready_or_not";
        let mut lib = library(vec![entry("mythix-1144200", "/lib/RoN/ReadyOrNot.exe", Some(pfx))]);
        let g = apply_loader_profile(&mut lib, &plan(), Some("ignored".into()),
            Some(PathBuf::from("/tools/Other")), "waitforexitandrun");

        assert_eq!(lib.games.len(), 1, "no duplicate entry");
        assert_eq!(g.exe_path, PathBuf::from(plan().stub_path), "now launches the stub");
        assert_eq!(g.config.prefix_path, Some(PathBuf::from(pfx)), "prefix untouched");
        assert_eq!(g.config.launch_args, vec!["-dx11".to_string()], "launch args kept");
        assert_eq!(g.config.proton_path, Some(PathBuf::from("/tools/Neutron")), "tool kept");
        assert_eq!((g.name.as_str(), g.notes.as_str(), g.playtime_secs),
                   ("My Ready or Not", "my notes", 42), "name/notes/playtime kept");
        assert_eq!(g.config.env_overrides.get("STEAM_COMPAT_INSTALL_PATH").map(String::as_str),
                   Some("/lib/steamapps/common/Ready Or Not"));
        assert_eq!(lib.games[0].exe_path, g.exe_path, "library updated in place");
    }

    #[test]
    fn loader_fills_in_a_missing_tool_only() {
        let mut e = entry("mythix-1144200", "/x.exe", None);
        e.config.proton_path = None;
        let mut lib = library(vec![e]);
        let g = apply_loader_profile(&mut lib, &plan(), None,
            Some(PathBuf::from("/tools/Neutron-11")), "waitforexitandrun");
        assert_eq!(g.config.proton_path, Some(PathBuf::from("/tools/Neutron-11")));
        assert_eq!(g.config.prefix_path, None, "still no prefix forced on it");
    }

    #[test]
    fn loader_matches_legacy_looni_ids() {
        let mut lib = library(vec![entry("looni-1144200", "/x.exe", None)]);
        apply_loader_profile(&mut lib, &plan(), None, None, "waitforexitandrun");
        assert_eq!(lib.games.len(), 1);
        assert_eq!(lib.games[0].exe_path, PathBuf::from(plan().stub_path));
    }

    #[test]
    fn loader_adds_a_new_entry_with_a_fresh_prefix() {
        let mut lib = library(vec![entry("mythix-999", "/other.exe", Some("/pfx/other"))]);
        let g = apply_loader_profile(&mut lib, &plan(), None,
            Some(PathBuf::from("/tools/Neutron")), "waitforexitandrun");
        assert_eq!(lib.games.len(), 2);
        assert_eq!(g.config.game_id, "mythix-1144200");
        assert_eq!(g.config.store, "steam");
        assert_eq!(g.name, "Ready or Not");
        assert!(g.config.launch_args.is_empty());
        let pfx = g.config.prefix_path.clone().unwrap();
        assert!(pfx.ends_with("gamedata/mythix-1144200"), "fresh default prefix, not a clone: {pfx:?}");
        assert_eq!(lib.games[0].config.prefix_path, Some(PathBuf::from("/pfx/other")), "others untouched");
    }

    #[test]
    fn prefix_exists_only_when_the_pfx_dir_is_there() {
        let tmp = tempfile::tempdir().unwrap();
        let mut g = entry("mythix-1", "/x.exe", Some(tmp.path().to_str().unwrap()));
        assert!(!prefix_exists(&g));
        std::fs::create_dir(tmp.path().join("pfx")).unwrap();
        assert!(prefix_exists(&g));
        g.config.prefix_path = None;
        assert!(!prefix_exists(&g));
    }
}
