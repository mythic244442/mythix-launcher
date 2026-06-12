// components/RuntimeSetup.jsx
// Shown on first launch when the Steam Linux Runtime isn't installed yet.
// Explains what SLR is, lets the user trigger the download, shows progress.

import React from "react";

const STAGE_LABELS = {
  starting:   "Preparing…",
  fetch_meta: "Fetching build info…",
  download:   "Downloading runtime…",
  verify:     "Verifying checksum…",
  extract:    "Extracting…",
  install:    "Installing…",
  done:       "Done!",
  up_to_date: "Already up to date",
};

function ProgressBar({ percent }) {
  const indeterminate = percent < 0;
  return (
    <div style={{
      width: "100%",
      height: 6,
      background: "var(--bg-4)",
      borderRadius: 3,
      overflow: "hidden",
      position: "relative",
    }}>
      {indeterminate ? (
        <div style={{
          position: "absolute",
          height: "100%",
          width: "35%",
          background: "var(--lime)",
          borderRadius: 3,
          animation: "indeterminate 1.4s ease infinite",
        }} />
      ) : (
        <div style={{
          height: "100%",
          width: `${Math.max(0, Math.min(100, percent))}%`,
          background: "var(--lime)",
          borderRadius: 3,
          transition: "width 0.3s ease",
          boxShadow: "0 0 8px rgba(168,255,62,0.4)",
        }} />
      )}
    </div>
  );
}

export default function RuntimeSetup({ progress, installing, error, onInstall, onSkip }) {
  const stage = progress?.stage;
  const pct   = progress?.percent ?? 0;

  return (
    <div style={{
      flex: 1,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      padding: 40,
      background: "var(--bg-1)",
    }}>
      <div style={{
        maxWidth: 520,
        width: "100%",
      }}>
        {/* Header */}
        <div style={{
          fontFamily: "var(--font-display)",
          fontWeight: 800,
          fontSize: 28,
          letterSpacing: "-0.03em",
          color: "var(--text-0)",
          marginBottom: 6,
          lineHeight: 1.2,
        }}>
          One-time setup
        </div>
        <div style={{
          fontFamily: "var(--font-display)",
          fontWeight: 700,
          fontSize: 16,
          color: "var(--lime)",
          marginBottom: 24,
          letterSpacing: "-0.01em",
        }}>
          Steam Linux Runtime
        </div>

        {/* Explanation */}
        <div style={{
          background: "var(--bg-2)",
          border: "1px solid var(--border)",
          borderLeft: "3px solid var(--lime)",
          borderRadius: "var(--r-lg)",
          padding: "16px 18px",
          marginBottom: 20,
        }}>
          <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.75, marginBottom: 10 }}>
            mythix uses Valve's{" "}
            <span style={{ color: "var(--lime)", fontWeight: 600 }}>Steam Linux Runtime</span>
            {" "}— a free, publicly downloadable container that gives your games a
            consistent library environment regardless of which Linux distro you're on.
          </p>
          <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.75, marginBottom: 10 }}>
            This is the same runtime Steam uses internally, but{" "}
            <span style={{ color: "var(--text-0)", fontWeight: 600 }}>you don't need Steam installed.</span>
            {" "}It's downloaded directly from Valve's CDN (~200 MB) and stored in{" "}
            <code style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--sky)" }}>
              ~/.local/share/mythix/steamrt3/
            </code>.
          </p>
          <p style={{ fontSize: 13, color: "var(--text-1)", lineHeight: 1.75 }}>
            The full launch chain becomes:{" "}
            <code style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--lime)", display: "block", marginTop: 6, lineHeight: 1.8 }}>
              SLR container → your wine-proton hybrid → game.exe
            </code>
          </p>
        </div>

        {/* Stats row */}
        <div style={{
          display: "flex",
          gap: 12,
          marginBottom: 24,
        }}>
          {[
            ["~200 MB",  "download size"],
            ["sniper",   "SLR variant (steamrt3)"],
            ["Valve CDN","source"],
          ].map(([val, label]) => (
            <div key={label} style={{
              flex: 1,
              background: "var(--bg-2)",
              border: "1px solid var(--border)",
              borderRadius: "var(--r-md)",
              padding: "10px 12px",
              textAlign: "center",
            }}>
              <div style={{ fontFamily: "var(--font-mono)", fontWeight: 700, fontSize: 13, color: "var(--lime)", marginBottom: 2 }}>
                {val}
              </div>
              <div style={{ fontSize: 10, color: "var(--text-3)", fontFamily: "var(--font-mono)", letterSpacing: "0.05em", textTransform: "uppercase" }}>
                {label}
              </div>
            </div>
          ))}
        </div>

        {/* Progress area */}
        {(installing || progress) && (
          <div style={{
            background: "var(--bg-2)",
            border: "1px solid var(--border)",
            borderRadius: "var(--r-lg)",
            padding: "14px 16px",
            marginBottom: 16,
          }}>
            <div style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "baseline",
              marginBottom: 10,
            }}>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: 11, fontWeight: 700, color: "var(--lime)", letterSpacing: "0.06em" }}>
                {STAGE_LABELS[stage] ?? stage ?? "Working…"}
              </span>
              {pct >= 0 && (
                <span style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--text-2)" }}>
                  {pct}%
                </span>
              )}
            </div>
            <ProgressBar percent={pct} />
            {progress?.detail && (
              <div style={{ fontFamily: "var(--font-mono)", fontSize: 10, color: "var(--text-3)", marginTop: 7 }}>
                {progress.detail}
              </div>
            )}
          </div>
        )}

        {/* Error */}
        {error && (
          <div style={{
            background: "rgba(255,95,95,0.08)",
            border: "1px solid rgba(255,95,95,0.3)",
            borderRadius: "var(--r-md)",
            padding: "10px 14px",
            marginBottom: 16,
            fontFamily: "var(--font-mono)",
            fontSize: 11,
            color: "var(--coral)",
          }}>
            {error}
          </div>
        )}

        {/* Actions */}
        <div style={{ display: "flex", gap: 10 }}>
          <button
            className="btn btn--primary"
            style={{ flex: 1, justifyContent: "center", fontSize: 14 }}
            onClick={onInstall}
            disabled={installing}
          >
            {installing ? "Downloading…" : "↓ Download Runtime (~200 MB)"}
          </button>
          <button
            className="btn btn--ghost"
            onClick={onSkip}
            disabled={installing}
            title="Skip for now — games may not launch without the runtime"
          >
            Skip
          </button>
        </div>
        <p style={{ fontSize: 11, color: "var(--text-3)", marginTop: 8, textAlign: "center", lineHeight: 1.5 }}>
          You can also install it later from the{" "}
          <span style={{ color: "var(--text-2)" }}>Compat Tools</span> tab.
          Skipping means games that need the SLR container may not launch correctly.
        </p>
      </div>

      {/* Indeterminate animation keyframe — injected inline */}
      <style>{`
        @keyframes indeterminate {
          0%   { left: -35%; }
          100% { left: 100%; }
        }
      `}</style>
    </div>
  );
}
