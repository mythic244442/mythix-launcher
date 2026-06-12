import { useState, useEffect, useCallback } from "react";
import { api } from "../tauri.js";

export function useLibrary() {
  const [games, setGames]     = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError]     = useState(null);

  const refresh = useCallback(async () => {
    try {
      const list = await api.getGames();
      setGames(list);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const addGame = useCallback(async (name, exePath) => {
    const game = await api.addGame(name, exePath);
    setGames((prev) => [...prev, game]);
    return game;
  }, []);

  const removeGame = useCallback(async (gameId) => {
    await api.removeGame(gameId);
    setGames((prev) => prev.filter((g) => g.id !== gameId));
  }, []);

  const updateConfig = useCallback(async (gameId, config) => {
    await api.updateGameConfig(gameId, config);
    setGames((prev) => prev.map((g) => g.id === gameId ? { ...g, config } : g));
  }, []);

  const replaceGame = useCallback((updated) => {
    if (!updated?.id) return;
    setGames((prev) => prev.map((g) => g.id === updated.id ? updated : g));
  }, []);

  return { games, loading, error, refresh, addGame, removeGame, updateConfig, replaceGame };
}
