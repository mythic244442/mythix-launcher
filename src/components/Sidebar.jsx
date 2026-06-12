import React from "react";

const NAV = [
  { id: "library",  label: "Library",      icon: "⊞" },
  { id: "steam",    label: "Steam Import", icon: "⇓" },
  { id: "tools",    label: "Compat Tools", icon: "⚙" },
  { id: "settings", label: "Settings",     icon: "◈" },
];

export default function Sidebar({ active, onNavigate }) {
  return (
    <nav className="sidebar">
      <div className="sidebar__section-label">Navigate</div>
      {NAV.map((item) => (
        <button
          key={item.id}
          className={`sidebar__nav-item ${active === item.id ? "active" : ""}`}
          onClick={() => onNavigate(item.id)}
        >
          <span className="icon">{item.icon}</span>
          {item.label}
        </button>
      ))}
      <div className="sidebar__spacer" />
      <div className="sidebar__footer">mythix v0.2.0</div>
    </nav>
  );
}
