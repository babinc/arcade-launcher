# Linux-Only Games to Test

These failed to build on Windows (typically due to `termion` or Unix-only deps).
Test on Linux — if they work, add them with a "Linux only" note.

## Previously Tested (failed on Windows)

| Crate | Binary | Type | Why it failed |
|-------|--------|------|---------------|
| `utf-crawler` | utf-crawler | ASCII dungeon crawler with roguelike elements | termion (no Windows) |
| `game-2048` | 2048 | Slide tiles to reach 2048 | termion (no Windows) |
| `flappy` | flappy | Flappy Bird clone (Piston engine) | glutin/OpenGL panic |
| `rust_tower_defense` | rust_tower_defense | Tower defense (Piston) | missing map file panic |

## New Candidates to Try

| Crate | Install | Description | Notes |
|-------|---------|-------------|-------|
| `tflap` | `cargo install tflap` | Flappy Bird in the terminal | Likely termion-based |
| `flappy-tui` | `cargo install flappy-tui` | Flappy Bird TUI with pixel graphics and sound | Check deps |
| `kingslayer` | `cargo install kingslayer` | Text adventure dungeon crawler | May work on Windows too — test both |
| `dungeoncli` | `cargo install dungeoncli` | CLI dungeon crawler with combat, loot, leveling | May work on Windows too — test both |
| `asciiarena` | `cargo install asciiarena` | Terminal multiplayer deathmatch game | Likely termion-based |
| `shards_of_aether` | `cargo install shards_of_aether` | Text-based adventure RPG | May work on Windows too — test both |
| `asmcahligzamaze` | `cargo install asmcahligzamaze` | 2D maze adventure game | Check deps |
| `chess-tui` | `cargo install chess-tui` | Terminal chess (v2.5.1, very popular) | Likely crossterm — may work on Windows too |
| `taurus` | `cargo install taurus` | WIP roguelike game | Check deps |
| `rec-wars` | `cargo install rec-wars` | Top-down vehicle shooter (browser playable too) | Check deps |
| `rustcycles` | `cargo install rustcycles` | Multiplayer shooter on wheels (Fyrox engine) | Graphical, check Windows too |
| `breakout-mazy` | `cargo install breakout-mazy` | Atari breakout with extras | Check deps |
| `terminal_gameboy` | `cargo install terminal_gameboy` | Game Boy emulator in terminal | Check deps |

## Priority to Test First
1. `chess-tui` — likely crossterm-based, might work on Windows too (replace rg-chess)
2. `kingslayer` — text adventure, good chance of cross-platform
3. `dungeoncli` — CLI dungeon crawler, good chance of cross-platform
4. `flappy-tui` — if it works, fills the "flappy bird" gap
5. `terminal_gameboy` — a Game Boy emulator would be awesome
