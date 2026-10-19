<<<<<<< HEAD
# Mythix Launcher
```
                     ____________________
                _____   ..            .  _____
             ///.         .     .+            \\\
           //              |\     /\            .\\\
         //  .     .       |#\\  /##\               \\
       //.      +   .     |####_/#/##|  .             \
      (       .   .+     /###::##//###|     .     +.  .))
    (( +. .   +       /////:::########|   .   .  ..      )
   (   .          .///################|    .   .     +    )
   (      . ..    |/############\######\\               . )
  ( .     .   ___//:\\<o<#####\::\#####\\_\   .++          )
  (      ____/###############:##\\\\###\\#\|   ..          )
  ( ..   |################::#:\\#\:\#####\#\\   .           )
  (       \\#############:\\:\\\\\\:###:\\#\|      ..       )
  ( .       \'''''''#######///\\\\\#\:\\\##:#|         ..   )
  (                 |#######:#######\\########|      .     )
  (                 |:/#/::/:#:##\::##########\|       +   )
   (        .      |////////://##/########:\#:#|     +*   )
   (  .            |:/#///////:/:##########\\:##\ .       )
    ((        .    |/#/://///:##################\| +.    )
      (+      .    \/////#/:/::#####\#########\::\\    ))
       \           |/:#####/////////########::\\###|  /
        \\   +     |:########//:/#########\\:##:\#\<//
          \\\     |/#:/:##################/###/##///
             \\\   |##//\####################////
                ----|###/:################----
                    ----------------------
          __  ____     _______ _    _ _______   __
         |  \/  \ \   / /__   __| |  | |_   _\ \ / /
         | \  / |\ \_/ /   | |  | |__| | | |  \ V /
         | |\/| | \   /    | |  |  __  | | |   > <
         | |  | |  | |     | |  | |  | |_| |_ / . \
         |_|  |_|  |_|     |_|  |_|  |_|_____/_/ \_\
```
=======
# mythix-build
>>>>>>> 335fc48 (1.1.3 runtime update)

**Wine & Neutron builders, delegated Proton builder, hybrid installer, GUI toolkit, and installers — all in one repo.**

Build Wine from 10 upstream sources, compile full Neutron packages for Steam, build
GE-Proton and proton-tkg using their own upstream build systems, merge custom Wine
over existing Proton installs, manage installations with one-click switching, deploy
builds and download pre-built releases, and handle prefixes, DXVK, runtimes, and DLL
overrides through a zenity-based GUI. Or do it all from the CLI.

Now with a **Python frontend** — a Textual TUI, Tkinter GUI, and Click CLI that wrap
the battle-tested shell engines with modern keyboard navigation, live build progress,
and a tool catalogue.
```
                     ____________________
                _____   ..            .  _____
             ///.         .     .+            \\\
           //              |\     /\            .\\\
         //  .     .       |#\\  /##\               \\
       //.      +   .     |####_/#/##|  .             \
      (       .   .+     /###::##//###|     .     +.  .))
    (( +. .   +       /////:::########|   .   .  ..      )
   (   .          .///################|    .   .     +    )
   (      . ..    |/############\######\\               . )
  ( .     .   ___//:\\<o<#####\::\#####\\_\   .++          )
  (      ____/###############:##\\\\###\\#\|   ..          )
  ( ..   |################::#:\\#\:\#####\#\\   .           )
  (       \\#############:\\:\\\\\\:###:\\#\|      ..       )
  ( .       \'''''''#######///\\\\\#\:\\\##:#|         ..   )
  (                 |#######:#######\\########|      .     )
  (                 |:/#/::/:#:##\::##########\|       +   )
   (        .      |////////://##/########:\#:#|     +*   )
   (  .            |:/#///////:/:##########\\:##\ .       )
    ((        .    |/#/://///:##################\| +.    )
      (+      .    \/////#/:/::#####\#########\::\\    ))
       \           |/:#####/////////########::\\###|  /
        \\   +     |:########//:/#########\\:##:\#\<//
          \\\     |/#:/:##################/###/##///
             \\\   |##//\####################////
                ----|###/:################----
                    ----------------------
          __  ____     _______ _    _ _______   __
         |  \/  \ \   / /__   __| |  | |_   _\ \ / /
         | \  / |\ \_/ /   | |  | |__| | | |  \ V /
         | |\/| | \   /    | |  |  __  | | |   > <
         | |  | |  | |     | |  | |  | |_| |_ / . \
         |_|  |_|  |_|     |_|  |_|  |_|_____/_/ \_\
```
---

## Table of Contents

- [Quick Start](#quick-start)
- [Install](#install)
- [Python Frontend](#python-frontend)
  - [TUI (Textual)](#tui-textual)
  - [GUI (Tkinter)](#gui-tkinter)
  - [CLI (Click)](#cli-click)
- [Tools](#tools)
  - [wine-builder](#-wine-builder--mythix-wine_builder)
  - [neutron-builder](#-neutron-builder--mythix-neutron_builder)
  - [proton-builder](#-proton-builder--mythix-proton_builder)
  - [neutron-install](#-neutron-install--mythix-neutron-install)
  - [proton-install](#-proton-install--mythix-proton-install)
  - [wine-proton_hybrid](#-wine-proton_hybrid--mythix-wine-proton_hybrid_builder)
  - [wine-neutron_hybrid](#-wine-neutron_hybrid--mythix-wine-neutron_hybrid_builder)
  - [wine_toolz](#-wine_toolz--mythix-winetoolz)
  - [Wine Install Manager](#-wine-install-manager)
- [Project Layout](#project-layout)
- [Data Directories](#data-directories)
- [Building with Containers (Recommended)](#building-with-containers-recommended)
- [Requirements](#requirements)
- [Uninstall](#uninstall)

---

## Quick Start

```bash
git clone https://github.com/mythic244442/mythix-build
cd mythix-build
make install            # installs to ~/.local (adds PATH to ~/.bashrc)
source ~/.bashrc
```
mythix-build             # main menu — pick any tool
wine-builder            # build Wine interactively
neutron-builder         # build Neutron interactively
proton-builder          # build GE-Proton or proton-tkg (delegated)
proton-install          # download / deploy Proton packages
neutron-install         # deploy locally-built Neutron packages
wine_toolz              # open the GUI toolkit
wine-proton_hybrid      # hybrid installer (Proton base)
wine-neutron_hybrid     # hybrid installer (Neutron base)

### Python Quick Start

```bash
make py-dev             # editable install (pip install -e .[dev])
```
```bash
mythix                   # open the Textual TUI
```
```bash
mythix --gui             # open the Tkinter GUI
```
```bash
mythix launch wine_toolz # run a tool with full TTY handoff
```
```bash
mythix list            # print the tool catalogue
```
```bash
mythix doctor            # diagnose discovery + check what's found
```

---

## Install

```bash
make install                        # user install → ~/.local  (default)
```
```bash
make install PREFIX=/usr/local      # system-wide  (needs sudo)
```
```bash
make install DESTDIR=/tmp/pkg       # staged for packaging
```

Selective install:

```bash
make install-neutron         # neutron_builder only
```
```bash
make install-proton          # proton_builder only
```
```bash
make install-wine            # wine_builder only
```
```bash
make install-hybrid          # wine-proton hybrid_builder only
```
```bash
make install-neutron-hybrid  # wine-neutron hybrid_builder only
```
```bash
make install-neutron-install # neutron-install only
```
```bash
make install-toolz           # winetoolz only
```
```bash
make install-launcher        # mythix-build main menu only
```

The installer adds a `PATH` block to `~/.bashrc` automatically. If `~/.local/bin`
is already in your `PATH`, it's skipped. Run `source ~/.bashrc` after first install.

### Python Frontend Install

```bash
make py-dev             # editable install — pip install -e .[dev]
make py-test            # run the test suite (pytest)
make py-tui             # open the Textual TUI launcher
```

Or install directly with pip:

```bash
pip install -e .        # minimal: textual + click
pip install -e ".[dev]" # adds pytest, pytest-asyncio, textual-dev
```

After install, the `mythix` console script is available:

```bash
mythix                   # TUI launcher (default)
mythix --gui             # Tkinter GUI launcher
mythix launch <tool>     # run a tool with full TTY handoff
mythix build <tool>      # run a tool with live progress screen
mythix list              # print tool catalogue
mythix doctor            # diagnose tool discovery
mythix version           # print version
```

---

## Python Frontend

The `mythix_build_system/` Python package provides three frontends over the shell build
engines — a modern TUI, a graphical GUI, and a scriptable CLI. The shell scripts
still do all the real work; Python handles the launcher UX, tool discovery,
subprocess orchestration, and live build progress.

### TUI (Textual)

A full-featured terminal UI built with [Textual](https://github.com/Textualize/textual).
Same wolf banner and tool list as the bash launcher, rendered with keyboard navigation,
grouped categories, and a keybind footer.

```bash
mythix                   # open the TUI
make py-tui             # same thing via Makefile
python -m mythix_build_system   # same thing via Python module
```

**Keybindings:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate the tool list |
| `Enter` | Launch the highlighted tool (auto-picks best mode) |
| `b` | Open in Build View — captured output with live progress bar |
| `l` | Legacy launch — suspend TUI, hand TTY to the shell tool |
| `r` | Refresh tool availability (useful after `make install`) |
| `?` | Show keybinding help |
| `q` / `Esc` | Quit |

**Two launch modes:**

- **Legacy (suspend)** — the TUI suspends and gives the shell tool a full TTY.
  `fzf` pickers, `zenity` dialogs, and `read` prompts all work exactly like
  running the script directly. This is the default for interactive tools.

- **Build View** — the TUI stays up and streams the tool's stdout/stderr into a
  live log pane with a progress bar, elapsed clock, and section headings parsed
  from the shell output. Use this for long non-interactive builds where watching
  progress without losing the UI is more valuable than interactive prompts.

### GUI (Tkinter)

A graphical launcher window using Python's built-in Tkinter. Tools are grouped by
category with colored headers and hover effects. Clicking a tool opens it in your
terminal emulator.

```bash
mythix --gui                 # from the CLI
python -m mythix_build_system.gui   # direct module launch
```

**Supported terminal emulators** (auto-detected in order):
xfce4-terminal, gnome-terminal, konsole, alacritty, kitty, xterm.

The GUI shows each tool's resolved path and greys out tools that can't be found.
A status bar at the bottom shows which terminal emulator was detected.

### CLI (Click)

A [Click](https://click.palletsprojects.com/)-based command-line interface for
scripting and direct tool invocation without opening a TUI.

```bash
mythix list                          # pretty-print the tool catalogue
mythix list --format=keys            # one key per line (for scripting)
mythix list --format=plain           # tab-separated key/path/blurb
mythix launch wine-builder           # full TTY handoff
mythix launch neutron-builder -- --source proton-wine --branch proton_9.0
mythix build neutron-builder -- --dry-run
mythix doctor                        # show repo root + tool resolution
mythix version                       # print version string
```

| Command | Description |
|---------|-------------|
| `mythix` | Open the TUI launcher (default) |
| `mythix --gui` | Open the Tkinter GUI launcher |
| `mythix launch TOOL [ARGS...]` | Run a tool with full TTY handoff (fzf/zenity work) |
| `mythix build TOOL [ARGS...]` | Run a tool with live progress screen (non-interactive) |
| `mythix list` | Print the tool catalogue |
| `mythix doctor` | Diagnose discovery — repo root and resolved tool paths |
| `mythix version` | Print the version |

**Tool discovery** works in three modes:

1. **Installed with PATH** — `make install` put binaries in `~/.local/bin/` (or
   your `PREFIX/bin/`). `shutil.which` finds them.
2. **Source tree** — running from the repo checkout. Tools resolve via their
   relative path from the repo root.
3. **Installed without PATH** — probes `~/.local`, `/usr/local`, `/usr` for
   `bin/` and `lib/` layouts.

Set `$MYTHIX_ROOT` to explicitly point at a source checkout.

---

## Tools

### 🍷 wine-builder — mythix-wine_builder

Builds Wine from 10 upstream sources with full 32+64-bit cross-compile, ccache
support, TKG patch integration, and automated staging application.

```bash
wine-builder                            # interactive source + version picker
wine-builder --source mainline          # upstream Wine (stable tags)
wine-builder --source staging           # Wine-Staging (pre-patched)
wine-builder --source tkg-patched       # mainline + staging patches (automated)
wine-builder --source proton            # Valve proton-wine (branch picker)
wine-builder --source kron4ek           # Kron4ek wine-tkg build
wine-builder --source custom --url <git-url> --branch main
wine-builder --source local --local-dir ~/my-wine-src
wine-builder --resume                   # continue an interrupted build
wine-builder --list                     # show completed builds
wine-builder --uninstall                # remove a build (interactive picker)
```

#### Wine Sources

| Key | Source | Notes |
|-----|--------|-------|
| `mainline` | Wine mainline | Official WineHQ stable releases |
| `experimental` | Wine experimental | WineHQ bleeding-edge (master) |
| `staging` | Wine-Staging | Mainline + community patches, pre-applied |
| `tkg-patched` | Wine + Staging (automated) | Mainline with staging applied automatically |
| `proton` | Valve proton-wine | Valve's Wine fork (interactive branch picker) |
| `proton-experimental` | Valve Bleeding Edge | Valve's latest experiments |
| `wine-tkg` | wine-tkg (Frogging-Family) | Community patchset framework |
| `kron4ek` | Kron4ek wine-tkg | Kron4ek's Wine + Staging + TKG + ntsync |
| `local` | Local source | Use an existing directory (requires `--local-dir`) |
| `custom` | Custom URL | Any git repo (requires `--url`) |

#### Full CLI Reference

| Flag | Description |
|------|-------------|
| `--source NAME` | Wine source key (see table above) |
| `--branch BRANCH` | Pin to a specific branch or tag |
| `--url URL` | Git URL (required with `--source custom`) |
| `--local-dir PATH` | Existing source tree (required with `--source local`) |
| `--no-pull` | Skip `git pull` on existing source |
| `--dest DIR` | Root for `build-run/` and `install/` output |
| `--src-dir DIR` | Root for git-cloned sources |
| `--name NAME` | Custom build name (default: auto-generated) |
| `--jobs N` | Parallel make jobs (default: `nproc`) |
| `--cfg PATH` | Alternate `customization.cfg` |
| `--skip-32` | Skip 32-bit build |
| `--no-ccache` | Disable ccache |
| `--keep-symbols` | Keep debug symbols (skip strip) |
| `--build-type TYPE` | `release` (default) \| `debug` \| `debugoptimized` |
| `--native` | Compile with `-march=native` (faster, non-portable) |
| `--lto` | Enable link-time optimisation |
| `--resume` | Continue an interrupted build (skip configure) |
| `--verbose` | Stream raw make output |
| `--list` | Show installed builds |
| `--uninstall [NAME]` | Remove a build (interactive if no name given) |
| `--update` | Re-fetch and rebuild last-used source |
| `--dry-run` | Print planned actions without executing |

Build output lands in `buildz/install/<name>/` — a complete Wine prefix-ready
tree with `bin/wine`, `lib/`, etc.

---

### 🎮 neutron-builder — mythix-neutron_builder

Builds a complete **Neutron package** (a Steam-compatible Proton tool) from source: a Wine build
(proton-wine, GE-Proton, or kron4ek-tkg) compiled with MinGW PE support, plus DXVK and
VKD3D-Proton, all assembled into a Steam-loadable package. The result drops into
`compatibilitytools.d/` and appears in Steam's per-game Compatibility dropdown —
just like GE-Proton, but built locally from the sources you choose with the
compiler flags you want.

```bash
neutron-builder --profile mythix-11.19 --container  # Mythix Wine 11.19 (pinned, Wayland-first)
neutron-builder --list-profiles                  # pinned Mythix Wine recipes
neutron-builder                                  # interactive wizard
neutron-builder --source proton-wine             # Valve proton-wine (branch picker)
neutron-builder --source proton-wine --branch proton_9.0
neutron-builder --source ge-proton               # GE-Proton — proton-wine + GE gaming patches
neutron-builder --source kron4ek-tkg             # Kron4ek wine-tkg + ntsync
neutron-builder --patches all                    # apply all patch groups
neutron-builder --patches custom                 # apply specific groups
neutron-builder --sniper                         # enable Steam Runtime Sniper container
neutron-builder --dxvk dxvk                      # compile DXVK from source (instead of downloading)
neutron-builder --vkd3d vkd3d-proton             # compile VKD3D-Proton from source
neutron-builder --dxvk-only                      # rebuild DXVK only, skip Wine
neutron-builder --vkd3d-only                     # rebuild VKD3D-Proton only
neutron-builder --reinstall-components           # re-package without rebuilding
neutron-builder --list                           # show installed Neutron builds
```

#### Mythix Wine profiles

Mythix Wine flavours are built from **profiles**: pinned recipes in
`mythix-neutron_builder/profiles/`. Every upstream input is pinned, and every
build writes a `mythix-source.lock` into the package. Today there are three:

- `mythix-11.19`: GE-based, Wayland-first.
- `mythix-11.11`: safety net.
- `mythix-10.17`: frozen esync/fsync build.

See [`profiles/README.md`](mythix-neutron_builder/profiles/README.md).

#### Proton Wine Sources

| Key | Source | Notes |
|-----|--------|-------|
| `mainline` | WineHQ mainline | Official stable releases (version picker) |
| `experimental` | WineHQ experimental | Bleeding-edge master branch |
| `staging` | Wine-Staging | Mainline + community patches (version picker) |
| `proton-wine` | Valve proton-wine | Stable branches with interactive version picker |
| `proton-wine-experimental` | Valve proton-wine experimental | Bleeding-edge, no picker |
| `ge-proton` | GE-Proton (GloriousEggroll) | proton-wine + GE's full gaming patch set (version picker) |
| `kron4ek-tkg` | Kron4ek wine-tkg | Mainline + Staging + TKG patches + ntsync |

#### GE Neutron (GloriousEggroll)

Select `ge-proton` as your source to build a **GE Neutron** — proton-wine with
GloriousEggroll's full gaming patch set applied automatically. The version picker
shows GE release tags (GE-Proton9-20, etc.) and resolves the matching proton-wine
branch. The builder clones `proton-ge-custom`, runs GE's `protonprep-valve-staging.sh`
to apply all patches, then compiles through the normal Neutron pipeline.

**What GE's patches include:**
- Wine-Staging (with GE's curated exclusions for Proton compatibility)
- Wine-Wayland patches
- ntsync hotfixes
- FSR (FidelityFX Super Resolution) fullscreen hack
- NVIDIA Reflex / VK_NV_low_latency2 support
- Media Foundation / GStreamer codec support (in-game cutscenes, video playback)
- Game-specific fixes: Star Citizen, Dragon Age Inquisition, Assetto Corsa, PSO2,
  Le Mans Ultimate, Ghost of Tsushima (PSN login), Clannad, and more
- Anti-cheat compatibility: EAC host block, hidden Wine exports
- Performance: fast audio polling, ALSA channel override, exe relocation
- DLSS upgrade patches, OpenXR support
- Unity crash hotfix, D2D crash fix, write-watches spam reduction

**First successful build:** 95,182 lines compiled, 106 files patched, 576 x64 DLLs +
572 x86 DLLs, 1.5 GB packaged — zero build errors.

```bash
neutron-builder --source ge-proton                    # interactive GE release picker
neutron-builder --source ge-proton --branch GE-Proton9-20  # pin to specific release
```

The resulting build appears in Steam as `mythix-ge-neutron-<version>`.

**Post-patch fixups:** The builder automatically fixes known incompatibilities between
GE's patches and certain proton-wine branches (e.g., `close_inproc_sync_obj` →
`NtClose` in ntdll thread suspension code).

#### DXVK & VKD3D-Proton Options

| DXVK Key | Description |
|----------|-------------|
| `dxvk-release` | Pre-built DXVK DLLs from GitHub releases (default, fastest) |
| `dxvk` | Standard DXVK — D3D9/10/11 → Vulkan, compiled from source (requires MinGW) |
| `dxvk-async` | DXVK + async pipeline compilation, compiled from source |
| `none` | Skip DXVK (falls back to WineD3D) |

| VKD3D Key | Description |
|-----------|-------------|
| `vkd3d-proton-release` | Pre-built VKD3D-Proton DLLs from GitHub releases (default, fastest) |
| `vkd3d-proton` | VKD3D-Proton — D3D12 → Vulkan, compiled from source (requires MinGW) |
| `none` | Skip VKD3D-Proton (D3D12 games won't work) |

#### Full CLI Reference

| Flag | Description |
|------|-------------|
| `--profile NAME` | Build a pinned Mythix Wine profile (`profiles/NAME`) |
| `--list-profiles` | Show available profiles |
| `--build-mode MODE` | `classic` (wine64 + 32-bit tree) \| `wow64` (unified) |
| `--allow-patch-failures` | Let profile patch layers fail (re-curation only) |
| `--source NAME` | Wine source key |
| `--branch BRANCH` | Pin proton-wine to a specific branch |
| `--no-pull` | Skip `git pull` |
| `--dxvk NAME` | DXVK variant (`dxvk-release` \| `dxvk` \| `dxvk-async` \| `none`) |
| `--vkd3d NAME` | VKD3D variant (`vkd3d-proton-release` \| `vkd3d-proton` \| `none`) |
| `--dxvk-branch BRANCH` | Pin DXVK to specific tag |
| `--vkd3d-branch BRANCH` | Pin VKD3D-Proton to specific tag |
| `--name NAME` | Build name (default: `mythix-neutron-<ver>`) |
| `--dest DIR` | Root for build artefacts |
| `--src-dir DIR` | Root for git clones |
| `--jobs N` | Parallel make threads (default: `nproc`) |
| `--skip-32` | Skip 32-bit Wine build |
| `--no-ccache` | Disable ccache |
| `--keep-symbols` | Keep debug symbols |
| `--build-type TYPE` | `release` \| `debug` \| `debugoptimized` |
| `--native` | `-march=native` optimisation |
| `--lto` | Link-time optimisation |
| `--resume` | Continue interrupted build |
| `--dxvk-only` | Rebuild DXVK only (skip Wine) |
| `--vkd3d-only` | Rebuild VKD3D-Proton only (skip Wine) |
| `--reinstall-components` | Re-package without rebuilding |
| `--patches GROUPS` | Patch groups to apply (`all`, `none`, or comma-separated names) |
| `--patches-dir PATH` | Alternate patches directory |
| `--sniper` | Enable Steam Runtime Sniper mode (container isolation) |
| `--cfg PATH` | Alternate `neutron-customization.cfg` |
| `--list` | Show installed Neutron builds |
| `--dry-run` | Print planned actions |

#### How neutron Works

neutron-builder assembles a **self-contained Neutron package** — the same
kind of package that Valve ships as Proton. The key
difference from a regular Wine build is that every component is compiled to run
inside Steam's Proton infrastructure:

**Wine with MinGW PE modules.** The `--with-mingw` configure flag tells Wine to build
its Windows DLLs (`.dll`) as genuine PE (Portable Executable) binaries using the
MinGW cross-compiler, rather than ELF Winelib objects. This is what makes Steam's
FPS counter overlay, Steam Input, and DRM/anti-cheat systems work correctly — they
require real PE DLLs to inject into.

**DXVK and VKD3D-Proton are statically self-contained.** The custom DXVK and
VKD3D-Proton builds produced by neutron-builder are compiled against bundled headers
(Vulkan, SPIR-V, DirectX) and linked without MinGW runtime DLL dependencies. The
resulting `.dll` files are fully standalone — no `libstdc++-6.dll` or other MinGW
runtime DLLs needed at runtime.

**toolmanifest.vdf — host-native or Sniper container.** By default, the generated
`toolmanifest.vdf` omits `require_tool_appid "1391110"`, so neutron packages run
**host-native** — direct access to Vulkan drivers, kernel features, and ntsync/fsync.
Pass `--sniper` to enable **Steam Runtime Sniper mode** instead, which adds
`require_tool_appid "1391110"` and runs the package inside Steam's SteamOS 3.x
container for SteamDeck-like isolation. The interactive builder offers this as an
fzf picker during the build wizard.

#### The neutron Python Launcher

Each built package contains a `neutron` Python script that Steam invokes in place of
Proton's `proton` launcher. It handles the full Steam Proton protocol:

**Steam verbs** — Steam passes a verb as the first argument to tell the launcher what
to do. `neutron` handles all standard Proton verbs:

| Verb | Behavior |
|------|----------|
| `waitforexitandrun` | Run the game via Wine and wait for it to exit (most common) |
| `run` | Run without waiting (Steam polls for exit) |
| `runinprefix` | Run a helper command inside the Wine prefix |
| `getcompatpath` | Convert a Unix path to a Windows path (printed to stdout) |
| `getnativepath` | Convert a Windows path to a Unix path (printed to stdout) |
| `stop` | Kill the wineserver for this prefix |
| `createprefix` | Create a prefix using a base Proton, then overlay Neutron DLLs (`--proton PATH`) |
| `destroyprefix` | Remove exactly the files Neutron installed (tracked-files manifest) |
| `makedefaultpfx` | (Re)create the prefix template (`--force` to regenerate) |
| `diag` | Print resolved configuration without launching anything (`--full-env` for all vars) |

**Sync primitive auto-detection.** The launcher inspects the `wineserver` binary for
compiled-in sync support and enables the best available mode automatically — no
manual env var required:

- **NTSync** — if `wineserver` has ntsync support compiled in (kron4ek-tkg builds
  do; Valve's proton-wine does not), ntsync activates automatically when
  `/dev/ntsync` is present. `WINEFSYNC=1` is also set as a fallback.
- **fsync** — enabled via `WINEFSYNC=1` when ntsync is absent but fsync is compiled
  in (Valve proton-wine).
- **esync** — enabled via `WINEESYNC=1` when only esync is available.
- The user's externally-set values are always respected (`setdefault` is used).

**Runtime gaming optimizations.** The launcher automatically applies performance
tuning that you'd otherwise have to set manually:

- **DXVK async** — `DXVK_ASYNC=1` enabled by default for stutter-free shader compilation.
- **Shipped `dxvk.conf`** — auto-loaded from the package's `files/` directory with
  async shaders, state cache, and sane defaults. Override with `DXVK_CONFIG_FILE`.
- **AMD GPU hints** — `RADV_PERFTEST=gpl,nggc,sam` for graphics pipeline library,
  NGG culling, and Smart Access Memory on RADV.
- **GameMode integration** — auto-detects [Feral GameMode](https://github.com/FeralInteractive/gamemode)
  and wraps the game process with `gamemoderun` for CPU governor and nice priority
  optimization. Controlled via `NEUTRON_GAMEMODE=auto|1|0`.
- **MangoHud passthrough** — set `MANGOHUD=1` in Steam launch options and it just
  works. The launcher sets `MANGOHUD_DLSYM=1` for proper hooking.

**Prefix initialization on first launch.** When a game runs for the first time in a
fresh prefix (no `system.reg` present), `neutron` automatically:

1. Creates a default prefix template by running `wineboot --init` once (cached for
   future launches — subsequent prefixes clone from it via symlinks + reflink copies).
2. Clones the template into the game's prefix with a unique MachineGuid.
3. Overlays DXVK DLLs (`d3d9.dll`, `d3d10*.dll`, `d3d11.dll`, `dxgi.dll`) and
   VKD3D-Proton DLLs (`d3d12.dll`, `d3d12core.dll`) into the prefix, tracked by a
   manifest so `destroyprefix` can cleanly remove them.
4. Installs Steam client DLLs, NVIDIA nvngx/DLSS DLLs, and VR client DLLs as needed.
5. Sets up DOS drives, OpenVR paths, and the `.update-timestamp` suppressor.

This means DXVK and VKD3D-Proton are active from the very first game launch with no
manual setup step.

**WINEDLLOVERRIDES — forcing native over builtin.** The launcher sets:

- Graphics DLL overrides (`d3d9=n,b`, `d3d10core=n,b`, `d3d11=n,b`, `dxgi=n,b`,
  `d3d8=n,b`, `d3d12=n,b`, `d3d12core=n,b`) — ensures DXVK and VKD3D-Proton DLLs
  take priority over Wine builtins.
- Steam bridge override (`lsteamclient=n,b`).

**Steam client bridge (builds with `STEAM_BRIDGE=ge` in `build-info`).** These
builds ship an `lsteamclient` built with the same Wine. A launch goes through
Proton's `steam.exe` helper, which registers the Steam client for the game, only
when both of these hold:

- `SteamGameId` (or `SteamAppId`) is a real Steam app id (all digits).
- The Steam client is running.

Other launches go straight to the game, for example the Mythix Launcher's hex
ids for non-Steam games, or any launch while Steam is closed.
`NEUTRON_USE_STEAM_SHIM=1` forces `steam.exe` and `=0` turns it off. Steam's
`steamclient{,64}.dll` are copied into the prefix only for `steam.exe` launches.

**DualShock 4 / DualSense.** On 11.19+ builds, xinput carries GE's Sony stack.
By default a Sony pad on hidraw is also an XInput controller
(`PROTON_SONY_AUTO_XINPUT=1`, GE-Proton's own default). That slot steps aside
when a game reads the pad natively or when Steam Input owns it.

For games with native PlayStation support, set `MYTHIX_DS4_GLYPHS=1`. The pad
then never shows up in `XInputGetState`, so the game reads it over HID and
shows PlayStation glyphs. `XInputSetState` still rumbles it. The Mythix
Launcher sets it for Cyberpunk 2077, RDR2, Elden Ring and The Witcher 3.
`neutron diag` shows the active mode.

#### Patch System

neutron-builder includes a full patch system (`neutron-patcher.sh`) that applies
patch groups to the Wine source between fetch and configure. For GE-Proton's
gaming patches, use `--source ge-proton` instead — it runs GE's own patch script
automatically. The patch system below is for your own custom patches.

`patches/` ships with these groups:

- `mythix-ds4`: Mythix DualShock 4 support in winebus (hidraw haptics and LED).
- `mythix-ds4-xinput`: Luna's xinput DS4 work for the Neutron 11.0 line, on
  Valve's stock xinput. The 11.19 profile carries its own xinput patch instead.
- `custom/`: an empty template.

Drop your own `.patch` files into `custom/` (or any new subdirectory) and the
patcher auto-discovers them.

**Usage:**

```bash
neutron-builder --patches all                    # apply everything discovered
neutron-builder --patches custom                 # apply only the custom group
neutron-builder --patches none                   # skip patching
neutron-builder                                  # interactive fzf multi-picker
```

**Adding patches:** Place `.patch` files in `patches/custom/` or create
additional subdirectories under `patches/`. Each directory with `.patch` files
is treated as a patch group. Add an optional `group.conf` for metadata:

```ini
description="My custom patches for game X"
priority=50
sources=
conflicts=
requires=
```

A `series` file (one filename per line) controls application order. Without it,
patches are applied in sorted filename order (`0001-*.patch` convention).

See `patches/custom/README.md` for full documentation on group.conf fields,
naming conventions, and examples.

The patcher creates a git checkpoint before applying, so you can revert with
`git checkout neutron-pre-patch-<timestamp>` in the Wine source directory.

#### Steam Component Bootstrap

neutron-builder auto-bootstraps proprietary Steam components during packaging.
It searches local Proton installs (Hotfix, Experimental, 9.0, 8.0, and any
`Proton*` directory) across all Steam library roots. If no local Proton is found,
it **downloads from [Kron4ek/proton-archive](https://github.com/Kron4ek/proton-archive)**
(defaults to proton-10.0-4).

**Components bootstrapped:**

| Component | Files | Purpose |
|-----------|-------|---------|
| `lsteamclient` | `.dll` (PE) + `.so` (Unix bridge), 32+64-bit | Steam API bridge — without it, Steamworks games hang at startup |
| `steam_helper.exe` | 32+64-bit | Steam overlay helper / steamwebhelper bridge |
| `steam.exe` | 32+64-bit | Steam client stub expected by some games |
| `gameoverlayrenderer.so` | 32+64-bit | In-game Steam overlay (Shift+Tab) |
| `steamclient.dll` | 32+64-bit | Steam client library |

Additional Steam library paths can be provided via the `STEAM_LIBRARY_PATHS`
environment variable (newline-separated).

#### NTSync Support

NTSync is a Linux kernel synchronization mechanism (introduced in kernel 6.14) that
dramatically reduces CPU overhead for Wine's thread synchronization. For builds using
kron4ek-tkg Wine, ntsync support is compiled in automatically — **no configure flag
is needed**. The `Containerfile.neutron` fetches the `ntsync.h` kernel header and
places it where Wine's configure scripts can find it. If `ntsync.h` is present at
build time, Wine's configure detects it and includes ntsync support. The `neutron`
launcher then activates it at runtime when `/dev/ntsync` exists.

For Valve's proton-wine fork, ntsync is not compiled in (Valve manages that
separately in their official builds); fsync is used instead.

#### neutron-customization.cfg

Copy `neutron-customization.cfg` and pass it with `--cfg` to tune the build for your
machine. Key knobs:

| Setting | Default | Description |
|---------|---------|-------------|
| `CC_64` / `CXX_64` | `x86_64-linux-gnu-gcc/g++` | Native (ELF side) compilers for 64-bit |
| `CC_32` / `CXX_32` | `i686-linux-gnu-gcc/g++` | Native compilers for 32-bit |
| `MINGW_CC_64` / `MINGW_CXX_64` | `x86_64-w64-mingw32-gcc/g++` | MinGW cross-compiler for 64-bit PE DLLs |
| `MINGW_CC_32` / `MINGW_CXX_32` | `i686-w64-mingw32-gcc/g++` | MinGW cross-compiler for 32-bit PE DLLs |
| `JOBS` | `$(nproc)` | Parallel build jobs |
| `_configure_args` | (array) | Configure flags applied to both 32-bit and 64-bit runs |
| `_configure_args32` | (array) | Configure flags for the 32-bit build only |
| `_configure_args64` | (array) | Configure flags for the 64-bit build only |
| `CFLAGS` / `CXXFLAGS` | `-O3 -march=native -mtune=native` | Compiler optimization flags (native side) |
| `CROSSCFLAGS` / `CROSSCXXFLAGS` | `-O3 -march=native -mtune=native` | Compiler flags for MinGW PE modules |

The default `CFLAGS`/`CROSSCFLAGS` use `-O3 -march=native -mtune=native`. This
gives a meaningful performance boost for Wine's hot paths over the default `-O2`, but
the resulting binaries are tuned for your CPU and **should not be copied to a machine
with a different CPU family**. Remove `-march=native -mtune=native` if you need a
portable build.

`_configure_args`, `_configure_args32`, and `_configure_args64` must be Bash arrays.
`neutron-build-core.sh` re-sources the cfg file to expand them at configure time.

#### GTA IV — Real-World Test Case

GTA IV on Steam works correctly with a neutron-built kron4ek-tkg 11.5 Wine package.
It is a useful reference case because it exercises several things simultaneously:

- **lsteamclient** — GTA IV's launcher calls SteamAPI. Without lsteamclient, the
  game hangs at a black screen. The auto-bootstrap step picks it up from Proton
  Experimental.
- **OpenVR disabled** — GTA IV bundles `openvr_api.dll`. Without the
  `openvr_api_dxvk=disabled;vrclient_x64=disabled;vrclient=disabled` overrides, Wine
  tries to load the VR client and hits an assertion crash at startup.
- **DXVK in prefix** — GTA IV uses D3D9. The first-launch prefix initialization
  installs DXVK automatically, so there is no manual `setup_dxvk.sh` step.
- **Host-native launch** — the `toolmanifest.vdf` omits `require_tool_appid`, so
  Steam runs the package directly without the SLR Sniper container. This is what
  makes ntsync and direct Vulkan driver access work correctly.

#### Installing to Steam

After a successful build:

```bash
cp -r ~/.local/share/mythix-neutron_builder/buildz/install/<name> \
      ~/.steam/steam/compatibilitytools.d/
```

Or use `neutron-install --deploy <path>` (see below) to copy and set permissions in
one step.

Restart Steam, then go to a game's Properties → Compatibility → select your build.

---

### 🔧 proton-builder — mythix-proton_builder

Builds **GE-Proton** or **proton-tkg** using their own upstream build systems —
"delegated builds." Unlike neutron-builder (which compiles Wine, DXVK, and VKD3D from
source), proton-builder clones the upstream project and runs *their* build scripts
inside containers. The result is identical to what the upstream maintainer ships, but
compiled locally on your machine.

Both GE-Proton and proton-tkg bring their own container environments (Podman or
Docker), so the only hard dependencies are `git`, `curl`, and a container engine.

```bash
proton-builder                              # interactive menu (ge, tkg, list, clean)
proton-builder --source ge                  # build latest GE-Proton
proton-builder --source tkg                 # build proton-tkg
proton-builder --list                       # show completed builds
proton-builder --clean                      # remove source checkouts & intermediates
proton-builder --dry-run --source ge        # preview GE-Proton build steps
```

#### Build Sources

| Key | Source | Notes |
|-----|--------|-------|
| `ge` | [GE-Proton](https://github.com/GloriousEggroll/proton-ge-custom) | GloriousEggroll's Proton fork — `configure.sh` + `make dist` inside the `umu-sdk` container |
| `tkg` | [proton-tkg](https://github.com/Frogging-Family/wine-tkg-git) | Frogging-Family's proton-tkg — `proton-tkg.sh` with its own Valve SDK container |

#### How It Works

**GE-Proton:** Clones `proton-ge-custom`, checks out the latest release tag,
runs `./configure.sh --container-engine=<engine> --build-name=<name>` followed by
`make dist`. The build happens inside the `ghcr.io/open-wine-components/umu-sdk:latest`
container. Output is copied to the install directory.

**proton-tkg:** Clones `wine-tkg-git`, enters the `proton-tkg/` subdirectory, and
runs `./proton-tkg.sh`. TKG handles its own container setup via the Valve SDK. The
`proton-tkg.cfg` configuration file can be edited before the build starts.

#### CLI Reference

| Flag | Description |
|------|-------------|
| `--source ge` | Build GE-Proton from source |
| `--source tkg` | Build proton-tkg from source |
| `--build-name NAME` | Override the build name (default: auto from upstream tag) |
| `--jobs N` | Parallel build jobs (default: `nproc`) |
| `--container-engine ENG` | Force `podman` or `docker` (default: auto-detect) |
| `--list` | List completed Proton builds |
| `--clean` | Remove source checkouts and build intermediates |
| `--dry-run` | Show what would happen without building |

Build output lands in `~/.local/share/mythix-proton_builder/buildz/install/`. Deploy
with `proton-install --deploy <path>` or the interactive `proton-install` menu.

---

### 🚀 neutron-install — mythix-neutron-install

A full CLI package manager for Neutron builds. Installs, deploys to Steam, sets up
as system Wine, switches active versions, and manages installed Neutron packages —
all with fzf pickers and numbered fallbacks.

```bash
neutron-install                             # interactive menu (7 actions)
neutron-install install                     # install from builder output / dir / tarball
neutron-install deploy                      # deploy to Steam compatibilitytools.d
neutron-install system-wine                 # install as system Wine (symlinks to ~/.local/bin)
neutron-install switch                      # switch active Neutron (swap symlinks)
neutron-install list                        # list managed installs with metadata
neutron-install remove                      # remove one or more installs
neutron-install info                        # disk usage, version, architecture details
```

#### Features

| Action | Description |
|--------|-------------|
| **Install** | Install from neutron-builder output, a directory, or a tarball |
| **Deploy** | Copy to `~/.steam/*/compatibilitytools.d/` for Steam |
| **System Wine** | Symlink Wine binaries + `neutron` launcher to `~/.local/bin/` |
| **Switch** | Change which install is the active system Wine |
| **List** | Show all managed installs with version, date, active status |
| **Remove** | Remove installs and clean up symlinks |
| **Info** | Disk usage, Wine version, source, contents breakdown |

**Managed installs** go to `~/.local/share/mythix-neutron-installs/<name>/` with
`.mythix-meta` metadata. Symlinks include all Wine binaries (`wine`, `wine64`,
`wineserver`, `wineboot`, `winecfg`, etc.) plus the `neutron` Python launcher.

---

### 🚀 proton-install — mythix-proton-install

Download pre-built Proton releases and deploy locally-built Proton packages into
Steam's `compatibilitytools.d/`. Handles GE-Proton downloads, any GitHub release
tarball URL, and deploying proton-builder output — all in one step.

```bash
proton-install                          # interactive menu
proton-install --list                   # list installed compatibility tools
proton-install --install-ge             # download + install latest GE-Proton
proton-install --install-url URL        # download + install from any URL
proton-install --deploy PATH            # deploy a locally-built package
proton-install --remove NAME            # remove an installed tool
proton-install --compat-dir DIR         # use a non-default compatibilitytools.d
proton-install --dry-run                # show planned actions without executing
```

#### What It Does

`proton-install` manages the `~/.steam/*/compatibilitytools.d/` directory so you do
not have to manually `cp -r` and `chmod` packages yourself. It:

- Auto-discovers Steam's `compatibilitytools.d` across common Steam installation
  paths (standard `~/.steam/steam`, Debian/Ubuntu `~/.steam/debian-installation`,
  Flatpak, etc.).
- Downloads GE-Proton releases from the official GitHub releases page.
- Accepts any direct download URL for other community Proton builds.
- Deploys locally-built Proton packages (from proton-builder) via the `deploy-local`
  action / `--deploy PATH` flag: copies the package directory into
  `compatibilitytools.d/` and sets correct permissions.
- Lists currently installed compatibility tools with name and path.
- Removes a named tool and its directory.

After any install, remove, or deploy operation, **restart Steam** to pick up the
changes.

#### CLI Reference

| Flag | Description |
|------|-------------|
| `--install-ge` | Download and install the latest GE-Proton release |
| `--install-url URL` | Download and install a Proton release from a direct URL (tar.gz / tar.xz / tar.zst) |
| `--deploy PATH` | Deploy a locally-built Proton package (proton-builder output, any compatible directory) into compatibilitytools.d |
| `--list` | List installed compatibility tools |
| `--remove NAME` | Remove a named compatibility tool |
| `--compat-dir DIR` | Override the auto-discovered compatibilitytools.d path |
| `--dry-run` | Print planned actions without executing anything |

#### After Changes

Always restart Steam after installing, deploying, or removing a compatibility tool:

```bash
# Graceful restart:
steam -shutdown && steam
```

The new tool will appear (or disappear) under a game's Properties → Compatibility.

---

### 🔀 wine-proton_hybrid — mythix-wine-proton_hybrid_builder

Merges any Wine build directly over an existing Proton install, preserving Proton's
DXVK, VKD3D-Proton, Steam overlay DLLs, and Python launcher. Run your custom Wine
inside Proton's infrastructure — get the best of both worlds.

```bash
wine-proton_hybrid                          # interactive wizard
wine-proton_hybrid \
    --wine-src ~/builds/wine-staging-10.5 \
    --proton-src ~/.steam/steam/compatibilitytools.d/GE-Proton10-32 \
    --name my-hybrid \
    --install-mode steam
wine-proton_hybrid --uninstall --name my-hybrid
```

#### CLI Reference

| Flag | Description |
|------|-------------|
| `--wine-src DIR` | Path to the custom Wine build directory |
| `--proton-src DIR` | Path to the Proton/GE-Proton source |
| `--name NAME` | Tool name (default: `wine-proton_mythix`) |
| `--protonfixes-dir DIR` | Path to protonfixes source (umu-protonfixes, etc.) |
| `--install-mode MODE` | `steam` \| `steam-pick` \| `custom` |
| `--install-dir DIR` | Parent directory for custom installs |
| `--dry-run` | Show commands without executing |
| `--verbose` | Print every command before running |
| `--debug` | Dump Proton lib/wine layout after install |
| `--uninstall` | Remove a previously installed hybrid |

#### Standalone Launcher Environment Variables

The hybrid installs a standalone launcher script. Control it with:

| Variable | Description |
|----------|-------------|
| `MYTHIX_PREFIX` | Exact prefix path (default: `~/.wine-proton-pfx`) |
| `WINE_USE_START` | Set to `1` for launcher-wrapped games (e.g., GTA IV) |
| `PROTON_LOG` | Set to `1` for verbose Wine debug log in `/tmp/` |
| `DXVK_HUD` | Set to `1` for DXVK overlay |
| `WINEARCH` | `win64` (default) or `win32` |

---

### 🔀 wine-neutron_hybrid — mythix-wine-neutron_hybrid_builder

Merges any Wine build directly over an existing **Neutron** package base, producing a
single hybrid tool that can be registered with Steam as a compatibility tool or used
standalone via `./neutron run <game.exe>`. Same concept as wine-proton_hybrid but
targeting Neutron packages instead of Proton installs.

```bash
wine-neutron_hybrid                          # interactive wizard
wine-neutron_hybrid \
    --wine-src ~/builds/wine-staging-10.5 \
    --neutron-src ~/.steam/steam/compatibilitytools.d/mythix-neutron-9.0 \
    --name my-hybrid \
    --install-mode steam
wine-neutron_hybrid --uninstall --name my-hybrid
```

#### CLI Reference

| Flag | Description |
|------|-------------|
| `--wine-src DIR` | Path to the custom Wine build directory |
| `--neutron-src DIR` | Path to the Neutron package directory |
| `--name NAME` | Tool name (default: `wine-neutron_mythix`) |
| `--protonfixes-dir DIR` | Path to protonfixes source (umu-protonfixes, etc.) |
| `--install-mode MODE` | `steam` \| `steam-pick` \| `custom` |
| `--install-dir DIR` | Parent directory for custom installs |
| `--dry-run` | Show commands without executing |
| `--verbose` | Print every command before running |
| `--debug` | Dump Neutron lib/wine layout after install |
| `--uninstall` | Remove a previously installed hybrid |

#### Standalone Launcher Environment Variables

The hybrid installs a standalone launcher script. Control it with:

| Variable | Description |
|----------|-------------|
| `MYTHIX_PREFIX` | Exact prefix path (default: `~/.wine-neutron-pfx`) |
| `WINE_USE_START` | Set to `1` for launcher-wrapped games (e.g., GTA IV) |
| `PROTON_LOG` | Set to `1` for verbose Wine debug log in `/tmp/` |
| `DXVK_HUD` | Set to `1` for DXVK overlay |
| `WINEARCH` | `win64` (default) or `win32` |

---

### 🛠️ wine_toolz — mythix-winetoolz

A zenity-based GUI for managing Wine prefixes, installing graphics layers, runtimes,
and components. 20+ modules across 7 categories.

```bash
wine_toolz
```

#### Categories & Modules

| Category | Module | Description |
|----------|--------|-------------|
| **Graphics** | DXVK Installer | Install/uninstall DXVK (D3D8/9/10/11 → Vulkan) into a prefix |
| | VKD3D-Proton Installer | Install/uninstall VKD3D-Proton (D3D12 → Vulkan) |
| | DXVK-NVAPI Installer | NVIDIA API layer for DLSS / NvAPI support |
| **Runtimes** | Runtime Libraries | .NET, XNA, XACT, VC++, DirectPlay, OpenAL, and more |
| | DirectX Jun 2010 | Legacy DirectX offline redistributable |
| **Wine** | Wine Tools | Uninstaller, control panel, taskmgr, regedit, explorer, wineboot |
| | DLL Override Manager | View, apply presets, add custom, remove DLL overrides |
| | **Wine Install Manager** | **Install, switch, uninstall, and manage custom Wine builds** |
| **Prefix** | Create Prefix | Bootstrap a new Wine/Proton prefix from scratch |
| | Winecfg | Open Wine configuration for an existing prefix |
| | Prefix Manager | Backup, restore, regedit, kill processes |
| | Prefix Diagnostics | Health check — arch, Windows ver, DXVK, disk usage, integrity |
| **Launch** | App Launcher | Save and launch Wine app shortcuts with env profiles |
| | Env Flags | Manage DXVK/VKD3D/Wine/Mesa environment variable profiles |
| | Log Viewer | Run an app and capture Wine stderr to a log file |
| **Components** | DLL Installer | Curated DLLs — d3dx9/10/11, d3dcomp, xinput, msxml, physx, more |
| | System File Installer | Extract + install DLLs from any archive into a prefix |
| **System** | About / Help | Dependency checker, module reference, config paths |

#### Smart Wine Binary Detection

Every module that needs a Wine binary uses `wt_select_wine_bin`, which auto-scans:

1. Custom Wine builds under `~/wine-custom/buildz/`
2. Proton/GE-Proton in `~/.steam/steam/compatibilitytools.d/`
3. System `wine` / `wine64` on `$PATH`
4. Manual browse fallback

Proton wrappers are automatically resolved to their inner wine binary.

#### Environment Profiles (Env Flags)

Create reusable environment variable profiles for launching games:

- **DXVK:** `DXVK_HUD`, `DXVK_ASYNC`, `DXVK_FRAME_RATE`
- **VKD3D:** `VKD3D_DEBUG`, `VKD3D_SHADER_DEBUG`
- **Wine:** `WINEDEBUG`, `WINEESYNC`, `WINEFSYNC`, `STAGING_SHARED_MEMORY`
- **Mesa/AMD:** `mesa_glthread`, `RADV_PERFTEST`
- **Vulkan ICD:** Force AMD or NVIDIA ICD

Profiles are saved to `~/.config/winetoolz/env_profiles/<name>.env` and can be
attached to app launcher shortcuts.

#### Prefix Diagnostics

Run a full health check on any prefix:

- Architecture detection (win32 vs win64)
- Windows version from registry
- Disk usage breakdown
- DLL override count
- Translation layer detection (DXVK, VKD3D presence)
- Key file integrity (system.reg, wineboot.exe, explorer.exe, etc.)
- Overall health verdict with actionable issue listing

---

### 📦 Wine Install Manager

A dedicated manager for installing, switching between, and
uninstalling custom Wine builds. Accessible from both the main `mythix-build` menu
and winetoolz's Wine category.

**Install location:** `~/.local/share/mythix-wine-installs/<name>/`
**Symlink directory:** `~/.local/bin/`

#### Features

| Action | Description |
|--------|-------------|
| **Install** | Install a Wine build from wine-builder output, a local directory, or a tarball (.tar.gz/.xz/.zst/.bz2) |
| **List** | Show all managed installations with version, install date, and active status |
| **Switch** | Set which Wine build is the system default (swaps symlinks in `~/.local/bin/`) |
| **Uninstall** | Remove one or more builds — cleans up symlinks and install directory |
| **Info** | Disk usage, version, source, architecture, contents breakdown |

#### How It Works

1. **Install** copies (via rsync) or extracts the Wine build into its own directory
   under `~/.local/share/mythix-wine-installs/`.
2. A `.mythix-meta` metadata file is written with version, source, and install date.
3. **Switch** creates symlinks (`wine`, `wine64`, `wineserver`, `wineboot`, `winecfg`,
   etc.) in `~/.local/bin/` pointing to the active install.
4. Only symlinks managed by mythix-build are touched — existing system Wine is never modified.
5. **Uninstall** removes the install directory and cleans up any symlinks pointing to it.

#### Source Detection

When installing from wine-builder output, the manager scans:

- `~/.local/share/mythix-wine_builder/buildz/install/`
- `~/wine-custom/buildz/`

Any directory containing `bin/wine` or `bin/wine64` is a valid source.

---

## Project Layout

```
mythix-build/
├── mythix-build.sh                              Main launcher menu (fzf or numbered)
├── Makefile                                    Install / uninstall
├── pyproject.toml                              Python package config (hatchling)
├── README.md
│
├── mythix_build_system/                                Python frontend package
│   ├── __init__.py                             Package root + version
│   ├── __main__.py                             python -m mythix_build_system entry
│   ├── cli.py                                  Click CLI (mythix command)
│   ├── core/
│   │   ├── __init__.py                         Re-exports: TOOLS, find_tool, run_tool
│   │   ├── tools.py                            Tool dataclass + catalogue (9 tools)
│   │   ├── discovery.py                        find_tool() / resolve_root()
│   │   └── runner.py                           Subprocess launcher with TTY handoff
│   ├── tui/
│   │   ├── __init__.py                         Re-exports LauncherApp
│   │   ├── app.py                              Textual LauncherApp (banner, OptionList)
│   │   ├── build_screen.py                     Live build progress screen
│   │   ├── build_runner.py                     Async subprocess + output streaming
│   │   └── progress.py                         Stage-marker parser (==> / ── / ✓)
│   └── gui/
│       ├── __init__.py                         Re-exports main
│       ├── __main__.py                         python -m mythix_build_system.gui entry
│       └── app.py                              Tkinter GUI launcher
│
├── tests/
│   ├── conftest.py                             Shared fixtures (fake tool trees)
│   ├── test_tools.py                           Tool catalogue tests
│   ├── test_discovery.py                       find_tool / resolve_root tests
│   ├── test_cli.py                             Click CLI tests
│   ├── test_tui.py                             Textual TUI tests
│   ├── test_build_screen.py                    BuildScreen tests
│   └── test_progress.py                        Progress parser tests
│
├── mythix-wine_builder/
│   ├── wine-builder.sh                         Entry point (CLI + interactive)
│   ├── wine-build-core.sh                      Build engine
│   ├── build-32.sh                             32-bit compile wrapper
│   ├── build-64.sh                             64-bit compile wrapper
│   ├── helper.sh                               Utility functions
│   ├── install-from-build.sh                   Post-build installation
│   ├── wine-tkg-patcher.sh                     TKG patch applicator
│   ├── _output_common.sh                       Shared output helpers (msg/ok/err/sep)
│   ├── customization.cfg                       Build config (compiler flags, configure args)
│   ├── Containerfile                           Container build environment
│   ├── patches/                                Drop .patch/.diff files here
│   └── deps-tkg                                TKG dependency list
│
├── mythix-neutron_builder/
│   ├── neutron-builder.sh                      Entry point
│   ├── neutron-build-core.sh                   Build engine
│   ├── neutron-dxvk-build.sh                   DXVK builder (meson/ninja, static MinGW)
│   ├── neutron-vkd3d-build.sh                  VKD3D-Proton builder
│   ├── neutron-package.sh                      Steam package assembler (launcher, bootstrap,
│   │                                           toolmanifest, dxvk.conf, Steam components)
│   ├── neutron-patcher.sh                      Patch system engine (fzf multi-picker)
│   ├── _output_common.sh                       → symlink to ../mythix-wine_builder/
│   ├── dxvk.conf                               Shipped DXVK config (async, state cache)
│   ├── vkd3d-proton.conf                       Shipped VKD3D-Proton config reference
│   ├── spinner.sh                              Progress animation
│   ├── ntsync.h                                Kernel header for ntsync support
│   ├── Containerfile.neutron                    Container build environment
│   ├── neutron-customization.cfg               Build config
│   ├── patches/                                User patch groups (add your own)
│   │   └── custom/                             Empty template — drop .patch files here
│   └── deps-neutron-tkg                        TKG dependency list
│
├── mythix-proton_builder/
│   └── proton-builder.sh                       Delegated build wrapper (GE / TKG)
│
├── mythix-neutron-install/
│   └── neutron-install.sh                      Neutron package deployer
│
├── mythix-proton-install/
│   └── proton-install.sh                       Proton downloader / deployer
│
├── mythix-wine-proton_hybrid_builder/
│   ├── wine-proton_hybrid-v1.0.0.sh            Hybrid installer (Proton base)
│   └── buildz/                                 Build output (local mode)
│
├── mythix-wine-neutron_hybrid_builder/
│   └── wine-neutron_hybrid-v1.0.0.sh           Hybrid installer (Neutron base)
│
└── mythix-winetoolz/
    ├── wine_toolz.sh                           Main launcher (zenity GUI)
    └── modules/
        ├── winetoolz-lib.sh                    Shared library (dialogs, Wine resolution)
        └── shared_lib/
            ├── about.sh                        Help / dependency checker
            ├── app_launcher.sh                 App shortcut manager
            ├── directx_installer-local.sh      DirectX Jun 2010 installer
            ├── dll_installer.sh                Component DLL installer
            ├── dll_override_manager.sh         DLL override management
            ├── dxvk_setup-gui.sh               DXVK install/uninstall
            ├── env_flags.sh                    Environment variable profiles
            ├── install_components-x86_64.sh    System component installer
            ├── install_nvapi.sh                DXVK-NVAPI installer
            ├── log_viewer.sh                   Wine stderr log capture
            ├── prefix_diagnostics.sh           Prefix health checker
            ├── prefix_manager.sh               Prefix backup/restore
            ├── runtime_manager.sh              Runtime version manager
            ├── runtimes_installer.sh           .NET/VC++/XNA installer
            ├── setup_vkd3d_proton-gui.sh       VKD3D-Proton installer
            ├── vcruntime_installer-gui.sh      Visual C++ runtime installer
            ├── wine_install_manager.sh         Wine Install Manager
            ├── wine_tools.sh                   Wine control/regedit/explorer
            └── winetoolz-prefix-maker.sh       Prefix creation wizard
```

---

## Data Directories

Everything lives under your home directory. Nothing touches system paths unless you
explicitly `make install PREFIX=/usr/local`.

### Install Layout (after `make install`)

```
~/.local/
├── bin/
│   ├── mythix-build             Main menu
│   ├── wine-builder            Wine builder
│   ├── neutron-builder         Neutron builder
│   ├── proton-builder          Delegated Proton builder (GE / TKG)
│   ├── neutron-install         Neutron package deployer
│   ├── proton-install          Proton downloader / deployer
│   ├── wine_toolz              winetoolz GUI
│   ├── wine-proton_hybrid      Hybrid installer (Proton base)
│   ├── wine-neutron_hybrid     Hybrid installer (Neutron base)
│   └── wine_install_mgr        Wine Install Manager (standalone entry point)
└── lib/
    ├── mythix-neutron_builder/  Engine scripts + patches/
    ├── mythix-wine_builder/     Engine scripts + patches/
    ├── mythix-neutron-install/  neutron-install script
    ├── mythix-proton-install/   proton-install script
    ├── mythix-wine-proton_hybrid_builder/
    ├── mythix-wine-neutron_hybrid_builder/
    └── mythix-winetoolz/        Modules + shared_lib/
```

### Runtime Data (created by scripts on first use)

```
~/.local/share/
├── mythix-wine_builder/
│   ├── buildz/
│   │   ├── install/            Completed Wine builds
│   │   └── build-run/          In-progress build trees
│   └── src/                    Git-cloned Wine sources
│
├── mythix-neutron_builder/
│   ├── buildz/
│   │   ├── install/            Completed Neutron packages
│   │   └── build-run/          In-progress builds
│   └── src/                    Git clones (proton-wine, dxvk, vkd3d-proton)
│
├── mythix-proton_builder/
│   ├── buildz/
│   │   └── install/            Completed Proton builds (GE / TKG)
│   └── src/                    Git clones (proton-ge-custom, wine-tkg-git)
│
├── mythix-neutron-installs/     Managed Neutron installations (neutron-install)
│   ├── my-neutron/
│   │   ├── files/              Wine build + DXVK + VKD3D + Steam components
│   │   ├── neutron             Python launcher
│   │   ├── compatibilitytool.vdf
│   │   ├── toolmanifest.vdf
│   │   └── .mythix-meta
│   └── ...
│
└── mythix-wine-installs/        Managed Wine installations (Wine Install Manager)
    ├── wine-staging-10.5/
    │   ├── bin/wine
    │   ├── lib/ lib64/ share/
    │   └── .mythix-meta         Version, source, install date
    └── wine-tkg-custom/
        └── ...
```

### Configuration

```
~/.config/mythix-build/
├── customization.cfg           wine-builder compile flags (never overwritten on reinstall)
├── neutron-customization.cfg   neutron-builder compile flags
└── winetoolz.cfg               winetoolz preferences

~/.config/winetoolz/
├── launchers/                  App launcher shortcut configs
├── env_profiles/               Environment variable profiles (.env)
└── icons/                      Cached .exe icons (PNG)
```

### Other Runtime Directories

```
~/wine-custom/buildz/           Runtime manager downloads (GE-Proton, Kron4ek, etc.)
~/winetoolz/
├── backups/                    Prefix backup archives
├── logs/                       Wine stderr capture logs
└── dxvk-vkd3d_proton-files/   Downloaded DXVK/VKD3D-Proton releases
```

---

## Building with Containers (Recommended)

**Containers are the recommended way to build Wine and Neutron.** They provide a
clean, reproducible Ubuntu 24.04 environment with every dependency pre-installed —
no risk of polluting your host system with hundreds of dev packages.

**proton-builder** also uses containers, but differently — GE-Proton and proton-tkg
bring their *own* container environments. proton-builder just needs a container engine
installed (Podman or Docker) and the upstream scripts handle the rest.

The wine-builder and neutron-builder ship their own Containerfiles:

| Builder | Containerfile | Image name |
|---------|--------------|------------|
| wine-builder | `mythix-wine_builder/Containerfile` | `wine-builder` |
| neutron-builder | `mythix-neutron_builder/Containerfile.neutron` | `mythix-neutron_builder` |

Podman (rootless) is strongly recommended. Docker works too — just drop the `:z`
volume flags if SELinux is not in use.

### Wine Builder Container

**Build the image** (once):

```bash
cd mythix-wine_builder

podman build \
    --build-arg BUILD_USER="$(whoami)" \
    --build-arg BUILD_UID="$(id -u)" \
    --build-arg BUILD_GID="$(id -g)" \
    -t wine-builder .
```

**Run a build:**

```bash
podman run --rm -it \
    -v "$(pwd)":/home/"$(whoami)"/wine-builder:z \
    -v "${HOME}/wine-builds":/home/"$(whoami)"/wine-builds:z \
    wine-builder \
    bash wine-builder.sh --source staging
```

### Neutron Builder Container

**Build the image** (once):

```bash
cd mythix-neutron_builder

podman build \
    --build-arg BUILD_USER="$(whoami)" \
    --build-arg BUILD_UID="$(id -u)" \
    --build-arg BUILD_GID="$(id -g)" \
    -t mythix-neutron_builder \
    -f Containerfile.neutron .
```

**Run a build:**

```bash
podman run --rm -it \
    -v "$(pwd)":/home/"$(whoami)"/mythix-neutron_builder:z \
    -v mythix-neutron_builder-ccache:/home/"$(whoami)"/.ccache:z \
    mythix-neutron_builder \
    bash neutron-builder.sh --source proton-wine
```

### Container Notes

- **Build args** (`BUILD_USER`, `BUILD_UID`, `BUILD_GID`) match your host user so
  bind-mounted files have correct ownership — no root permission headaches.
- **ccache volume** — the neutron container example uses a named volume for ccache.
  This persists across runs so rebuilds are dramatically faster.
- **Image size** is ~5–6 GB. Only rebuild when the Containerfile changes.
- **ntsync header** — the neutron container includes a bundled `ntsync.h` (Linux 6.14+
  kernel header) since Ubuntu 24.04's `linux-libc-dev` predates ntsync. No manual
  download needed. Configure detects it automatically and compiles ntsync support in.
- All interactive features (fzf pickers, version selectors) work inside the container
  as long as you pass `-it` for an interactive TTY.

### Installing Podman

```bash
sudo apt install podman            # Debian / Ubuntu
sudo dnf install podman            # Fedora
sudo pacman -S podman              # Arch
```

---

## Requirements

> **Using containers?** Skip straight to
> [Building with Containers](#building-with-containers-recommended) — the
> Containerfiles handle all build dependencies. The requirements below are only
> needed for native (host) builds.

### Core

| Tool | Used by | Notes |
|------|---------|-------|
| `bash` 4.4+ | All | Required |
| `git` | wine-builder, neutron-builder, proton-builder | Source fetching |
| `zenity` | winetoolz, Wine Install Manager | GUI dialogs |
| `fzf` | All builders, main menu | Optional — nicer interactive pickers |

### Build Dependencies

| Tool | Used by |
|------|---------|
| `make`, `gcc`, `g++`, `autoconf`, `automake` | wine-builder, neutron-builder |
| `pkg-config` | wine-builder, neutron-builder |
| `i686-linux-gnu-gcc`, `i686-linux-gnu-g++` | 32-bit Wine builds |
| `x86_64-w64-mingw32-gcc`, `x86_64-w64-mingw32-g++` | MinGW cross-compile (PE DLLs) |
| `meson`, `ninja` | neutron-builder (DXVK / VKD3D-Proton) |
| `glslangValidator` | neutron-builder (DXVK / VKD3D-Proton) |
| `ccache` | Optional — strongly recommended for rebuilds |

### Runtime Dependencies

| Tool | Used by |
|------|---------|
| `rsync` | wine-proton_hybrid, Wine Install Manager |
| `python3` | neutron launcher, wine-proton_hybrid, Python frontend |
| `curl` | winetoolz, proton-install, proton-builder (GitHub release downloads) |
| `tar`, `zstd` | winetoolz, proton-install (archive extraction) |
| `podman` or `docker` | proton-builder (container engine for delegated builds) |

### Python Frontend Dependencies

| Package | Version | Used by |
|---------|---------|---------|
| `textual` | ≥ 0.80 | TUI launcher |
| `click` | ≥ 8.1 | CLI (`mythix` command) |
| `tkinter` | (stdlib) | GUI launcher (usually pre-installed) |
| `pytest` | ≥ 8.0 | Test suite (dev only) |
| `pytest-asyncio` | ≥ 0.23 | Async test support (dev only) |

Install with `pip install -e .` or `make py-dev` (which also installs dev deps).

### One-Liner (Debian/Ubuntu)

```bash
sudo apt install git make gcc g++ autoconf automake pkg-config \
    gcc-i686-linux-gnu g++-i686-linux-gnu \
    gcc-mingw-w64 g++-mingw-w64 \
    meson ninja-build glslang-tools \
    rsync python3 curl ccache fzf zenity zstd \
    python3-tk python3-pip
```

---

## Uninstall

```bash
make uninstall
```

This removes all installed scripts and lib directories. You'll be asked whether to
clean up `~/.bashrc` entries.

**Config files** in `~/.config/mythix-build/` are **kept** — remove manually if wanted.

**Build output** in `~/.local/share/mythix-wine_builder/`,
`~/.local/share/mythix-neutron_builder/`, and
`~/.local/share/mythix-proton_builder/` is **kept** — these are your compiled builds.

**Managed Wine installs** in `~/.local/share/mythix-wine-installs/` are **kept** —
use the Wine Install Manager to remove individual builds first, or delete the
directory manually.

**Python frontend:** `pip uninstall mythix-build` removes the Python package and the
`mythix` console script.

---

*Made by Mythix with steel, grit, and hard work beyond measure*
