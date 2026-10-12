// controller/focus.js — where controller focus may go and how it moves.
//
// Rules:
//  * Focus lives inside the current *scope*: the top-most open modal, else the
//    content area in the middle. The section frame (Library / Import / Tools /
//    Settings) is never a target — LB/RB own that.
//  * The game grid and list use index math (the real column count comes from
//    the grid's computed tracks, so the focused card's lift/scale transform
//    can't skew it). At their edges we fall back to geometry, which can only
//    reach other content (header, spotlight, config panel).
//  * Everything else uses beam-based geometric search with edge distances.

const FOCUSABLE = "button, a[href], input, select, textarea, [tabindex], [role='button']";
const NATIVE_FOCUSABLE = "button, a[href], input, select, textarea";
const SKIP_ANCESTORS =
  "[data-nav-skip], .titlebar, [inert], [aria-hidden='true'], .game-card__actions, .game-list__actions";
const MODALS = ".modal-backdrop, [data-nav-modal]";

export const GAME_ITEM = ".game-card, .game-list__item";

export function isNavigable(el) {
  if (!(el instanceof HTMLElement) || !el.isConnected) return false;
  if (el.disabled) return false;
  if (el.matches("input[type='hidden'], input[type='file']")) return false;
  const tabindex = el.getAttribute("tabindex");
  if (tabindex !== null && Number(tabindex) < 0) return false;
  if (tabindex === null && !el.matches(NATIVE_FOCUSABLE)) return false; // can't take focus
  if (el.closest(SKIP_ANCESTORS)) return false;
  const rect = el.getBoundingClientRect();
  if (rect.width < 2 || rect.height < 2) return false;
  const style = getComputedStyle(el);
  return style.visibility !== "hidden" && style.display !== "none";
}

function isShown(el) {
  return el.getClientRects().length > 0;
}

export function topModal() {
  const modals = [...document.querySelectorAll(MODALS)].filter(isShown);
  return modals.length ? modals[modals.length - 1] : null;
}

export function contentRoot() {
  return document.querySelector(".nav-spiral__content, .nav-layout__content");
}

export function getScope() {
  return topModal() || contentRoot() || document.querySelector(".app-shell") || document.body;
}

export function getCandidates(scope = getScope()) {
  return [...scope.querySelectorAll(FOCUSABLE)].filter(isNavigable);
}

export function focusElement(el, { scroll = true } = {}) {
  if (!el) return false;
  try {
    el.focus({ preventScroll: true });
  } catch (_) {
    el.focus();
  }
  if (scroll) {
    try {
      el.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
    } catch (_) {
      el.scrollIntoView(false);
    }
  }
  return document.activeElement === el;
}

// ── Scrolling helpers ───────────────────────────────────────────────────────

function isScrollable(el, axis = "y") {
  if (!(el instanceof HTMLElement)) return false;
  const style = getComputedStyle(el);
  const overflow = axis === "y" ? style.overflowY : style.overflowX;
  if (!/(auto|scroll)/.test(overflow)) return false;
  return axis === "y" ? el.scrollHeight > el.clientHeight + 1 : el.scrollWidth > el.clientWidth + 1;
}

export function scrollParent(el, axis = "y") {
  for (let p = el?.parentElement; p; p = p.parentElement) {
    if (isScrollable(p, axis)) return p;
  }
  return null;
}

/** Biggest scrollable area near the top of `root` (breadth-first, shallow). */
export function findScroller(root, axis = "y") {
  if (!root) return null;
  let best = null;
  let queue = [root];
  for (let depth = 0; depth < 6 && queue.length; depth++) {
    const next = [];
    for (const node of queue) {
      if (isScrollable(node, axis) &&
          (!best || node.clientHeight * node.clientWidth > best.clientHeight * best.clientWidth)) {
        best = node;
      }
      next.push(...node.children);
    }
    queue = next;
  }
  return best;
}

// Off-screen recovery only applies when the *user* scrolled focus out of view
// (right stick, page jump, mouse wheel) — not while our own smooth scroll is
// still catching up to a freshly focused element.
let scrolledAwayFrom = null;

export function noteUserScroll() {
  scrolledAwayFrom = document.activeElement;
}

if (typeof window !== "undefined") {
  window.addEventListener("wheel", noteUserScroll, { passive: true, capture: true });
}

function visibleWithin(el, container) {
  const r = el.getBoundingClientRect();
  const c = container
    ? container.getBoundingClientRect()
    : { top: 0, left: 0, bottom: window.innerHeight, right: window.innerWidth };
  return r.bottom > c.top + 4 && r.top < c.bottom - 4 && r.right > c.left + 4 && r.left < c.right - 4;
}

// ── Grid / list index navigation ────────────────────────────────────────────

export function gridColumns(grid, items) {
  const tracks = getComputedStyle(grid).gridTemplateColumns;
  if (tracks && tracks !== "none") {
    const count = tracks.trim().split(/\s+/).filter((t) => /^[\d.]+px$/.test(t)).length;
    if (count > 0) return count;
  }
  // offsetTop ignores CSS transforms, unlike getBoundingClientRect.
  const top = items[0]?.offsetTop ?? 0;
  let cols = 0;
  for (const it of items) {
    if (Math.abs(it.offsetTop - top) < 2) cols++;
    else break;
  }
  return Math.max(1, cols);
}

function moveInGrid(grid, current, dir) {
  const items = [...grid.children];
  const idx = items.indexOf(current);
  if (idx < 0) return null;
  const cols = gridColumns(grid, items);
  const row = Math.floor(idx / cols);
  const lastRow = Math.floor((items.length - 1) / cols);
  let next = -1;
  if (dir === "left" && idx % cols > 0) next = idx - 1;
  else if (dir === "right" && idx % cols < cols - 1 && idx + 1 < items.length) next = idx + 1;
  else if (dir === "up" && row > 0) next = idx - cols;
  else if (dir === "down" && row < lastRow) next = Math.min(idx + cols, items.length - 1);
  return next >= 0 ? items[next] : null;
}

function moveInList(list, current, dir) {
  const items = [...list.children];
  const idx = items.indexOf(current);
  if (idx < 0) return null;
  if (dir === "up" && idx > 0) return items[idx - 1];
  if (dir === "down" && idx < items.length - 1) return items[idx + 1];
  return null;
}

// ── Geometric search ────────────────────────────────────────────────────────

export function findInDirection(from, dir, candidates) {
  const a = from.getBoundingClientRect();
  const ax = a.left + a.width / 2;
  const ay = a.top + a.height / 2;
  let best = null;
  let bestScore = Infinity;

  for (const el of candidates) {
    if (el === from || el.contains(from) || from.contains(el)) continue;
    const b = el.getBoundingClientRect();
    const bx = b.left + b.width / 2;
    const by = b.top + b.height / 2;

    // Must lie past the current element's centre in that direction — this is
    // what stops "down" from picking a neighbour in the same row.
    let primary;
    let ortho;
    let align;
    switch (dir) {
      case "down":
        if (b.top < ay) continue;
        primary = Math.max(0, b.top - a.bottom);
        ortho = Math.max(0, a.left - b.right, b.left - a.right);
        align = Math.abs(bx - ax);
        break;
      case "up":
        if (b.bottom > ay) continue;
        primary = Math.max(0, a.top - b.bottom);
        ortho = Math.max(0, a.left - b.right, b.left - a.right);
        align = Math.abs(bx - ax);
        break;
      case "right":
        if (b.left < ax) continue;
        primary = Math.max(0, b.left - a.right);
        ortho = Math.max(0, a.top - b.bottom, b.top - a.bottom);
        align = Math.abs(by - ay);
        break;
      case "left":
        if (b.right > ax) continue;
        primary = Math.max(0, a.left - b.right);
        ortho = Math.max(0, a.top - b.bottom, b.top - a.bottom);
        align = Math.abs(by - ay);
        break;
      default:
        continue;
    }
    // Out-of-beam targets must still be roughly in the pressed direction.
    if (ortho > 0 && ortho > primary * 2 + 48) continue;
    const score = primary + ortho * 2.5 + align * 0.05;
    if (score < bestScore) {
      bestScore = score;
      best = el;
    }
  }
  return best;
}

// ── Entry points ────────────────────────────────────────────────────────────

/** Remember something stable about the focused element (survives re-render). */
export function describeFocus(scope = getScope()) {
  const el = document.activeElement;
  if (!el || el === document.body || !scope.contains(el)) return null;
  if (el.dataset?.gameId && el.matches(GAME_ITEM)) return { gameId: el.dataset.gameId };
  const index = getCandidates(scope).indexOf(el);
  return index >= 0 ? { index } : null;
}

export function resolveFocus(memory, candidates) {
  if (!memory) return null;
  if (memory.gameId) {
    return candidates.find((el) => el.dataset?.gameId === memory.gameId && el.matches(GAME_ITEM)) || null;
  }
  if (memory.index != null) return candidates[memory.index] || null;
  return null;
}

/** Where focus should land when entering a scope with nothing focused. */
export function pickEntry(scope, candidates, memory) {
  if (!candidates.length) return null;
  const remembered = resolveFocus(memory, candidates);
  if (remembered) return remembered;
  if (scope.matches?.(MODALS)) {
    return (
      candidates.find((el) => el.matches("[autofocus], input, textarea, select")) ||
      candidates.find((el) => !el.matches("[data-nav-close], .modal__close")) ||
      candidates[0]
    );
  }
  return (
    candidates.find((el) => el.matches(GAME_ITEM)) ||
    candidates.find((el) => !el.closest(".content-header")) ||
    candidates[0]
  );
}

/** Land focus inside the scope without moving (nothing focused yet). */
export function landFocus(memory) {
  const scope = getScope();
  const candidates = getCandidates(scope);
  return focusElement(pickEntry(scope, candidates, memory));
}

/** Focus the candidate closest to where something used to be. */
export function rescueFocus(rect) {
  const scope = getScope();
  const candidates = getCandidates(scope);
  if (!candidates.length) return false;
  if (!rect) return focusElement(pickEntry(scope, candidates));
  const x = rect.left + rect.width / 2;
  const y = rect.top + rect.height / 2;
  let best = null;
  let bestDist = Infinity;
  for (const el of candidates) {
    const r = el.getBoundingClientRect();
    const d = Math.hypot(r.left + r.width / 2 - x, r.top + r.height / 2 - y);
    if (d < bestDist) {
      bestDist = d;
      best = el;
    }
  }
  return focusElement(best);
}

export function moveFocus(dir) {
  const scope = getScope();
  const candidates = getCandidates(scope);
  if (!candidates.length) return false;

  const current = document.activeElement;
  const usable = current && current !== document.body && current.isConnected && scope.contains(current);
  if (!usable) {
    return focusElement(pickEntry(scope, candidates));
  }
  if (!candidates.includes(current)) {
    // e.g. a button that just went disabled ("Importing…"): still a fine
    // place to navigate *from*.
    return focusElement(findInDirection(current, dir, candidates));
  }

  // After right-stick scrolling the focused element may be off-screen: pick up
  // from what's visible instead of yanking the view back.
  const userScrolled = scrolledAwayFrom === current;
  scrolledAwayFrom = null;
  const scroller = userScrolled ? scrollParent(current) : null;
  if (scroller && !visibleWithin(current, scroller)) {
    const visible = candidates.filter((el) => scroller.contains(el) && visibleWithin(el, scroller));
    if (visible.length) {
      return focusElement(dir === "up" || dir === "left" ? visible[visible.length - 1] : visible[0]);
    }
  }

  const grid = current.parentElement?.matches(".game-grid") ? current.parentElement : null;
  if (grid) {
    const next = moveInGrid(grid, current, dir);
    if (next) return isNavigable(next) && focusElement(next);
    return focusElement(findInDirection(current, dir, candidates.filter((el) => !grid.contains(el))));
  }

  const list = current.parentElement?.matches(".game-list") ? current.parentElement : null;
  if (list) {
    const next = moveInList(list, current, dir);
    if (next) return isNavigable(next) && focusElement(next);
    return focusElement(findInDirection(current, dir, candidates.filter((el) => !list.contains(el))));
  }

  return focusElement(findInDirection(current, dir, candidates));
}

/**
 * LT/RT: jump about a screenful. Grid/list move by whole rows (keeping the
 * column); anything else scrolls a page and focuses what's now under the old
 * focus position.
 */
export function pageFocus(direction) {
  const scope = getScope();
  const candidates = getCandidates(scope);
  const current = candidates.includes(document.activeElement) ? document.activeElement : null;
  if (!current) return landFocus();

  const container = current.parentElement;
  if (container?.matches(".game-grid, .game-list")) {
    const items = [...container.children];
    const idx = items.indexOf(current);
    const cols = container.matches(".game-grid") ? gridColumns(container, items) : 1;
    const rowHeight = items.length > cols
      ? Math.max(1, items[cols].offsetTop - items[0].offsetTop)
      : Math.max(1, current.offsetHeight);
    const viewport = scrollParent(current)?.clientHeight ?? window.innerHeight;
    const rows = Math.max(1, Math.floor(viewport / rowHeight));
    let next = idx + direction * rows * cols;
    if (next < 0) next = idx % cols;
    if (next >= items.length) {
      const lastRowStart = Math.floor((items.length - 1) / cols) * cols;
      next = Math.min(lastRowStart + (idx % cols), items.length - 1);
    }
    return next !== idx && isNavigable(items[next]) && focusElement(items[next]);
  }

  const scroller = scrollParent(current) || findScroller(scope);
  if (!scroller) return focusElement(findInDirection(current, direction > 0 ? "down" : "up", candidates));
  const ref = current.getBoundingClientRect();
  const x = ref.left + ref.width / 2;
  const y = ref.top + ref.height / 2;
  scroller.scrollBy({ top: direction * scroller.clientHeight * 0.85, behavior: "auto" });

  let best = null;
  let bestDist = Infinity;
  for (const el of candidates) {
    if (!scroller.contains(el) || !visibleWithin(el, scroller)) continue;
    const r = el.getBoundingClientRect();
    const d = Math.hypot(r.left + r.width / 2 - x, r.top + r.height / 2 - y);
    if (d < bestDist) {
      bestDist = d;
      best = el;
    }
  }
  if (!best || best === current) {
    best = findInDirection(current, direction > 0 ? "down" : "up", candidates) || best;
  }
  return focusElement(best);
}
