use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Mode};
use crate::catalog::{self, GAMES};
use crate::util::format_size;

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
            Span::styled(
                title,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ] ", Style::default().fg(Color::DarkGray)),
        ]))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 100)))
}

fn detail_row<'a>(label: &'a str, value: &'a str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Rgb(140, 140, 170))),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(size);

    render_banner(f, app, outer[0]);

    if let Mode::ViewLog { scroll } = &app.mode {
        render_log_view(f, app, outer[1], *scroll);
        render_log_footer(f, outer[2]);
        return;
    }

    render_main(f, app, outer[1]);
    render_footer(f, app, outer[2]);
}

fn render_banner(f: &mut Frame, app: &App, area: Rect) {
    let mut banner_lines: Vec<Line> = Vec::new();

    let bar_width = area.width.saturating_sub(2) as usize;
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
    f.render_widget(banner_widget, area);
}

fn render_log_view(f: &mut Frame, app: &App, area: Rect, scroll: u16) {
    let log_text = app.last_log.as_deref().unwrap_or("(no log)");
    let lines: Vec<Line> = log_text
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(Color::Rgb(200, 200, 220)))))
        .collect();
    let log_block = panel_block("LOG", Color::Yellow);
    let log_widget = Paragraph::new(lines)
        .block(log_block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(log_widget, area);
}

fn render_log_footer(f: &mut Frame, area: Rect) {
    let footer_spans = vec![
        Span::styled(
            " j/k",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "PgUp/PgDn",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" page ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" close", Style::default().fg(Color::DarkGray)),
    ];
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 100)));
    let footer = Paragraph::new(Line::from(footer_spans)).block(footer_block);
    f.render_widget(footer, area);
}

fn render_main(f: &mut Frame, app: &mut App, area: Rect) {
    let is_installing = matches!(app.mode, Mode::Installing { .. });
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if is_installing {
            vec![
                Constraint::Length(26),
                Constraint::Percentage(40),
                Constraint::Min(30),
            ]
        } else {
            vec![Constraint::Length(26), Constraint::Min(30)]
        })
        .split(area);

    render_game_list(f, app, cols[0]);
    render_details(f, app, cols[1]);

    if is_installing {
        render_install_panel(f, app, cols[2]);
    }
}

fn render_game_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = GAMES
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let supported = catalog::game_supports_platform(g);
            let cloned_not_ready = crate::util::is_git_cloned_not_ready(g);
            let (dot, dot_color) = if !supported {
                ("x", Color::Rgb(100, 100, 110))
            } else if app.installed[i] {
                ("*", Color::Green)
            } else if cloned_not_ready {
                ("~", Color::Yellow)
            } else {
                ("-", Color::Rgb(120, 120, 130))
            };
            let name_style = if !supported {
                Style::default().fg(Color::Rgb(100, 100, 110))
            } else if app.installed[i] {
                Style::default().fg(Color::White)
            } else if cloned_not_ready {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(140, 140, 150))
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", dot), Style::default().fg(dot_color)),
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
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_details(f: &mut Frame, app: &App, area: Rect) {
    let idx = app.selected();
    let game = &GAMES[idx];
    let installed = app.installed[idx];
    let cloned_not_ready = crate::util::is_git_cloned_not_ready(game);
    let (status_text, status_color) = if installed {
        ("READY", Color::Green)
    } else if cloned_not_ready {
        ("CLONED — press [i] to build", Color::Yellow)
    } else {
        ("NOT INSTALLED", Color::Red)
    };

    let source = catalog::default_source(game);
    let method = source.map(|s| s.method).unwrap_or("unknown");
    let runtime_available = crate::util::has_runtime(&app.toolchains, method);

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
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(game.desc, Style::default().fg(Color::Rgb(200, 200, 220))),
        ]),
        Line::from(""),
        detail_row("  Type:    ", game.category, Color::Yellow),
        detail_row("  Keys:    ", game.keys, Color::Rgb(220, 180, 100)),
        Line::from(""),
        Line::from(Span::styled(
            "  ── TECHNICAL ──",
            Style::default()
                .fg(Color::Rgb(100, 100, 160))
                .add_modifier(Modifier::BOLD),
        )),
        detail_row("  Engine:  ", game.engine, Color::Rgb(200, 160, 100)),
    ];

    if let Some(src) = source {
        details.push(detail_row("  Method:  ", src.label, Color::Rgb(180, 140, 220)));
        if !src.bin.is_empty() {
            details.push(detail_row("  Bin:     ", src.bin, Color::Green));
        }
    }

    details.push(detail_row(
        "  Source:  ",
        game.repo,
        Color::Rgb(100, 160, 220),
    ));
    details.push(Line::from(vec![
        Span::styled(
            "  Runtime: ",
            Style::default().fg(Color::Rgb(140, 140, 170)),
        ),
        Span::styled(
            method,
            Style::default().fg(if runtime_available {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::styled(
            if runtime_available {
                " (found)"
            } else {
                " (missing!)"
            },
            Style::default().fg(if runtime_available {
                Color::DarkGray
            } else {
                Color::Red
            }),
        ),
    ]));

    let size_str = if installed {
        app.sizes[idx]
            .map(format_size)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "-".to_string()
    };
    details.push(detail_row(
        "  Size:    ",
        &size_str,
        Color::Rgb(150, 150, 170),
    ));

    // Platform support
    let supported = catalog::game_supports_platform(game);
    let platform_label = if game.platforms.is_empty() {
        "all".to_string()
    } else {
        game.platforms.join(", ")
    };
    details.push(Line::from(vec![
        Span::styled("  Platform:", Style::default().fg(Color::Rgb(140, 140, 170))),
        Span::styled(
            format!(" {}", platform_label),
            Style::default().fg(if supported { Color::Green } else { Color::Rgb(140, 140, 150) }),
        ),
        if !supported {
            Span::styled(
                format!(" (not {})", catalog::current_platform()),
                Style::default().fg(Color::Red),
            )
        } else {
            Span::styled("", Style::default())
        },
    ]));

    // Platform-specific deps
    if let Some(deps) = catalog::platform_deps_for_current(game) {
        let satisfied = crate::util::deps_check_satisfied(deps);
        if satisfied {
            details.push(Line::from(vec![
                Span::styled("  Deps:    ", Style::default().fg(Color::Rgb(140, 140, 170))),
                Span::styled(deps.deps, Style::default().fg(Color::Green)),
                Span::styled(" (installed)", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            details.push(Line::from(vec![
                Span::styled("  Deps:    ", Style::default().fg(Color::Rgb(140, 140, 170))),
                Span::styled(deps.deps, Style::default().fg(Color::Yellow)),
                Span::styled(" (needed)", Style::default().fg(Color::Yellow)),
            ]));
            if !deps.install_cmd.is_empty() {
                let cmd_str = deps.install_cmd.join(" ");
                details.push(Line::from(vec![
                    Span::styled("           ", Style::default()),
                    Span::styled(cmd_str, Style::default().fg(Color::DarkGray)),
                ]));
                if deps.needs_sudo {
                    details.push(Line::from(vec![
                        Span::styled("           ", Style::default()),
                        Span::styled("(requires admin)", Style::default().fg(Color::Red)),
                    ]));
                }
            }
        }
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
    f.render_widget(detail_widget, area);
}

fn render_install_panel(f: &mut Frame, app: &App, area: Rect) {
    if let Mode::Installing {
        ref lines,
        ref done,
        ref scroll,
        label,
        ..
    } = app.mode
    {
        let locked = lines.lock().unwrap();
        let done_val = *done.lock().unwrap();
        let is_done = done_val.is_some();
        let ok = done_val.unwrap_or(false);
        let is_running = label == "RUNNING";

        let mut output_lines: Vec<Line> = locked
            .iter()
            .map(|l| {
                let color = if l.starts_with("   Compiling") {
                    Color::Cyan
                } else if l.starts_with(" Downloading") || l.starts_with("  Downloaded") {
                    Color::Rgb(120, 120, 180)
                } else if l.starts_with("    Finished")
                    || l.starts_with("  Installing")
                    || l.starts_with("   Installed")
                {
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
            })
            .collect();

        if is_done {
            output_lines.push(Line::from(""));
            if ok {
                let msg = if is_running { "  Exited. Press Esc to close." } else { "  Done! Press Esc to close." };
                output_lines.push(Line::from(Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
            } else {
                let msg = if is_running { "  Crashed! Press Esc to close." } else { "  Failed! Press Esc to close." };
                output_lines.push(Line::from(Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )));
                output_lines.push(Line::from(Span::styled(
                    "  Full log available with [l]",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        } else {
            let dots = ".".repeat((app.tick as usize % 4) + 1);
            output_lines.push(Line::from(""));
            let msg = if is_running {
                format!("  Running{} (Esc to stop)", dots)
            } else {
                format!("  Working{}", dots)
            };
            output_lines.push(Line::from(Span::styled(
                msg,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let title = if is_done {
            if ok { "EXITED" } else { "FAILED" }
        } else {
            label
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
        f.render_widget(install_widget, area);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_width = area.width.saturating_sub(2) as usize;

    let mut left_spans: Vec<Span> = Vec::new();
    if let Mode::Installing { ref done, label, .. } = app.mode {
        let is_done = done.lock().unwrap().is_some();
        let is_running = label == "RUNNING";
        if is_done {
            left_spans.push(Span::styled(
                " Esc",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ));
            left_spans.push(Span::styled(" close", Style::default().fg(Color::DarkGray)));
        } else {
            left_spans.push(Span::styled(
                " j/k",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            left_spans.push(Span::styled(" scroll ", Style::default().fg(Color::DarkGray)));
            if is_running {
                left_spans.push(Span::styled(
                    "| ",
                    Style::default().fg(Color::Rgb(50, 50, 80)),
                ));
                left_spans.push(Span::styled(
                    "Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
                left_spans.push(Span::styled(" stop", Style::default().fg(Color::DarkGray)));
            }
        }
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
        left_spans.push(Span::styled(" ", Style::default()));
        for (i, (key, label, color)) in keys.iter().enumerate() {
            left_spans.push(Span::styled(
                *key,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
            left_spans.push(Span::styled(
                format!(" {}", label),
                Style::default().fg(Color::DarkGray),
            ));
            if i < keys.len() - 1 {
                left_spans.push(Span::styled(
                    " | ",
                    Style::default().fg(Color::Rgb(50, 50, 80)),
                ));
            }
        }
    }

    // Right side: toolchains + game count
    let installed_count = app.installed.iter().filter(|&&x| x).count();
    let right_parts: &[(&str, bool)] = &[
        ("cargo", app.toolchains.cargo),
        ("python", app.toolchains.python),
        ("cmake", app.toolchains.cmake),
    ];
    let mut right_spans: Vec<Span> = Vec::new();
    for (name, found) in right_parts {
        let (fg, bg, label) = if *found {
            (Color::Black, Color::Green, format!(" {} ", name))
        } else {
            (
                Color::DarkGray,
                Color::Rgb(40, 40, 40),
                format!(" {} ", name),
            )
        };
        right_spans.push(Span::styled(label, Style::default().fg(fg).bg(bg)));
        right_spans.push(Span::styled(" ", Style::default()));
    }
    right_spans.push(Span::styled(
        format!(" {}/{} ", installed_count, GAMES.len()),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));

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
    f.render_widget(footer, area);
}
