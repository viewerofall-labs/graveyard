use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

mod store;

// ── FART ascii art ────────────────────────────────────────────────────────────

fn fart_art() -> Vec<&'static str> {
    vec![
        " ███████╗ █████╗ ██████╗ ████████╗",
        " ██╔════╝██╔══██╗██╔══██╗╚══██╔══╝",
        " █████╗  ███████║██████╔╝   ██║   ",
        " ██╔══╝  ██╔══██║██╔══██╗   ██║   ",
        " ██║     ██║  ██║██║  ██║   ██║   ",
        " ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ",
    ]
}

// ── Rank ──────────────────────────────────────────────────────────────────────

fn get_rank(words: u64) -> String {
    let tier = words / 10;
    match tier {
        0  => "Fart Novice".into(),
        1  => "Fart Appreciator".into(),
        2  => "Fart Enthusiast".into(),
        3  => "Fart Connoisseur".into(),
        4  => "Fart Pilgrim".into(),
        5  => "Fart Devotee".into(),
        6  => "Fart Sage".into(),
        7  => "Fart Archon".into(),
        8  => "Fart Deity".into(),
        9  => "Fart Transcended".into(),
        10..=19 => "Fart Void Walker".into(),
        20..=29 => "Fart Reality Glitch".into(),
        30..=39 => "Fart Entropy God".into(),
        40..=49 => "Fart Dimension Lord".into(),
        50..=79 => {
            let suffixes = ["Sovereign", "Obliterator", "Manifold", "Singularity", "Paradox"];
            format!("Trans-Fart {}", suffixes[(tier as usize) % suffixes.len()])
        }
        _ => {
            let h = tier.wrapping_mul(0x9e3779b97f4a7c15) ^ (tier >> 30);
            let prefixes = ["Hyper", "Meta", "Ultra", "Void", "Null", "Omni"];
            let suffixes = ["Fart", "FART", "F.A.R.T", "fart?", "fart!", "...fart"];
            format!(
                "{}-{}",
                prefixes[(h as usize) % prefixes.len()],
                suffixes[((h >> 8) as usize) % suffixes.len()]
            )
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

/// What the app is doing right now
enum Screen {
    /// User is setting their target text
    Setting { draft: String },
    /// User is typing their target
    Typing,
}

struct App {
    screen: Screen,

    /// The confirmed target to type
    target: String,
    /// What the user has typed so far in Typing mode
    input: String,

    // stats
    words_all_time: u64,
    completions_all_time: u64,
    saved_secs: u64,
    session_words: u64,
    session_completions: u64,
    session_start: Instant,

    round_start: Option<Instant>,
    last_wpm: f64,

    flash: Option<Instant>,

    messages: Vec<&'static str>,
    message_idx: usize,
}

impl App {
    fn new() -> Self {
        let (words_all_time, completions_all_time, saved_secs) = store::load();
        Self {
            // Start in setter — no target yet
            screen: Screen::Setting { draft: String::new() },
            target: String::new(),
            input: String::new(),
            words_all_time,
            completions_all_time,
            saved_secs,
            session_words: 0,
            session_completions: 0,
            session_start: Instant::now(),
            round_start: None,
            last_wpm: 0.0,
            flash: None,
            messages: vec![
                "nice.", "clean.", "smooth.", "fast.", "again.",
                "let's go.", "keep it up.", "wordsmith.", "effortless.",
                "on a roll.", "don't stop.", "flawless.", "blaze it.",
            ],
            message_idx: 0,
        }
    }

    fn confirm_target(&mut self, draft: String) {
        let trimmed = draft.trim().to_string();
        if trimmed.is_empty() { return; }
        self.target = trimmed;
        self.input.clear();
        self.round_start = None;
        self.screen = Screen::Typing;
    }

    fn on_complete(&mut self) {
        let elapsed = self.round_start
            .map(|s| s.elapsed().as_secs_f64())
            .unwrap_or(1.0)
            .max(0.1);

        let wc = self.target.split_whitespace().count() as f64;
        self.last_wpm = (wc / elapsed) * 60.0;

        let wc_u = wc as u64;
        self.words_all_time += wc_u;
        self.session_words += wc_u;
        self.completions_all_time += 1;
        self.session_completions += 1;

        self.flash = Some(Instant::now());
        self.message_idx = (self.message_idx + 1) % self.messages.len();
        store::save(self.words_all_time, self.completions_all_time, self.total_secs());

        // Stay on same target, just clear input for next rep
        self.input.clear();
        self.round_start = None;
    }

    fn total_secs(&self) -> u64 {
        self.saved_secs + self.session_start.elapsed().as_secs()
    }

    fn is_flashing(&self) -> bool {
        self.flash
            .map(|t| t.elapsed() < Duration::from_millis(120))
            .unwrap_or(false)
    }

    fn current_message(&self) -> &'static str {
        self.messages[self.message_idx]
    }

    /// Split target into (correct, cursor_char, rest, has_error)
    fn split_input(&self) -> (String, Option<char>, String, bool) {
        let tc: Vec<char> = self.target.chars().collect();
        let ic: Vec<char> = self.input.chars().collect();

        let mut correct_len = 0;
        let mut has_error = false;

        for (i, ic) in ic.iter().enumerate() {
            if i >= tc.len() { has_error = true; break; }
            if *ic == tc[i] {
                if !has_error { correct_len = i + 1; }
            } else {
                has_error = true;
            }
        }

        let correct: String = tc[..correct_len].iter().collect();
        let cursor_char = tc.get(correct_len).copied();
        let rest: String = if correct_len + 1 < tc.len() {
            tc[correct_len + 1..].iter().collect()
        } else {
            String::new()
        };

        (correct, cursor_char, rest, has_error)
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    // CLI: fart -r → wipe save
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("-r") {
        let (words, completions, secs) = store::load();
        if words == 0 && completions == 0 {
            println!("No save data found. Nothing to reset.");
            return Ok(());
        }
        println!("Current save:");
        println!("  words all-time       : {}", words);
        println!("  completions all-time : {}", completions);
        println!("  time logged          : {}m {}s", secs / 60, secs % 60);
        println!();
        print!("Wipe all progress? This cannot be undone. [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            store::wipe();
            println!("Wiped. You are nothing again.");
        } else {
            println!("Aborted. Your words are safe.");
        }
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick = Duration::from_millis(50);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let flash = app.is_flashing();
            let accent = if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(180, 120, 255) };

            // Background
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Rgb(10, 10, 14))),
                size,
            );

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(size);

            // ── Header ──────────────────────────────────────────────────
            let screen_label = match &app.screen {
                Screen::Setting { .. } => "set target",
                Screen::Typing => "typing",
            };
            let header = Paragraph::new(Line::from(vec![
                Span::styled("  ◆ ", Style::default().fg(accent)),
                Span::styled(
                    screen_label,
                    Style::default()
                        .fg(if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(180, 180, 200) })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ◆  ", Style::default().fg(accent)),
                Span::styled(
                    get_rank(app.words_all_time),
                    Style::default()
                        .fg(Color::Rgb(80, 160, 80))
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::styled(
                    "   tab: change target",
                    Style::default().fg(Color::Rgb(50, 50, 70)),
                ),
            ]))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::Rgb(40, 40, 60))),
            );
            f.render_widget(header, chunks[0]);

            // ── Main: left | right ───────────────────────────────────────
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(chunks[1]);

            // Art
            let mut left_lines: Vec<Line> = fart_art()
                .iter()
                .map(|l| Line::from(Span::styled(
                    *l,
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )))
                .collect();
            left_lines.push(Line::from(""));

            match &app.screen {
                Screen::Setting { draft } => {
                    // ── Target setter ────────────────────────────────────
                    left_lines.push(Line::from(vec![
                        Span::styled(
                            "  type your target: ",
                            Style::default().fg(Color::Rgb(100, 180, 255)),
                        ),
                    ]));
                    left_lines.push(Line::from(""));
                    left_lines.push(Line::from(vec![
                        Span::styled("  › ", Style::default().fg(Color::Rgb(100, 180, 255))),
                        Span::styled(
                            draft.clone(),
                            Style::default()
                                .fg(Color::Rgb(220, 220, 255))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            "█",
                            Style::default()
                                .fg(Color::Rgb(180, 120, 255))
                                .add_modifier(Modifier::SLOW_BLINK),
                        ),
                    ]));
                    left_lines.push(Line::from(""));
                    left_lines.push(Line::from(Span::styled(
                        "  enter to confirm",
                        Style::default()
                            .fg(Color::Rgb(60, 60, 90))
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                Screen::Typing => {
                    // ── Target prompt (colour-coded) ─────────────────────
                    let (correct, cursor_ch, rest, has_error) = app.split_input();
                    let mut prompt = vec![Span::styled("  ", Style::default())];
                    prompt.push(Span::styled(correct, Style::default().fg(Color::Rgb(80, 200, 140))));
                    if let Some(ch) = cursor_ch {
                        prompt.push(Span::styled(
                            ch.to_string(),
                            Style::default()
                                .fg(if has_error { Color::Rgb(220, 80, 80) } else { Color::Rgb(220, 220, 255) })
                                .add_modifier(Modifier::UNDERLINED),
                        ));
                    }
                    prompt.push(Span::styled(rest, Style::default().fg(Color::Rgb(60, 60, 80))));
                    left_lines.push(Line::from(prompt));
                    left_lines.push(Line::from(""));

                    // Input echo
                    left_lines.push(if app.input.is_empty() {
                        Line::from(vec![
                            Span::styled("  › ", Style::default().fg(Color::Rgb(60, 60, 80))),
                            Span::styled(
                                "_",
                                Style::default()
                                    .fg(Color::Rgb(60, 80, 100))
                                    .add_modifier(Modifier::SLOW_BLINK),
                            ),
                        ])
                    } else {
                        Line::from(vec![
                            Span::styled("  › ", Style::default().fg(Color::Rgb(180, 120, 255))),
                            Span::styled(
                                app.input.clone(),
                                Style::default()
                                    .fg(if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(200, 220, 255) })
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    });

                    left_lines.push(Line::from(""));
                    left_lines.push(Line::from(Span::styled(
                        format!("  {}", app.current_message()),
                        Style::default()
                            .fg(Color::Rgb(80, 100, 80))
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }

            f.render_widget(
                Paragraph::new(left_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(
                                if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(30, 40, 60) },
                            ))
                            .title(Span::styled(
                                " fart ",
                                Style::default().fg(Color::Rgb(60, 80, 120)),
                            )),
                    )
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false }),
                main_chunks[0],
            );

            // ── Right: stats ─────────────────────────────────────────────
            let elapsed = app.session_start.elapsed().as_secs();
            let session_wpm = if elapsed > 0 {
                (app.session_words as f64 / elapsed as f64) * 60.0
            } else {
                0.0
            };
            let count_color = if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(120, 220, 180) };
            let progress = (app.words_all_time % 10) as usize;
            let filled: String = "█".repeat(progress);
            let empty: String  = "░".repeat(10 - progress);

            let target_preview = if app.target.is_empty() {
                "—".to_string()
            } else if app.target.len() > 18 {
                format!("{}…", &app.target[..18])
            } else {
                app.target.clone()
            };

            let right_lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    app.words_all_time.to_string(),
                    Style::default().fg(count_color).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "words all time",
                    Style::default().fg(Color::Rgb(60, 80, 80)).add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  target     ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        target_preview,
                        Style::default().fg(Color::Rgb(160, 140, 200)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  completions ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        app.completions_all_time.to_string(),
                        Style::default().fg(Color::Rgb(180, 120, 255)).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  session    ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        format!("{} words", app.session_words),
                        Style::default().fg(Color::Rgb(140, 180, 140)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  last wpm   ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        format!("{:.0}", app.last_wpm),
                        Style::default().fg(Color::Rgb(200, 180, 100)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  avg wpm    ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        format!("{:.0}", session_wpm),
                        Style::default().fg(Color::Rgb(140, 140, 180)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  time       ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled(
                        format!("{}m {}s", elapsed / 60, elapsed % 60),
                        Style::default().fg(Color::Rgb(140, 140, 180)),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  [", Style::default().fg(Color::Rgb(40, 40, 60))),
                    Span::styled(filled, Style::default().fg(Color::Rgb(80, 200, 140))),
                    Span::styled(empty, Style::default().fg(Color::Rgb(30, 30, 50))),
                    Span::styled("]", Style::default().fg(Color::Rgb(40, 40, 60))),
                ]),
                Line::from(Span::styled(
                    format!("  rank up in {} more", 10 - (app.words_all_time % 10)),
                    Style::default().fg(Color::Rgb(50, 70, 50)).add_modifier(Modifier::ITALIC),
                )),
            ];

            f.render_widget(
                Paragraph::new(right_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(
                                if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(30, 40, 60) },
                            ))
                            .title(Span::styled(
                                " stats ",
                                Style::default().fg(Color::Rgb(60, 80, 120)),
                            )),
                    )
                    .alignment(Alignment::Center),
                main_chunks[1],
            );

            // ── Footer ───────────────────────────────────────────────────
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("  q", Style::default().fg(Color::Rgb(200, 80, 80))),
                    Span::styled(" quit  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled("tab", Style::default().fg(Color::Rgb(180, 120, 255))),
                    Span::styled(" change target  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled("r", Style::default().fg(Color::Rgb(100, 200, 255))),
                    Span::styled(" reset session  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                    Span::styled("ctrl+c", Style::default().fg(Color::Rgb(200, 80, 80))),
                    Span::styled(" exit", Style::default().fg(Color::Rgb(60, 60, 80))),
                ]))
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_type(BorderType::Plain)
                        .border_style(Style::default().fg(Color::Rgb(40, 40, 60))),
                ),
                chunks[2],
            );
        })?;

        // ── Event handling ───────────────────────────────────────────────
        if event::poll(tick)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match &mut app.screen {
                    // ── Setting screen ───────────────────────────────────
                    Screen::Setting { draft } => {
                        match code {
                            KeyCode::Enter => {
                                let d = draft.clone();
                                app.confirm_target(d);
                            }
                            KeyCode::Backspace => { draft.pop(); }
                            KeyCode::Char(c) if modifiers == KeyModifiers::NONE
                                             || modifiers == KeyModifiers::SHIFT => {
                                draft.push(c);
                            }
                            _ => {}
                        }
                    }

                    // ── Typing screen ────────────────────────────────────
                    Screen::Typing => {
                        match (code, modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                store::save(app.words_all_time, app.completions_all_time, app.total_secs());
                                break;
                            }
                            (KeyCode::Char('q'), KeyModifiers::NONE) if app.input.is_empty() => {
                                store::save(app.words_all_time, app.completions_all_time, app.total_secs());
                                break;
                            }
                            // Tab → back to setter
                            (KeyCode::Tab, _) => {
                                app.input.clear();
                                app.screen = Screen::Setting { draft: app.target.clone() };
                            }
                            // r → reset session stats
                            (KeyCode::Char('r'), KeyModifiers::NONE) if app.input.is_empty() => {
                                app.session_words = 0;
                                app.session_completions = 0;
                                app.last_wpm = 0.0;
                                app.flash = None;
                            }
                            (KeyCode::Backspace, _) => { app.input.pop(); }
                            (KeyCode::Char(c), _) => {
                                if app.input.is_empty() {
                                    app.round_start = Some(Instant::now());
                                }
                                app.input.push(c);
                                if app.input == app.target {
                                    app.on_complete();
                                } else if app.input.len() > app.target.len() {
                                    app.input.clear();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
