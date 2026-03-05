use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

struct Game {
    name: &'static str,
    icon: &'static str,
    bin: &'static str,
    crate_name: &'static str,
    desc: &'static str,
    keys: &'static str,
    category: &'static str,
    runtime: &'static str, // "cargo", "java", "python"
    repo: &'static str,
}

const GAMES: &[Game] = &[
    Game {
        name: "Minesweeper",
        icon: "#",
        bin: "cmd-minesweeper",
        crate_name: "cmd-minesweeper",
        desc: "Sweep mines in the terminal. Flag bombs, reveal safe tiles.",
        keys: "WASD move, Q uncover, E flag",
        category: "Puzzle",
        runtime: "cargo",
        repo: "crates.io/crates/cmd-minesweeper",
    },
    Game {
        name: "Sudoku",
        icon: "9",
        bin: "sudoku",
        crate_name: "sudoku-tui",
        desc: "Classic number puzzles with multiple difficulty levels.",
        keys: "Arrows, 1-9 place, Bksp clear",
        category: "Puzzle",
        runtime: "cargo",
        repo: "crates.io/crates/sudoku-tui",
    },
    Game {
        name: "Tetris",
        icon: "T",
        bin: "sxtetris",
        crate_name: "sxtetris",
        desc: "Tetrominos fall and stack. Clear lines, chase high scores!",
        keys: "Arrows move, Space drop, P pause",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/sxtetris",
    },
    Game {
        name: "Snake",
        icon: "~",
        bin: "snake-tui",
        crate_name: "snake-tui",
        desc: "Eat food, grow longer, don't hit yourself.",
        keys: "Arrows/WASD",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/snake-tui",
    },
    Game {
        name: "Wordle",
        icon: "W",
        bin: "wordlet",
        crate_name: "wordlet",
        desc: "Guess the 5-letter word in 6 tries. Colors show hints.",
        keys: "Type letters, Enter submit",
        category: "Word",
        runtime: "cargo",
        repo: "crates.io/crates/wordlet",
    },
    Game {
        name: "Rustle",
        icon: "R",
        bin: "rustle-game",
        crate_name: "rustle-game",
        desc: "Play Wordle or Nerdle (math version) in the terminal.",
        keys: "Type letters/numbers, Enter",
        category: "Word",
        runtime: "cargo",
        repo: "crates.io/crates/rustle-game",
    },
    Game {
        name: "Terminal RPG",
        icon: "+",
        bin: "terminal_rpg",
        crate_name: "terminal_rpg",
        desc: "A text-based RPG adventure in your terminal.",
        keys: "Follow prompts",
        category: "RPG",
        runtime: "cargo",
        repo: "crates.io/crates/terminal_rpg",
    },
    Game {
        name: "Mastermind",
        icon: "?",
        bin: "mastermind-rs",
        crate_name: "mastermind-rs",
        desc: "Crack the secret color code. Bulls & Cows in the terminal.",
        keys: "Type guesses, Enter submit",
        category: "Puzzle",
        runtime: "cargo",
        repo: "crates.io/crates/mastermind-rs",
    },
    Game {
        name: "Flappy",
        icon: "^",
        bin: "flappy",
        crate_name: "flappy",
        desc: "Flappy bird in your terminal. Tap to fly!",
        keys: "Space to flap",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/flappy",
    },
    Game {
        name: "Albion RPG",
        icon: "&",
        bin: "albion_terminal_rpg",
        crate_name: "albion_terminal_rpg",
        desc: "A text-based RPG set in the world of Albion.",
        keys: "Follow prompts",
        category: "RPG",
        runtime: "cargo",
        repo: "crates.io/crates/albion_terminal_rpg",
    },
    Game {
        name: "2048",
        icon: "2",
        bin: "2048",
        crate_name: "cli_2048",
        desc: "Slide numbered tiles on a grid to combine them and reach 2048.",
        keys: "Arrow keys / WASD",
        category: "Puzzle",
        runtime: "cargo",
        repo: "crates.io/crates/cli_2048",
    },
    Game {
        name: "Tower Defense",
        icon: "O",
        bin: "rtd",
        crate_name: "rust_tower_defense",
        desc: "Place towers and defend against waves of enemies.",
        keys: "Mouse + keyboard",
        category: "Strategy",
        runtime: "cargo",
        repo: "crates.io/crates/rust_tower_defense",
    },
    Game {
        name: "Sokoban",
        icon: "B",
        bin: "sokoban-rs",
        crate_name: "sokoban-rs",
        desc: "Push boxes onto targets. Classic puzzle game with Piston graphics.",
        keys: "Arrow keys",
        category: "Puzzle",
        runtime: "cargo",
        repo: "crates.io/crates/sokoban-rs",
    },
    Game {
        name: "Rocket",
        icon: "A",
        bin: "rocket-game",
        crate_name: "rocket-game",
        desc: "Asteroids-style space shooter. Blast rocks, survive waves.",
        keys: "Arrows rotate/thrust, Space shoot",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/rocket-game",
    },
    Game {
        name: "Snake GFX",
        icon: "S",
        bin: "gusbunce-snake",
        crate_name: "gusbunce-snake",
        desc: "Graphical snake game built with the Piston engine.",
        keys: "Arrow keys",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/gusbunce-snake",
    },
    Game {
        name: "Block Breaker",
        icon: "=",
        bin: "block-breaker-tui",
        crate_name: "block-breaker-tui",
        desc: "Breakout-style brick breaker in the terminal.",
        keys: "Left/Right move, Space launch",
        category: "Action",
        runtime: "cargo",
        repo: "crates.io/crates/block-breaker-tui",
    },
];

const BANNER: &[&str] = &[
    r"                                                          ",
    r"     █████╗ ██████╗  ██████╗ █████╗ ██████╗ ███████╗      ",
    r"    ██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝      ",
    r"    ███████║██████╔╝██║     ███████║██║  ██║█████╗        ",
    r"    ██╔══██║██╔══██╗██║     ██╔══██║██║  ██║██╔══╝        ",
    r"    ██║  ██║██║  ██║╚██████╗██║  ██║██████╔╝███████╗      ",
    r"    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚═════╝ ╚══════╝      ",
];

const SUB_BANNER: &str = "- - - T E R M I N A L  *  G A M E  *  L A U N C H E R - - -";


// === HIGH SCORES ===

#[derive(Serialize, Deserialize, Default)]
struct ScoreStore {
    scores: HashMap<String, Vec<ScoreEntry>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ScoreEntry {
    score: u64,
    date: String,
}

fn scores_path() -> PathBuf {
    let mut p = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("arcade-launcher");
    let _ = std::fs::create_dir_all(&p);
    p.push("scores.json");
    p
}

fn load_scores() -> ScoreStore {
    let path = scores_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_scores(store: &ScoreStore) {
    let path = scores_path();
    if let Ok(json) = serde_json::to_string_pretty(store) {
        let _ = std::fs::write(path, json);
    }
}

fn add_score(store: &mut ScoreStore, game: &str, score: u64) {
    let now = chrono_lite_now();
    let entry = ScoreEntry {
        score,
        date: now,
    };
    let entries = store.scores.entry(game.to_string()).or_default();
    entries.push(entry);
    entries.sort_by(|a, b| b.score.cmp(&a.score));
    entries.truncate(5); // keep top 5
    save_scores(store);
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert epoch seconds to YYYY-MM-DD HH:MM (UTC)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    // Days since 1970-01-01
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let year_days = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < year_days { break; }
        remaining -= year_days;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0;
    for &md in &month_days {
        if remaining < md as i64 { break; }
        remaining -= md as i64;
        m += 1;
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m + 1, remaining + 1, hours, minutes)
}

fn top_score(store: &ScoreStore, game: &str) -> Option<u64> {
    store
        .scores
        .get(game)
        .and_then(|entries| entries.first())
        .map(|e| e.score)
}

// === APP STATE ===

enum Mode {
    Normal,
    ScoreInput { buffer: String },
    ViewLog { scroll: u16 },
    Installing {
        lines: Arc<Mutex<Vec<String>>>,
        done: Arc<Mutex<Option<bool>>>,
        scroll: u16,
    },
}

struct Toolchains {
    cargo: bool,
    java: bool,
    python: bool,
}

impl Toolchains {
    fn detect() -> Self {
        Self {
            cargo: which("cargo"),
            java: which("java"),
            python: which("python") || which("python3"),
        }
    }
}

struct App {
    list_state: ListState,
    installed: Vec<bool>,
    should_quit: bool,
    message: Option<(String, bool)>,
    tick: u64,
    scores: ScoreStore,
    mode: Mode,
    toolchains: Toolchains,
    last_log: Option<String>,
}

impl App {
    fn new() -> Self {
        let installed: Vec<bool> = GAMES.iter().map(|g| which(g.bin)).collect();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let toolchains = Toolchains::detect();
        let message = if !toolchains.cargo {
            Some(("cargo not found! Install Rust to manage games.".to_string(), false))
        } else {
            None
        };
        Self {
            list_state,
            installed,
            should_quit: false,
            message,
            tick: 0,
            scores: load_scores(),
            mode: Mode::Normal,
            toolchains,
            last_log: None,
        }
    }

    fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    fn next(&mut self) {
        let i = (self.selected() + 1) % GAMES.len();
        self.list_state.select(Some(i));
        self.message = None;
    }

    fn prev(&mut self) {
        let i = if self.selected() == 0 {
            GAMES.len() - 1
        } else {
            self.selected() - 1
        };
        self.list_state.select(Some(i));
        self.message = None;
    }

    fn refresh(&mut self) {
        self.installed = GAMES.iter().map(|g| which(g.bin)).collect();
        self.toolchains = Toolchains::detect();
    }
}

fn which(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let full = dir.join(bin);
                full.is_file()
                    || full.with_extension("exe").is_file()
                    || full.with_extension("cmd").is_file()
            })
        })
        .unwrap_or(false)
}

fn run_visible(
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cmd: &str,
    args: &[&str],
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

    let status = Command::new(cmd).args(args).status();

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
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    term.clear()?;

    let ok = status.map(|s| s.success()).unwrap_or(false);
    Ok((term, ok))
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    while !app.should_quit {
        // Auto-scroll install panel to bottom
        if let Mode::Installing { ref lines, ref done, ref mut scroll } = app.mode {
            let line_count = lines.lock().unwrap().len() as u16;
            *scroll = line_count.saturating_sub(1);
            // Save log when done
            if let Some(_ok) = *done.lock().unwrap() {
                app.last_log = Some(lines.lock().unwrap().join("\n"));
            }
        }

        terminal.draw(|f| ui(f, &mut app))?;
        app.tick = app.tick.wrapping_add(1);

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match &mut app.mode {
                    Mode::Installing { done, scroll, .. } => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                *scroll = scroll.saturating_add(1);
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                *scroll = scroll.saturating_sub(1);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                // Only allow exit if done
                                if done.lock().unwrap().is_some() {
                                    let ok = done.lock().unwrap().unwrap_or(false);
                                    app.refresh();
                                    let idx = app.selected();
                                    let msg = if ok {
                                        format!("{} done!", GAMES[idx].name)
                                    } else {
                                        format!("{} failed. Press [l] for log.", GAMES[idx].name)
                                    };
                                    app.message = Some((msg, ok));
                                    app.mode = Mode::Normal;
                                }
                            }
                            _ => {}
                        }
                    }
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
                    Mode::ScoreInput { buffer } => match key.code {
                        KeyCode::Enter => {
                            if let Ok(score) = buffer.parse::<u64>() {
                                let game_name = GAMES[app.selected()].name;
                                add_score(&mut app.scores, game_name, score);
                                app.message = Some((
                                    format!("Score {} saved!", score),
                                    true,
                                ));
                            } else if !buffer.is_empty() {
                                app.message =
                                    Some(("Invalid number".to_string(), false));
                            }
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.message = Some(("Score skipped".to_string(), true));
                        }
                        KeyCode::Backspace => {
                            buffer.pop();
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if buffer.len() < 12 {
                                buffer.push(c);
                            }
                        }
                        _ => {}
                    },
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev(),

                        KeyCode::Enter => {
                            let idx = app.selected();
                            if !app.installed[idx] {
                                app.message = Some((
                                    "Not installed! Press [i] to install".to_string(),
                                    false,
                                ));
                            } else {
                                let bin = GAMES[idx].bin;
                                let (t, _ok) = run_visible(terminal, bin, &[])?;
                                terminal = t;
                                // Prompt for score after game
                                app.mode = Mode::ScoreInput {
                                    buffer: String::new(),
                                };
                                app.message = None;
                            }
                        }

                        KeyCode::Char('i') | KeyCode::Char('d') => {
                            let idx = app.selected();
                            let installing = key.code == KeyCode::Char('i');
                            let already = if installing { app.installed[idx] } else { !app.installed[idx] };
                            if already {
                                let msg = if installing {
                                    format!("{} already installed!", GAMES[idx].name)
                                } else {
                                    format!("{} not installed", GAMES[idx].name)
                                };
                                app.message = Some((msg, installing));
                            } else {
                                let action = if installing { "install" } else { "uninstall" };
                                let lines = Arc::new(Mutex::new(vec![
                                    format!("$ cargo {} {}", action, GAMES[idx].crate_name),
                                    String::new(),
                                ]));
                                let done = Arc::new(Mutex::new(None));

                                let lines_clone = Arc::clone(&lines);
                                let done_clone = Arc::clone(&done);
                                let cmd_action = action.to_string();
                                let cmd_crate = GAMES[idx].crate_name.to_string();

                                thread::spawn(move || {
                                    let result = Command::new("cargo")
                                        .args([&cmd_action, &cmd_crate])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::piped())
                                        .spawn();

                                    match result {
                                        Ok(mut child) => {
                                            // Read stderr (cargo writes progress there)
                                            let stderr = child.stderr.take();
                                            let stdout = child.stdout.take();
                                            let lines2 = Arc::clone(&lines_clone);
                                            let stderr_thread = thread::spawn(move || {
                                                if let Some(stderr) = stderr {
                                                    for line in BufReader::new(stderr).lines().flatten() {
                                                        lines2.lock().unwrap().push(line);
                                                    }
                                                }
                                            });
                                            let lines3 = Arc::clone(&lines_clone);
                                            let stdout_thread = thread::spawn(move || {
                                                if let Some(stdout) = stdout {
                                                    for line in BufReader::new(stdout).lines().flatten() {
                                                        lines3.lock().unwrap().push(line);
                                                    }
                                                }
                                            });
                                            let _ = stderr_thread.join();
                                            let _ = stdout_thread.join();
                                            let ok = child.wait().map(|s| s.success()).unwrap_or(false);
                                            *done_clone.lock().unwrap() = Some(ok);
                                        }
                                        Err(e) => {
                                            lines_clone.lock().unwrap().push(format!("Error: {}", e));
                                            *done_clone.lock().unwrap() = Some(false);
                                        }
                                    }
                                });

                                app.mode = Mode::Installing { lines, done, scroll: 0 };
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

fn banner_color(tick: u64, col: u16) -> Color {
    let hue = ((tick * 3 + col as u64 * 8) % 360) as f64;
    let (r, g, b) = hsl_to_rgb(hue, 0.7, 0.6);
    Color::Rgb(r, g, b)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn panel_block(title: &str, color: Color) -> Block<'_> {
    Block::default()
        .title(Line::from(vec![
            Span::styled(" [ ", Style::default().fg(Color::DarkGray)),
            Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(" ] ", Style::default().fg(Color::DarkGray)),
        ]))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 100)))
}

fn detail_row<'a>(label: &'a str, value: &'a str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Rgb(100, 100, 130))),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(size);

    // === BANNER ===
    let banner_area = outer[0];
    let mut banner_lines: Vec<Line> = Vec::new();

    let bar_width = banner_area.width.saturating_sub(2) as usize;
    let top_bar: String = (0..bar_width)
        .map(|i| {
            let chars = ['=', '~', '*', '~'];
            chars[(i + app.tick as usize) % chars.len()]
        })
        .collect();
    banner_lines.push(Line::from(Span::styled(
        top_bar,
        Style::default().fg(Color::Rgb(80, 80, 120)),
    )));

    for line in BANNER {
        let spans: Vec<Span> = line
            .chars()
            .enumerate()
            .map(|(col, ch)| {
                let color = if ch == ' ' {
                    Color::Reset
                } else {
                    banner_color(app.tick, col as u16)
                };
                Span::styled(ch.to_string(), Style::default().fg(color))
            })
            .collect();
        banner_lines.push(Line::from(spans));
    }

    banner_lines.push(Line::from(Span::styled(
        SUB_BANNER,
        Style::default()
            .fg(Color::Rgb(180, 180, 220))
            .add_modifier(Modifier::BOLD),
    )));

    let banner_widget = Paragraph::new(banner_lines).alignment(Alignment::Center);
    f.render_widget(banner_widget, banner_area);

    // === MAIN CONTENT ===
    let main_area = outer[1];

    if let Mode::ViewLog { scroll } = &app.mode {
        let log_text = app.last_log.as_deref().unwrap_or("(no log)");
        let lines: Vec<Line> = log_text
            .lines()
            .map(|l| Line::from(Span::styled(l, Style::default().fg(Color::Rgb(200, 200, 220)))))
            .collect();
        let log_block = panel_block("LOG", Color::Yellow);
        let log_widget = Paragraph::new(lines)
            .block(log_block)
            .scroll((*scroll, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(log_widget, main_area);

        // Simplified footer for log view
        let footer_spans = vec![
            Span::styled(" j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" page ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" close", Style::default().fg(Color::DarkGray)),
        ];
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 100)));
        let footer = Paragraph::new(Line::from(footer_spans)).block(footer_block);
        f.render_widget(footer, outer[2]);
        return;
    }
    let is_installing = matches!(app.mode, Mode::Installing { .. });
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if is_installing {
            vec![
                Constraint::Length(26),        // game list
                Constraint::Percentage(40),    // details
                Constraint::Min(30),           // install output
            ]
        } else {
            vec![
                Constraint::Length(26),        // game list
                Constraint::Min(30),           // details
            ]
        })
        .split(main_area);

    // --- Game List ---
    let items: Vec<ListItem> = GAMES
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let (dot, dot_color) = if app.installed[i] {
                ("*", Color::Green)
            } else {
                ("-", Color::Rgb(80, 80, 80))
            };
            let name_style = if app.installed[i] {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Rgb(100, 100, 100))
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", dot),
                    Style::default().fg(dot_color),
                ),
                Span::styled(
                    format!("[{}] ", g.icon),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
                Span::styled(g.name, name_style),
            ]))
        })
        .collect();

    let list_block = panel_block("GAMES", Color::Cyan);

    let list = List::new(items).block(list_block).highlight_style(
        Style::default()
            .bg(Color::Rgb(30, 30, 55))
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(list, cols[0], &mut app.list_state);

    // --- Details + Scores Panel ---
    let idx = app.selected();
    let game = &GAMES[idx];
    let installed = app.installed[idx];

    let best = top_score(&app.scores, game.name);

    let (status_text, status_color) = if installed {
        ("INSTALLED", Color::Green)
    } else {
        ("NOT INSTALLED", Color::Red)
    };

    let runtime_available = match game.runtime {
        "cargo" => app.toolchains.cargo,
        "java" => app.toolchains.java,
        "python" => app.toolchains.python,
        _ => false,
    };

    let mut details = vec![
        Line::from(vec![
            Span::styled(
                format!("[{}] ", game.icon),
                Style::default().fg(Color::Rgb(100, 80, 160)),
            ),
            Span::styled(
                game.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                status_text,
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                game.desc,
                Style::default().fg(Color::Rgb(200, 200, 220)),
            ),
        ]),
        Line::from(""),
        detail_row("  Type:    ", game.category, Color::Yellow),
        detail_row("  Keys:    ", game.keys, Color::Rgb(220, 180, 100)),
        Line::from(""),
        Line::from(Span::styled(
            "  ── TECHNICAL ──",
            Style::default().fg(Color::Rgb(100, 100, 160)).add_modifier(Modifier::BOLD),
        )),
        detail_row("  Bin:     ", game.bin, Color::Green),
        detail_row("  Crate:   ", game.crate_name, Color::Rgb(180, 140, 220)),
        detail_row("  Source:  ", game.repo, Color::Rgb(100, 160, 220)),
        Line::from(vec![
            Span::styled("  Runtime: ", Style::default().fg(Color::Rgb(100, 100, 130))),
            Span::styled(game.runtime, Style::default().fg(if runtime_available { Color::Green } else { Color::Red })),
            Span::styled(
                if runtime_available { " (found)" } else { " (missing!)" },
                Style::default().fg(if runtime_available { Color::DarkGray } else { Color::Red }),
            ),
        ]),
    ];
    let install_cmd = format!("cargo install {}", game.crate_name);
    details.push(detail_row("  Install: ", &install_cmd, Color::Rgb(150, 150, 170)));

    if let Some(score) = best {
        details.push(Line::from(vec![
            Span::styled(
                "  Best:  ",
                Style::default().fg(Color::Rgb(100, 100, 130)),
            ),
            Span::styled(
                format!("{}", score),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // High scores section
    if let Some(entries) = app.scores.scores.get(game.name) {
        if !entries.is_empty() {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "  ── HIGH SCORES ──",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for (rank, entry) in entries.iter().enumerate().take(5) {
                let medal = match rank {
                    0 => "1st",
                    1 => "2nd",
                    2 => "3rd",
                    _ => "   ",
                };
                let medal_color = match rank {
                    0 => Color::Yellow,
                    1 => Color::Rgb(180, 180, 200),
                    2 => Color::Rgb(180, 120, 60),
                    _ => Color::DarkGray,
                };
                details.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", medal),
                        Style::default().fg(medal_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<8}", entry.score),
                        Style::default()
                            .fg(if rank == 0 {
                                Color::Yellow
                            } else {
                                Color::White
                            })
                            .add_modifier(if rank == 0 {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    Span::styled(
                        &entry.date,
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
    }

    // Score input prompt
    if let Mode::ScoreInput { ref buffer } = app.mode {
        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "  ── ENTER SCORE ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        let cursor_char = if app.tick % 6 < 3 { "█" } else { " " };
        details.push(Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Cyan)),
            Span::styled(
                buffer.as_str(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(cursor_char, Style::default().fg(Color::Cyan)),
        ]));
        details.push(Line::from(Span::styled(
            "  Enter to save, Esc to skip",
            Style::default().fg(Color::DarkGray),
        )));
    }

    if let Some((ref msg, is_ok)) = app.message {
        details.push(Line::from(""));
        let prefix = if is_ok { "  >> " } else { "  !! " };
        details.push(Line::from(Span::styled(
            format!("{}{}", prefix, msg),
            Style::default()
                .fg(if is_ok { Color::Green } else { Color::Red })
                .add_modifier(Modifier::BOLD),
        )));
    }

    let detail_block = panel_block("INFO", Color::Yellow);

    let detail_widget = Paragraph::new(details)
        .wrap(Wrap { trim: false })
        .block(detail_block);
    f.render_widget(detail_widget, cols[1]);

    // --- Install Output Panel (third column) ---
    if let Mode::Installing { ref lines, ref done, ref scroll } = app.mode {
        let locked = lines.lock().unwrap();
        let is_done = done.lock().unwrap().is_some();
        let ok = done.lock().unwrap().unwrap_or(false);

        let mut output_lines: Vec<Line> = locked.iter().map(|l| {
            let color = if l.starts_with("   Compiling") {
                Color::Cyan
            } else if l.starts_with(" Downloading") || l.starts_with("  Downloaded") {
                Color::Rgb(120, 120, 180)
            } else if l.starts_with("    Finished") || l.starts_with("  Installing") || l.starts_with("   Installed") {
                Color::Green
            } else if l.starts_with("error") || l.starts_with("Error") {
                Color::Red
            } else if l.starts_with("warning") {
                Color::Yellow
            } else if l.starts_with("$") {
                Color::Rgb(180, 180, 220)
            } else {
                Color::Rgb(150, 150, 160)
            };
            Line::from(Span::styled(l.as_str(), Style::default().fg(color)))
        }).collect();

        if is_done {
            output_lines.push(Line::from(""));
            if ok {
                output_lines.push(Line::from(Span::styled(
                    "  Done! Press Esc to close.",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
            } else {
                output_lines.push(Line::from(Span::styled(
                    "  Failed! Press Esc to close.",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                output_lines.push(Line::from(Span::styled(
                    "  Full log available with [l]",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        } else {
            let dots = ".".repeat((app.tick as usize % 4) + 1);
            output_lines.push(Line::from(""));
            output_lines.push(Line::from(Span::styled(
                format!("  Working{}", dots),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
        }

        let title = if is_done {
            if ok { "COMPLETE" } else { "FAILED" }
        } else {
            "CARGO"
        };
        let title_color = if is_done {
            if ok { Color::Green } else { Color::Red }
        } else {
            Color::Yellow
        };
        let install_block = panel_block(title, title_color);
        let install_widget = Paragraph::new(output_lines)
            .block(install_block)
            .scroll((*scroll, 0));
        f.render_widget(install_widget, cols[2]);
    }

    // === FOOTER ===
    let footer_area = outer[2];
    let footer_width = footer_area.width.saturating_sub(2) as usize; // inside borders

    let mut left_spans: Vec<Span> = Vec::new();
    if let Mode::Installing { ref done, .. } = app.mode {
        let is_done = done.lock().unwrap().is_some();
        if is_done {
            left_spans.push(Span::styled(" Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            left_spans.push(Span::styled(" close", Style::default().fg(Color::DarkGray)));
        } else {
            left_spans.push(Span::styled(" j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            left_spans.push(Span::styled(" scroll ", Style::default().fg(Color::DarkGray)));
            left_spans.push(Span::styled("| ", Style::default().fg(Color::Rgb(50, 50, 80))));
            left_spans.push(Span::styled("cargo running...", Style::default().fg(Color::Yellow)));
        }
    } else if matches!(app.mode, Mode::ScoreInput { .. }) {
        left_spans.push(Span::styled(" Score: ", Style::default().fg(Color::Cyan)));
        left_spans.push(Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        left_spans.push(Span::styled(" save ", Style::default().fg(Color::DarkGray)));
        left_spans.push(Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
        left_spans.push(Span::styled(" skip", Style::default().fg(Color::DarkGray)));
    } else {
        let mut keys: Vec<(&str, &str, Color)> = vec![
            ("j/k", "nav", Color::Cyan),
            ("Enter", "play", Color::Green),
            ("i", "install", Color::Yellow),
            ("d", "remove", Color::Red),
            ("r", "refresh", Color::Magenta),
        ];
        if app.last_log.is_some() {
            keys.push(("l", "log", Color::Rgb(180, 160, 100)));
        }
        keys.push(("q", "quit", Color::Rgb(180, 80, 80)));
        let keys = &keys;
        left_spans.push(Span::styled(" ", Style::default()));
        for (i, (key, label, color)) in keys.iter().enumerate() {
            left_spans.push(Span::styled(*key, Style::default().fg(*color).add_modifier(Modifier::BOLD)));
            left_spans.push(Span::styled(format!(" {}", label), Style::default().fg(Color::DarkGray)));
            if i < keys.len() - 1 {
                left_spans.push(Span::styled(" | ", Style::default().fg(Color::Rgb(50, 50, 80))));
            }
        }
    }

    // Right side: toolchains + game count
    let installed_count = app.installed.iter().filter(|&&x| x).count();
    let right_parts: &[(&str, bool)] = &[
        ("cargo", app.toolchains.cargo),
        ("java", app.toolchains.java),
        ("python", app.toolchains.python),
    ];
    let mut right_spans: Vec<Span> = Vec::new();
    for (name, found) in right_parts {
        let (fg, bg, label) = if *found {
            (Color::Black, Color::Green, format!(" {} ", name))
        } else {
            (Color::DarkGray, Color::Rgb(40, 40, 40), format!(" {} ", name))
        };
        right_spans.push(Span::styled(label, Style::default().fg(fg).bg(bg)));
        right_spans.push(Span::styled(" ", Style::default()));
    }
    right_spans.push(Span::styled(
        format!(" {}/{} ", installed_count, GAMES.len()),
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));

    // Calculate padding to right-align the right side
    let left_len: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_len: usize = right_spans.iter().map(|s| s.width()).sum();
    let pad = footer_width.saturating_sub(left_len + right_len);

    let mut footer_spans = left_spans;
    footer_spans.push(Span::styled(" ".repeat(pad), Style::default()));
    footer_spans.extend(right_spans);

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 100)));
    let footer = Paragraph::new(Line::from(footer_spans)).block(footer_block);
    f.render_widget(footer, footer_area);
}
