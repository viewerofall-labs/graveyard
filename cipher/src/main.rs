use eframe::egui;
use std::collections::HashMap;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "CIPHER OS - Decrypt and Discover",
        options,
        Box::new(|_cc| Ok(Box::new(CipherApp::new()))),
    )
}

// ============================================================================
// APP WINDOW HELPER
// ============================================================================

struct AppWindow {
    open: bool,
    pos: egui::Pos2,
    size: egui::Vec2,
}

impl AppWindow {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            open: true,
            pos: egui::Pos2::new(x, y),
            size: egui::Vec2::new(width, height),
        }
    }
}

// ============================================================================
// MAIN APP
// ============================================================================

struct CipherApp {
    game_state: GameState,
    terminal_window: Option<AppWindow>,
    explorer_window: Option<AppWindow>,
    editor_window: Option<AppWindow>,
    decryptor_window: Option<AppWindow>,
    selected_file_content: String,
}

impl CipherApp {
    fn new() -> Self {
        Self {
            game_state: GameState::new(),
            terminal_window: None,
            explorer_window: None,
            editor_window: None,
            decryptor_window: None,
            selected_file_content: String::new(),
        }
    }
}

impl eframe::App for CipherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Desktop background
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 30, 50));

            // Grid pattern
            let painter = ui.painter();
            for i in 0..20 {
                let y = rect.min.y + (i as f32 * 40.0);
                painter.line_segment(
                    [egui::Pos2::new(rect.min.x, y), egui::Pos2::new(rect.max.x, y)],
                                     egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(255, 255, 255, 10)),
                );
            }
            for i in 0..30 {
                let x = rect.min.x + (i as f32 * 40.0);
                painter.line_segment(
                    [egui::Pos2::new(x, rect.min.y), egui::Pos2::new(x, rect.max.y)],
                                     egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(255, 255, 255, 10)),
                );
            }

            // Desktop icons
            ui.vertical(|ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(30.0);

                    if ui.add_sized([100.0, 100.0], egui::Button::new(
                        egui::RichText::new("💻\nTerminal").size(16.0)
                    ).fill(egui::Color32::from_rgba_premultiplied(40, 50, 70, 200))).clicked() {
                        self.terminal_window = Some(AppWindow::new(100.0, 100.0, 600.0, 400.0));
                    }

                    ui.add_space(20.0);

                    if ui.add_sized([100.0, 100.0], egui::Button::new(
                        egui::RichText::new("📁\nFiles").size(16.0)
                    ).fill(egui::Color32::from_rgba_premultiplied(40, 50, 70, 200))).clicked() {
                        self.explorer_window = Some(AppWindow::new(150.0, 150.0, 500.0, 500.0));
                    }

                    ui.add_space(20.0);

                    if ui.add_sized([100.0, 100.0], egui::Button::new(
                        egui::RichText::new("🔓\nDecryptor").size(16.0)
                    ).fill(egui::Color32::from_rgba_premultiplied(40, 50, 70, 200))).clicked() {
                        self.decryptor_window = Some(AppWindow::new(200.0, 200.0, 600.0, 500.0));
                    }
                });

                ui.add_space(300.0);
                ui.horizontal(|ui| {
                    ui.add_space(30.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("CIPHER OS v1.0 - Level {}", self.game_state.level_manager.current_level + 1))
                        .size(18.0).color(egui::Color32::from_rgb(100, 200, 255)));

                        if !self.game_state.unlocked_keys.is_empty() {
                            ui.label(egui::RichText::new(format!("🔑 Keys: {}", self.game_state.unlocked_keys.join(", ")))
                            .color(egui::Color32::GOLD));
                        }

                        if self.game_state.environment.all_objectives_complete() {
                            ui.label(egui::RichText::new("✅ All files decrypted! Use Terminal to advance.")
                            .color(egui::Color32::GREEN).size(16.0));
                        }
                    });
                });
            });
        });

        // Show windows
        if self.terminal_window.is_some() {
            self.show_terminal_window(ctx);
        }

        if self.explorer_window.is_some() {
            self.show_explorer_window(ctx);
        }

        if self.editor_window.is_some() {
            self.show_editor_window(ctx);
        }

        if self.decryptor_window.is_some() {
            self.show_decryptor_window(ctx);
        }
    }
}

impl CipherApp {
    fn show_terminal_window(&mut self, ctx: &egui::Context) {
        let Some(window) = &mut self.terminal_window else {
            return;
        };
        let mut open = window.open;

        egui::Window::new("💻 Terminal")
        .open(&mut open)
        .default_pos(window.pos)
        .default_size(window.size)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(300.0).stick_to_bottom(true).show(ui, |ui| {
                for line in &self.game_state.terminal_buffer {
                    ui.label(egui::RichText::new(line).font(egui::FontId::monospace(12.0)));
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(">");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.game_state.terminal_input)
                    .desired_width(ui.available_width() - 80.0)
                    .font(egui::FontId::monospace(12.0))
                );

                let should_execute = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if ui.button("Send").clicked() || should_execute {
                    let cmd = self.game_state.terminal_input.clone();
                    self.game_state.terminal_buffer.push(format!("> {}", cmd));

                    match self.game_state.execute_terminal_command(&cmd) {
                        Ok(output) => {
                            if !output.is_empty() {
                                self.game_state.terminal_buffer.push(output);
                            }
                        }
                        Err(e) => {
                            self.game_state.terminal_buffer.push(format!("Error: {}", e));
                        }
                    }

                    self.game_state.terminal_input.clear();
                    if should_execute {
                        response.request_focus();
                    }
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("Commands: scan, list, clear, advance <file> <key>, help").size(10.0).color(egui::Color32::GRAY));
        });

        if !open {
            self.terminal_window = None;
        }
    }

    fn show_explorer_window(&mut self, ctx: &egui::Context) {
        let Some(window) = &mut self.explorer_window else {
            return;
        };
        let mut open = window.open;

        egui::Window::new("📁 File Explorer")
        .open(&mut open)
        .default_pos(window.pos)
        .default_size(window.size)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Location: /home/user/documents").size(12.0).color(egui::Color32::GRAY));
            ui.separator();

            let files = self.game_state.environment.list_files();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (filename, obj) in files {
                    let icon = if obj.encrypted { "🔒" } else { "📄" };
                    let color = if obj.encrypted {
                        egui::Color32::from_rgb(255, 150, 150)
                    } else {
                        egui::Color32::WHITE
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icon).size(18.0));

                        if ui.add_sized([300.0, 25.0], egui::Button::new(
                            egui::RichText::new(filename.as_str()).color(color).size(14.0)
                        )).clicked() {
                            if !obj.encrypted {
                                if let Some(key) = &obj.unlocks_key {
                                    if !self.game_state.unlocked_keys.contains(key) {
                                        self.game_state.unlocked_keys.push(key.clone());
                                        self.game_state.terminal_buffer.push(format!("🔑 KEY DISCOVERED: {}", key));
                                    }
                                }
                            }

                            self.selected_file_content = obj.file_content.clone().unwrap_or_default();
                            self.editor_window = Some(AppWindow::new(250.0, 100.0, 500.0, 400.0));
                            self.game_state.current_app = App::Editor(filename.clone());
                        }

                        if obj.encrypted {
                            ui.label(egui::RichText::new("[ENCRYPTED]").color(egui::Color32::from_rgb(255, 150, 150)).size(11.0));
                        }
                    });

                    ui.add_space(3.0);
                }
            });
        });

        if !open {
            self.explorer_window = None;
        }
    }

    fn show_editor_window(&mut self, ctx: &egui::Context) {
        let Some(window) = &mut self.editor_window else {
            return;
        };
        let mut open = window.open;
        let filename = if let App::Editor(ref name) = self.game_state.current_app {
            name.clone()
        } else {
            "Unknown".to_string()
        };

        egui::Window::new(format!("📝 {}", filename))
        .open(&mut open)
        .default_pos(window.pos)
        .default_size(window.size)
        .resizable(true)
        .show(ctx, |ui| {
            if let Some(obj) = self.game_state.environment.get_file(&filename) {
                if obj.encrypted {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label(egui::RichText::new("🔒 ENCRYPTED FILE").size(20.0).color(egui::Color32::from_rgb(255, 150, 150)));
                        ui.add_space(15.0);
                        ui.label("Copy the encrypted text below and paste it into the Decryptor.");
                        ui.add_space(10.0);
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.selected_file_content.as_str())
                            .font(egui::FontId::monospace(12.0))
                            .desired_width(f32::INFINITY)
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy to Clipboard").clicked() {
                            ui.output_mut(|o| o.copied_text = self.selected_file_content.clone());
                        }
                        ui.label(egui::RichText::new("← Copy this and paste into Decryptor").color(egui::Color32::GRAY).size(11.0));
                    });
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.selected_file_content.as_str())
                            .font(egui::FontId::monospace(12.0))
                            .desired_width(f32::INFINITY)
                        );
                    });
                }
            }
        });

        if !open {
            self.editor_window = None;
        }
    }

    fn show_decryptor_window(&mut self, ctx: &egui::Context) {
        let Some(window) = &mut self.decryptor_window else {
            return;
        };
        let mut open = window.open;

        egui::Window::new("🔓 Decryptor Utility")
        .open(&mut open)
        .default_pos(window.pos)
        .default_size(window.size)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Paste encrypted text and select decryption method").size(14.0));
            ui.add_space(5.0);

            ui.label(egui::RichText::new(format!("🔑 Available Keys: {}",
                                                 if self.game_state.unlocked_keys.is_empty() {
                                                     "None (read files to find keys!)".to_string()
                                                 } else {
                                                     self.game_state.unlocked_keys.join(", ")
                                                 })).color(egui::Color32::GOLD).size(12.0));

            ui.separator();

            ui.label("Encrypted Text:");
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.game_state.decryptor_input)
                    .font(egui::FontId::monospace(11.0))
                    .desired_width(f32::INFINITY)
                    .hint_text("Paste encrypted text here...")
                );
            });

            ui.add_space(10.0);
            ui.label("Select Decryption Method:");

            ui.horizontal_wrapped(|ui| {
                if ui.button("Base64").clicked() {
                    self.game_state.decrypt_with_method("base64");
                }
                if ui.button("ROT13").clicked() {
                    self.game_state.decrypt_with_method("rot13");
                }
                if ui.button("ROT5").clicked() {
                    self.game_state.decrypt_with_method("rot5");
                }
                if ui.button("Caesar +3").clicked() {
                    self.game_state.decrypt_with_method("caesar3");
                }
                if ui.button("Caesar +7").clicked() {
                    self.game_state.decrypt_with_method("caesar7");
                }
                if ui.button("Reverse").clicked() {
                    self.game_state.decrypt_with_method("reverse");
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("⚠ Some methods are decoys and won't work!").color(egui::Color32::from_rgb(255, 200, 100)).size(11.0));

            ui.separator();
            ui.label("Decrypted Output:");
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.game_state.decryptor_output.as_str())
                    .font(egui::FontId::monospace(11.0))
                    .desired_width(f32::INFINITY)
                    .interactive(false)
                );
            });

            ui.add_space(5.0);
            if !self.game_state.decryptor_output.is_empty() {
                if ui.button("💾 Save Decrypted File").clicked() {
                    self.game_state.save_decrypted_content();
                }
            }
        });

        if !open {
            self.decryptor_window = None;
        }
    }
}

// ============================================================================
// GAME STATE
// ============================================================================

#[derive(PartialEq, Clone)]
enum App {
    Desktop,
    Terminal,
    FileExplorer,
    Editor(String),
    Decryptor,
}

struct GameState {
    environment: Environment,
    current_app: App,
    terminal_buffer: Vec<String>,
    terminal_input: String,
    unlocked_keys: Vec<String>,
    level_manager: LevelManager,
    decryptor_input: String,
    decryptor_output: String,
}

impl GameState {
    fn new() -> Self {
        let mut game = GameState {
            environment: Environment::new(0),
            current_app: App::Desktop,
            terminal_buffer: vec![
                "CIPHER OS v1.0 initialized...".to_string(),
                "Welcome to the system.".to_string(),
                "Type 'help' for available commands.".to_string(),
                "".to_string(),
            ],
            terminal_input: String::new(),
            unlocked_keys: Vec::new(),
            level_manager: LevelManager::new(),
            decryptor_input: String::new(),
            decryptor_output: String::new(),
        };

        game.level_manager.load_level(0);
        game
    }

    fn decrypt_with_method(&mut self, method: &str) {
        if self.decryptor_input.trim().is_empty() {
            self.decryptor_output = "Error: No input text provided".to_string();
            return;
        }

        let input = self.decryptor_input.trim();

        if let Some(colon_pos) = input.find(':') {
            let required_key = &input[..colon_pos];
            let encrypted_part = &input[colon_pos + 1..];

            if !self.unlocked_keys.contains(&required_key.to_string()) {
                self.decryptor_output = format!("Error: Missing required key '{}'\nRead files to discover keys!", required_key);
                return;
            }

            if method == "rot13" {
                self.decryptor_output = rot13(encrypted_part);
                self.terminal_buffer.push(format!("✅ Successfully decrypted using key: {}", required_key));
            } else {
                self.decryptor_output = "Error: Wrong decryption method for this key-encrypted file\nTry: ROT13".to_string();
            }
        } else {
            match method {
                "base64" => {
                    match decode_base64(input) {
                        Ok(decoded) => {
                            self.decryptor_output = decoded;
                            self.terminal_buffer.push("✅ Base64 decryption successful".to_string());
                        }
                        Err(e) => {
                            self.decryptor_output = format!("Decryption failed: {}", e);
                        }
                    }
                }
                "rot13" => {
                    self.decryptor_output = rot13(input);
                }
                "rot5" => {
                    self.decryptor_output = input.chars().map(|c| {
                        match c {
                            'A'..='U' | 'a'..='u' => ((c as u8) + 5) as char,
                                                              'V'..='Z' | 'v'..='z' => ((c as u8) - 21) as char,
                                                              _ => c,
                        }
                    }).collect();
                }
                "caesar3" => {
                    self.decryptor_output = input.chars().map(|c| {
                        match c {
                            'A'..='W' | 'a'..='w' => ((c as u8) + 3) as char,
                                                              'X'..='Z' | 'x'..='z' => ((c as u8) - 23) as char,
                                                              _ => c,
                        }
                    }).collect();
                }
                "caesar7" => {
                    self.decryptor_output = input.chars().map(|c| {
                        match c {
                            'A'..='S' | 'a'..='s' => ((c as u8) + 7) as char,
                                                              'T'..='Z' | 't'..='z' => ((c as u8) - 19) as char,
                                                              _ => c,
                        }
                    }).collect();
                }
                "reverse" => {
                    self.decryptor_output = input.chars().rev().collect();
                }
                _ => {
                    self.decryptor_output = "Unknown decryption method".to_string();
                }
            }
        }
    }

    fn save_decrypted_content(&mut self) {
        for (filename, obj) in self.environment.objects.iter_mut() {
            if let Some(content) = &obj.file_content {
                if content == &self.decryptor_input.trim() && obj.encrypted {
                    obj.encrypted = false;
                    obj.file_content = Some(self.decryptor_output.clone());
                    self.terminal_buffer.push(format!("✅ File '{}' decrypted and saved!", filename));

                    if let Some(key) = &obj.unlocks_key {
                        if !self.unlocked_keys.contains(key) {
                            self.unlocked_keys.push(key.clone());
                            self.terminal_buffer.push(format!("🔑 NEW KEY DISCOVERED: {}", key));
                        }
                    }

                    self.decryptor_input.clear();
                    self.decryptor_output.clear();
                    return;
                }
            }
        }

        self.terminal_buffer.push("Warning: Could not match decrypted content to a file".to_string());
    }

    fn execute_terminal_command(&mut self, cmd: &str) -> Result<String, String> {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        match parts[0] {
            "help" => {
                Ok("Available commands:\n  scan - Scan for files\n  list - List all files\n  clear - Clear terminal\n  advance <file> <key> - Progress to next level\n  help - Show this help".to_string())
            }
            "scan" => {
                let total = self.environment.list_files().len();
                let encrypted = self.environment.list_files().iter().filter(|(_, o)| o.encrypted).count();
                Ok(format!("Scanning system...\nTotal files: {}\nEncrypted: {}\nKeys found: {}",
                           total, encrypted, self.unlocked_keys.len()))
            }
            "list" => {
                let mut output = "Files:\n".to_string();
                for (name, obj) in self.environment.list_files() {
                    let status = if obj.encrypted { "[ENCRYPTED]" } else { "[DECRYPTED]" };
                    output.push_str(&format!("  {} {}\n", status, name));
                }
                Ok(output)
            }
            "clear" => {
                self.terminal_buffer.clear();
                Ok(String::new())
            }
            "advance" => {
                if parts.len() < 3 {
                    return Err("Usage: advance <main_file> <key>".to_string());
                }

                let file = parts[1];
                let key = parts[2];

                if let Some(progression) = &self.level_manager.current_progression {
                    if progression.required_file == file && progression.required_key == key {
                        self.level_manager.current_level += 1;

                        if self.level_manager.current_level >= self.level_manager.total_levels {
                            return Ok("🎉 GAME COMPLETE! You've uncovered all the secrets!".to_string());
                        }

                        self.level_manager.load_level(self.level_manager.current_level);
                        self.environment = Environment::new(self.level_manager.current_level);
                        self.unlocked_keys.clear();
                        self.current_app = App::Desktop;
                        self.decryptor_input.clear();
                        self.decryptor_output.clear();

                        Ok(format!("✅ Advancing to Level {}...", self.level_manager.current_level + 1))
                    } else {
                        Err("Incorrect file or key. Hint: The main file contains the final secret, and you need its decryption key.".to_string())
                    }
                } else {
                    Err("No progression configured for this level.".to_string())
                }
            }
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }
}

// ============================================================================
// ENVIRONMENT
// ============================================================================

struct Environment {
    objects: HashMap<String, GameObject>,
}

struct GameObject {
    name: String,
    file_content: Option<String>,
    encrypted: bool,
    cipher_type: Option<String>,
    unlocks_key: Option<String>,
}

impl Environment {
    fn new(level: usize) -> Self {
        let mut env = Environment {
            objects: HashMap::new(),
        };

        match level {
            0 => {
                env.objects.insert("README.txt".to_string(), GameObject {
                    name: "README.txt".to_string(),
                                   file_content: Some("Welcome to CIPHER OS.\n\nYour mission: Decrypt all files.\n\nStart by reading files to discover keys.\nSome files contain encryption keys.\n\nTo progress to the next level:\n  1. Decrypt all files\n  2. Use Terminal command: advance <main_file> <key>\n\nGood luck.".to_string()),
                                   encrypted: false,
                                   cipher_type: None,
                                   unlocks_key: None,
                });

                env.objects.insert("note.txt".to_string(), GameObject {
                    name: "note.txt".to_string(),
                                   file_content: Some("Personal Log - Day 1:\n\nI've hidden the key in plain sight.\n\nKey discovered: ALPHA\n\nThis key unlocks the secret file.\nUse it wisely.".to_string()),
                                   encrypted: false,
                                   cipher_type: None,
                                   unlocks_key: Some("ALPHA".to_string()),
                });

                env.objects.insert("data.enc".to_string(), GameObject {
                    name: "data.enc".to_string(),
                                   file_content: Some("VGhlIGJhc2ljcyBhcmUgaW1wb3J0YW50Lg==".to_string()),
                                   encrypted: true,
                                   cipher_type: Some("base64".to_string()),
                                   unlocks_key: None,
                });

                env.objects.insert("secret.enc".to_string(), GameObject {
                    name: "secret.enc".to_string(),
                                   file_content: Some("ALPHA:Jrypbzr gb Yriry 2! Gur cnffjbeq vf: OENGB".to_string()),
                                   encrypted: true,
                                   cipher_type: Some("keyed_rot13".to_string()),
                                   unlocks_key: None,
                });
            }
            1 => {
                env.objects.insert("README.txt".to_string(), GameObject {
                    name: "README.txt".to_string(),
                                   file_content: Some("Level 2 - Intermediate Challenges\n\nThe puzzles grow more complex.\nMultiple keys may be required.\n\nKeep track of all discovered keys.".to_string()),
                                   encrypted: false,
                                   cipher_type: None,
                                   unlocks_key: None,
                });

                env.objects.insert("clue.txt".to_string(), GameObject {
                    name: "clue.txt".to_string(),
                                   file_content: Some("The first key is: BRAVO\nThe second key is hidden in encrypted files.".to_string()),
                                   encrypted: false,
                                   cipher_type: None,
                                   unlocks_key: Some("BRAVO".to_string()),
                });

                env.objects.insert("message.enc".to_string(), GameObject {
                    name: "message.enc".to_string(),
                                   file_content: Some("BRAVO:Gur frpbaq xrl vf: PUNEYVRR".to_string()),
                                   encrypted: true,
                                   cipher_type: Some("keyed_rot13".to_string()),
                                   unlocks_key: Some("CHARLIE".to_string()),
                });

                env.objects.insert("final.enc".to_string(), GameObject {
                    name: "final.enc".to_string(),
                                   file_content: Some("CHARLIE:Lbh'ir znfgrerq gur onfvpf! Svany cnffjbeq: QRYG N".to_string()),
                                   encrypted: true,
                                   cipher_type: Some("keyed_rot13".to_string()),
                                   unlocks_key: None,
                });
            }
            _ => {
                env.objects.insert("victory.txt".to_string(), GameObject {
                    name: "victory.txt".to_string(),
                                   file_content: Some("Congratulations!\n\nYou have completed all levels.\n\nYou are a master cryptographer.".to_string()),
                                   encrypted: false,
                                   cipher_type: None,
                                   unlocks_key: None,
                });
            }
        }

        env
    }

    fn list_files(&self) -> Vec<(&String, &GameObject)> {
        self.objects.iter().collect()
    }

    fn get_file(&self, name: &str) -> Option<&GameObject> {
        self.objects.get(name)
    }

    fn all_objectives_complete(&self) -> bool {
        self.objects.values().all(|obj| !obj.encrypted)
    }
}

// ============================================================================
// LEVEL MANAGER
// ============================================================================

struct LevelManager {
    current_level: usize,
    total_levels: usize,
    current_progression: Option<LevelProgression>,
}

struct LevelProgression {
    required_file: String,
    required_key: String,
}

impl LevelManager {
    fn new() -> Self {
        Self {
            current_level: 0,
            total_levels: 2,
            current_progression: None,
        }
    }

    fn load_level(&mut self, level: usize) {
        self.current_progression = match level {
            0 => Some(LevelProgression {
                required_file: "secret.enc".to_string(),
                      required_key: "BRAVO".to_string(),
            }),
            1 => Some(LevelProgression {
                required_file: "final.enc".to_string(),
                      required_key: "DELTA".to_string(),
            }),
            _ => None,
        };
    }
}

// ============================================================================
// CIPHER UTILITIES
// ============================================================================

fn rot13(input: &str) -> String {
    input.chars().map(|c| {
        match c {
            'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
                      'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
                      _ => c,
        }
    }).collect()
}

fn decode_base64(input: &str) -> Result<String, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    let bytes = base64_decode(&cleaned).map_err(|e| format!("Base64 decode error: {}", e))?;

    String::from_utf8(bytes).map_err(|e| format!("UTF-8 conversion error: {}", e))
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in input.chars() {
        if c == '=' {
            break;
        }

        let value = BASE64_CHARS.find(c).ok_or("Invalid base64 character")? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}
