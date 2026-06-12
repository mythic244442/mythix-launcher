import React, { useState, useEffect, useRef } from "react";

// Internal representation uses an array of {key, value} so editing keys
// doesn't destroy the entry on every keystroke.
function toEntries(obj) {
  return Object.entries(obj ?? {}).map(([k, v], i) => ({ id: i, key: k, value: v }));
}

let _nextId = 1000;

export default function EnvVarsEditor({ value, onChange }) {
  const [rows, setRows] = useState(() => toEntries(value));
  const skipSync = useRef(false);

  // Sync from parent when value changes externally
  useEffect(() => {
    if (skipSync.current) {
      skipSync.current = false;
      return;
    }
    setRows(toEntries(value));
  }, [value]);

  function commit(newRows) {
    setRows(newRows);
    // Build object from rows, skipping empty keys
    const obj = {};
    for (const r of newRows) {
      if (r.key.trim()) obj[r.key] = r.value;
    }
    skipSync.current = true;
    onChange(obj);
  }

  function updateRow(id, field, val) {
    commit(rows.map((r) => r.id === id ? { ...r, [field]: val } : r));
  }

  function removeRow(id) {
    commit(rows.filter((r) => r.id !== id));
  }

  function addRow() {
    const id = ++_nextId;
    const newRows = [...rows, { id, key: "", value: "" }];
    setRows(newRows);
  }

  return (
    <div className="env-editor">
      {rows.map((r) => (
        <div key={r.id} className="env-row">
          <input className="form-input" value={r.key}
            onChange={(e) => updateRow(r.id, "key", e.target.value)}
            placeholder="KEY" spellCheck={false}
            style={{ flex:"0 0 38%", fontFamily:"var(--font-mono)", fontSize:11 }} />
          <input className="form-input" value={r.value}
            onChange={(e) => updateRow(r.id, "value", e.target.value)}
            placeholder="value" spellCheck={false}
            style={{ fontFamily:"var(--font-mono)", fontSize:11 }} />
          <button className="btn btn--ghost btn--icon btn--sm"
            onClick={() => removeRow(r.id)}
            style={{ color:"var(--coral)", flexShrink:0 }}>✕</button>
        </div>
      ))}
      <button className="env-add-btn" onClick={addRow}>
        <span>＋</span> Add variable
      </button>
    </div>
  );
}
