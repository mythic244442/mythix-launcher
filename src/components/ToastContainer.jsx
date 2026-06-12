import React from "react";

const ICONS = { success: "✓", error: "✖", info: "◆" };

export default function ToastContainer({ toasts }) {
  if (!toasts.length) return null;
  return (
    <div className="toast-container">
      {toasts.map((t) => (
        <div key={t.id} className={`toast toast--${t.type}`}>
          <span style={{ opacity: 0.7 }}>{ICONS[t.type] ?? "◆"}</span>
          {t.message}
        </div>
      ))}
    </div>
  );
}
