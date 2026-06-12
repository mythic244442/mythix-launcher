import React from "react";

export default function RuntimeStatusBadge({ runtime }) {
  if (!runtime || !runtime.installed) {
    return (
      <span style={{
        fontFamily:"var(--font-mono)",fontSize:9,fontWeight:700,
        letterSpacing:"0.08em",textTransform:"uppercase",
        color:"var(--amber)",background:"rgba(255,187,56,0.12)",
        border:"1px solid rgba(255,187,56,0.3)",borderRadius:4,padding:"2px 6px",
      }}>not installed</span>
    );
  }
  return (
    <span style={{
      fontFamily:"var(--font-mono)",fontSize:9,fontWeight:700,
      letterSpacing:"0.08em",textTransform:"uppercase",
      color:"var(--lime)",background:"rgba(168,255,62,0.1)",
      border:"1px solid rgba(168,255,62,0.3)",borderRadius:4,padding:"2px 6px",
    }}>
      SLR ✓ {runtime.build_id ? `(${runtime.build_id.slice(0,8)})` : ""}
    </span>
  );
}
