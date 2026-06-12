use tauri::Emitter;
use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader, Read};

use serde::{Deserialize, Serialize};

use crate::error::LauncherError;
use crate::game_library::{CompatTool, Game, PrefixType};
use crate::runtime::{CompatLayer, RuntimeVariant, is_runtime_installed, local_runtime_path};
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::symlink;
use std::collections::HashMap;

// ── Prefix setup (mirrors mythix's setup_pfx) ──────────────────────────────────

pub fn setup_pfx(prefix: &Path) -> Result<(), LauncherError> {
    // prefix = STEAM_COMPAT_DATA_PATH (the compat data directory)
    // prefix/pfx/ = the actual WINEPREFIX
    // This mirrors Proton/Neutron's expected layout.
    fs::create_dir_all(prefix)
        .map_err(|e| LauncherError::Prefix(format!("create prefix dir: {e}")))?;
    for subdir in &["shadercache", "gstreamer-1.0"] {
        fs::create_dir_all(prefix.join(subdir))
            .map_err(|e| LauncherError::Prefix(format!("create {subdir}: {e}")))?;
    }

    // pfx/ is the actual wineprefix directory — NOT a symlink to itself
    let pfx = prefix.join("pfx");
    if pfx.is_symlink() {
        // Clean up any self-referencing symlink from old code
        fs::remove_file(&pfx).map_err(|e| LauncherError::Prefix(e.to_string()))?;
    }
    if !pfx.is_dir() {
        fs::create_dir_all(&pfx)
            .map_err(|e| LauncherError::Prefix(format!("create pfx dir: {e}")))?;
    }

    let tracked = prefix.join("tracked_files");
    if !tracked.exists() {
        fs::File::create(&tracked).map_err(|e| LauncherError::Prefix(e.to_string()))?;
    }

    let drive_c_users = pfx.join("drive_c").join("users");
    fs::create_dir_all(&drive_c_users)
        .map_err(|e| LauncherError::Prefix(e.to_string()))?;
    let unix_username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "user".to_string());
    let steamuser = drive_c_users.join("steamuser");
    let wineuser  = drive_c_users.join(&unix_username);
    match (wineuser.exists(), steamuser.exists()) {
        (false, false) => {
            fs::create_dir_all(&steamuser).map_err(|e| LauncherError::Prefix(e.to_string()))?;
            symlink("steamuser", &wineuser).map_err(|e| LauncherError::Prefix(e.to_string()))?;
        }
        (true, false) => { symlink(&unix_username, &steamuser).map_err(|e| LauncherError::Prefix(e.to_string()))?; }
        (false, true) => { symlink("steamuser", &wineuser).map_err(|e| LauncherError::Prefix(e.to_string()))?; }
        _ => {}
    }
    Ok(())
}


// ── Environment assembly ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEnv {
    pub vars: HashMap<String, String>,
    pub command: Vec<String>,
    pub using_runtime: bool,
    pub runtime_variant: Option<String>,
}

pub fn build_launch_env(game: &Game, tool: &CompatTool) -> Result<LaunchEnv, LauncherError> {
    let mut env: HashMap<String, String> = HashMap::new();
    let config = &game.config;

    let exe_path    = game.exe_path.canonicalize().unwrap_or_else(|_| game.exe_path.clone());
    let proton_path = tool.path.clone();
    let prefix_path = config.prefix_path.clone().unwrap_or_else(|| {
        crate::paths::default_prefix_dir(&config.game_id)
            .unwrap_or_else(|| PathBuf::from("/tmp").join("gamez-pfx_data").join(&config.game_id))
    });

    let install_path = exe_path.parent()
        .unwrap_or(Path::new("/"))
        .to_string_lossy().into_owned();

    let steam_app_id = crate::paths::strip_game_id_prefix(&config.game_id)
        .filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(String::from)
        .unwrap_or_else(|| format!("{:x}", fnv1a(&prefix_path.to_string_lossy())));

    // Set prefix for all launcher modes:
    // - MYTHIX_PREFIX / LOONI_PREFIX: used by neutron's standalone mode only
    // - WINEPREFIX: used by direct wine fallback (no launcher script)
    // - STEAM_COMPAT_DATA_PATH: used by Proton/Neutron (always set)
    let has_neutron = crate::game_library::neutron_script(&tool.path).is_some();
    let has_proton = tool.path.join("proton").is_file();
    // Neutron: uses MYTHIX_PREFIX for its standalone mode. We also set the
    //   legacy LOONI_PREFIX so Neutron runtimes built before the rebrand (which
    //   only read LOONI_PREFIX) keep working. The mythix build system reads
    //   MYTHIX_PREFIX with LOONI_PREFIX as a fallback, so setting both is safe.
    // Proton (hybrid): uses WINEPREFIX for Mode A ("run" verb)
    //   Do NOT set MYTHIX_PREFIX/LOONI_PREFIX for proton — it triggers standalone
    //   mode which skips prefix DLL setup (missing dxgi.dll, d3d11.dll etc.)
    // Direct wine: uses WINEPREFIX
    if has_neutron && !has_proton {
        let pfx = prefix_path.join("pfx").to_string_lossy().into_owned();
        env.insert("MYTHIX_PREFIX".into(), pfx.clone());
        env.insert("LOONI_PREFIX".into(), pfx);
    } else if !has_neutron && !has_proton {
        // Direct wine — no launcher script
        env.insert("WINEPREFIX".into(), prefix_path.join("pfx").to_string_lossy().into());
    } else {
        // Proton (hybrid) — set WINEPREFIX so Mode A uses our prefix
        env.insert("WINEPREFIX".into(), prefix_path.join("pfx").to_string_lossy().into());
    }
    env.insert("PROTONPATH".into(),              proton_path.to_string_lossy().into());
    env.insert("EXE".into(),                     exe_path.to_string_lossy().into());
    env.insert("GAMEID".into(),                  config.game_id.clone());
    env.insert("STORE".into(),                   config.store.clone());
    env.insert("PROTON_VERB".into(),             config.proton_verb.clone());
    env.insert("STEAM_COMPAT_DATA_PATH".into(),  prefix_path.to_string_lossy().into());
    env.insert("STEAM_COMPAT_INSTALL_PATH".into(), install_path.clone());
    env.insert("STEAM_COMPAT_APP_ID".into(),     steam_app_id.clone());
    env.insert("SteamAppId".into(),              steam_app_id.clone());
    env.insert("SteamGameId".into(),             steam_app_id.clone());
    env.insert("STEAM_COMPAT_SHADER_PATH".into(),
        prefix_path.join("shadercache").to_string_lossy().into());
    // Point at Steam client install if present (Proton needs this for some prefix ops)
    let steam_client = dirs::home_dir()
        .map(|h| h.join(".steam").join("steam"))
        .filter(|p| p.is_dir())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    env.insert("STEAM_COMPAT_CLIENT_INSTALL_PATH".into(), steam_client);
    env.insert("PROTON_CRASH_REPORT_DIR".into(),
        std::env::temp_dir().join("mythix_crashreports").to_string_lossy().into());

    // Debug logging — PROTON_LOG=1 makes the proton script override WINEDEBUG
    // to "+all" which enables ftrace (needs root) and causes debug buffer overflow
    // → SIGABRT. Don't set it by default. Users can enable via env overrides.
    if !config.env_overrides.contains_key("WINEDEBUG") {
        env.insert("WINEDEBUG".into(), "-all,+err".into());
    }

    // Load CompatLayer to detect required runtime
    let compat_layer = CompatLayer::load(&proton_path).ok();

    // Sniper-built tools already bundle the runtime — wrapping them in
    // the container again causes library conflicts (ntdll.so etc.).
    let is_sniper_built = {
        let name = proton_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        name.contains("sniper") || proton_path.join(".sniper-built").is_file()
    };

    // Resolve runtime usage: per-game override > auto-detect from toolmanifest.
    // Sniper-built tools never use the container regardless of the setting.
    let (mut using_runtime, mut runtime_variant_name, tool_paths) = if is_sniper_built {
        eprintln!("[mythix] Sniper-built tool detected — skipping runtime container");
        (false, None, proton_path.to_string_lossy().into_owned())
    } else {
        let want_runtime = match config.use_runtime {
            Some(true) => true,
            Some(false) => false,
            None => {
                // Auto: check toolmanifest for require_tool_appid
                compat_layer.as_ref()
                    .and_then(|l| l.required_runtime())
                    .map(|rv| is_runtime_installed(rv))
                    .unwrap_or(false)
            }
        };

        if want_runtime {
            // Per-game override > toolmanifest > default (sniper)
            let variant = config.runtime_variant.as_deref()
                .and_then(|v| match v {
                    "sniper" | "steamrt3"  => Some(RuntimeVariant::Sniper),
                    "soldier" | "steamrt2" => Some(RuntimeVariant::Soldier),
                    "steamrt4"             => Some(RuntimeVariant::SteamRT4),
                    _ => None,
                })
                .or_else(|| compat_layer.as_ref().and_then(|l| l.required_runtime()))
                .unwrap_or(RuntimeVariant::Sniper);

            if is_runtime_installed(variant) {
                let rt_path = local_runtime_path(variant);
                let paths = format!("{}:{}", proton_path.display(), rt_path.display());
                let layer_name = compat_layer.as_ref()
                    .map(|l| l.layer_name.clone())
                    .unwrap_or_else(|| "neutron".to_string());
                env.insert("STEAM_COMPAT_LAUNCHER_SERVICE".into(), layer_name);
                (true, Some(variant.repo_dir().to_string()), paths)
            } else {
                eprintln!("[mythix] Runtime requested but not installed — launching without container");
                (false, None, proton_path.to_string_lossy().into_owned())
            }
        } else {
            (false, None, proton_path.to_string_lossy().into_owned())
        }
    };

    env.insert("STEAM_COMPAT_TOOL_PATHS".into(), tool_paths.clone());
    env.insert("STEAM_COMPAT_MOUNTS".into(), tool_paths);

    // Library paths — only include game-relevant dirs, NOT the current process's
    // LD_LIBRARY_PATH which is polluted by Cargo's build dirs during `cargo tauri dev`.
    // The proton/neutron script sets up its own LD_LIBRARY_PATH with the correct
    // Wine/Proton libs and host GPU drivers.
    let mut lib_paths: Vec<String> = Vec::new();
    if !install_path.is_empty() { lib_paths.push(install_path); }
    for p in &["/usr/lib/x86_64-linux-gnu", "/usr/lib/i386-linux-gnu",
               "/usr/lib", "/usr/lib32", "/usr/lib64",
               "/lib/x86_64-linux-gnu", "/lib/i386-linux-gnu",
               "/lib", "/lib32", "/lib64"] {
        if Path::new(p).is_dir() { lib_paths.push(p.to_string()); }
    }
    lib_paths.dedup();
    env.insert("STEAM_RUNTIME_LIBRARY_PATH".into(), lib_paths.join(":"));

    // Global env from settings (lower priority than per-game)
    let settings = crate::settings::load_settings();
    for (k, v) in &settings.global_env {
        env.insert(k.clone(), v.clone());
    }

    // Per-game overrides (highest priority)
    for (k, v) in &config.env_overrides {
        env.insert(k.clone(), v.clone());
    }

    // MYTHIX_PRIME_RENDER: expand into the NVIDIA Prime offload env vars
    if env.get("MYTHIX_PRIME_RENDER").map(|v| v == "1").unwrap_or(false) {
        env.insert("__NV_PRIME_RENDER_OFFLOAD".into(), "1".into());
        env.insert("__VK_LAYER_NV_optimus".into(), "NVIDIA_only".into());
        env.insert("__GLX_VENDOR_LIBRARY_NAME".into(), "nvidia".into());
    }

    // Atom mode: when NEUTRON_ATOM_PROTON is set (via per-game env overrides),
    // include the Proton path in STEAM_COMPAT_TOOL_PATHS so Neutron can find it,
    // and force runtime container if available.
    if let Some(atom_proton) = env.get("NEUTRON_ATOM_PROTON").cloned() {
        let ap = PathBuf::from(&atom_proton);
        if ap.join("proton").is_file() {
            eprintln!("[mythix] Atom mode — Proton: {}", ap.display());
            // Add Proton path to tool paths so the container can see it
            if let Some(tp) = env.get("STEAM_COMPAT_TOOL_PATHS").cloned() {
                if !tp.contains(&atom_proton) {
                    let new_tp = format!("{}:{}", tp, atom_proton);
                    env.insert("STEAM_COMPAT_TOOL_PATHS".into(), new_tp.clone());
                    env.insert("STEAM_COMPAT_MOUNTS".into(), new_tp);
                }
            }
            // Force runtime container — detect which one from the Proton's
            // own toolmanifest (Proton 11.0 needs steamrt4, not sniper).
            if !using_runtime {
                let atom_variant = CompatLayer::load(&ap).ok()
                    .and_then(|l| l.required_runtime())
                    .or_else(|| compat_layer.as_ref().and_then(|l| l.required_runtime()))
                    .unwrap_or(RuntimeVariant::SteamRT4);
                if is_runtime_installed(atom_variant) {
                    eprintln!("[mythix] Atom mode — forcing {} container", atom_variant.display_name());
                    using_runtime = true;
                    runtime_variant_name = Some(atom_variant.repo_dir().to_string());
                    let rt_path = local_runtime_path(atom_variant);
                    let paths = format!("{}:{}:{}", proton_path.display(), atom_proton, rt_path.display());
                    env.insert("STEAM_COMPAT_TOOL_PATHS".into(), paths.clone());
                    env.insert("STEAM_COMPAT_MOUNTS".into(), paths);
                } else {
                    eprintln!("[mythix] Atom mode — {} not installed, launching without container",
                              atom_variant.display_name());
                }
            }
        } else {
            eprintln!("[mythix] Warning: NEUTRON_ATOM_PROTON set but no proton script at {}", ap.display());
        }
    }

    // Build command
    // Only use the CompatLayer command chain (mythix-shim → SLR container) when
    // the required runtime is actually installed. Otherwise launch directly
    // with the neutron/proton/wine script — no container, no shim.
    let verb = config.proton_verb.as_str();
    let exe  = exe_path.to_string_lossy().into_owned();
    let mut command = if using_runtime {
        if let Some(ref rt_name) = runtime_variant_name {
            // Find the right runtime variant for the container entry point
            let rt_variant = match rt_name.as_str() {
                "steamrt4" => Some(RuntimeVariant::SteamRT4),
                "steamrt3" => Some(RuntimeVariant::Sniper),
                "steamrt2" => Some(RuntimeVariant::Soldier),
                _ => None,
            };
            if let Some(variant) = rt_variant {
                let rt_path = local_runtime_path(variant);
                let entry_point = CompatLayer::resolve_entry_point_pub(&rt_path);
                let shim = crate::runtime::shim_path_pub();
                // [entry_point, --verb=verb, --, shim, neutron/neutron, verb, exe]
                let mut cmd = vec![
                    entry_point,
                    format!("--verb={verb}"),
                    "--".into(),
                    shim,
                ];
                cmd.extend(build_fallback_command(&proton_path, verb, exe));
                cmd
            } else if let Some(ref layer) = compat_layer {
                let mut cmd = layer.build_command(verb);
                cmd.push(exe);
                cmd
            } else {
                build_fallback_command(&proton_path, verb, exe)
            }
        } else if let Some(ref layer) = compat_layer {
            let mut cmd = layer.build_command(verb);
            cmd.push(exe);
            cmd
        } else {
            build_fallback_command(&proton_path, verb, exe)
        }
    } else {
        build_fallback_command(&proton_path, verb, exe)
    };
    command.extend(config.launch_args.clone());

    // GameMode wrapping for non-neutron tools (neutron handles its own via NEUTRON_GAMEMODE)
    let has_neutron = crate::game_library::neutron_script(&proton_path).is_some();
    if !has_neutron
        && env.get("NEUTRON_GAMEMODE").map(|v| v == "1").unwrap_or(false)
        && which_bin("gamemoderun").is_some()
    {
        command.insert(0, "gamemoderun".into());
    }

    Ok(LaunchEnv { vars: env, command, using_runtime, runtime_variant: runtime_variant_name })
}

fn build_fallback_command(proton_path: &Path, verb: &str, exe: String) -> Vec<String> {
    // Neutron uses its own launcher script
    if let Some(neutron) = crate::game_library::neutron_script(proton_path) {
        return vec![neutron.to_string_lossy().into_owned(), verb.to_string(), exe];
    }
    // Proton uses its own launcher script
    let proton = proton_path.join("proton");
    if proton.is_file() {
        return vec![proton.to_string_lossy().into_owned(), verb.to_string(), exe];
    }
    // Direct wine fallback — check all possible locations
    // Neutron: files/bin/wine, Proton: bin/wine, standalone: wine
    let wine = [proton_path.join("files").join("bin").join("wine64"),
                proton_path.join("files").join("bin").join("wine"),
                proton_path.join("bin").join("wine64"),
                proton_path.join("bin").join("wine"),
                proton_path.join("wine")]
        .iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wine".to_string());
    vec![wine, exe]
}

// ── Process spawning ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchResult {
    pub pid: u32,
    pub game_id: String,
    pub using_runtime: bool,
    pub runtime_variant: Option<String>,
}

/// Environment variables to pass through from the host session.
const PASSTHROUGH_VARS: &[&str] = &[
    "HOME", "USER", "PATH", "TERM",
    // Display — X11 and Wayland
    "DISPLAY", "XAUTHORITY",
    "WAYLAND_DISPLAY", "XDG_SESSION_TYPE", "XDG_CURRENT_DESKTOP",
    "XDG_RUNTIME_DIR", "XDG_DATA_HOME",
    "DBUS_SESSION_BUS_ADDRESS",
    // Audio
    "PULSE_SERVER", "PULSE_COOKIE", "PIPEWIRE_RUNTIME_DIR",
    // System
    "XDG_DATA_DIRS", "XDG_CONFIG_DIRS", "XDG_CONFIG_HOME",
    // NOTE: LD_LIBRARY_PATH intentionally excluded — the proton/neutron script
    // builds its own with the correct Wine libs + host GPU drivers. Passing ours
    // through would contaminate it with Cargo build dirs during development.
    "LANG", "LC_ALL",
    // GPU / Vulkan
    "VULKAN_ICD_FILENAMES", "VK_ICD_FILENAMES", "VK_DRIVER_FILES",
    "LIBVA_DRIVER_NAME", "MESA_LOADER_DRIVER_OVERRIDE",
    "__GLX_VENDOR_LIBRARY_NAME", "DRI_PRIME",
    // Container / runtime
    "PRESSURE_VESSEL_SHARE_HOME", "CONTAINER_RUNTIME",
    // Gamemode
    "LD_PRELOAD",
    // Python (neutron script is Python)
    "PYTHONPATH", "PYTHONDONTWRITEBYTECODE",
    // Temp
    "TMPDIR", "TEMP", "TMP",
    // Hostname (some games/Wine need this)
    "HOSTNAME", "LOGNAME",
];

/// When mythix is launched from a non-graphical context (SSH, systemd service),
/// critical display variables like DISPLAY, WAYLAND_DISPLAY, and XAUTHORITY
/// won't be in our environment. This function probes the running desktop session
/// by reading /proc/<pid>/environ from a known graphical process.
fn discover_display_env() -> HashMap<String, String> {
    let needed = ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "XDG_SESSION_TYPE",
                  "XDG_CURRENT_DESKTOP", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"];

    // If we have display AND auth, we're fully set up — no need to probe
    let have_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    let have_auth = std::env::var("XAUTHORITY").is_ok();
    if have_display && have_auth {
        return HashMap::new();
    }
    // If we have DISPLAY but no XAUTHORITY, we still need to probe for it
    if have_display {
        eprintln!("[mythix] DISPLAY set but XAUTHORITY missing — probing desktop session...");
    }

    eprintln!("[mythix] No DISPLAY/WAYLAND_DISPLAY — probing running desktop session...");

    // Try well-known desktop processes to find one with display vars
    for proc_name in &["plasmashell", "kwin_wayland", "kwin_x11", "xfce4-session",
                       "gnome-shell", "cinnamon", "mate-session"] {
        if let Ok(output) = Command::new("pgrep").arg("-u").arg(whoami()).arg("-x").arg(proc_name)
            .output()
        {
            let pids = String::from_utf8_lossy(&output.stdout);
            if let Some(pid) = pids.lines().next() {
                let environ_path = format!("/proc/{}/environ", pid.trim());
                if let Ok(data) = fs::read(&environ_path) {
                    let mut found = HashMap::new();
                    for entry in data.split(|&b| b == 0) {
                        if let Ok(s) = std::str::from_utf8(entry) {
                            if let Some((k, v)) = s.split_once('=') {
                                if needed.contains(&k) {
                                    found.insert(k.to_string(), v.to_string());
                                }
                            }
                        }
                    }
                    if found.contains_key("DISPLAY") || found.contains_key("WAYLAND_DISPLAY") {
                        eprintln!("[mythix] Found display env from {} (PID {}): {:?}",
                            proc_name, pid.trim(),
                            found.keys().collect::<Vec<_>>());
                        return found;
                    }
                }
            }
        }
    }

    eprintln!("[mythix] WARNING: Could not discover display environment — games may not render");
    HashMap::new()
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "1000".to_string())
}

/// Build a verbose debug dump of the full launch environment.
pub fn debug_launch_dump(game: &Game, launch_env: &LaunchEnv) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════╗\n");
    out.push_str("║           MYTHIX LAUNCH DEBUG DUMP               ║\n");
    out.push_str("╚══════════════════════════════════════════════════╝\n\n");

    out.push_str(&format!("Game:       {}\n", game.name));
    out.push_str(&format!("Exe:        {}\n", game.exe_path.display()));
    out.push_str(&format!("Game ID:    {}\n", game.config.game_id));
    out.push_str(&format!("Store:      {}\n", if game.config.store.is_empty() { "(none)" } else { &game.config.store }));
    out.push_str(&format!("Verb:       {}\n", game.config.proton_verb));
    out.push_str(&format!("Prefix:     {}\n", game.config.prefix_path.as_ref().map(|p| p.display().to_string()).unwrap_or("(default)".into())));
    out.push_str(&format!("Proton/Tool:{}\n", game.config.proton_path.as_ref().map(|p| p.display().to_string()).unwrap_or("(none)".into())));
    let rt_override = match game.config.use_runtime {
        Some(true) => " [forced ON]",
        Some(false) => " [forced OFF]",
        None => " [auto]",
    };
    out.push_str(&format!("Runtime:    {} ({}){}\n",
        if launch_env.using_runtime { "yes" } else { "no" },
        launch_env.runtime_variant.as_deref().unwrap_or("none"),
        rt_override));

    out.push_str("\n── Command ─────────────────────────────────────\n");
    for (i, part) in launch_env.command.iter().enumerate() {
        out.push_str(&format!("  [{}] {}\n", i, part));
    }

    out.push_str("\n── Environment Variables ────────────────────────\n");
    let mut keys: Vec<&String> = launch_env.vars.keys().collect();
    keys.sort();
    for k in keys {
        let v = &launch_env.vars[k];
        // Truncate very long values for readability
        let display_v = if v.len() > 120 { format!("{}…", &v[..120]) } else { v.clone() };
        out.push_str(&format!("  {}={}\n", k, display_v));
    }

    out.push_str("\n── Host Passthrough ─────────────────────────────\n");
    let discovered = discover_display_env();
    for var in PASSTHROUGH_VARS {
        if let Some(val) = discovered.get(*var) {
            let display = if val.len() > 100 { format!("{}…", &val[..100]) } else { val.clone() };
            out.push_str(&format!("  {} = {} (discovered)\n", var, display));
        } else {
            match std::env::var(var) {
                Ok(val) => {
                    let display = if val.len() > 100 { format!("{}…", &val[..100]) } else { val };
                    out.push_str(&format!("  {} = {}\n", var, display));
                }
                Err(_) => out.push_str(&format!("  {} = (unset)\n", var)),
            }
        }
    }

    out.push_str("\n── Prefix Check ─────────────────────────────────\n");
    if let Some(pfx) = &game.config.prefix_path {
        out.push_str(&format!("  exists: {}\n", pfx.exists()));
        out.push_str(&format!("  pfx/: {}\n", pfx.join("pfx").exists()));
        out.push_str(&format!("  drive_c/: {}\n", pfx.join("pfx").join("drive_c").exists()));
        out.push_str(&format!("  user.reg: {}\n", pfx.join("pfx").join("user.reg").exists()));
    } else {
        out.push_str("  (no prefix configured)\n");
    }

    out.push_str("\n── Tool Check ───────────────────────────────────\n");
    if let Some(tp) = &game.config.proton_path {
        out.push_str(&format!("  proton script: {}\n", tp.join("proton").is_file()));
        out.push_str(&format!("  neutron script: {}\n", tp.join("neutron").is_file()));
        out.push_str(&format!("  bin/wine: {}\n", tp.join("bin").join("wine").is_file()));
        out.push_str(&format!("  bin/wine64: {}\n", tp.join("bin").join("wine64").is_file()));
        out.push_str(&format!("  files/bin/wine: {}\n", tp.join("files").join("bin").join("wine").is_file()));
        out.push_str(&format!("  files/bin/wine64: {}\n", tp.join("files").join("bin").join("wine64").is_file()));
        out.push_str(&format!("  toolmanifest.vdf: {}\n", tp.join("toolmanifest.vdf").is_file()));
        out.push_str(&format!("  compatibilitytool.vdf: {}\n", tp.join("compatibilitytool.vdf").is_file()));
        out.push_str(&format!("  files/: {}\n", tp.join("files").is_dir()));
    }

    out
}

pub fn spawn_game(
    game: &Game, 
    launch_env: &LaunchEnv,
    app: &tauri::AppHandle,
) -> Result<LaunchResult, LauncherError> {
    if launch_env.command.is_empty() {
        return Err(LauncherError::Launch("Empty command".into()));
    }
    let (bin, args) = launch_env.command.split_first().unwrap();

    if let Some(pfx) = &game.config.prefix_path {
        let settings = crate::settings::load_settings();
        if settings.auto_setup_prefix && game.config.prefix_type == PrefixType::Proton && !pfx.join("pfx").join("system.reg").exists() {
            setup_pfx(pfx)?;
        }
    }

    // DLL syncing is handled by the Neutron script's update_builtin_libs(),
    // which correctly distinguishes Wine builtins from DXVK/native DLLs.

    // Spawn through `setsid` binary to fully detach the game process from
    // Tauri's process tree. WebKitGTK (Tauri's renderer) sets up a seccomp
    // sandbox that child processes inherit — Wine uses syscalls that the
    // sandbox blocks, causing SIGSYS. Using the external `setsid` command
    // creates a new session AND a new process group, breaking the seccomp
    // inheritance chain.
    let mut cmd = Command::new("setsid");
    cmd.arg("--fork");  // double-fork to fully detach
    cmd.arg(bin);
    cmd.args(args);
    // Set working directory to the game's install folder — many games
    // use relative paths to find their data files (.era, .pak, etc.)
    if let Some(game_dir) = game.exe_path.parent() {
        cmd.current_dir(game_dir);
    }
    cmd.env_clear();

    // Game stdout/stderr can be extremely noisy. Piping every line into Tauri/React
    // will freeze the renderer for chatty games, so quiet mode is the default.
    // Set MYTHIX_CAPTURE_GAME_OUTPUT=1 per-game to capture bounded UI logs.
    let capture_game_output = env_flag(&launch_env.vars, "MYTHIX_CAPTURE_GAME_OUTPUT");
    let tee_game_output = env_flag(&launch_env.vars, "MYTHIX_TEE_GAME_OUTPUT");
    if capture_game_output {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    // Pass through host environment, with auto-detection for display vars
    // when launched from a non-graphical context (e.g. SSH)
    let display_overrides = discover_display_env();
    for var in PASSTHROUGH_VARS {
        if let Some(val) = display_overrides.get(*var) {
            cmd.env(var, val);
        } else if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    for (k, v) in &launch_env.vars { cmd.env(k, v); }

    // Print debug dump to terminal (visible in cargo tauri dev output)
    let dump = debug_launch_dump(game, launch_env);
    eprintln!("{}", dump);

    let mut child: Child = cmd.spawn()
        .map_err(|e| LauncherError::Launch(
            format!("{e}\nCommand: {}", launch_env.command.join(" "))))?;
    let pid = child.id();

    eprintln!("[mythix] Process spawned: PID {}", pid);

    // Drain/capture stdout/stderr in background threads. Capture is bounded so
    // noisy games cannot freeze the launcher by flooding IPC/React state.
    let game_name = game.name.clone();
    let app_handle = app.clone();
    if let Some(stdout) = child.stdout.take() {
        spawn_log_drain(stdout, game_name.clone(), "stdout", app_handle.clone(), tee_game_output);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_drain(stderr, game_name.clone(), "stderr", app_handle.clone(), tee_game_output);
    }

    // Monitor process exit in background — record playtime and notify frontend
    let exit_name = game_name;
    let exit_game_id = game.id.clone();
    let exit_app = app.clone();
    let exit_prefix = game.config.prefix_path.clone();
    std::thread::spawn(move || {
        // setsid --fork double-forks, so child.wait() returns almost immediately.
        // Instead, poll for the wineserver process associated with this prefix.
        let _ = child.wait();
        let start = std::time::Instant::now();

        // Find wineserver for this prefix by polling /proc
        let pfx_str = exit_prefix.as_ref()
            .map(|p| p.join("pfx").to_string_lossy().to_string())
            .unwrap_or_default();

        if pfx_str.is_empty() {
            eprintln!("[mythix] No prefix for playtime tracking, skipping");
            return;
        }

        // Wait a moment for wineserver to start
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Poll until no processes reference this prefix
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let alive = std::process::Command::new("bash")
                .args(["-c", &format!(
                    "grep -rl '{}' /proc/*/environ 2>/dev/null | head -1",
                    pfx_str
                )])
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);
            if !alive { break; }
        }

        let elapsed_secs = start.elapsed().as_secs();
        eprintln!("[mythix] {} exited after {}s", exit_name, elapsed_secs);

        if elapsed_secs > 5 {
            if let Err(e) = crate::game_library::record_playtime(&exit_game_id, elapsed_secs) {
                eprintln!("[mythix] Failed to record playtime: {}", e);
            } else {
                eprintln!("[mythix] Recorded {}s playtime for {}", elapsed_secs, exit_name);
            }
        }
        let _ = exit_app.emit("game:exited", serde_json::json!({
            "gameId": exit_game_id,
            "playtime": elapsed_secs,
        }));
    });

    Ok(LaunchResult {
        pid,
        game_id: game.id.clone(),
        using_runtime: launch_env.using_runtime,
        runtime_variant: launch_env.runtime_variant.clone(),
    })
}

const MAX_GAME_LOG_LINES_PER_STREAM: usize = 500;

fn env_flag(env: &HashMap<String, String>, key: &str) -> bool {
    env.get(key)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn spawn_log_drain<R>(
    stream: R,
    game_name: String,
    stream_name: &'static str,
    app: tauri::AppHandle,
    tee_to_terminal: bool,
) where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        let mut emitted = 0usize;
        let prefix = format!("[{}]", stream_name);

        for line in reader.lines().map_while(Result::ok) {
            if emitted < MAX_GAME_LOG_LINES_PER_STREAM {
                if tee_to_terminal {
                    eprintln!("[{}:{}] {}", game_name, stream_name, line);
                }
                let _ = app.emit("game:log", format!("{} {}", prefix, line));
            } else if emitted == MAX_GAME_LOG_LINES_PER_STREAM {
                let notice = format!(
                    "{} output suppressed after {} lines; set MYTHIX_CAPTURE_GAME_OUTPUT=0 or inspect external logs for very noisy games",
                    prefix,
                    MAX_GAME_LOG_LINES_PER_STREAM
                );
                if tee_to_terminal {
                    eprintln!("[{}:{}] {}", game_name, stream_name, notice);
                }
                let _ = app.emit("game:log", notice);
            }
            emitted = emitted.saturating_add(1);
        }
    });
}

fn which_bin(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .and_then(|paths| std::env::split_paths(&paths)
            .map(|p| p.join(name))
            .find(|p| p.is_file()))
}

fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in s.bytes() { h ^= b as u32; h = h.wrapping_mul(16777619); }
    h
}
