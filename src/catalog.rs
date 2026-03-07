/// How a game gets acquired and run.
pub struct Game {
    pub name: &'static str,
    pub icon: &'static str,
    pub desc: &'static str,
    pub keys: &'static str,
    pub category: &'static str,
    pub engine: &'static str,
    pub repo: &'static str,
    pub platforms: &'static [&'static str],       // empty = all
    pub platform_deps: &'static [PlatformDeps],
    pub sources: &'static [Source],               // ways to get this game, tried in order
}

/// A way to acquire and run a game.
#[allow(dead_code)]
pub struct Source {
    pub method: &'static str,   // "cargo", "cmake", "git", "binary"
    pub label: &'static str,    // shown in UI, e.g. "Build from source", "Download binary"

    // Acquire
    pub clone_url: &'static str,             // git repo URL (for git/cmake)
    pub clone_dir: &'static str,             // local dir name under games/
    pub binary_url: &'static str,            // download URL (for binary), supports {platform}
    pub shallow: bool,                       // true = --depth 1 clone (default for cmake), false = full clone

    // Build (empty = no build step needed)
    pub build_cmd: &'static [&'static str],  // e.g. &["npm", "install"] — "&&" separates steps

    // Run
    pub play_cmd: &'static [&'static str],   // how to launch — empty = just run bin
    pub bin: &'static str,                   // binary name for PATH lookup (cargo) or display

    // Uninstall
    pub uninstall_cmd: &'static [&'static str],
}

pub struct PlatformDeps {
    pub os: &'static str,
    pub deps: &'static str,
    pub install_cmd: &'static [&'static str],
    pub check_cmd: &'static [&'static str],
    pub needs_sudo: bool,
}

pub fn current_platform() -> &'static str {
    match std::env::consts::OS {
        "linux" => "linux",
        "windows" => "windows",
        "macos" => "macos",
        other => other,
    }
}

pub fn game_supports_platform(game: &Game) -> bool {
    game.platforms.is_empty() || game.platforms.contains(&current_platform())
}

pub fn platform_deps_for_current(game: &Game) -> Option<&'static PlatformDeps> {
    let os = current_platform();
    game.platform_deps.iter().find(|d| d.os == os)
}

/// Get the first source for a game (default install method).
/// Skips "binary" sources since binary download isn't implemented yet.
pub fn default_source(game: &Game) -> Option<&'static Source> {
    game.sources.iter().find(|s| s.method != "binary")
        .or(game.sources.first())
}

/// Build the install command for a source.
pub fn source_install_cmd(source: &Source) -> Vec<&'static str> {
    match source.method {
        "cargo" => vec!["cargo", "install", source.bin],
        "cmake" => source.build_cmd.to_vec(),
        "git" => {
            let depth = if source.shallow { "--shallow" } else { "--full" };
            let mut cmd = vec!["git-game", source.clone_dir, source.clone_url, depth];
            cmd.extend(source.build_cmd);
            cmd
        }
        _ => vec![],
    }
}

pub const GAMES: &[Game] = &[
    Game {
        name: "Minesweeper", icon: "#",
        desc: "Reveal tiles without hitting mines. Numbers show adjacent mine count. Flag tiles you think are mines. Clear all safe tiles to win.",
        keys: "WASD move, Q uncover tile, E flag/unflag", category: "Puzzle",
        engine: "crossterm", repo: "https://crates.io/crates/cmd-minesweeper",
        platforms: &[], platform_deps: &[],
        sources: &[Source {
            method: "cargo", label: "cargo install",
            clone_url: "", clone_dir: "", binary_url: "", shallow: true,
            build_cmd: &[],
            play_cmd: &[], bin: "cmd-minesweeper",
            uninstall_cmd: &["cargo", "uninstall", "cmd-minesweeper"],
        }],
    },
    Game {
        name: "Tetris", icon: "T",
        desc: "Falling block puzzle. Rotate and position tetrominoes to fill complete rows. Completed rows disappear. Game ends when blocks stack to the top.",
        keys: "Left/Right move, Up rotate, Down soft drop, Space hard drop, P pause", category: "Action",
        engine: "ratatui", repo: "https://crates.io/crates/sxtetris",
        platforms: &[], platform_deps: &[],
        sources: &[Source {
            method: "cargo", label: "cargo install",
            clone_url: "", clone_dir: "", binary_url: "", shallow: true,
            build_cmd: &[],
            play_cmd: &[], bin: "sxtetris",
            uninstall_cmd: &["cargo", "uninstall", "sxtetris"],
        }],
    },
    Game {
        name: "Chess TUI", icon: "K",
        desc: "Full chess in the terminal. Play vs a friend locally, against a UCI engine (Stockfish), or online via Lichess. Supports multiple board skins.",
        keys: "Arrows/hjkl move cursor, Space select/place, ? help, s cycle skins, q quit", category: "Strategy",
        engine: "ratatui", repo: "https://crates.io/crates/chess-tui",
        platforms: &[], platform_deps: &[],
        sources: &[Source {
            method: "cargo", label: "cargo install",
            clone_url: "", clone_dir: "", binary_url: "", shallow: true,
            build_cmd: &[],
            play_cmd: &[], bin: "chess-tui",
            uninstall_cmd: &["cargo", "uninstall", "chess-tui"],
        }],
    },
    Game {
        name: "2048", icon: "2",
        desc: "Slide all tiles in one direction. Matching numbers merge and double. Keep merging to reach 2048. Board fills up = game over.",
        keys: "Arrow keys / WASD to slide all tiles", category: "Puzzle",
        engine: "crossterm", repo: "https://crates.io/crates/cli_2048",
        platforms: &[], platform_deps: &[],
        sources: &[Source {
            method: "cargo", label: "cargo install",
            clone_url: "", clone_dir: "", binary_url: "", shallow: true,
            build_cmd: &[],
            play_cmd: &[], bin: "2048",
            uninstall_cmd: &["cargo", "uninstall", "cli_2048"],
        }],
    },
    Game {
        name: "RecWars", icon: "W",
        desc: "Top-down tank deathmatch. Drive tanks, hovercraft, or hummers and blast opponents with 8 weapons. Free-for-all, team war, or capture the cow. Graphical window.",
        keys: "Arrows drive, Ctrl fire, ; or ` console, Tab switch weapon",
        category: "Action",
        engine: "macroquad", repo: "https://crates.io/crates/rec-wars",
        platforms: &[], platform_deps: &[],
        sources: &[Source {
            method: "cargo", label: "cargo install",
            clone_url: "", clone_dir: "", binary_url: "", shallow: true,
            build_cmd: &[],
            play_cmd: &[], bin: "rec-wars",
            uninstall_cmd: &["cargo", "uninstall", "rec-wars"],
        }],
    },
    Game {
        name: "Space Invaders", icon: "V",
        desc: "Classic Space Invaders in a graphical window. Shoot descending waves of aliens before they reach the bottom. Built with raylib — pure C, zero dependencies.",
        keys: "Left/Right move, Space shoot",
        category: "Action",
        engine: "raylib", repo: "https://github.com/raysan5/raylib-games",
        platforms: &["linux", "windows", "macos"],
        platform_deps: &[
            PlatformDeps { os: "linux", deps: "build-essential libgl1-mesa-dev",
                install_cmd: &["sudo", "apt", "install", "-y", "build-essential", "libgl1-mesa-dev"],
                check_cmd: &["dpkg", "-s", "libgl1-mesa-dev"], needs_sudo: true },
            PlatformDeps { os: "windows", deps: "Visual Studio Build Tools",
                install_cmd: &[], check_cmd: &[], needs_sudo: false },
            PlatformDeps { os: "macos", deps: "Xcode CLI tools",
                install_cmd: &["xcode-select", "--install"],
                check_cmd: &["xcode-select", "-p"], needs_sudo: false },
        ],
        sources: &[Source {
            method: "cmake", label: "Build from source (cmake)",
            clone_url: "https://github.com/raysan5/raylib-games",
            clone_dir: "space_invaders", binary_url: "", shallow: true,
            build_cmd: &["cmake-game", "space_invaders", "classics/src/space_invaders.c"],
            play_cmd: &[], bin: "space_invaders",
            uninstall_cmd: &["cmake-game-remove", "space_invaders"],
        }],
    },
    Game {
        name: "Asteroids", icon: "A",
        desc: "Classic Asteroids in a graphical window. Pilot a ship, rotate and thrust to dodge, shoot to break asteroids into smaller pieces. Built with raylib — pure C.",
        keys: "Left/Right rotate, Up thrust, Space shoot",
        category: "Action",
        engine: "raylib", repo: "https://github.com/raysan5/raylib-games",
        platforms: &["linux", "windows", "macos"],
        platform_deps: &[
            PlatformDeps { os: "linux", deps: "build-essential libgl1-mesa-dev",
                install_cmd: &["sudo", "apt", "install", "-y", "build-essential", "libgl1-mesa-dev"],
                check_cmd: &["dpkg", "-s", "libgl1-mesa-dev"], needs_sudo: true },
            PlatformDeps { os: "windows", deps: "Visual Studio Build Tools",
                install_cmd: &[], check_cmd: &[], needs_sudo: false },
            PlatformDeps { os: "macos", deps: "Xcode CLI tools",
                install_cmd: &["xcode-select", "--install"],
                check_cmd: &["xcode-select", "-p"], needs_sudo: false },
        ],
        sources: &[Source {
            method: "cmake", label: "Build from source (cmake)",
            clone_url: "https://github.com/raysan5/raylib-games",
            clone_dir: "asteroids", binary_url: "", shallow: true,
            build_cmd: &["cmake-game", "asteroids", "classics/src/asteroids.c"],
            play_cmd: &[], bin: "asteroids",
            uninstall_cmd: &["cmake-game-remove", "asteroids"],
        }],
    },
    Game {
        name: "Anarch", icon: "F",
        desc: "Retro first-person shooter inspired by Doom. 10 levels, multiple weapons, all assets embedded in source code. Tiny, fast, zero external files needed.",
        keys: "WASD move, Mouse aim, LMB shoot, RMB next weapon, Space jump",
        category: "Action",
        engine: "SDL2", repo: "https://gitlab.com/drummyfish/anarch",
        platforms: &["linux", "windows", "macos"],
        platform_deps: &[
            PlatformDeps { os: "linux", deps: "libsdl2-dev g++",
                install_cmd: &["sudo", "apt", "install", "-y", "libsdl2-dev", "g++"],
                check_cmd: &["dpkg", "-s", "libsdl2-dev"], needs_sudo: true },
            PlatformDeps { os: "windows", deps: "SDL2, MinGW or MSVC",
                install_cmd: &[], check_cmd: &[], needs_sudo: false },
            PlatformDeps { os: "macos", deps: "SDL2",
                install_cmd: &["brew", "install", "sdl2"],
                check_cmd: &["brew", "list", "sdl2"], needs_sudo: false },
        ],
        sources: &[Source {
            method: "git", label: "Build from source (git + gcc)",
            clone_url: "https://gitlab.com/drummyfish/anarch.git",
            clone_dir: "anarch", binary_url: "", shallow: true,
            build_cmd: &["bash", "make.sh", "sdl"],
            play_cmd: &["./anarch"], bin: "anarch",
            uninstall_cmd: &["git-game-remove", "anarch"],
        }],
    },
];
