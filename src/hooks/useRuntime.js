// hooks/useRuntime.js
// Manages Steam Linux Runtime state:
//  - Polls install status on mount
//  - Exposes install() to trigger a download
//  - Listens to runtime:progress events from the Rust backend

import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// Default variant — sniper (steamrt3) is what GE-Proton needs
export const DEFAULT_VARIANT = "steamrt3";

export function useRuntime() {
  const [runtimes, setRuntimes]       = useState([]);
  const [loading, setLoading]         = useState(true);
  const [installing, setInstalling]   = useState(false);
  const [progress, setProgress]       = useState(null); // ProgressEvent | null
  const [error, setError]             = useState(null);
  const unlistenRef                   = useRef(null);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke("get_runtime_status");
      setRuntimes(list);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Subscribe to progress events from the Rust download
  useEffect(() => {
    let active = true;
    listen("runtime:progress", (event) => {
      if (!active) return;
      setProgress(event.payload);
      // Clear progress a moment after completion
      if (event.payload.stage === "done" || event.payload.stage === "up_to_date") {
        setTimeout(() => {
          if (active) setProgress(null);
        }, 2000);
      }
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });
    return () => {
      active = false;
      unlistenRef.current?.();
    };
  }, []);

  const install = useCallback(async (variant = DEFAULT_VARIANT) => {
    setInstalling(true);
    setError(null);
    setProgress({ stage: "starting", variant, percent: 0, detail: "Preparing…" });
    try {
      await invoke("install_runtime_cmd", { variant });
      await refresh();
    } catch (e) {
      setError(String(e));
      setProgress(null);
    } finally {
      setInstalling(false);
    }
  }, [refresh]);

  const defaultRuntime = runtimes.find((r) => r.variant === DEFAULT_VARIANT);
  const isInstalled    = defaultRuntime?.installed ?? false;

  return {
    runtimes,
    loading,
    installing,
    progress,
    error,
    isInstalled,
    defaultRuntime,
    refresh,
    install,
  };
}
