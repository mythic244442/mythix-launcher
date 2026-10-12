// controller/bus.js — the one place controller input enters the UI.
//
// Why this exists: every hook used to register its own Tauri listener inside an
// async import().then(listen). Under React StrictMode the effect cleanup runs
// before that promise resolves, so the first listener leaked and every press was
// handled twice (→ skipped entries, double launches). Now:
//
//  * ONE subscription to `gamepad:action` for the whole app (module singleton,
//    HMR-safe via a window handle).
//  * Press dedupe: an action that is already down can't press again, so a pad
//    exposed twice (or any stray duplicate event) can never double-step.
//  * Auto-repeat for held directions lives here, not in each consumer.
//  * Layered routing: each press goes to exactly ONE layer — the highest
//    priority one (on-screen keyboard > option picker > quick settings > base
//    nav). Opening an overlay automatically mutes everything under it.
//  * "guide" is global and never routed through layers.

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const REPEAT_DELAY = 380;
const REPEAT_RATE = 110;
const DIRECTIONS = new Set(["up", "down", "left", "right"]);

const pressed = new Set();
const pressedAt = new Map(); // action -> time of the press we acted on
// A second press of a held action within this window is a duplicate (same pad
// seen twice) and is dropped. Later than that, the release must have been lost
// somewhere — treat it as a fresh press instead of leaving the button "stuck".
const DUPLICATE_WINDOW = 250;
const owners = new Map(); // action -> id of the layer that got the press
const timers = new Map(); // action -> repeat timeout handle
const layers = [];
const globals = new Map(); // action -> Set<fn>
let layerSeq = 0;
let topId = null;

// ── Observable state (controller type / connection / "controller mode") ─────

let snapshot = { type: "xbox", connected: 0, active: false };
const subscribers = new Set();

function setState(patch) {
  let changed = false;
  for (const key in patch) {
    if (snapshot[key] !== patch[key]) { changed = true; break; }
  }
  if (!changed) return;
  snapshot = { ...snapshot, ...patch };
  if ("active" in patch && typeof document !== "undefined") {
    document.body.classList.toggle("controller-active", snapshot.active);
  }
  subscribers.forEach((fn) => { try { fn(); } catch (e) { console.error(e); } });
}

export function subscribe(fn) {
  subscribers.add(fn);
  return () => subscribers.delete(fn);
}

export function getSnapshot() {
  return snapshot;
}

function normalizeType(t) {
  return t === "playstation" || t === "switch" ? t : "xbox";
}

// ── Layers ──────────────────────────────────────────────────────────────────

function topLayer() {
  let best = null;
  for (const layer of layers) {
    if (!best || layer.priority > best.priority ||
        (layer.priority === best.priority && layer.seq > best.seq)) {
      best = layer;
    }
  }
  return best;
}

function safeCall(fn, ...args) {
  if (!fn) return;
  try { fn(...args); } catch (e) { console.error("[mythix] controller handler failed:", e); }
}

function onTopChanged() {
  const top = topLayer();
  const id = top ? top.id : null;
  if (id === topId) return;
  topId = id;
  // Whatever is still held belongs to the old top layer: hand it its release
  // and stop repeating, so nothing "leaks" into the new layer mid-hold.
  for (const [action, ownerId] of [...owners]) {
    if (ownerId === id) continue;
    stopRepeat(action);
    owners.delete(action);
    safeCall(layers.find((l) => l.id === ownerId)?.onRelease, action);
  }
}

/**
 * Register an input layer. Returns { update, remove }.
 *   priority  higher wins (base nav 0, quick settings 50, picker 90, keyboard 100)
 *   onPress   (action, { repeat }) => void
 *   onRelease (action) => void              — optional
 *   repeat    extra actions that auto-repeat while held (directions always do)
 */
export function addLayer({ priority = 0, onPress, onRelease, repeat = [] }) {
  const layer = { id: ++layerSeq, seq: layerSeq, priority, onPress, onRelease, repeat: new Set(repeat) };
  layers.push(layer);
  onTopChanged();
  return {
    update(patch) {
      Object.assign(layer, patch);
      if (patch.repeat) layer.repeat = new Set(patch.repeat);
    },
    remove() {
      const i = layers.indexOf(layer);
      if (i >= 0) {
        layers.splice(i, 1);
        onTopChanged();
      }
    },
  };
}

/** Handlers that run no matter which layer is on top (used for Guide). */
export function onGlobal(action, fn) {
  if (!globals.has(action)) globals.set(action, new Set());
  globals.get(action).add(fn);
  return () => globals.get(action)?.delete(fn);
}

// ── Press / release / repeat ────────────────────────────────────────────────

function stopRepeat(action) {
  clearTimeout(timers.get(action));
  timers.delete(action);
}

function startRepeat(action, layerId) {
  stopRepeat(action);
  const tick = () => {
    const layer = layers.find((l) => l.id === layerId);
    if (!layer || topId !== layerId || !pressed.has(action)) {
      stopRepeat(action);
      return;
    }
    safeCall(layer.onPress, action, { repeat: true });
    timers.set(action, setTimeout(tick, REPEAT_RATE));
  };
  timers.set(action, setTimeout(tick, REPEAT_DELAY));
}

export function press(action) {
  if (typeof action !== "string") return;
  if (pressed.has(action)) {
    if (performance.now() - (pressedAt.get(action) ?? 0) < DUPLICATE_WINDOW) return; // dedupe
    release(action); // lost release — recover
  }
  pressed.add(action);
  pressedAt.set(action, performance.now());
  setState({ active: true });

  if (globals.has(action)) {
    globals.get(action).forEach((fn) => safeCall(fn, action));
    return;
  }

  const layer = topLayer();
  if (!layer) return;
  owners.set(action, layer.id);
  safeCall(layer.onPress, action, { repeat: false });
  if (DIRECTIONS.has(action) || layer.repeat.has(action)) startRepeat(action, layer.id);
}

export function release(action) {
  if (!pressed.delete(action)) return;
  pressedAt.delete(action);
  stopRepeat(action);
  const ownerId = owners.get(action);
  owners.delete(action);
  if (ownerId != null) safeCall(layers.find((l) => l.id === ownerId)?.onRelease, action);
}

export function releaseAll() {
  [...pressed].forEach(release);
}

// ── "Controller mode" ⇄ mouse mode ──────────────────────────────────────────
// Controller input turns controller mode on (focus rings, hidden cursor, no
// hover effects). Real mouse movement turns it off. WebKit fires synthetic
// mousemoves while content scrolls under a parked cursor — those keep the same
// screen position, so they're ignored.

let lastMouse = null;

function onMouseMove(e) {
  const pos = [e.screenX, e.screenY];
  if (!snapshot.active || !lastMouse) {
    lastMouse = pos;
    return;
  }
  const dx = pos[0] - lastMouse[0];
  const dy = pos[1] - lastMouse[1];
  if (dx * dx + dy * dy > 36) {
    lastMouse = pos;
    setState({ active: false });
  }
}

function onPointerDown(e) {
  if (e.pointerType === "mouse" || e.pointerType === "touch" || e.pointerType === "pen") {
    if (snapshot.active) setState({ active: false });
  }
}

// ── Sources ─────────────────────────────────────────────────────────────────

const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

async function connectTauri() {
  // HMR / double-import guard: drop any listeners a previous instance left.
  try { window.__mythixPadUnlisten?.(); } catch (_) { /* ignore */ }
  window.__mythixPadUnlisten = null;

  const unlistens = await Promise.all([
    listen("gamepad:action", (ev) => {
      const [action, down] = Array.isArray(ev.payload) ? ev.payload : [];
      if (down) press(action); else release(action);
    }),
    listen("gamepad:type", (ev) => setState({ type: normalizeType(ev.payload) })),
    listen("gamepad:connected", (ev) => setState({ connected: Number(ev.payload) || 0 })),
  ]);
  window.__mythixPadUnlisten = () => unlistens.forEach((fn) => fn());

  invoke("get_controller_status")
    .then((s) => setState({ connected: Number(s?.connected) || 0, type: normalizeType(s?.type) }))
    .catch(() => {
      // Older backend without get_controller_status.
      invoke("get_controller_type").then((t) => setState({ type: normalizeType(t) })).catch(() => {});
    });
}

// Plain-browser dev (`npm run dev` outside Tauri): read the Gamepad API instead.
function startGamepadApiFallback() {
  let prev = {};
  const T = 0.55;
  function poll() {
    const pads = navigator.getGamepads ? [...navigator.getGamepads()].filter(Boolean) : [];
    const now = {};
    for (const gp of pads) {
      const b = (i) => !!gp.buttons[i]?.pressed;
      const ax = (i) => gp.axes[i] ?? 0;
      const state = {
        a: b(0), b: b(1), x: b(2), y: b(3), lb: b(4), rb: b(5), lt: b(6), rt: b(7),
        back: b(8), start: b(9), guide: b(16),
        up: b(12) || ax(1) < -T, down: b(13) || ax(1) > T,
        left: b(14) || ax(0) < -T, right: b(15) || ax(0) > T,
        rs_up: ax(3) < -T, rs_down: ax(3) > T, rs_left: ax(2) < -T, rs_right: ax(2) > T,
      };
      for (const k in state) now[k] = now[k] || state[k];
    }
    for (const k of new Set([...Object.keys(now), ...Object.keys(prev)])) {
      if (now[k] && !prev[k]) press(k);
      else if (!now[k] && prev[k]) release(k);
    }
    prev = now;
    setState({ connected: pads.length });
    requestAnimationFrame(poll);
  }
  requestAnimationFrame(poll);
}

if (typeof window !== "undefined") {
  window.addEventListener("mousemove", onMouseMove, { passive: true });
  window.addEventListener("pointerdown", onPointerDown, { capture: true, passive: true });
  window.addEventListener("blur", releaseAll);

  if (isTauri) {
    connectTauri().catch((e) => console.error("[mythix] gamepad bus failed to connect:", e));
  } else {
    startGamepadApiFallback();
  }

  // Devtools / test hook: __mythixPad.tap("down"), __mythixPad.press("rb") …
  window.__mythixPad = {
    press,
    release,
    tap(action) { press(action); release(action); },
    setType: (t) => setState({ type: normalizeType(t) }),
    state: () => ({ ...snapshot, pressed: [...pressed], layers: layers.map((l) => l.priority) }),
  };
}

// This module owns process-wide listeners; reload the page instead of hot-swapping it.
if (import.meta.hot) import.meta.hot.decline();
