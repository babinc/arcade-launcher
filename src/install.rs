use crate::catalog;
use crate::util::games_dir;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Check and install platform dependencies for a game.
/// Returns true if deps are satisfied (already present or successfully installed).
/// Returns false if deps are needed but couldn't be installed.
pub fn check_and_install_deps(
    game_name: &str,
    lines: &Arc<Mutex<Vec<String>>>,
) -> bool {
    let games = catalog::GAMES;
    let game = match games.iter().find(|g| g.name == game_name) {
        Some(g) => g,
        None => return true,
    };

    let deps = match catalog::platform_deps_for_current(game) {
        Some(d) => d,
        None => return true, // no deps for this platform
    };

    // Check if already satisfied
    if !deps.check_cmd.is_empty() && crate::util::deps_check_satisfied(deps) {
        lines.lock().unwrap().push(format!("Dependencies already installed: {}", deps.deps));
        return true;
    }

    if deps.install_cmd.is_empty() {
        lines.lock().unwrap().push(format!(
            "Manual setup required: {}",
            deps.deps
        ));
        lines.lock().unwrap().push("Cannot auto-install on this platform.".to_string());
        return false;
    }

    // Install deps
    lines.lock().unwrap().push(String::new());
    lines.lock().unwrap().push(format!("Installing dependencies: {}", deps.deps));
    if deps.needs_sudo {
        lines.lock().unwrap().push("(this may prompt for your password)".to_string());
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if run_step(deps.install_cmd[0], &deps.install_cmd[1..], &cwd, lines) {
        lines.lock().unwrap().push("Dependencies installed!".to_string());
        true
    } else {
        lines.lock().unwrap().push("Failed to install dependencies.".to_string());
        if deps.needs_sudo {
            lines.lock().unwrap().push("You may need to run this manually:".to_string());
            lines.lock().unwrap().push(format!("  {}", deps.install_cmd.join(" ")));
        }
        false
    }
}

const CMAKE_TEMPLATE: &str = "\
cmake_minimum_required(VERSION 3.14)\n\
project({name} C)\n\
include(FetchContent)\n\
FetchContent_Declare(raylib GIT_REPOSITORY https://github.com/raysan5/raylib.git GIT_TAG 5.5 GIT_SHALLOW TRUE)\n\
FetchContent_MakeAvailable(raylib)\n\
add_executable({name} {name}.c)\n\
target_link_libraries({name} raylib)\n\
if(WIN32)\n\
  target_link_libraries({name} winmm)\n\
endif()\n";

pub fn run_step(
    cmd: &str,
    args: &[&str],
    cwd: &std::path::Path,
    lines: &Arc<Mutex<Vec<String>>>,
) -> bool {
    lines
        .lock()
        .unwrap()
        .push(format!("$ {} {}", cmd, args.join(" ")));
    let result = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    match result {
        Ok(mut child) => {
            let stderr = child.stderr.take();
            let stdout = child.stdout.take();
            let l2 = Arc::clone(lines);
            let t1 = thread::spawn(move || {
                if let Some(s) = stderr {
                    for line in BufReader::new(s).lines().map_while(Result::ok) {
                        l2.lock().unwrap().push(line);
                    }
                }
            });
            let l3 = Arc::clone(lines);
            let t2 = thread::spawn(move || {
                if let Some(s) = stdout {
                    for line in BufReader::new(s).lines().map_while(Result::ok) {
                        l3.lock().unwrap().push(line);
                    }
                }
            });
            let _ = t1.join();
            let _ = t2.join();
            child.wait().map(|s| s.success()).unwrap_or(false)
        }
        Err(e) => {
            lines.lock().unwrap().push(format!("Error: {}", e));
            false
        }
    }
}

fn ensure_raylib_games_repo(lines: &Arc<Mutex<Vec<String>>>) -> Option<PathBuf> {
    let repo_dir = games_dir().join("raylib-games");
    if repo_dir.join(".git").is_dir() {
        lines
            .lock()
            .unwrap()
            .push("Source repo already cloned.".to_string());
        return Some(repo_dir);
    }
    lines
        .lock()
        .unwrap()
        .push("Cloning raylib-games repo (shallow)...".to_string());
    let parent = games_dir();
    if run_step(
        "git",
        &[
            "clone",
            "--depth",
            "1",
            "https://github.com/raysan5/raylib-games.git",
        ],
        &parent,
        lines,
    ) {
        Some(repo_dir)
    } else {
        None
    }
}

pub fn run_cmake_install(
    cmd_parts: &[String],
    lines: &Arc<Mutex<Vec<String>>>,
    done: &Arc<Mutex<Option<bool>>>,
) {
    let name = &cmd_parts[1];
    let src_rel = &cmd_parts[2];

    let mut game_dir = games_dir();
    game_dir.push(name);
    let _ = std::fs::create_dir_all(&game_dir);

    let repo_dir = match ensure_raylib_games_repo(lines) {
        Some(d) => d,
        None => {
            lines
                .lock()
                .unwrap()
                .push("Failed to clone source repo!".to_string());
            *done.lock().unwrap() = Some(false);
            return;
        }
    };

    let src_file = repo_dir.join(src_rel);
    let dest_file = game_dir.join(format!("{}.c", name));
    lines
        .lock()
        .unwrap()
        .push(format!("Copying {} ...", src_rel));
    if let Err(e) = std::fs::copy(&src_file, &dest_file) {
        lines
            .lock()
            .unwrap()
            .push(format!("Failed to copy source: {}", e));
        *done.lock().unwrap() = Some(false);
        return;
    }

    let cmake_content = CMAKE_TEMPLATE.replace("{name}", name);
    let cmake_path = game_dir.join("CMakeLists.txt");
    if std::fs::write(&cmake_path, &cmake_content).is_err() {
        lines
            .lock()
            .unwrap()
            .push("Failed to write CMakeLists.txt".to_string());
        *done.lock().unwrap() = Some(false);
        return;
    }
    lines
        .lock()
        .unwrap()
        .push("Wrote CMakeLists.txt".to_string());

    lines.lock().unwrap().push(String::new());
    lines
        .lock()
        .unwrap()
        .push("Configuring cmake (fetching raylib)...".to_string());
    if !run_step(
        "cmake",
        &["-B", "build", "-DCMAKE_BUILD_TYPE=Release"],
        &game_dir,
        lines,
    ) {
        lines
            .lock()
            .unwrap()
            .push("cmake configure failed!".to_string());
        *done.lock().unwrap() = Some(false);
        return;
    }

    lines.lock().unwrap().push(String::new());
    lines.lock().unwrap().push("Building...".to_string());
    let ok = run_step(
        "cmake",
        &["--build", "build", "--config", "Release"],
        &game_dir,
        lines,
    );
    *done.lock().unwrap() = Some(ok);
}

/// Remove a game directory. Used by both cmake-game-remove and git-game-remove.
pub fn run_dir_remove(
    cmd_parts: &[String],
    lines: &Arc<Mutex<Vec<String>>>,
    done: &Arc<Mutex<Option<bool>>>,
) {
    let name = &cmd_parts[1];
    let game_dir = games_dir().join(name);
    lines
        .lock()
        .unwrap()
        .push(format!("Removing {} ...", game_dir.display()));
    match std::fs::remove_dir_all(&game_dir) {
        Ok(_) => {
            lines.lock().unwrap().push("Removed!".to_string());
            *done.lock().unwrap() = Some(true);
        }
        Err(e) => {
            lines.lock().unwrap().push(format!("Error: {}", e));
            *done.lock().unwrap() = Some(false);
        }
    }
}

pub fn run_shell_install(
    cmd_parts: &[String],
    lines: &Arc<Mutex<Vec<String>>>,
    done: &Arc<Mutex<Option<bool>>>,
) {
    let pid = Arc::new(Mutex::new(None));
    run_captured(cmd_parts, None, lines, done, &pid);
}

/// Install a game by cloning a git repo and running build steps.
/// cmd_parts: ["git-game", "dir_name", "repo_url", "--shallow"|"--full", build_cmd...]
pub fn run_git_install(
    cmd_parts: &[String],
    lines: &Arc<Mutex<Vec<String>>>,
    done: &Arc<Mutex<Option<bool>>>,
) {
    let name = &cmd_parts[1];
    let repo_url = &cmd_parts[2];
    let shallow = cmd_parts.get(3).map(|s| s.as_str()) == Some("--shallow");
    let build_steps = &cmd_parts[4..]; // after the depth flag

    let game_dir = games_dir().join(name);

    // Clone if not already cloned
    if game_dir.join(".git").is_dir() {
        lines
            .lock()
            .unwrap()
            .push(format!("{} already cloned, pulling latest...", name));
        // Unshallow if needed (e.g. was shallow but recipe changed to full)
        if !shallow {
            let shallow_file = game_dir.join(".git").join("shallow");
            if shallow_file.is_file() {
                lines.lock().unwrap().push("Fetching full history...".to_string());
                let _ = run_step("git", &["fetch", "--unshallow"], &game_dir, lines);
                let _ = run_step("git", &["fetch", "--tags"], &game_dir, lines);
            }
        }
        if !run_step("git", &["pull"], &game_dir, lines) {
            lines.lock().unwrap().push("git pull failed, continuing with existing code...".to_string());
        }
    } else {
        lines
            .lock()
            .unwrap()
            .push(format!("Cloning {} ...", repo_url));
        let parent = games_dir();
        let clone_args: Vec<&str> = if shallow {
            vec!["clone", "--depth", "1", repo_url, name]
        } else {
            vec!["clone", repo_url, name]
        };
        if !run_step("git", &clone_args, &parent, lines) {
            lines
                .lock()
                .unwrap()
                .push("git clone failed!".to_string());
            *done.lock().unwrap() = Some(false);
            return;
        }
    }

    // Run build steps — split on "&&" to get separate commands
    let mut current_cmd: Vec<&str> = Vec::new();
    for part in build_steps {
        if part == "&&" {
            if !current_cmd.is_empty() {
                lines.lock().unwrap().push(String::new());
                if !run_step(current_cmd[0], &current_cmd[1..], &game_dir, lines) {
                    lines
                        .lock()
                        .unwrap()
                        .push(format!("{} failed!", current_cmd[0]));
                    *done.lock().unwrap() = Some(false);
                    return;
                }
                current_cmd.clear();
            }
        } else {
            current_cmd.push(part);
        }
    }
    // Run last command if any
    if !current_cmd.is_empty() {
        lines.lock().unwrap().push(String::new());
        if !run_step(current_cmd[0], &current_cmd[1..], &game_dir, lines) {
            lines
                .lock()
                .unwrap()
                .push(format!("{} failed!", current_cmd[0]));
            *done.lock().unwrap() = Some(false);
            return;
        }
    }

    // Write marker file so the launcher knows this game is ready
    let marker = game_dir.join(".arcade-ready");
    let _ = std::fs::write(&marker, format!("installed {}", name));
    lines.lock().unwrap().push("Game ready!".to_string());

    *done.lock().unwrap() = Some(true);
}


/// Run a command with captured output, storing the child PID for potential killing.
pub fn run_captured(
    cmd_parts: &[String],
    cwd: Option<PathBuf>,
    lines: &Arc<Mutex<Vec<String>>>,
    done: &Arc<Mutex<Option<bool>>>,
    pid: &Arc<Mutex<Option<u32>>>,
) {
    let mut command = Command::new(&cmd_parts[0]);
    command.args(&cmd_parts[1..]);
    if let Some(ref dir) = cwd {
        command.current_dir(dir);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let result = command.spawn();
    match result {
        Ok(mut child) => {
            *pid.lock().unwrap() = Some(child.id());
            let stderr = child.stderr.take();
            let stdout = child.stdout.take();
            let l2 = Arc::clone(lines);
            let t1 = thread::spawn(move || {
                if let Some(s) = stderr {
                    for line in BufReader::new(s).lines().map_while(Result::ok) {
                        l2.lock().unwrap().push(line);
                    }
                }
            });
            let l3 = Arc::clone(lines);
            let t2 = thread::spawn(move || {
                if let Some(s) = stdout {
                    for line in BufReader::new(s).lines().map_while(Result::ok) {
                        l3.lock().unwrap().push(line);
                    }
                }
            });
            let _ = t1.join();
            let _ = t2.join();
            let ok = child.wait().map(|s| s.success()).unwrap_or(false);
            *pid.lock().unwrap() = None;
            *done.lock().unwrap() = Some(ok);
        }
        Err(e) => {
            lines.lock().unwrap().push(format!("Error: {}", e));
            *done.lock().unwrap() = Some(false);
        }
    }
}

pub fn kill_pid(pid: u32) {
    let _ = if cfg!(windows) {
        Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("kill")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
}

pub fn kill_game_process(bin: &str) {
    let _ = if cfg!(windows) {
        Command::new("taskkill")
            .args(["/F", "/IM", &format!("{}.exe", bin)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("pkill")
            .args(["-f", bin])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
}
