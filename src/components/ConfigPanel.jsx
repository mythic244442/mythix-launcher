import React, { useState, useEffect } from "react";
import { api, pickDirectory, pickFile } from "../tauri.js";
import { useCoverUrl, invalidateCover } from "../hooks/useCoverUrl.js";
import EnvVarsEditor from "./EnvVarsEditor.jsx";
import EnvQuickToggles from "./EnvQuickToggles.jsx";

const LAUNCH_VERBS = ["waitforexitandrun","run","runinprefix","destroyprefix","getcompatpath","getnativepath"];
const STORES = ["","steam","gog","egs","origin","ubisoft","itch","heroic","amazon","humble","manual"];
const RUNTIME_VARIANTS = [
  { value: "", label: "Auto (detect from toolmanifest)" },
  { value: "sniper", label: "Steam Runtime 3.0 — Sniper (Proton 8+, GE-Proton)" },
  { value: "soldier", label: "Steam Runtime 2.0 — Soldier (Proton 5-7, legacy)" },
  { value: "steamrt4", label: "Steam Runtime 4.0 — Heavy (Proton 11+, experimental)" },
];

export default function ConfigPanel({ game, onSave, onClose, onRemove, onLaunch, onMetaUpdate, toast }) {
  const [cfg, setCfg]           = useState(null);
  const [compatTools, setTools] = useState([]);
  const [saving, setSaving]     = useState(false);
  const [tab, setTab]           = useState("general");
  const [dryRun, setDryRun]     = useState(null);
  const [showDryRun, setShowDryRun] = useState(false);
  const [coverBusy, setCoverBusy] = useState(false);
  const coverUrl = useCoverUrl(game?.cover_art);

  // Editable metadata state
  const [metaName, setMetaName] = useState("");
  const [metaExe, setMetaExe]   = useState("");
  const [metaNotes, setMetaNotes] = useState("");

  useEffect(() => {
    if (!game) return;
    setCfg(structuredClone(game.config));
    setMetaName(game.name || "");
    setMetaExe(game.exe_path || "");
    setMetaNotes(game.notes || "");
    setDryRun(null); setShowDryRun(false);
  }, [game?.id, game?.exe_path]);

  useEffect(() => { api.getCompatTools().then(setTools).catch(() => {}); }, []);

  if (!game || !cfg) return null;

  function patch(key, value) { setCfg((prev) => ({ ...prev, [key]: value })); }

  async function handleSave() {
    setSaving(true);
    try {
      await api.updateGameMeta(game.id, {
        name: metaName,
        exePath: metaExe,
        notes: metaNotes
      });
      await onSave(game.id, cfg);
      onMetaUpdate?.({
        ...game,
        name: metaName,
        exe_path: metaExe,
        notes: metaNotes,
        config: cfg
      });
    } catch(e) {
      toast?.(`Save failed: ${e}`, "error");
    } finally {
      setSaving(false);
    }
  }

  async function handleDryRun() {
    await handleSave();
    try { setDryRun(await api.dryRunLaunch(game.id)); setShowDryRun(true); }
    catch (e) { setDryRun({ error: String(e) }); setShowDryRun(true); }
  }

  const TABS = [{ id:"general", label:"General" },{ id:"env", label:"Env Vars" },{ id:"advanced", label:"Advanced" }];

  return (
    <div className="config-panel">
      <div className="config-panel__header">
        <div style={{ flex:1 }}>
          <div className="config-panel__title">{metaName || game.name}</div>
          <div style={{ fontFamily:"var(--font-mono)", fontSize:15, color:"var(--text-3)", marginTop:2 }}>{cfg.game_id}</div>
        </div>
        <button className="btn btn--ghost btn--icon btn--sm" onClick={onClose}>✕</button>
      </div>

      <div style={{ display:"flex", borderBottom:"2px solid var(--border)", padding:"0 18px", gap:2, flexShrink:0 }}>
        {TABS.map((t) => (
          <button key={t.id} onClick={() => setTab(t.id)} style={{
            padding:"6px 12px", background:"none", border:"none",
            borderBottom:`2px solid ${tab===t.id?"var(--lime)":"transparent"}`,
            color: tab===t.id?"var(--lime)":"var(--text-2)",
            cursor:"pointer", fontSize:12, fontWeight:600, fontFamily:"var(--font-body)", transition:"all 0.12s",
          }}>{t.label}</button>
        ))}
      </div>

      <div className="config-panel__body">
        {tab === "general" && <>
          <div className="config-panel__section">
            <div className="config-panel__section-title">Cover Art</div>
            <div style={{ display:"flex", gap:12, alignItems:"center" }}>
              <div style={{
                width:64, height:96, borderRadius:"var(--r-sm)",
                background:"var(--bg-3)", border:"2px solid var(--border)",
                overflow:"hidden", display:"flex", alignItems:"center", justifyContent:"center",
                flexShrink:0,
              }}>
                {coverUrl ? (
                  <img src={coverUrl} alt=""
                    style={{ width:"100%", height:"100%", objectFit:"cover" }} />
                ) : (
                  <span style={{ fontFamily:"var(--font-display)", fontSize:24,
                    color:"var(--text-3)", fontWeight:800 }}>
                    {game.name?.[0]?.toUpperCase() ?? "?"}
                  </span>
                )}
              </div>
              <div style={{ flex:1, display:"flex", flexDirection:"column", gap:6 }}>
                <div style={{ display:"flex", gap:6, flexWrap:"wrap" }}>
                  <button className="btn btn--primary btn--sm" disabled={coverBusy}
                    onClick={async () => {
                      setCoverBusy(true);
                      try {
                        const updated = await api.fetchCover(game.id);
                        if (updated?.cover_art) invalidateCover(updated.cover_art);
                        onMetaUpdate?.(updated);
                        toast?.("Cover fetched", "success");
                      } catch (e) { toast?.(`Cover fetch failed: ${e}`, "error"); }
                      finally { setCoverBusy(false); }
                    }}>
                    {coverBusy ? "Fetching…" : "↓ Auto-Fetch Cover"}
                  </button>
                  <button className="btn btn--secondary btn--sm" disabled={coverBusy}
                    onClick={async () => {
                      const file = await pickFile([
                        { name: "Images", extensions: ["jpg","jpeg","png","webp","bmp","gif"] },
                        { name: "All Files", extensions: ["*"] },
                      ]);
                      if (!file) return;
                      setCoverBusy(true);
                      try {
                        const updated = await api.importCover(game.id, file);
                        if (updated?.cover_art) invalidateCover(updated.cover_art);
                        onMetaUpdate?.(updated);
                        toast?.("Cover set", "success");
                      } catch (e) { toast?.(`Import failed: ${e}`, "error"); }
                      finally { setCoverBusy(false); }
                    }}>
                    Choose Image
                  </button>
                  {game.cover_art && (
                    <button className="btn btn--secondary btn--sm" disabled={coverBusy}
                      onClick={async () => {
                        setCoverBusy(true);
                        try {
                          await api.clearCover(game.id);
                          onMetaUpdate?.({ ...game, cover_art: null });
                          toast?.("Cover cleared", "info");
                        } catch (e) { toast?.(`Clear failed: ${e}`, "error"); }
                        finally { setCoverBusy(false); }
                      }}>
                      Clear
                    </button>
                  )}
                </div>
                <p className="form-hint" style={{ margin:0 }}>
                  Auto-fetches cover art for imported games, or choose any image file.
                </p>
              </div>
            </div>
          </div>

          <div className="config-panel__section">
            <div className="config-panel__section-title">Game Info</div>
            <div className="form-group">
              <label className="form-label">Display Name</label>
              <input className="form-input" value={metaName}
                onChange={(e) => setMetaName(e.target.value)} />
            </div>
            <div className="form-group">
              <label className="form-label">Executable Path</label>
              <div className="form-path-row">
                <input className="form-input form-input--mono" value={metaExe}
                  onChange={(e) => setMetaExe(e.target.value)}
                  placeholder="Path to executable…" spellCheck={false} />
                <button className="btn btn--secondary btn--sm"
                  onClick={async () => {
                    const f = await pickFile();
                    if (f) setMetaExe(f);
                  }}>
                  Browse
                </button>
              </div>
            </div>
            <div className="form-group">
              <label className="form-label">Notes</label>
              <textarea className="form-input" style={{ resize: "vertical", minHeight: 64, fontFamily: "var(--font-body)" }}
                value={metaNotes} onChange={(e) => setMetaNotes(e.target.value)}
                placeholder="Custom notes, launch details, etc.…" />
            </div>
          </div>

          <div className="config-panel__section">
            <div className="config-panel__section-title">Compatibility Tool</div>
            <div className="form-group">
              <label className="form-label">wine / proton / neutron path</label>
              <div className="form-path-row">
                <input className="form-input form-input--mono" value={cfg.proton_path ?? ""}
                  onChange={(e) => patch("proton_path", e.target.value || null)}
                  placeholder="Select or paste path…" spellCheck={false} />
                <button className="btn btn--secondary btn--sm"
                  onClick={async () => { const d = await pickDirectory(); if (d) patch("proton_path", d); }}>
                  Browse
                </button>
              </div>
              {compatTools.length > 0 && (
                <select className="form-select" style={{ marginTop:6 }}
                  value={cfg.proton_path ?? ""}
                  onChange={(e) => patch("proton_path", e.target.value || null)}>
                  <option value="">— pick from detected tools —</option>
                  {compatTools.map((t) => (
                    <option key={t.path} value={t.path}>[{t.kind}] {t.name}</option>
                  ))}
                </select>
              )}
            </div>
            <div className="form-group">
              <label className="form-label">Launch Verb</label>
              <select className="form-select" value={cfg.proton_verb}
                onChange={(e) => patch("proton_verb", e.target.value)}>
                {LAUNCH_VERBS.map((v) => <option key={v} value={v}>{v}</option>)}
              </select>
            </div>
            <div className="form-group">
              <label className="form-label">Linux Runtime</label>
              <select className="form-select"
                value={cfg.use_runtime === true ? "on" : cfg.use_runtime === false ? "off" : "auto"}
                onChange={(e) => {
                  const v = e.target.value;
                  patch("use_runtime", v === "on" ? true : v === "off" ? false : null);
                }}>
                <option value="auto">Auto (use toolmanifest)</option>
                <option value="on">Force ON — run inside runtime container</option>
                <option value="off">Force OFF — launch directly on host</option>
              </select>
              <p className="form-hint">Sniper-built tools skip the container automatically.</p>
            </div>
            {cfg.use_runtime !== false && (
            <div className="form-group">
              <label className="form-label">Runtime Container</label>
              <select className="form-select"
                value={cfg.runtime_variant ?? ""}
                onChange={(e) => patch("runtime_variant", e.target.value || null)}>
                {RUNTIME_VARIANTS.map(rv =>
                  <option key={rv.value} value={rv.value}>{rv.label}</option>
                )}
              </select>
              <p className="form-hint">Which SLR container to wrap the game in. Most tools use Sniper.</p>
            </div>
            )}
          </div>

          <div className="config-panel__section">
            <div className="config-panel__section-title">wine / proton / neutron Prefix</div>
            <div className="form-group">
              <label className="form-label">Prefix path</label>
              <div className="form-path-row">
                <input className="form-input form-input--mono" value={cfg.prefix_path ?? ""}
                  onChange={(e) => patch("prefix_path", e.target.value || null)}
                  placeholder="~/Games/mythix/…" spellCheck={false} />
                <button className="btn btn--secondary btn--sm"
                  onClick={async () => { const d = await pickDirectory(); if (d) patch("prefix_path", d); }}>
                  Browse
                </button>
              </div>
              <p className="form-hint">Will be created if it doesn't exist.</p>
            </div>
          </div>

          <div className="config-panel__section">
            <div className="config-panel__section-title">Game Identity</div>
            <div className="form-group">
              <label className="form-label">Game ID</label>
              <input className="form-input form-input--mono" value={cfg.game_id}
                onChange={(e) => patch("game_id", e.target.value)} spellCheck={false} />
              <p className="form-hint">Auto-generated 8-character ID. Used for prefix naming and compat tool integration.</p>
            </div>
            <div className="form-group">
              <label className="form-label">Store</label>
              <select className="form-select" value={cfg.store}
                onChange={(e) => patch("store", e.target.value)}>
                {STORES.map((s) => <option key={s} value={s}>{
                  { "": "— none —", steam: "Steam", gog: "GOG", egs: "Epic Games Store",
                    origin: "EA / Origin", ubisoft: "Ubisoft Connect", itch: "itch.io",
                    heroic: "Heroic (GOG/Epic)", amazon: "Amazon Games",
                    humble: "Humble Bundle", manual: "Manual / DRM-free" }[s] || s
                }</option>)}
              </select>
            </div>
          </div>


          <div className="config-panel__section">
            <div className="config-panel__section-title">Launch Arguments</div>
            <div className="form-group">
              <label className="form-label">Extra args</label>
              <input className="form-input form-input--mono"
                value={(cfg.launch_args ?? []).join(" ")}
                onChange={(e) => patch("launch_args", e.target.value.split(" ").filter(Boolean))}
                placeholder="-windowed -nosound" spellCheck={false} />
            </div>
          </div>
        </>}

        {tab === "env" && (
          <div className="config-panel__section">
            <div className="config-panel__section-title">Environment Overrides</div>
            <p style={{ fontSize:12, color:"var(--text-2)", marginBottom:12, lineHeight:1.4 }}>
              Injected at launch time. Common uses:{" "}
              <code style={{color:"var(--sky)"}}>DXVK_HUD</code>,{" "}
              <code style={{color:"var(--sky)"}}>PROTON_USE_WINED3D</code>,{" "}
              <code style={{color:"var(--sky)"}}>WINE_FULLSCREEN_FSR</code>.
            </p>
            <EnvQuickToggles envVars={cfg.env_overrides ?? {}} onChange={(v) => patch("env_overrides", v)} />
            <EnvVarsEditor value={cfg.env_overrides ?? {}} onChange={(v) => patch("env_overrides", v)} />
          </div>
        )}

        {tab === "advanced" && <>
          <div className="config-panel__section">
            <div className="config-panel__section-title">Dry-run Inspector</div>
            <button className="btn btn--secondary btn--sm" onClick={handleDryRun}>◎ Inspect launch env</button>
            {showDryRun && dryRun && (
              <div style={{ marginTop:12, background:"var(--bg-1)", border:"2px solid var(--border)",
                borderRadius:"var(--r-md)", padding:"10px 12px", overflow:"auto", maxHeight:300 }}>
                {dryRun.error
                  ? <span style={{ color:"var(--coral)", fontFamily:"var(--font-mono)", fontSize:11 }}>{dryRun.error}</span>
                  : <>
                    <div style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--lime)", marginBottom:8 }}>COMMAND</div>
                    <div style={{ fontFamily:"var(--font-mono)", fontSize:11, marginBottom:14, wordBreak:"break-all" }}>
                      {(dryRun.command ?? []).join(" ")}
                    </div>
                    <div style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--lime)", marginBottom:8 }}>ENVIRONMENT</div>
                    {Object.entries(dryRun.vars ?? {}).sort(([a],[b])=>a.localeCompare(b)).map(([k,v]) => (
                      <div key={k} style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-1)", marginBottom:3, wordBreak:"break-all" }}>
                        <span style={{color:"var(--sky)"}}>{k}</span>=<span>{v}</span>
                      </div>
                    ))}
                  </>
                }
              </div>
            )}
          </div>
          <div className="config-panel__section">
            <div className="config-panel__section-title">Danger Zone</div>
            <button className="btn btn--danger btn--sm" onClick={() => onRemove(game.id)}>✕ Remove from library</button>
            <p className="form-hint" style={{marginTop:6}}>Does not delete prefix data or game files.</p>
          </div>
        </>}
      </div>

      <div className="config-panel__footer">
        <button className="btn btn--primary" style={{flex:1}} onClick={handleSave} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </button>
        <button className="btn btn--secondary" style={{flex:1}} onClick={() => onLaunch(game.id)}>▶ Launch</button>
      </div>
    </div>
  );
}
