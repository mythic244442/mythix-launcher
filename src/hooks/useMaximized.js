import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useMaximized() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    win.isMaximized().then(setMaximized).catch(() => {});

    let unlisten;
    win.onResized(() => {
      win.isMaximized().then(setMaximized).catch(() => {});
    }).then((fn) => { unlisten = fn; });

    return () => { if (unlisten) unlisten(); };
  }, []);

  return maximized;
}
