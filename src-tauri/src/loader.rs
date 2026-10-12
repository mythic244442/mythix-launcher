// loader.rs
// Vendored mythix-loader integration.
//
// The loader (Python profile manager + Windows stub exes) lives in <repo>/loader/
// and is embedded at build time. At runtime the launcher keeps its own copy in
// $XDG_DATA_HOME/mythix/loader/ — it never touches a user's ~/.mythix/mythix-loader
// — and drives the script with `--json` to create profiles in
// ~/.mythix/loader_profiles/<name>/. A profile's stub launches the game's real exe
// from its install folder with the real steamclient (Steam must be running).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::LauncherError;

const LOADER_SCRIPT: &[u8] = include_bytes!("../../loader/mythix-loader");
const LOADER_EXE_64: &[u8] = include_bytes!("../../loader/mythix_loader.exe");
const LOADER_EXE_32: &[u8] = include_bytes!("../../loader/mythix_loader_32.exe");

/// What `mythix-loader --json` reports for a planned (or created) profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoaderPlan {
    pub appid: String,
    pub name: String,
    /// The game's real install folder.
    pub game_dir: String,
    /// The real game exe the stub launches.
    pub exe: String,
    /// Arguments baked into the profile ini (empty — Steam's options are choices).
    #[serde(default)]
    pub args: String,
    /// Argument strings Steam's launch config offers for this exe (-dx12, -dx11…).
    /// Informational only; never baked into the profile.
    #[serde(default)]
    pub launch_options: Vec<String>,
    #[serde(default)]
    pub arch: u32,
    /// "appinfo" (Steam's own launch config), "heuristic", or "manual".
    #[serde(default)]
    pub launch_source: String,
    pub profile_name: String,
    pub profile_dir: String,
    /// The stub exe inside the profile — what the library entry launches.
    pub stub_path: String,
    #[serde(default)]
    pub ini_path: String,
    /// Steam's compatdata dir for this app, when it has one (clonable prefix).
    #[serde(default)]
    pub steam_prefix: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
    /// Filled in by the launcher (not the script): this appid is already in the
    /// library, and that entry already launches through a loader profile.
    #[serde(default)]
    pub in_library: bool,
    #[serde(default)]
    pub uses_loader: bool,
}

/// The launcher-managed copy of the loader: $XDG_DATA_HOME/mythix/loader.
pub fn loader_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("mythix").join("loader"))
}

/// ~/.mythix/loader_profiles — where the script writes profiles.
pub fn profiles_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".mythix").join("loader_profiles"))
}

/// Does this library exe point at a loader profile's stub?
pub fn is_loader_stub(exe: &Path) -> bool {
    profiles_dir().map(|d| exe.starts_with(&d)).unwrap_or(false)
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<(), LauncherError> {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes)?;
    }
    Ok(())
}

/// Install (or refresh) the embedded loader into `loader_dir()`. Only files whose
/// content changed are rewritten. Returns the directory.
pub fn install_loader() -> Result<PathBuf, LauncherError> {
    let dir = loader_dir()
        .ok_or_else(|| LauncherError::Io("Cannot resolve XDG_DATA_HOME".into()))?;
    fs::create_dir_all(&dir)?;

    let script = dir.join("mythix-loader");
    write_if_changed(&script, LOADER_SCRIPT)?;
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&script)?.permissions();
        if perm.mode() & 0o111 == 0 {
            perm.set_mode(0o755);
            fs::set_permissions(&script, perm)?;
        }
    }
    write_if_changed(&dir.join("mythix_loader.exe"), LOADER_EXE_64)?;
    write_if_changed(&dir.join("mythix_loader_32.exe"), LOADER_EXE_32)?;
    Ok(dir)
}

/// Plan (`dry_run`) or create a loader profile for the game in `game_dir`.
/// `appid` overrides detection from the Steam library.
pub fn run_loader(
    game_dir: &str,
    appid: Option<&str>,
    dry_run: bool,
) -> Result<LoaderPlan, LauncherError> {
    let dir = install_loader()?;

    let mut cmd = Command::new("python3");
    cmd.arg(dir.join("mythix-loader"))
        .arg(game_dir)
        .arg("--json")
        .arg("--loader-dir")
        .arg(&dir);
    if let Some(a) = appid.map(str::trim).filter(|a| !a.is_empty()) {
        cmd.arg("--appid").arg(a);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }

    let out = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LauncherError::Launch("python3 not found — the loader needs Python 3".into())
        } else {
            LauncherError::Launch(format!("could not run mythix-loader: {e}"))
        }
    })?;
    parse_loader_output(&out.stdout, &out.stderr)
}

/// The script prints exactly one JSON object on stdout: a plan (`"ok": true`) or
/// `{"ok": false, "error": "..."}`. Anything else is reported with stderr's tail.
pub fn parse_loader_output(stdout: &[u8], stderr: &[u8]) -> Result<LoaderPlan, LauncherError> {
    let text = String::from_utf8_lossy(stdout);
    let Some(line) = text.lines().rev().find(|l| l.trim_start().starts_with('{')) else {
        let err = strip_ansi(&String::from_utf8_lossy(stderr));
        let mut tail: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).rev().take(6).collect();
        tail.reverse();
        return Err(LauncherError::Launch(if tail.is_empty() {
            "mythix-loader gave no result".into()
        } else {
            format!("mythix-loader gave no result: {}", tail.join(" | "))
        }));
    };

    let v: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| LauncherError::Serialization(format!("mythix-loader output: {e}")))?;
    if v.get("ok").and_then(|o| o.as_bool()) != Some(true) {
        let msg = v.get("error").and_then(|e| e.as_str()).unwrap_or("unknown error");
        return Err(LauncherError::Launch(msg.to_string()));
    }
    serde_json::from_value(v)
        .map_err(|e| LauncherError::Serialization(format!("mythix-loader plan: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plan_and_ignores_noise() {
        let out = b"stray line\n{\"ok\": true, \"appid\": \"1144200\", \"name\": \"Ready or Not\", \
            \"game_dir\": \"/g\", \"exe\": \"/g/ReadyOrNot.exe\", \"args\": \"\", \
            \"launch_options\": [\"-dx12\", \"-dx11\"], \"arch\": 64, \
            \"launch_source\": \"appinfo\", \"profile_name\": \"ready-or-not\", \"profile_dir\": \"/p\", \
            \"stub_path\": \"/p/readyornot.exe\", \"ini_path\": \"/p/mythix_loader.ini\", \
            \"steam_prefix\": null, \"dry_run\": true}\n";
        let p = parse_loader_output(out, b"").unwrap();
        assert_eq!(p.appid, "1144200");
        assert_eq!(p.args, "");
        assert_eq!(p.launch_options, vec!["-dx12", "-dx11"]);
        assert_eq!(p.launch_source, "appinfo");
        assert!(p.steam_prefix.is_none());
        assert!(p.dry_run);
        assert!(!p.in_library && !p.uses_loader); // launcher-only fields default off
    }

    #[test]
    fn recognises_loader_stubs() {
        let d = profiles_dir().unwrap();
        assert!(is_loader_stub(&d.join("ready-or-not").join("readyornot.exe")));
        assert!(!is_loader_stub(Path::new("/games/Ready Or Not/ReadyOrNot.exe")));
    }

    #[test]
    fn script_errors_surface() {
        let e = parse_loader_output(b"{\"ok\": false, \"error\": \"No .exe files found\"}\n", b"")
            .unwrap_err();
        assert!(e.to_string().contains("No .exe files found"));
    }

    #[test]
    fn missing_json_reports_stderr_without_ansi() {
        let e = parse_loader_output(b"", b"\x1b[1;31merror:\x1b[0m boom\n").unwrap_err().to_string();
        assert!(e.contains("boom"));
        assert!(!e.contains('\u{1b}'));
    }

    #[test]
    fn embedded_loader_is_present() {
        assert!(LOADER_SCRIPT.starts_with(b"#!/usr/bin/env python3"));
        assert_eq!(&LOADER_EXE_64[..2], b"MZ");
        assert_eq!(&LOADER_EXE_32[..2], b"MZ");
    }
}

/// Drop ANSI colour sequences (the script colours its human-readable logs).
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(n) = chars.next() {
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
