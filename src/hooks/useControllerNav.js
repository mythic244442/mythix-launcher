import { useState, useCallback, useEffect, useRef } from "react";
import { useGamepad } from "./useGamepad.js";

const SPIRAL_MAP = {
  library:  { down: "content", right: "steam",    left: "tools"  },
  tools:    { up: "library",   right: "content",  down: "settings" },
  steam:    { up: "library",   left: "content",   down: "settings" },
  settings: { up: "content",   left: "tools",     right: "steam" },
};

function measureGrid() {
  const grid = document.querySelector(".game-grid");
  if (grid && grid.children.length > 0) {
    const count = grid.children.length;
    let cols = 1;
    if (count >= 2) {
      const firstRect = grid.children[0].getBoundingClientRect();
      for (let i = 1; i < count; i++) {
        const rect = grid.children[i].getBoundingClientRect();
        if (Math.abs(rect.top - firstRect.top) < 10) cols++;
        else break;
      }
    }
    return { count, cols, mode: "grid" };
  }

  const list = document.querySelector(".game-list");
  if (list && list.children.length > 0) {
    return { count: list.children.length, cols: 1, mode: "list" };
  }

  return { count: 0, cols: 1, mode: "none" };
}

function scrollFocusedIntoView() {
  requestAnimationFrame(() => {
    const el = document.querySelector(".game-card.controller-focus, .game-list__item.controller-focus");
    if (el) el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });
}

export function useControllerNav({ view, onNavigate, games, onSelectGame, onLaunchGame, onKillGame, onGuide, onQuickSettings, maximized, runningGame, overlayOpen }) {
  const [zone, setZone] = useState("content");
  const [gridIndex, setGridIndex] = useState(0);
  const [controllerActive, setControllerActive] = useState(false);

  const stateRef = useRef({ zone: "content", gridIndex: 0, runningGame: null, overlayOpen: false });

  useEffect(() => { stateRef.current.zone = zone; }, [zone]);
  useEffect(() => { stateRef.current.gridIndex = gridIndex; }, [gridIndex]);
  useEffect(() => { stateRef.current.runningGame = runningGame; }, [runningGame]);
  useEffect(() => { stateRef.current.overlayOpen = overlayOpen; }, [overlayOpen]);

  // Reset grid index when view changes
  useEffect(() => { setGridIndex(0); stateRef.current.gridIndex = 0; }, [view]);

  useEffect(() => {
    function onConnect() { setControllerActive(true); }
    function onDisconnect() {
      const gps = navigator.getGamepads?.() ?? [];
      if (!Array.from(gps).some(Boolean)) setControllerActive(false);
    }
    window.addEventListener("gamepadconnected", onConnect);
    window.addEventListener("gamepaddisconnected", onDisconnect);
    const gps = navigator.getGamepads?.() ?? [];
    if (Array.from(gps).some(Boolean)) setControllerActive(true);
    return () => {
      window.removeEventListener("gamepadconnected", onConnect);
      window.removeEventListener("gamepaddisconnected", onDisconnect);
    };
  }, []);

  const handleInput = useCallback((action) => {
    if (!maximized) return;
    setControllerActive(true);

    if (stateRef.current.overlayOpen) return;

    const cur = stateRef.current;

    // --- Global shortcuts ---
    if (action === "guide") { onGuide?.(); return; }
    if ((action === "y" || action === "x") && stateRef.current.runningGame) { onKillGame?.(); return; }

    // --- Spiral button navigation ---
    if (cur.zone !== "content") {
      const nav = SPIRAL_MAP[cur.zone];
      if (action === "a") {
        onNavigate(cur.zone);
        setZone("content");
        setGridIndex(0);
        stateRef.current.zone = "content";
        stateRef.current.gridIndex = 0;
        return;
      }
      if (action === "b") {
        setZone("content");
        stateRef.current.zone = "content";
        return;
      }
      const target = nav?.[action];
      if (target) {
        setZone(target);
        stateRef.current.zone = target;
        if (target === "content") {
          setGridIndex(0);
          stateRef.current.gridIndex = 0;
        }
      }
      return;
    }

    // --- Content zone navigation ---
    const { count, cols } = measureGrid();

    if (count > 0) {
      const idx = Math.min(cur.gridIndex, count - 1);

      if (action === "up") {
        if (idx < cols) {
          setZone("library");
          stateRef.current.zone = "library";
        } else {
          const next = idx - cols;
          setGridIndex(next);
          stateRef.current.gridIndex = next;
          scrollFocusedIntoView();
        }
      } else if (action === "down") {
        const next = idx + cols;
        if (next >= count) {
          setZone("settings");
          stateRef.current.zone = "settings";
        } else {
          setGridIndex(next);
          stateRef.current.gridIndex = next;
          scrollFocusedIntoView();
        }
      } else if (action === "left") {
        if (cols === 1 || idx % cols === 0) {
          setZone("tools");
          stateRef.current.zone = "tools";
        } else {
          const next = idx - 1;
          setGridIndex(next);
          stateRef.current.gridIndex = next;
          scrollFocusedIntoView();
        }
      } else if (action === "right") {
        if (cols === 1 || (idx + 1) % cols === 0 || idx === count - 1) {
          setZone("steam");
          stateRef.current.zone = "steam";
        } else {
          const next = idx + 1;
          setGridIndex(next);
          stateRef.current.gridIndex = next;
          scrollFocusedIntoView();
        }
      } else if (action === "a") {
        if (!stateRef.current.runningGame) {
          onLaunchGame?.(games?.[idx]?.id);
        }
      } else if (action === "x") {
        const g = games?.[idx];
        if (g) onQuickSettings?.(g);
      } else if (action === "b") {
        onSelectGame?.(null);
      }
    } else {
      // No items — just escape to spiral
      if (action === "up")    { setZone("library");  stateRef.current.zone = "library"; }
      if (action === "down")  { setZone("settings"); stateRef.current.zone = "settings"; }
      if (action === "left")  { setZone("tools");    stateRef.current.zone = "tools"; }
      if (action === "right") { setZone("steam");    stateRef.current.zone = "steam"; }
    }
  }, [maximized, view, games, onNavigate, onSelectGame, onLaunchGame, onKillGame, onGuide, onQuickSettings]);

  useGamepad(handleInput);

  return {
    zone,
    gridIndex,
    controllerActive: controllerActive && maximized,
  };
}
