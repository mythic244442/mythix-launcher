# Mythix Launcher

A native Linux game launcher for Wine, Proton, and Neutron compatibility tools. Built with Tauri v2 + React. Designed for gamers who want full control over how their Windows games run on Linux - without the walled garden.

Mythix ships with first-class support for Mythix's new compatibility tool for gaming on Linux called [Neutron](https://github.com/mythix-org/mythix-build), but works with any Proton, GE-Proton, or standalone Wine build.

---

## Features

**Library Management**
- Add games from any source — Steam, GOG, Epic, Itch, Heroic, Amazon, or manual installs
- One-click Steam library import with automatic prefix cloning
- Custom loader builder for running game binaries from a custom location with the ability to set settings for them via a configuration file
- Grid and list view with search, cover art (Steam CDN + local import), and playtime tracking
- Spotlight hero banner showing your most-played game

**Launch Engine**
- Full Linux compatibility tool support (Proton, Neutron, Wine, hybrids)
- Automatic prefix creation and management (Proton-style `pfx/` layout)
- Steam Runtime container integration — Sniper (RT 3.0), Soldier (RT 2.0), SteamRT4
- Per-game environment variable overrides with 60+ quick toggles (DXVK, MangoHud, Fsync, NTSync, FSR, GameMode, and more)
- Per-game launch arguments, DLL overrides, and Proton verb selection
- Dry-run inspector — see exactly what command, env, and paths will be used before launching
- Automatic game drive symlinks for non-C: installs
- Process isolation via `setsid --fork` to escape Tauri's seccomp sandbox

**Controller Support**
- Full gamepad navigation — browse your library, launch games, and configure settings without a keyboard
- LB / RB switch sections; the d-pad / left stick only ever moves inside the content
- On-screen keyboard for every text field, plus a controller picker for dropdowns
- System-level input via `/dev/input/js*` (mapped by kernel event code, merged across devices, paused while a game has focus) — hide and restore the launcher even outside the webview
- Quick Settings overlay (X button) — change compatibility tool, runtime container, and env toggles with a controller
- Controller legend in the titlebar showing context-sensitive button hints
- Connection indicator with live status

**Per-Game Configuration**
- Compatibility tool selector (auto-scans `compatibilitytools.d` directories)
- Prefix path and type
- Store identifier for organization
- Runtime container override (Auto / Sniper / Soldier / SteamRT4)
- Force runtime on/off per game
- Custom environment variables with a full KEY=VALUE editor
- Quick toggles organized by category: Sync, Neutron, DXVK/VKD3D, Proton/Wine, GPU/Display, Mythix

**Platform**
- Native Linux desktop app — Tauri v2, no Electron
- Frameless window with custom titlebar, drag-to-move, double-click maximize
- KDE Wayland window activation (kdotool + KWin D-Bus + xdotool fallback)
- Global settings persistence

---

# Quick Start

## Prerequisites
- **Rust** (stable toolchain)
- **Node.js** 20+
- **Tauri CLI v2**: `cargo install tauri-cli@^2`

## System libraries required
### Ubuntu/Debian
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
  ```
### Arch Linux
  ```bash
  sudo pacman -S webkit2gtk-4.1 libayatana-appindicator librsvg
  ```

### Install & Run
#### First, download the git repository and navigate to the directory it downloaded to in your files:
```bash
git clone https://github.com/mythix/mythix-launcher.git
cd mythix-launcher
```
#### Then install the node packages and run in dev mode:
```bash
npm install
npm run tauri dev
```
#### Alternatively you can build and run the launcher with cargo (Direct Rust):
```bash
cargo tauri dev
```
#### And if you want, you can build a release with cargo as well:
```bash
cargo tauri build
```
#### The release binary lands in `src-tauri/target/release/mythix-launcher`. A `.deb` and `.AppImage` are generated in `src-tauri/target/release/bundle/`.

---

## Controller Mapping

Mythix reads standard gamepads (Xbox, PlayStation, Switch Pro, generic) from `/dev/input/js*` in the Rust backend and drives the UI from there. Actions are positional, so "A" means the bottom face button on every pad (✕ on PlayStation, B on Switch).

| Button | Action |
|--------|--------|
| **LB / RB** | Previous / next section — walks the frame clockwise (Library → Import → Settings → Tools) |
| **D-pad / Left Stick** | Move focus inside the current section (never onto the frame) |
| **A** | Launch game · open the keyboard on a text field · open the picker on a dropdown · press buttons |
| **B** | Back — close the dialog, picker, keyboard, or config panel |
| **X** | Quick Settings for the focused game |
| **Y** | Stop the running game |
| **Start / Options** | Open the focused game's config panel |
| **Back / View / Create** | Jump to the section's search box with the keyboard open |
| **LT / RT** | Page up / down |
| **Right Stick** | Scroll |
| **Guide / Home** | Hide / Show launcher |

On-screen keyboard (A on any text field):
- **D-pad** picks a key, **A** types it
- **X** backspace, **Y** space, **LB/RB** move the cursor
- **LT** shift, **RT** caps lock, **Start** done, **B** close
- Physical keyboard and mouse keep working while it's open

Quick Settings overlay (press X on a game):
- **Left/Right** on Tool or Runtime fields cycles through options
- **D-pad** navigates between fields and env toggles
- **A** toggles env variables or activates Save / Save & Launch
- **B** closes the overlay

--------|--------|
| **A** | Launch game / Confirm |
| **B** | Back / Close panel |
| **X** | Quick Settings (no game running) / Stop game (game running) |
| **Y** | Stop game (game running) |
| **D-pad / Left Stick** | Navigate library grid and menus |
| **Guide / Home** | Hide / Show launcher |

Quick Settings overlay (press X on a game):
- **Left/Right** on Tool or Runtime fields cycles through options
- **D-pad** navigates between fields and env toggles
- **A** toggles env variables or activates Save / Save & Launch
- **B** closes the overlay

---

## Architecture

```
mythix-launcher/
├── src/                          # React frontend
│   ├── App.jsx                   # Root — state, controller nav, game lifecycle
│   ├── styles.css                # Deep-space industrial theme
│   ├── tauri.js                  # IPC bridge (all invoke() calls)
│   ├── hooks/
│   │   ├── useLibrary.js         # Game library CRUD state
│   │   ├── useGamepad.js         # Input layers (exclusive per overlay) + controller state
│   │   ├── useControllerNav.js   # Base controller layer: sections, focus, A/B/X/Y…
│   │   ├── useRuntime.js         # Steam Runtime install/status
│   │   ├── useCoverUrl.js        # Cover art URL resolution
│   │   ├── useMaximized.js       # Window maximize state tracking
│   │   └── useToast.js           # Notification toasts
│   ├── controller/
│   │   ├── bus.js                # Single gamepad:action listener, dedupe, repeat, layers
│   │   ├── focus.js              # Scoped spatial navigation (grid/list index math)
│   │   ├── overlays.js           # Keyboard / picker state + React-safe value setters
│   │   ├── sections.js           # LB/RB section order
│   │   └── glyphs.js             # Button labels per controller family
│   └── components/
│       ├── TitleBar.jsx          # Custom titlebar + controller legend
│       ├── OnScreenKeyboard.jsx  # Controller keyboard for text fields
│       ├── OptionPicker.jsx      # Controller picker for <select>
│       ├── NavRail.jsx           # Sidebar navigation
│       ├── LibraryView.jsx       # Game grid/list + spotlight hero
│       ├── GameCard.jsx          # Game tile with cover art
│       ├── ConfigPanel.jsx       # Full per-game configuration
│       ├── QuickSettings.jsx     # Controller-friendly quick config overlay
│       ├── EnvVarsEditor.jsx     # KEY=VALUE environment editor
│       ├── EnvQuickToggles.jsx   # 60+ categorized env var toggles
│       ├── SteamImportView.jsx   # Steam library scanner + importer
│       ├── CompatToolsView.jsx   # Installed compatibility tools browser
│       ├── SettingsView.jsx      # Global launcher settings
│       ├── RuntimeSetup.jsx      # First-run Steam Runtime installer
│       ├── AddGameModal.jsx      # Add game dialog
│       ├── LogConsole.jsx        # Live game output viewer
│       └── ToastContainer.jsx    # Notification overlay
└── src-tauri/
    └── src/
        ├── lib.rs                # App setup, plugin registration, gamepad monitor
        ├── commands.rs           # Tauri IPC command handlers
        ├── game_library.rs       # Library persistence, CRUD, compat tool scanner
        ├── launcher.rs           # Prefix setup, env assembly, process spawning
        ├── gamepad.rs            # /dev/input/js* monitor → merged gamepad:action events
        ├── runtime.rs            # Steam Runtime download + management, mythix-shim
        ├── runtime_bridge.rs     # Host libraries a tool needs but the runtime lacks
        ├── steam_import.rs       # Steam library detection + prefix cloning
        ├── coverart.rs           # Steam CDN cover art fetcher
        ├── settings.rs           # Global settings persistence
        ├── paths.rs              # XDG path resolution + legacy migration
        ├── vdf.rs                # Valve Data Format parser
        └── error.rs              # Error types
```

---

## Data Storage

| What | Where |
|------|-------|
| Game library | `~/.local/share/mythix/library.json` |
| Global settings | `~/.local/share/mythix/settings.json` |
| Cover art cache | `~/.local/share/mythix/covers/` |
| Game prefixes | `~/.mythix/gamedata/<game_id>/` |
| Steam Runtime | `~/.local/share/mythix/steamrt{2,3,4}/` |
| Runtime library bridge | `~/.local/share/mythix/runtime-bridge/<runtime>/` |
| Launch log | `/tmp/mythix_launch.log` |

---

## Per-Game Config Reference

| Field | Env Var | Description |
|-------|---------|-------------|
| `proton_path` | `PROTONPATH` | Directory containing `proton`, `neutron`, or `bin/wine` |
| `prefix_path` | `WINEPREFIX` | Created and initialized automatically |
| `game_id` | `STEAM_COMPAT_APP_ID` | `mythix-<appid>` for Steam, UUID for manual |
| `store` | `STORE` | steam, gog, egs, origin, ubisoft, itch, heroic, amazon, humble, manual |
| `proton_verb` | — | waitforexitandrun, run, runinprefix, etc. |
| `runtime_variant` | — | Auto, sniper, soldier, steamrt4 |
| `use_runtime` | — | Auto (toolmanifest), force on, force off |
| `env_overrides` | injected at launch | Any KEY=VALUE pairs |
| `launch_args` | appended to command | Extra arguments after the exe |

## Runtime container: host library bridge

Wine builds made on a desktop distro can link libraries the Steam Linux Runtime doesn't ship — Mythix Neutron's `ntdll.so` needs `libunwind.so.8`, `winewayland.so` needs `libxkbregistry.so.0`. Inside the container Wine then dies with a misleading `could not load ntdll.so`. Before every container launch the launcher reads the tool's native binaries, checks each needed library against the runtime's own file list (`*_platform_*/usr-mtree.txt.gz`), and copies the host's copy of anything missing into `~/.local/share/mythix/runtime-bridge/<runtime>/`; `mythix-shim` appends that directory to `LD_LIBRARY_PATH` inside the container. Only libraries the runtime lacks are bridged, so the runtime's own libraries are never shadowed.

The launch debug dump has a **Runtime Library Bridge** section listing what was bridged and anything still missing. `MYTHIX_RUNTIME_BRIDGE=0` (global or per game) turns it off. Launches that don't use the container get `NEUTRON_NO_CONTAINER=1`, so Neutron builds that wrap themselves into Steam's runtime leave that decision to the per-game setting.

---

## Bundled Compatibility Tools

Mythix Launcher release ships with:

- **Neutron** — A full compatibility tool home-baked and built by Mythix. It includes DXVK, VKD3D-Proton, and the Neutron launch script with FSR, GameMode, Gamescope, anti-cheat runtime, and async shader support. The build included is Mythix-Neutron-11.19-4 and is the latest version.

- **Mythix Wine** — A custom Neutron using a Wine built against Valve's Bleeding Edge Wine tree with patches applied from Wine HQ's staging patches and both GE and TkG patches as well as our own fixups, tweaks, and patches from Mythix. It is a Wine 11 build with ntsync and is our most experimental version so far.

Install it to `~/.steam/root/compatibilitytools.d`, `~/.steam/steam/compatibilitytools.d`, or any directory you like. If you don't use Steam, set a **Custom tools directory** in Settings and point it to wherever you keep your compatibility tools. The launcher auto-detects them on startup.

---

## Contributing

Mythix Launcher is open source. Pull requests, bug reports, and feature requests are welcome.

---

## License

GPL-2.0 — see [LICENSE](LICENSE) for details.

---

Built by Mythix with steel, grit, and hard work beyond measure.
