// components/OnScreenKeyboard.jsx — controller-driven keyboard for text fields.
//
// Opens when A is pressed on an input/textarea. DOM focus stays on the field
// the whole time (keys never take focus), so a physical keyboard keeps working
// alongside it and React's onChange fires exactly like real typing.
//
//   D-pad / stick  pick a key          A      press it
//   X              backspace (repeats)  Y      space
//   LB / RB        move the cursor      LT     shift      RT  caps lock
//   Start          done                 B      close

import React, { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useControllerState, useGamepadLayer } from "../hooks/useGamepad.js";
import { closeKeyboard, fieldLabel, setNativeValue, useOverlays } from "../controller/overlays.js";
import { glyphsFor } from "../controller/glyphs.js";
import { scrollParent } from "../controller/focus.js";

const ch = (base, shifted, w = 1) => ({ id: `k${base}`, kind: "char", base, shifted: shifted ?? base.toUpperCase(), w });
const fn = (kind, label, w, id = kind) => ({ id, kind, label, w });

// US layout, every row 15 units wide.
const ROWS = [
  [ch("`", "~"), ch("1", "!"), ch("2", "@"), ch("3", "#"), ch("4", "$"), ch("5", "%"), ch("6", "^"),
    ch("7", "&"), ch("8", "*"), ch("9", "("), ch("0", ")"), ch("-", "_"), ch("=", "+"), fn("backspace", "⌫", 2)],
  [fn("caps", "⇪ Caps", 1.5), ch("q"), ch("w"), ch("e"), ch("r"), ch("t"), ch("y"), ch("u"), ch("i"),
    ch("o"), ch("p"), ch("[", "{"), ch("]", "}"), ch("\\", "|", 1.5)],
  [fn("clear", "Clear", 1.75), ch("a"), ch("s"), ch("d"), ch("f"), ch("g"), ch("h"), ch("j"), ch("k"),
    ch("l"), ch(";", ":"), ch("'", "\""), fn("enter", "↵", 2.25)],
  [fn("shift", "⇧ Shift", 2.25), ch("z"), ch("x"), ch("c"), ch("v"), ch("b"), ch("n"), ch("m"),
    ch(",", "<"), ch(".", ">"), ch("/", "?"), fn("shift", "⇧ Shift", 2.75, "shift2")],
  [fn("left", "◀", 2.5), fn("space", "Space", 7.5), fn("right", "▶", 2.5), fn("done", "Done", 2.5)],
];

// Key centres (in units) so up/down can pick the key physically above/below.
const LAYOUT = ROWS.map((row) => {
  let x = 0;
  return row.map((key) => {
    const placed = { ...key, cx: x + key.w / 2 };
    x += key.w;
    return placed;
  });
});

const START = { r: 1, c: 1 }; // "q"
const ICON_KEYS = new Set(["backspace", "enter", "left", "right"]);

function readField(el) {
  const value = el.value ?? "";
  let start = value.length;
  let end = value.length;
  try {
    if (typeof el.selectionStart === "number") {
      start = el.selectionStart;
      end = el.selectionEnd ?? start;
    }
  } catch (_) { /* inputs like type=number have no selection API */ }
  return { value, start, end };
}

export default function OnScreenKeyboard() {
  const { keyboard } = useOverlays();
  if (!keyboard) return null;
  return <KeyboardSheet key={keyboard.id} target={keyboard.target} />;
}

function KeyboardSheet({ target }) {
  const { type } = useControllerState();
  const g = glyphsFor(type);
  const isTextarea = target instanceof HTMLTextAreaElement;
  const label = fieldLabel(target);
  const placeholder = target.getAttribute("placeholder") || "";

  const [sel, setSel] = useState(START);
  const [shift, setShift] = useState(false);
  const [caps, setCaps] = useState(false);
  const [armedClear, setArmedClear] = useState(false);
  const [field, setField] = useState(() => readField(target));

  const goalX = useRef(LAYOUT[START.r][START.c].cx);
  const live = useRef({});
  live.current = { sel, shift, caps, armedClear };

  const sheetRef = useRef(null);
  const previewRef = useRef(null);
  const caretRef = useRef(null);

  const sync = useCallback(() => setField(readField(target)), [target]);

  // ── Editing primitives (all go through the native setter → onChange) ─────
  const commit = useCallback((value, caret) => {
    if (value !== target.value) setNativeValue(target, value);
    try { target.setSelectionRange(caret, caret); } catch (_) { /* ignore */ }
    sync();
  }, [target, sync]);

  const insert = useCallback((text) => {
    const { value, start, end } = readField(target);
    const max = target.maxLength > 0 ? target.maxLength : Infinity;
    const room = Math.max(0, max - (value.length - (end - start)));
    const piece = text.slice(0, room);
    if (!piece && start === end) return;
    commit(value.slice(0, start) + piece + value.slice(end), start + piece.length);
  }, [target, commit]);

  const backspace = useCallback(() => {
    const { value, start, end } = readField(target);
    if (start !== end) commit(value.slice(0, start) + value.slice(end), start);
    else if (start > 0) commit(value.slice(0, start - 1) + value.slice(start), start - 1);
  }, [target, commit]);

  const moveCaret = useCallback((delta) => {
    const { value, start, end } = readField(target);
    const pos = start !== end
      ? (delta < 0 ? start : end)
      : Math.max(0, Math.min(value.length, start + delta));
    try { target.setSelectionRange(pos, pos); } catch (_) { /* ignore */ }
    sync();
  }, [target, sync]);

  // ── Key activation ───────────────────────────────────────────────────────
  const activate = useCallback((key) => {
    const { shift: sh, caps: cp, armedClear: armed } = live.current;
    if (key.kind !== "clear" && armed) setArmedClear(false);
    switch (key.kind) {
      case "char": {
        const letter = /^[a-z]$/.test(key.base);
        const upper = letter ? cp !== sh : sh;
        insert(upper ? key.shifted : key.base);
        if (sh) setShift(false);
        break;
      }
      case "space": insert(" "); break;
      case "backspace": backspace(); break;
      case "enter":
        if (isTextarea) insert("\n");
        else closeKeyboard();
        break;
      case "shift": setShift((s) => !s); break;
      case "caps": setCaps((c) => !c); break;
      case "left": moveCaret(-1); break;
      case "right": moveCaret(1); break;
      case "clear":
        if (armed) {
          commit("", 0);
          setArmedClear(false);
        } else {
          setArmedClear(true);
        }
        break;
      case "done": closeKeyboard(); break;
      default:
    }
  }, [insert, backspace, moveCaret, commit, isTextarea]);

  const moveSelection = useCallback((dir) => {
    setSel(({ r, c }) => {
      if (dir === "left" || dir === "right") {
        const row = LAYOUT[r];
        const nc = (c + (dir === "right" ? 1 : -1) + row.length) % row.length;
        goalX.current = row[nc].cx;
        return { r, c: nc };
      }
      const nr = r + (dir === "down" ? 1 : -1);
      if (nr < 0 || nr >= LAYOUT.length) return { r, c };
      let best = 0;
      let bestDist = Infinity;
      LAYOUT[nr].forEach((key, i) => {
        const d = Math.abs(key.cx - goalX.current);
        if (d < bestDist) { bestDist = d; best = i; }
      });
      return { r: nr, c: best };
    });
  }, []);

  // ── Controller layer (sits above everything else) ─────────────────────────
  useGamepadLayer((action) => {
    if (!target.isConnected) {
      closeKeyboard({ refocus: false });
      return;
    }
    switch (action) {
      case "up": case "down": case "left": case "right": moveSelection(action); break;
      case "a": {
        const { r, c } = live.current.sel;
        activate(LAYOUT[r][c]);
        break;
      }
      case "x": backspace(); break;
      case "y": insert(" "); break;
      case "lb": moveCaret(-1); break;
      case "rb": moveCaret(1); break;
      case "lt": case "back": setShift((s) => !s); break;
      case "rt": setCaps((c) => !c); break;
      case "start": case "b": closeKeyboard(); break;
      default:
    }
  }, { priority: 100, repeat: ["x", "y", "lb", "rb"] });

  // ── Lifecycle: focus, caret, staying in sync, closing on blur/detach ──────
  useLayoutEffect(() => {
    target.focus({ preventScroll: true });
    const len = (target.value ?? "").length;
    try { target.setSelectionRange(len, len); } catch (_) { /* ignore */ }
    sync();

    // Keep the field visible above the sheet.
    const sheet = sheetRef.current;
    if (sheet) {
      const limit = window.innerHeight - sheet.offsetHeight - 24;
      const rect = target.getBoundingClientRect();
      const scroller = scrollParent(target);
      if (scroller && rect.bottom > limit) scroller.scrollTop += rect.bottom - limit;
      else if (scroller && rect.top < 0) scroller.scrollTop += rect.top - 24;
    }
  }, [target, sync]);

  useEffect(() => {
    const onChange = () => sync();
    const onBlur = () => {
      // Clicked somewhere else with the mouse → close without stealing focus back.
      setTimeout(() => {
        if (document.activeElement !== target) closeKeyboard({ refocus: false });
      }, 0);
    };
    const onKey = (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        closeKeyboard();
      }
    };
    const detachCheck = setInterval(() => {
      if (!target.isConnected) closeKeyboard({ refocus: false });
    }, 400);
    target.addEventListener("input", onChange);
    target.addEventListener("blur", onBlur);
    target.addEventListener("keydown", onKey);
    document.addEventListener("selectionchange", onChange);
    return () => {
      clearInterval(detachCheck);
      target.removeEventListener("input", onChange);
      target.removeEventListener("blur", onBlur);
      target.removeEventListener("keydown", onKey);
      document.removeEventListener("selectionchange", onChange);
    };
  }, [target, sync]);

  useEffect(() => {
    if (!armedClear) return undefined;
    const t = setTimeout(() => setArmedClear(false), 2000);
    return () => clearTimeout(t);
  }, [armedClear]);

  // Keep the caret visible in the preview line.
  useLayoutEffect(() => {
    const preview = previewRef.current;
    const caret = caretRef.current;
    if (!preview || !caret) return;
    const left = caret.offsetLeft - preview.offsetLeft;
    if (left < preview.scrollLeft + 24) preview.scrollLeft = Math.max(0, left - 24);
    else if (left > preview.scrollLeft + preview.clientWidth - 24) preview.scrollLeft = left - preview.clientWidth + 48;
    const top = caret.offsetTop - preview.offsetTop;
    if (top < preview.scrollTop) preview.scrollTop = top;
    else if (top > preview.scrollTop + preview.clientHeight - 20) preview.scrollTop = top - preview.clientHeight + 24;
  }, [field]);

  const { value, start, end } = field;
  const upperLetters = caps !== shift;

  function keyLabel(key) {
    if (key.kind !== "char") {
      if (key.kind === "clear" && armedClear) return "Clear?";
      return key.label;
    }
    const letter = /^[a-z]$/.test(key.base);
    if (letter) return upperLetters ? key.shifted : key.base;
    return shift ? key.shifted : key.base;
  }

  return (
    <div className="osk" data-nav-skip role="dialog" aria-label="On-screen keyboard">
      <div className="osk__sheet" ref={sheetRef}>
        <div className="osk__head">
          <span className="osk__label">{label}</span>
          {(shift || caps) && (
            <span className="osk__mode">{caps ? "CAPS" : "SHIFT"}</span>
          )}
        </div>
        <div className={`osk__preview ${isTextarea ? "osk__preview--multi" : ""}`} ref={previewRef}>
          {value ? (
            <>
              <span>{value.slice(0, start)}</span>
              {end > start && <span className="osk__selection">{value.slice(start, end)}</span>}
              <span className="osk__caret" ref={caretRef} />
              <span>{value.slice(end)}</span>
            </>
          ) : (
            <>
              <span className="osk__caret" ref={caretRef} />
              <span className="osk__placeholder">{placeholder}</span>
            </>
          )}
        </div>

        <div className="osk__keys">
          {LAYOUT.map((row, r) => (
            <div className="osk__row" key={r}>
              {row.map((key, c) => {
                const selected = sel.r === r && sel.c === c;
                const on = (key.kind === "shift" && shift) || (key.kind === "caps" && caps)
                  || (key.kind === "clear" && armedClear);
                return (
                  <button
                    key={key.id}
                    type="button"
                    tabIndex={-1}
                    className={[
                      "osk__key",
                      key.kind !== "char" && "osk__key--fn",
                      ICON_KEYS.has(key.kind) && "osk__key--icon",
                      key.kind === "done" && "osk__key--done",
                      key.kind === "clear" && "osk__key--danger",
                      on && "osk__key--on",
                      selected && "osk__key--selected",
                    ].filter(Boolean).join(" ")}
                    style={{ flexGrow: key.w }}
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => {
                      setSel({ r, c });
                      goalX.current = key.cx;
                      activate(key);
                    }}
                  >
                    {keyLabel(key)}
                  </button>
                );
              })}
            </div>
          ))}
        </div>

        <div className="osk__hints">
          <Hint btn={g.a} label="Type" />
          <Hint btn={g.x} label="⌫" />
          <Hint btn={g.y} label="Space" />
          <Hint btn={`${g.lb}/${g.rb}`} label="Cursor" />
          <Hint btn={g.lt} label="Shift" />
          <Hint btn={g.rt} label="Caps" />
          <Hint btn={g.start} label="Done" />
          <Hint btn={g.b} label="Close" />
        </div>
      </div>
    </div>
  );
}

function Hint({ btn, label }) {
  return (
    <span className="osk__hint">
      <span className="controller-legend__btn">{btn}</span>
      <span className="osk__hint-label">{label}</span>
    </span>
  );
}
