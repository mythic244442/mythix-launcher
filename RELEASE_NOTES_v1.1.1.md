## ◈ Mythix - Built for all ◈

Mythix Launcher v1.1.1 - controller navigation, runtime & polish update

---

## What's new in v1.1.1

This release makes the launcher fully usable with just a controller, fixes forced Steam Linux Runtime launches for Neutron builds, and adds launch wrappers.

---

## Mythix Launcher

### Controller navigation overhaul
- **One step per press** — d-pad/stick navigation no longer skips entries (duplicate input handling fixed at the source)
- **LB / RB section switching** — cycles Library → Import → Settings → Tools; the d-pad stays inside the content area and never lands on the side rail or bottom bar
- **On-screen keyboard** — every text field (search, paths, env vars, launch args) can now be typed into with a controller
- **Option picker** — dropdowns open as a controller-friendly picker
- **Smarter focus** — remembers your spot per section, recovers focus when items change, LT/RT page, right stick scrolls, Back/View jumps to search

### Runtime container fix
- **Forced runtime now works with Neutron** — launches inside steamrt2/3/4 no longer die with `could not load ntdll.so`. The launcher detects host libraries a Wine build needs that the runtime doesn't ship (e.g. `libunwind`, `libxkbregistry`) and bridges them into the container automatically
- **Runtime Library Bridge** section in the launch debug dump shows exactly what was bridged; opt out with `MYTHIX_RUNTIME_BRIDGE=0`
- Host (non-container) launches keep Neutron from re-wrapping itself into Steam's runtime, so the per-game runtime setting stays in charge

### New
- **Wrapper commands** — per-game "Wrapper command" field (Config → Launch Arguments) to put `prime-run`, `gamescope -f --`, `mangohud`, etc. in front of the launch command. Runs on the host, so it works with the runtime container too

### Fixes
- **Windowed-mode nav dropdown** no longer hidden behind the library content

---

## Neutron (builder changes, next build)

- **Portable CPU baseline** — builds now target `x86-64-v3` (AVX2: Intel Haswell 2013+ / AMD Zen+) instead of `-march=native`, so they run on other people's machines. Earlier builds could crash at startup with "illegal instruction" on CPUs older than the build machine's
- **Built without libunwind** (same as Valve's Proton) so Neutron runs cleanly inside Steam Linux Runtime containers
- Prefix templates can now be created from inside a container

---

**Install** — same as v1.1.0, just swap `1.1.0` for `1.1.1` in the file names:
`mythix_1.1.1_amd64.AppImage` · `mythix_1.1.1_amd64.deb` · `mythix-1.1.1-1.x86_64.rpm`

---

- Built with steel, grit, and hard work beyond measure, by Luna & CC :3
