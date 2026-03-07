use crate::catalog::Game;
use std::path::PathBuf;

pub struct Toolchains {
    pub cargo: bool,
    pub python: bool,
    pub cmake: bool,
}

impl Toolchains {
    pub fn detect() -> Self {
        Self {
            cargo: which("cargo"),
            python: which("python") || which("python3"),
            cmake: which("cmake"),
        }
    }
}

pub fn which(bin: &str) -> bool {
    bin_path(bin).is_some()
}

pub fn bin_path(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(bin);
            [
                full.clone(),
                full.with_extension("exe"),
                full.with_extension("cmd"),
            ].into_iter().find(|candidate| candidate.is_file())
        })
    })
}

pub fn has_runtime(toolchains: &Toolchains, method: &str) -> bool {
    match method {
        "cargo" => toolchains.cargo,
        "cmake" => toolchains.cmake,
        "git" => which("git"),
        _ => true,
    }
}

pub fn runtime_install_hint(method: &str) -> &'static str {
    match method {
        "cargo" => "Install Rust: https://rustup.rs",
        "cmake" => "Need: cmake (cmake.org), git (git-scm.com), C compiler (gcc/MSVC)",
        "git" => "Install Git: https://git-scm.com",
        _ => "Unknown runtime",
    }
}

pub fn games_dir() -> PathBuf {
    let mut p = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("arcade-launcher");
    p.push("games");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Check if platform deps are satisfied by running the check command.
pub fn deps_check_satisfied(deps: &crate::catalog::PlatformDeps) -> bool {
    if deps.check_cmd.is_empty() {
        return false;
    }
    std::process::Command::new(deps.check_cmd[0])
        .args(&deps.check_cmd[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// For git-based games: repo is cloned but install didn't complete (.arcade-ready missing)
pub fn is_git_cloned_not_ready(game: &Game) -> bool {
    let source = match crate::catalog::default_source(game) {
        Some(s) => s,
        None => return false,
    };
    if source.method != "git" {
        return false;
    }
    let game_dir = games_dir().join(source.clone_dir);
    game_dir.join(".git").is_dir() && !game_dir.join(".arcade-ready").is_file()
}

pub fn cmake_game_exe(source: &crate::catalog::Source) -> PathBuf {
    let mut p = games_dir();
    p.push(source.clone_dir);
    p.push("build");
    if cfg!(windows) {
        p.push("Release");
        p.push(format!("{}.exe", source.bin));
    } else {
        p.push(source.bin);
    }
    p
}

pub fn install_size(game: &Game) -> Option<u64> {
    let source = crate::catalog::default_source(game)?;
    match source.method {
        "cmake" | "git" => {
            let dir = games_dir().join(source.clone_dir);
            if dir.is_dir() {
                Some(dir_size_recursive(&dir))
            } else {
                None
            }
        }
        _ => {
            if let Some(p) = bin_path(source.bin) {
                let game_dir = p.parent().and_then(|parent| {
                    let dir = parent.join(source.bin);
                    if dir.is_dir() { Some(dir) } else { None }
                });
                if let Some(dir) = game_dir {
                    let dir_size = dir_size_recursive(&dir);
                    let bin_bytes = std::fs::metadata(&p).ok().map(|m| m.len()).unwrap_or(0);
                    return Some(dir_size + bin_bytes);
                }
                std::fs::metadata(&p).ok().map(|m| m.len())
            } else {
                None
            }
        }
    }
}

fn dir_size_recursive(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let meta = entry.metadata();
            if let Ok(meta) = meta {
                if meta.is_dir() {
                    total += dir_size_recursive(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
