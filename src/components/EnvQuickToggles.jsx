import React, { useState } from "react";

const PRESETS = [
  {
    category: "Sync",
    vars: [
      { key: "WINEFSYNC",       label: "Fsync",       value: "1", desc: "Futex-based sync (fast, needs kernel support)" },
      { key: "WINEESYNC",       label: "Esync",       value: "1", desc: "Eventfd-based sync (wider compat, slower than fsync)" },
      { key: "WINENTSYNC",      label: "NTSync",      value: "1", desc: "NT synchronization primitives (newest, fastest)" },
    ],
  },
  {
    category: "Neutron",
    vars: [
      { key: "NEUTRON_FSR",           label: "FSR",            value: "auto", desc: "AMD FidelityFX Super Resolution upscaling" },
      { key: "NEUTRON_FPS_LIMIT",     label: "FPS Limit",      value: "60",   desc: "Cap framerate (set custom value as needed)" },
      { key: "NEUTRON_GAMEMODE",      label: "GameMode",       value: "auto", desc: "Feral GameMode CPU governor optimization" },
      { key: "NEUTRON_GAMESCOPE",     label: "Gamescope",      value: "auto", desc: "SteamOS compositor session (HDR, FSR, VRR)" },
      { key: "NEUTRON_DXVK_ASYNC",    label: "DXVK Async",     value: "1",    desc: "Async shader compilation (reduces stutter)" },
      { key: "NEUTRON_DEBUG",         label: "Debug Log",      value: "1",    desc: "Enable Neutron debug logging" },
      { key: "NEUTRON_BATTLEYE_RUNTIME", label: "BattlEye",    value: "auto", desc: "BattlEye anti-cheat runtime" },
      { key: "NEUTRON_EAC_RUNTIME",   label: "EasyAntiCheat",  value: "auto", desc: "EAC anti-cheat runtime" },
    ],
  },
  {
    category: "DXVK / VKD3D",
    vars: [
      { key: "DXVK_HUD",             label: "DXVK HUD",       value: "1",    desc: "Show DXVK overlay (fps, devinfo, gpuload)" },
      { key: "DXVK_ASYNC",           label: "DXVK Async",     value: "1",    desc: "Async pipeline compilation (standalone DXVK)" },
      { key: "DXVK_STATE_CACHE",     label: "State Cache",    value: "1",    desc: "DXVK shader state cache on disk" },
      { key: "DXVK_LOG_LEVEL",       label: "DXVK Log",       value: "info", desc: "DXVK log level (none/error/warn/info/debug)" },
      { key: "DXVK_CONFIG_FILE",     label: "DXVK Config",    value: "",     desc: "Path to dxvk.conf for per-game tuning" },
      { key: "DXVK_FRAME_RATE",      label: "DXVK FPS Cap",   value: "60",   desc: "DXVK-level framerate limiter" },
      { key: "DXVK_HDR",             label: "DXVK HDR",       value: "1",    desc: "HDR output passthrough" },
      { key: "DXVK_ENABLE_NVAPI",    label: "NVAPI",          value: "1",    desc: "NVIDIA API support (DLSS, Reflex)" },
      { key: "VKD3D_CONFIG",         label: "VKD3D Config",   value: "",     desc: "VKD3D-proton config flags (dxr, force_static_cbv)" },
    ],
  },
  {
    category: "Proton / Wine",
    vars: [
      { key: "PROTON_USE_WINED3D",     label: "WineD3D",        value: "1",    desc: "Use OpenGL WineD3D instead of Vulkan DXVK" },
      { key: "PROTON_NO_ESYNC",        label: "No Esync",       value: "1",    desc: "Disable Esync in Proton" },
      { key: "PROTON_NO_FSYNC",        label: "No Fsync",       value: "1",    desc: "Disable Fsync in Proton" },
      { key: "PROTON_ENABLE_NVAPI",    label: "Proton NVAPI",   value: "1",    desc: "Enable NVIDIA API in Proton (DLSS)" },
      { key: "WINE_FULLSCREEN_FSR",    label: "Wine FSR",       value: "1",    desc: "Wine's built-in FSR upscaling" },
      { key: "WINE_LARGE_ADDRESS_AWARE", label: "Large Address", value: "1",   desc: "4GB address space for 32-bit games" },
      { key: "WINEDEBUG",             label: "Wine Debug",     value: "-all", desc: "Wine debug channels (-all to silence)" },
      { key: "WINEDLLOVERRIDES",      label: "DLL Overrides",  value: "",     desc: "Force native/builtin DLLs (e.g. d3d11=n)" },
      { key: "WINEPREFIX",            label: "Prefix Override", value: "",    desc: "Override Wine prefix path" },
    ],
  },
  {
    category: "GPU / Display",
    vars: [
      { key: "DRI_PRIME",             label: "DRI Prime",      value: "1",    desc: "Use discrete GPU (hybrid graphics laptops)" },
      { key: "__GL_SHADER_DISK_CACHE", label: "GL Shader Cache", value: "1",  desc: "NVIDIA OpenGL shader disk cache" },
      { key: "RADV_PERFTEST",         label: "RADV Perftest",  value: "",     desc: "AMD RADV flags (gpl,ngg,sam,rt)" },
      { key: "MESA_GL_VERSION_OVERRIDE", label: "Mesa GL Ver",  value: "4.6", desc: "Override reported OpenGL version" },
      { key: "ENABLE_VKBASALT",       label: "vkBasalt",       value: "1",    desc: "Vulkan post-processing layer (ReShade-like)" },
      { key: "MANGOHUD",              label: "MangoHud",       value: "1",    desc: "MangoHud performance overlay" },
      { key: "MANGOHUD_DLSYM",        label: "MangoHud DLSYM", value: "1",    desc: "MangoHud OpenGL via dlsym hook" },
    ],
  },
  {
    category: "Mythix Launcher",
    vars: [
      { key: "MYTHIX_PRIME_RENDER",        label: "Prime Render",  value: "1", desc: "Use offload rendering for discrete GPU" },
      { key: "MYTHIX_CAPTURE_GAME_OUTPUT",  label: "Capture Output", value: "1", desc: "Capture game stdout/stderr to log" },
      { key: "MYTHIX_TEE_GAME_OUTPUT",      label: "Tee Output",   value: "1", desc: "Tee game output to terminal and log" },
    ],
  },
];

export default function EnvQuickToggles({ envVars, onChange }) {
  const [expanded, setExpanded] = useState(null);

  function toggle(key, value) {
    const next = { ...envVars };
    if (key in next) {
      delete next[key];
    } else {
      next[key] = value;
    }
    onChange(next);
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10, marginBottom: 16 }}>
      {PRESETS.map(({ category, vars }) => {
        const isOpen = expanded === category;
        const activeCount = vars.filter((v) => v.key in envVars).length;

        return (
          <div key={category}>
            <button
              onClick={() => setExpanded(isOpen ? null : category)}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                gap: 8,
                padding: "6px 8px",
                background: "var(--bg-3)",
                border: "1px solid var(--border)",
                borderRadius: "var(--r-sm)",
                color: "var(--text-1)",
                cursor: "pointer",
                fontFamily: "var(--font-mono)",
                fontSize: 10,
                fontWeight: 700,
                letterSpacing: "0.06em",
                textTransform: "uppercase",
                transition: "all 0.12s",
              }}
            >
              <span style={{
                fontSize: 9,
                transition: "transform 0.15s",
                transform: isOpen ? "rotate(90deg)" : "rotate(0deg)",
              }}>▶</span>
              <span style={{ flex: 1, textAlign: "left" }}>{category}</span>
              {activeCount > 0 && (
                <span style={{
                  background: "var(--lime-glow)",
                  color: "var(--lime)",
                  borderRadius: 8,
                  padding: "1px 6px",
                  fontSize: 9,
                }}>{activeCount}</span>
              )}
            </button>

            {isOpen && (
              <div style={{
                display: "grid",
                gridTemplateColumns: "repeat(2, 1fr)",
                gap: 4,
                padding: "6px 0 0",
              }}>
                {vars.map(({ key, label, value, desc }) => {
                  const isActive = key in envVars;
                  const displayVal = isActive ? envVars[key] : value;

                  return (
                    <button
                      key={key}
                      onClick={() => toggle(key, value)}
                      title={`${key}=${displayVal}\n${desc}`}
                      style={{
                        padding: "5px 8px",
                        fontSize: 11,
                        fontFamily: "var(--font-mono)",
                        borderRadius: "var(--r-sm)",
                        border: "1px solid",
                        cursor: "pointer",
                        transition: "all 0.12s",
                        background: isActive ? "rgba(180, 167, 214, 0.15)" : "var(--bg-3)",
                        borderColor: isActive ? "var(--lime)" : "var(--border)",
                        color: isActive ? "var(--lime)" : "var(--text-1)",
                        textAlign: "left",
                        display: "flex",
                        alignItems: "center",
                        gap: 6,
                      }}
                    >
                      <span style={{ fontSize: 10 }}>{isActive ? "✓" : "○"}</span>
                      <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{label}</span>
                      {displayVal && (
                        <span style={{
                          fontSize: 9,
                          color: isActive ? "var(--lime-dim)" : "var(--text-3)",
                          flexShrink: 0,
                        }}>{displayVal}</span>
                      )}
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
