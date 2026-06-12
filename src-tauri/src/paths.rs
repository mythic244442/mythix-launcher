// Centralised data-path resolution + one-time legacy migration.
//
// mythix-launcher stores data in three roots:
//   - $XDG_DATA_HOME/mythix/      (library.json, runtimes, covers…)
//   - $HOME/.mythix/              (gamez-pfx_data prefixes)
//   - $HOME/Games/mythix/         (default per-game install root)
//
// Older builds used "looni" in place of "mythix". On first run we move the
// legacy directories to their new homes so existing libraries and prefixes
// keep working after the rebrand. Migration is a one-time rename: if the new
// dir already exists we leave it alone and never touch the legacy dir again.

use std::path::PathBuf;
use std::sync::Once;

/// New brand directory name.
pub const APP_DIR: &str = "mythix";
/// Legacy brand directory name (pre-rebrand). Kept only for migration.
pub const LEGACY_APP_DIR: &str = "looni";

static MIGRATE: Once = Once::new();

/// Move a legacy directory to its new name if — and only if — the legacy dir
/// exists and the new one does not. Safe to call repeatedly.
fn migrate_one(legacy: &PathBuf, new: &PathBuf) {
    if legacy.exists() && !new.exists() {
        if let Some(parent) = new.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::rename(legacy, new) {
            Ok(_) => eprintln!(
                "[mythix] migrated {} -> {}",
                legacy.display(),
                new.display()
            ),
            Err(e) => eprintln!(
                "[mythix] WARNING: could not migrate {} -> {}: {}",
                legacy.display(),
                new.display(),
                e
            ),
        }
    }
}

/// Run the one-time legacy → mythix directory migration. Idempotent; the actual
/// work happens at most once per process via `Once`.
pub fn migrate_legacy_data() {
    MIGRATE.call_once(|| {
        if let Some(data) = dirs::data_dir() {
            migrate_one(&data.join(LEGACY_APP_DIR), &data.join(APP_DIR));
        }
        if let Some(home) = dirs::home_dir() {
            migrate_one(
                &home.join(format!(".{LEGACY_APP_DIR}")),
                &home.join(format!(".{APP_DIR}")),
            );
            migrate_one(
                &home.join("Games").join(LEGACY_APP_DIR),
                &home.join("Games").join(APP_DIR),
            );
            // gamez-pfx_data → gamedata rename
            let dot = home.join(format!(".{APP_DIR}"));
            migrate_one(&dot.join("gamez-pfx_data"), &dot.join("gamedata"));
        }
    });
}

/// $XDG_DATA_HOME/mythix — root for library.json, runtimes, covers.
pub fn data_dir() -> Option<PathBuf> {
    migrate_legacy_data();
    dirs::data_dir().map(|d| d.join(APP_DIR))
}

/// $HOME/.mythix — root for prefix data.
pub fn dot_dir() -> Option<PathBuf> {
    migrate_legacy_data();
    dirs::home_dir().map(|h| h.join(format!(".{APP_DIR}")))
}

/// $HOME/.mythix/gamedata/<game_id> — default prefix path for a game.
pub fn default_prefix_dir(game_id: &str) -> Option<PathBuf> {
    dot_dir().map(|d| d.join("gamedata").join(game_id))
}

/// Strip the launcher game-id prefix, accepting both the new "mythix-" form and
/// the legacy "looni-" form so games saved before the rebrand still resolve.
pub fn strip_game_id_prefix(game_id: &str) -> Option<&str> {
    game_id
        .strip_prefix("mythix-")
        .or_else(|| game_id.strip_prefix("looni-"))
}
