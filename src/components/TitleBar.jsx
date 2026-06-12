import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../tauri.js";
import { NavDropdown } from "./NavRail.jsx";

function ControllerIndicator() {
  const [status, setStatus] = useState("disconnected");

  useEffect(() => {
    function check() {
      const gps = navigator.getGamepads?.() ?? [];
      const connected = Array.from(gps).some(Boolean);
      setStatus(connected ? "connected" : "disconnected");
    }

    check();
    window.addEventListener("gamepadconnected", () => setStatus("connected"));
    window.addEventListener("gamepaddisconnected", check);
    const interval = setInterval(check, 3000);

    return () => {
      clearInterval(interval);
      window.removeEventListener("gamepadconnected", () => setStatus("connected"));
      window.removeEventListener("gamepaddisconnected", check);
    };
  }, []);

  const connected = status === "connected";

  return (
    <div className="controller-indicator" title={connected ? "Controller connected" : "No controller"}>
      <span className="controller-indicator__dot" style={{
        background: connected ? "#39ff14" : "transparent",
        boxShadow: connected ? "0 0 6px #39ff14" : "none",
        border: connected ? "none" : "1px solid rgba(180,167,214,0.2)",
      }} />
      <span className="controller-indicator__icon" style={{
        opacity: connected ? 0.9 : 0.25,
      }}>🎮</span>
    </div>
  );
}

function ControllerLegend({ runningGame, controllerActive }) {
  if (!controllerActive) return null;
  const hints = [];
  if (runningGame) {
    hints.push({ btn: "X/Y", label: "Stop" });
  } else {
    hints.push({ btn: "X", label: "Settings" });
  }
  hints.push({ btn: "A", label: "Launch" });
  hints.push({ btn: "B", label: "Back" });
  hints.push({ btn: "⌂", label: "Hide" });

  return (
    <div className="controller-legend">
      {hints.map(({ btn, label }) => (
        <div key={btn} className="controller-legend__hint">
          <span className="controller-legend__btn">{btn}</span>
          <span className="controller-legend__label">{label}</span>
        </div>
      ))}
    </div>
  );
}

export default function TitleBar({ maximized, active, onNavigate, runningGame, onKillGame, controllerActive }) {
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

      <ControllerIndicator />
      <ControllerLegend runningGame={runningGame} controllerActive={controllerActive} />

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
