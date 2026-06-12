import React, { useEffect, useState } from "react";
import { useRuntime, DEFAULT_VARIANT } from "../hooks/useRuntime.js";
import RuntimeStatusBadge from "./RuntimeStatusBadge.jsx";
import { api } from "../tauri.js";

const VERB_OPTIONS = [
  { value: "waitforexitandrun", label: "Wait for exit and run" },
  { value: "run", label: "Run (don't wait)" },
  { value: "runinprefix", label: "Run in prefix" },
];

export default function SettingsView() {
  const { runtimes, isInstalled, installing, install } = useRuntime();
  const defaultRt = runtimes.find((r) => r.variant === DEFAULT_VARIANT);

  const [settings, setSettings]   = useState(null);
  const [dirty, setDirty]         = useState(false);
  const [saveMsg, setSaveMsg]     = useState(null);

  useEffect(() => {
    api.getSettings().then((s) => setSettings(s)).catch(() => {});
  }, []);

  function patch(key, value) {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setDirty(true);
    setSaveMsg(null);
  }

  async function saveGlobal() {
    if (!settings) return;
    setSaveMsg(null);
    try {
      await api.saveSettings(settings);
      setDirty(false);
      setSaveMsg({ kind: "ok", text: "Saved" });
    } catch (e) {
      setSaveMsg({ kind: "err", text: String(e) });
    }
  }

  return (
    <div className="content" style={{ overflow:"hidden", display:"flex", flexDirection:"column" }}>
      <div className="content-header">
        <span className="content-header__title">Settings</span>
      </div>
      <div style={{ flex:1, overflowY:"auto", padding:"20px 24px", maxWidth:640 }}>

        {/* SLR status */}
        <div style={{
          padding:"16px 18px", background:"var(--bg-2)",
          border:"1px solid var(--border)",
          borderLeft: isInstalled ? "3px solid var(--lime)" : "3px solid var(--amber)",
          borderRadius:"var(--r-lg)", marginBottom:24,
          display:"flex", alignItems:"center", gap:16,
        }}>
          <div style={{ flex:1 }}>
            <div style={{ fontFamily:"var(--font-display)", fontWeight:700, fontSize:14, marginBottom:4 }}>
              Steam Linux Runtime
            </div>
            <div style={{ fontSize:12, color:"var(--text-2)" }}>
              {isInstalled
                ? `Installed — build ${defaultRt?.build_id || "unknown"}`
                : "Not installed — games may not launch correctly"}
            </div>
          </div>
          <RuntimeStatusBadge runtime={defaultRt} />
          {!isInstalled && (
            <button className="btn btn--primary btn--sm"
              onClick={() => install(DEFAULT_VARIANT)} disabled={installing}>
              Install
            </button>
          )}
        </div>

        {/* Global defaults */}
        {settings && (
          <>
            <SectionLabel>Defaults</SectionLabel>

            <SettingRow label="Default prefix root" help="Where new game prefixes are created">
              <input className="form-input form-input--mono" style={{ width:"100%" }}
                value={settings.defaultPrefixRoot || ""}
                onChange={(e) => patch("defaultPrefixRoot", e.target.value)}
                placeholder="~/.mythix/gamedata" />
            </SettingRow>

            <SettingRow label="Default Wine/Proton/Neutron path" help="Auto-selected for new games (leave blank to choose each time)">
              <input className="form-input form-input--mono" style={{ width:"100%" }}
                value={settings.defaultProtonPath || ""}
                onChange={(e) => patch("defaultProtonPath", e.target.value)}
                placeholder="~/.steam/steam/compatibilitytools.d" />
            </SettingRow>

            <SettingRow label="Custom tools directory" help="Extra folder to scan for compatibility tools (Proton, Neutron, Wine). Leave blank to use Steam defaults only.">
              <input className="form-input form-input--mono" style={{ width:"100%" }}
                value={settings.customToolsDir || ""}
                onChange={(e) => patch("customToolsDir", e.target.value)}
                placeholder="/path/to/your/compatibilitytools.d" />
            </SettingRow>

            <SettingRow label="Default launch verb">
              <select className="form-input" style={{ width:220 }}
                value={settings.defaultProtonVerb || "waitforexitandrun"}
                onChange={(e) => patch("defaultProtonVerb", e.target.value)}>
                {VERB_OPTIONS.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
              </select>
            </SettingRow>

            <SettingRow label="Library default view">
              <select className="form-input" style={{ width:120 }}
                value={settings.libraryView || "grid"}
                onChange={(e) => patch("libraryView", e.target.value)}>
                <option value="grid">Grid</option>
                <option value="list">List</option>
              </select>
            </SettingRow>

            <SettingRow label="Auto-setup prefix on launch" help="Create prefix directories automatically when launching a game">
              <Toggle checked={settings.autoSetupPrefix} onChange={(v) => patch("autoSetupPrefix", v)} />
            </SettingRow>

            <div style={{ display:"flex", gap:8, alignItems:"center", marginTop:16, marginBottom:24 }}>
              <button className="btn btn--primary btn--sm" onClick={saveGlobal} disabled={!dirty}>
                Save Settings
              </button>
              {saveMsg && (
                <span style={{ fontSize:11, fontFamily:"var(--font-mono)",
                  color: saveMsg.kind === "err" ? "var(--coral)" : "var(--lime)" }}>
                  {saveMsg.text}
                </span>
              )}
            </div>
          </>
        )}

        {/* Architecture */}
        <SectionLabel>Architecture</SectionLabel>
        {[
          ["Frontend",       "React 18 + Vite 5",             "var(--sky)"],
          ["Desktop shell",  "Tauri v2 (Rust)",               "var(--lime)"],
          ["Env assembly",   "Rust port of original runner",  "var(--lime)"],
          ["VDF parser",     "Custom Rust (vdf.rs)",          "var(--lime)"],
          ["Runtime",        "Steam Linux Runtime (sniper)",  "var(--lime)"],
          ["Persistence",    "JSON → $XDG_DATA_HOME/mythix",  "var(--text-1)"],
        ].map(([label, value, color]) => (
          <div key={label} style={{ display:"flex", justifyContent:"space-between",
            padding:"8px 0", borderBottom:"1px solid var(--border)", fontSize:12 }}>
            <span style={{ color:"var(--text-2)" }}>{label}</span>
            <span style={{ fontFamily:"var(--font-mono)", fontSize:11, color }}>{value}</span>
          </div>
        ))}

      </div>
    </div>
  );
}

function SectionLabel({ children }) {
  return (
    <div style={{ fontFamily:"var(--font-mono)", fontSize:10, fontWeight:700,
      letterSpacing:"0.12em", color:"var(--text-3)", textTransform:"uppercase", marginBottom:12 }}>
      {children}
    </div>
  );
}

function SettingRow({ label, help, children }) {
  return (
    <div style={{ marginBottom:14 }}>
      <div style={{ fontFamily:"var(--font-display)", fontWeight:600, fontSize:13, color:"var(--text-0)", marginBottom:3 }}>
        {label}
      </div>
      {help && <div style={{ fontSize:11, color:"var(--text-3)", marginBottom:6 }}>{help}</div>}
      {children}
    </div>
  );
}

function Toggle({ checked, onChange }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        width: 24, height: 12, borderRadius: 6, border: "none", cursor: "pointer",
        background: checked ? "var(--lime)" : "var(--bg-4)",
        position: "relative", transition: "background 0.2s",
      }}>
      <div style={{
        width: 14, height: 14, borderRadius: 7,
        background: "var(--bg-1)",
        position: "absolute", top: 3,
        left: checked ? 19 : 3,
        transition: "left 0.2s",
      }} />
    </button>
  );
}
