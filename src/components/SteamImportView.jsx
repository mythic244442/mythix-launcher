// components/SteamImportView.jsx
// Scans Steam libraries and lets users import games with prefix cloning.

import React, { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../tauri.js";

function formatSize(bytes) {
  if (!bytes || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let val = bytes;
  while (val >= 1024 && i < units.length - 1) { val /= 1024; i++; }
  return `${val.toFixed(1)} ${units[i]}`;
}

const STATUS_COLORS = {
  ready:    "var(--lime)",
  steam:    "var(--amber)",
  none:     "var(--coral)",
  imported: "var(--sky)",
};

export default function SteamImportView({ toast, onRefreshLibrary }) {
  const [games, setGames]         = useState([]);
  const [loading, setLoading]     = useState(true);
  const [importing, setImporting] = useState(null); // appid being imported
  const [importStatus, setStatus] = useState("");  // live status from backend
  const [search, setSearch]       = useState("");
  const unlistenRef               = useRef(null);
  const [compatTools, setTools]   = useState([]);
  const [selectedTool, setTool]   = useState("");

  async function scan() {
    setLoading(true);
    try {
      const [g, t] = await Promise.all([api.scanSteam(), api.getCompatTools()]);
      setGames(g);
      setTools(t);
      if (t.length > 0 && !selectedTool) setTool(t[0].path);
    } catch (e) {
      toast?.(`Scan failed: ${e}`, "error");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { scan(); }, []);

  // Listen for import progress events from the Rust backend
  useEffect(() => {
    let active = true;
    listen("import:status", (event) => {
      if (active) setStatus(event.payload);
    }).then((unlisten) => { unlistenRef.current = unlisten; });
    return () => { active = false; unlistenRef.current?.(); };
  }, []);

  async function handleImport(game) {
    setImporting(game.appid);
    setStatus("Starting import…");
    try {
      await api.importSteamGame({
        appid: game.appid,
        name: game.name,
        installPath: game.install_path,
        steamPrefix: game.steam_prefix,
        protonPath: selectedTool || null,
      });
      toast?.(`Imported ${game.name}`, "success");
      onRefreshLibrary?.();
      setGames((prev) => prev.map((g) =>
        g.appid === game.appid ? { ...g, already_imported: true } : g
      ));
    } catch (e) {
      toast?.(`Import failed: ${e}`, "error");
    } finally {
      setImporting(null);
      setTimeout(() => setStatus(""), 3000);
    }
  }

  const filtered = search.trim()
    ? games.filter((g) => g.name.toLowerCase().includes(search.toLowerCase()) ||
                          g.appid.includes(search))
    : games;

  const withPrefix = games.filter((g) => g.steam_prefix).length;
  const imported = games.filter((g) => g.already_imported).length;

  return (
    <div className="content" style={{ overflow:"hidden", display:"flex", flexDirection:"column" }}>
      <div className="content-header">
        <span className="content-header__title">Import Games</span>
        <span className="content-header__count">{games.length} games</span>
        <div className="content-header__spacer" />
        <div style={{ position:"relative" }}>
          <span style={{ position:"absolute", left:9, top:"50%", transform:"translateY(-50%)",
            color:"var(--text-3)", fontSize:13, pointerEvents:"none" }}>⌕</span>
          <input className="form-input" style={{ paddingLeft:26, width:200 }}
            placeholder="Search…" value={search}
            onChange={(e) => setSearch(e.target.value)} />
        </div>
        <button className="btn btn--secondary btn--sm" onClick={scan} disabled={loading}>
          {loading ? "Scanning…" : "↻ Rescan"}
        </button>
      </div>

      {/* Summary bar */}
      <div style={{
        padding:"10px 24px", borderBottom:"1px solid var(--border)",
        display:"flex", gap:20, fontSize:12, color:"var(--text-2)", flexShrink:0,
        background:"var(--bg-2)",
      }}>
        <span><strong style={{color:"var(--text-0)"}}>{games.length}</strong> games found</span>
        <span><span style={{color:"var(--amber)"}}>●</span> {withPrefix} have prefixes</span>
        <span><span style={{color:"var(--sky)"}}>●</span> {imported} already imported</span>

        <div style={{ marginLeft:"auto", display:"flex", alignItems:"center", gap:8 }}>
          <span style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-3)" }}>Default tool:</span>
          <select className="form-select" style={{ width:280, padding:"4px 8px", fontSize:11 }}
            value={selectedTool} onChange={(e) => setTool(e.target.value)}>
            <option value="">— none —</option>
            {compatTools.map((t) => (
              <option key={t.path} value={t.path}>[{t.kind}] {t.name}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Import status bar */}
      {importing && importStatus && (
        <div style={{
          padding:"8px 24px", borderBottom:"1px solid var(--border)",
          background:"rgba(180,167,214,0.06)", display:"flex", alignItems:"center", gap:10,
          flexShrink:0,
        }}>
          <div style={{
            width:14, height:14, border:"2px solid var(--lime)", borderTopColor:"transparent",
            borderRadius:"50%", animation:"spin 0.8s linear infinite", flexShrink:0,
          }} />
          <span style={{ fontFamily:"var(--font-mono)", fontSize:11, color:"var(--lime)" }}>
            {importStatus}
          </span>
          <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
        </div>
      )}

      {/* Game list */}
      <div style={{ flex:1, overflowY:"auto", padding:"12px 24px" }}>
        {loading ? (
          <div style={{ textAlign:"center", padding:60, color:"var(--text-3)",
            fontFamily:"var(--font-mono)", fontSize:12 }}>
            Scanning Steam libraries…
          </div>
        ) : filtered.length === 0 ? (
          <div className="empty-state">
            <span className="empty-state__icon">⊞</span>
            <span className="empty-state__title">
              {games.length === 0 ? "No Steam games found" : "No results"}
            </span>
            <span className="empty-state__subtitle">
              {games.length === 0
                ? "Make sure Steam is installed and has games in a library folder."
                : `No games match "${search}".`}
            </span>
          </div>
        ) : (
          <div style={{ display:"flex", flexDirection:"column", gap:6 }}>
            {filtered.map((game) => {
              const hasPrefix = !!game.steam_prefix;
              const isImported = game.already_imported;
              const isImporting = importing === game.appid;

              return (
                <div key={game.appid} style={{
                  background:"var(--bg-2)", border:"1px solid var(--border)",
                  borderLeft: isImported ? "3px solid var(--sky)"
                    : hasPrefix ? "3px solid var(--amber)"
                    : "3px solid var(--border)",
                  borderRadius:"var(--r-lg)", padding:"12px 16px",
                  display:"flex", alignItems:"center", gap:14,
                  opacity: isImported ? 0.6 : 1,
                  transition:"all 0.15s",
                }}>
                  {/* Game info */}
                  <div style={{ flex:1, minWidth:0 }}>
                    <div style={{
                      fontFamily:"var(--font-display)", fontWeight:700, fontSize:14,
                      color:"var(--text-0)", marginBottom:3,
                      whiteSpace:"nowrap", overflow:"hidden", textOverflow:"ellipsis",
                    }}>
                      {game.name}
                    </div>
                    <div style={{ display:"flex", gap:12, flexWrap:"wrap" }}>
                      <span style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-3)" }}>
                        AppID: {game.appid}
                      </span>
                      <span style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-3)" }}>
                        {formatSize(game.size_on_disk)}
                      </span>
                      <span style={{ fontFamily:"var(--font-mono)", fontSize:10, color:"var(--text-3)",
                        overflow:"hidden", textOverflow:"ellipsis", whiteSpace:"nowrap", maxWidth:200 }}>
                        {game.library_path.replace(/^\/home\/[^/]+/, "~")}
                      </span>
                    </div>
                  </div>

                  {/* Prefix status */}
                  <div style={{ textAlign:"right", flexShrink:0, minWidth:100 }}>
                    {isImported ? (
                      <span style={{
                        fontFamily:"var(--font-mono)", fontSize:9, fontWeight:700,
                        letterSpacing:"0.08em", textTransform:"uppercase",
                        color:STATUS_COLORS.imported,
                        background:"rgba(91,200,255,0.12)", border:"1px solid rgba(91,200,255,0.3)",
                        borderRadius:4, padding:"2px 6px",
                      }}>imported</span>
                    ) : hasPrefix ? (
                      <span style={{
                        fontFamily:"var(--font-mono)", fontSize:9, fontWeight:700,
                        letterSpacing:"0.08em", textTransform:"uppercase",
                        color:STATUS_COLORS.steam,
                        background:"rgba(255,187,56,0.12)", border:"1px solid rgba(255,187,56,0.3)",
                        borderRadius:4, padding:"2px 6px",
                      }}>has prefix</span>
                    ) : (
                      <span style={{
                        fontFamily:"var(--font-mono)", fontSize:9, fontWeight:700,
                        letterSpacing:"0.08em", textTransform:"uppercase",
                        color:STATUS_COLORS.none,
                        background:"rgba(255,95,95,0.08)", border:"1px solid rgba(255,95,95,0.2)",
                        borderRadius:4, padding:"2px 6px",
                      }}>no prefix</span>
                    )}
                  </div>

                  {/* Import button */}
                  <button
                    className={`btn ${isImported ? "btn--ghost" : hasPrefix ? "btn--primary" : "btn--secondary"} btn--sm`}
                    onClick={() => handleImport(game)}
                    disabled={isImported || isImporting}
                    style={{ flexShrink:0, minWidth:90 }}
                  >
                    {isImporting ? "Importing…" : isImported ? "Done" : hasPrefix ? "Import" : "Import (no pfx)"}
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
