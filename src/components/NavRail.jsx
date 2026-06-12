import React, { useState, useRef, useEffect } from "react";

const NAV_ITEMS = [
  { id: "library",  label: "Library",            icon: "⊞" },
  { id: "steam",    label: "Import Games",        icon: "⇓" },
  { id: "tools",    label: "Compatibility Tools", icon: "⚙" },
  { id: "settings", label: "Settings",            icon: "◈" },
];

function NavDropdown({ active, onNavigate }) {
  const [open, setOpen] = useState(false);
  const ref = useRef(null);
  const current = NAV_ITEMS.find((i) => i.id === active) ?? NAV_ITEMS[0];

  useEffect(() => {
    if (!open) return;
    function close(e) {
      if (ref.current && !ref.current.contains(e.target)) setOpen(false);
    }
    document.addEventListener("pointerdown", close);
    return () => document.removeEventListener("pointerdown", close);
  }, [open]);

  return (
    <div className="nav-dropdown" ref={ref}>
      <button
        className="nav-dropdown__trigger"
        onClick={() => setOpen(!open)}
      >
        <span className="nav-dropdown__icon">{current.icon}</span>
        <span className="nav-dropdown__label">{current.label}</span>
        <span className={`nav-dropdown__chevron ${open ? "open" : ""}`}>▾</span>
      </button>
      {open && (
        <div className="nav-dropdown__menu">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              className={`nav-dropdown__option ${active === item.id ? "active" : ""}`}
              onClick={() => { onNavigate(item.id); setOpen(false); }}
            >
              <span className="nav-dropdown__option-icon">{item.icon}</span>
              <span>{item.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function NavBtn({ id, icon, label, active, onNavigate, className, focused }) {
  return (
    <button
      className={`nav-frame__btn ${className || ""} ${active === id ? "active" : ""} ${focused ? "controller-focus" : ""}`}
      onClick={() => onNavigate(id)}
    >
      <span className="nav-frame__icon">{icon}</span>
      <span className="nav-frame__label">{label}</span>
    </button>
  );
}

export default function NavRail({ active, onNavigate, maximized, controllerZone, children }) {
  if (maximized) {
    return (
      <div className="nav-spiral">
        <NavBtn id="library"  icon="⊞" label="Library"  active={active} onNavigate={onNavigate} className="nav-spiral__top"    focused={controllerZone === "library"} />
        <NavBtn id="steam"    icon="⇓" label="Import"   active={active} onNavigate={onNavigate} className="nav-spiral__right"  focused={controllerZone === "steam"} />
        <NavBtn id="tools"    icon="⚙" label="Tools"    active={active} onNavigate={onNavigate} className="nav-spiral__left"   focused={controllerZone === "tools"} />
        <NavBtn id="settings" icon="◈" label="Settings" active={active} onNavigate={onNavigate} className="nav-spiral__bottom" focused={controllerZone === "settings"} />
        <div className={`nav-spiral__content ${controllerZone === "content" ? "controller-focus-content" : ""}`}>{children}</div>
      </div>
    );
  }

  return (
    <div className="nav-layout nav-layout--windowed">
      <div className="nav-layout__content">{children}</div>
    </div>
  );
}

export { NavDropdown };
