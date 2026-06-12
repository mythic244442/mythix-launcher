import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

export { convertFileSrc };

// Generate an invoke with error handling
const generateInvoke = (command, params = {}) => {
  return invoke(command, params).catch(error => {
    console.error(`Error invoking ${command}:`, error);
    throw error;
  });
};

export const api = {
  getGames:          ()                              => generateInvoke("get_games"),
  addGame:           (name, exePath)                 => generateInvoke("add_game", { name, exePath }),
  removeGame:        (gameId)                        => generateInvoke("remove_game", { gameId }),
  updateGameConfig:  (gameId, config)                => generateInvoke("update_game_config", { gameId, config }),
  updateGameMeta:    (gameId, { name, coverArt, notes, exePath }) => generateInvoke("update_game_meta", {
    gameId, game_id: gameId,
    name,
    coverArt, cover_art: coverArt,
    notes,
    exePath, exe_path: exePath
  }),
  getCompatTools:    ()                              => generateInvoke("get_compat_tools"),
  getRuntimeStatus:  ()                              => generateInvoke("get_runtime_status"),
  installRuntime:    (variant)                       => generateInvoke("install_runtime_cmd", { variant }),
  checkRuntimeUpdate:(variant)                       => generateInvoke("check_runtime_update", { variant }),
  launchGame:        (gameId)                        => generateInvoke("launch_game", { gameId }),
  killGame:          (pid, gameId)                    => generateInvoke("kill_game", { pid, gameId }),
  dryRunLaunch:      (gameId)                        => generateInvoke("dry_run_launch", { gameId }),
  setupPrefix:       (path)                          => generateInvoke("setup_prefix", { path }),
  scanSteam:         ()                              => generateInvoke("scan_steam"),
  importSteamGame:   (args)                          => generateInvoke("import_steam_game", args),
  findExe:           (installPath)                   => generateInvoke("find_exe", { installPath }),
  getSettings:       ()                              => generateInvoke("get_settings"),
  saveSettings:      (settings)                      => generateInvoke("save_settings", { settings }),
  fetchCover:        (gameId)                        => generateInvoke("fetch_cover", { gameId }),
  importCover:       (gameId, imagePath)             => generateInvoke("import_cover", { gameId, imagePath }),
  clearCover:        (gameId)                        => generateInvoke("clear_cover", { gameId }),
  windowMinimize:    ()                              => generateInvoke("window_minimize"),
  windowMaximize:    ()                              => generateInvoke("window_maximize"),
  windowClose:       ()                              => generateInvoke("window_close"),
  windowHide:        ()                              => generateInvoke("window_hide"),
  windowShow:        ()                              => generateInvoke("window_show"),
};

export async function pickFile(filters = []) {
  return openDialog({
    multiple: false,
    filters: filters.length ? filters : [
      { name: "Windows Executable", extensions: ["exe"] },
      { name: "All Files", extensions: ["*"] },
    ],
  });
}

export async function pickDirectory() {
  return openDialog({ multiple: false, directory: true });
}
