# Changelog

## 1.1.4 — 2026-10-26

### Neutron diagnostics in Compat Tools

- Each Neutron tool in **Compat Tools** now has **Run diag**: the full resolved
  launch configuration Neutron would use (distribution, bundled DXVK/VKD3D-Proton,
  Steam, VR, NVIDIA, Sony controllers, Mono/Gecko, sync backend, prefix, container,
  wrappers and the Neutron-managed environment), without launching a game.
- Rows are colour-coded: problems in coral, optional or expected-missing states in
  amber (e.g. `wine64` on WoW64 builds, an absent `/dev/ntsync`).
- **Copy raw** puts the plain-text report on the clipboard for bug reports.
- Diag runs in the background with a 30-second limit, follows symlinked
  `compatibilitytools.d` folders into the Steam Runtime container, and shows the
  tool's own output when it can't produce a report instead of an empty panel.
- Reads diag output from current and already-installed Neutron builds alike;
  long DLL rows from older builds split correctly too.

### Fixes

- Games saved with a bare numeric Steam ID (e.g. `1091500`) now use that ID for
  `STEAM_COMPAT_APP_ID`, `SteamAppId` and `SteamGameId` instead of a hash of the
  prefix path, so Neutron's steam.exe setup runs for them.

Thanks to Berry for the diag panel.

## 1.1.3 — 2026-10-19

### Runtime setup refresh

- First-run setup now offers Steam Linux Runtime 4 (Heavy) instead of Sniper.
- Updated setup instructions, installation path, and compatibility-tool terminology.
- Refreshed runtime labels in Settings and compatibility-tool configuration.
- Removed the fixed download-size estimate, which varies between runtime releases.

Existing game configurations are unchanged. Games continue to use their compatibility
tool's required runtime; Sniper (Runtime 3) and Soldier (Runtime 2) remain available
in Compat Tools. Installing Heavy does not force older tools to use Runtime 4.
