import React, { useState, useCallback, useEffect, useRef, Suspense, lazy } from "react";
import "./styles.css";
import TitleBar from "./components/TitleBar.jsx";
import NavRail from "./components/NavRail.jsx";
const LibraryView = lazy(() => import("./components/LibraryView.jsx"));
const SteamImportView = lazy(() => import("./components/SteamImportView.jsx"));
const CompatToolsView = lazy(() => import("./components/CompatToolsView.jsx"));
const SettingsView = lazy(() => import("./components/SettingsView.jsx"));
import RuntimeSetup from "./components/RuntimeSetup.jsx";
import ToastContainer from "./components/ToastContainer.jsx";
import QuickSettings from "./components/QuickSettings.jsx";
import { useLibrary } from "./hooks/useLibrary.js";
import { useRuntime, DEFAULT_VARIANT } from "./hooks/useRuntime.js";
import { useToast } from "./hooks/useToast.js";
import { useMaximized } from "./hooks/useMaximized.js";
import { useControllerNav } from "./hooks/useControllerNav.js";
import { api } from "./tauri.js";

export default function App() {
  const [view, setView] = useState("library");
  const [selectedGame, setSelectedGame] = useState(null);
  const [runningGame, setRunningGame] = useState(null);
  const { games, addGame, removeGame, updateConfig, refresh, replaceGame } = useLibrary();
  const { toasts, toast } = useToast();
  const runtime = useRuntime();
  const maximized = useMaximized();
  const [setupSkipped, setSetupSkipped] = useState(false);

  const [quickSettingsGame, setQuickSettingsGame] = useState(null);
  const [setupDone, setSetupDone] = useState(false);
  const showSetup = !runtime.loading && !runtime.isInstalled && !setupSkipped && !setupDone;

  const handleLaunch = useCallback((gameId) => {
    api.launchGame(gameId).then((result) => {
      const game = games.find((g) => g.id === gameId);
      setRunningGame({ pid: result.pid, gameId, name: game?.name ?? gameId });
      toast("Launched!", "success");
    }).catch((e) => toast(`Launch failed: ${e}`, "error"));
  }, [toast, games]);

  const handleKillGame = useCallback(() => {
    if (!runningGame) return;
    api.killGame(runningGame.pid, runningGame.gameId).then(() => {
      toast(`Stopped ${runningGame.name}`, "info");
      setRunningGame(null);
    }).catch((e) => toast(`Kill failed: ${e}`, "error"));
  }, [runningGame, toast]);

  const guideSuppressed = useRef(false);

  useEffect(() => {
    let unlistenRestore, unlistenExit;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen("gamepad:restored", () => {
        guideSuppressed.current = true;
        setTimeout(() => { guideSuppressed.current = false; }, 1000);
      }).then(fn => { unlistenRestore = fn; });
      listen("game:exited", (ev) => {
        const { gameId, playtime } = ev.payload;
        setRunningGame(null);
        refresh();
      }).then(fn => { unlistenExit = fn; });
    });
    return () => { unlistenRestore?.(); unlistenExit?.(); };
  }, [refresh]);

  const handleGuide = useCallback(() => {
    if (guideSuppressed.current) return;
    api.windowHide();
  }, []);

  const { zone, gridIndex, controllerActive } = useControllerNav({
    view,
    onNavigate: setView,
    games,
    onSelectGame: setSelectedGame,
    onLaunchGame: handleLaunch,
    onKillGame: handleKillGame,
    onGuide: handleGuide,
    onQuickSettings: setQuickSettingsGame,
    maximized,
    runningGame,
    overlayOpen: !!quickSettingsGame,
  });

  React.useEffect(() => {
    if (runtime.isInstalled && !setupSkipped) {
      setSetupDone(true);
    }
  }, [runtime.isInstalled, setupSkipped]);

  return (
    <div className="app-shell">
      <TitleBar maximized={maximized} active={view} onNavigate={setView} runningGame={runningGame} onKillGame={handleKillGame} controllerActive={controllerActive} />
      {showSetup ? (
        <RuntimeSetup
          progress={runtime.progress}
          installing={runtime.installing}
          error={runtime.error}
          onInstall={() => runtime.install(DEFAULT_VARIANT)}
          onSkip={() => setSetupSkipped(true)}
        />
      ) : (
        <NavRail
          active={view}
          onNavigate={setView}
          maximized={maximized}
          controllerZone={controllerActive ? zone : null}
        >
          <Suspense fallback={<div>Loading...</div>}>
            {view === "library" && (
              <LibraryView
                games={games}
                onAdd={addGame}
                onRemove={removeGame}
                onUpdateConfig={updateConfig}
                onReplaceGame={replaceGame}
                onLaunch={handleLaunch}
                toast={toast}
                maximized={maximized}
                controllerIndex={controllerActive && zone === "content" ? gridIndex : -1}
                controllerSelected={selectedGame}
                onControllerSelect={setSelectedGame}
                runningGame={runningGame}
                onKillGame={handleKillGame}
              />
            )}
            {view === "steam" && (
              <SteamImportView toast={toast} onRefreshLibrary={refresh} />
            )}
            {view === "tools" && <CompatToolsView />}
            {view === "settings" && <SettingsView />}
          </Suspense>
        </NavRail>
      )}
      <ToastContainer toasts={toasts} />
      {quickSettingsGame && (
        <QuickSettings
          game={quickSettingsGame}
          onSave={updateConfig}
          onClose={() => setQuickSettingsGame(null)}
          onLaunch={handleLaunch}
        />
      )}
    </div>
  );
}
