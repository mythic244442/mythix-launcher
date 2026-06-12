import React, { useState, useEffect, useMemo } from "react";
import GameCard from "./GameCard.jsx";
import ConfigPanel from "./ConfigPanel.jsx";
import LogConsole from "./LogConsole.jsx";
import AddGameModal from "./AddGameModal.jsx";
import { api } from "../tauri.js";
import { useCoverUrl } from "../hooks/useCoverUrl.js";

function formatPlaytime(secs) {
  if (!secs) return null;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m played`;
  if (m > 0) return `${m}m played`;
  return "< 1m played";
}


function GameListItem({ game, selected, onSelect, onLaunch, onConfigure, controllerFocused }) {
  const coverUrl = useCoverUrl(game.cover_art);
  const initial = game.name?.[0]?.toUpperCase() ?? "?";
  const playtime = formatPlaytime(game.playtime_secs) ?? "Never played";
  const itemRef = React.useRef(null);

  React.useEffect(() => {
    if (controllerFocused && itemRef.current) {
      itemRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [controllerFocused]);

  // Format store platform name cleanly
  const storeRaw = game.config?.store || "";
  const storeLabel = storeRaw ? (storeRaw.charAt(0).toUpperCase() + storeRaw.slice(1)) : "Custom";
  const storeClass = `store-badge store-badge--${storeRaw.toLowerCase() || "custom"}`;
  const itemClass = `game-list__item game-list__item--${storeRaw.toLowerCase() || "custom"} ${selected ? "selected" : ""} ${controllerFocused ? "controller-focus" : ""}`;

  // Environment variables and launch arguments count
  const envCount = Object.keys(game.config?.env_overrides || {}).length;
  const argCount = game.config?.launch_args?.length || 0;

  // Prefix type/runtime label
  const prefixLabel = game.config?.prefix_type === "Wine" ? "Proton" : "Neutron";

  // Parse exact runner name/version from proton_path
  const runnerPath = game.config?.proton_path || "";
  const runnerParts = runnerPath.split(/[/\\]/);
  const runnerName = runnerParts.pop() || "";

  // Parse Date Added cleanly
  let addedLabel = "Unknown";
  if (game.added_at) {
    if (game.added_at.startsWith("unix:")) {
      const ts = parseInt(game.added_at.slice(5), 10);
      if (!isNaN(ts)) {
        addedLabel = new Date(ts * 1000).toLocaleDateString(undefined, {
          year: "numeric",
          month: "short",
          day: "numeric"
        });
      }
    } else {
      addedLabel = new Date(game.added_at).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric"
      });
    }
  }

  // Format executable filename/path nicely
  const exePath = game.exe_path || "";
  const exeParts = exePath.split(/[/\\]/);
  const exeName = exeParts.pop() || "";
  const folderName = exeParts.pop() || "";
  const shortExePath = folderName ? `.../${folderName}/${exeName}` : exeName;

  return (
    <div
      ref={itemRef}
      className={itemClass}
      onClick={() => onSelect(game)}
    >
      <div className="game-list__cell game-list__cell--main" style={{ flex: 1, gap: 16 }}>
        {coverUrl ? (
          <img
            className="game-list__cover"
            src={coverUrl}
            alt=""
            style={{ width: 48, height: 64, borderRadius: "var(--r-sm)" }}
          />
        ) : (
          <div className="game-list__cover-placeholder" style={{ width: 48, height: 64 }}>
            <span>{initial}</span>
          </div>
        )}
        <div className="game-list__info" style={{ flex: 1 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <span className="game-list__name" style={{ fontSize: "15px", fontWeight: "700" }}>{game.name}</span>
            <span className={storeClass} style={{ fontSize: "9px", padding: "3px 6px", textTransform: "uppercase" }}>{storeLabel}</span>
            <span className="runtime-badge" style={{ fontSize: "9px", padding: "3px 6px", textTransform: "uppercase" }}>
              {prefixLabel} {runnerName ? `• ${runnerName}` : ""}
            </span>
          </div>
          <div className="game-list__id-and-exe" style={{ marginTop: 6, display: "flex", alignItems: "center", gap: 12, color: "var(--text-3)", fontSize: "12px", flexWrap: "wrap" }}>
            {game.config?.game_id && (
              <span className="game-list__id" style={{ display: "flex", alignItems: "center", gap: 6 }}>
                ID: <strong style={{ color: "var(--text-2)" }}>{game.config.game_id}</strong>
              </span>
            )}
            {shortExePath && (
              <span className="game-list__exe-path" title={exePath}>
                📁 {shortExePath}
              </span>
            )}
            <span className="playtime-info" style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "12px", color: "var(--text-3)" }}>
              ⏱ <span style={{ color: "var(--text-2)" }}>{playtime}</span>
            </span>
            <span className="added-info" style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "12px", color: "var(--text-3)" }}>
              📅 Added <span style={{ color: "var(--text-2)" }}>{addedLabel}</span>
            </span>
            {envCount > 0 && (
              <span className="info-badge info-badge--env" title={`${envCount} environment overrides`} style={{ fontSize: "9px" }}>
                ⚡ {envCount} env
              </span>
            )}
            {argCount > 0 && (
              <span className="info-badge info-badge--args" title={`${argCount} launch arguments`} style={{ fontSize: "9px" }}>
                ⚙ {argCount} args
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="game-list__actions" style={{ marginLeft: 16 }}>
        <button
          className="btn btn--primary btn--sm btn--icon"
          onClick={(e) => {
            e.stopPropagation();
            onLaunch(game.id);
          }}
          title="Launch"
          style={{ width: 24, height: 24, borderRadius: "48%", display: "flex", alignItems: "center", justifyContent: "center" }}
        >
          ▶
        </button>
        <button
          className="btn btn--secondary btn--sm btn--icon"
          onClick={(e) => {
            e.stopPropagation();
            onConfigure(game);
          }}
          title="Configure"
          style={{ width: 24, height: 24, borderRadius: "48%", display: "flex", alignItems: "center", justifyContent: "center" }}
        >
          ⚙
        </button>
      </div>
    </div>
  );
}

function SpotlightHero({ game, onLaunch, onSelect }) {
  const coverUrl = useCoverUrl(game.cover_art);
  const initial = game.name?.[0]?.toUpperCase() ?? "?";
  const playtime = formatPlaytime(game.playtime_secs);

  return (
    <div className="spotlight" onClick={() => onSelect(game)}>
      <div className="spotlight__bg">
        {coverUrl && <img src={coverUrl} alt="" draggable={false} />}
      </div>
      <div className="spotlight__content">
        <div className="spotlight__info">
          <div className="spotlight__label">Most Played</div>
          <div className="spotlight__title">{game.name}</div>
          <div className="spotlight__meta">
            {playtime ?? "Never played"}
            {game.config?.store ? ` · ${game.config.store.charAt(0).toUpperCase() + game.config.store.slice(1)}` : ""}
          </div>
          <div className="spotlight__actions">
            <button
              className="btn btn--primary spotlight__play-btn"
              onClick={(e) => { e.stopPropagation(); onLaunch(game.id); }}
            >
              ▶ Play
            </button>
            <button
              className="btn btn--secondary spotlight__cfg-btn"
              onClick={(e) => { e.stopPropagation(); onSelect(game); }}
            >
              ⚙ Configure
            </button>
          </div>
        </div>
        <div className="spotlight__cover">
          {coverUrl ? (
            <img src={coverUrl} alt={game.name} draggable={false} />
          ) : (
            <div className="spotlight__cover-placeholder">{initial}</div>
          )}
        </div>
      </div>
    </div>
  );
}

export default function LibraryView({
  games,
  onAdd,
  onRemove,
  onUpdateConfig,
  onReplaceGame,
  onLaunch,
  toast,
  maximized,
  controllerIndex = -1,
  controllerSelected,
  onControllerSelect,
  runningGame,
  onKillGame,
}) {
  const [selected, setSelected] = useState(null);
  const [showAdd, setShowAdd] = useState(false);
  const [search, setSearch] = useState("");
  const [viewMode, setViewMode] = useState("grid");

  useEffect(() => {
    api.getSettings().then((s) => {
      if (s?.libraryView) setViewMode(s.libraryView);
    }).catch(() => {});
  }, []);

  function switchView(mode) {
    setViewMode(mode);
    api.getSettings().then((s) => api.saveSettings({ ...s, libraryView: mode })).catch(() => {});
  }

  const filtered = useMemo(() => {
    if (!search.trim()) return games;
    const q = search.toLowerCase();
    return games.filter(
      (g) =>
        g.name.toLowerCase().includes(q) ||
        g.config.game_id.toLowerCase().includes(q) ||
        g.config.store.toLowerCase().includes(q)
    );
  }, [games, search]);

  const spotlightGame = useMemo(() => {
    if (!maximized || games.length === 0) return null;
    return [...games].sort((a, b) => (b.playtime_secs || 0) - (a.playtime_secs || 0))[0];
  }, [maximized, games]);


  async function handleAdd(name, exePath) {
    const game = await onAdd(name, exePath);
    toast("Game added to library", "success");
    setSelected(game);
  }

  async function handleSaveConfig(gameId, config) {
    await onUpdateConfig(gameId, config);
    toast("Config saved", "success");
    setSelected((prev) =>
      prev?.id === gameId ? { ...prev, config } : prev
    );
  }

  async function handleRemove(gameId) {
    try {
      await onRemove(gameId);
      toast("Removed from library", "info");
      setSelected(null);
    } catch (e) {
      toast(`Remove failed: ${e}`, "error");
    }
  }

  async function handleLaunch(gameId) {
    try {
      await onLaunch(gameId);
      toast("Launched!", "success");
    } catch (e) {
      toast(`Launch failed: ${e}`, "error");
    }
  }

  return (
    <>
      <div
        style={{
          display: "flex",
          flex: 1,
          minHeight: 0,
          overflow: "hidden",
        }}
      >
        <div className="content">
          <div className="content-header">
            <span className="content-header__title">Library</span>
            <span className="content-header__count">{games.length}</span>

            {runningGame && (
              <button
                className="btn btn--danger btn--sm"
                onClick={onKillGame}
                title={`Stop ${runningGame.name} (PID ${runningGame.pid})`}
              >
                ■ Stop {runningGame.name}
              </button>
            )}

            <div className="content-header__spacer" />

            <div className="view-toggle">
              <button
                className={`view-toggle__btn ${viewMode === "grid" ? "active" : ""}`}
                onClick={() => switchView("grid")}
                title="Grid view"
              >
                ⊞ Grid
              </button>
              <button
                className={`view-toggle__btn ${viewMode === "list" ? "active" : ""}`}
                onClick={() => switchView("list")}
                title="List view"
              >
                ☰ List
              </button>
            </div>

            <div style={{ position: "relative" }}>
              <span
                style={{
                  position: "absolute",
                  left: 12,
                  top: "50%",
                  transform: "translateY(-50%)",
                  color: "var(--text-3)",
                  fontSize: 12,
                  pointerEvents: "none",
                }}
              >
                ⌕
              </span>
              <input
                className="form-input"
                style={{ paddingLeft: 28, width: 200 }}
                placeholder="Search games…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            <button
              className="btn btn--primary btn--sm"
              onClick={() => setShowAdd(true)}
            >
              ＋ Add Game
            </button>
          </div>

          {maximized && spotlightGame && !search.trim() && (
            <SpotlightHero
              game={spotlightGame}
              onLaunch={handleLaunch}
              onSelect={setSelected}
            />
          )}

          {filtered.length === 0 ? (
            <div
              className="empty-state"
              style={{ flex: 1, display: "flex" }}
            >
              <span className="empty-state__icon">⊞</span>
              {games.length === 0 ? (
                <>
                  <span className="empty-state__title">No games yet</span>
                  <span className="empty-state__subtitle">
                    Add your first game and configure your wine-proton hybrid
                    to get started.
                  </span>
                  <button
                    className="btn btn--primary"
                    style={{ marginTop: 9 }}
                    onClick={() => setShowAdd(true)}
                  >
                    ＋ Add Game
                  </button>
                </>
              ) : (
                <>
                  <span className="empty-state__title">No results</span>
                  <span className="empty-state__subtitle">
                    No games match "{search}".
                  </span>
                </>
              )}
            </div>
          ) : viewMode === "grid" ? (
            <div className={`game-grid-wrapper ${maximized ? "game-grid-wrapper--showcase" : ""}`}>
              <div className={`game-grid ${maximized ? "game-grid--showcase" : ""}`}>
                {filtered.map((game, i) => (
                  <GameCard
                    key={game.id}
                    game={game}
                    selected={selected?.id === game.id}
                    onSelect={setSelected}
                    onLaunch={handleLaunch}
                    onConfigure={setSelected}
                    controllerFocused={controllerIndex === i}
                  />
                ))}
              </div>
            </div>
          ) : (
            <div className="game-grid-wrapper">
              <div className="game-list">
                {filtered.map((game, i) => (
                  <GameListItem
                    key={game.id}
                    game={game}
                    selected={selected?.id === game.id}
                    onSelect={setSelected}
                    onLaunch={handleLaunch}
                    onConfigure={setSelected}
                    controllerFocused={controllerIndex === i}
                  />
                ))}
              </div>
            </div>
          )}
        </div>

        {selected && (
          <div style={{ display: 'flex', flexDirection: 'column', flex: '0 0 364px', height: '100%', minHeight: 0, overflow: 'hidden' }}>
            <ConfigPanel
              game={selected}
              onSave={handleSaveConfig}
              onClose={() => setSelected(null)}
              onRemove={handleRemove}
              onLaunch={handleLaunch}
              onMetaUpdate={(g) => {
                onReplaceGame(g);
                setSelected(g);
              }}
              toast={toast}
            />
            <LogConsole />
          </div>
        )}
      </div>

      {showAdd && (
        <AddGameModal onAdd={handleAdd} onClose={() => setShowAdd(false)} />
      )}
    </>
  );
}
