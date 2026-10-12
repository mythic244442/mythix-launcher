## ◈ Mythix - Built for all ◈

Mythix Launcher v1.1.1 - controller navigation, runtime & polish update

---

## What is the Mythix Launcher?

Mythix is a native Linux game launcher and compatibility toolkit built from the ground up. It pairs a custom runtime in the form of Neutron, a full-featured brand-new compatibility tool that handles prefix management, game configuration, and runtime detection — built for all to be able to enjoy and configure your games library with ease, while maintaining the depth and complexity of a true games launcher geared towards runtime customization and having the tools necessary to get games running the way you want them to.

## What's new in v1.1.1

v1.1.0 brought native PlayStation controller support to Linux. v1.1.1 finishes the job on the launcher side: the whole UI is now properly usable with nothing but a controller — including typing — and forced Steam Linux Runtime launches finally work with Neutron. On top of that you get per-game wrapper commands and a round of fixes.

---

## Mythix Launcher

### Controller navigation overhaul
- **One step per press** — d-pad and stick navigation no longer skips entries. Every input is now handled exactly once, with clean auto-repeat when you hold a direction
- **LB / RB section switching** — bumpers cycle Library → Import → Settings → Tools, while the d-pad and stick stay inside the content area and never wander onto the side rail or bottom bar
- **On-screen keyboard** — every text field (search, paths, env vars, launch arguments) can be typed into with the controller. No keyboard needed, ever
- **Controller option picker** — dropdowns open as a big, controller-friendly list
- **Smarter focus** — remembers where you were in each section, recovers gracefully when the library changes, and lands on your games when you switch sections
- **More controls** — LT / RT page up and down, the right stick scrolls, Back/View jumps straight to search with the keyboard open, Start opens the focused game's config, X opens Quick Settings, Y stops the running game, and Guide hides/shows the launcher from anywhere
- **Focus-aware input** — controller presses are ignored while your game has focus, so in-game button mashing can't click things in the launcher behind it

### Steam Linux Runtime fix
- **Forced runtime now works with Neutron** — launching inside the Steam Linux Runtime (Soldier, Sniper or SteamRT4) no longer dies with `wine: could not load ntdll.so`
- **Runtime library bridge** — before a container launch, the launcher checks which libraries your Wine build needs that the runtime doesn't ship (like `libunwind` and `libxkbregistry`) and safely brings your system's copies into the container. Libraries the runtime already has are never touched
- **Runtime Library Bridge** section in the launch debug dump shows exactly what was bridged and anything still missing. Set `MYTHIX_RUNTIME_BRIDGE=0` to turn it off
- **Per-game runtime setting is in charge** — games launched without the container stay outside it, even with Neutron builds that would otherwise wrap themselves into Steam's runtime

### New
- **Wrapper commands** — a new per-game "Wrapper command" field (Config → Launch Arguments) to put tools like `prime-run`, `mangohud` or `gamescope -f --` in front of the launch. Wrappers run on your system, so they work with the runtime container too

### Fixes
- **Windowed-mode nav dropdown** is no longer cut off behind the library content
- **Runtime shim** is kept up to date automatically instead of only being written once

---

## Neutron builder changes

These apply to the next Neutron builds:

- **Portable builds** — Neutron is now built for a common CPU baseline (`x86-64-v3`: Intel Haswell 2013+ / AMD Zen and newer) instead of only the build machine's CPU. Earlier builds could crash at startup with an "illegal instruction" error on some CPUs — if that was you, the next build fixes it
- **Built without libunwind**, the same way Valve builds Proton, so Neutron runs cleanly inside the Steam Linux Runtime on its own
- **Prefix templates** can now be created from inside the runtime container

---

**Portable (AppImage)**

Download `mythix_1.1.1_amd64.AppImage`, extract, and run the binary directly — no installation required.

First, navigate to the directory containing the app image:
```
cd /path/to/app_image
```
Then, make it executable:
```
chmod u+x mythix_1.1.1_amd64.AppImage
```
Then, run the app image using either this or just clicking on the executable itself:
```
./mythix_1.1.1_amd64.AppImage
```
Or alternatively, you can download the .zip archive containing the app image's source version of the launcher and extract it wherever you want and run from the launch wrapper.

First, navigate to the directory containing the app image's files:
```
cd /path/to/mythix_1.1.1_amd64_AppImage
```
Then, run the launcher with the wrapper:
```
./AppRun
```
To get a more diagnostic and verbose launch, use this (optional):
```
bash +x -v ./AppRun
```
Or you could just run the launcher's binary directly by running:
```
./usr/bin/mythix-launcher
```
Just download the `mythix_1.1.1_amd64_AppImage.zip`, extract it, and launch using either set of commands above. Or alternatively, you can just click on the binary itself, which works perfectly fine as well.

**Debian / Ubuntu / Mint (.deb)**
Just download the .deb file and install by running:
```
sudo dpkg -i /path/to/mythix_1.1.1_amd64.deb
```
Or alternatively, you can download the .zip archive containing a debian version of the launcher and extract it wherever you want and run from the mythix-launcher binary by running:
```
cd /path/to/mythix_1.1.1_amd64
```
Then, run the launcher's binary directly by running:
```
./data/usr/bin/mythix-launcher
```
Just download the `mythix_1.1.1_debian_amd64.zip`, extract it, and launch using the command above. Or alternatively, you can just click on the binary itself, which works perfectly fine as well.

**Fedora / openSUSE (.rpm)**
Just download the .rpm file and install by running:
```
sudo rpm -i /path/to/mythix-1.1.1-1.x86_64.rpm
```
**Neutron (optional)**

Download `Neutron-11.0-21199.zip` and extract to your compatibility tools directory (e.g. `~/.steam/steam/compatibilitytools.d/`). If you don't use Steam, set a custom tools directory in the launcher's Settings. The launcher auto-detects installed tools in the custom tools directory upon restarting the launcher.

---

- Built with steel, grit, and hard work beyond measure, by Luna & CC :3
