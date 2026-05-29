use std::time::{Duration, Instant};

pub struct ProgressTracker {
    start_time: Option<Instant>,
    total_duration: Duration,
    paused: bool,
    paused_elapsed: Duration,
    manual_override: Option<u64>,
}

impl ProgressTracker {
    pub fn new(total_duration: Duration) -> Self {
        Self {
            start_time: None,
            total_duration,
            paused: true,
            paused_elapsed: Duration::ZERO,
            manual_override: None,
        }
    }

    pub fn set_manual_level(&mut self, level: u64) {
        self.manual_override = Some(level);
    }

    pub fn clear_manual_level(&mut self) {
        self.manual_override = None;
    }

    pub fn update(&mut self) {
        // Nothing to do, progress is tracked on-demand
    }

    pub fn get_progress_percent(&self) -> u64 {
        // If manually overridden, use that
        if let Some(level) = self.manual_override {
            return level;
        }

        let elapsed = if let Some(start) = self.start_time {
            if self.paused {
                self.paused_elapsed
            } else {
                start.elapsed()
            }
        } else {
            Duration::ZERO
        };

        let percent = (elapsed.as_secs_f64() / self.total_duration.as_secs_f64() * 100.0) as u64;
        percent.min(100)
    }

    pub fn set_running(&mut self, running: bool) {
        if running && self.paused {
            // Resume: reset start time so elapsed tracking works
            self.start_time = Some(Instant::now() - self.paused_elapsed);
            self.paused = false;
            self.manual_override = None; // Clear override when resuming time-based tracking
        } else if !running && !self.paused {
            // Pause: save elapsed time
            if let Some(start) = self.start_time {
                self.paused_elapsed = start.elapsed();
            }
            self.paused = true;
        }
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.paused = true;
        self.paused_elapsed = Duration::ZERO;
        self.manual_override = None;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_stage(&self) -> u8 {
        let percent = self.get_progress_percent();
        ((percent / 20).min(5)) as u8
    }
}
