use std::{
    f32::consts::PI,
    io::{self, Cursor, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{self, ClearType},
};
use rodio::{Decoder, OutputStreamBuilder, Sink, source::Source};

// Donut math constants
const THETA_STEP: f32 = 0.07;
const PHI_STEP: f32 = 0.02;
const R1: f32 = 1.0;
const R2: f32 = 2.0;
const K2: f32 = 5.0;
const LUMINANCE: &[u8] = b".,-~:;=!*#$@";

struct Donut {
    width: u16,
    height: u16,
    a: f32,
    b: f32,
    explode: f32,
    explode_active: bool,
    pub should_clone: bool, // set true at end of explosion — main spawns a copy
}

impl Donut {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            a: 0.0,
            b: 0.0,
            explode: 0.0,
            explode_active: false,
            should_clone: false,
        }
    }

    fn trigger_explode(&mut self) {
        self.explode_active = true;
        self.explode = 0.0;
    }

    fn render(&mut self) -> Vec<(u16, u16, char, Color)> {
        let w = self.width as usize;
        let h = self.height as usize;
        // Terminal chars are ~2x taller than wide, so multiply rows by 2
        // to get the equivalent column-space height, then pick the smaller
        // dimension so the donut always fits without overflowing either axis.
        let scale_dim = (w as f32).min(h as f32 * 2.0) * 0.38;
        let k1 = scale_dim * K2 / (R1 + R2);

        let mut output = vec![' '; w * h];
        let mut zbuf = vec![0f32; w * h];
        let mut colors = vec![Color::White; w * h];

        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;

        let sin_a = self.a.sin();
        let cos_a = self.a.cos();
        let sin_b = self.b.sin();
        let cos_b = self.b.cos();

        let mut theta = 0f32;
        while theta < 2.0 * PI {
            let sin_t = theta.sin();
            let cos_t = theta.cos();

            let mut phi = 0f32;
            while phi < 2.0 * PI {
                let sin_p = phi.sin();
                let cos_p = phi.cos();

                let circle_x = R2 + R1 * cos_t;
                let circle_y = R1 * sin_t;

                // 3D point on torus
                let x = circle_x * (cos_b * cos_p + sin_a * sin_b * sin_p) - circle_y * cos_a * sin_b;
                let y = circle_x * (sin_b * cos_p - sin_a * cos_b * sin_p) + circle_y * cos_a * cos_b;
                let z = K2 + cos_a * circle_x * sin_p + circle_y * sin_a;
                let ooz = 1.0 / z;

                // Project normally — no explosion in 3D space
                let xp_f = cx + k1 * ooz * x;
                let yp_f = cy - k1 * ooz * 0.5 * y;

                // Apply explosion as a fixed screen-space radial offset
                let (xp_f, yp_f) = if self.explode_active {
                    let dx = xp_f - cx;
                    let dy = yp_f - cy;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                    let push = self.explode * 8.0;
                    (xp_f + dx / dist * push, yp_f + dy / dist * push)
                } else {
                    (xp_f, yp_f)
                };

                if xp_f < 0.0 || yp_f < 0.0 { phi += PHI_STEP; continue; }
                let xp = xp_f as usize;
                let yp = yp_f as usize;

                if xp < w && yp < h {
                    let idx = yp * w + xp;
                    let lum = cos_p * cos_t * sin_b - cos_a * cos_t * sin_p
                    - sin_a * sin_t + cos_b * (cos_a * sin_t - cos_t * sin_a * sin_p);
                    let lum_i = ((lum * 8.0) as i32).clamp(0, 11) as usize;

                    if ooz > zbuf[idx] {
                        zbuf[idx] = ooz;
                        output[idx] = LUMINANCE[lum_i] as char;

                        // Color based on luminance + explode state
                        colors[idx] = if self.explode_active {
                            // Rainbow explosion colors
                            let hue = (phi / (2.0 * PI) + self.explode * 0.5) % 1.0;
                            hue_to_color(hue)
                        } else {
                            // Wii-ish blue/purple palette
                            let t = lum_i as f32 / 11.0;
                            if t > 0.7 {
                                Color::White
                            } else if t > 0.4 {
                                Color::Rgb { r: 130, g: 180, b: 255 }
                            } else {
                                Color::Rgb { r: 60, g: 80, b: 180 }
                            }
                        };
                    }
                }

                phi += PHI_STEP;
            }
            theta += THETA_STEP;
        }

        // Update rotation — faster during explode
        let speed = if self.explode_active { 1.0 + self.explode * 2.0 } else { 1.0 };
        self.a += 0.04 * speed;
        self.b += 0.02 * speed;

        if self.explode_active {
            self.explode += 0.08;
            if self.explode > 5.0 {
                self.explode = 0.0;
                self.explode_active = false;
                self.should_clone = true; // signal main to spawn a clone
            }
        }

        // Collect non-space cells
        let mut cells = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let ch = output[idx];
                if ch != ' ' {
                    cells.push((x as u16, y as u16, ch, colors[idx]));
                }
            }
        }
        cells
    }
}

fn hue_to_color(h: f32) -> Color {
    let i = (h * 6.0) as u32;
    let f = h * 6.0 - i as f32;
    let (r, g, b) = match i % 6 {
        0 => (255, (f * 255.0) as u8, 0),
        1 => ((255.0 * (1.0 - f)) as u8, 255, 0),
        2 => (0, 255, (f * 255.0) as u8),
        3 => (0, (255.0 * (1.0 - f)) as u8, 255),
        4 => ((f * 255.0) as u8, 0, 255),
        _ => (255, 0, (255.0 * (1.0 - f)) as u8),
    };
    Color::Rgb { r, g, b }
}

// Embed the music at compile time — wii.mp3 must be at src/assets/wii.mp3
static MUSIC: &[u8] = include_bytes!("assets/wii.mp3");

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // _stream_handle MUST stay alive or audio stops immediately.
    // Cursor wraps the embedded bytes — implements Read+Seek so Decoder accepts it.
    let _stream_handle = OutputStreamBuilder::open_default_stream()
    .expect("failed to open audio stream");
    let mixer = _stream_handle.mixer();
    let source = Decoder::new(Cursor::new(MUSIC))
    .expect("failed to decode embedded audio");
    let sink = Sink::connect_new(&mixer);
    sink.append(source.repeat_infinite());
    sink.set_volume(0.7);

    let (cols, rows) = terminal::size()?;
    let mut donut = Donut::new(cols, rows - 2);

    let hint = "[q/ctrl+c] quit  [e] explode";

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Spawn input thread
    let (tx, rx) = std::sync::mpsc::channel::<char>();
    thread::spawn(move || {
        while r.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            let _ = tx.send('q');
                            break;
                        }
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            let _ = tx.send('q');
                            break;
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            let _ = tx.send('e');
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let frame_time = Duration::from_millis(33); // ~30fps

    'outer: loop {
        // Handle input
        while let Ok(ch) = rx.try_recv() {
            match ch {
                'q' => break 'outer,
                'e' => donut.trigger_explode(),
                _ => {}
            }
        }

        let t0 = Instant::now();

        // Clear
        execute!(stdout, terminal::Clear(ClearType::All))?;

        // Draw donut
        let cells = donut.render();

        // Spawn a clone of ourselves if the explosion just finished
        if donut.should_clone {
            donut.should_clone = false;
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn(); // detached — fire and forget
            }
        }

        for (x, y, ch, color) in cells {
            execute!(
                stdout,
                cursor::MoveTo(x, y),
                     SetForegroundColor(color),
                     Print(ch),
                     ResetColor
            )?;
        }

        // Status bar at bottom
        execute!(
            stdout,
            cursor::MoveTo(0, rows - 1),
                 SetForegroundColor(Color::Rgb { r: 100, g: 120, b: 200 }),
                 Print(hint),
                 ResetColor,
        )?;

        stdout.flush()?;

        // Frame cap
        let elapsed = t0.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
    }

    running.store(false, Ordering::Relaxed);
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
