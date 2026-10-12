# Mythix Loader

Launch Steam games you own from wherever they're installed: through the **real**
steamclient, with Steam running.

The loader is a tiny Windows stub (`mythix_loader.exe` / `mythix_loader_32.exe`).
Each game gets a *profile* in `~/.mythix/loader_profiles/<name>/` holding a copy of
the stub, a `mythix_loader.ini`, and the steamclient files copied from your own Steam
install. Run the stub under Neutron and it sets the Steam app id (the same
environment Steam itself sets) and starts the game's real exe from its install
folder. Your running Steam client does the ownership checks, and no game files are
touched.

## Layout

| Path | What |
|---|---|
| `mythix-loader` | Python 3 profile manager: the CLI, also driven by the launcher via `--json` |
| `src/mythix_loader.c`, `src/Makefile` | Loader stub source |
| `mythix_loader.exe`, `mythix_loader_32.exe` | Prebuilt stubs, embedded into the launcher at build time |

## Building the stubs

Needs mingw-w64 (`x86_64-w64-mingw32-gcc`, `i686-w64-mingw32-gcc`):

```sh
npm run build:loader        # or: make -C loader/src
```

After changing `mythix_loader.c`, rebuild and commit the exes. The launcher embeds
whatever is in this folder.

## CLI

```sh
loader/mythix-loader --games                      # installed games across all libraries
loader/mythix-loader --launch-info <appid>        # Steam's launch entries + the default pick
loader/mythix-loader "<install dir>" --dry-run    # plan a profile, write nothing
loader/mythix-loader "<install dir>"              # create / refresh the profile
loader/mythix-loader --list                       # existing profiles
loader/mythix-loader --setup                      # check stubs + Steam files
```

Launch a profile by hand:
`<neutron tool>/neutron waitforexitandrun ~/.mythix/loader_profiles/<name>/<stub>.exe`

`--json` prints a single result object on stdout (logs go to stderr). The stubs are
looked up in `--loader-dir`, then `$MYTHIX_LOADER_DIR`, then next to the script, then
`~/.mythix/mythix-loader/`.

## In the launcher

Import Games has two separate buttons per Steam game:

- **Loader** builds (or rebuilds) the profile and points the library at its stub.
  It never touches prefixes: a game already in your library keeps its own, and a
  new entry gets a fresh one.
- **Prefix** brings in a copy of Steam's prefix. It adds the game, or gives an
  existing entry Steam's prefix if that entry doesn't have one yet.

**＋ From folder → Steam game (loader)** does the Loader step for any folder. The
launcher keeps its own copy of this folder in `~/.local/share/mythix/loader/` and
runs `mythix-loader --json`. Entries get the game id `mythix-<appid>`.

## Launch arguments

Profiles take the exe from Steam's own launch config but don't bake in its
arguments. Options like `-dx12` / `-dx11` are choices, and forcing one would
override the game's own settings. Anything passed to the stub (except `--ini <path>`
and `--debug`) goes on to the game, so per-game launch args set in the launcher reach
it. Use `ExeCommandLine=` in the ini only for arguments the game should always get.
The ini also lists Steam's options as a comment.
