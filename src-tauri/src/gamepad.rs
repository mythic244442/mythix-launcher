use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;

const JS_EVENT_BUTTON: u8 = 0x01;
const JS_EVENT_INIT: u8 = 0x80;
// Guide/Home button on the Linux js interface (NOT select/back which is 8 on Switch)
const GUIDE_BUTTON: u8 = 12;

static WINDOW_HIDDEN: AtomicBool = AtomicBool::new(false);

pub fn start_monitor(app: tauri::AppHandle) {
    thread::spawn(move || {
        eprintln!("[mythix] Gamepad monitor started");
        let mut active: HashMap<PathBuf, Arc<AtomicBool>> = HashMap::new();

        loop {
            // Clean up disconnected devices
            active.retain(|_, alive| alive.load(Ordering::Relaxed));

            // Scan for new js devices
            if let Ok(entries) = fs::read_dir("/dev/input") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("js") {
                        continue;
                    }
                    let path = entry.path();
                    if active.contains_key(&path) {
                        continue;
                    }

                    let file = match File::open(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("[mythix] Can't open {}: {} (need input group?)", path.display(), e);
                            continue;
                        }
                    };

                    eprintln!("[mythix] Monitoring gamepad: {}", path.display());
                    let alive = Arc::new(AtomicBool::new(true));
                    active.insert(path.clone(), alive.clone());

                    let app_clone = app.clone();
                    let path_clone = path.clone();
                    thread::spawn(move || {
                        monitor_device(file, &app_clone);
                        alive.store(false, Ordering::Relaxed);
                        eprintln!("[mythix] Gamepad disconnected: {}", path_clone.display());
                    });
                }
            }

            thread::sleep(Duration::from_secs(3));
        }
    });
}

fn monitor_device(mut file: File, app: &tauri::AppHandle) {
    let mut buf = [0u8; 8];
    let mut last_toggle = Instant::now() - Duration::from_secs(1);
    let mut event_count: u64 = 0;

    loop {
        match file.read_exact(&mut buf) {
            Ok(()) => {
                let value = i16::from_ne_bytes([buf[4], buf[5]]);
                let event_type = buf[6];
                let number = buf[7];

                if event_type & JS_EVENT_INIT != 0 {
                    continue;
                }

                event_count += 1;
                // Periodic heartbeat so we know the monitor is alive
                if event_count % 100 == 0 {
                    eprintln!("[mythix] Gamepad monitor alive ({} events received)", event_count);
                }

                if event_type == JS_EVENT_BUTTON && value == 1 {
                    eprintln!("[mythix] js button {} pressed", number);
                    if number == GUIDE_BUTTON {
                        if last_toggle.elapsed() > Duration::from_millis(800) {
                            eprintln!("[mythix] Guide -> bring_to_front (WINDOW_HIDDEN={})",
                                WINDOW_HIDDEN.load(Ordering::Relaxed));
                            bring_to_front(app);
                            last_toggle = Instant::now();
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[mythix] Gamepad monitor read error: {}", e);
                return;
            }
        }
    }
}

fn bring_to_front(app: &tauri::AppHandle) {
    use tauri::Emitter;
    if let Some(window) = app.get_webview_window("main") {
        let was_hidden = WINDOW_HIDDEN.load(Ordering::Relaxed);
        let is_focused = window.is_focused().unwrap_or(false);

        if is_focused && !was_hidden {
            // Launcher is focused and visible — let the frontend handle hiding
            return;
        }

        eprintln!("[mythix] Bringing launcher to front (was_hidden={}, is_focused={})", was_hidden, is_focused);

        if was_hidden {
            let _ = window.show();
            let _ = window.unminimize();
        }
        let _ = window.set_focus();
        activate_window();
        WINDOW_HIDDEN.store(false, Ordering::Relaxed);
        let _ = app.emit("gamepad:restored", ());
    }
}

pub fn notify_hidden() {
    WINDOW_HIDDEN.store(true, Ordering::Relaxed);
}

fn activate_window() {
    // KDE Wayland: use kdotool if available
    if let Ok(out) = std::process::Command::new("kdotool")
        .args(["search", "--name", "Mythix"])
        .output()
    {
        let ids = String::from_utf8_lossy(&out.stdout);
        for wid in ids.lines().filter(|l| !l.is_empty()) {
            if std::process::Command::new("kdotool")
                .args(["windowactivate", wid])
                .status()
                .is_ok()
            {
                eprintln!("[mythix] Activated via kdotool");
                return;
            }
        }
    }

    // KDE Wayland fallback: D-Bus to KWin
    if std::process::Command::new("gdbus")
        .args([
            "call", "--session",
            "--dest", "org.kde.KWin",
            "--object-path", "/KWin",
            "--method", "org.kde.KWin.activateWindow",
            "Mythix",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        eprintln!("[mythix] Activated via KWin D-Bus");
        return;
    }

    // X11 fallback: xdotool
    if let Ok(out) = std::process::Command::new("xdotool")
        .args(["search", "--name", "Mythix"])
        .output()
    {
        let ids = String::from_utf8_lossy(&out.stdout);
        for wid in ids.lines().filter(|l| !l.is_empty()) {
            let _ = std::process::Command::new("xdotool")
                .args(["windowactivate", "--sync", wid])
                .status();
        }
        eprintln!("[mythix] Activated via xdotool");
    }
}
