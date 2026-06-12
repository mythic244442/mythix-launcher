# Mythix Launcher

A native Linux game launcher for Wine, Proton, and Neutron compatibility tools. Built with Tauri v2 + React. Designed for gamers who want full control over how their Windows games run on Linux - without the walled garden.

Mythix ships with first-class support for Mythix's new compatibility tool for gaming on Linux called [Neutron](https://github.com/mythix-org/mythix-build), but works with any Proton, GE-Proton, or standalone Wine build.

---

## Features

**Library Management**
- Add games from any source — Steam, GOG, Epic, Itch, Heroic, Amazon, or manual installs
- One-click Steam library import with automatic prefix cloning
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
- System-level guide button monitoring via `/dev/input/js*` — hide and restore the launcher even outside the webview
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

## Quick Start

### Prerequisites

- **Rust** (stable toolchain)
- **Node.js** 20+
- **Tauri CLI v2**: `cargo install tauri-cli@^2`
- **System libraries** (Ubuntu/Debian):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
  ```
- **Arch Linux**:
  ```bash
  sudo pacman -S webkit2gtk-4.1 libayatana-appindicator librsvg
  ```

### Install & Run

```bash
git clone https://github.com/mythix-org/mythix-launcher.git
cd mythix-launcher
npm install
cargo tauri dev          # development mode with hot reload
```

### Build for Release

```bash
cargo tauri build
```

The release binary lands in `src-tauri/target/release/mythix-launcher`.
A `.deb` and `.AppImage` are generated in `src-tauri/target/release/bundle/`.

---

## Bundled Compatibility Tools

Mythix Launcher release ships with:

- **Neutron** — A full compatibility tool built on Wine Proton bleeding-edge. Includes DXVK, VKD3D-Proton, and the Neutron launch script with FSR, GameMode, Gamescope, anti-cheat runtime, and async shader support. The build included is Neutron-11.0-21199 and is the latest version.

- **Mythix Wine** — A custom Neutron using a Wine built against Valve's Bleeding Edge Wine tree with patches applied from Wine HQ's staging patches and both GE and TkG patches as well as our own fixups and tweaks from Mythix. It is a Wine 10 build overall with fsync and esync and is our most experimental version so far.

Install them to `~/.steam/root/compatibilitytools.d`, `~/.steam/steam/compatibilitytools.d`, or any directory you like. If you don't use Steam, set a **Custom tools directory** in Settings and point it to wherever you keep your compatibility tools. The launcher auto-detects them on startup.

---

## Controller Mapping

Mythix supports standard gamepads (Xbox, PlayStation, Switch Pro) via the Gamepad API and system-level `/dev/input/js*` monitoring.

| Button | Action |
|--------|--------|
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
│   │   ├── useGamepad.js         # Gamepad API polling + button edge detection
│   │   ├── useControllerNav.js   # D-pad grid navigation + global shortcuts
│   │   ├── useRuntime.js         # Steam Runtime install/status
│   │   ├── useCoverUrl.js        # Cover art URL resolution
│   │   ├── useMaximized.js       # Window maximize state tracking
│   │   └── useToast.js           # Notification toasts
│   └── components/
│       ├── TitleBar.jsx          # Custom titlebar + controller legend
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
        ├── gamepad.rs            # System-level /dev/input/js* guide button monitor
        ├── runtime.rs            # Steam Runtime download + management
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
| Game library | `~/.local/share/mythix-launcher/library.json` |
| Global settings | `~/.local/share/mythix-launcher/settings.json` |
| Cover art cache | `~/.local/share/mythix-launcher/covers/` |
| Game prefixes | `~/.mythix/gamedata/<game_id>/` |
| Steam Runtime | `~/.local/share/mythix-launcher/runtime/` |
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

---

## Contributing

Mythix Launcher is open source. Pull requests, bug reports, and feature requests are welcome.

```bash
# Run in dev mode
npm install
cargo tauri dev

# Run frontend only (no Rust)
npm run dev

# Build for release
cargo tauri build
```

---

## License

GPL-2.0 — see [LICENSE](LICENSE) for details.

---

Built by **Luna** and **CC** with steel, grit, and an unreasonable amount of caffeine.
