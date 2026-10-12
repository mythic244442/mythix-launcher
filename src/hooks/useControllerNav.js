// hooks/useControllerNav.js — base controller layer for the whole launcher.
//
//   LB / RB        switch section (Library → Import → Settings → Tools, clockwise)
//   D-pad / stick  move focus inside the content area only — never onto the frame
//   A              launch game · open keyboard on text fields · picker on selects · press buttons
//   B              close dialog / config panel
//   X              quick settings for the focused game
//   Y              stop the running game
//   Start          open the focused game's config panel
//   Back / View    jump to the section's search box with the keyboard open
//   LT / RT        page up / down
//   Right stick    scroll
//   Guide          hide / show (global, handled even over overlays)

import { useEffect, useRef } from "react";
import { getSnapshot, onGlobal } from "../controller/bus.js";
import { useGamepadLayer, useControllerState } from "./useGamepad.js";
import {
  GAME_ITEM,
  contentRoot,
  describeFocus,
  findScroller,
  focusElement,
  getCandidates,
  getScope,
  isNavigable,
  landFocus,
  moveFocus,
  noteUserScroll,
  pageFocus,
  pickEntry,
  rescueFocus,
  scrollParent,
  topModal,
} from "../controller/focus.js";
import { getOverlays, isTextField, openKeyboard, openPicker } from "../controller/overlays.js";
import { neighborSection } from "../controller/sections.js";

const DIRECTIONS = new Set(["up", "down", "left", "right"]);
const SCROLL_START = 900; // px/s
const SCROLL_MAX = 2400; // px/s after ~1s held

function waitFor(find, onFound, timeout = 800) {
  const started = performance.now();
  const attempt = () => {
    const el = find();
    if (el) {
      onFound(el);
      return;
    }
    if (performance.now() - started < timeout) requestAnimationFrame(attempt);
  };
  requestAnimationFrame(attempt);
}

function gameElement(gameId) {
  return document.querySelector(`.game-card[data-game-id="${CSS.escape(gameId)}"], .game-list__item[data-game-id="${CSS.escape(gameId)}"]`);
}

export function useControllerNav({
  view,
  onNavigate,
  games,
  onLaunchGame,
  onKillGame,
  onGuide,
  onQuickSettings,
  runningGame,
}) {
  const props = useRef({});
  props.current = { view, onNavigate, games, onLaunchGame, onKillGame, onGuide, onQuickSettings, runningGame };

  const memory = useRef({}); // section -> focus snapshot to restore when coming back
  const landToken = useRef(0); // bump to cancel a pending landing
  const landingUntil = useRef(0); // while a landing is pending, focus rescue stands down
  const lastActivated = useRef(null); // element A last pressed (to restore after a dialog)
  const scrolling = useRef(null);

  const { active: controllerActive, type: controllerType, connected } = useControllerState();

  // Guide works everywhere, even with the keyboard or quick settings open.
  useEffect(() => onGlobal("guide", () => props.current.onGuide?.()), []);

  useEffect(() => () => stopScroll(), []);

  // Focus rescue: when the focused element is removed by a re-render (a game
  // removed, a button swapped out while loading, a panel closed by mouse), put
  // focus back near where it was instead of letting it fall to <body>.
  useEffect(() => {
    const last = { el: null, rect: null };
    const onFocusIn = (e) => {
      const el = e.target;
      if (el instanceof HTMLElement && !el.closest("[data-nav-skip]")) {
        last.el = el;
        last.rect = el.getBoundingClientRect();
      }
    };
    let queued = false;
    const observer = new MutationObserver(() => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(() => {
        queued = false;
        const active = document.activeElement;
        if (!getSnapshot().active || (active && active !== document.body)) return;
        if (performance.now() < landingUntil.current) return;
        if (!last.el || last.el.isConnected) return;
        const { keyboard, picker } = getOverlays();
        if (keyboard || picker || !document.hasFocus()) return;
        rescueFocus(last.rect);
      });
    });
    document.addEventListener("focusin", onFocusIn);
    observer.observe(document.body, { childList: true, subtree: true });
    return () => {
      document.removeEventListener("focusin", onFocusIn);
      observer.disconnect();
    };
  }, []);

  // ── Landing focus after a section switch ──────────────────────────────────
  function landWhenReady(targetView) {
    const token = ++landToken.current;
    const started = performance.now();
    landingUntil.current = started + 2500;
    const attempt = () => {
      if (token !== landToken.current) return;
      const scope = getScope();
      const loading = !!scope.querySelector(".view-loading");
      const candidates = loading ? [] : getCandidates(scope);
      const entry = pickEntry(scope, candidates, memory.current[targetView]);
      const waited = performance.now() - started;
      // Library cards render a beat after the header; wait for them so we land
      // on a game rather than the search box.
      const ready = entry && (targetView !== "library" || entry.matches(GAME_ITEM) || waited > 600);
      if (ready) {
        landingUntil.current = 0;
        focusElement(entry);
        return;
      }
      if (waited < 2500) requestAnimationFrame(attempt);
      else landingUntil.current = 0;
    };
    requestAnimationFrame(attempt);
  }

  function cancelLanding() {
    landToken.current++;
    landingUntil.current = 0;
  }

  const shownView = useRef(view);
  useEffect(() => {
    if (shownView.current === view) return;
    shownView.current = view;
    if (controllerActive) landWhenReady(view);
  }, [view, controllerActive]);

  // ── Right-stick scrolling ─────────────────────────────────────────────────
  function startScroll(action) {
    const axis = action === "rs_left" || action === "rs_right" ? "x" : "y";
    const sign = action === "rs_up" || action === "rs_left" ? -1 : 1;
    const scope = getScope();
    const focused = document.activeElement;
    const el = (scope.contains(focused) && scrollParent(focused, axis)) || findScroller(scope, axis);
    if (!el) return;
    stopScroll();
    noteUserScroll();
    const state = { action, raf: 0, started: performance.now(), last: performance.now() };
    const step = (now) => {
      if (scrolling.current !== state) return;
      const dt = Math.min(0.05, Math.max(0, now - state.last) / 1000);
      state.last = now;
      const ramp = Math.min(1, (now - state.started) / 1000);
      const delta = sign * (SCROLL_START + (SCROLL_MAX - SCROLL_START) * ramp) * dt;
      if (axis === "y") el.scrollTop += delta;
      else el.scrollLeft += delta;
      state.raf = requestAnimationFrame(step);
    };
    scrolling.current = state;
    state.raf = requestAnimationFrame(step);
  }

  function stopScroll(action) {
    const state = scrolling.current;
    if (!state || (action && state.action !== action)) return;
    cancelAnimationFrame(state.raf);
    scrolling.current = null;
  }

  // ── B: back out of whatever is on top ─────────────────────────────────────
  function goBack() {
    const modal = topModal();
    if (modal) {
      modal.querySelector("[data-nav-close], .modal__close")?.click();
      const opener = lastActivated.current;
      waitFor(
        () => (!topModal() ? true : null),
        () => {
          const active = document.activeElement;
          if ((!active || active === document.body) && opener?.isConnected && isNavigable(opener)) {
            focusElement(opener);
          }
        },
      );
      return;
    }

    const panel = document.querySelector(".config-panel");
    if (panel) {
      const gameId = panel.dataset.panelGameId;
      const focusWasInPanel = panel.contains(document.activeElement);
      const close = panel.querySelector("[data-nav-close]") || panel.querySelector(".config-panel__header .btn--ghost");
      close?.click();
      if (gameId) {
        waitFor(
          () => (!document.querySelector(".config-panel") ? gameElement(gameId) : null),
          (card) => {
            const active = document.activeElement;
            if (focusWasInPanel || !active || active === document.body) focusElement(card);
          },
        );
      }
    }
  }

  // ── The layer ─────────────────────────────────────────────────────────────
  function handlePress(action, { repeat }) {
    const p = props.current;

    if (action.startsWith("rs_")) {
      if (!repeat) startScroll(action);
      return;
    }

    if (DIRECTIONS.has(action)) {
      cancelLanding(); // user took over
      moveFocus(action);
      return;
    }

    if (action === "lb" || action === "rb") {
      if (topModal()) return; // don't pull the section out from under a dialog
      memory.current[p.view] = describeFocus(contentRoot() || undefined);
      p.onNavigate(neighborSection(p.view, action === "rb" ? 1 : -1));
      return;
    }

    if (action === "lt" || action === "rt") {
      cancelLanding();
      pageFocus(action === "rt" ? 1 : -1);
      return;
    }

    const scope = getScope();
    const active = document.activeElement;
    const focused = active && active !== document.body && scope.contains(active) ? active : null;

    switch (action) {
      case "a": {
        if (!focused) {
          cancelLanding();
          landFocus(memory.current[p.view]);
          return;
        }
        if (isTextField(focused)) {
          openKeyboard(focused);
          return;
        }
        if (focused instanceof HTMLSelectElement) {
          openPicker(focused);
          return;
        }
        const gameId = focused.matches(GAME_ITEM) ? focused.dataset.gameId : null;
        if (gameId && !p.runningGame) {
          p.onLaunchGame?.(gameId);
          return;
        }
        lastActivated.current = focused;
        focused.click();
        return;
      }

      case "b":
        goBack();
        return;

      case "x": {
        const holder = focused?.closest("[data-game-id], [data-panel-game-id]");
        const id = holder?.dataset.gameId || holder?.dataset.panelGameId;
        const game = id && p.games?.find((g) => g.id === id);
        if (game) p.onQuickSettings?.(game);
        return;
      }

      case "y":
        if (p.runningGame) p.onKillGame?.();
        return;

      case "start": {
        if (!focused?.matches(GAME_ITEM)) return;
        const gameId = focused.dataset.gameId;
        focused.click(); // selects → opens the config panel
        waitFor(
          () => document.querySelector(`.config-panel[data-panel-game-id="${CSS.escape(gameId)}"]`),
          (panel) => {
            const inBody = getCandidates(panel).filter((el) => el.closest(".config-panel__body"));
            focusElement(inBody[0] || getCandidates(panel)[0]);
          },
        );
        return;
      }

      case "back": {
        const input = scope.querySelector(".content-header input");
        if (input && isTextField(input) && isNavigable(input)) {
          focusElement(input);
          openKeyboard(input);
        }
        return;
      }

      default:
    }
  }

  function handleRelease(action) {
    if (action.startsWith("rs_")) stopScroll(action);
  }

  useGamepadLayer(handlePress, { priority: 0, onRelease: handleRelease });

  return { controllerActive, controllerType, connected };
}
