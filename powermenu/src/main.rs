use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, CssProvider, Label, Orientation};
use libadwaita as adw;
use std::process::Command;

const APP_ID: &str = "com.omarchy.powermenu";

// Catppuccin Mocha colors
const BASE: &str = "#1e1e2e";
const SURFACE0: &str = "#313244";
const SURFACE1: &str = "#45475a";
const TEXT: &str = "#cdd6f4";
const LAVENDER: &str = "#b4befe";
const BLUE: &str = "#89b4fa";
const RED: &str = "#f38ba8";

struct PowerOption {
    icon: &'static str,
    label: &'static str,
    command: &'static str,
    args: Vec<&'static str>,
}

impl PowerOption {
    fn new(
        icon: &'static str,
        label: &'static str,
        command: &'static str,
        args: Vec<&'static str>,
    ) -> Self {
        Self {
            icon,
            label,
            command,
            args,
        }
    }
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
        load_css();
    });

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Power Menu")
        .default_width(340)
        .default_height(480)
        .resizable(false)
        .build();

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // Title
    let title = Label::builder()
        .label("Power Options")
        .css_classes(vec!["title".to_string()])
        .build();
    main_box.append(&title);

    // Power options
    let options = vec![
        PowerOption::new("⏻", "Shutdown", "systemctl", vec!["poweroff"]),
        PowerOption::new("↻", "Reboot", "systemctl", vec!["reboot"]),
        PowerOption::new("⏾", "Suspend", "systemctl", vec!["suspend"]),
        PowerOption::new("☾", "Hibernate", "systemctl", vec!["hibernate"]),
        PowerOption::new("→", "Logout", "niri", vec!["msg", "action", "quit"]),
        PowerOption::new("🔒", "Lock", "loginctl", vec!["lock-session"]),
        PowerOption::new(
            "⚙",
            "UEFI Settings",
            "systemctl",
            vec!["reboot", "--firmware-setup"],
        ),
    ];

    for option in options {
        let button = create_power_button(&option);
        main_box.append(&button);
    }

    // Cancel button
    let cancel_btn = Button::builder()
        .label("Cancel")
        .css_classes(vec!["cancel-button".to_string()])
        .build();

    cancel_btn.connect_clicked(move |_| {
        std::process::exit(0);
    });

    main_box.append(&cancel_btn);
    window.set_child(Some(&main_box));
    window.present();
}

fn create_power_button(option: &PowerOption) -> Button {
    let button = Button::builder()
        .css_classes(vec!["power-button".to_string()])
        .build();

    let btn_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Start)
        .build();

    let icon = Label::builder()
        .label(option.icon)
        .css_classes(vec!["power-icon".to_string()])
        .build();

    let label = Label::builder()
        .label(option.label)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    btn_box.append(&icon);
    btn_box.append(&label);
    button.set_child(Some(&btn_box));

    let cmd = option.command.to_string();
    let args: Vec<String> = option.args.iter().map(|s| s.to_string()).collect();

    button.connect_clicked(move |_| {
        let _ = Command::new(&cmd).args(&args).spawn();
        std::process::exit(0);
    });

    button
}

fn load_css() {
    let provider = CssProvider::new();
    let css = format!(
        r#"
        window {{
        background-color: {BASE};
}}

.title {{
color: {LAVENDER};
font-size: 22px;
font-weight: bold;
margin-bottom: 12px;
}}

.power-button {{
background-color: {SURFACE0};
color: {TEXT};
border: 2px solid {SURFACE1};
border-radius: 10px;
padding: 14px;
min-height: 52px;
}}

.power-button:hover {{
background-color: {SURFACE1};
border-color: {BLUE};
transition: all 200ms ease;
}}

.power-icon {{
font-size: 20px;
color: {BLUE};
min-width: 32px;
}}

.cancel-button {{
background-color: {RED};
color: {BASE};
border-radius: 10px;
padding: 14px;
margin-top: 8px;
font-weight: bold;
}}

.cancel-button:hover {{
background-color: {TEXT};
transition: all 200ms ease;
}}
"#
    );

    provider.load_from_data(&css);

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
