```
  ╔══════════════════════════════════════════════════════════════════════╗
  ║                                                                    ║
  ║    ███╗   ███╗██╗   ██╗████████╗██╗  ██╗██╗██╗  ██╗               ║
  ║    ████╗ ████║╚██╗ ██╔╝╚══██╔══╝██║  ██║██║╚██╗██╔╝               ║
  ║    ██╔████╔██║ ╚████╔╝    ██║   ███████║██║ ╚███╔╝                ║
  ║    ██║╚██╔╝██║  ╚██╔╝     ██║   ██╔══██║██║ ██╔██╗                ║
  ║    ██║ ╚═╝ ██║   ██║      ██║   ██║  ██║██║██╔╝ ██╗               ║
  ║    ╚═╝     ╚═╝   ╚═╝      ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝               ║
  ║                                                                    ║
  ║                    v1.1.0 — Controller Overhaul                    ║
  ║                                                                    ║
  ╚══════════════════════════════════════════════════════════════════════╝
```

---

## What is Mythix?

Mythix is a native Linux game launcher and compatibility toolkit built from the ground up for controller-first gaming without Steam. It pairs a custom Wine runtime (Mythix Neutron) with a full-featured launcher that handles prefix management, game configuration, and runtime detection — all navigable entirely by gamepad.

## What's new in v1.1.0

This release is a controller overhaul. Mythix now delivers **full native PlayStation controller support on Linux** — something neither upstream Wine nor Valve's Proton currently offer. DualShock 4 and DualSense controllers get correct PS4/PS5 button glyphs, working rumble motors with directional haptics, and LED color control, all running simultaneously through a custom Wine HID pipeline built specifically for Mythix Neutron.

The launcher now detects your controller the moment you plug it in, automatically configures supported games for the correct button icons, and lets you navigate the entire UI without touching a keyboard.

---

```
  ┌──────────────────────────────────────────────────────────────────┐
  │                    ◤ Mythix Launcher ◥                          │
  └──────────────────────────────────────────────────────────────────┘
```

## Mythix Launcher

- **Live controller detection** — hidraw monitoring identifies DualShock 4, DualSense, Xbox, and Switch controllers by VID/PID in real time
- **Auto glyph configuration** — automatically patches game configs for correct button icons (Cyberpunk 2077, RDR2, Elden Ring, Witcher 3)
- **Controller type indicator** — titlebar shows connected controller type with status dot
- **Guide button overlay** — press the PS/Xbox/Home button for navigation hints
- **Gamepad navigation** — browse your library, launch games, and access Quick Settings entirely with a controller
- **Udev rules installer** — one-click install from Settings to grant controller HID access
- **Removed stale env presets** — SDL_GAMECONTROLLERTYPE and MYTHIX_BLOCK_HIDRAW toggles removed; auto-detection handles this now
- **Fixed prefix path** — new prefixes now correctly go to `~/.mythix/gamedata/`

---

```
  ┌──────────────────────────────────────────────────────────────────┐
  │                   ◤ Mythix Neutron 11.0 ◥                       │
  └──────────────────────────────────────────────────────────────────┘
```

## Mythix Neutron 11.0

Mythix Neutron is a complete Wine-based game runtime — not a patch set, not a wrapper, not a script that calls someone else's Proton. It's a full compatibility tool: **2,250 lines of Python engine**, **1,241 Windows DLLs**, **72 unix libraries**, and a **1.7 GB runtime** built from Valve's bleeding-edge Wine with custom modifications you won't find anywhere else. Drop it into `~/.steam/steam/compatibilitytools.d/`, select it in Steam, or point the Mythix Launcher at it — and your games just run.

### The engine

Neutron's Python launcher speaks Steam's full compatibility tool interface (`run`, `waitforexitandrun`, `runinprefix`, `createprefix`, `destroyprefix`, `diag`, and more) while adding features Proton doesn't expose:

- **Template prefixes** — boot Wine once, clone new prefixes via reflink copies. New game? Prefix ready in seconds, not minutes
- **Tracked runtime overlay** — DXVK, VKD3D-Proton, and NVAPI DLLs installed and tracked per-prefix with automatic upgrade detection
- **Config hashing** — re-overlays only when the build or inputs actually change
- **File locking** — concurrent launches don't race the prefix
- **Wine-builtin detection** — runtime overlays never clobber user-set native DLL overrides
- **NVIDIA DLSS/NGX discovery** — automatic `NVIDIA_WINE_DLL_DIR` configuration
- **Prefix versioning** — logged upgrades with downgrade recovery
- **Diagnostics verb** — `diag` dumps prefix state, DLL versions, and runtime info for troubleshooting

### Controller support (NEW)

This is the headline. Plug in a DualShock 4, launch Cyberpunk 2077, and you get the triangle/circle/cross/square button prompts on screen, the lightbar turns blue, rumble hits the correct motor when you dash left or right, and haptics fade naturally with sustained fire instead of cutting out. No env vars to set, no Steam Input to configure, no workarounds.

- **Full native DualShock 4 support** — PS4 glyphs, rumble motors, directional haptics, and LED color control working simultaneously. Not available in upstream Wine or Proton
- **Sony direct rumble shim** — custom xinput layer intercepts Wwise audio engine rumble calls and translates them to native DS4 motor reports via HID, preserving PlayStation glyph detection
- **DualSense support** — haptics translation for both USB and Bluetooth, including DualSense Edge
- **DS4 output report passthrough** — native lightbar, LED, and motor reports forwarded directly to hardware
- **Shipping-clean build** — all diagnostic logging removed

```
  ┌──────────────────────────────────────────────────────────────────┐
  │  DS4 ──► hidraw ──► winebus (HID) ──► game sees VID 054C       │
  │                                        ╰──► PS4 glyphs ✓       │
  │                                                                  │
  │  Wwise ──► XInputSetState ──► DS4_set_state ──► HID 0x05       │
  │                                ╰──► winebus passthrough         │
  │                                      ╰──► DS4 motors spin ✓    │
  └──────────────────────────────────────────────────────────────────┘
```

---

```
  ┌──────────────────────────────────────────────────────────────────┐
  │                        ◤ Install ◥                              │
  └──────────────────────────────────────────────────────────────────┘
```

## Install

**Debian / Ubuntu / Mint (.deb)**
```
sudo dpkg -i mythix_1.1.0_amd64.deb
```

**Fedora / openSUSE (.rpm)**
```
sudo rpm -i mythix-1.1.0-1.x86_64.rpm
```

**Portable (AppImage)**

Download `mythix-app_image.zip`, extract, and run the binary directly — no installation required.
```
unzip mythix-app_image.zip
chmod +x mythix-app_image/usr/bin/mythix-launcher
./mythix-app_image/usr/bin/mythix-launcher
```

**Neutron (optional)**

Download `Neutron-11.0-21199.zip` and extract to your compatibility tools directory (e.g. `~/.steam/steam/compatibilitytools.d/`). If you don't use Steam, set a custom tools directory in the launcher's Settings. The launcher auto-detects installed tools on startup.

Additional Mythix Wine builds are available at [mythix-tools](https://github.com/mythic244442/mythix-tools).

---

```
  ╔══════════════════════════════════════════════════════════════════╗
  ║  Built with steel, grit, and caffeine by Luna & CC    ◈ 2026   ║
  ╚══════════════════════════════════════════════════════════════════╝
```
