use cairo::Context;
use rand::Rng;

pub struct CorruptionRenderer {
    rng: rand::rngs::ThreadRng,
}

impl CorruptionRenderer {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    pub fn render(&mut self, cr: &Context, width: i32, height: i32, corruption_percent: u64) {
        // Clear frame
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cr.set_operator(cairo::Operator::Source);
        cr.paint();
        cr.set_operator(cairo::Operator::Over);

        // 50% chance to skip frame (stuttering effect)
        if self.rng.gen::<f64>() > 0.5 {
            return;
        }

        let stage = (corruption_percent / 20).min(5) as u8;

        match stage {
            0 => {} // Nothing
            1 => self.render_green_dots(cr, width, height),
            2 => {
                self.render_green_dots(cr, width, height);
                self.render_tearing(cr, width, height);
            }
            3 => {
                self.render_green_dots(cr, width, height);
                self.render_tearing(cr, width, height);
                self.render_tearing(cr, width, height);
            }
            4 => {
                self.render_green_dots(cr, width, height);
                self.render_tearing(cr, width, height);
                self.render_tearing(cr, width, height);
                self.render_minor_corruption(cr, width, height);
            }
            5 => {
                // Full corruption
                self.render_green_dots(cr, width, height);
                self.render_tearing(cr, width, height);
                self.render_tearing(cr, width, height);
                self.render_minor_corruption(cr, width, height);
                self.render_screen_error(cr, width, height);
            }
            _ => {}
        }
    }

    fn render_green_dots(&mut self, cr: &Context, width: i32, height: i32) {
        cr.set_source_rgba(0.0, 1.0, 0.0, 0.9); // Lime green
        let spacing = 16;
        let start_y = if self.rng.gen::<bool>() {
            0
        } else {
            height / 4
        };
        let end_y = (start_y + height / 2).min(height);

        for x in (0..width).step_by(spacing as usize) {
            for y in (start_y..end_y).step_by(spacing as usize) {
                if self.rng.gen::<f64>() > 0.7 {
                    cr.rectangle(x as f64, y as f64, 4.0, 4.0);
                }
            }
        }
        cr.fill().ok();
    }

    fn render_tearing(&mut self, cr: &Context, width: i32, height: i32) {
        let band_y = self.rng.gen_range(0..(height - 200).max(1));
        let band_h = self.rng.gen_range(50..200);

        for _ in 0..self.rng.gen_range(40..100) {
            let bx = self.rng.gen_range(0..width);
            let by = band_y + self.rng.gen_range(0..band_h);
            let bw = self.rng.gen_range(20..150);
            let bh = self.rng.gen_range(2..12);

            let color_choice = self.rng.gen::<f64>();
            if color_choice > 0.6 {
                cr.set_source_rgba(0.0, 1.0, 0.0, 0.8); // Lime green
            } else if color_choice > 0.2 {
                cr.set_source_rgba(1.0, 0.0, 1.0, 0.8); // Magenta
            } else {
                cr.set_source_rgba(0.8, 0.8, 0.8, 0.9); // White/Grey
            }

            cr.rectangle(bx as f64, by as f64, bw as f64, bh as f64);
            cr.fill().ok();
        }
    }

    fn render_minor_corruption(&mut self, cr: &Context, width: i32, height: i32) {
        for _ in 0..15 {
            cr.set_source_rgba(
                self.rng.gen::<f64>(),
                self.rng.gen::<f64>(),
                self.rng.gen::<f64>(),
                0.9,
            );
            let x = self.rng.gen_range(0..width);
            let y = self.rng.gen_range(0..height);
            let w = self.rng.gen_range(5..20);
            let h = self.rng.gen_range(5..20);
            cr.rectangle(x as f64, y as f64, w as f64, h as f64);
            cr.fill().ok();
        }
    }

    fn render_screen_error(&mut self, cr: &Context, width: i32, height: i32) {
        // Render text overlay for full corruption
        cr.set_source_rgba(0.2, 0.2, 0.2, 0.85);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().ok();

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.select_font_face("monospace", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(48.0);

        let messages = vec!["Display not found", "GPU not found"];
        let msg = messages[self.rng.gen_range(0..messages.len())];

        // Simple centered text - just use rough positioning
        let x = (width as f64 - msg.len() as f64 * 20.0) / 2.0;
        let y = (height as f64) / 2.0;
        cr.move_to(x, y);
        cr.show_text(msg).ok();
    }
}
