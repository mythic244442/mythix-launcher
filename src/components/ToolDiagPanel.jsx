// components/ToolDiagPanel.jsx
// Expandable `neutron diag` report for a compat tool — the exact resolved
// launch configuration Neutron would use, without launching anything.

import React, { useState } from "react";
import { api } from "../tauri.js";

// Severity heuristic. Neutron's own output has known-benign states:
//  - wine64 [MISSING] is normal on unified WoW64 builds (one loader does both)
//  - "BAD — ... (not in manifest, never installed)" is informational: those
//    libvkd3d-*.dll placeholders are Wine builtins Neutron never installs
//  - absent /dev/ntsync, missing gamemoderun: optional features, fallback exists
function severity(key, value) {
  const v = value.toLowerCase();
  if (key === "wine64" && v.includes("missing")) return "warn";
  if (v.includes("bad") && v.includes("not in manifest")) return "warn";
  if (v.includes("missing") || v.includes("bad")) return "bad";
  if (v.includes("absent") || v.includes("(not found)")) return "warn";
  return "ok";
}

const SEV_COLOR = {
  bad: "var(--coral)",
  warn: "var(--amber)",
  ok: "var(--text-1)",
};

function EntryRow({ entry }) {
  const sev = severity(entry.key, entry.value);
  const isEnv = entry.key.includes("=");
  return (
    <div style={{
      display: "flex",
      gap: 12,
      padding: "3px 0",
      fontFamily: "var(--font-mono)",
      fontSize: 11,
      lineHeight: 1.6,
      borderBottom: "1px solid var(--border)",
    }}>
      {isEnv ? (
        <span style={{ color: "var(--text-2)", wordBreak: "break-all" }}>{entry.key}</span>
      ) : (
        <>
          <span style={{ flexShrink: 0, width: 190, color: "var(--text-3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {entry.key}
          </span>
          <span style={{ color: SEV_COLOR[sev], wordBreak: "break-all" }}>
            {entry.value || "—"}
          </span>
        </>
      )}
    </div>
  );
}

export default function ToolDiagPanel({ toolPath }) {
  const [open, setOpen]       = useState(false);
  const [diag, setDiag]       = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError]     = useState(null);
  const [copied, setCopied]   = useState(false);

  async function toggle() {
    if (open) { setOpen(false); return; }
    setOpen(true);
    if (diag || loading) return;
    setLoading(true);
    setError(null);
    try {
      setDiag(await api.neutronDiag(toolPath));
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function copyRaw() {
    if (!diag) return;
    try {
      await navigator.clipboard.writeText(diag.raw);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch { /* clipboard can be unavailable; not worth an error state */ }
  }

  return (
    <div style={{ marginTop: 10 }}>
      <div style={{ display: "flex", gap: 8 }}>
        <button className="btn btn--ghost btn--sm" onClick={toggle}>
          {open ? "▾ Hide diag" : "▸ Run diag"}
        </button>
        {open && diag && (
          <button className="btn btn--ghost btn--sm" onClick={copyRaw}>
            {copied ? "✓ Copied" : "⧉ Copy raw"}
          </button>
        )}
      </div>

      {open && (
        <div style={{
          marginTop: 8,
          background: "var(--bg-1)",
          border: "1px solid var(--border)",
          borderRadius: "var(--r-md)",
          padding: "12px 14px",
        }}>
          {loading && (
            <div style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--text-3)" }}>
              Running neutron diag…
            </div>
          )}
          {error && (
            <div style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--coral)", lineHeight: 1.6 }}>
              {error}
            </div>
          )}
          {diag && diag.sections.map((section) => (
            <div key={section.title} style={{ marginBottom: 14 }}>
              <div style={{
                fontFamily: "var(--font-mono)", fontSize: 10, fontWeight: 700,
                letterSpacing: "0.1em", textTransform: "uppercase",
                color: "var(--neutron)", marginBottom: 6,
              }}>
                {section.title}
              </div>
              {section.entries.map((entry, i) => (
                <EntryRow key={`${entry.key}-${i}`} entry={entry} />
              ))}
            </div>
          ))}
          {diag && diag.sections.length === 0 && !loading && (
            <div style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--text-3)" }}>
              diag produced no parseable sections — check the raw output.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
