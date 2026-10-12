// components/SteamImportView.jsx
// Scans Steam libraries and imports games. Two independent actions per game:
//  • Prefix — bring in a copy of Steam's prefix. Adds the game if it isn't in the
//    library yet (launching the game exe directly), or gives the existing entry
//    Steam's prefix.
//  • Loader — build a mythix-loader profile and launch the game from where it's
//    installed. Never touches prefixes: an existing entry keeps its own, a new one
//    gets a fresh prefix.

import React, { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../tauri.js";
import CustomImportModal from "./CustomImportModal.jsx";

function formatSize(bytes) {
  if (!bytes || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let val = bytes;
  while (val >= 1024 && i < units.length - 1) { val /= 1024; i++; }
  return `${val.toFixed(1)} ${units[i]}`;
}

const BADGE_TONES = {
  loader: { color:"var(--lime)",  bg:"rgba(180,167,214,0.12)", border:"rgba(180,167,214,0.3)" },
  sky:    { color:"var(--sky)",   bg:"rgba(91,200,255,0.12)",  border:"rgba(91,200,255,0.3)" },
  amber:  { color:"var(--amber)", bg:"rgba(255,187,56,0.12)",  border:"rgba(255,187,56,0.3)" },
  coral:  { color:"var(--coral)", bg:"rgba(255,95,95,0.08)",   border:"rgba(255,95,95,0.2)" },
};

function Badge({ tone, children }) {
  const t = BADGE_TONES[tone];
  return (
    <span style={{
      fontFamily:"var(--font-mono)", fontSize:9, fontWeight:700,
      letterSpacing:"0.08em", textTransform:"uppercase", whiteSpace:"nowrap",
      color:t.color, background:t.bg, border:`1px solid ${t.border}`,
      borderRadius:4, padding:"2px 6px",
    }}>{children}</span>
  );
}

export default function SteamImportView({ toast, onRefreshLibrary }) {
  const [games, setGames]         = useState([]);
  const [loading, setLoading]     = useState(true);
  const [importing, setImporting] = useState(null); // { appid, kind: "prefix" | "loader" }
  const [importStatus, setStatus] = useState("");  // live status from backend
  const [search, setSearch]       = useState("");
  const unlistenRef               = useRef(null);
  const [compatTools, setTools]   = useState([]);
  const [selectedTool, setTool]   = useState("");
  const [showCustom, setShowCustom] = useState(false); // custom-folder import modal

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

  // Re-read library state for every row without blanking the list.
  async function refreshQuiet() {
    try { setGames(await api.scanSteam()); } catch { /* keep what we have */ }
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

  async function runImport(game, kind, startMsg, doImport, okMsg) {
    setImporting({ appid: game.appid, kind });
    setStatus(startMsg);
    try {
      await doImport();
      toast?.(okMsg, "success");
      onRefreshLibrary?.();
      await refreshQuiet();
    } catch (e) {
      toast?.(`${kind === "loader" ? "Loader" : "Import"} failed: ${e}`, "error");
    } finally {
      setImporting(null);
      setTimeout(() => setStatus(""), 3000);
    }
  }

  // Prefix: copy Steam's prefix in. Adds the game, or gives its entry Steam's prefix.
  function handleImport(game) {
    const inLibrary = game.already_imported;
    return runImport(game, "prefix", inLibrary ? "Importing Steam's prefix…" : "Starting import…",
      () => api.importSteamGame({
        appid: game.appid,
        name: game.name,
        installPath: game.install_path,
        steamPrefix: game.steam_prefix,
        protonPath: selectedTool || null,
      }),
      inLibrary ? `${game.name} now uses a copy of Steam's prefix` : `Imported ${game.name}`);
  }

  // Loader: build (or rebuild) the mythix-loader profile and launch through it.
  // The game keeps whatever prefix it has; a new entry gets a fresh one.
  function handleLoaderImport(game) {
    const okMsg = game.uses_loader ? `Rebuilt ${game.name}'s loader profile`
      : game.already_imported ? `${game.name} now launches through the loader`
      : `Added ${game.name} via the loader`;
    return runImport(game, "loader", "Building loader profile…",
      () => api.importLoaderGame({
        installPath: game.install_path,
        appid: game.appid,
        name: game.name,
        protonPath: selectedTool || null,
      }),
      okMsg);
  }

  const filtered = search.trim()
    ? games.filter((g) => g.name.toLowerCase().includes(search.toLowerCase()) ||
                          g.appid.includes(search))
    : games;

  const withPrefix = games.filter((g) => g.steam_prefix).length;
  const imported = games.filter((g) => g.already_imported).length;
  const viaLoader = games.filter((g) => g.already_imported && g.uses_loader).length;

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
        <button className="btn btn--primary btn--sm" onClick={() => setShowCustom(true)}>
          ＋ From folder
        </button>
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
        <span><span style={{color:"var(--amber)"}}>●</span> {withPrefix} have Steam prefixes</span>
        <span><span style={{color:"var(--sky)"}}>●</span> {imported} in library</span>
        <span><span style={{color:"var(--lime)"}}>●</span> {viaLoader} via loader</span>

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
              const hasPrefix   = !!game.steam_prefix;          // Steam has a prefix to copy
              const inLibrary   = game.already_imported;
              const viaLoader   = inLibrary && game.uses_loader;
              const prefixReady = inLibrary && game.prefix_ready;  // entry's prefix exists on disk
              const busy        = importing?.appid === game.appid;
              const accent = viaLoader ? "var(--lime)" : inLibrary ? "var(--sky)"
                : hasPrefix ? "var(--amber)" : "var(--border)";

              // Prefix button: hidden only when there's nothing to bring in for an
              // entry that already exists.
              const showPrefix = hasPrefix || !inLibrary;
              const prefixLabel = busy && importing.kind === "prefix" ? "Importing…"
                : prefixReady && hasPrefix ? "Prefix ✓"
                : hasPrefix ? "Prefix" : "Import";
              const prefixTitle = prefixReady
                ? "This game already has a prefix"
                : !hasPrefix ? "Add the game with a fresh prefix (launches the game exe directly)"
                : inLibrary ? "Give this game a copy of Steam's prefix"
                : "Add the game with a copy of Steam's prefix (launches the game exe directly)";

              const loaderLabel = busy && importing.kind === "loader" ? "Building…"
                : viaLoader ? "Rebuild loader" : "Loader";
              const loaderTitle = viaLoader
                ? "Rebuild this game's mythix-loader profile"
                : inLibrary
                  ? "Launch this game through a mythix-loader profile — keeps its prefix"
                  : "Make a mythix-loader profile and add the game with a fresh prefix — launches from where it's installed";

              return (
                <div key={game.appid} style={{
                  background:"var(--bg-2)", border:"1px solid var(--border)",
                  borderLeft:`3px solid ${accent}`,
                  borderRadius:"var(--r-lg)", padding:"12px 16px",
                  display:"flex", alignItems:"center", gap:14,
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

                  {/* Status: library state on top, Steam's prefix below */}
                  <div style={{ display:"flex", flexDirection:"column", alignItems:"flex-end",
                    gap:4, flexShrink:0, minWidth:100 }}>
                    {inLibrary && <Badge tone={viaLoader ? "loader" : "sky"}>{viaLoader ? "via loader" : "in library"}</Badge>}
                    {hasPrefix ? <Badge tone="amber">steam prefix</Badge> : <Badge tone="coral">no steam prefix</Badge>}
                  </div>

                  {/* Prefix: bring in Steam's prefix (adds the game if needed) */}
                  {showPrefix && (
                    <button
                      className={`btn ${prefixReady ? "btn--ghost" : hasPrefix ? "btn--primary" : "btn--secondary"} btn--sm`}
                      onClick={() => handleImport(game)}
                      disabled={busy || prefixReady}
                      title={prefixTitle}
                      style={{ flexShrink:0, minWidth:84 }}
                    >
                      {prefixLabel}
                    </button>
                  )}

                  {/* Loader: profile in ~/.mythix/loader_profiles, launch in place */}
                  <button
                    className={`btn ${viaLoader ? "btn--ghost" : "btn--secondary"} btn--sm`}
                    onClick={() => handleLoaderImport(game)}
                    disabled={busy}
                    title={loaderTitle}
                    style={{ flexShrink:0, minWidth:84 }}
                  >
                    {loaderLabel}
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {showCustom && (
        <CustomImportModal
          toast={toast}
          onClose={() => setShowCustom(false)}
          onImported={() => { refreshQuiet(); onRefreshLibrary?.(); }}
        />
      )}
    </div>
  );
}
