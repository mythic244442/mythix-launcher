//! System-level gamepad monitor.
//!
//! Reads every `/dev/input/js*` node, turns raw joydev button/axis events into
//! semantic UI actions (`"up"`, `"a"`, `"lb"`, `"rs_down"`, …) and emits them to
//! the webview as `gamepad:action` → `[action, pressed]`.
//!
//! How it stays sane:
//! * Buttons and axes are mapped by their **kernel event code** (JSIOCGBTNMAP /
//!   JSIOCGAXMAP), not by joydev index, so xpad, xpadneo, xone, hid-playstation,
//!   hid-nintendo and generic pads all land on the same actions.
//! * Every device feeds **one merged action state**. An action is pressed while
//!   any source holds it, and press/release are only emitted on edges, so a pad
//!   that shows up twice (physical + Steam's virtual pad, d-pad hat + d-pad
//!   buttons, analog + digital triggers) can never double-step the UI.
//! * Motion-sensor / accelerometer nodes (the DualSense exposes one as a js
//!   device) are skipped.
//! * Sticks produce ONE dominant direction with a radial deadzone and
//!   hysteresis, so a diagonal or jittery push doesn't fire two moves.
//! * Input is paused while another window (usually the game) has focus, so
//!   playing can't click buttons in the launcher sitting behind it. Guide always
//!   works.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

const JS_EVENT_BUTTON: u8 = 0x01;
const JS_EVENT_AXIS: u8 = 0x02;
const JS_EVENT_INIT: u8 = 0x80;

// ── linux/input-event-codes.h ───────────────────────────────────────────────
const ABS_X: u8 = 0x00;
const ABS_Y: u8 = 0x01;
const ABS_Z: u8 = 0x02;
const ABS_RX: u8 = 0x03;
const ABS_RY: u8 = 0x04;
const ABS_RZ: u8 = 0x05;
const ABS_GAS: u8 = 0x09;
const ABS_BRAKE: u8 = 0x0a;
const ABS_HAT0X: u8 = 0x10;
const ABS_HAT0Y: u8 = 0x11;

const BTN_JOYSTICK: u16 = 0x120;
const BTN_SOUTH: u16 = 0x130;
const BTN_EAST: u16 = 0x131;
const BTN_NORTH: u16 = 0x133;
const BTN_WEST: u16 = 0x134;
const BTN_TL: u16 = 0x136;
const BTN_TR: u16 = 0x137;
const BTN_TL2: u16 = 0x138;
const BTN_TR2: u16 = 0x139;
const BTN_SELECT: u16 = 0x13a;
const BTN_START: u16 = 0x13b;
const BTN_MODE: u16 = 0x13c;
const BTN_THUMBR: u16 = 0x13e;
const BTN_DPAD_UP: u16 = 0x220;
const BTN_DPAD_DOWN: u16 = 0x221;
const BTN_DPAD_LEFT: u16 = 0x222;
const BTN_DPAD_RIGHT: u16 = 0x223;

const INPUT_PROP_ACCELEROMETER: u32 = 0x06;

// ── Tuning (values normalised to -1.0..1.0) ─────────────────────────────────
/// Stick must pass this to start a direction…
const STICK_PRESS: f32 = 0.55;
/// …and fall below this to end it (hysteresis kills edge jitter).
const STICK_RELEASE: f32 = 0.35;
/// While held, the other axis must beat the current one by this ratio before
/// the direction flips — keeps ~45° pushes from flickering between two dirs.
const STICK_SWITCH_RATIO: f32 = 1.3;
const HAT_THRESHOLD: f32 = 0.5;
/// joydev maps a released analog trigger to -1.0 and fully pulled to +1.0.
const TRIGGER_PRESS: f32 = 0.3;
const TRIGGER_RELEASE: f32 = -0.2;

const LEFT_STICK_DIRS: [&str; 4] = ["up", "down", "left", "right"];
const RIGHT_STICK_DIRS: [&str; 4] = ["rs_up", "rs_down", "rs_left", "rs_right"];

static WINDOW_HIDDEN: AtomicBool = AtomicBool::new(false);
/// Starts true: if the compositor never reports focus we'd rather work than
/// silently ignore the controller.
static WINDOW_FOCUSED: AtomicBool = AtomicBool::new(true);
static NEXT_DEVICE_ID: AtomicU32 = AtomicU32::new(1);

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

    /// xpad / xpadneo / xone report the Xbox *labels* (BTN_X = 0x133 is the
    /// left button); hid-playstation and hid-nintendo report *positions*
    /// (0x133 = north). We normalise to positional actions: "x" = west, "y" = north.
    fn positional_face_buttons(self) -> bool {
        matches!(self, ControllerType::PlayStation | ControllerType::Switch)
    }

    /// ABS_Z/ABS_RZ are triggers on known pads, but generic DirectInput pads
    /// often use them for the right stick — don't guess there.
    fn analog_triggers_are_trustworthy(self) -> bool {
        matches!(self, ControllerType::Xbox | ControllerType::PlayStation)
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Never let a poisoned lock take the launcher down (release profile aborts on panic).
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ── Controller identification ───────────────────────────────────────────────

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
        0x045e | 0x0738 | 0x0e6f | 0x24c6 | 0x1532 => (ControllerType::Xbox, None),
        0x057e => (ControllerType::Switch, None),
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

fn read_hidraw_vid_pid(js_path: &Path) -> Option<(u16, u16)> {
    let js_name = js_path.file_name()?.to_str()?;
    let js_num = js_name.strip_prefix("js")?;
    let input_dir = format!("/sys/class/input/js{}/device", js_num);
    let input_path = Path::new(&input_dir);
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

        controllers.push(ControllerInfo { controller_type, ps_model, vid, pid, name });
    }
    controllers
}

// ── joydev ioctls ───────────────────────────────────────────────────────────

const fn jsioc_read(nr: u64, size: u64) -> u64 {
    // _IOC(_IOC_READ, 'j', nr, size)
    (2u64 << 30) | (size << 16) | ((b'j' as u64) << 8) | nr
}
const JSIOCGAXES: u64 = jsioc_read(0x11, 1);
const JSIOCGBUTTONS: u64 = jsioc_read(0x12, 1);
const JSIOCGAXMAP: u64 = jsioc_read(0x32, 64); // __u8[ABS_CNT]
const JSIOCGBTNMAP: u64 = jsioc_read(0x34, 1024); // __u16[KEY_MAX - BTN_MISC + 1]

fn read_device_name(file: &File) -> Option<String> {
    let mut buf = [0u8; 256];
    let ret = unsafe { libc::ioctl(file.as_raw_fd(), jsioc_read(0x13, 256) as _, buf.as_mut_ptr()) };
    if ret < 0 {
        return None;
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Some(String::from_utf8_lossy(&buf[..end]).to_string())
}

/// joydev index → kernel code tables for one device.
struct Layout {
    axes: Vec<u8>,
    buttons: Vec<u16>,
}

fn query_layout(file: &File) -> Option<Layout> {
    let fd = file.as_raw_fd();
    let mut naxes: u8 = 0;
    let mut nbuttons: u8 = 0;
    let mut axmap = [0u8; 64];
    let mut btnmap = [0u16; 512];
    unsafe {
        if libc::ioctl(fd, JSIOCGAXES as _, &mut naxes as *mut u8) < 0 { return None; }
        if libc::ioctl(fd, JSIOCGBUTTONS as _, &mut nbuttons as *mut u8) < 0 { return None; }
        if libc::ioctl(fd, JSIOCGAXMAP as _, axmap.as_mut_ptr()) < 0 { return None; }
        if libc::ioctl(fd, JSIOCGBTNMAP as _, btnmap.as_mut_ptr()) < 0 { return None; }
    }
    Some(Layout {
        axes: axmap.iter().take(naxes as usize).copied().collect(),
        buttons: btnmap.iter().take(nbuttons as usize).copied().collect(),
    })
}

/// Only used if the map ioctls fail (they shouldn't on any modern kernel):
/// the classic xpad / hid-playstation joydev ordering.
fn legacy_layout(ct: ControllerType) -> Layout {
    let axes = vec![ABS_X, ABS_Y, ABS_Z, ABS_RX, ABS_RY, ABS_RZ, ABS_HAT0X, ABS_HAT0Y];
    let buttons = match ct {
        ControllerType::PlayStation => vec![
            BTN_SOUTH, BTN_EAST, BTN_NORTH, BTN_WEST, BTN_TL, BTN_TR,
            BTN_TL2, BTN_TR2, BTN_SELECT, BTN_START, BTN_MODE,
        ],
        _ => vec![
            BTN_SOUTH, BTN_EAST, BTN_NORTH, BTN_WEST, BTN_TL, BTN_TR,
            BTN_SELECT, BTN_START, BTN_MODE,
        ],
    };
    Layout { axes, buttons }
}

fn sysfs_device_attr(js_path: &Path, attr: &str) -> Option<String> {
    let js = js_path.file_name()?.to_str()?;
    fs::read_to_string(format!("/sys/class/input/{}/device/{}", js, attr))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Accelerometer / gyro / touchpad nodes look like joysticks to joydev but
/// spew axis noise. The DualSense "Motion Sensors" device is the usual suspect.
fn is_motion_or_touch_device(js_path: &Path, name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.contains("motion sensor") || lower.contains("accelerometer") || lower.contains("touchpad") {
        return true;
    }
    if let Some(props) = sysfs_device_attr(js_path, "properties") {
        if let Some(word) = props.split_whitespace().last() {
            if let Ok(bits) = u64::from_str_radix(word, 16) {
                return bits & (1u64 << INPUT_PROP_ACCELEROMETER) != 0;
            }
        }
    }
    false
}

fn has_gamepad_buttons(layout: &Layout) -> bool {
    layout.buttons.iter().any(|&c| {
        (BTN_JOYSTICK..=BTN_THUMBR).contains(&c) || (BTN_DPAD_UP..=BTN_DPAD_RIGHT).contains(&c)
    })
}

// ── Semantic mapping ────────────────────────────────────────────────────────

fn button_action(ct: ControllerType, code: u16) -> Option<&'static str> {
    let positional = ct.positional_face_buttons();
    Some(match code {
        BTN_SOUTH => "a",
        BTN_EAST => "b",
        BTN_NORTH => if positional { "y" } else { "x" },
        BTN_WEST => if positional { "x" } else { "y" },
        BTN_TL => "lb",
        BTN_TR => "rb",
        BTN_TL2 => "lt",
        BTN_TR2 => "rt",
        BTN_SELECT => "back",
        BTN_START => "start",
        BTN_MODE => "guide",
        BTN_DPAD_UP => "up",
        BTN_DPAD_DOWN => "down",
        BTN_DPAD_LEFT => "left",
        BTN_DPAD_RIGHT => "right",
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisRole {
    LeftX,
    LeftY,
    RightX,
    RightY,
    HatX,
    HatY,
    TriggerL,
    TriggerR,
    Ignored,
}

fn axis_role(ct: ControllerType, code: u8) -> AxisRole {
    let triggers = ct.analog_triggers_are_trustworthy();
    match code {
        ABS_X => AxisRole::LeftX,
        ABS_Y => AxisRole::LeftY,
        ABS_RX => AxisRole::RightX,
        ABS_RY => AxisRole::RightY,
        ABS_HAT0X => AxisRole::HatX,
        ABS_HAT0Y => AxisRole::HatY,
        ABS_Z | ABS_BRAKE if triggers => AxisRole::TriggerL,
        ABS_RZ | ABS_GAS if triggers => AxisRole::TriggerR,
        _ => AxisRole::Ignored,
    }
}

/// Index into a `[up, down, left, right]` table, or None when centred.
/// Radial deadzone + hysteresis + dominant-axis selection.
fn stick_direction(x: f32, y: f32, current: Option<usize>) -> Option<usize> {
    let magnitude = (x * x + y * y).sqrt();
    let threshold = if current.is_some() { STICK_RELEASE } else { STICK_PRESS };
    if magnitude < threshold {
        return None;
    }
    let candidate = if x.abs() > y.abs() {
        if x < 0.0 { 2 } else { 3 }
    } else if y < 0.0 {
        0
    } else {
        1
    };
    match current {
        Some(cur) if cur != candidate => {
            let toward = |d: usize| match d { 0 => -y, 1 => y, 2 => -x, _ => x };
            let cur_push = toward(cur);
            if cur_push > 0.0 && toward(candidate) < cur_push * STICK_SWITCH_RATIO {
                Some(cur)
            } else {
                Some(candidate)
            }
        }
        _ => Some(candidate),
    }
}

// ── Merged action state ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hold {
    action: &'static str,
    device: u32,
    source: u32,
}

/// Every (device, source) currently holding an action. An action is "down"
/// while at least one hold exists for it.
static HOLDS: Mutex<Vec<Hold>> = Mutex::new(Vec::new());

fn emit_action(app: &tauri::AppHandle, action: &'static str, pressed: bool) {
    let _ = app.emit("gamepad:action", (action, pressed));
}

fn set_action(app: &tauri::AppHandle, action: &'static str, device: u32, source: u32, down: bool) {
    let edge = {
        let mut holds = lock(&HOLDS);
        let was_down = holds.iter().any(|h| h.action == action);
        if down {
            // Paused while another window has focus — except Guide, which is
            // how you get back here.
            if action != "guide" && !WINDOW_FOCUSED.load(Ordering::Relaxed) {
                return;
            }
            let hold = Hold { action, device, source };
            if !holds.contains(&hold) {
                holds.push(hold);
            }
        } else {
            holds.retain(|h| !(h.action == action && h.device == device && h.source == source));
        }
        let is_down = holds.iter().any(|h| h.action == action);
        (was_down != is_down).then_some(is_down)
    };
    if let Some(pressed) = edge {
        emit_action(app, action, pressed);
    }
}

/// Drop every hold matching `filter` and emit releases for actions that end up up.
fn release_where(app: &tauri::AppHandle, filter: impl Fn(&Hold) -> bool) {
    let released: Vec<&'static str> = {
        let mut holds = lock(&HOLDS);
        let before: HashSet<&'static str> = holds.iter().map(|h| h.action).collect();
        holds.retain(|h| !filter(h));
        let after: HashSet<&'static str> = holds.iter().map(|h| h.action).collect();
        before.difference(&after).copied().collect()
    };
    for action in released {
        emit_action(app, action, false);
    }
}

/// Called from the window-event hook in lib.rs.
pub fn set_window_focused(app: &tauri::AppHandle, focused: bool) {
    let was = WINDOW_FOCUSED.swap(focused, Ordering::Relaxed);
    if was != focused {
        eprintln!("[mythix] Launcher {} focus — controller nav {}",
                  if focused { "gained" } else { "lost" },
                  if focused { "active" } else { "paused" });
    }
    if !focused {
        // Let go of everything so the UI stops auto-repeating a held direction.
        release_where(app, |_| true);
    }
}

// ── Device registry (type + connection status for the UI) ───────────────────

static DEVICES: Mutex<Vec<(u32, ControllerInfo)>> = Mutex::new(Vec::new());
static CURRENT_INFO: Mutex<Option<ControllerInfo>> = Mutex::new(None);

pub fn current_type() -> ControllerType {
    lock(&CURRENT_INFO).as_ref()
        .map(|i| i.controller_type)
        .unwrap_or(ControllerType::Unknown)
}

pub fn current_info() -> Option<ControllerInfo> {
    lock(&CURRENT_INFO).clone()
}

pub fn connected_count() -> usize {
    lock(&DEVICES).len()
}

/// Re-pick the "current" controller (newest device with a known type wins)
/// and tell the UI about type / connection changes.
fn refresh_devices(app: &tauri::AppHandle) {
    let (count, pick) = {
        let devices = lock(&DEVICES);
        let pick = devices.iter().rev()
            .find(|(_, info)| info.controller_type != ControllerType::Unknown)
            .or_else(|| devices.last())
            .map(|(_, info)| info.clone());
        (devices.len(), pick)
    };

    let type_changed = {
        let mut current = lock(&CURRENT_INFO);
        let old = current.as_ref().map(|i| i.controller_type.as_str());
        // Keep the last known pad when everything disconnects so a launch
        // right after a Bluetooth blip still gets the right glyph config.
        if let Some(info) = pick {
            let changed = old != Some(info.controller_type.as_str());
            *current = Some(info);
            changed
        } else {
            false
        }
    };

    if type_changed {
        let _ = app.emit("gamepad:type", current_type().as_str());
    }
    let _ = app.emit("gamepad:connected", count);
}

// ── Monitor ─────────────────────────────────────────────────────────────────

struct Stick {
    x: f32,
    y: f32,
    dir: Option<usize>,
}

impl Stick {
    const fn new() -> Self {
        Self { x: 0.0, y: 0.0, dir: None }
    }
}

struct DeviceState {
    id: u32,
    axes: Vec<AxisRole>,
    buttons: Vec<Option<&'static str>>,
    left: Stick,
    right: Stick,
    hat: [Option<&'static str>; 2],
    trigger: [bool; 2],
}

const SRC_BUTTON: u32 = 0x1000;
const SRC_HAT: u32 = 0x2000;
const SRC_LEFT_STICK: u32 = 0x3000;
const SRC_RIGHT_STICK: u32 = 0x3001;
const SRC_TRIGGER: u32 = 0x4000;

static LAST_GUIDE: Mutex<Option<Instant>> = Mutex::new(None);

pub fn start_monitor(app: tauri::AppHandle) {
    thread::spawn(move || {
        eprintln!("[mythix] Gamepad monitor started");
        let mut active: HashMap<PathBuf, Arc<AtomicBool>> = HashMap::new();
        // Nodes we decided not to monitor, keyed by path + identity so a new
        // device reusing the same jsN still gets picked up.
        let mut skipped: HashSet<String> = HashSet::new();

        loop {
            active.retain(|_, alive| alive.load(Ordering::Relaxed));

            if let Ok(entries) = fs::read_dir("/dev/input") {
                let mut seen_skip_keys = HashSet::new();
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if !file_name.starts_with("js") {
                        continue;
                    }
                    let path = entry.path();
                    if active.contains_key(&path) {
                        continue;
                    }

                    let skip_key = format!(
                        "{}|{}|{}",
                        path.display(),
                        sysfs_device_attr(&path, "name").unwrap_or_default(),
                        sysfs_device_attr(&path, "uevent").unwrap_or_default(),
                    );
                    seen_skip_keys.insert(skip_key.clone());
                    if skipped.contains(&skip_key) {
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
                            let ct = if ct == ControllerType::Unknown { detect_type_from_name(&dev_name) } else { ct };
                            (ct, pm, v, p)
                        } else {
                            (detect_type_from_name(&dev_name), None, 0, 0)
                        };

                    let layout = query_layout(&file).unwrap_or_else(|| {
                        eprintln!("[mythix] {}: axis/button map ioctls failed, using legacy layout", path.display());
                        legacy_layout(controller_type)
                    });

                    if is_motion_or_touch_device(&path, &dev_name) || !has_gamepad_buttons(&layout) {
                        eprintln!("[mythix] Ignoring {} — \"{}\" (not a gamepad: motion/touch node or no buttons)",
                                  path.display(), dev_name);
                        skipped.insert(skip_key);
                        continue;
                    }

                    let id = NEXT_DEVICE_ID.fetch_add(1, Ordering::Relaxed);
                    let state = DeviceState {
                        id,
                        axes: layout.axes.iter().map(|&code| axis_role(controller_type, code)).collect(),
                        buttons: layout.buttons.iter().map(|&code| button_action(controller_type, code)).collect(),
                        left: Stick::new(),
                        right: Stick::new(),
                        hat: [None, None],
                        trigger: [false, false],
                    };

                    eprintln!(
                        "[mythix] Monitoring gamepad: {} — \"{}\" {:04x}:{:04x} ({}) axes={:?} buttons={:?}",
                        path.display(), dev_name, vid, pid, controller_type.as_str(),
                        layout.axes, layout.buttons
                    );

                    lock(&DEVICES).push((id, ControllerInfo {
                        controller_type,
                        ps_model,
                        vid,
                        pid,
                        name: dev_name.clone(),
                    }));
                    refresh_devices(&app);

                    let alive = Arc::new(AtomicBool::new(true));
                    active.insert(path.clone(), alive.clone());

                    let app_clone = app.clone();
                    let path_clone = path.clone();
                    thread::spawn(move || {
                        let mut state = state;
                        monitor_device(file, &app_clone, &mut state);
                        alive.store(false, Ordering::Relaxed);
                        release_where(&app_clone, |h| h.device == id);
                        lock(&DEVICES).retain(|(dev, _)| *dev != id);
                        refresh_devices(&app_clone);
                        eprintln!("[mythix] Gamepad disconnected: {}", path_clone.display());
                    });
                }
                // Forget skip decisions for nodes that went away.
                skipped.retain(|k| seen_skip_keys.contains(k));
            }

            thread::sleep(Duration::from_secs(3));
        }
    });
}

fn monitor_device(mut file: File, app: &tauri::AppHandle, st: &mut DeviceState) {
    let mut buf = [0u8; 8];
    loop {
        if let Err(e) = file.read_exact(&mut buf) {
            eprintln!("[mythix] Gamepad monitor read error: {}", e);
            return;
        }
        let value = i16::from_ne_bytes([buf[4], buf[5]]);
        let event_type = buf[6];
        let number = buf[7] as usize;
        let init = event_type & JS_EVENT_INIT != 0;

        match event_type & !JS_EVENT_INIT {
            JS_EVENT_BUTTON => {
                if init {
                    continue; // initial-state snapshot, not a press
                }
                let Some(Some(action)) = st.buttons.get(number).copied() else { continue };
                let down = value != 0;
                if action == "guide" && down {
                    let mut last = lock(&LAST_GUIDE);
                    if last.map_or(true, |t| t.elapsed() > Duration::from_millis(800)) {
                        *last = Some(Instant::now());
                        drop(last);
                        bring_to_front(app);
                    }
                }
                set_action(app, action, st.id, SRC_BUTTON + number as u32, down);
            }
            JS_EVENT_AXIS => {
                let role = st.axes.get(number).copied().unwrap_or(AxisRole::Ignored);
                let v = (value as f32 / 32767.0).clamp(-1.0, 1.0);
                handle_axis(app, st, role, number, v, init);
            }
            _ => {}
        }
    }
}

fn handle_axis(app: &tauri::AppHandle, st: &mut DeviceState, role: AxisRole, number: usize, v: f32, init: bool) {
    let id = st.id;
    match role {
        AxisRole::LeftX | AxisRole::LeftY | AxisRole::RightX | AxisRole::RightY => {
            let left = matches!(role, AxisRole::LeftX | AxisRole::LeftY);
            let (stick, names, source) = if left {
                (&mut st.left, &LEFT_STICK_DIRS, SRC_LEFT_STICK)
            } else {
                (&mut st.right, &RIGHT_STICK_DIRS, SRC_RIGHT_STICK)
            };
            if matches!(role, AxisRole::LeftX | AxisRole::RightX) { stick.x = v; } else { stick.y = v; }
            if init {
                return;
            }
            let new_dir = stick_direction(stick.x, stick.y, stick.dir);
            if new_dir != stick.dir {
                let old_dir = stick.dir;
                stick.dir = new_dir;
                if let Some(old) = old_dir {
                    set_action(app, names[old], id, source, false);
                }
                if let Some(new) = new_dir {
                    set_action(app, names[new], id, source, true);
                }
            }
        }
        AxisRole::HatX | AxisRole::HatY => {
            let slot = if role == AxisRole::HatX { 0 } else { 1 };
            let new_dir = match (role, v) {
                (AxisRole::HatX, v) if v < -HAT_THRESHOLD => Some("left"),
                (AxisRole::HatX, v) if v > HAT_THRESHOLD => Some("right"),
                (AxisRole::HatY, v) if v < -HAT_THRESHOLD => Some("up"),
                (AxisRole::HatY, v) if v > HAT_THRESHOLD => Some("down"),
                _ => None,
            };
            if init {
                st.hat[slot] = None;
                return;
            }
            let old_dir = st.hat[slot];
            if new_dir != old_dir {
                st.hat[slot] = new_dir;
                let source = SRC_HAT + number as u32;
                if let Some(old) = old_dir {
                    set_action(app, old, id, source, false);
                }
                if let Some(new) = new_dir {
                    set_action(app, new, id, source, true);
                }
            }
        }
        AxisRole::TriggerL | AxisRole::TriggerR => {
            let slot = if role == AxisRole::TriggerL { 0 } else { 1 };
            let was = st.trigger[slot];
            let now = if was { v > TRIGGER_RELEASE } else { v > TRIGGER_PRESS };
            st.trigger[slot] = now;
            if init || now == was {
                return;
            }
            let action = if slot == 0 { "lt" } else { "rt" };
            set_action(app, action, id, SRC_TRIGGER + number as u32, now);
        }
        AxisRole::Ignored => {}
    }
}

// ── Window restore ──────────────────────────────────────────────────────────

fn bring_to_front(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let was_hidden = WINDOW_HIDDEN.load(Ordering::Relaxed);
        let is_focused = window.is_focused().unwrap_or(false);

        if is_focused && !was_hidden {
            return;
        }

        eprintln!("[mythix] Bringing launcher to front (was_hidden={}, is_focused={})", was_hidden, is_focused);

        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        activate_window();
        WINDOW_HIDDEN.store(false, Ordering::Relaxed);
        // Treat a Guide restore as focused even if the compositor refuses to
        // activate us (KDE/Wayland focus-stealing prevention) — otherwise the
        // pad would look dead until the user clicks the window.
        WINDOW_FOCUSED.store(true, Ordering::Relaxed);
        let _ = app.emit("gamepad:restored", ());
    }
}

pub fn notify_hidden() {
    WINDOW_HIDDEN.store(true, Ordering::Relaxed);
}

fn activate_window() {
    if let Ok(out) = std::process::Command::new("kdotool")
        .args(["search", "--name", "mythix"])
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
            "mythix",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        eprintln!("[mythix] Activated via KWin D-Bus");
        return;
    }

    if let Ok(out) = std::process::Command::new("xdotool")
        .args(["search", "--name", "mythix"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_needs_real_push_and_hysteresis() {
        assert_eq!(stick_direction(0.0, 0.5, None), None);
        assert_eq!(stick_direction(0.0, 0.6, None), Some(1));
        // held: stays down until below the release threshold
        assert_eq!(stick_direction(0.0, 0.4, Some(1)), Some(1));
        assert_eq!(stick_direction(0.0, 0.3, Some(1)), None);
    }

    #[test]
    fn stick_diagonal_picks_one_direction_without_flicker() {
        // slightly-right-of-down push → down only
        assert_eq!(stick_direction(0.5, 0.7, None), Some(1));
        // wobble across 45° while held keeps the original direction
        assert_eq!(stick_direction(0.72, 0.7, Some(1)), Some(1));
        assert_eq!(stick_direction(0.7, 0.72, Some(3)), Some(3));
        // a clear move to the side does switch
        assert_eq!(stick_direction(0.95, 0.2, Some(1)), Some(3));
        // reversing past centre switches immediately
        assert_eq!(stick_direction(0.0, -0.8, Some(1)), Some(0));
    }

    #[test]
    fn face_buttons_are_positional() {
        // xpad reports Xbox X (west) as 0x133
        assert_eq!(button_action(ControllerType::Xbox, BTN_NORTH), Some("x"));
        assert_eq!(button_action(ControllerType::Xbox, BTN_WEST), Some("y"));
        // hid-playstation reports triangle (north) as 0x133
        assert_eq!(button_action(ControllerType::PlayStation, BTN_NORTH), Some("y"));
        assert_eq!(button_action(ControllerType::PlayStation, BTN_WEST), Some("x"));
        assert_eq!(button_action(ControllerType::Switch, BTN_DPAD_LEFT), Some("left"));
        assert_eq!(button_action(ControllerType::Unknown, 0x2c0), None);
    }

    #[test]
    fn analog_triggers_only_on_known_pads() {
        assert_eq!(axis_role(ControllerType::Xbox, ABS_Z), AxisRole::TriggerL);
        assert_eq!(axis_role(ControllerType::PlayStation, ABS_RZ), AxisRole::TriggerR);
        assert_eq!(axis_role(ControllerType::Unknown, ABS_Z), AxisRole::Ignored);
        assert_eq!(axis_role(ControllerType::Switch, ABS_HAT0Y), AxisRole::HatY);
    }

    #[test]
    fn ioctl_numbers_match_linux_headers() {
        assert_eq!(JSIOCGAXES, 0x8001_6a11);
        assert_eq!(JSIOCGBUTTONS, 0x8001_6a12);
        assert_eq!(JSIOCGAXMAP, 0x8040_6a32);
        assert_eq!(JSIOCGBTNMAP, 0x8400_6a34);
        assert_eq!(jsioc_read(0x13, 256), 0x8100_6a13);
    }
}
