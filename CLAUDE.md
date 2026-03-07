# Arcade Launcher

## What This Is

A **portable game manager** — a single TUI binary that lets users browse, install, and play games without admin access or system-level installation. Everything lives in userspace. Target audience: people who want games on machines they don't fully control (work computers, shared systems, lab machines).

## Core Principles

- **Portable first** — all game data goes under `~/.local/share/arcade-launcher/` (userspace). No sudo, no system packages, no admin privileges required.
- **Network-aware** — corporate networks may monitor or block `curl`/`wget`. Prefer `git clone` as the default acquisition method since it's almost always allowed on corp networks. Offer direct binary download as an opt-in alternative for users who can use it.
- **Recipe-based installs** — each game has a structured recipe, not just a flat command. Recipes define multiple install sources, pinned versions, build steps, and platform-specific details.
- **Version pinning** — always pin what's known to work (exact crate version, git tag, commit hash). Things break over time; pinned versions prevent "worked last year, broken now" surprises.
- **Platform-aware** — each game declares supported platforms (linux, windows, macos) and platform-specific dependencies. Unsupported games are greyed out. Dep hints shown in the details panel.
- **Let the user choose** — don't assume what's available. Detect toolchains and let users pick their preferred install method.
- **Clean uninstall** — everything the launcher installs must be fully removable from the TUI.

## Architecture

```
src/
  main.rs      — entry point, event loop, key handling
  app.rs       — App state, Mode enum, navigation
  catalog.rs   — Game struct, GAMES array, platform detection
  install.rs   — install/uninstall (shell, cmake, git-game, process kill)
  ui.rs        — all TUI rendering (banner, panels, footer)
  util.rs      — toolchain detection, path helpers, size formatting
```

- Uses `ratatui` + `crossterm` for the terminal UI
- Game catalog: static array in code + TOML recipe files under `games/` (recipe files are the target format, static array is transitional)
- Game wishlist: `games/wishlist.md`
- Dependencies: crossterm, ratatui, dirs

## Install Methods

- `cargo install <crate>` — Rust crate games (runtime: "cargo")
- `cmake` build from source — C/raylib games (runtime: "cmake")
- `git clone` + build steps — larger projects like citybound (runtime: "git")
- Internal commands: `git-game`, `git-game-remove`, `cmake-game`, `cmake-game-remove`

## Game Recipes (TOML)

Recipe files live in `games/`. Format:

```toml
[game]
name = "Citybound"
version = "0.4.0"
platforms = ["linux", "windows", "macos"]

[source.git]
repo = "https://github.com/citybound/citybound"
ref = "master"
build = ["npm", "install"]
play = ["npm", "start"]

[deps.linux]
packages = "libssl-dev pkg-config"
hint = "sudo apt install libssl-dev pkg-config"

[deps.windows]
packages = "VS2017 Build Tools"
hint = "Install VS Build Tools. Run all commands in Git Bash."

[deps.macos]
packages = "Xcode CLI tools, openssl"
hint = "xcode-select --install && brew install openssl"
```

## Platform Support

- `Game.platforms`: empty slice = all platforms, otherwise list of supported ones
- `Game.platform_deps`: per-OS dependency info (packages needed + install hint)
- `catalog::current_platform()` returns "linux", "windows", or "macos"
- `catalog::game_supports_platform()` checks if current OS is supported
- Unsupported games show "x" in list, greyed out, install blocked with message

## Build & Run

```sh
cargo run        # dev
cargo build --release  # release binary
```

## Code Style

- Modules split by concern, not over-abstracted
- No unnecessary abstractions — this is a practical tool, not a framework
