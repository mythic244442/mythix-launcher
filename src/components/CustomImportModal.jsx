// components/CustomImportModal.jsx
// "Import from folder" — two ways in:
//  • Steam game (loader): build a mythix-loader profile in ~/.mythix/loader_profiles/
//    and add its stub to the library. The game launches from where it's installed,
//    through the real steamclient (Steam must be running). Nothing is moved.
//  • Other game: plain import for non-Steam games — detect the exe, pick a tool.

import React, { useState, useEffect, useRef } from "react";
import { api, pickDirectory, pickFile } from "../tauri.js";

function nameFromPath(p) {
  const base = p.split(/[\\/]/).filter(Boolean).pop() || "";
  return base
    .replace(/\.exe$/i, "")
    .replace(/[_-]+/g, " ")
    .trim()
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

const tildify  = (p) => (p || "").replace(/^\/home\/[^/]+/, "~");
const baseName = (p) => (p || "").split(/[\\/]/).pop();

const SOURCE_LABEL = {
  appinfo:   "from Steam's launch config",
  heuristic: "best guess — double-check it",
  manual:    "picked manually",
};

export default function CustomImportModal({ onImported, onClose, toast }) {
  const [mode, setMode]           = useState("loader"); // "loader" | "plain"
  const [folder, setFolder]       = useState("");
  const [exe, setExe]             = useState("");
  const [name, setName]           = useState("");
  const [nameTouched, setTouched] = useState(false);
  const [appid, setAppid]         = useState("");
  const [plan, setPlan]           = useState(null);
  const [tools, setTools]         = useState([]);
  const [tool, setTool]           = useState("");
  const [busy, setBusy]           = useState("");      // "" | "detecting" | "importing"
  const [err, setErr]             = useState("");
  const previewReq                = useRef(0);

  useEffect(() => {
    api.getCompatTools()
      .then((t) => { setTools(t); if (t.length > 0) setTool(t[0].path); })
      .catch(() => {});
  }, []);

  // Dry-run the loader: appid, name, launch target, prefix plan — writes nothing.
  async function preview(dir, id) {
    if (!dir) return;
    const req = ++previewReq.current;
    setBusy("detecting"); setErr(""); setPlan(null);
    try {
      const p = await api.previewLoaderImport(dir, id || null);
      if (req !== previewReq.current) return;           // superseded by a newer preview
      setPlan(p);
      if (!id && p.appid && p.appid !== "0") setAppid(p.appid);
      if (!nameTouched && p.name) setName(p.name);
    } catch (ex) {
      if (req !== previewReq.current) return;
      setErr(String(ex));
    } finally {
      if (req === previewReq.current) setBusy("");
    }
  }

  async function detectExe(dir) {
    setBusy("detecting");
    try {
      const found = await api.findExe(dir);
      if (found) setExe(found);
    } catch {
      /* best-effort; the user can Browse for the exe */
    } finally {
      setBusy("");
    }
  }

  async function handlePickFolder() {
    const dir = await pickDirectory();
    if (!dir) return;
    setFolder(dir);
    setErr("");
    if (!nameTouched) setName(nameFromPath(dir));
    if (mode === "loader") {
      setAppid("");
      await preview(dir, "");
    } else {
      await detectExe(dir);
    }
  }

  async function handlePickExe() {
    const p = await pickFile();
    if (!p) return;
    setExe(p);
    if (!nameTouched) setName(nameFromPath(p));
  }

  function switchMode(m) {
    if (m === mode) return;
    setMode(m);
    setErr("");
    if (m === "loader" && folder.trim() && !plan) preview(folder.trim(), appid.trim());
    if (m === "plain" && folder.trim() && !exe) detectExe(folder.trim());
  }

  async function handleImport(e) {
    e.preventDefault();
    if (!name.trim()) { setErr("Name is required."); return; }
    setErr("");
    setBusy("importing");
    try {
      if (mode === "loader") {
        const id = appid.trim();
        await api.importLoaderGame({
          installPath: folder.trim(),
          appid: id || null,
          name: name.trim(),
          protonPath: tool || null,
        });
        const same = plan && plan.appid === id;
        toast?.(same && plan.uses_loader ? `Rebuilt ${name.trim()}'s loader profile`
          : same && plan.in_library ? `${name.trim()} now launches through the loader`
          : `Added ${name.trim()} via the loader`, "success");
      } else {
        await api.importCustomGame({
          name: name.trim(),
          installPath: folder.trim() || exe.trim(),
          exePath: exe.trim() || null,
          protonPath: tool || null,
        });
        toast?.(`Imported ${name.trim()}`, "success");
      }
      onImported?.();
      onClose();
    } catch (ex) {
      setErr(String(ex));
    } finally {
      setBusy("");
    }
  }

  const appidMissing = mode === "loader" && plan && plan.appid === "0" && !appid.trim();
  const canImport = busy === "" && !!name.trim() && !appidMissing &&
    (mode === "loader" ? !!folder.trim() : !!(folder.trim() || exe.trim()));
  const submitLabel = mode !== "loader" ? "Import"
    : plan?.uses_loader ? "Rebuild loader"
    : plan?.in_library ? "Switch to loader"
    : "Import";

  return (
    <div className="modal-backdrop" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal">
        <div className="modal__header">
          <span className="modal__title">Import from folder</span>
          <button className="modal__close" onClick={onClose} data-nav-close>✕</button>
        </div>
        <form onSubmit={handleImport}>
          <div className="form-group" style={{ display:"flex", gap:6, marginBottom:6 }}>
            <button type="button" onClick={() => switchMode("loader")}
              className={`btn btn--sm ${mode === "loader" ? "btn--primary" : "btn--ghost"}`}>
              Steam game (loader)
            </button>
            <button type="button" onClick={() => switchMode("plain")}
              className={`btn btn--sm ${mode === "plain" ? "btn--primary" : "btn--ghost"}`}>
              Other game
            </button>
          </div>
          <p style={{ fontSize:11, color:"var(--text-3)", margin:"0 0 12px" }}>
            {mode === "loader"
              ? "Makes a mythix-loader profile and launches the game right where it's installed, through your running Steam. Prefixes are left alone."
              : "For non-Steam games: point at the exe and pick a compatibility tool."}
          </p>

          <div className="form-group">
            <label className="form-label">Game folder</label>
            <div className="form-path-row">
              <input className="form-input form-input--mono" value={folder}
                onChange={(e) => { setFolder(e.target.value); setPlan(null); }}
                onBlur={() => {
                  if (mode === "loader" && folder.trim() && !plan && busy === "") preview(folder.trim(), appid.trim());
                }}
                placeholder="/path/to/game" spellCheck={false} />
              <button type="button" className="btn btn--secondary btn--sm" onClick={handlePickFolder}>Browse</button>
            </div>
          </div>

          {mode === "loader" ? (
            <>
              <div className="form-group">
                <label className="form-label">
                  Steam AppID
                  {busy === "detecting" && <span style={{ color:"var(--text-3)", fontSize:11, marginLeft:6 }}>· detecting…</span>}
                </label>
                <div className="form-path-row">
                  <input className="form-input form-input--mono" value={appid}
                    onChange={(e) => setAppid(e.target.value.replace(/\D/g, ""))}
                    placeholder="auto-detected from your Steam library" spellCheck={false} />
                  <button type="button" className="btn btn--secondary btn--sm"
                    onClick={() => preview(folder.trim(), appid.trim())}
                    disabled={!folder.trim() || busy !== ""}>
                    Re-check
                  </button>
                </div>
              </div>
              {plan && <LoaderPlanCard plan={plan} />}
            </>
          ) : (
            <div className="form-group">
              <label className="form-label">
                Executable
                {busy === "detecting" && <span style={{ color:"var(--text-3)", fontSize:11, marginLeft:6 }}>· detecting…</span>}
              </label>
              <div className="form-path-row">
                <input className="form-input form-input--mono" value={exe}
                  onChange={(e) => setExe(e.target.value)}
                  placeholder="auto-detected from the folder" spellCheck={false} />
                <button type="button" className="btn btn--secondary btn--sm" onClick={handlePickExe}>Browse</button>
              </div>
            </div>
          )}

          <div className="form-group">
            <label className="form-label">Name</label>
            <input className="form-input" value={name}
              onChange={(e) => { setName(e.target.value); setTouched(true); }}
              placeholder="My Game" />
          </div>
          <div className="form-group">
            <label className="form-label">Compatibility tool</label>
            <select className="form-select" value={tool} onChange={(e) => setTool(e.target.value)}>
              <option value="">— none (set later) —</option>
              {tools.map((t) => (
                <option key={t.path} value={t.path}>[{t.kind}] {t.name}</option>
              ))}
            </select>
          </div>
          {err && <p style={{ color:"var(--coral)", fontSize:12, marginBottom:12 }}>{err}</p>}
          <div className="modal__footer">
            <button type="button" className="btn btn--ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn--primary" disabled={!canImport}>
              {busy === "importing" ? "Working…" : submitLabel}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// What the loader will do, from a dry run of mythix-loader.
function LoaderPlanCard({ plan }) {
  const found = plan.appid && plan.appid !== "0";
  const opts = plan.launch_options || [];
  const library = plan.uses_loader ? "rebuilds its loader profile"
    : plan.in_library ? "switches its library entry to the loader"
    : "adds it to your library";
  const prefix = plan.in_library ? "keeps the game's own prefix"
    : plan.steam_prefix ? "fresh — or give it a copy of Steam's with Prefix in the list"
    : "fresh prefix";
  return (
    <div style={{
      background:"var(--bg-2)", border:"1px solid var(--border)",
      borderLeft:`3px solid ${found ? "var(--lime)" : "var(--amber)"}`,
      borderRadius:"var(--r-lg)", padding:"10px 12px", marginBottom:14,
      fontSize:12, lineHeight:1.7, minWidth:0,
    }}>
      {found ? (
        <div style={{ fontWeight:700, color:"var(--text-0)" }}>
          {plan.name}
          <span style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-2)", marginLeft:8 }}>
            AppID {plan.appid}
          </span>
        </div>
      ) : (
        <div style={{ color:"var(--amber)" }}>
          Not found in your Steam library — enter its AppID above and hit Re-check.
        </div>
      )}
      <PlanRow k="Launches" v={baseName(plan.exe)} note={SOURCE_LABEL[plan.launch_source] || ""} />
      {opts.length > 0 && (
        <PlanRow k="Steam opts" v={opts.join(" · ")} note="not applied — add one as a launch arg" />
      )}
      {found && <PlanRow k="Library" v={library} />}
      <PlanRow k="Prefix" v={prefix} />
      <PlanRow k="Profile" v={tildify(plan.profile_dir)} />
    </div>
  );
}

function PlanRow({ k, v, note }) {
  return (
    <div style={{ display:"flex", gap:8, minWidth:0, alignItems:"baseline" }}>
      <span style={{ fontFamily:"var(--font-mono)", fontSize:9, color:"var(--text-2)", minWidth:72,
        textTransform:"uppercase", letterSpacing:"0.06em", flexShrink:0 }}>{k}</span>
      <span title={v} style={{ fontFamily:"var(--font-mono)", fontSize:11, color:"var(--text-1)",
        overflow:"hidden", textOverflow:"ellipsis", whiteSpace:"nowrap", minWidth:0 }}>{v}</span>
      {note && <span style={{ fontSize:10, color:"var(--text-2)", whiteSpace:"nowrap", flexShrink:0 }}>· {note}</span>}
    </div>
  );
}
