import React, { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { api } from "../tauri.js";
import { useGamepad } from "../hooks/useGamepad.js";

const RUNTIME_VARIANTS = [
  { value: "", label: "Auto" },
  { value: "sniper", label: "Sniper (RT 3.0)" },
  { value: "soldier", label: "Soldier (RT 2.0)" },
  { value: "steamrt4", label: "Heavy (RT 4.0)" },
];

const ENV_TOGGLES = [
  { key: "DXVK_ASYNC", label: "DXVK Async", value: "1" },
  { key: "MANGOHUD", label: "MangoHud", value: "1" },
  { key: "WINEFSYNC", label: "Fsync", value: "1" },
  { key: "WINEESYNC", label: "Esync", value: "1" },
  { key: "WINENTSYNC", label: "NTSync", value: "1" },
  { key: "DXVK_HUD", label: "DXVK HUD", value: "1" },
  { key: "DRI_PRIME", label: "DRI Prime", value: "1" },
  { key: "PROTON_USE_WINED3D", label: "WineD3D", value: "1" },
  { key: "PROTON_ENABLE_NVAPI", label: "NVAPI", value: "1" },
  { key: "WINE_FULLSCREEN_FSR", label: "Wine FSR", value: "1" },
  { key: "WINEDEBUG", label: "Debug", value: "+err,+module" },
];

const ENV_COLS = 2;

export default function QuickSettings({ game, onSave, onClose, onLaunch }) {
  const [tools, setTools] = useState([]);
  const [protonPath, setProtonPath] = useState(game.config?.proton_path || "");
  const [runtimeVariant, setRuntimeVariant] = useState(game.config?.runtime_variant || "");
  const [envOverrides, setEnvOverrides] = useState(game.config?.env_overrides || {});
  // zone: "tool" | "runtime" | "env" | "actions"
  const [zone, setZone] = useState("tool");
  const [envIndex, setEnvIndex] = useState(0);
  const [actionIndex, setActionIndex] = useState(0);

  const stateRef = useRef({});
  useEffect(() => {
    stateRef.current = { zone, envIndex, actionIndex, protonPath, runtimeVariant, envOverrides, tools, game };
  });

  useEffect(() => { api.getCompatTools().then(setTools).catch(() => {}); }, []);

  useEffect(() => {
    setProtonPath(game.config?.proton_path || "");
    setRuntimeVariant(game.config?.runtime_variant || "");
    setEnvOverrides(game.config?.env_overrides || {});
    setZone("tool");
    setEnvIndex(0);
    setActionIndex(0);
  }, [game?.id]);

  function buildConfig() {
    const s = stateRef.current;
    return {
      ...(s.game?.config || game.config),
      proton_path: s.protonPath || null,
      runtime_variant: s.runtimeVariant || null,
      env_overrides: s.envOverrides,
    };
  }

  async function doSave() {
    const s = stateRef.current;
    const cfg = buildConfig();
    await onSave(s.game?.id || game.id, cfg);
  }

  async function handleSaveAndLaunch() {
    const s = stateRef.current;
    await doSave();
    onLaunch(s.game?.id || game.id);
    onClose();
  }

  async function handleSave() {
    await doSave();
    onClose();
  }

  function cycleDropdown(field, dir) {
    const s = stateRef.current;
    if (field === "tool") {
      const options = ["", ...s.tools.map(t => t.path)];
      const idx = options.indexOf(s.protonPath);
      setProtonPath(options[(idx + dir + options.length) % options.length]);
    } else {
      const options = RUNTIME_VARIANTS.map(r => r.value);
      const idx = options.indexOf(s.runtimeVariant);
      setRuntimeVariant(options[(idx + dir + options.length) % options.length]);
    }
  }

  function toggleEnv(i) {
    const t = ENV_TOGGLES[i];
    setEnvOverrides(prev => {
      const next = { ...prev };
      if (t.key in next) delete next[t.key];
      else next[t.key] = t.value;
      return next;
    });
  }

  const handleInput = useCallback((action) => {
    const s = stateRef.current;

    if (action === "b") { onClose(); return; }

    if (s.zone === "tool") {
      if (action === "left") cycleDropdown("tool", -1);
      else if (action === "right") cycleDropdown("tool", 1);
      else if (action === "down") setZone("runtime");
    } else if (s.zone === "runtime") {
      if (action === "left") cycleDropdown("runtime", -1);
      else if (action === "right") cycleDropdown("runtime", 1);
      else if (action === "up") setZone("tool");
      else if (action === "down") { setZone("env"); setEnvIndex(0); }
    } else if (s.zone === "env") {
      const idx = s.envIndex;
      const rows = Math.ceil(ENV_TOGGLES.length / ENV_COLS);
      const row = Math.floor(idx / ENV_COLS);
      const col = idx % ENV_COLS;

      if (action === "a") { toggleEnv(idx); return; }
      if (action === "up") {
        if (row === 0) setZone("runtime");
        else setEnvIndex(idx - ENV_COLS);
      } else if (action === "down") {
        if (row >= rows - 1) { setZone("actions"); setActionIndex(0); }
        else {
          const next = idx + ENV_COLS;
          setEnvIndex(Math.min(next, ENV_TOGGLES.length - 1));
        }
      } else if (action === "left") {
        if (col > 0) setEnvIndex(idx - 1);
      } else if (action === "right") {
        if (col < ENV_COLS - 1 && idx + 1 < ENV_TOGGLES.length) setEnvIndex(idx + 1);
      }
    } else if (s.zone === "actions") {
      if (action === "up") {
        setZone("env");
        setEnvIndex(Math.min(ENV_TOGGLES.length - 1, (Math.ceil(ENV_TOGGLES.length / ENV_COLS) - 1) * ENV_COLS + s.actionIndex));
      }
      else if (action === "left") setActionIndex(0);
      else if (action === "right") setActionIndex(1);
      else if (action === "a") {
        if (s.actionIndex === 0) handleSave();
        else handleSaveAndLaunch();
      }
    }
  }, [onClose, game?.id]);

  useGamepad(handleInput);

  const toolLabel = tools.find(t => t.path === protonPath)?.name || "None";
  const rtLabel = RUNTIME_VARIANTS.find(r => r.value === runtimeVariant)?.label || "Auto";

  return (
    <div className="quick-settings-overlay" onClick={onClose}>
      <div className="quick-settings" onClick={(e) => e.stopPropagation()}>
        <div className="quick-settings__header">
          <span className="quick-settings__title">{game.name}</span>
          <button className="quick-settings__close" onClick={onClose}>✕</button>
        </div>

        <label className="quick-settings__label">Compatibility Tool</label>
        <div className={`quick-settings__field ${zone === "tool" ? "quick-settings__field--focus" : ""}`}>
          <button className="quick-settings__arrow" onClick={() => cycleDropdown("tool", -1)}>◀</button>
          <span className="quick-settings__value">{toolLabel}</span>
          <button className="quick-settings__arrow" onClick={() => cycleDropdown("tool", 1)}>▶</button>
        </div>
        <select className="form-select" value={protonPath} onChange={(e) => setProtonPath(e.target.value)} style={{ marginTop: 4 }}>
          <option value="">— None —</option>
          {tools.map((t) => (<option key={t.path} value={t.path}>{t.name} ({t.kind})</option>))}
        </select>

        <label className="quick-settings__label">Runtime Container</label>
        <div className={`quick-settings__field ${zone === "runtime" ? "quick-settings__field--focus" : ""}`}>
          <button className="quick-settings__arrow" onClick={() => cycleDropdown("runtime", -1)}>◀</button>
          <span className="quick-settings__value">{rtLabel}</span>
          <button className="quick-settings__arrow" onClick={() => cycleDropdown("runtime", 1)}>▶</button>
        </div>
        <select className="form-select" value={runtimeVariant} onChange={(e) => setRuntimeVariant(e.target.value)} style={{ marginTop: 4 }}>
          {RUNTIME_VARIANTS.map((rv) => (<option key={rv.value} value={rv.value}>{rv.label}</option>))}
        </select>

        <label className="quick-settings__label">Quick Toggles</label>
        <div className="quick-settings__toggles">
          {ENV_TOGGLES.map((t, i) => {
            const active = t.key in envOverrides;
            const focused = zone === "env" && envIndex === i;
            return (
              <button
                key={t.key}
                className={`quick-settings__toggle ${active ? "quick-settings__toggle--active" : ""} ${focused ? "quick-settings__toggle--focus" : ""}`}
                onClick={() => toggleEnv(i)}
                title={`${t.key}=${active ? envOverrides[t.key] : t.value}`}
              >
                <span className="quick-settings__toggle-check">{active ? "✓" : "○"}</span>
                <span className="quick-settings__toggle-label">{t.label}</span>
              </button>
            );
          })}
        </div>

        <div className="quick-settings__actions">
          <button
            className={`btn btn--secondary btn--sm ${zone === "actions" && actionIndex === 0 ? "quick-settings__btn--focus" : ""}`}
            onClick={handleSave}
          >Save</button>
          <button
            className={`btn btn--primary btn--sm ${zone === "actions" && actionIndex === 1 ? "quick-settings__btn--focus" : ""}`}
            onClick={handleSaveAndLaunch}
          >▶ Save & Launch</button>
        </div>
      </div>
    </div>
  );
}
