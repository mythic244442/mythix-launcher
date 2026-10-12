// runtime_bridge.rs
// Host-library bridge for the Steam Linux Runtime container.
//
// Wine builds compiled on a regular desktop distro can link against libraries
// the Steam Linux Runtime doesn't ship. Mythix Neutron is built in an Ubuntu
// 24.04 container, so its 64-bit ntdll.so links libunwind.so.8 — which no SLR
// has. Inside the container dlopen("ntdll.so") fails, and Wine only prints the
// error for the *last* path it tried, so the launch dies with the misleading
//
//   wine: could not load ntdll.so: …/i386-windows/ntdll.so: cannot open shared object file
//
// (winewayland.so has the same problem with libxkbregistry.so.0.)
//
// Before a containerized launch we therefore:
//   1. read DT_NEEDED from the tool's native binaries (Wine's *-unix/ modules
//      and bin/),
//   2. check every soname against the runtime's own file list
//      (<runtime>/<suite>_platform_*/usr-mtree.txt.gz),
//   3. copy the host's copy of whatever is missing — plus anything *that*
//      needs which the runtime lacks — into
//      $XDG_DATA_HOME/mythix/runtime-bridge/<runtime>/<triplet>/,
//   4. export those dirs as MYTHIX_RUNTIME_LIBRARY_PATH. mythix-shim runs
//      inside the container and appends them to LD_LIBRARY_PATH (pressure-
//      vessel never forwards the host's LD_LIBRARY_PATH into the container).
//
// Only sonames the runtime doesn't have are ever bridged, and they're searched
// after the runtime's own paths, so a runtime library can never be shadowed.
// Set MYTHIX_RUNTIME_BRIDGE=0 (global or per-game env) to opt out.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::os::unix::fs::{FileExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

/// Unix modules backing optional peripherals (cameras, scanners, packet
/// capture, OpenCL). When one can't load, Wine just reports that feature as
/// unavailable — same as Proton inside the runtime — so their libraries are
/// listed in the report but not bridged.
const OPTIONAL_MODULES: &[&str] = &["gphoto2.so", "sane.so", "wpcap.so", "opencl.so"];

/// Safety valve for the dependency walk.
const MAX_BRIDGED: usize = 32;

// ── Architectures ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Arch {
    X86_64,
    I386,
}

impl Arch {
    pub const ALL: [Arch; 2] = [Arch::X86_64, Arch::I386];

    pub fn triplet(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64-linux-gnu",
            Arch::I386 => "i386-linux-gnu",
        }
    }

    fn wine_unix_dir(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64-unix",
            Arch::I386 => "i386-unix",
        }
    }

    pub fn matches(self, elf: &ElfInfo) -> bool {
        match self {
            Arch::X86_64 => elf.class64 && elf.machine == EM_X86_64,
            Arch::I386 => !elf.class64 && elf.machine == EM_386,
        }
    }

    /// Where distros keep this arch's libraries (fallback when `ldconfig -p`
    /// is unavailable). Candidates are ELF-checked, so overlap is harmless.
    fn host_lib_dirs(self) -> &'static [&'static str] {
        match self {
            Arch::X86_64 => &[
                "/lib/x86_64-linux-gnu", "/usr/lib/x86_64-linux-gnu",
                "/usr/lib64", "/lib64", "/usr/lib", "/usr/local/lib",
            ],
            Arch::I386 => &[
                "/lib/i386-linux-gnu", "/usr/lib/i386-linux-gnu",
                "/usr/lib32", "/lib32", "/usr/lib",
            ],
        }
    }
}

// ── Minimal ELF reader ────────────────────────────────────────────────────────
// Just enough of the format to answer "which libraries does this need?":
// header → program headers → PT_DYNAMIC → DT_NEEDED names out of DT_STRTAB.

const EM_386: u16 = 3;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const DT_NULL: u64 = 0;
const DT_NEEDED: u64 = 1;
const DT_STRTAB: u64 = 5;
const DT_STRSZ: u64 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElfInfo {
    pub class64: bool,
    pub machine: u16,
    /// DT_NEEDED entries, in file order.
    pub needed: Vec<String>,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u32_at(b, at) as u64 | (u32_at(b, at + 4) as u64) << 32
}

/// `size` bytes at `offset`, refusing ranges past EOF or larger than `cap`.
fn read_range(file: &File, file_len: u64, offset: u64, size: u64, cap: u64) -> Option<Vec<u8>> {
    if size > cap || offset.checked_add(size)? > file_len {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    file.read_exact_at(&mut buf, offset).ok()?;
    Some(buf)
}

/// Class, machine and DT_NEEDED list of an ELF file. Little-endian only.
/// None for anything that isn't a readable ELF object.
pub fn read_elf(path: &Path) -> Option<ElfInfo> {
    let file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let ident = read_range(&file, len, 0, 16, 16)?;
    if &ident[..4] != b"\x7fELF" || ident[5] != 1 {
        return None;
    }
    let class64 = match ident[4] {
        1 => false,
        2 => true,
        _ => return None,
    };
    let hdr = read_range(&file, len, 0, if class64 { 64 } else { 52 }, 64)?;
    let machine = u16_at(&hdr, 18);
    let (phoff, phentsize, phnum) = if class64 {
        (u64_at(&hdr, 32), u16_at(&hdr, 54) as u64, u16_at(&hdr, 56) as u64)
    } else {
        (u32_at(&hdr, 28) as u64, u16_at(&hdr, 42) as u64, u16_at(&hdr, 44) as u64)
    };
    let mut info = ElfInfo { class64, machine, needed: Vec::new() };
    if phnum == 0 {
        return Some(info);
    }
    if phentsize < if class64 { 56 } else { 32 } {
        return None;
    }
    let phdrs = read_range(&file, len, phoff, phentsize * phnum, 1 << 20)?;

    let mut loads = Vec::new(); // (vaddr, file offset, file size)
    let mut dynamic = None; // (file offset, size)
    for ph in phdrs.chunks_exact(phentsize as usize) {
        let (p_type, offset, vaddr, filesz) = if class64 {
            (u32_at(ph, 0), u64_at(ph, 8), u64_at(ph, 16), u64_at(ph, 32))
        } else {
            (u32_at(ph, 0), u32_at(ph, 4) as u64, u32_at(ph, 8) as u64, u32_at(ph, 16) as u64)
        };
        match p_type {
            PT_LOAD => loads.push((vaddr, offset, filesz)),
            PT_DYNAMIC => dynamic = Some((offset, filesz)),
            _ => {}
        }
    }
    // Statically linked (wine-preloader): needs nothing.
    let Some((dyn_offset, dyn_size)) = dynamic else { return Some(info) };
    let dyn_bytes = read_range(&file, len, dyn_offset, dyn_size, 1 << 20)?;

    let (mut needed, mut strtab, mut strsz) = (Vec::new(), None, None);
    for entry in dyn_bytes.chunks_exact(if class64 { 16 } else { 8 }) {
        let (tag, val) = if class64 {
            (u64_at(entry, 0), u64_at(entry, 8))
        } else {
            (u32_at(entry, 0) as u64, u32_at(entry, 4) as u64)
        };
        match tag {
            DT_NULL => break,
            DT_NEEDED => needed.push(val),
            DT_STRTAB => strtab = Some(val),
            DT_STRSZ => strsz = Some(val),
            _ => {}
        }
    }
    if needed.is_empty() {
        return Some(info);
    }
    // DT_STRTAB holds a virtual address; map it back through the PT_LOADs.
    let strtab = strtab?;
    let str_offset = loads
        .iter()
        .find(|&&(vaddr, _, filesz)| strtab >= vaddr && strtab - vaddr < filesz)
        .map(|&(vaddr, offset, _)| offset + (strtab - vaddr))?;
    let strings = read_range(&file, len, str_offset, strsz?, 16 << 20)?;
    for name_offset in needed {
        let tail = strings.get(name_offset as usize..)?;
        let end = tail.iter().position(|&b| b == 0)?;
        let name = std::str::from_utf8(&tail[..end]).ok()?;
        if !name.is_empty() {
            info.needed.push(name.to_string());
        }
    }
    Some(info)
}

// ── What the runtime ships ────────────────────────────────────────────────────

/// File names (sonames and real names) in a runtime's library directories.
#[derive(Debug, Default)]
pub struct RuntimeLibs {
    /// e.g. "steamrt4_platform_4.0.20260714.251823"
    pub platform: String,
    names: HashMap<Arch, HashSet<String>>,
}

impl RuntimeLibs {
    pub fn has(&self, arch: Arch, soname: &str) -> bool {
        self.names.get(&arch).is_some_and(|set| set.contains(soname))
    }

    /// Reads the platform's usr-mtree.txt.gz. (Its `files/` directory can't be
    /// used: it has no soname symlinks — pressure-vessel recreates them from
    /// the mtree when it builds the container — so the mtree is the real list.)
    pub fn load(runtime_root: &Path) -> Result<RuntimeLibs, String> {
        let platform = find_platform_dir(runtime_root).ok_or_else(|| {
            format!("no *_platform_*/usr-mtree.txt.gz in {}", runtime_root.display())
        })?;
        let mtree = platform.join("usr-mtree.txt.gz");
        let file = File::open(&mtree).map_err(|e| format!("{}: {e}", mtree.display()))?;
        let mut libs = Self::from_mtree(BufReader::new(GzDecoder::new(file)))
            .map_err(|e| format!("{}: {e}", mtree.display()))?;
        libs.platform = file_name(&platform).unwrap_or_default();
        Ok(libs)
    }

    /// Parses mtree lines such as
    ///   ./lib/x86_64-linux-gnu/libz.so.1 type=link link=libz.so.1.3.1
    /// (paths are relative to the container's /usr).
    pub fn from_mtree<R: BufRead>(reader: R) -> io::Result<RuntimeLibs> {
        let mut libs = RuntimeLibs::default();
        for line in reader.lines() {
            let line = line?;
            let Some(path) = line.split_whitespace().next() else { continue };
            let Some((dir, name)) = path.strip_prefix("./").and_then(|p| p.rsplit_once('/')) else {
                continue;
            };
            let arches: &[Arch] = match dir {
                "lib/x86_64-linux-gnu" | "lib64" => &[Arch::X86_64],
                "lib/i386-linux-gnu" | "lib32" => &[Arch::I386],
                "lib" => &Arch::ALL,
                _ => continue,
            };
            for &arch in arches {
                libs.names.entry(arch).or_default().insert(name.to_string());
            }
        }
        if libs.names.values().all(|set| set.is_empty()) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "no libraries listed"));
        }
        Ok(libs)
    }
}

/// The platform directory the runtime actually starts: the `run` script names
/// it (`dir=steamrt4_platform_…`); otherwise the newest one with an mtree.
fn find_platform_dir(runtime_root: &Path) -> Option<PathBuf> {
    if let Ok(script) = fs::read_to_string(runtime_root.join("run")) {
        for line in script.lines() {
            let Some(name) = line.trim().strip_prefix("dir=") else { continue };
            let name = name.trim().trim_matches(|c| c == '"' || c == '\'');
            let dir = runtime_root.join(name);
            if !name.is_empty() && !name.contains('/') && dir.join("usr-mtree.txt.gz").is_file() {
                return Some(dir);
            }
        }
    }
    let mut candidates: Vec<PathBuf> = fs::read_dir(runtime_root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            file_name(p).is_some_and(|n| n.contains("_platform_"))
                && p.join("usr-mtree.txt.gz").is_file()
        })
        .collect();
    candidates.sort();
    candidates.pop()
}

// ── Host library lookup ───────────────────────────────────────────────────────

/// Finds the host's copy of a library. A trait so tests can fake the host.
pub trait HostLibs {
    /// Real path + ELF info of the host's `soname` for `arch`, if installed.
    fn find(&mut self, arch: Arch, soname: &str) -> Option<(PathBuf, ElfInfo)>;
}

/// The real system: `ldconfig -p`, falling back to the usual library dirs.
#[derive(Default)]
pub struct SystemLibs {
    ldconfig: Option<HashMap<String, Vec<PathBuf>>>,
}

impl SystemLibs {
    fn ldconfig(&mut self) -> &HashMap<String, Vec<PathBuf>> {
        self.ldconfig.get_or_insert_with(|| {
            // Absolute paths first: a launcher started from the desktop often
            // has no /sbin in PATH.
            for bin in ["/sbin/ldconfig", "/usr/sbin/ldconfig", "/usr/bin/ldconfig", "ldconfig"] {
                if let Ok(out) = Command::new(bin).arg("-p").env("LC_ALL", "C").output() {
                    if out.status.success() {
                        return parse_ldconfig(&String::from_utf8_lossy(&out.stdout));
                    }
                }
            }
            HashMap::new()
        })
    }
}

impl HostLibs for SystemLibs {
    fn find(&mut self, arch: Arch, soname: &str) -> Option<(PathBuf, ElfInfo)> {
        if soname.contains('/') {
            return None;
        }
        let mut candidates = self.ldconfig().get(soname).cloned().unwrap_or_default();
        candidates.extend(arch.host_lib_dirs().iter().map(|dir| Path::new(dir).join(soname)));
        candidates.into_iter().find_map(|path| {
            let real = fs::canonicalize(&path).ok()?;
            let info = read_elf(&real)?;
            arch.matches(&info).then_some((real, info))
        })
    }
}

/// `ldconfig -p` lines look like
///   libunwind.so.8 (libc6,x86-64) => /lib/x86_64-linux-gnu/libunwind.so.8
/// The arch flags aren't trusted — candidates get their ELF header checked.
pub fn parse_ldconfig(text: &str) -> HashMap<String, Vec<PathBuf>> {
    let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for line in text.lines() {
        let Some((lhs, path)) = line.split_once(" => ") else { continue };
        let Some(name) = lhs.split_whitespace().next() else { continue };
        map.entry(name.to_string()).or_default().push(PathBuf::from(path.trim()));
    }
    map
}

// ── The bridge ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeReport {
    /// Runtime directory name, e.g. "steamrt4".
    pub runtime: String,
    /// Platform the runtime starts, e.g. "steamrt4_platform_4.0.20260714.251823".
    pub platform: String,
    /// Directories exported as MYTHIX_RUNTIME_LIBRARY_PATH.
    pub library_path: Vec<String>,
    pub bridged: Vec<BridgedLib>,
    /// Needed by a core module but not available on the host either.
    pub missing: Vec<MissingLib>,
    /// Only needed by optional modules (see OPTIONAL_MODULES) — not bridged.
    pub optional: Vec<MissingLib>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgedLib {
    pub soname: String,
    pub arch: String,
    /// Host file it was copied from.
    pub source: String,
    pub needed_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingLib {
    pub soname: String,
    pub arch: String,
    pub needed_by: Vec<String>,
    pub reason: String,
}

impl BridgeReport {
    /// One line per finding, for the launch log and the debug dump.
    pub fn summary_lines(&self) -> Vec<String> {
        let platform = if self.platform.is_empty() { "unknown platform" } else { self.platform.as_str() };
        let mut lines = vec![format!("runtime: {} ({platform})", self.runtime)];
        for lib in &self.bridged {
            lines.push(format!("bridged   {} [{}] ← {} (needed by {})",
                lib.soname, lib.arch, lib.source, lib.needed_by.join(", ")));
        }
        for lib in &self.missing {
            lines.push(format!("MISSING   {} [{}] (needed by {}): {}",
                lib.soname, lib.arch, lib.needed_by.join(", "), lib.reason));
        }
        for lib in &self.optional {
            lines.push(format!("optional  {} [{}] (needed by {}): {}",
                lib.soname, lib.arch, lib.needed_by.join(", "), lib.reason));
        }
        for note in &self.notes {
            lines.push(format!("note: {note}"));
        }
        if self.bridged.is_empty() && self.missing.is_empty() && self.optional.is_empty()
            && self.notes.is_empty()
        {
            lines.push("nothing missing — the runtime has every library this tool links".into());
        }
        if !self.library_path.is_empty() {
            lines.push(format!("MYTHIX_RUNTIME_LIBRARY_PATH={}", self.library_path.join(":")));
        }
        lines
    }
}

/// Bridges whatever `tool` needs from the host into the container of the
/// runtime installed at `runtime_root` (`runtime_name` = "steamrt4" etc.).
/// Never fails: problems land in the report and the launch goes ahead exactly
/// as it would have without the bridge.
pub fn prepare(tool: &Path, runtime_root: &Path, runtime_name: &str) -> BridgeReport {
    // One bridge update at a time (two games launched at once share the dirs).
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    let fail = |note: String| BridgeReport {
        runtime: runtime_name.to_string(),
        notes: vec![note],
        ..Default::default()
    };
    let Some(data_dir) = crate::paths::data_dir() else {
        return fail("no data directory — bridge skipped".into());
    };
    let runtime = match RuntimeLibs::load(runtime_root) {
        Ok(runtime) => runtime,
        Err(e) => return fail(format!("couldn't read the runtime's library list ({e}) — bridge skipped")),
    };
    let bridge_root = data_dir.join("runtime-bridge").join(runtime_name);
    prepare_with(tool, runtime_name, &runtime, &mut SystemLibs::default(), &bridge_root)
}

/// `prepare` with the runtime list, host lookup and bridge directory supplied.
pub fn prepare_with(
    tool: &Path,
    runtime_name: &str,
    runtime: &RuntimeLibs,
    host: &mut dyn HostLibs,
    bridge_root: &Path,
) -> BridgeReport {
    let mut report = BridgeReport {
        runtime: runtime_name.to_string(),
        platform: runtime.platform.clone(),
        ..Default::default()
    };
    prune(bridge_root, runtime, &mut report);
    let provided = tool_provided_names(tool);

    // (arch, soname) → the modules that need it, and whether any is core.
    let mut wanted: BTreeMap<(Arch, String), (BTreeSet<String>, bool)> = BTreeMap::new();
    let mut scanned = 0;
    for arch in Arch::ALL {
        for dir in native_dirs(tool, arch) {
            let files: Vec<PathBuf> = match fs::read_dir(&dir) {
                Ok(entries) => entries.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect(),
                Err(_) => continue,
            };
            let siblings: HashSet<String> = files.iter().filter_map(|p| file_name(p)).collect();
            for path in &files {
                let Some(elf) = read_elf(path) else { continue };
                if !arch.matches(&elf) {
                    continue;
                }
                scanned += 1;
                let module = file_name(path).unwrap_or_default();
                let core = !OPTIONAL_MODULES.contains(&module.as_str());
                for soname in elf.needed {
                    if runtime.has(arch, &soname) || provided.contains(&soname) || siblings.contains(&soname) {
                        continue;
                    }
                    let entry = wanted.entry((arch, soname)).or_insert_with(|| (BTreeSet::new(), false));
                    entry.0.insert(module.clone());
                    entry.1 |= core;
                }
            }
        }
    }
    if scanned == 0 {
        report.notes.push(format!("no Unix-side Wine modules found in {}", tool.display()));
        return report;
    }

    let mut queue = VecDeque::new();
    for ((arch, soname), (by, core)) in wanted {
        let needed_by: Vec<String> = by.into_iter().collect();
        if core {
            queue.push_back((arch, soname, needed_by));
        } else {
            report.optional.push(MissingLib {
                soname,
                arch: arch.triplet().into(),
                needed_by,
                reason: "optional module, not bridged (Wine just disables that feature)".into(),
            });
        }
    }

    // Bridge, then walk what each bridged library needs in turn.
    let mut seen: HashSet<(Arch, String)> = HashSet::new();
    while let Some((arch, soname, needed_by)) = queue.pop_front() {
        if !seen.insert((arch, soname.clone())) {
            continue;
        }
        if report.bridged.len() >= MAX_BRIDGED {
            report.notes.push(format!("stopped after {MAX_BRIDGED} libraries"));
            break;
        }
        let missing = |reason: String| MissingLib {
            soname: soname.clone(),
            arch: arch.triplet().into(),
            needed_by: needed_by.clone(),
            reason,
        };
        let Some((source, elf)) = host.find(arch, &soname) else {
            report.missing.push(missing("not installed on this system either".into()));
            continue;
        };
        if let Err(e) = install(&source, &bridge_root.join(arch.triplet()), &soname) {
            report.missing.push(missing(format!("couldn't copy {}: {e}", source.display())));
            continue;
        }
        for dep in &elf.needed {
            if !runtime.has(arch, dep) && !provided.contains(dep) && !seen.contains(&(arch, dep.clone())) {
                queue.push_back((arch, dep.clone(), vec![soname.clone()]));
            }
        }
        report.bridged.push(BridgedLib {
            soname,
            arch: arch.triplet().into(),
            source: source.display().to_string(),
            needed_by,
        });
    }

    for arch in Arch::ALL {
        if report.bridged.iter().any(|lib| lib.arch == arch.triplet()) {
            report.library_path.push(bridge_root.join(arch.triplet()).display().to_string());
        }
    }
    report
}

/// Where the tool keeps native binaries for `arch`: the Unix-side Wine module
/// directory (`files/lib/wine/x86_64-unix`, older Proton's `files/lib64/wine/…`,
/// plain Wine trees without `files/`) and `bin/` (wine, wineserver). Files of
/// the other architecture are skipped by the ELF check.
fn native_dirs(tool: &Path, arch: Arch) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    [tool.join("files"), tool.to_path_buf()]
        .iter()
        .flat_map(|base| {
            let unix = ["lib", "lib64"].map(|lib| base.join(lib).join("wine").join(arch.wine_unix_dir()));
            unix.into_iter().chain([base.join("bin")])
        })
        .filter(|dir| dir.is_dir())
        .filter(|dir| seen.insert(fs::canonicalize(dir).unwrap_or_else(|_| dir.clone())))
        .collect()
}

/// Libraries the tool ships itself (its launcher script puts these dirs on
/// LD_LIBRARY_PATH), which therefore never need bridging.
fn tool_provided_names(tool: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    for base in [tool.join("files"), tool.to_path_buf()] {
        for sub in ["lib", "lib64", "lib32", "lib/x86_64-linux-gnu", "lib/i386-linux-gnu"] {
            let Ok(entries) = fs::read_dir(base.join(sub)) else { continue };
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    names.insert(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    names
}

/// Drops bridged copies of anything the runtime ships itself now — after a
/// runtime update the runtime's copy must win — plus abandoned temp files.
fn prune(bridge_root: &Path, runtime: &RuntimeLibs, report: &mut BridgeReport) {
    for arch in Arch::ALL {
        let Ok(entries) = fs::read_dir(bridge_root.join(arch.triplet())) else { continue };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') && name.ends_with(".tmp") {
                // A fresh one may belong to a launch that's copying right now.
                let age = entry.metadata().and_then(|m| m.modified()).ok()
                    .and_then(|t| SystemTime::now().duration_since(t).ok());
                if !matches!(age, Some(age) if age < Duration::from_secs(60)) {
                    let _ = fs::remove_file(entry.path());
                }
            } else if runtime.has(arch, &name) && fs::remove_file(entry.path()).is_ok() {
                report.notes.push(format!("dropped bridged {name} [{}] — the runtime ships it now", arch.triplet()));
            }
        }
    }
}

/// Copies `source` to `dir/soname` unless an identical copy is already there.
/// Writes a temp file and renames it into place, so a running game that has
/// the previous copy mapped is never disturbed.
fn install(source: &Path, dir: &Path, soname: &str) -> io::Result<()> {
    let data = fs::read(source)?;
    let dest = dir.join(soname);
    if fs::read(&dest).is_ok_and(|old| old == data) {
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{soname}.{}-{seq}.tmp", std::process::id()));
    let result = fs::write(&tmp, &data)
        .and_then(|_| fs::set_permissions(&tmp, fs::Permissions::from_mode(0o644)))
        .and_then(|_| fs::rename(&tmp, &dest));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn file_name(path: &Path) -> Option<String> {
    path.file_name().map(|n| n.to_string_lossy().into_owned())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A minimal but well-formed ELF shared object: header, one PT_LOAD
    /// covering the file, PT_DYNAMIC, and a string table with the needed names.
    fn fake_elf(class64: bool, machine: u16, needed: &[&str]) -> Vec<u8> {
        let (ehsize, phentsize, dynent) = if class64 { (64, 56, 16) } else { (52, 32, 8) };
        let base: u64 = 0x1000; // non-zero vaddr so the address mapping matters
        let mut strtab = vec![0u8];
        let mut offsets = Vec::new();
        for name in needed {
            offsets.push(strtab.len() as u64);
            strtab.extend_from_slice(name.as_bytes());
            strtab.push(0);
        }
        let phoff = ehsize as u64;
        let dyn_off = phoff + 2 * phentsize as u64;
        let mut dyn_entries: Vec<(u64, u64)> = offsets.iter().map(|&o| (DT_NEEDED, o)).collect();
        let str_off = dyn_off + (dyn_entries.len() as u64 + 3) * dynent as u64;
        dyn_entries.push((DT_STRTAB, base + str_off));
        dyn_entries.push((DT_STRSZ, strtab.len() as u64));
        dyn_entries.push((DT_NULL, 0));
        let total = str_off + strtab.len() as u64;

        let mut out = Vec::new();
        let word = |out: &mut Vec<u8>, v: u64| {
            if class64 { out.extend_from_slice(&v.to_le_bytes()) } else { out.extend_from_slice(&(v as u32).to_le_bytes()) }
        };
        out.extend_from_slice(b"\x7fELF");
        out.extend_from_slice(&[if class64 { 2 } else { 1 }, 1, 1, 0]);
        out.extend_from_slice(&[0u8; 8]);
        out.extend_from_slice(&3u16.to_le_bytes()); // ET_DYN
        out.extend_from_slice(&machine.to_le_bytes());
        out.extend_from_slice(&1u32.to_le_bytes());
        word(&mut out, 0); // e_entry
        word(&mut out, phoff);
        word(&mut out, 0); // e_shoff
        out.extend_from_slice(&0u32.to_le_bytes()); // e_flags
        out.extend_from_slice(&(ehsize as u16).to_le_bytes());
        out.extend_from_slice(&(phentsize as u16).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes()); // e_phnum
        out.extend_from_slice(&[0u8; 6]); // shentsize, shnum, shstrndx
        assert_eq!(out.len(), ehsize);

        let phdr = |out: &mut Vec<u8>, p_type: u32, offset: u64, vaddr: u64, size: u64| {
            out.extend_from_slice(&p_type.to_le_bytes());
            if class64 {
                out.extend_from_slice(&4u32.to_le_bytes()); // p_flags
                for v in [offset, vaddr, vaddr, size, size, 8] {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            } else {
                for v in [offset, vaddr, vaddr, size, size, 4, 4] {
                    out.extend_from_slice(&(v as u32).to_le_bytes());
                }
            }
        };
        phdr(&mut out, PT_LOAD, 0, base, total);
        phdr(&mut out, PT_DYNAMIC, dyn_off, base + dyn_off, dyn_entries.len() as u64 * dynent as u64);
        for (tag, val) in dyn_entries {
            word(&mut out, tag);
            word(&mut out, val);
        }
        assert_eq!(out.len() as u64, str_off);
        out.extend_from_slice(&strtab);
        out
    }

    fn write(path: &Path, data: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }

    /// Sets a file's mtime to an hour ago.
    fn backdate(path: &Path) {
        let c_path = std::ffi::CString::new(path.as_os_str().to_string_lossy().as_bytes()).unwrap();
        let then = (SystemTime::now() - Duration::from_secs(3600))
            .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as libc::time_t;
        let times = [libc::timespec { tv_sec: then, tv_nsec: 0 }; 2];
        assert_eq!(unsafe { libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) }, 0);
    }

    fn runtime(lines: &str) -> RuntimeLibs {
        RuntimeLibs::from_mtree(lines.as_bytes()).unwrap()
    }

    /// Fake host: soname → path of a fake library file.
    struct FakeHost(HashMap<(Arch, String), PathBuf>);

    impl HostLibs for FakeHost {
        fn find(&mut self, arch: Arch, soname: &str) -> Option<(PathBuf, ElfInfo)> {
            let path = self.0.get(&(arch, soname.to_string()))?.clone();
            let info = read_elf(&path)?;
            arch.matches(&info).then_some((path, info))
        }
    }

    #[test]
    fn reads_needed_from_64_and_32_bit_objects() {
        let dir = tempfile::tempdir().unwrap();
        let so64 = dir.path().join("ntdll64.so");
        let so32 = dir.path().join("ntdll32.so");
        write(&so64, &fake_elf(true, EM_X86_64, &["libunwind.so.8", "libc.so.6"]));
        write(&so32, &fake_elf(false, EM_386, &["libc.so.6"]));

        let info = read_elf(&so64).unwrap();
        assert!(info.class64 && Arch::X86_64.matches(&info) && !Arch::I386.matches(&info));
        assert_eq!(info.needed, ["libunwind.so.8", "libc.so.6"]);

        let info = read_elf(&so32).unwrap();
        assert!(Arch::I386.matches(&info) && !Arch::X86_64.matches(&info));
        assert_eq!(info.needed, ["libc.so.6"]);
    }

    #[test]
    fn rejects_non_elf_and_truncated_files() {
        let dir = tempfile::tempdir().unwrap();
        let text = dir.path().join("libc.so");
        write(&text, b"/* GNU ld script */ GROUP ( libc.so.6 )");
        assert!(read_elf(&text).is_none());

        let mut elf = fake_elf(true, EM_X86_64, &["libfoo.so.1"]);
        elf.truncate(elf.len() - 4); // string table cut short
        let cut = dir.path().join("cut.so");
        write(&cut, &elf);
        assert!(read_elf(&cut).is_none());
    }

    #[test]
    fn mtree_lists_sonames_per_arch() {
        let rt = runtime(
            "#mtree\n\
             ./lib/x86_64-linux-gnu/libz.so.1 type=link link=libz.so.1.3.1\n\
             ./lib/x86_64-linux-gnu/libz.so.1.3.1 type=file size=1\n\
             ./lib/i386-linux-gnu/libc.so.6 type=file size=1\n\
             ./lib64/ld-linux-x86-64.so.2 type=link link=../lib/x86_64-linux-gnu/ld-linux-x86-64.so.2\n\
             ./lib/x86_64-linux-gnu/gconv/UTF-16.so type=file size=1\n\
             ./share/doc/libunwind.so.8 type=file size=1\n",
        );
        assert!(rt.has(Arch::X86_64, "libz.so.1"));
        assert!(!rt.has(Arch::I386, "libz.so.1"));
        assert!(rt.has(Arch::I386, "libc.so.6"));
        assert!(!rt.has(Arch::X86_64, "libc.so.6"));
        assert!(rt.has(Arch::X86_64, "ld-linux-x86-64.so.2"));
        assert!(!rt.has(Arch::X86_64, "UTF-16.so")); // subdirectories aren't on the search path
        assert!(!rt.has(Arch::X86_64, "libunwind.so.8"));
        assert!(RuntimeLibs::from_mtree("#mtree\n. type=dir\n".as_bytes()).is_err());
    }

    #[test]
    fn finds_the_platform_named_by_the_run_script() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["steamrt4_platform_4.0.1", "steamrt4_platform_4.0.2"] {
            write(&dir.path().join(name).join("usr-mtree.txt.gz"), b"");
        }
        assert_eq!(find_platform_dir(dir.path()).unwrap(), dir.path().join("steamrt4_platform_4.0.2"));
        write(&dir.path().join("run"), b"#!/bin/sh\nset -eu\ndir=steamrt4_platform_4.0.1\n");
        assert_eq!(find_platform_dir(dir.path()).unwrap(), dir.path().join("steamrt4_platform_4.0.1"));
    }

    #[test]
    fn loads_a_gzipped_mtree() {
        let dir = tempfile::tempdir().unwrap();
        let platform = dir.path().join("sniper_platform_3.0.1");
        fs::create_dir_all(&platform).unwrap();
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gz.write_all(b"./lib/x86_64-linux-gnu/libc.so.6 type=file\n").unwrap();
        fs::write(platform.join("usr-mtree.txt.gz"), gz.finish().unwrap()).unwrap();
        let rt = RuntimeLibs::load(dir.path()).unwrap();
        assert_eq!(rt.platform, "sniper_platform_3.0.1");
        assert!(rt.has(Arch::X86_64, "libc.so.6"));
    }

    #[test]
    fn parses_ldconfig_output() {
        let map = parse_ldconfig(
            "1234 libs found in cache `/etc/ld.so.cache'\n\
             \tlibunwind.so.8 (libc6,x86-64) => /lib/x86_64-linux-gnu/libunwind.so.8\n\
             \tlibz.so.1 (libc6,x86-64) => /lib/x86_64-linux-gnu/libz.so.1\n\
             \tlibz.so.1 (libc6) => /lib/i386-linux-gnu/libz.so.1\n\
             \tlibfoo.so.2 (libc6,x86-64, OS ABI: Linux 3.2.0) => /usr/lib/libfoo.so.2\n\
             Cache generated by: ldconfig (GNU libc) stable release version 2.39\n",
        );
        assert_eq!(map["libunwind.so.8"], [PathBuf::from("/lib/x86_64-linux-gnu/libunwind.so.8")]);
        assert_eq!(map["libz.so.1"].len(), 2);
        assert_eq!(map["libfoo.so.2"], [PathBuf::from("/usr/lib/libfoo.so.2")]);
        assert_eq!(map.len(), 3);
    }

    /// The real-world case: Neutron's ntdll.so needs libunwind.so.8 (which in
    /// turn needs liblzma, which the runtime has) and winewayland.so needs
    /// libxkbregistry.so.0, missing for i386 on the host too.
    #[test]
    fn bridges_missing_libraries_and_their_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("Mythix-Neutron");
        let unix64 = tool.join("files/lib/wine/x86_64-unix");
        let unix32 = tool.join("files/lib/wine/i386-unix");
        write(&unix64.join("ntdll.so"), &fake_elf(true, EM_X86_64, &["libunwind.so.8", "libc.so.6"]));
        write(&unix64.join("winewayland.so"), &fake_elf(true, EM_X86_64, &["libxkbregistry.so.0", "ntdll.so", "libc.so.6"]));
        write(&unix64.join("gphoto2.so"), &fake_elf(true, EM_X86_64, &["libgphoto2.so.6", "libc.so.6"]));
        write(&unix64.join("wine-preloader"), &fake_elf(true, EM_X86_64, &[]));
        write(&unix64.join("README"), b"not a library");
        write(&unix32.join("winewayland.so"), &fake_elf(false, EM_386, &["libxkbregistry.so.0", "libc.so.6"]));
        write(&tool.join("files/lib/libtoolprovided.so.1"), b"");
        write(&unix64.join("winex11.so"), &fake_elf(true, EM_X86_64, &["libtoolprovided.so.1", "libX11.so.6"]));
        write(&tool.join("files/bin/wineserver"), &fake_elf(true, EM_X86_64, &["libunwind.so.8", "libc.so.6"]));
        write(&tool.join("files/bin/winetricks"), b"#!/bin/sh\n");

        let host_dir = dir.path().join("host");
        let mut host = FakeHost(HashMap::new());
        for (arch, name, elf) in [
            (Arch::X86_64, "libunwind.so.8", fake_elf(true, EM_X86_64, &["liblzma.so.5", "libc.so.6"])),
            (Arch::X86_64, "libxkbregistry.so.0", fake_elf(true, EM_X86_64, &["libxml2.so.2", "libextra.so.1"])),
            (Arch::X86_64, "libextra.so.1", fake_elf(true, EM_X86_64, &["libc.so.6"])),
        ] {
            let path = host_dir.join(arch.triplet()).join(name);
            write(&path, &elf);
            host.0.insert((arch, name.to_string()), path);
        }

        let rt = runtime(
            "./lib/x86_64-linux-gnu/libc.so.6 type=file\n\
             ./lib/x86_64-linux-gnu/liblzma.so.5 type=link\n\
             ./lib/x86_64-linux-gnu/libxml2.so.2 type=link\n\
             ./lib/x86_64-linux-gnu/libX11.so.6 type=link\n\
             ./lib/x86_64-linux-gnu/libstale.so.3 type=link\n\
             ./lib/i386-linux-gnu/libc.so.6 type=file\n",
        );
        let bridge = dir.path().join("bridge/steamrt4");
        write(&bridge.join("x86_64-linux-gnu/libstale.so.3"), b"old copy");
        write(&bridge.join("x86_64-linux-gnu/.libunwind.so.8.99-0.tmp"), b"junk");
        backdate(&bridge.join("x86_64-linux-gnu/.libunwind.so.8.99-0.tmp"));
        write(&bridge.join("x86_64-linux-gnu/.libextra.so.1.99-1.tmp"), b"in flight");

        let report = prepare_with(&tool, "steamrt4", &rt, &mut host, &bridge);

        let bridged: Vec<(&str, &str)> = report.bridged.iter().map(|b| (b.soname.as_str(), b.arch.as_str())).collect();
        assert_eq!(bridged, [
            ("libunwind.so.8", "x86_64-linux-gnu"),
            ("libxkbregistry.so.0", "x86_64-linux-gnu"),
            ("libextra.so.1", "x86_64-linux-gnu"),
        ]);
        assert_eq!(report.bridged[0].needed_by, ["ntdll.so", "wineserver"]);
        assert_eq!(report.bridged[2].needed_by, ["libxkbregistry.so.0"]);
        for (name, _) in &bridged {
            let copy = bridge.join("x86_64-linux-gnu").join(name);
            assert_eq!(fs::read(&copy).unwrap(), fs::read(&host.0[&(Arch::X86_64, name.to_string())]).unwrap());
            assert_eq!(fs::metadata(&copy).unwrap().permissions().mode() & 0o777, 0o644);
        }

        assert_eq!(report.missing.len(), 1);
        assert_eq!(report.missing[0].soname, "libxkbregistry.so.0");
        assert_eq!(report.missing[0].arch, "i386-linux-gnu");

        assert_eq!(report.optional.len(), 1);
        assert_eq!(report.optional[0].soname, "libgphoto2.so.6");

        // Stale copy of something the runtime now ships is gone, junk too.
        assert!(!bridge.join("x86_64-linux-gnu/libstale.so.3").exists());
        assert!(!bridge.join("x86_64-linux-gnu/.libunwind.so.8.99-0.tmp").exists());
        assert!(bridge.join("x86_64-linux-gnu/.libextra.so.1.99-1.tmp").exists()); // may be another launch's
        assert!(report.notes.iter().any(|n| n.contains("libstale.so.3")));

        assert_eq!(report.library_path, [bridge.join("x86_64-linux-gnu").display().to_string()]);

        // Second run: nothing changes, nothing breaks.
        let again = prepare_with(&tool, "steamrt4", &rt, &mut host, &bridge);
        assert_eq!(again.bridged.len(), 3);
        assert!(again.notes.is_empty());
    }

    #[test]
    fn nothing_to_do_for_a_tool_the_runtime_fully_covers() {
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("proton-11.0");
        write(&tool.join("files/lib/wine/x86_64-unix/ntdll.so"), &fake_elf(true, EM_X86_64, &["libgcc_s.so.1", "libc.so.6"]));
        let rt = runtime("./lib/x86_64-linux-gnu/libgcc_s.so.1 type=file\n./lib/x86_64-linux-gnu/libc.so.6 type=file\n");
        let bridge = dir.path().join("bridge");
        let report = prepare_with(&tool, "steamrt4", &rt, &mut FakeHost(HashMap::new()), &bridge);
        assert!(report.bridged.is_empty() && report.missing.is_empty() && report.library_path.is_empty());
        assert!(!bridge.exists());
        assert!(report.summary_lines().iter().any(|l| l.starts_with("nothing missing")));
    }

    /// Checks a real tool against a real runtime:
    ///   MYTHIX_TEST_TOOL=/path/to/Mythix-Neutron-11.19-4 \
    ///   MYTHIX_TEST_RUNTIME=~/.local/share/mythix/steamrt4 \
    ///   cargo test runtime_bridge -- --ignored --nocapture
    #[test]
    #[ignore]
    fn real_tool_against_real_runtime() {
        let tool = PathBuf::from(std::env::var("MYTHIX_TEST_TOOL").expect("MYTHIX_TEST_TOOL"));
        let rt_root = PathBuf::from(std::env::var("MYTHIX_TEST_RUNTIME").expect("MYTHIX_TEST_RUNTIME"));
        let bridge = std::env::var("MYTHIX_TEST_BRIDGE").map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("mythix-bridge-test"));
        let rt = RuntimeLibs::load(&rt_root).unwrap();
        let name = file_name(&rt_root).unwrap();
        let report = prepare_with(&tool, &name, &rt, &mut SystemLibs::default(), &bridge.join(&name));
        for line in report.summary_lines() {
            println!("{line}");
        }
    }
}
