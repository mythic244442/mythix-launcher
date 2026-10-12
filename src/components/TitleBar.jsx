import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../tauri.js";
import { NavDropdown } from "./NavRail.jsx";
import { glyphsFor } from "../controller/glyphs.js";
import { useOverlays } from "../controller/overlays.js";
import { useControllerState } from "../hooks/useGamepad.js";

function ControllerIndicator({ controllerType, connected }) {
  const isConnected = connected > 0;
  const label = controllerType === "playstation" ? "🎮 PS"
    : controllerType === "switch" ? "🎮 NS"
    : "🎮";

  return (
    <div className="controller-indicator" title={isConnected ? `${controllerType} controller connected` : "No controller"}>
      <span className="controller-indicator__dot" style={{
        background: isConnected ? "#39ff14" : "transparent",
        boxShadow: isConnected ? "0 0 6px #39ff14" : "none",
        border: isConnected ? "none" : "1px solid rgba(180,167,214,0.2)",
      }} />
      <span className="controller-indicator__icon" style={{
        opacity: isConnected ? 0.9 : 0.25,
      }}>{label}</span>
    </div>
  );
}

// What kind of thing has focus — drives the A-button hint.
function useFocusKind() {
  const [kind, setKind] = useState("none");
  useEffect(() => {
    function update() {
      const el = document.activeElement;
      if (!el || el === document.body) setKind("none");
      else if (el.matches(".game-card, .game-list__item")) setKind("game");
      else if (el.matches("input, textarea")) setKind("text");
      else if (el.matches("select")) setKind("select");
      else setKind("other");
    }
    const deferred = () => setTimeout(update, 0);
    document.addEventListener("focusin", update);
    document.addEventListener("focusout", deferred);
    update();
    return () => {
      document.removeEventListener("focusin", update);
      document.removeEventListener("focusout", deferred);
    };
  }, []);
  return kind;
}

function ControllerLegend({ runningGame, controllerActive, controllerType }) {
  const focusKind = useFocusKind();
  const { keyboard, picker } = useOverlays();
  // The keyboard and picker draw their own hint bars.
  if (!controllerActive || keyboard || picker) return null;
  const g = glyphsFor(controllerType);
  const aLabel = {
    game: runningGame ? "Details" : "Launch",
    text: "Type",
    select: "Choose",
  }[focusKind] || "Select";

  const hints = [
    { btn: `${g.lb}/${g.rb}`, label: "Sections" },
    { btn: g.a, label: aLabel },
  ];
  if (focusKind === "game") {
    hints.push({ btn: g.start, label: "Configure" });
    hints.push({ btn: g.x, label: "Quick Cfg" });
  }
  hints.push({ btn: g.b, label: "Back" });
  if (runningGame) hints.push({ btn: g.y, label: "Stop" });
  hints.push({ btn: g.guide, label: "Hide" });

  return (
    <div className="controller-legend">
      {hints.map(({ btn, label }) => (
        <div key={label} className="controller-legend__hint">
          <span className="controller-legend__btn">{btn}</span>
          <span className="controller-legend__label">{label}</span>
        </div>
      ))}
    </div>
  );
}

export default function TitleBar({ maximized, active, onNavigate, runningGame, onKillGame, controllerActive, controllerType }) {
  const { connected } = useControllerState();
  function onMouseDown(e) {
    if (e.button !== 0) return;
    if (e.target.closest("button")) return;
    if (e.target.closest(".nav-dropdown")) return;
    e.preventDefault();
    getCurrentWindow().startDragging().catch(() => {});
  }

  function onDoubleClick(e) {
    if (e.target.closest("button")) return;
    if (e.target.closest(".nav-dropdown")) return;
    api.windowMaximize();
  }

  return (
    <div
      className="titlebar"
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
    >
      <span className="titlebar__wordmark">━━━━  MYTHIX  ━━━━</span>
      {!maximized && (
        <NavDropdown active={active} onNavigate={onNavigate} />
      )}
      <span className="titlebar__spacer" />

      {runningGame && (
        <button
          className="titlebar__running"
          onClick={onKillGame}
          title={`Stop ${runningGame.name} (PID ${runningGame.pid})`}
        >
          <span className="titlebar__running-dot" />
          <span className="titlebar__running-name">{runningGame.name}</span>
          <span className="titlebar__running-stop">■</span>
        </button>
      )}

      <ControllerIndicator controllerType={controllerType} connected={connected} />
      <ControllerLegend runningGame={runningGame} controllerActive={controllerActive} controllerType={controllerType} />

      <div className="titlebar__controls">
        <button className="titlebar__btn" onClick={api.windowMinimize}>
          ─
        </button>
        <button className="titlebar__btn" onClick={api.windowMaximize}>
          ▢
        </button>
        <button className="titlebar__btn titlebar__btn--close" onClick={api.windowClose}>
          ✕
        </button>
      </div>
    </div>
  );
}
