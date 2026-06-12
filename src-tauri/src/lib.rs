pub mod commands;
pub mod error;
pub mod game_library;
pub mod gamepad;
pub mod launcher;
pub mod paths;
pub mod runtime;
pub mod settings;
pub mod steam_import;
pub mod coverart;
pub mod vdf;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // One-time rebrand migration: move legacy ~/.local/share/looni, ~/.looni,
    // and ~/Games/looni to their mythix equivalents before anything reads them.
    paths::migrate_legacy_data();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_games, add_game, remove_game, update_game_config, update_game_meta,
            get_compat_tools,
            get_runtime_status, install_runtime_cmd, check_runtime_update,
            launch_game, kill_game, dry_run_launch,
            setup_prefix,
            scan_steam, import_steam_game, find_exe,
            fetch_cover, import_cover, clear_cover,
            get_settings, save_settings,
            window_minimize, window_maximize, window_close, window_hide, window_show,
        ])
        .setup(|app| {
            gamepad::start_monitor(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running mythix launcher");
}
