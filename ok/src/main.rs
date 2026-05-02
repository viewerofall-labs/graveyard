use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

// ── Infinite rank title generator ────────────────────────────────────────────
//
// Tiers are every 10 ok's. The algorithm has 5 escalating phases:
//   0-4   (0-49)    grounded: normal human descriptors
//   5-9   (50-99)   elevated: mystical / cosmic
//   10-19 (100-199) unhinged: void/reality-breaking
//   20-39 (200-399) fully detached: nonsense compounds
//   40+   (400+)    post-language: pure chaos, numbers bleed in
//
// Past the curated pools the generator hashes the tier index to pick
// deterministic-but-unique combos, so every rank is stable across restarts.

const OK_PHASE0: &[(&str, &str)] = &[
    ("OK", "Enjoyer"),   ("OK", "Appreciator"), ("OK", "Enthusiast"),
    ("OK", "Aficionado"),("OK", "Pilgrim"),      ("OK", "Devotee"),
    ("OK", "Connoisseur"),("OK", "Practitioner"),("OK", "Adherent"),
    ("OK", "Acolyte"),
];
const OK_PHASE1: &[(&str, &str)] = &[
    ("OK", "Sage"),         ("OK", "Grandmaster"),   ("OK", "Archon"),
    ("OK", "Deity"),        ("OK", "Transcended"),   ("OK", "Ascendant"),
    ("OK", "Cosmic"),       ("OK", "Singularity"),   ("OK", "Void Walker"),
    ("OK", "Dimension Lord"),
];
const OK_PHASE2: &[(&str, &str)] = &[
    ("OK", "Reality Glitch"),   ("OK", "Time Eater"),     ("OK", "Entropy God"),
    ("OK", "Universe Ender"),   ("OK", "Big Banger"),     ("OK", "Multiverse Sovereign"),
    ("OK", "Fabric Weaver"),    ("OK", "Causality Breaker"),("OK", "Infinite Regress"),
    ("OK", "Heat Death"),       ("OK", "Null Point"),      ("OK", "Axiom Shatterer"),
    ("OK", "False Vacuum"),     ("OK", "Planck Sovereign"),("OK", "Dark Flow Rider"),
    ("OK", "Boltzmann Brain"),  ("OK", "Quantum Foam"),    ("OK", "Naked Singularity"),
    ("OK", "Event Horizon"),    ("OK", "Brane Collider"),
];

const OKAY_PHASE0: &[(&str, &str)] = &[
    ("okay...", "person"),  ("okay", "appreciator"), ("okay", "enthusiast"),
    ("okay", "devotee"),    ("okay", "oracle"),       ("okay", "archon"),
    ("okay", "i guess"),    ("okay", "if you say so"),("okay", "sure"),
    ("okay", "fine"),
];
const OKAY_PHASE1: &[(&str, &str)] = &[
    ("okay", "beyond"),         ("okay", "without end"),   ("okay", "shapeless"),
    ("okay", "heat death"),     ("okay", "there buddy"),   ("okay", "void"),
    ("okay", "unraveling"),     ("okay", "adrift"),        ("okay", "untethered"),
    ("okay", "past caring"),
];
const OKAY_PHASE2: &[(&str, &str)] = &[
    ("okay", "reality leak"),   ("okay", "time smear"),    ("okay", "concept error"),
    ("okay", "null reference"), ("okay", "stack overflow"), ("okay", "undefined behavior"),
    ("okay", "cosmic shrug"),   ("okay", "entropy enjoyer"),("okay", "heat death"),
    ("okay", "vacuum decay"),   ("okay", "false floor"),    ("okay", "brane wanderer"),
    ("okay", "eigenstate"),     ("okay", "superposed"),     ("okay", "decoherent"),
    ("okay", "many-worlded"),   ("okay", "retrocausal"),    ("okay", "acausal"),
    ("okay", "pre-geometric"),  ("okay", "post-semantic"),
];

// Chaos pools for phase 3+ — mixed freely by hash
const CHAOS_PREFIX: &[&str] = &[
    "Trans-OK", "Hyper-OK", "Meta-OK", "Ultra-OK", "Post-OK",
"Void-OK", "Null-OK", "Anti-OK", "Para-OK", "Quasi-OK",
"Neo-OK", "Omni-OK", "Pan-OK", "Xeno-OK", "Retro-OK",
"Proto-OK", "Pseudo-OK", "Infra-OK", "Supra-OK", "Macro-OK",
];
const CHAOS_SUFFIX: &[&str] = &[
    "Sovereign",    "Obliterator", "Paradox",      "Fugue",       "Specter",
"Remnant",      "Substrate",   "Eigenstate",   "Manifold",    "Topology",
"Residue",      "Phantom",     "Gradient",     "Threshold",   "Lattice",
"Curvature",    "Asymptote",   "Divergence",   "Singularity", "Hologram",
"Perturbation", "Recursion",   "Tautology",    "Axiom",       "Lemma",
"Corollary",    "Postulate",   "Conjecture",   "Theorem",     "Proof",
"Negation",     "Complement",  "Isomorphism",  "Functor",     "Monad",
"Sheaf",        "Topos",       "Groupoid",     "Cobordism",   "Fibration",
];
const DERANGED_PREFIX: &[&str] = &[
    "Screaming-OK", "Wet-OK", "Confused-OK", "Inverted-OK", "Recursive-OK",
"Haunted-OK",   "Cursed-OK", "Blessed-OK", "Forgotten-OK", "Remembered-OK",
"Imaginary-OK", "Borrowed-OK", "Expired-OK", "Pending-OK",  "Deprecated-OK",
"Segfaulting-OK","Panicking-OK","Unwrapping-OK","Borrowing-OK","Cloning-OK",
];
const DERANGED_SUFFIX: &[&str] = &[
    "of the Abyss",      "of Nothing",        "of Everything",
"of the Drain",      "of Pure Vibes",     "of Incomprehensible Scale",
"Beyond the OK",     "Past the Point",    "Without Context",
"With No Explanation","Untitled",          "Redacted",
"404 Not Found",     "NaN",               "undefined",
"of Type Any",       "impl Display",      "dyn Error",
"Send + Sync",       "where T: OK",
];

/// Cheap deterministic hash — maps a u64 to a usize index into a slice.
fn pick(tier: u64, salt: u64, len: usize) -> usize {
    // xorshift-style mix
    let mut h = tier.wrapping_add(salt).wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;
    (h as usize) % len
}

fn generate_title(tier: u64, mode: Mode) -> String {
    match mode {
        Mode::Ok => match tier {
            0..=9  => { let (p,s) = OK_PHASE0[tier as usize % OK_PHASE0.len()]; format!("{} {}", p, s) }
            10..=19 => { let (p,s) = OK_PHASE1[(tier-10) as usize % OK_PHASE1.len()]; format!("{} {}", p, s) }
            20..=39 => { let (p,s) = OK_PHASE2[(tier-20) as usize % OK_PHASE2.len()]; format!("{} {}", p, s) }
            40..=79 => {
                let p = CHAOS_PREFIX[pick(tier, 0xdeadbeef, CHAOS_PREFIX.len())];
                let s = CHAOS_SUFFIX[pick(tier, 0xcafebabe, CHAOS_SUFFIX.len())];
                format!("{} {}", p, s)
            }
            _ => {
                // Full derangement — mix all four pools
                let use_deranged_p = pick(tier, 0x1337, 2) == 0;
                let use_deranged_s = pick(tier, 0xf00d, 2) == 0;
                let p = if use_deranged_p {
                    DERANGED_PREFIX[pick(tier, 0xaaaa, DERANGED_PREFIX.len())]
                } else {
                    CHAOS_PREFIX[pick(tier, 0xbbbb, CHAOS_PREFIX.len())]
                };
                let s = if use_deranged_s {
                    DERANGED_SUFFIX[pick(tier, 0xcccc, DERANGED_SUFFIX.len())]
                } else {
                    CHAOS_SUFFIX[pick(tier, 0xdddd, CHAOS_SUFFIX.len())]
                };
                // Every 13 tiers past 80, inject the tier number for extra chaos
                if tier % 13 == 0 {
                    format!("{} {} (×{})", p, s, tier)
                } else {
                    format!("{} {}", p, s)
                }
            }
        },
        Mode::Okay => match tier {
            0..=9  => { let (p,s) = OKAY_PHASE0[tier as usize % OKAY_PHASE0.len()]; format!("{} {}", p, s) }
            10..=19 => { let (p,s) = OKAY_PHASE1[(tier-10) as usize % OKAY_PHASE1.len()]; format!("{} {}", p, s) }
            20..=39 => { let (p,s) = OKAY_PHASE2[(tier-20) as usize % OKAY_PHASE2.len()]; format!("{} {}", p, s) }
            40..=79 => {
                let p = pick(tier, 0x0b00, 2);
                let s = CHAOS_SUFFIX[pick(tier, 0xb00b, CHAOS_SUFFIX.len())];
                if p == 0 { format!("okay {}", s.to_lowercase()) }
                else      { format!("okay, {}", s.to_lowercase()) }
            }
            _ => {
                let s = DERANGED_SUFFIX[pick(tier, 0xeeee, DERANGED_SUFFIX.len())];
                if tier % 7 == 0 {
                    format!("okay {} (tier {})", s.to_lowercase(), tier)
                } else {
                    format!("okay {}", s.to_lowercase())
                }
            }
        },
    }
}

fn get_title(count: u64, mode: Mode) -> String {
    generate_title(count / 10, mode)
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Ok,
    Okay,
}

fn ok_art(count: u64) -> Vec<&'static str> {
    if count == 0 {
        return vec![
            "  ██████╗ ██╗  ██╗  ",
            " ██╔═══██╗██║ ██╔╝  ",
            " ██║   ██║█████╔╝   ",
            " ██║   ██║██╔═██╗   ",
            " ╚██████╔╝██║  ██╗  ",
            "  ╚═════╝ ╚═╝  ╚═╝  ",
        ];
    }
    match count % 4 {
        0 => vec![
            " ██████╗ ██╗  ██╗ ",
            "██╔═══██╗██║ ██╔╝ ",
            "██║   ██║█████╔╝  ",
            "██║   ██║██╔═██╗  ",
            "╚██████╔╝██║  ██╗ ",
            " ╚═════╝ ╚═╝  ╚═╝ ",
        ],
        1 => vec![
            " ██████╗ ██╗  ██╗ ",
            "██╔═══██╗██║ ██╔╝ ",
            "██║   ██║█████╔╝  ",
            "██║   ██║██╔═██╗  ",
            "╚██████╔╝██║  ██╗ ",
            " ╚═════╝ ╚═╝  ╚═╝ ",
        ],
        2 => vec![
            " ██████╗ ██╗  ██╗ ",
            "██╔═══██╗██║ ██╔╝ ",
            "██║   ██║█████╔╝  ",
            "██║   ██║██╔═██╗  ",
            "╚██████╔╝██║  ██╗ ",
            " ╚═════╝ ╚═╝  ╚═╝ ",
        ],
        _ => vec![
            " ██████╗ ██╗  ██╗ ",
            "██╔═══██╗██║ ██╔╝ ",
            "██║   ██║█████╔╝  ",
            "██║   ██║██╔═██╗  ",
            "╚██████╔╝██║  ██╗ ",
            " ╚═════╝ ╚═╝  ╚═╝ ",
        ],
    }
}

struct App {
    count: u64,
    all_time: u64,
    okay_count: u64,        // session okay count
    okay_all_time: u64,     // persistent okay count
    saved_secs: u64,
    mode: Mode,
    input: String,
    flash: Option<Instant>,
    messages: Vec<String>,
    okay_messages: Vec<String>,
    session_start: Instant,
}

impl App {
    fn new() -> Self {
        let (all_time, okay_all_time, saved_secs) = store::load();
        Self {
            count: 0,
            all_time,
            okay_count: 0,
            okay_all_time,
            saved_secs,
            mode: Mode::Ok,
            input: String::new(),
            flash: None,
            messages: vec![
                "ok.".into(),
                "noted.".into(),
                "acknowledged.".into(),
                "understood.".into(),
                "received.".into(),
                "cool.".into(),
                "word.".into(),
                "sure.".into(),
                "right.".into(),
                "mhm.".into(),
            ],
            okay_messages: vec![
                "okay...".into(),
                "if you say so.".into(),
                "i guess.".into(),
                "sure, okay.".into(),
                "fine.".into(),
                "alright then.".into(),
                "as you wish.".into(),
                "whatever you say.".into(),
                "noted, okay.".into(),
                "okay okay.".into(),
            ],
            session_start: Instant::now(),
        }
    }

    fn on_ok(&mut self) {
        self.count += 1;
        self.all_time += 1;
        self.flash = Some(Instant::now());
        self.input.clear();
        store::save(self.all_time, self.okay_all_time, self.total_secs());
    }

    fn on_okay(&mut self) {
        self.okay_count += 1;
        self.okay_all_time += 1;
        self.flash = Some(Instant::now());
        self.input.clear();
        store::save(self.all_time, self.okay_all_time, self.total_secs());
    }

    fn total_secs(&self) -> u64 {
        self.saved_secs + self.session_start.elapsed().as_secs()
    }

    fn current_message(&self) -> &str {
        match self.mode {
            Mode::Ok => {
                if self.count == 0 { return "type ok. no enter needed."; }
                &self.messages[(self.count as usize - 1) % self.messages.len()]
            }
            Mode::Okay => {
                if self.okay_count == 0 { return "type okay. no enter needed."; }
                &self.okay_messages[(self.okay_count as usize - 1) % self.okay_messages.len()]
            }
        }
    }

    fn is_flashing(&self) -> bool {
        self.flash
        .map(|t| t.elapsed() < Duration::from_millis(120))
        .unwrap_or(false)
    }

    fn elapsed_secs(&self) -> u64 {
        self.session_start.elapsed().as_secs()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage((100 - percent_y) / 2),
                 Constraint::Percentage(percent_y),
                 Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);
    Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage((100 - percent_x) / 2),
                 Constraint::Percentage(percent_x),
                 Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn main() -> io::Result<()> {
    // ── CLI: okcounter -r  →  wipe save with y/n prompt ──────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("-r") {
        let (ok, okay, secs) = store::load();
        if ok == 0 && okay == 0 {
            println!("No save data found. Nothing to reset.");
            return Ok(());
        }
        println!("Current save:");
        println!("  ok all-time   : {}", ok);
        println!("  okay all-time : {}", okay);
        println!("  time logged   : {}m {}s", secs / 60, secs % 60);
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
            println!("Aborted. Your ok's are safe.");
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
            let count = app.count;

            // Background
            let bg = Block::default().style(Style::default().bg(Color::Rgb(10, 10, 14)));
            f.render_widget(bg, size);

            // Outer layout
            let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),  // header
                         Constraint::Min(10),    // main
                         Constraint::Length(3),  // footer
            ])
            .split(size);

            // ── Header ──
            let title_color = if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(180, 180, 200) };
            let (mode_label, mode_color, active_all_time) = match app.mode {
                Mode::Ok   => ("ok mode",   Color::Rgb(100, 200, 255), app.all_time),
                      Mode::Okay => ("okay mode", Color::Rgb(180, 120, 255), app.okay_all_time),
            };
            let header = Paragraph::new(Line::from(vec![
                Span::styled("  ◆ ", Style::default().fg(mode_color)),
                                                   Span::styled(mode_label, Style::default().fg(title_color).add_modifier(Modifier::BOLD)),
                                                   Span::styled("  ◆  ", Style::default().fg(mode_color)),
                                                   Span::styled(
                                                       get_title(active_all_time, app.mode),
                                                                Style::default().fg(Color::Rgb(80, 160, 80)).add_modifier(Modifier::ITALIC),
                                                   ),
                                                   Span::styled("   tab: switch mode", Style::default().fg(Color::Rgb(50, 50, 70))),
            ]))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Rgb(40, 40, 60))),
            );
            f.render_widget(header, chunks[0]);

            // ── Main area: art left, stats right ──
            let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

            // Left: giant OK art + input display
            let art_accent = if flash {
                Color::Rgb(255, 220, 60)
            } else {
                match app.mode {
                    Mode::Ok   => Color::Rgb(100, 200, 255),
                      Mode::Okay => Color::Rgb(180, 120, 255),
                }
            };

            let art_lines: Vec<Line> = ok_art(count)
            .iter()
            .map(|l| Line::from(Span::styled(*l, Style::default().fg(art_accent).add_modifier(Modifier::BOLD))))
            .collect();

            let mut left_lines = art_lines;

            // Spacer
            left_lines.push(Line::from(""));

            // Input display
            let input_display = if app.input.is_empty() {
                Line::from(vec![
                    Span::styled("  › ", Style::default().fg(Color::Rgb(60, 60, 80))),
                           Span::styled("_", Style::default().fg(Color::Rgb(60, 80, 100)).add_modifier(Modifier::SLOW_BLINK)),
                ])
            } else {
                let col = if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(200, 220, 255) };
                Line::from(vec![
                    Span::styled("  › ", Style::default().fg(Color::Rgb(100, 200, 255))),
                           Span::styled(app.input.clone(), Style::default().fg(col).add_modifier(Modifier::BOLD)),
                ])
            };
            left_lines.push(input_display);

            // Message
            left_lines.push(Line::from(""));
            left_lines.push(Line::from(Span::styled(
                format!("  {}", app.current_message()),
                    Style::default()
                    .fg(Color::Rgb(80, 100, 80))
                    .add_modifier(Modifier::ITALIC),
            )));

            let left = Paragraph::new(left_lines)
            .block(
                Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if flash {
                    Color::Rgb(255, 220, 60)
                } else {
                    Color::Rgb(30, 40, 60)
                }))
                .title(Span::styled(
                    " input ",
                    Style::default().fg(Color::Rgb(60, 80, 120)),
                )),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

            f.render_widget(left, main_chunks[0]);

            // Right: counter stats
            let elapsed = app.elapsed_secs();
            let (session_count, active_all_time) = match app.mode {
                Mode::Ok   => (app.count, app.all_time),
                      Mode::Okay => (app.okay_count, app.okay_all_time),
            };
            let rate = if elapsed > 0 { session_count as f64 / elapsed as f64 } else { 0.0 };
            let count_color = if flash { Color::Rgb(255, 220, 60) } else { Color::Rgb(120, 220, 180) };
            let okay_color  = Color::Rgb(180, 120, 255);

            let right_lines = vec![
                Line::from(""),
                      // Big number = all-time (drives rank)
                      Line::from(Span::styled(
                          active_all_time.to_string(),
                                              Style::default().fg(count_color).add_modifier(Modifier::BOLD),
                      )),
                      Line::from(Span::styled(
                          "all time",
                          Style::default().fg(Color::Rgb(60, 80, 80)).add_modifier(Modifier::ITALIC),
                      )),
                      Line::from(""),
                      Line::from(vec![
                          Span::styled("  ok      ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                 Span::styled(
                                     app.all_time.to_string(),
                                              Style::default().fg(Color::Rgb(200, 180, 100)).add_modifier(Modifier::BOLD),
                                 ),
                      ]),
                      Line::from(vec![
                          Span::styled("  okay    ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                 Span::styled(
                                     app.okay_all_time.to_string(),
                                              Style::default().fg(okay_color).add_modifier(Modifier::BOLD),
                                 ),
                      ]),
                      Line::from(vec![
                          Span::styled("  session ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                 Span::styled(
                                     session_count.to_string(),
                                              Style::default().fg(Color::Rgb(140, 180, 140)),
                                 ),
                      ]),
                      Line::from(vec![
                          Span::styled("  time    ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                 Span::styled(
                                     format!("{}m {}s", elapsed / 60, elapsed % 60),
                                         Style::default().fg(Color::Rgb(140, 140, 180)),
                                 ),
                      ]),
                      Line::from(vec![
                          Span::styled("  rate    ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                 Span::styled(
                                     format!("{:.2}/min", rate * 60.0),
                                         Style::default().fg(Color::Rgb(140, 140, 180)),
                                 ),
                      ]),
                      Line::from(""),
                      {
                          let progress = (active_all_time % 10) as usize;
                          let filled: String = "█".repeat(progress);
                          let empty: String = "░".repeat(10 - progress);
                          Line::from(vec![
                              Span::styled("  [", Style::default().fg(Color::Rgb(40, 40, 60))),
                                     Span::styled(filled, Style::default().fg(Color::Rgb(80, 200, 140))),
                                     Span::styled(empty, Style::default().fg(Color::Rgb(30, 30, 50))),
                                     Span::styled("]", Style::default().fg(Color::Rgb(40, 40, 60))),
                          ])
                      },
                      Line::from(Span::styled(
                          format!("  rank up in {} more", 10 - (active_all_time % 10)),
                              Style::default().fg(Color::Rgb(50, 70, 50)).add_modifier(Modifier::ITALIC),
                      )),
            ];

            let right = Paragraph::new(right_lines)
            .block(
                Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if flash {
                    Color::Rgb(255, 220, 60)
                } else {
                    Color::Rgb(30, 40, 60)
                }))
                .title(Span::styled(
                    " count ",
                    Style::default().fg(Color::Rgb(60, 80, 120)),
                )),
            )
            .alignment(Alignment::Center);

            f.render_widget(right, main_chunks[1]);

            // ── Footer ──
            let footer = Paragraph::new(Line::from(vec![
                Span::styled("  q", Style::default().fg(Color::Rgb(200, 80, 80))),
                                                   Span::styled(" quit  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                                   Span::styled("r", Style::default().fg(Color::Rgb(100, 200, 255))),
                                                   Span::styled(" reset  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                                                   Span::styled("ctrl+c", Style::default().fg(Color::Rgb(200, 80, 80))),
                                                   Span::styled(" exit", Style::default().fg(Color::Rgb(60, 60, 80))),
            ]))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Rgb(40, 40, 60))),
            );
            f.render_widget(footer, chunks[2]);
        })?;

        // Event handling
        if event::poll(tick)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match (code, modifiers) {
                    (KeyCode::Tab, _) if app.input.is_empty() => {
                        app.mode = match app.mode {
                            Mode::Ok   => Mode::Okay,
                            Mode::Okay => Mode::Ok,
                        };
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        store::save(app.all_time, app.okay_all_time, app.total_secs());
                        break;
                    }
                    (KeyCode::Char('q'), KeyModifiers::NONE) if app.input.is_empty() => {
                        store::save(app.all_time, app.okay_all_time, app.total_secs());
                        break;
                    }
                    (KeyCode::Char('r'), KeyModifiers::NONE) if app.input.is_empty() => {
                        app.count = 0;
                        app.okay_count = 0;
                        app.input.clear();
                        app.flash = None;
                    }
                    (KeyCode::Char(c), _) => {
                        app.input.push(c);
                        let lower = app.input.to_lowercase();
                        match app.mode {
                            Mode::Ok => {
                                if lower == "ok" {
                                    app.on_ok();
                                } else if lower.len() >= 2 && !"ok".starts_with(&lower as &str) {
                                    app.input.clear();
                                }
                            }
                            Mode::Okay => {
                                if lower == "okay" {
                                    app.on_okay();
                                } else if lower.len() >= 4 && !"okay".starts_with(&lower as &str) {
                                    app.input.clear();
                                } else if lower.len() < 4 && !"okay".starts_with(&lower as &str) {
                                    app.input.clear();
                                }
                            }
                        }
                    }
                    (KeyCode::Backspace, _) => {
                        app.input.pop();
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
             LeaveAlternateScreen,
             DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
