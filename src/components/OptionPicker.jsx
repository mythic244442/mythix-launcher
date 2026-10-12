// components/OptionPicker.jsx — controller-friendly replacement for a <select>
// popup. WebKitGTK opens native GTK menus for selects, which a gamepad can't
// drive, so A on a select opens this list instead.

import React, { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useControllerState, useGamepadLayer } from "../hooks/useGamepad.js";
import { closePicker, fieldLabel, setNativeValue, useOverlays } from "../controller/overlays.js";
import { glyphsFor } from "../controller/glyphs.js";

const PAGE = 8;

function nextEnabled(options, from, delta) {
  const dir = delta < 0 ? -1 : 1;
  const target = Math.max(0, Math.min(options.length - 1, from + delta));
  for (let i = target; i >= 0 && i < options.length; i += dir) if (!options[i].disabled) return i;
  for (let i = target; i >= 0 && i < options.length; i -= dir) if (!options[i].disabled) return i;
  return from;
}

export default function OptionPicker() {
  const { picker } = useOverlays();
  if (!picker) return null;
  return <PickerDialog key={picker.id} select={picker.select} />;
}

function PickerDialog({ select }) {
  const { type } = useControllerState();
  const g = glyphsFor(type);
  const label = fieldLabel(select);
  const options = useMemo(
    () => [...select.options].map((o, i) => ({
      i,
      value: o.value,
      label: (o.textContent || o.value || "—").trim(),
      disabled: o.disabled,
    })),
    [select],
  );
  const current = select.value;
  const [index, setIndex] = useState(() => {
    const i = select.selectedIndex;
    return i >= 0 && !options[i]?.disabled ? i : nextEnabled(options, 0, 0);
  });
  const indexRef = useRef(index);
  indexRef.current = index;
  const listRef = useRef(null);

  function choose(i) {
    const option = options[i];
    if (!option || option.disabled) return;
    if (select.value !== option.value) setNativeValue(select, option.value);
    closePicker();
  }

  function step(delta) {
    setIndex((i) => nextEnabled(options, i, delta));
  }

  useGamepadLayer((action) => {
    if (!select.isConnected) {
      closePicker({ refocus: false });
      return;
    }
    switch (action) {
      case "up": step(-1); break;
      case "down": step(1); break;
      case "lb": case "lt": step(-PAGE); break;
      case "rb": case "rt": step(PAGE); break;
      case "a": case "start": choose(indexRef.current); break;
      case "b": closePicker(); break;
      default:
    }
  }, { priority: 90, repeat: ["lb", "rb", "lt", "rt"] });

  // Physical keyboard works too.
  useEffect(() => {
    const onKey = (e) => {
      if (e.key === "Escape") { e.preventDefault(); closePicker(); }
      else if (e.key === "ArrowDown") { e.preventDefault(); step(1); }
      else if (e.key === "ArrowUp") { e.preventDefault(); step(-1); }
      else if (e.key === "Enter") { e.preventDefault(); choose(indexRef.current); }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  useLayoutEffect(() => {
    listRef.current
      ?.querySelector(".picker__option--active")
      ?.scrollIntoView({ block: "nearest" });
  }, [index]);

  return (
    <div
      className="picker-backdrop"
      data-nav-skip
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) {
          e.preventDefault();
          closePicker();
        }
      }}
    >
      <div className="picker" role="listbox" aria-label={label}>
        <div className="picker__title">{label}</div>
        <div className="picker__list" ref={listRef}>
          {options.map((o) => (
            <button
              key={o.i}
              type="button"
              tabIndex={-1}
              role="option"
              aria-selected={o.value === current}
              disabled={o.disabled}
              className={[
                "picker__option",
                o.i === index && "picker__option--active",
                o.value === current && "picker__option--current",
              ].filter(Boolean).join(" ")}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => choose(o.i)}
            >
              <span className="picker__check">{o.value === current ? "✓" : ""}</span>
              <span className="picker__text">{o.label}</span>
            </button>
          ))}
        </div>
        <div className="picker__hints">
          <span className="osk__hint"><span className="controller-legend__btn">{g.a}</span><span className="osk__hint-label">Choose</span></span>
          <span className="osk__hint"><span className="controller-legend__btn">{`${g.lb}/${g.rb}`}</span><span className="osk__hint-label">Page</span></span>
          <span className="osk__hint"><span className="controller-legend__btn">{g.b}</span><span className="osk__hint-label">Cancel</span></span>
        </div>
      </div>
    </div>
  );
}
