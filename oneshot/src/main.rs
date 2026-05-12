use eframe::egui;
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    current_mode: GameMode,
    solstice: ModeSettings,
    base_game: ModeSettings,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
enum GameMode {
    Solstice,
    BaseGame,
}

#[derive(Serialize, Deserialize, Clone)]
struct ModeSettings {
    volume: f32,
    theme: ThemeColors,
    background_path: Option<String>,
    last_song_index: usize,
    last_position: u64,
    library_path: Option<String>,
    shuffle_enabled: bool,
}

impl Default for ModeSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            theme: ThemeColors::default(),
            background_path: None,
            last_song_index: 0,
            last_position: 0,
            library_path: None,
            shuffle_enabled: false,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current_mode: GameMode::BaseGame,
            solstice: ModeSettings {
                theme: ThemeColors::solstice_theme(),
                ..Default::default()
            },
            base_game: ModeSettings {
                theme: ThemeColors::default(),
                ..Default::default()
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ThemeColors {
    background: [u8; 4],
    button: [u8; 4],
    text: [u8; 4],
    accent: [u8; 4],
    sidebar: [u8; 4],
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Base game theme - darker purples
        Self {
            background: [20, 20, 30, 255],
            button: [80, 60, 100, 255],
            text: [255, 255, 200, 255],
            accent: [150, 100, 200, 255],
            sidebar: [30, 30, 40, 255],
        }
    }
}

impl ThemeColors {
    fn solstice_theme() -> Self {
        // Solstice theme - warmer, brighter colors
        Self {
            background: [25, 30, 40, 255],
            button: [100, 80, 60, 255],
            text: [255, 240, 200, 255],
            accent: [200, 150, 100, 255],
            sidebar: [35, 40, 50, 255],
        }
    }

    fn to_color32(&self, color: [u8; 4]) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
    }

    fn from_color32(color: egui::Color32) -> [u8; 4] {
        [color.r(), color.g(), color.b(), color.a()]
    }
}

struct MusicPlayer {
    songs: Vec<PathBuf>,
    current_index: usize,
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Option<OutputStream>,
    is_playing: bool,
    shuffle_order: Vec<usize>,
    settings: Settings,
    current_position: Arc<Mutex<Duration>>,
    total_duration: Arc<Mutex<Duration>>,
    background_image: Option<egui::TextureHandle>,
    show_settings: bool,
    search_query: String,
    filtered_songs: Vec<usize>,
    settings_path: PathBuf,
    repeat_mode: RepeatMode,
    last_mode_switch: Option<Instant>,
    mode_switch_cooldown: Duration,
}

#[derive(PartialEq, Clone, Copy)]
enum RepeatMode {
    Off,
    All,
    One,
}

impl MusicPlayer {
    fn new() -> Self {
        let settings_path = PathBuf::from("player_settings.json");
        let settings = Self::load_settings(&settings_path);

        let mut player = Self {
            songs: Vec::new(),
            current_index: 0,
            sink: Arc::new(Mutex::new(None)),
            _stream: None,
            is_playing: false,
            shuffle_order: Vec::new(),
            settings: settings.clone(),
            current_position: Arc::new(Mutex::new(Duration::from_secs(0))),
            total_duration: Arc::new(Mutex::new(Duration::from_secs(0))),
            background_image: None,
            show_settings: false,
            search_query: String::new(),
            filtered_songs: Vec::new(),
            settings_path,
            repeat_mode: RepeatMode::Off,
            last_mode_switch: None,
            mode_switch_cooldown: Duration::from_secs(5),
        };

        // Load library and background for current mode
        player.load_current_mode_data();

        player
    }

    fn get_current_mode_settings(&self) -> &ModeSettings {
        match self.settings.current_mode {
            GameMode::Solstice => &self.settings.solstice,
            GameMode::BaseGame => &self.settings.base_game,
        }
    }

    fn get_current_mode_settings_mut(&mut self) -> &mut ModeSettings {
        match self.settings.current_mode {
            GameMode::Solstice => &mut self.settings.solstice,
            GameMode::BaseGame => &mut self.settings.base_game,
        }
    }

    fn load_current_mode_data(&mut self) {
        let mode_settings = self.get_current_mode_settings().clone();

        // Load library from saved path
        if let Some(path) = &mode_settings.library_path {
            self.load_library_from_path(PathBuf::from(path));
        }
    }

    fn can_switch_mode(&self) -> bool {
        if let Some(last_switch) = self.last_mode_switch {
            last_switch.elapsed() >= self.mode_switch_cooldown
        } else {
            true
        }
    }

    fn switch_mode(&mut self, ctx: &egui::Context) {
        if !self.can_switch_mode() {
            return;
        }

        // Stop current playback
        self.stop_playback();

        // Save current mode settings
        self.save_current_mode_state();

        // Switch mode
        self.settings.current_mode = match self.settings.current_mode {
            GameMode::Solstice => GameMode::BaseGame,
            GameMode::BaseGame => GameMode::Solstice,
        };

        // Clear current state
        self.songs.clear();
        self.background_image = None;
        self.search_query.clear();

        // Load new mode data
        self.load_current_mode_data();

        // Load background for new mode
        let bg_path = self.get_current_mode_settings().background_path.clone();
        if let Some(path) = bg_path {
            self.apply_background_from_path(&PathBuf::from(path), ctx);
        }

        // Auto-play first song if library exists
        if !self.songs.is_empty() {
            self.current_index = 0;
            self.play_current();
        }

        self.last_mode_switch = Some(Instant::now());
        self.save_settings();
    }

    fn save_current_mode_state(&mut self) {
        let current_index = self.current_index;
        let last_position = self.current_position.lock().unwrap().as_secs();
        let shuffle_enabled = self.get_current_mode_settings().shuffle_enabled;

        let mode_settings = self.get_current_mode_settings_mut();
        mode_settings.last_song_index = current_index;
        mode_settings.last_position = last_position;
        mode_settings.shuffle_enabled = shuffle_enabled;
    }

    fn stop_playback(&mut self) {
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
        self.is_playing = false;
    }

    fn load_settings(path: &PathBuf) -> Settings {
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Settings::default()
        }
    }

    fn save_settings(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            if let Ok(mut file) = File::create(&self.settings_path) {
                file.write_all(json.as_bytes()).ok();
            }
        }
    }

    fn load_library(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg"])
            .pick_file()
            {
                if let Some(parent_dir) = path.parent() {
                    let mode_settings = self.get_current_mode_settings_mut();
                    mode_settings.library_path = Some(parent_dir.to_string_lossy().to_string());
                    self.load_library_from_path(parent_dir.to_path_buf());
                    self.save_settings();
                }
            }
    }

    fn load_library_from_path(&mut self, dir: PathBuf) {
        self.songs.clear();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if let Some(ext) = entry_path.extension() {
                    if ["mp3", "wav", "flac", "ogg"].contains(&ext.to_str().unwrap_or("")) {
                        self.songs.push(entry_path);
                    }
                }
            }
        }
        self.songs.sort();
        self.initialize_shuffle();
        self.update_filtered_songs();

        // Restore last position
        let last_index = self.get_current_mode_settings().last_song_index;
        if last_index < self.songs.len() {
            self.current_index = last_index;
        }
    }

    fn load_background_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
            {
                let mode_settings = self.get_current_mode_settings_mut();
                mode_settings.background_path = Some(path.to_string_lossy().to_string());
                self.apply_background_from_path(&path, ctx);
                self.save_settings();
            }
    }

    fn apply_background_from_path(&mut self, path: &PathBuf, ctx: &egui::Context) {
        if let Ok(image) = image::open(path) {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.background_image = Some(ctx.load_texture("background", color_image, Default::default()));
        }
    }

    fn initialize_shuffle(&mut self) {
        self.shuffle_order = (0..self.songs.len()).collect();
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.shuffle_order.shuffle(&mut rng);
    }

    fn get_actual_index(&self) -> usize {
        let mode_settings = self.get_current_mode_settings();
        if mode_settings.shuffle_enabled && self.current_index < self.shuffle_order.len() {
            self.shuffle_order[self.current_index]
        } else {
            self.current_index
        }
    }

    fn play_current(&mut self) {
        if self.songs.is_empty() {
            return;
        }

        let actual_idx = self.get_actual_index();
        let path = &self.songs[actual_idx];

        if let Ok(file) = File::open(path) {
            let (stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            let buf_reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(buf_reader) {
                let duration = source.total_duration().unwrap_or(Duration::from_secs(0));
                *self.total_duration.lock().unwrap() = duration;
                *self.current_position.lock().unwrap() = Duration::from_secs(0);

                let volume = self.get_current_mode_settings().volume;
                sink.set_volume(volume);
                sink.append(source);
                sink.play();

                *self.sink.lock().unwrap() = Some(sink);
                self._stream = Some(stream);
                self.is_playing = true;
                self.save_settings();
            }
        }
    }

    fn play_song_at_index(&mut self, index: usize) {
        if index < self.songs.len() {
            self.current_index = index;
            self.play_current();
        }
    }

    fn toggle_play_pause(&mut self) {
        let has_sink = self.sink.lock().unwrap().is_some();

        if has_sink {
            if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                if self.is_playing {
                    sink.pause();
                    self.is_playing = false;
                } else {
                    sink.play();
                    self.is_playing = true;
                }
            }
        } else if !self.songs.is_empty() {
            self.play_current();
        }
    }

    fn next_song(&mut self) {
        if !self.songs.is_empty() {
            match self.repeat_mode {
                RepeatMode::One => {
                    self.play_current();
                },
                _ => {
                    self.current_index = (self.current_index + 1) % self.songs.len();
                    self.play_current();
                }
            }
        }
    }

    fn previous_song(&mut self) {
        if !self.songs.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.songs.len() - 1
            } else {
                self.current_index - 1
            };
            self.play_current();
        }
    }

    fn seek_to(&mut self, position: f32) {
        *self.current_position.lock().unwrap() =
        Duration::from_secs_f32(self.total_duration.lock().unwrap().as_secs_f32() * position);
        self.play_current();
    }

    fn update_position(&self) {
        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
            if !sink.is_paused() && !sink.empty() {
                let mut pos = self.current_position.lock().unwrap();
                *pos += Duration::from_millis(16);
            }
        }
    }

    fn update_filtered_songs(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_songs = (0..self.songs.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_songs = self.songs
            .iter()
            .enumerate()
            .filter(|(_, path)| {
                path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        }
    }

    fn format_duration(secs: f32) -> String {
        let total_secs = secs as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

impl eframe::App for MusicPlayer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_position();
        ctx.request_repaint_after(Duration::from_millis(100)); // Reduced from 16ms to fix hanging

        let current_theme = self.get_current_mode_settings().theme.clone();

        // Apply theme
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.inactive.bg_fill = current_theme.to_color32(current_theme.button);
        style.visuals.widgets.hovered.bg_fill = current_theme.to_color32(current_theme.accent);
        style.visuals.widgets.active.bg_fill = current_theme.to_color32(current_theme.accent);
        style.visuals.override_text_color = Some(current_theme.to_color32(current_theme.text));
        style.visuals.extreme_bg_color = current_theme.to_color32(current_theme.sidebar);
        ctx.set_style(style);

        // Background
        if self.background_image.is_none() {
            if let Some(bg_path) = &self.get_current_mode_settings().background_path.clone() {
                self.apply_background_from_path(&PathBuf::from(bg_path), ctx);
            }
        }

        egui::SidePanel::left("sidebar")
        .default_width(250.0)
        .frame(egui::Frame::none().fill(current_theme.to_color32(current_theme.sidebar)))
        .show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                let mode_name = match self.settings.current_mode {
                    GameMode::Solstice => "♪ Solstice ☀",
                    GameMode::BaseGame => "♪ OneShot 🌙",
                };
                ui.heading(egui::RichText::new(mode_name).size(24.0));
            });
            ui.add_space(20.0);

            // Mode switch button with cooldown indicator
            let can_switch = self.can_switch_mode();
            let button_text = if can_switch {
                match self.settings.current_mode {
                    GameMode::Solstice => "🌙 Switch to Base Game",
                    GameMode::BaseGame => "☀ Switch to Solstice",
                }
            } else {
                let remaining = self.mode_switch_cooldown.as_secs()
                - self.last_mode_switch.unwrap().elapsed().as_secs();
                &format!("⏳ Wait {}s...", remaining)
            };

            ui.add_enabled_ui(can_switch, |ui| {
                if ui.button(button_text).clicked() {
                    self.switch_mode(ctx);
                }
            });

            ui.add_space(10.0);

            if ui.button("📁 Load Library").clicked() {
                self.load_library();
            }

            if ui.button("🖼 Background").clicked() {
                self.load_background_image(ctx);
            }

            if ui.button("⚙ Settings").clicked() {
                self.show_settings = !self.show_settings;
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(egui::RichText::new("Library").size(16.0));
            ui.add_space(5.0);

            // Search bar
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("🔍 Search songs...")
            );
            if search_response.changed() {
                self.update_filtered_songs();
            }

            ui.add_space(10.0);

            // Song list
            egui::ScrollArea::vertical().show(ui, |ui| {
                let filtered = self.filtered_songs.clone();
                for &song_idx in &filtered {
                    let song_name = self.songs[song_idx]
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();

                    let is_current = song_idx == self.get_actual_index() && self.is_playing;
                    let text = if is_current {
                        egui::RichText::new(format!("▶ {}", song_name)).color(current_theme.to_color32(current_theme.accent))
                    } else {
                        egui::RichText::new(song_name.to_string())
                    };

                    if ui.selectable_label(is_current, text).clicked() {
                        self.play_song_at_index(song_idx);
                    }
                }
            });
        });

        egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(current_theme.to_color32(current_theme.background)))
        .show(ctx, |ui| {
            // Draw background image BEHIND everything with proper opacity
            if let Some(texture) = &self.background_image {
                let painter = ui.painter();
                let rect = ui.max_rect();

                // Calculate aspect-ratio-preserving size
                let image_size = texture.size_vec2();
                let aspect = image_size.x / image_size.y;
                let screen_aspect = rect.width() / rect.height();

                let (width, height) = if aspect > screen_aspect {
                    (rect.width(), rect.width() / aspect)
                } else {
                    (rect.height() * aspect, rect.height())
                };

                let x_offset = (rect.width() - width) / 2.0;
                let y_offset = (rect.height() - height) / 2.0;

                let image_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + x_offset, rect.min.y + y_offset),
                                                           egui::vec2(width, height)
                );

                // Draw with reduced opacity so it doesn't overwhelm controls
                painter.image(
                    texture.id(),
                              image_rect,
                              egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                              egui::Color32::from_white_alpha(60) // Very transparent
                );
            }

            ui.vertical_centered(|ui| {
                ui.add_space(40.0);

                // Now playing
                if !self.songs.is_empty() {
                    let actual_idx = self.get_actual_index();
                    let song_name = self.songs[actual_idx]
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();

                    ui.heading(egui::RichText::new("Now Playing").size(18.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(song_name.to_string()).size(28.0).strong());
                    ui.label(format!("Track {} of {}", self.current_index + 1, self.songs.len()));
                } else {
                    ui.heading("No Library Loaded");
                    ui.label("Click 'Load Library' to get started");
                }

                ui.add_space(40.0);

                // Progress bar
                let pos = self.current_position.lock().unwrap().as_secs_f32();
                let total = self.total_duration.lock().unwrap().as_secs_f32();
                let progress = if total > 0.0 { pos / total } else { 0.0 };

                ui.label(format!("{} / {}",
                                 Self::format_duration(pos),
                                 Self::format_duration(total)
                ));

                ui.add_space(5.0);

                let mut seek_value = progress;
                let slider_response = ui.add_sized(
                    [400.0, 20.0],
                    egui::Slider::new(&mut seek_value, 0.0..=1.0).show_value(false)
                );

                // Only seek when user releases the slider
                if slider_response.drag_stopped() {
                    self.seek_to(seek_value);
                }

                ui.add_space(30.0);

                // Main controls
                ui.horizontal(|ui| {
                    ui.add_space(120.0);

                    if ui.add_sized([50.0, 50.0], egui::Button::new("⏮")).clicked() {
                        self.previous_song();
                    }

                    if ui.add_sized([60.0, 60.0], egui::Button::new(
                        if self.is_playing { "⏸" } else { "▶" }
                    )).clicked() {
                        self.toggle_play_pause();
                    }

                    if ui.add_sized([50.0, 50.0], egui::Button::new("⏭")).clicked() {
                        self.next_song();
                    }
                });

                ui.add_space(20.0);

                // Secondary controls
                ui.horizontal(|ui| {
                    ui.add_space(80.0);

                    let shuffle_enabled = self.get_current_mode_settings().shuffle_enabled;
                    let shuffle_text = if shuffle_enabled { "🔀 ON" } else { "🔀" };
                    if ui.button(shuffle_text).clicked() {
                        let mode_settings = self.get_current_mode_settings_mut();
                        mode_settings.shuffle_enabled = !mode_settings.shuffle_enabled;
                        if mode_settings.shuffle_enabled {
                            self.shuffle();
                        }
                        self.save_settings();
                    }

                    let repeat_text = match self.repeat_mode {
                        RepeatMode::Off => "🔁",
                        RepeatMode::All => "🔁 All",
                        RepeatMode::One => "🔂 One",
                    };
                    if ui.button(repeat_text).clicked() {
                        self.repeat_mode = match self.repeat_mode {
                            RepeatMode::Off => RepeatMode::All,
                            RepeatMode::All => RepeatMode::One,
                            RepeatMode::One => RepeatMode::Off,
                        };
                    }

                    ui.label("🔊");
                    let mut volume = self.get_current_mode_settings().volume;
                    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false)).changed() {
                        let mode_settings = self.get_current_mode_settings_mut();
                        mode_settings.volume = volume;
                        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                            sink.set_volume(volume);
                        }
                        self.save_settings();
                    }
                });
            });
        });

        // Settings window
        if self.show_settings {
            egui::Window::new("⚙ Settings")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Theme Colors");
                ui.label("Note: Themes auto-switch with game modes");
                ui.add_space(10.0);

                // Clone values we need to avoid borrow issues
                let current_mode = self.settings.current_mode.clone();
                let mut theme = self.get_current_mode_settings().theme.clone();
                let mut bg_path = self.get_current_mode_settings().background_path.clone();
                let mut settings_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Background:");
                    let mut color = theme.to_color32(theme.background);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        theme.background = ThemeColors::from_color32(color);
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Sidebar:");
                    let mut color = theme.to_color32(theme.sidebar);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        theme.sidebar = ThemeColors::from_color32(color);
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Button:");
                    let mut color = theme.to_color32(theme.button);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        theme.button = ThemeColors::from_color32(color);
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Text:");
                    let mut color = theme.to_color32(theme.text);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        theme.text = ThemeColors::from_color32(color);
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Accent:");
                    let mut color = theme.to_color32(theme.accent);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        theme.accent = ThemeColors::from_color32(color);
                        settings_changed = true;
                    }
                });

                ui.add_space(10.0);

                if ui.button("Reset Current Theme").clicked() {
                    theme = match current_mode {
                        GameMode::Solstice => ThemeColors::solstice_theme(),
                  GameMode::BaseGame => ThemeColors::default(),
                    };
                    settings_changed = true;
                }

                if ui.button("Clear Background").clicked() {
                    bg_path = None;
                    self.background_image = None;
                    settings_changed = true;
                }

                // Apply changes after UI is done
                if settings_changed {
                    let mode_settings = self.get_current_mode_settings_mut();
                    mode_settings.theme = theme;
                    mode_settings.background_path = bg_path;
                    self.save_settings();
                }
            });
        }

        // Auto-advance
        let should_advance = {
            if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                sink.empty() && self.is_playing && self.repeat_mode != RepeatMode::One
            } else {
                false
            }
        };

        if should_advance {
            if self.repeat_mode == RepeatMode::All || self.current_index < self.songs.len() - 1 {
                self.next_song();
            } else {
                self.is_playing = false;
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_current_mode_state();
        self.save_settings();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([1000.0, 700.0])
        .with_title("OneShot Music Player"),
        ..Default::default()
    };

    eframe::run_native(
        "OneShot Music Player",
        options,
        Box::new(|_cc| Ok(Box::new(MusicPlayer::new()))),
    )
}

// Cargo.toml:
// [dependencies]
// eframe = "0.28"
// egui = "0.28"
// rodio = "0.17"
// rfd = "0.14"
// image = "0.24"
// rand = "0.8"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
