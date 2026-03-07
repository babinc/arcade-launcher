mod app;
mod catalog;
mod install;
mod ui;
mod util;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::prelude::*;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use app::{App, Mode};
use catalog::GAMES;
use install::{check_and_install_deps, kill_game_process, kill_pid, run_captured, run_cmake_install, run_dir_remove, run_git_install, run_shell_install};
use util::{has_runtime, runtime_install_hint};

fn run_visible(
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cmd: &str,
    args: &[&str],
    cwd: Option<&std::path::Path>,
) -> io::Result<(Terminal<CrosstermBackend<io::Stdout>>, bool)> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        cursor::Show,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    drop(terminal);

    let mut command = Command::new(cmd);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let status = command.status();

    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Show,
        cursor::MoveTo(0, 0)
    );
    io::stdout().flush()?;

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;
    // Force full redraw — clear + resize resets ratatui's diff state
    term.clear()?;
    let size = term.size()?;
    term.resize(size)?;

    let ok = status.map(|s| s.success()).unwrap_or(false);
    Ok((term, ok))
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    while !app.should_quit {
        // Auto-scroll install panel
        if let Mode::Installing {
            ref lines,
            ref done,
            ref mut scroll,
            ..
        } = app.mode
        {
            let line_count = lines.lock().unwrap().len() as u16;
            let panel_height = terminal
                .size()
                .map(|s| s.height.saturating_sub(16))
                .unwrap_or(10);
            *scroll = line_count.saturating_sub(panel_height);
            if let Some(_ok) = *done.lock().unwrap() {
                app.last_log = Some(lines.lock().unwrap().join("\n"));
            }
        }

        terminal.draw(|f| ui::ui(f, &mut app))?;
        app.tick = app.tick.wrapping_add(1);

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match &mut app.mode {
                    Mode::Installing { done, scroll, label, pid, .. } => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            *scroll = scroll.saturating_add(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            *scroll = scroll.saturating_sub(1);
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            let done_val = *done.lock().unwrap();
                            if done_val.is_some() {
                                let ok = done_val.unwrap_or(false);
                                app.refresh();
                                let idx = app.selected();
                                let msg = if ok {
                                    format!("{} done!", GAMES[idx].name)
                                } else {
                                    format!(
                                        "{} failed. Press [l] for log.",
                                        GAMES[idx].name
                                    )
                                };
                                app.message = Some((msg, ok));
                                app.mode = Mode::Normal;
                            } else if *label == "RUNNING" {
                                // Kill the running game process
                                if let Some(p) = *pid.lock().unwrap() {
                                    kill_pid(p);
                                }
                                app.refresh();
                                app.mode = Mode::Normal;
                                app.message = None;
                            }
                        }
                        _ => {}
                    },

                    Mode::ViewLog { scroll } => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('l') => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            *scroll = scroll.saturating_add(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            *scroll = scroll.saturating_sub(1);
                        }
                        KeyCode::PageDown => {
                            *scroll = scroll.saturating_add(10);
                        }
                        KeyCode::PageUp => {
                            *scroll = scroll.saturating_sub(10);
                        }
                        _ => {}
                    },

                    Mode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev(),

                        KeyCode::Enter => {
                            let idx = app.selected();
                            let game = &GAMES[idx];
                            if util::is_git_cloned_not_ready(game) {
                                app.message = Some((
                                    "Cloned but not built. Press [i] to build.".to_string(),
                                    false,
                                ));
                            } else if !app.installed[idx] {
                                app.message = Some((
                                    "Not installed. Press [i] to install.".to_string(),
                                    false,
                                ));
                            } else if let Some(src) = catalog::default_source(game) {
                                // Terminal games (crossterm/ratatui) need the full terminal
                                let is_terminal_game = matches!(game.engine, "crossterm" | "ratatui");

                                if is_terminal_game {
                                    let (t, ok) = if !src.play_cmd.is_empty() {
                                        let cmd = src.play_cmd[0];
                                        let args: Vec<&str> = src.play_cmd[1..].to_vec();
                                        run_visible(terminal, cmd, &args, None)?
                                    } else {
                                        run_visible(terminal, src.bin, &[], None)?
                                    };
                                    terminal = t;
                                    if !ok {
                                        app.message = Some((
                                            format!("{} exited with error", game.name),
                                            false,
                                        ));
                                    } else {
                                        app.message = None;
                                    }
                                } else {
                                    // Non-terminal games: capture output in a panel
                                    let (cmd_parts, cwd): (Vec<String>, Option<std::path::PathBuf>) = match src.method {
                                        "cmake" => {
                                            let exe = util::cmake_game_exe(src);
                                            (vec![exe.to_string_lossy().to_string()], None)
                                        }
                                        "git" | "binary" => {
                                            let game_dir = util::games_dir().join(src.clone_dir);
                                            let parts: Vec<String> = src.play_cmd.iter().map(|s| s.to_string()).collect();
                                            (parts, Some(game_dir))
                                        }
                                        _ if !src.play_cmd.is_empty() => {
                                            let parts: Vec<String> = src.play_cmd.iter().map(|s| s.to_string()).collect();
                                            (parts, None)
                                        }
                                        _ => {
                                            (vec![src.bin.to_string()], None)
                                        }
                                    };

                                    let lines = Arc::new(Mutex::new(vec![
                                        format!("$ {}", cmd_parts.join(" ")),
                                        String::new(),
                                    ]));
                                    let done = Arc::new(Mutex::new(None));
                                    let pid = Arc::new(Mutex::new(None));

                                    let lines_clone = Arc::clone(&lines);
                                    let done_clone = Arc::clone(&done);
                                    let pid_clone = Arc::clone(&pid);

                                    thread::spawn(move || {
                                        run_captured(&cmd_parts, cwd, &lines_clone, &done_clone, &pid_clone);
                                    });

                                    app.mode = Mode::Installing {
                                        lines,
                                        done,
                                        scroll: 0,
                                        label: "RUNNING",
                                        pid,
                                    };
                                    app.message = None;
                                }
                            }
                        }

                        KeyCode::Char('i') | KeyCode::Char('d') => {
                            let idx = app.selected();
                            let installing = key.code == KeyCode::Char('i');
                            let already = if installing {
                                app.installed[idx]
                            } else {
                                !app.installed[idx]
                            };
                            let source = catalog::default_source(&GAMES[idx]);
                            if installing && !crate::catalog::game_supports_platform(&GAMES[idx]) {
                                app.message = Some((
                                    format!(
                                        "{} not supported on {}",
                                        GAMES[idx].name,
                                        crate::catalog::current_platform()
                                    ),
                                    false,
                                ));
                            } else if source.is_none() {
                                app.message = Some((
                                    format!("{} has no install source", GAMES[idx].name),
                                    false,
                                ));
                            } else if already {
                                let msg = if installing {
                                    format!("{} already installed!", GAMES[idx].name)
                                } else {
                                    format!("{} not installed", GAMES[idx].name)
                                };
                                app.message = Some((msg, installing));
                            } else if installing
                                && !has_runtime(&app.toolchains, source.unwrap().method)
                            {
                                let src = source.unwrap();
                                let hint = runtime_install_hint(src.method);
                                app.message = Some((
                                    format!(
                                        "Missing {}! {}",
                                        src.method, hint
                                    ),
                                    false,
                                ));
                            } else {
                                let src = source.unwrap();
                                let cmd_parts: Vec<String> = if installing {
                                    catalog::source_install_cmd(src)
                                        .iter().map(|s| s.to_string()).collect()
                                } else {
                                    src.uninstall_cmd
                                        .iter().map(|s| s.to_string()).collect()
                                };
                                if cmd_parts.is_empty() {
                                    app.message = Some((
                                        format!("{} method not yet supported", src.method),
                                        false,
                                    ));
                                    continue;
                                }

                                // Install platform deps visibly (user sees full terminal output)
                                if installing {
                                    if let Some(deps) = catalog::platform_deps_for_current(&GAMES[idx]) {
                                        if !deps.install_cmd.is_empty()
                                            && !util::deps_check_satisfied(deps)
                                        {
                                            let args: Vec<&str> = deps.install_cmd[1..].to_vec();
                                            let (t, ok) = run_visible(
                                                terminal, deps.install_cmd[0], &args, None,
                                            )?;
                                            terminal = t;
                                            if !ok {
                                                app.message = Some((
                                                    "Dep install failed. Press [i] to retry.".to_string(),
                                                    false,
                                                ));
                                                continue;
                                            }
                                        }
                                    }
                                }

                                let lines = Arc::new(Mutex::new(vec![
                                    format!("$ {}", cmd_parts.join(" ")),
                                    String::new(),
                                ]));
                                let done = Arc::new(Mutex::new(None));
                                let pid = Arc::new(Mutex::new(None));

                                let lines_clone = Arc::clone(&lines);
                                let done_clone = Arc::clone(&done);

                                let kill_bin = if !installing {
                                    Some(src.bin.to_string())
                                } else {
                                    None
                                };

                                let game_name = GAMES[idx].name.to_string();
                                let is_install = installing;

                                thread::spawn(move || {
                                    if let Some(ref bin) = kill_bin {
                                        kill_game_process(bin);
                                    }

                                    // Check and install platform deps before game install
                                    if is_install {
                                        if !check_and_install_deps(&game_name, &lines_clone) {
                                            lines_clone.lock().unwrap().push(String::new());
                                            lines_clone.lock().unwrap().push(
                                                "Continuing anyway — build may fail if deps are missing.".to_string()
                                            );
                                        }
                                        lines_clone.lock().unwrap().push(String::new());
                                    }

                                    match cmd_parts[0].as_str() {
                                        "cmake-game" => run_cmake_install(&cmd_parts, &lines_clone, &done_clone),
                                        "cmake-game-remove" | "git-game-remove" => run_dir_remove(&cmd_parts, &lines_clone, &done_clone),
                                        "git-game" => run_git_install(&cmd_parts, &lines_clone, &done_clone),
                                        _ => run_shell_install(&cmd_parts, &lines_clone, &done_clone),
                                    }
                                });

                                let install_label = if installing { "INSTALLING" } else { "REMOVING" };
                                app.mode = Mode::Installing {
                                    lines,
                                    done,
                                    scroll: 0,
                                    label: install_label,
                                    pid,
                                };
                                app.message = None;
                            }
                        }

                        KeyCode::Char('r') => {
                            app.refresh();
                            app.message = Some(("Refreshed!".to_string(), true));
                        }
                        KeyCode::Char('l') => {
                            if app.last_log.is_some() {
                                app.mode = Mode::ViewLog { scroll: 0 };
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}
