use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, text_input, Column},
    Application, Command, Element, Length, Settings, Theme, Size,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command as SysCommand;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--rofi" => {
                rofi_mode();
                std::process::exit(0);
            }
            "--add" => {
                if args.len() >= 4 {
                    add_app_cli(&args[2], &args[3]);
                } else {
                    println!("Usage: app_launcher --add <name> <path>");
                }
                std::process::exit(0);
            }
            "--list" => {
                list_apps();
                std::process::exit(0);
            }
            "--remove" => {
                if args.len() >= 3 {
                    remove_app_cli(&args[2]);
                } else {
                    println!("Usage: app_launcher --remove <name>");
                }
                std::process::exit(0);
            }
            "-l" | "--launch" => {
                if args.len() >= 3 {
                    launch_app_cli(&args[2]);
                } else {
                    println!("Usage: app_launcher -l <name>");
                }
                std::process::exit(0);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                println!("Unknown option: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    AppLauncher::run(Settings {
        window: iced::window::Settings {
            size: Size::new(700.0, 500.0),
                     ..Default::default()
        },
        ..Default::default()
    })
}

fn rofi_mode() {
    let config_path = dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("app_launcher")
    .join("config.json");

    if let Ok(data) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
            for app in &config.apps {
                println!("{}", app.name);
            }
            return;
        }
    }
    eprintln!("Error: Could not load apps configuration");
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("app_launcher")
    .join("config.json")
}

fn load_config_cli() -> AppConfig {
    let config_path = get_config_path();
    if let Ok(data) = fs::read_to_string(&config_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_config_cli(config: &AppConfig) {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(&config_path, json);
    }
}

fn add_app_cli(name: &str, path: &str) {
    let mut config = load_config_cli();
    config.apps.push(App {
        name: name.to_string(),
                     path: path.to_string(),
    });
    save_config_cli(&config);
    println!("✓ Added '{}' -> '{}'", name, path);
}

fn list_apps() {
    let config = load_config_cli();
    if config.apps.is_empty() {
        println!("No apps configured.");
    } else {
        println!("Configured apps:");
        for (i, app) in config.apps.iter().enumerate() {
            println!("  [{}] {} -> {}", i, app.name, app.path);
        }
    }
}

fn remove_app_cli(name: &str) {
    let mut config = load_config_cli();
    let original_len = config.apps.len();
    config.apps.retain(|app| app.name != name);

    if config.apps.len() < original_len {
        save_config_cli(&config);
        println!("✓ Removed '{}'", name);
    } else {
        println!("✗ App '{}' not found", name);
    }
}

fn launch_app_cli(name: &str) {
    let config = load_config_cli();

    if let Some(app) = config.apps.iter().find(|a| a.name == name) {
        println!("Launching '{}'...", app.name);

        // Use the same launch logic as GUI
        let needs_terminal = needs_terminal_cli(&app.path);

        let result = if cfg!(target_os = "windows") {
            if needs_terminal {
                SysCommand::new("cmd").args(&["/K", &app.path]).spawn()
            } else {
                SysCommand::new("cmd").args(&["/C", &app.path]).spawn()
            }
        } else {
            if needs_terminal {
                try_terminal_launch_cli(&app.path)
            } else {
                SysCommand::new("sh").args(&["-c", &app.path]).spawn()
            }
        };

        match result {
            Ok(_) => println!("✓ Launched successfully"),
            Err(e) => eprintln!("✗ Failed to launch: {}", e),
        }
    } else {
        eprintln!("✗ App '{}' not found", name);
        eprintln!("Run 'app_launcher --list' to see available apps");
    }
}

fn needs_terminal_cli(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    let cli_programs = [
        "stress", "htop", "top", "vim", "nvim", "nano", "emacs",
        "python", "node", "ruby", "bash", "zsh", "fish",
        "npm", "cargo", "git", "ssh", "tmux", "screen"
    ];

    for prog in &cli_programs {
        if cmd_lower.starts_with(prog) || cmd_lower.contains(&format!("/{}", prog)) {
            return true;
        }
    }
    !cmd.trim().ends_with('&')
}

fn try_terminal_launch_cli(cmd: &str) -> std::io::Result<std::process::Child> {
    let xfce_cmd = format!("bash -c '{}'", cmd);
    let term_cmd = format!("bash -c '{}'", cmd);

    let terminals: Vec<(&str, Vec<&str>)> = vec![
        ("gnome-terminal", vec!["--", "bash", "-c", cmd]),
        ("konsole", vec!["-e", "bash", "-c", cmd]),
        ("xfce4-terminal", vec!["-e", xfce_cmd.as_str()]),
        ("alacritty", vec!["-e", "bash", "-c", cmd]),
        ("kitty", vec!["bash", "-c", cmd]),
        ("xterm", vec!["-e", "bash", "-c", cmd]),
        ("terminator", vec!["-e", term_cmd.as_str()]),
    ];

    for (term, args) in &terminals {
        if let Ok(child) = SysCommand::new(term).args(args).spawn() {
            return Ok(child);
        }
    }
    SysCommand::new("sh").args(&["-c", cmd]).spawn()
}

fn print_help() {
    println!("App Launcher - Catppuccin themed application launcher\n");
    println!("USAGE:");
    println!("  app_launcher                    Launch GUI");
    println!("  app_launcher --rofi             Output app names for rofi");
    println!("  app_launcher --add <name> <path> Add an app");
    println!("  app_launcher --list             List all apps");
    println!("  app_launcher -l <name>          Launch app by name");
    println!("  app_launcher --remove <name>    Remove an app by name");
    println!("  app_launcher --help             Show this help\n");
    println!("EXAMPLES:");
    println!("  app_launcher --add Firefox firefox");
    println!("  app_launcher --add \"VS Code\" code");
    println!("  app_launcher -l Firefox");
    println!("  app_launcher --remove Firefox");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct App {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    apps: Vec<App>,
    close_on_launch: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            close_on_launch: true,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    PathChanged(String),
    AddApp,
    LaunchApp(usize),
    DeleteApp(usize),
    ToggleCloseOnLaunch(bool),
}

struct AppLauncher {
    config: AppConfig,
    name_input: String,
    path_input: String,
    config_path: PathBuf,
    should_exit: bool,
}

impl Application for AppLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("app_launcher")
        .join("config.json");

        let config = Self::load_config(&config_path);

        (
            Self {
                config,
                name_input: String::new(),
         path_input: String::new(),
         config_path,
         should_exit: false,
            },
         Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("App Launcher")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NameChanged(value) => {
                self.name_input = value;
            }
            Message::PathChanged(value) => {
                self.path_input = value;
            }
            Message::AddApp => {
                if !self.name_input.is_empty() && !self.path_input.is_empty() {
                    self.config.apps.push(App {
                        name: self.name_input.clone(),
                                          path: self.path_input.clone(),
                    });
                    self.name_input.clear();
                    self.path_input.clear();
                    self.save_config();
                }
            }
            Message::LaunchApp(idx) => {
                if let Some(app) = self.config.apps.get(idx) {
                    Self::launch_command(&app.path);

                    if self.config.close_on_launch {
                        return iced::window::close(iced::window::Id::MAIN);
                    }
                }
            }
            Message::DeleteApp(idx) => {
                self.config.apps.remove(idx);
                self.save_config();
            }
            Message::ToggleCloseOnLaunch(value) => {
                self.config.close_on_launch = value;
                self.save_config();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text("App Launcher")
        .size(32);

        let name_input = text_input("App Name", &self.name_input)
        .on_input(Message::NameChanged)
        .padding(10);

        let path_input = text_input("Path/Command", &self.path_input)
        .on_input(Message::PathChanged)
        .padding(10);

        let add_button = button(text("Add App").size(16))
        .on_press(Message::AddApp)
        .padding([10, 20]);

        let close_on_launch_toggle = checkbox(
            "Close launcher after opening app",
            self.config.close_on_launch,
        )
        .on_toggle(Message::ToggleCloseOnLaunch);

        let input_section = column![
            row![name_input, path_input].spacing(10),
            row![add_button, close_on_launch_toggle]
            .spacing(20)
            .align_items(iced::Alignment::Center),
        ]
        .spacing(10);

        let apps_list: Element<_> = if self.config.apps.is_empty() {
            container(
                text("No apps added yet. Add one above!")
                .size(16),
            )
            .padding(20)
            .into()
        } else {
            let items = self.config.apps.iter().enumerate().fold(
                Column::new().spacing(8),
                                                                 |col, (idx, app)| {
                                                                     let launch_btn = button(text("Launch").size(14))
                                                                     .on_press(Message::LaunchApp(idx))
                                                                     .padding([8, 16]);

                                                                     let delete_btn = button(text("✕").size(14))
                                                                     .on_press(Message::DeleteApp(idx))
                                                                     .padding([8, 12]);

                                                                     let app_row = container(
                                                                         row![
                                                                             column![
                                                                                 text(&app.name).size(18),
                                                                                             text(&app.path).size(12),
                                                                             ]
                                                                             .width(Length::Fill),
                                                                                             launch_btn,
                                                                                             delete_btn
                                                                         ]
                                                                         .spacing(10)
                                                                         .align_items(iced::Alignment::Center),
                                                                     )
                                                                     .padding(12);

                                                                     col.push(app_row)
                                                                 },
            );

            scrollable(items).into()
        };

        let content = column![title, input_section, apps_list]
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinMocha
    }
}

impl AppLauncher {
    fn load_config(path: &PathBuf) -> AppConfig {
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    }

    fn save_config(&self) {
        if let Some(parent) = self.config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write(&self.config_path, json);
        }
    }

    fn launch_command(cmd: &str) {
        // Detect if this needs a terminal window
        let needs_terminal = Self::needs_terminal(cmd);

        let result = if cfg!(target_os = "windows") {
            if needs_terminal {
                SysCommand::new("cmd")
                .args(&["/K", cmd])
                .spawn()
            } else {
                SysCommand::new("cmd")
                .args(&["/C", cmd])
                .spawn()
            }
        } else {
            if needs_terminal {
                // Try common terminal emulators
                Self::try_terminal_launch(cmd)
            } else {
                SysCommand::new("sh")
                .args(&["-c", cmd])
                .spawn()
            }
        };

        let _ = result;
    }

    fn needs_terminal(cmd: &str) -> bool {
        // Check if command is a terminal-only program or doesn't end with &
        let cmd_lower = cmd.to_lowercase();

        // Common CLI-only programs
        let cli_programs = [
            "stress", "htop", "top", "vim", "nvim", "nano", "emacs",
            "python", "node", "ruby", "bash", "zsh", "fish",
            "npm", "cargo", "git", "ssh", "tmux", "screen"
        ];

        // Check if command starts with any CLI program
        for prog in &cli_programs {
            if cmd_lower.starts_with(prog) || cmd_lower.contains(&format!("/{}", prog)) {
                return true;
            }
        }

        // If command doesn't background itself with &, might need terminal
        !cmd.trim().ends_with('&')
    }

    fn try_terminal_launch(cmd: &str) -> std::io::Result<std::process::Child> {
        // Try different terminal emulators in order of preference
        let xfce_cmd = format!("bash -c '{}'", cmd);
        let term_cmd = format!("bash -c '{}'", cmd);

        let terminals: Vec<(&str, Vec<&str>)> = vec![
            ("gnome-terminal", vec!["--", "bash", "-c", cmd]),
            ("konsole", vec!["-e", "bash", "-c", cmd]),
            ("xfce4-terminal", vec!["-e", xfce_cmd.as_str()]),
            ("alacritty", vec!["-e", "bash", "-c", cmd]),
            ("kitty", vec!["bash", "-c", cmd]),
            ("xterm", vec!["-e", "bash", "-c", cmd]),
            ("terminator", vec!["-e", term_cmd.as_str()]),
        ];

        for (term, args) in &terminals {
            if let Ok(child) = SysCommand::new(term).args(args).spawn() {
                return Ok(child);
            }
        }

        // Fallback to regular shell execution
        SysCommand::new("sh").args(&["-c", cmd]).spawn()
    }
}
