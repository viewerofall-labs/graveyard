mod store;

use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Vec2};
use rand::Rng;
use std::time::{Duration, Instant};

// ── Forced delay between valid keypresses (anti-hold) ────────────────────────
const KEY_COOLDOWN: Duration = Duration::from_millis(120);

struct FloatingOk {
    pos: Pos2,
    born: Instant,
    lifetime: Duration,
}

impl FloatingOk {
    fn new(pos: Pos2) -> Self {
        Self { pos, born: Instant::now(), lifetime: Duration::from_millis(900) }
    }
    fn progress(&self) -> f32 {
        self.born.elapsed().as_secs_f32() / self.lifetime.as_secs_f32()
    }
    fn is_dead(&self) -> bool {
        self.born.elapsed() >= self.lifetime
    }
}

#[derive(PartialEq)]
enum SpaceMode { Ok, Okay }

struct OkClickerApp {
    ok_total: u64,
    okay_total: u64,
    session_start: Instant,
    saved_secs: u64,
    last_save: Instant,
    space_mode: SpaceMode,
    // letter index within current word (0..seq_len)
    letter_index: usize,
    // in-progress word buffer shown while typing
    current_word: String,
    // committed words shown scrolling
    typed_display: String,
    floating: Vec<FloatingOk>,
    // cooldown tracker per key
    last_space: Instant,
    last_tab: Instant,
    last_r: Instant,
}

impl OkClickerApp {
    fn new() -> Self {
        let (ok_total, okay_total, saved_secs) = store::load();
        let epoch = Instant::now() - Duration::from_secs(999);
        Self {
            ok_total, okay_total, saved_secs,
            session_start: Instant::now(),
            last_save: Instant::now(),
            space_mode: SpaceMode::Ok,
            letter_index: 0,
            current_word: String::new(),
            typed_display: String::new(),
            floating: Vec::new(),
            last_space: epoch,
            last_tab: epoch,
            last_r: epoch,
        }
    }

    fn seq(&self) -> &'static [char] {
        match self.space_mode {
            SpaceMode::Ok   => &['o', 'k'],
            SpaceMode::Okay => &['o', 'k', 'a', 'y'],
        }
    }

    fn current_char(&self) -> char {
        self.seq()[self.letter_index % self.seq().len()]
    }

    /// Returns true when a full word was just completed (score tick)
    fn advance_space(&mut self) -> bool {
        let seq = self.seq();
        let ch = seq[self.letter_index % seq.len()];
        self.current_word.push(ch);
        self.letter_index += 1;

        if self.letter_index >= seq.len() {
            // Word complete — commit to display, award point
            self.typed_display.push_str(&self.current_word);
            self.typed_display.push(' ');
            if self.typed_display.len() > 50 {
                let trim = self.typed_display.len() - 50;
                self.typed_display = self.typed_display[trim..].to_string();
            }
            self.current_word.clear();
            self.letter_index = 0;

            match self.space_mode {
                SpaceMode::Ok   => self.ok_total += 1,
                SpaceMode::Okay => self.okay_total += 1,
            }
            return true;
        }
        false
    }

    fn toggle_mode(&mut self) {
        self.space_mode = match self.space_mode {
            SpaceMode::Ok   => SpaceMode::Okay,
            SpaceMode::Okay => SpaceMode::Ok,
        };
        self.letter_index = 0;
        self.current_word.clear();
    }

    fn reset(&mut self) {
        store::wipe();
        self.ok_total = 0;
        self.okay_total = 0;
        self.saved_secs = 0;
        self.session_start = Instant::now();
        self.letter_index = 0;
        self.current_word.clear();
        self.typed_display.clear();
        self.floating.clear();
    }

    fn spawn_floating(&mut self, center: Pos2) {
        let mut rng = rand::rng();
        let offset = Vec2::new(rng.random_range(-30.0..30.0), rng.random_range(-30.0..30.0));
        self.floating.push(FloatingOk::new(center + offset));
        self.ok_total += 1;
    }

    fn total_secs(&self) -> u64 {
        self.saved_secs + self.session_start.elapsed().as_secs()
    }

    fn maybe_save(&mut self) {
        if self.last_save.elapsed() >= Duration::from_secs(10) {
            store::save(self.ok_total, self.okay_total, self.total_secs());
            self.last_save = Instant::now();
        }
    }

    fn score_display(&self) -> u64 {
        self.ok_total + self.okay_total
    }
}

impl eframe::App for OkClickerApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        store::save(self.ok_total, self.okay_total, self.total_secs());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));

        // ── Key input with cooldown (no hold stacking) ───────────────────
        let now = Instant::now();
        ctx.input(|i| {
            if i.key_pressed(Key::Space) && now.duration_since(self.last_space) >= KEY_COOLDOWN {
                self.last_space = now;
                self.advance_space();
            }
            if i.key_pressed(Key::Tab) && now.duration_since(self.last_tab) >= KEY_COOLDOWN {
                self.last_tab = now;
                self.toggle_mode();
            }
            if i.key_pressed(Key::R) && now.duration_since(self.last_r) >= KEY_COOLDOWN {
                self.last_r = now;
                self.reset();
            }
        });

        self.maybe_save();
        self.floating.retain(|f| !f.is_dead());

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(10, 10, 18)))
            .show(ctx, |ui| {
                let panel_rect = ui.max_rect();

                // ── Allocate all interactive rects before painter borrow ──

                // OK button (centre)
                let btn_size = egui::Vec2::splat(160.0);
                let btn_rect = egui::Rect::from_center_size(
                    panel_rect.center() + Vec2::new(0.0, 10.0),
                    btn_size,
                );
                let btn_resp = ui.allocate_rect(btn_rect, egui::Sense::click());
                let btn_hovered = btn_resp.hovered();
                if btn_resp.clicked() {
                    self.spawn_floating(btn_rect.center());
                }

                // Reset button (bottom-right)
                let rst_size = egui::Vec2::new(90.0, 32.0);
                let rst_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        panel_rect.right() - rst_size.x - 14.0,
                        panel_rect.bottom() - rst_size.y - 14.0,
                    ),
                    rst_size,
                );
                let rst_resp = ui.allocate_rect(rst_rect, egui::Sense::click());
                let rst_hovered = rst_resp.hovered();
                if rst_resp.clicked() {
                    self.reset();
                }

                // ── Painter ───────────────────────────────────────────────
                let painter = ui.painter();

                // Score
                painter.text(
                    panel_rect.center_top() + Vec2::new(0.0, 24.0),
                    Align2::CENTER_TOP,
                    format!("{} ok", self.score_display()),
                    FontId::proportional(52.0),
                    Color32::from_rgb(180, 140, 255),
                );

                // Breakdown
                painter.text(
                    panel_rect.center_top() + Vec2::new(0.0, 86.0),
                    Align2::CENTER_TOP,
                    format!("ok: {}  •  okay: {}", self.ok_total, self.okay_total),
                    FontId::proportional(13.0),
                    Color32::from_rgb(100, 80, 160),
                );

                // Mode badge
                let mode_label = match self.space_mode {
                    SpaceMode::Ok   => "mode: ok  [TAB → okay]",
                    SpaceMode::Okay => "mode: okay  [TAB → ok]",
                };
                painter.text(
                    panel_rect.center_top() + Vec2::new(0.0, 106.0),
                    Align2::CENTER_TOP,
                    mode_label,
                    FontId::proportional(13.0),
                    Color32::from_rgb(80, 60, 130),
                );

                // OK button
                painter.rect_filled(
                    btn_rect, 20.0,
                    if btn_hovered { Color32::from_rgb(120, 80, 220) }
                    else           { Color32::from_rgb(70, 40, 140)  },
                );
                painter.text(
                    btn_rect.center(),
                    Align2::CENTER_CENTER,
                    "ok",
                    FontId::proportional(60.0),
                    Color32::WHITE,
                );

                // Reset button
                painter.rect_filled(
                    rst_rect, 8.0,
                    if rst_hovered { Color32::from_rgb(160, 40, 60) }
                    else           { Color32::from_rgb(90, 20, 35)  },
                );
                painter.text(
                    rst_rect.center(),
                    Align2::CENTER_CENTER,
                    "reset [R]",
                    FontId::proportional(13.0),
                    Color32::from_rgb(255, 160, 170),
                );

                // In-progress word (dim, shows partial typing)
                if !self.current_word.is_empty() {
                    let full: String = self.seq().iter().collect();
                    let partial = &self.current_word;
                    let remaining = &full[partial.len()..];
                    let preview = format!("{}{}", partial, remaining);
                    // Draw full word dimly, then overlay typed portion brightly
                    painter.text(
                        panel_rect.center_bottom() - Vec2::new(0.0, 78.0),
                        Align2::CENTER_BOTTOM,
                        &preview,
                        FontId::monospace(20.0),
                        Color32::from_rgb(60, 50, 100),
                    );
                    painter.text(
                        panel_rect.center_bottom() - Vec2::new(0.0, 78.0),
                        Align2::CENTER_BOTTOM,
                        partial,
                        FontId::monospace(20.0),
                        Color32::from_rgb(200, 170, 255),
                    );
                }

                // Committed typed display
                if !self.typed_display.is_empty() {
                    painter.text(
                        panel_rect.center_bottom() - Vec2::new(0.0, 50.0),
                        Align2::CENTER_BOTTOM,
                        &self.typed_display,
                        FontId::monospace(14.0),
                        Color32::from_rgb(100, 140, 200),
                    );
                }

                // Next key hint
                painter.text(
                    panel_rect.center_bottom() - Vec2::new(0.0, 24.0),
                    Align2::CENTER_BOTTOM,
                    format!("next: [{}]  (SPACE)", self.current_char()),
                    FontId::monospace(12.0),
                    Color32::from_rgb(70, 70, 110),
                );

                // Floating oks
                for f in &self.floating {
                    let t = f.progress();
                    let alpha = ((1.0 - t) * 255.0) as u8;
                    painter.text(
                        f.pos - Vec2::new(0.0, t * 70.0),
                        Align2::CENTER_CENTER,
                        "ok",
                        FontId::proportional(18.0 + t * 12.0),
                        Color32::from_rgba_premultiplied(200, 160, 255, alpha),
                    );
                }
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ok clicker")
            .with_inner_size([520.0, 460.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "ok clicker",
        options,
        Box::new(|_cc| Ok(Box::new(OkClickerApp::new()))),
    )
}
