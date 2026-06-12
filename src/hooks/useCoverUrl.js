import { useState, useEffect } from "react";
import { readFile } from "@tauri-apps/plugin-fs";

// WebKitGTK 2.5x drops asset:// requests from http origins, so convertFileSrc
// is unreliable on Linux. Read the file through the fs plugin instead and
// serve it as a blob URL.
const cache = new Map(); // absolute path -> object URL

export function invalidateCover(path) {
  const url = cache.get(path);
  if (url) {
    URL.revokeObjectURL(url);
    cache.delete(path);
  }
}

export function useCoverUrl(path) {
  const [url, setUrl] = useState(() => cache.get(path) ?? null);

  useEffect(() => {
    if (!path) { setUrl(null); return; }
    if (cache.has(path)) { setUrl(cache.get(path)); return; }
    let alive = true;
    readFile(path)
      .then((bytes) => {
        const u = URL.createObjectURL(new Blob([bytes]));
        cache.set(path, u);
        if (alive) setUrl(u);
      })
      .catch((e) => console.error("cover load failed:", path, e));
    return () => { alive = false; };
  }, [path]);

  return url;
}
