# arcade-launcher

A terminal game launcher built with Rust. Browse, install, and play terminal and graphical games from a single TUI menu.

![Rust](https://img.shields.io/badge/rust-stable-orange)

## Features

- Browse 16 games from a slick TUI with rainbow ASCII banner
- Install/uninstall games directly from the menu via `cargo install`
- Live cargo output panel shows compilation progress in a split pane
- High score tracking with persistent JSON storage
- Toolchain detection (cargo, java, python) with status indicators
- Full log viewer for debugging failed installs
- Cross-platform (Windows, Linux, macOS)

## Games

| Game | Type | Category |
|------|------|----------|
| Minesweeper | Terminal | Puzzle |
| Sudoku | Terminal | Puzzle |
| Tetris | Terminal | Action |
| Snake | Terminal | Action |
| Wordle | Terminal | Word |
| Rustle | Terminal | Word |
| Terminal RPG | Terminal | RPG |
| Mastermind | Terminal | Puzzle |
| Flappy | Graphical | Action |
| Albion RPG | Terminal | RPG |
| 2048 | Terminal | Puzzle |
| Tower Defense | Graphical | Strategy |
| Sokoban | Graphical | Puzzle |
| Rocket | Graphical | Action |
| Snake GFX | Graphical | Action |
| Block Breaker | Terminal | Action |

## Install

```sh
cargo install --git https://github.com/carman/arcade-launcher
```

Or clone and build:

```sh
git clone https://github.com/carman/arcade-launcher.git
cd arcade-launcher
cargo run
```

## Controls

| Key | Action |
|-----|--------|
| `j/k` | Navigate game list |
| `Enter` | Play selected game |
| `i` | Install selected game |
| `d` | Uninstall selected game |
| `l` | View last install log |
| `r` | Refresh install status |
| `q` | Quit |

## Requirements

- Rust toolchain (`cargo`) — required for installing games
- Games are installed via `cargo install` from crates.io

## License

MIT
