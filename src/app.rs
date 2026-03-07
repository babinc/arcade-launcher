use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

use crate::catalog::GAMES;
use crate::util::{bin_path, install_size, Toolchains};

pub enum Mode {
    Normal,
    ViewLog { scroll: u16 },
    Installing {
        lines: Arc<Mutex<Vec<String>>>,
        done: Arc<Mutex<Option<bool>>>,
        scroll: u16,
        label: &'static str,
        pid: Arc<Mutex<Option<u32>>>,
    },
}

pub struct App {
    pub list_state: ListState,
    pub installed: Vec<bool>,
    pub sizes: Vec<Option<u64>>,
    pub should_quit: bool,
    pub message: Option<(String, bool)>,
    pub tick: u64,
    pub mode: Mode,
    pub toolchains: Toolchains,
    pub last_log: Option<String>,
}

fn compute_installed() -> Vec<bool> {
    GAMES
        .iter()
        .map(|g| {
            let source = match crate::catalog::default_source(g) {
                Some(s) => s,
                None => return false,
            };
            match source.method {
                "cmake" => {
                    let mut p = crate::util::games_dir();
                    p.push(source.clone_dir);
                    p.push("build");
                    p.is_dir()
                }
                "git" => {
                    let p = crate::util::games_dir().join(source.clone_dir);
                    p.join(".arcade-ready").is_file()
                }
                _ => bin_path(source.bin).is_some(),
            }
        })
        .collect()
}

fn compute_sizes(installed: &[bool]) -> Vec<Option<u64>> {
    GAMES
        .iter()
        .enumerate()
        .map(|(i, g)| {
            if installed[i] {
                install_size(g)
            } else {
                None
            }
        })
        .collect()
}

impl App {
    pub fn new() -> Self {
        let installed = compute_installed();
        let sizes = compute_sizes(&installed);
        let toolchains = Toolchains::detect();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let message = if !toolchains.cargo {
            Some((
                "cargo not found! Install: https://rustup.rs".to_string(),
                false,
            ))
        } else {
            None
        };
        Self {
            list_state,
            installed,
            sizes,
            should_quit: false,
            message,
            tick: 0,
            mode: Mode::Normal,
            toolchains,
            last_log: None,
        }
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn next(&mut self) {
        let i = (self.selected() + 1) % GAMES.len();
        self.list_state.select(Some(i));
        self.message = None;
    }

    pub fn prev(&mut self) {
        let i = if self.selected() == 0 {
            GAMES.len() - 1
        } else {
            self.selected() - 1
        };
        self.list_state.select(Some(i));
        self.message = None;
    }

    pub fn refresh(&mut self) {
        self.installed = compute_installed();
        self.sizes = compute_sizes(&self.installed);
        self.toolchains = Toolchains::detect();
    }
}
