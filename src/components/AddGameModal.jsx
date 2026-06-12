import React, { useState } from "react";
import { pickFile } from "../tauri.js";

export default function AddGameModal({ onAdd, onClose }) {
  const [name, setName]       = useState("");
  const [exePath, setExePath] = useState("");
  const [saving, setSaving]   = useState(false);
  const [err, setErr]         = useState("");

  async function handlePickExe() {
    const path = await pickFile();
    if (!path) return;
    setExePath(path);
    if (!name) {
      const base = path.split(/[\\/]/).pop().replace(/\.exe$/i,"").replace(/[_-]/g," ");
      setName(base.replace(/\b\w/g, (c) => c.toUpperCase()));
    }
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (!name.trim()) { setErr("Name is required."); return; }
    if (!exePath.trim()) { setErr("Executable path is required."); return; }
    setErr(""); setSaving(true);
    try { await onAdd(name.trim(), exePath.trim()); onClose(); }
    catch (ex) { setErr(String(ex)); }
    finally { setSaving(false); }
  }

  return (
    <div className="modal-backdrop" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal">
        <div className="modal__header">
          <span className="modal__title">Add Game</span>
          <button className="modal__close" onClick={onClose}>✕</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="form-label">Executable</label>
            <div className="form-path-row">
              <input className="form-input form-input--mono" value={exePath}
                onChange={(e) => setExePath(e.target.value)}
                placeholder="/path/to/game.exe" spellCheck={false} />
              <button type="button" className="btn btn--secondary btn--sm" onClick={handlePickExe}>Browse</button>
            </div>
          </div>
          <div className="form-group">
            <label className="form-label">Name</label>
            <input className="form-input" value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Game" autoFocus />
          </div>
          {err && <p style={{ color:"var(--coral)", fontSize:12, marginBottom:12 }}>{err}</p>}
          <div className="modal__footer">
            <button type="button" className="btn btn--ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn--primary" disabled={saving}>
              {saving ? "Adding…" : "Add Game"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
