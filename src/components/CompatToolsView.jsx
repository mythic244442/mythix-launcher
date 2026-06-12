// components/CompatToolsView.jsx — updated with SLR management
import React, { useState, useEffect } from "react";
import { api } from "../tauri.js";
import { useRuntime, DEFAULT_VARIANT } from "../hooks/useRuntime.js";
import RuntimeStatusBadge from "./RuntimeStatusBadge.jsx";

const KIND_LABELS = {
  "neutron":            { label: "Neutron",            color: "var(--neutron)" },
  "proton":             { label: "Proton",             color: "var(--sky)" },
  "wine":               { label: "Wine",               color: "var(--amber)" },
  "other":              { label: "Other",              color: "var(--text-2)" },
};

function KindBadge({ kind }) {
  const { label, color } = KIND_LABELS[kind] ?? KIND_LABELS["other"];
  return (
    <span style={{
      fontFamily:"var(--font-mono)",fontSize:9,fontWeight:700,letterSpacing:"0.08em",
      textTransform:"uppercase",color,background:`${color}18`,border:`1px solid ${color}40`,
      borderRadius:4,padding:"2px 6px",
    }}>{label}</span>
  );
}

const STAGE_LABELS = {
  starting:"Starting…",fetch_meta:"Fetching metadata…",download:"Downloading…",
  verify:"Verifying…",extract:"Extracting…",install:"Installing…",
  done:"Done!",up_to_date:"Up to date",
};

export default function CompatToolsView() {
  const [tools, setTools]           = useState([]);
  const [toolsLoading, setLoading]  = useState(true);
  const { runtimes, installing, progress, error: rtError, install, refresh: refreshRt } = useRuntime();

  async function reloadTools() {
    setLoading(true);
    try { setTools(await api.getCompatTools()); } finally { setLoading(false); }
  }
  useEffect(() => { reloadTools(); }, []);

  return (
    <div className="content" style={{overflow:"hidden",display:"flex",flexDirection:"column"}}>
      <div className="content-header">
        <span className="content-header__title">Compatibility Tools</span>
        <span className="content-header__count">{tools.length}</span>
        <div className="content-header__spacer"/>
        <button className="btn btn--secondary btn--sm" onClick={() => { reloadTools(); refreshRt(); }}>↻ Rescan</button>
      </div>

      <div style={{flex:1,overflowY:"auto",padding:"20px 24px",display:"flex",flexDirection:"column",gap:24}}>

        {/* SLR section */}
        <div>
          <div style={{fontFamily:"var(--font-mono)",fontSize:10,fontWeight:700,letterSpacing:"0.12em",color:"var(--text-3)",textTransform:"uppercase",marginBottom:12}}>
            Steam Linux Runtime
          </div>
          {runtimes.map((rt) => (
            <div key={rt.variant} style={{
              background:"var(--bg-2)",border:"1px solid var(--border)",
              borderLeft: rt.installed ? "3px solid var(--lime)" : "3px solid var(--border)",
              borderRadius:"var(--r-lg)",padding:"14px 16px",marginBottom:8,
            }}>
              <div style={{display:"flex",alignItems:"center",gap:10,marginBottom:8}}>
                <div style={{flex:1}}>
                  <div style={{fontFamily:"var(--font-display)",fontWeight:700,fontSize:14,color:"var(--text-0)",marginBottom:3}}>{rt.display_name}</div>
                  <div style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-3)"}}>{rt.local_path}</div>
                  {rt.installed && rt.build_id && (
                    <div style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-3)",marginTop:2}}>Build: {rt.build_id}</div>
                  )}
                </div>
                <RuntimeStatusBadge runtime={rt}/>
              </div>
              {installing && progress && (
                <div style={{marginBottom:10}}>
                  <div style={{display:"flex",justifyContent:"space-between",marginBottom:4}}>
                    <span style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--lime)"}}>{STAGE_LABELS[progress.stage] ?? progress.stage}</span>
                    {progress.percent >= 0 && <span style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-2)"}}>{progress.percent}%</span>}
                  </div>
                  <div style={{height:4,background:"var(--bg-4)",borderRadius:2,overflow:"hidden",position:"relative"}}>
                    {progress.percent < 0
                      ? <div style={{position:"absolute",height:"100%",width:"35%",background:"var(--lime)",borderRadius:2,animation:"indeterminate 1.4s ease infinite"}}/>
                      : <div style={{height:"100%",width:`${progress.percent}%`,background:"var(--lime)",borderRadius:2,transition:"width 0.3s ease"}}/>
                    }
                  </div>
                  {progress.detail && <div style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-3)",marginTop:4}}>{progress.detail}</div>}
                </div>
              )}
              <div style={{display:"flex",gap:8}}>
                {!rt.installed
                  ? <button className="btn btn--primary btn--sm" onClick={() => install(rt.variant)} disabled={installing}>↓ Install</button>
                  : <button className="btn btn--secondary btn--sm" onClick={() => install(rt.variant)} disabled={installing}>↻ Reinstall / Update</button>
                }
              </div>
            </div>
          ))}
          {rtError && <div style={{fontFamily:"var(--font-mono)",fontSize:11,color:"var(--coral)",padding:"8px 12px",background:"rgba(255,95,95,0.08)",border:"1px solid rgba(255,95,95,0.2)",borderRadius:"var(--r-md)",marginTop:8}}>{rtError}</div>}
          <div style={{marginTop:4,padding:"10px 14px",background:"var(--bg-2)",border:"1px solid var(--border)",borderRadius:"var(--r-md)"}}>
            <div style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-3)",marginBottom:6,letterSpacing:"0.06em"}}>LAUNCH CHAIN (when installed)</div>
            <div style={{fontFamily:"var(--font-mono)",fontSize:11,color:"var(--text-1)",lineHeight:2}}>
              <span style={{color:"var(--lime)"}}>SLR/_v2-entry-point</span>
              <span style={{color:"var(--text-3)"}}> → </span>
              <span style={{color:"var(--sky)"}}>mythix-shim</span>
              <span style={{color:"var(--text-3)"}}> → </span>
              <span style={{color:"var(--neutron)"}}>neutron</span>
              <span style={{color:"var(--text-3)"}}> / </span>
              <span style={{color:"var(--sky)"}}>proton</span>
              <span style={{color:"var(--text-3)"}}> / </span>
              <span style={{color:"var(--amber)"}}>wine</span>
              <span style={{color:"var(--text-3)"}}> → </span>
              <span style={{color:"var(--text-0)"}}>game.exe</span>
            </div>
          </div>
        </div>

        {/* Tools section */}
        <div>
          <div style={{fontFamily:"var(--font-mono)",fontSize:10,fontWeight:700,letterSpacing:"0.12em",color:"var(--text-3)",textTransform:"uppercase",marginBottom:12}}>
            Installed Proton / Wine Tools
          </div>
          {toolsLoading
            ? <div style={{color:"var(--text-3)",fontFamily:"var(--font-mono)",fontSize:12}}>Scanning…</div>
            : tools.length === 0
              ? <div style={{padding:"16px",background:"var(--bg-2)",border:"1px solid var(--border)",borderRadius:"var(--r-lg)",color:"var(--text-3)",fontSize:13}}>
                  No tools found. Install Proton, Neutron, or Wine into <code style={{fontFamily:"var(--font-mono)",fontSize:11,color:"var(--sky)",margin:"0 4px"}}>~/.steam/steam/compatibilitytools.d</code> then rescan.
                </div>
              : <div style={{display:"flex",flexDirection:"column",gap:8}}>
                  {tools.map((tool) => (
                    <div key={tool.path} style={{background:"var(--bg-2)",border:"1px solid var(--border)",borderRadius:"var(--r-lg)",padding:"12px 16px",display:"flex",alignItems:"center",gap:12}}>
                      <div style={{flex:1,minWidth:0}}>
                        <div style={{fontFamily:"var(--font-display)",fontWeight:700,fontSize:14,color:"var(--text-0)",marginBottom:4}}>{tool.name}</div>
                        <div style={{fontFamily:"var(--font-mono)",fontSize:10,color:"var(--text-3)",overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap"}}>{tool.path}</div>
                      </div>
                      <KindBadge kind={tool.kind}/>
                    </div>
                  ))}
                </div>
          }
        </div>
      </div>
      <style>{`@keyframes indeterminate { 0%{left:-35%} 100%{left:100%} }`}</style>
    </div>
  );
}
