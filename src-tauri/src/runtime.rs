// runtime.rs
// Steam Linux Runtime (SLR) management.
//
// Ports the core logic from mythix's runtime + run CompatLayer logic.
//
// The SLR is Valve's open container runtime, publicly available at:
//   https://repo.steampowered.com/{variant}/images/latest-public-beta/
//
// It provides a reproducible Debian/Ubuntu environment so Proton games run
// consistently across distros. No Steam installation required.
//
// Layout after install ($XDG_DATA_HOME/mythix/steamrt3/):
//   _v2-entry-point          ← pressure-vessel entry point (executable)
//   mythix -> _v2-entry-point  ← symlink (looni kept too for backwards compat)
//   pressure-vessel/         ← pv tooling
//   sniper_platform_*/       ← the actual container image
//   VERSIONS.txt             ← used to detect when updates are needed
//   .mythix-installed        ← our marker, written after successful install

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::error::LauncherError;
use crate::vdf;

// ── Runtime catalogue ─────────────────────────────────────────────────────────
// Mirrors mythix's RUNTIME_VERSIONS dict.
// Format: (name/codename, variant/dir_name, steam_appid)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeVariant {
    /// Steam Linux Runtime 3 "sniper" — used by GE-Proton, Proton 8+
    Sniper,
    /// Steam Linux Runtime 2 "soldier" — older Proton / legacy
    Soldier,
    /// Steam Linux Runtime 4 — future / experimental
    SteamRT4,
}

impl RuntimeVariant {
    /// Short codename used in archive names and directory globs.
    pub fn codename(&self) -> &'static str {
        match self {
            Self::Sniper   => "sniper",
            Self::Soldier  => "soldier",
            Self::SteamRT4 => "steamrt4",
        }
    }

    /// Valve's repo subdirectory / variant name.
    pub fn repo_dir(&self) -> &'static str {
        match self {
            Self::Sniper   => "steamrt3",
            Self::Soldier  => "steamrt2",
            Self::SteamRT4 => "steamrt4",
        }
    }

    /// Local install directory name (inside $XDG_DATA_HOME/mythix/).
    pub fn local_dir(&self) -> &'static str {
        self.repo_dir()
    }

    /// Steam App ID (matches the original RUNTIME_VERSIONS).
    pub fn appid(&self) -> &'static str {
        match self {
            Self::Sniper   => "1628350",
            Self::Soldier  => "1391110",
            Self::SteamRT4 => "4183110",
        }
    }

    /// Human-readable label.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sniper   => "Steam Linux Runtime 3 (sniper)",
            Self::Soldier  => "Steam Linux Runtime 2 (soldier)",
            Self::SteamRT4 => "Steam Linux Runtime 4",
        }
    }

    /// Derive the variant from a `require_tool_appid` value in toolmanifest.vdf.
    pub fn from_appid(appid: &str) -> Option<Self> {
        match appid {
            "1628350" => Some(Self::Sniper),
            "1391110" => Some(Self::Soldier),
            "4183110" => Some(Self::SteamRT4),
            _ => None,
        }
    }

    /// Archive filename on the CDN.
    fn archive_name(&self) -> String {
        match self {
            Self::SteamRT4 => format!("SteamLinuxRuntime_{}.tar.xz", "4"),
            _              => format!("SteamLinuxRuntime_{}.tar.xz", self.codename()),
        }
    }

    /// CDN base URL for this variant.
    fn cdn_base(&self) -> String {
        format!(
            "https://repo.steampowered.com/{}/images/latest-public-beta",
            self.repo_dir()
        )
    }
}

// ── Install status ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    /// Not yet installed, no local directory.
    NotInstalled,
    /// Installed and looks healthy.
    Installed { build_id: String },
    /// Installed but may need an update.
    UpdateAvailable { current: String, latest: String },
    /// Files are missing or corrupted.
    Corrupt,
}

pub fn local_runtime_path(variant: RuntimeVariant) -> PathBuf {
    crate::paths::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share").join(crate::paths::APP_DIR))
        .join(variant.local_dir())
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join(crate::paths::APP_DIR)
}

fn install_marker(local: &Path) -> PathBuf {
    local.join(".mythix-installed")
}

pub fn shim_path_pub() -> String {
    shim_path().to_string_lossy().into_owned()
}

fn shim_path() -> PathBuf {
    crate::paths::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share").join(crate::paths::APP_DIR))
        .join("mythix-shim")
}

/// Returns true if the runtime looks fully installed. Accepts both the new
/// `.mythix-installed` marker and the legacy `.looni-installed` one so runtimes
/// migrated from a pre-rebrand install aren't needlessly re-downloaded.
pub fn is_runtime_installed(variant: RuntimeVariant) -> bool {
    let local = local_runtime_path(variant);
    (local.join(".mythix-installed").is_file() || local.join(".looni-installed").is_file())
        && local.join("pressure-vessel").is_dir()
        && local.join("VERSIONS.txt").is_file()
}

/// Read the current build ID from VERSIONS.txt, or return empty string.
pub fn current_build_id(variant: RuntimeVariant) -> String {
    let path = local_runtime_path(variant).join("VERSIONS.txt");
    fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("build-id"))
                .map(|l| l.split('=').nth(1).unwrap_or("").trim().to_string())
        })
        .unwrap_or_default()
}

// ── Progress events (emitted to frontend) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub stage: String,
    pub variant: String,
    /// 0–100, or -1 for indeterminate
    pub percent: i32,
    pub detail: String,
}

fn emit_progress(app: &AppHandle, stage: &str, variant: RuntimeVariant, percent: i32, detail: &str) {
    let _ = app.emit(
        "runtime:progress",
        ProgressEvent {
            stage: stage.into(),
            variant: variant.display_name().into(),
            percent,
            detail: detail.into(),
        },
    );
}

// ── Downloader ────────────────────────────────────────────────────────────────

/// Download, verify, and install the Steam Linux Runtime.
///
/// Emits `runtime:progress` events to the frontend throughout.
/// Supports resumable downloads via cached partial files.
pub async fn install_runtime(
    variant: RuntimeVariant,
    app: AppHandle,
) -> Result<(), LauncherError> {
    let local = local_runtime_path(variant);
    let cache = cache_dir();
    fs::create_dir_all(&local)
        .map_err(|e| LauncherError::Io(e.to_string()))?;
    fs::create_dir_all(&cache)
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    let client = Client::builder()
        .user_agent("mythix-launcher/0.1")
        .build()
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    let base     = variant.cdn_base();
    let archive  = variant.archive_name();

    emit_progress(&app, "fetch_meta", variant, -1, "Fetching build info…");

    // ── Step 1: get BUILD_ID ──────────────────────────────────────────────
    let build_id = fetch_text(&client, &format!("{base}/BUILD_ID.txt")).await?;
    let build_id = build_id.trim().to_string();
    log::debug!("Remote BUILD_ID: {build_id}");

    // ── Step 2: get expected SHA256 ────────────────────────────────────────
    let sha256sums = fetch_text(&client, &format!("{base}/SHA256SUMS")).await?;
    let expected_digest = sha256sums
        .lines()
        .find(|l| l.ends_with(&archive))
        .and_then(|l| l.split_whitespace().next())
        .ok_or_else(|| LauncherError::Io(format!("SHA256 for {archive} not found")))?
        .to_string();
    log::debug!("Expected SHA256: {expected_digest}");

    // ── Step 3: check if we already have this build ───────────────────────
    let local_build = current_build_id(variant);
    if is_runtime_installed(variant) && local_build == build_id {
        emit_progress(&app, "up_to_date", variant, 100, "Runtime is up to date");
        log::info!("{} is up to date (build {build_id})", variant.display_name());
        return Ok(());
    }

    // ── Step 4: download (with resume support) ────────────────────────────
    let parts_name  = format!("{archive}.{build_id}.parts");
    let parts_path  = cache.join(&parts_name);
    let archive_path = cache.join(&archive);

    let (start_byte, mut hasher) = if parts_path.is_file() {
        let sz = parts_path.metadata().map(|m| m.len()).unwrap_or(0);
        emit_progress(&app, "download", variant, 0, &format!("Resuming from {sz} bytes…"));
        let mut h = Sha256::new();
        let mut f = fs::File::open(&parts_path)
            .map_err(|e| LauncherError::Io(e.to_string()))?;
        let mut buf = vec![0u8; 65536];
        loop {
            let n = f.read(&mut buf).map_err(|e| LauncherError::Io(e.to_string()))?;
            if n == 0 { break; }
            h.update(&buf[..n]);
        }
        (sz, h)
    } else {
        emit_progress(&app, "download", variant, 0, &format!("Downloading {}", variant.display_name()));
        (0, Sha256::new())
    };

    let mut req = client.get(&format!("{base}/{archive}"));
    if start_byte > 0 {
        req = req.header("Range", format!("bytes={start_byte}-"));
    }

    let resp = req.send().await.map_err(|e| LauncherError::Io(e.to_string()))?;
    let total_opt: Option<u64> = resp.content_length().map(|n| n + start_byte);
    let status = resp.status();

    use reqwest::StatusCode;
    if ![StatusCode::OK, StatusCode::PARTIAL_CONTENT, StatusCode::RANGE_NOT_SATISFIABLE]
        .contains(&status)
    {
        return Err(LauncherError::Io(format!("CDN returned {status}")));
    }

    // Open file for appending (or create fresh)
    let mut out = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&parts_path)
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    if status != StatusCode::RANGE_NOT_SATISFIABLE {
        use futures_util::StreamExt;
        let mut downloaded = start_byte;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LauncherError::Io(e.to_string()))?;
            hasher.update(&chunk);
            out.write_all(&chunk)
                .map_err(|e| LauncherError::Io(e.to_string()))?;
            downloaded += chunk.len() as u64;

            if let Some(total) = total_opt {
                let pct = (downloaded * 100 / total) as i32;
                let mb_done  = downloaded as f64 / 1_048_576.0;
                let mb_total = total as f64 / 1_048_576.0;
                emit_progress(
                    &app,
                    "download",
                    variant,
                    pct,
                    &format!("{mb_done:.0} / {mb_total:.0} MB"),
                );
            }
        }
    }
    drop(out);

    // ── Step 5: verify SHA256 ─────────────────────────────────────────────
    emit_progress(&app, "verify", variant, -1, "Verifying checksum…");
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_digest {
        // Remove corrupt partial file
        let _ = fs::remove_file(&parts_path);
        return Err(LauncherError::Io(format!(
            "SHA256 mismatch: expected {expected_digest}, got {actual}"
        )));
    }
    log::info!("SHA256 OK for {archive}");

    // Rename .parts → final archive path
    fs::rename(&parts_path, &archive_path)
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    // ── Step 6: extract ───────────────────────────────────────────────────
    emit_progress(&app, "extract", variant, -1, "Extracting archive…");
    let tmp_dir = tempfile::Builder::new()
        .prefix(".mythix-extract-")
        .tempdir_in(&cache)
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    let status = Command::new("tar")
        .args(["xf", archive_path.to_str().unwrap_or(""), "-C", tmp_dir.path().to_str().unwrap_or("")])
        .status()
        .map_err(|e| LauncherError::Io(format!("tar: {e}")))?;

    if !status.success() {
        return Err(LauncherError::Io("tar extraction failed".into()));
    }

    // ── Step 7: atomic swap into local dir ───────────────────────────────
    // Extract produces: tmp_dir/SteamLinuxRuntime_{name}/
    let extracted_name = archive.strip_suffix(".tar.xz").unwrap_or(&archive);
    let extracted_path = tmp_dir.path().join(extracted_name);

    emit_progress(&app, "install", variant, -1, "Installing runtime…");

    // Remove old install if present, then move new one in
    if local.exists() {
        let old = cache.join(format!("{}.old", variant.local_dir()));
        let _ = fs::remove_dir_all(&old);
        fs::rename(&local, &old).ok();
    }
    fs::rename(&extracted_path, &local)
        .map_err(|e| LauncherError::Io(format!("move runtime: {e}")))?;

    // Ensure the entry-point symlink exists (backwards compat with older
    // pressure-vessel). We create the new `mythix` name and also keep the legacy
    // `looni` name so anything still referencing the old symlink keeps working.
    if local.join("_v2-entry-point").exists() {
        let mythix_link = local.join("mythix");
        if !mythix_link.exists() {
            symlink("_v2-entry-point", &mythix_link).ok();
        }
        let looni_link = local.join("looni");
        if !looni_link.exists() {
            symlink("_v2-entry-point", &looni_link).ok();
        }
    }

    // Write our install marker
    fs::write(install_marker(&local), build_id.as_bytes())
        .map_err(|e| LauncherError::Io(e.to_string()))?;

    // Clean up the downloaded archive (we have the installed version now)
    let _ = fs::remove_file(&archive_path);

    emit_progress(&app, "done", variant, 100, "Runtime installed");
    log::info!("{} installed (build {build_id})", variant.display_name());

    // Ensure the shim exists
    ensure_shim()?;

    Ok(())
}

/// Check remote BUILD_ID against local installation, return status.
pub async fn check_for_update(variant: RuntimeVariant) -> RuntimeStatus {
    if !is_runtime_installed(variant) {
        return RuntimeStatus::NotInstalled;
    }

    let client = match Client::builder().user_agent("mythix-launcher/0.1").build() {
        Ok(c) => c,
        Err(_) => return RuntimeStatus::Installed { build_id: current_build_id(variant) },
    };

    let base = variant.cdn_base();
    match fetch_text(&client, &format!("{base}/BUILD_ID.txt")).await {
        Ok(remote) => {
            let remote = remote.trim().to_string();
            let local  = current_build_id(variant);
            if local == remote || remote.is_empty() {
                RuntimeStatus::Installed { build_id: local }
            } else {
                RuntimeStatus::UpdateAvailable { current: local, latest: remote }
            }
        }
        Err(_) => RuntimeStatus::Installed { build_id: current_build_id(variant) },
    }
}

async fn fetch_text(client: &Client, url: &str) -> Result<String, LauncherError> {
    client
        .get(url)
        .send()
        .await
        .map_err(|e| LauncherError::Io(e.to_string()))?
        .text()
        .await
        .map_err(|e| LauncherError::Io(e.to_string()))
}

// ── Shim ─────────────────────────────────────────────────────────────────────
// Creates the mythix shim. A tiny shell script that fixes DISPLAY
// for gamescope before exec'ing its arguments.

pub fn ensure_shim() -> Result<(), LauncherError> {
    let path = shim_path();
    if path.is_file() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let content = "#!/bin/sh\n\
        \n\
        if [ \"${XDG_CURRENT_DESKTOP}\" = \"gamescope\" ] || \
            [ \"${XDG_SESSION_DESKTOP}\" = \"gamescope\" ]; then\n\
            if [ \"${STEAM_MULTIPLE_XWAYLANDS}\" = \"1\" ]; then\n\
                if [ -z \"${DISPLAY}\" ]; then\n\
                    export DISPLAY=\":1\"\n\
                fi\n\
            fi\n\
        fi\n\
        \n\
        exec \"$@\"\n";
    fs::write(&path, content).map_err(|e| LauncherError::Io(e.to_string()))?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .map_err(|e| LauncherError::Io(e.to_string()))?;
    Ok(())
}

// ── CompatLayer ───────────────────────────────────────────────────────────────
// Reads toolmanifest.vdf and compatibilitytool.vdf to understand how to
// build the full launch command for a given tool + runtime.
//
// This is a direct Rust port of the original CompatLayer class.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatLayer {
    /// Absolute path to the tool directory.
    pub tool_path: PathBuf,
    /// Raw commandline template from toolmanifest.vdf, e.g. "/proton %verb%"
    pub commandline: String,
    /// "proton", "container-runtime", "mythix-passthrough", etc.
    pub layer_name: String,
    /// appid of the required runtime (e.g. "1628350" for sniper)
    pub require_tool_appid: Option<String>,
    /// Display name from compatibilitytool.vdf
    pub display_name: String,
    /// Whether this is a proton-type layer
    pub is_proton: bool,
}

impl CompatLayer {
    /// Load a CompatLayer from a tool directory that contains toolmanifest.vdf.
    pub fn load(path: &Path) -> Result<Self, LauncherError> {
        let manifest_path = path.join("toolmanifest.vdf");
        let doc = vdf::load(&manifest_path)?;
        let mf = doc
            .get("manifest")
            .ok_or_else(|| LauncherError::Io("toolmanifest.vdf: no 'manifest' key".into()))?;

        let commandline = mf
            .str_val("commandline")
            .unwrap_or("")
            .to_string();
        let layer_name = mf
            .str_val("compatmanager_layer_name")
            .unwrap_or("")
            .to_string();
        let require_tool_appid = mf
            .str_val("require_tool_appid")
            .map(String::from);

        // Read display_name from compatibilitytool.vdf if present
        let display_name = Self::read_display_name(path)
            .unwrap_or_else(|| path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string());

        let is_proton = layer_name == "proton";

        Ok(Self {
            tool_path: path.to_path_buf(),
            commandline,
            layer_name,
            require_tool_appid,
            display_name,
            is_proton,
        })
    }

    fn read_display_name(path: &Path) -> Option<String> {
        let p = path.join("compatibilitytool.vdf");
        let doc = vdf::load(&p).ok()?;
        let tools = doc.get("compatibilitytools")?.get("compat_tools")?;
        let map = tools.as_map()?;
        // Take the first entry
        map.values().next().and_then(|v| {
            v.str_val("display_name").map(String::from)
        })
    }

    /// The required SLR variant for this tool, if any.
    pub fn required_runtime(&self) -> Option<RuntimeVariant> {
        self.require_tool_appid
            .as_deref()
            .and_then(RuntimeVariant::from_appid)
    }

    /// Expand the commandline template with the given verb and tool_path prefix.
    ///
    /// e.g. "/proton %verb%" + verb="waitforexitandrun" + path="/opt/proton"
    ///   → ["/opt/proton/proton", "waitforexitandrun"]
    fn expand_commandline(&self, verb: &str) -> Vec<String> {
        let full = format!(
            "{}{}",
            self.tool_path.to_string_lossy(),
            self.commandline.replace("%verb%", verb)
        );
        // Simple split — handles the patterns Valve actually uses
        let mut parts: Vec<String> = full.split_whitespace()
            .map(String::from)
            .collect();
        // The rewritten Neutron ships `neutron.py` while its toolmanifest still
        // says `/neutron %verb%` — fall back to the .py script if the bare
        // name doesn't exist.
        if let Some(script) = parts.first_mut() {
            if !std::path::Path::new(script.as_str()).is_file() {
                let py = format!("{script}.py");
                if std::path::Path::new(&py).is_file() {
                    *script = py;
                }
            }
        }
        parts
    }

    /// Build the full launch command, mirroring the original CompatLayer.command(verb, unwrapped=False).
    ///
    /// With a runtime:
    ///   [runtime/_v2-entry-point, --verb=verb, --, shim, proton/proton, verb, exe, ...args]
    ///
    /// Without a runtime (fallback):
    ///   [shim, proton/proton, verb, exe, ...args]
    ///
    /// The caller appends [exe, ...launch_args] after the returned vec.
    pub fn build_command(&self, verb: &str) -> Vec<String> {
        let shim = shim_path();
        let shim_str = shim.to_string_lossy().into_owned();

        match self.required_runtime() {
            Some(variant) if is_runtime_installed(variant) => {
                let runtime_path = local_runtime_path(variant);
                let entry_point = Self::resolve_entry_point(&runtime_path);

                // SLR entry: [_v2-entry-point, --verb=verb, --, shim]
                let mut cmd = vec![
                    entry_point,
                    format!("--verb={verb}"),
                    "--".into(),
                    shim_str,
                ];

                // Proton part: [proton_path/proton, verb]
                cmd.extend(self.expand_commandline(verb));

                cmd
            }
            _ => {
                // No runtime or not installed — bare shim + proton
                let mut cmd = vec![shim_str];
                cmd.extend(self.expand_commandline(verb));
                cmd
            }
        }
    }

    pub fn resolve_entry_point_pub(runtime_path: &Path) -> String {
        Self::resolve_entry_point(runtime_path)
    }

    fn resolve_entry_point(runtime_path: &Path) -> String {
        // Prefer the mythix symlink, then the legacy looni one (backwards compat).
        for name in ["mythix", "looni"] {
            let link = runtime_path.join(name);
            if link.is_file() || link.is_symlink() {
                return link.to_string_lossy().into_owned();
            }
        }
        runtime_path
            .join("_v2-entry-point")
            .to_string_lossy()
            .into_owned()
    }
}

// ── Runtime status query (sync, for commands.rs) ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub variant: String,
    pub display_name: String,
    pub installed: bool,
    pub build_id: String,
    pub local_path: String,
}

pub fn get_runtime_info(variant: RuntimeVariant) -> RuntimeInfo {
    let installed = is_runtime_installed(variant);
    RuntimeInfo {
        variant: variant.repo_dir().into(),
        display_name: variant.display_name().into(),
        installed,
        build_id: if installed { current_build_id(variant) } else { String::new() },
        local_path: local_runtime_path(variant).to_string_lossy().into(),
    }
}

pub fn all_runtime_info() -> Vec<RuntimeInfo> {
    [RuntimeVariant::Sniper, RuntimeVariant::Soldier, RuntimeVariant::SteamRT4]
        .iter()
        .map(|v| get_runtime_info(*v))
        .collect()
}