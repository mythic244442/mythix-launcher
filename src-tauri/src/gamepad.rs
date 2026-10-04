use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;

const JS_EVENT_BUTTON: u8 = 0x01;
const JS_EVENT_INIT: u8 = 0x80;

static WINDOW_HIDDEN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ControllerType {
    Xbox,
    PlayStation,
    Switch,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PlayStationModel {
    DualShock3,
    DualShock4,
    DualSense,
    DualSenseEdge,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ControllerInfo {
    pub controller_type: ControllerType,
    pub ps_model: Option<PlayStationModel>,
    pub vid: u16,
    pub pid: u16,
    pub name: String,
}

impl ControllerType {
    fn guide_button(self) -> u8 {
        match self {
            ControllerType::Xbox => 8,
            ControllerType::PlayStation => 10,
            ControllerType::Switch => 12,
            ControllerType::Unknown => 8,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ControllerType::Xbox => "xbox",
            ControllerType::PlayStation => "playstation",
            ControllerType::Switch => "switch",
            ControllerType::Unknown => "xbox",
        }
    }

    pub fn sdl_type_str(self, ps_model: Option<PlayStationModel>) -> &'static str {
        match self {
            ControllerType::PlayStation => match ps_model {
                Some(PlayStationModel::DualSense | PlayStationModel::DualSenseEdge) => "PS5",
                _ => "PS4",
            },
            ControllerType::Switch => "SwitchPro",
            _ => "XBox360",
        }
    }
}

fn detect_type_from_vid_pid(vid: u16, pid: u16) -> (ControllerType, Option<PlayStationModel>) {
    match vid {
        0x054c => {
            let model = match pid {
                0x0268 => Some(PlayStationModel::DualShock3),
                0x05c4 | 0x09cc => Some(PlayStationModel::DualShock4),
                0x0ce6 => Some(PlayStationModel::DualSense),
                0x0df2 => Some(PlayStationModel::DualSenseEdge),
                _ => None,
            };
            (ControllerType::PlayStation, model)
        }
        0x045e | 0x0738 | 0x0e6f | 0x24c6 | 0x1532 => {
            (ControllerType::Xbox, None)
        }
        0x057e => {
            (ControllerType::Switch, None)
        }
        _ => (ControllerType::Unknown, None),
    }
}

fn detect_type_from_name(name: &str) -> ControllerType {
    let lower = name.to_ascii_lowercase();
    if lower.contains("dualshock") || lower.contains("dualsense")
        || lower.contains("sony") || lower.contains("playstation")
        || lower.contains("ps3") || lower.contains("ps4") || lower.contains("ps5")
        || lower.contains("054c")
    {
        ControllerType::PlayStation
    } else if lower.contains("nintendo") || lower.contains("switch")
        || lower.contains("pro controller") || lower.contains("joy-con")
    {
        ControllerType::Switch
    } else if lower.contains("xbox") || lower.contains("microsoft")
        || lower.contains("x-box") || lower.contains("xinput")
    {
        ControllerType::Xbox
    } else {
        ControllerType::Unknown
    }
}

fn read_hidraw_vid_pid(js_path: &std::path::Path) -> Option<(u16, u16)> {
    let js_name = js_path.file_name()?.to_str()?;
    let js_num = js_name.strip_prefix("js")?;
    let input_dir = format!("/sys/class/input/js{}/device", js_num);
    let input_path = std::path::Path::new(&input_dir);
    if !input_path.is_dir() { return None; }

    let uevent = fs::read_to_string(input_path.join("uevent")).ok()?;
    for line in uevent.lines() {
        if let Some(rest) = line.strip_prefix("HID_ID=") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() == 3 {
                let vid = u16::from_str_radix(parts[1].trim_start_matches("0000"), 16).ok()?;
                let pid = u16::from_str_radix(parts[2].trim_start_matches("0000"), 16).ok()?;
                return Some((vid, pid));
            }
        }
    }

    for line in uevent.lines() {
        if let Some(rest) = line.strip_prefix("PRODUCT=") {
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() >= 3 {
                let vid = u16::from_str_radix(parts[1], 16).ok()?;
                let pid = u16::from_str_radix(parts[2], 16).ok()?;
                return Some((vid, pid));
            }
        }
    }
    None
}

pub fn scan_hidraw_controllers() -> Vec<ControllerInfo> {
    let mut controllers = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hidraw") else { return controllers };

    for entry in entries.flatten() {
        let hidraw_name = entry.file_name().to_string_lossy().to_string();
        let device_dir = entry.path().join("device");
        let uevent_path = device_dir.join("uevent");
        let Ok(uevent) = fs::read_to_string(&uevent_path) else { continue };

        let mut vid: u16 = 0;
        let mut pid: u16 = 0;
        let mut name = String::new();

        for line in uevent.lines() {
            if let Some(rest) = line.strip_prefix("HID_ID=") {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 3 {
                    vid = u16::from_str_radix(parts[1].trim_start_matches("0000"), 16).unwrap_or(0);
                    pid = u16::from_str_radix(parts[2].trim_start_matches("0000"), 16).unwrap_or(0);
                }
            }
            if let Some(rest) = line.strip_prefix("HID_NAME=") {
                name = rest.to_string();
            }
        }

        if vid == 0 { continue; }
        let (controller_type, ps_model) = detect_type_from_vid_pid(vid, pid);
        if controller_type == ControllerType::Unknown { continue; }

        eprintln!("[mythix] hidraw {} — {:04x}:{:04x} \"{}\" → {}",
                  hidraw_name, vid, pid, name, controller_type.as_str());

        controllers.push(ControllerInfo {
            controller_type,
            ps_model,
            vid,
            pid,
            name,
        });
    }
    controllers
}

static CURRENT_INFO: Mutex<Option<ControllerInfo>> = Mutex::new(None);

fn read_device_name(file: &File) -> Option<String> {
    // JSIOCGNAME(len) = _IOC(_IOC_READ, 'j', 0x13, len)
    const IOC_READ: u64 = 2;
    const fn jsiocgname(len: u64) -> u64 {
        (IOC_READ << 30) | ((b'j' as u64) << 8) | (0x13) | (len << 16)
    }

    let mut buf = [0u8; 256];
    let ret = unsafe {
        libc::ioctl(file.as_raw_fd(), jsiocgname(256), buf.as_mut_ptr())
    };
    if ret < 0 {
        return None;
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Some(String::from_utf8_lossy(&buf[..end]).to_string())
}

pub fn current_type() -> ControllerType {
    CURRENT_INFO.lock().unwrap().as_ref()
        .map(|i| i.controller_type)
        .unwrap_or(ControllerType::Unknown)
}

pub fn current_info() -> Option<ControllerInfo> {
    CURRENT_INFO.lock().unwrap().clone()
}

pub fn start_monitor(app: tauri::AppHandle) {
    thread::spawn(move || {
        eprintln!("[mythix] Gamepad monitor started");
        let mut active: HashMap<PathBuf, Arc<AtomicBool>> = HashMap::new();

        loop {
            active.retain(|_, alive| alive.load(Ordering::Relaxed));

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

                    let dev_name = read_device_name(&file).unwrap_or_default();

                    let (controller_type, ps_model, vid, pid) =
                        if let Some((v, p)) = read_hidraw_vid_pid(&path) {
                            let (ct, pm) = detect_type_from_vid_pid(v, p);
                            (ct, pm, v, p)
                        } else {
                            (detect_type_from_name(&dev_name), None, 0, 0)
                        };

                    eprintln!("[mythix] Monitoring gamepad: {} — \"{}\" {:04x}:{:04x} ({})",
                              path.display(), dev_name, vid, pid, controller_type.as_str());

                    let info = ControllerInfo {
                        controller_type,
                        ps_model,
                        vid,
                        pid,
                        name: dev_name.clone(),
                    };
                    *CURRENT_INFO.lock().unwrap() = Some(info);

                    {
                        use tauri::Emitter;
                        let _ = app.emit("gamepad:type", controller_type.as_str());
                    }

                    let alive = Arc::new(AtomicBool::new(true));
                    active.insert(path.clone(), alive.clone());

                    let app_clone = app.clone();
                    let path_clone = path.clone();
                    thread::spawn(move || {
                        monitor_device(file, &app_clone, controller_type);
                        alive.store(false, Ordering::Relaxed);
                        eprintln!("[mythix] Gamepad disconnected: {}", path_clone.display());
                    });
                }
            }

            thread::sleep(Duration::from_secs(3));
        }
    });
}

fn monitor_device(mut file: File, app: &tauri::AppHandle, controller_type: ControllerType) {
    let mut buf = [0u8; 8];
    let mut last_toggle = Instant::now() - Duration::from_secs(1);
    let guide_btn = controller_type.guide_button();

    loop {
        match file.read_exact(&mut buf) {
            Ok(()) => {
                let value = i16::from_ne_bytes([buf[4], buf[5]]);
                let event_type = buf[6];
                let number = buf[7];

                if event_type & JS_EVENT_INIT != 0 {
                    continue;
                }

                if event_type == JS_EVENT_BUTTON && value == 1 {
                    if number == guide_btn {
                        if last_toggle.elapsed() > Duration::from_millis(800) {
                            eprintln!("[mythix] Guide (btn {}) -> bring_to_front", guide_btn);
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
