use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use walkdir::WalkDir;

// ── CLI definitions ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "hashcmp", about = "SHA-256 file comparator — CLI & GUI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,

    /// Launch GUI (default if no subcommand given)
    #[arg(long, global = true)]
    gui: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compare two files
    Cmp {
        file_a: PathBuf,
        file_b: PathBuf,
    },
    /// Watch two files and re-compare on change
    Watch {
        file_a: PathBuf,
        file_b: PathBuf,
        /// Poll interval in ms (default 500)
        #[arg(long, default_value = "500")]
        interval: u64,
    },
    /// Compare all files in two directories by name
    Dir {
        dir_a: PathBuf,
        dir_b: PathBuf,
    },
}

// ── Hashing ──────────────────────────────────────────────────────────────────

fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn compare(a: &Path, b: &Path) -> io::Result<(String, String, bool)> {
    let ha = hash_file(a)?;
    let hb = hash_file(b)?;
    let matched = ha == hb;
    Ok((ha, hb, matched))
}

// ── CLI modes ────────────────────────────────────────────────────────────────

fn cli_cmp(a: &Path, b: &Path) -> i32 {
    match compare(a, b) {
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
        Ok((ha, hb, matched)) => {
            println!("A  {}  {}", ha, a.display());
            println!("B  {}  {}", hb, b.display());
            if matched {
                println!("\x1b[32m✓ MATCH\x1b[0m");
                0
            } else {
                println!("\x1b[31m✗ MISMATCH\x1b[0m");
                1
            }
        }
    }
}

fn cli_watch(a: &Path, b: &Path, interval_ms: u64) {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    println!("watching {} ↔ {} (Ctrl-C to stop)", a.display(), b.display());
    println!();

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        tx,
        Config::default().with_poll_interval(Duration::from_millis(interval_ms)),
    )
    .expect("watcher init failed");

    watcher.watch(a, RecursiveMode::NonRecursive).ok();
    watcher.watch(b, RecursiveMode::NonRecursive).ok();

    // initial check
    cli_cmp(a, b);

    loop {
        if rx.recv_timeout(Duration::from_millis(interval_ms * 2)).is_ok() {
            println!("\n--- change detected ---");
            cli_cmp(a, b);
        }
    }
}

fn cli_dir(da: &Path, db: &Path) {
    let files_a: Vec<PathBuf> = WalkDir::new(da)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    if files_a.is_empty() {
        println!("no files in {}", da.display());
        return;
    }

    let mut matched = 0;
    let mut mismatched = 0;
    let mut missing = 0;

    for path_a in &files_a {
        let rel = path_a.strip_prefix(da).unwrap();
        let path_b = db.join(rel);

        if !path_b.exists() {
            println!("\x1b[33m? MISSING\x1b[0m  {}", rel.display());
            missing += 1;
            continue;
        }

        match compare(path_a, &path_b) {
            Err(e) => eprintln!("error reading {}: {e}", rel.display()),
            Ok((_, _, true)) => {
                println!("\x1b[32m✓ MATCH   \x1b[0m  {}", rel.display());
                matched += 1;
            }
            Ok((ha, hb, false)) => {
                println!("\x1b[31m✗ MISMATCH\x1b[0m  {}", rel.display());
                println!("  A: {ha}");
                println!("  B: {hb}");
                mismatched += 1;
            }
        }
    }

    println!();
    println!(
        "total: {} files — \x1b[32m{} match\x1b[0m  \x1b[31m{} mismatch\x1b[0m  \x1b[33m{} missing\x1b[0m",
        files_a.len(), matched, mismatched, missing
    );

    if mismatched > 0 || missing > 0 {
        std::process::exit(1);
    }
}

// ── GUI ──────────────────────────────────────────────────────────────────────

#[derive(Default)]
enum CompareResult {
    #[default]
    None,
    Match,
    Mismatch,
    Error(String),
}

struct HashCmpApp {
    // file compare tab
    path_a: String,
    path_b: String,
    hash_a: String,
    hash_b: String,
    cmp_result: CompareResult,

    // dir compare tab
    dir_a: String,
    dir_b: String,
    dir_results: Vec<DirEntry>,
    dir_summary: String,

    // watch tab
    watch_a: String,
    watch_b: String,
    watch_result: CompareResult,
    watch_hash_a: String,
    watch_hash_b: String,
    watch_active: bool,
    watch_state: Arc<Mutex<WatchState>>,
    watch_handle: Option<std::thread::JoinHandle<()>>,

    active_tab: Tab,
}

#[derive(PartialEq)]
enum Tab {
    Compare,
    Dir,
    Watch,
}

#[derive(Clone)]
struct DirEntry {
    name: String,
    status: EntryStatus,
    hash_a: String,
    hash_b: String,
}

#[derive(Clone, PartialEq)]
enum EntryStatus {
    Match,
    Mismatch,
    Missing,
}

#[derive(Default)]
struct WatchState {
    hash_a: String,
    hash_b: String,
    matched: Option<bool>,
    dirty: bool,
}

impl Default for HashCmpApp {
    fn default() -> Self {
        Self {
            path_a: String::new(),
            path_b: String::new(),
            hash_a: String::new(),
            hash_b: String::new(),
            cmp_result: CompareResult::None,
            dir_a: String::new(),
            dir_b: String::new(),
            dir_results: vec![],
            dir_summary: String::new(),
            watch_a: String::new(),
            watch_b: String::new(),
            watch_result: CompareResult::None,
            watch_hash_a: String::new(),
            watch_hash_b: String::new(),
            watch_active: false,
            watch_state: Arc::new(Mutex::new(WatchState::default())),
            watch_handle: None,
            active_tab: Tab::Compare,
        }
    }
}

impl HashCmpApp {
    fn do_compare(&mut self) {
        let a = PathBuf::from(&self.path_a);
        let b = PathBuf::from(&self.path_b);
        match compare(&a, &b) {
            Err(e) => {
                self.cmp_result = CompareResult::Error(e.to_string());
            }
            Ok((ha, hb, matched)) => {
                self.hash_a = ha;
                self.hash_b = hb;
                self.cmp_result = if matched {
                    CompareResult::Match
                } else {
                    CompareResult::Mismatch
                };
            }
        }
    }

    fn do_dir_compare(&mut self) {
        let da = PathBuf::from(&self.dir_a);
        let db = PathBuf::from(&self.dir_b);
        self.dir_results.clear();

        let files: Vec<PathBuf> = WalkDir::new(&da)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();

        let mut matched = 0;
        let mut mismatched = 0;
        let mut missing = 0;

        for path_a in &files {
            let rel = path_a.strip_prefix(&da).unwrap();
            let path_b = db.join(rel);
            let name = rel.display().to_string();

            if !path_b.exists() {
                self.dir_results.push(DirEntry {
                    name,
                    status: EntryStatus::Missing,
                    hash_a: String::new(),
                    hash_b: String::new(),
                });
                missing += 1;
                continue;
            }

            match compare(path_a, &path_b) {
                Err(e) => {
                    self.dir_results.push(DirEntry {
                        name,
                        status: EntryStatus::Missing,
                        hash_a: format!("error: {e}"),
                        hash_b: String::new(),
                    });
                }
                Ok((ha, hb, m)) => {
                    if m {
                        matched += 1;
                    } else {
                        mismatched += 1;
                    }
                    self.dir_results.push(DirEntry {
                        name,
                        status: if m {
                            EntryStatus::Match
                        } else {
                            EntryStatus::Mismatch
                        },
                        hash_a: ha,
                        hash_b: hb,
                    });
                }
            }
        }

        self.dir_summary = format!(
            "{} files — {} match  {} mismatch  {} missing",
            files.len(),
            matched,
            mismatched,
            missing
        );
    }

    fn start_watch(&mut self, ctx: egui::Context) {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc;

        self.watch_active = true;
        let state = Arc::clone(&self.watch_state);
        let a = PathBuf::from(&self.watch_a);
        let b = PathBuf::from(&self.watch_b);

        self.watch_handle = Some(std::thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let mut watcher = match RecommendedWatcher::new(
                tx,
                Config::default().with_poll_interval(Duration::from_millis(400)),
            ) {
                Ok(w) => w,
                Err(e) => {
                    let mut s = state.lock().unwrap();
                    s.hash_a = format!("watcher error: {e}");
                    s.dirty = true;
                    return;
                }
            };
            watcher.watch(&a, RecursiveMode::NonRecursive).ok();
            watcher.watch(&b, RecursiveMode::NonRecursive).ok();

            // initial
            Self::update_watch_state(&state, &a, &b);
            ctx.request_repaint();

            loop {
                match rx.recv_timeout(Duration::from_millis(600)) {
                    Ok(_) => {
                        std::thread::sleep(Duration::from_millis(100)); // debounce
                        Self::update_watch_state(&state, &a, &b);
                        ctx.request_repaint();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(_) => {
                        // check if we should stop
                        let s = state.lock().unwrap();
                        if s.hash_a == "__STOP__" {
                            break;
                        }
                    }
                }
            }
        }));
    }

    fn update_watch_state(state: &Arc<Mutex<WatchState>>, a: &Path, b: &Path) {
        let mut s = state.lock().unwrap();
        match compare(a, b) {
            Ok((ha, hb, m)) => {
                s.hash_a = ha;
                s.hash_b = hb;
                s.matched = Some(m);
            }
            Err(e) => {
                s.hash_a = format!("error: {e}");
                s.hash_b = String::new();
                s.matched = None;
            }
        }
        s.dirty = true;
    }

    fn stop_watch(&mut self) {
        self.watch_active = false;
        {
            let mut s = self.watch_state.lock().unwrap();
            s.hash_a = "__STOP__".to_string();
        }
        if let Some(h) = self.watch_handle.take() {
            let _ = h.join();
        }
        // reset state for next run
        self.watch_state = Arc::new(Mutex::new(WatchState::default()));
    }

    fn poll_watch(&mut self) {
        let mut s = self.watch_state.lock().unwrap();
        if s.dirty {
            s.dirty = false;
            self.watch_hash_a = s.hash_a.clone();
            self.watch_hash_b = s.hash_b.clone();
            self.watch_result = match s.matched {
                Some(true) => CompareResult::Match,
                Some(false) => CompareResult::Mismatch,
                None => CompareResult::Error(s.hash_a.clone()),
            };
        }
    }
}

const GREEN: egui::Color32 = egui::Color32::from_rgb(0x4c, 0xaf, 0x50);
const RED: egui::Color32 = egui::Color32::from_rgb(0xe5, 0x39, 0x35);
const YELLOW: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb3, 0x00);
const DIM: egui::Color32 = egui::Color32::from_rgb(0x88, 0x88, 0x88);

fn result_label(ui: &mut egui::Ui, result: &CompareResult) {
    match result {
        CompareResult::None => {}
        CompareResult::Match => {
            ui.colored_label(GREEN, "✓  MATCH");
        }
        CompareResult::Mismatch => {
            ui.colored_label(RED, "✗  MISMATCH");
        }
        CompareResult::Error(e) => {
            ui.colored_label(RED, format!("error: {e}"));
        }
    }
}

fn hash_display(ui: &mut egui::Ui, label: &str, hash: &str) {
    if hash.is_empty() {
        return;
    }
    ui.horizontal(|ui| {
        ui.colored_label(DIM, label);
        ui.monospace(hash);
    });
}

impl eframe::App for HashCmpApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.watch_active {
            self.poll_watch();
            ui.ctx().request_repaint_after(Duration::from_millis(200));
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("hashcmp");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == Tab::Compare, "Compare").clicked() {
                    self.active_tab = Tab::Compare;
                }
                if ui.selectable_label(self.active_tab == Tab::Dir, "Directory").clicked() {
                    self.active_tab = Tab::Dir;
                }
                if ui.selectable_label(self.active_tab == Tab::Watch, "Watch").clicked() {
                    self.active_tab = Tab::Watch;
                }
            });

            ui.separator();

            let ctx = ui.ctx().clone();
            match self.active_tab {
                Tab::Compare => self.tab_compare(ui),
                Tab::Dir => self.tab_dir(ui),
                Tab::Watch => self.tab_watch(ui, ctx),
            }
        });
    }
}

impl HashCmpApp {
    fn tab_compare(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("cmp_grid")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("File A");
                ui.text_edit_singleline(&mut self.path_a);
                ui.end_row();

                ui.label("File B");
                ui.text_edit_singleline(&mut self.path_b);
                ui.end_row();
            });

        ui.add_space(6.0);

        if ui.button("Compare").clicked() {
            self.do_compare();
        }

        ui.add_space(8.0);
        result_label(ui, &self.cmp_result);
        hash_display(ui, "A: ", &self.hash_a);
        hash_display(ui, "B: ", &self.hash_b);
    }

    fn tab_dir(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("dir_grid")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("Dir A");
                ui.text_edit_singleline(&mut self.dir_a);
                ui.end_row();

                ui.label("Dir B");
                ui.text_edit_singleline(&mut self.dir_b);
                ui.end_row();
            });

        ui.add_space(6.0);

        if ui.button("Compare Dirs").clicked() {
            self.do_dir_compare();
        }

        if !self.dir_summary.is_empty() {
            ui.add_space(8.0);
            ui.label(&self.dir_summary);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in &self.dir_results {
                    ui.horizontal(|ui| {
                        match entry.status {
                            EntryStatus::Match => {
                                ui.colored_label(GREEN, "✓");
                            }
                            EntryStatus::Mismatch => {
                                ui.colored_label(RED, "✗");
                            }
                            EntryStatus::Missing => {
                                ui.colored_label(YELLOW, "?");
                            }
                        }
                        ui.monospace(&entry.name);
                    });

                    if entry.status == EntryStatus::Mismatch {
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            ui.colored_label(DIM, "A:");
                            ui.monospace(&entry.hash_a[..16]);
                            ui.colored_label(DIM, "…  B:");
                            ui.monospace(&entry.hash_b[..16]);
                            ui.colored_label(DIM, "…");
                        });
                    }
                }
            });
        }
    }

    fn tab_watch(&mut self, ui: &mut egui::Ui, ctx: egui::Context) {
        egui::Grid::new("watch_grid")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("File A");
                ui.add_enabled(!self.watch_active, egui::TextEdit::singleline(&mut self.watch_a));
                ui.end_row();

                ui.label("File B");
                ui.add_enabled(!self.watch_active, egui::TextEdit::singleline(&mut self.watch_b));
                ui.end_row();
            });

        ui.add_space(6.0);

        if self.watch_active {
            if ui.button("Stop Watching").clicked() {
                self.stop_watch();
                self.watch_result = CompareResult::None;
                self.watch_hash_a.clear();
                self.watch_hash_b.clear();
            }
            ui.add_space(4.0);
            ui.colored_label(DIM, "● watching for changes…");
        } else {
            if ui.button("Start Watching").clicked() {
                self.start_watch(ctx);
            }
        }

        ui.add_space(8.0);
        result_label(ui, &self.watch_result);
        hash_display(ui, "A: ", &self.watch_hash_a);
        hash_display(ui, "B: ", &self.watch_hash_b);
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Cmd::Cmp { file_a, file_b }) => {
            let code = cli_cmp(&file_a, &file_b);
            std::process::exit(code);
        }
        Some(Cmd::Watch { file_a, file_b, interval }) => {
            cli_watch(&file_a, &file_b, interval);
        }
        Some(Cmd::Dir { dir_a, dir_b }) => {
            cli_dir(&dir_a, &dir_b);
        }
        None => {
            // GUI
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_title("hashcmp")
                    .with_inner_size([540.0, 400.0]),
                ..Default::default()
            };
            eframe::run_native(
                "hashcmp",
                options,
                Box::new(|_cc| Ok(Box::new(HashCmpApp::default()))),
            )
            .expect("eframe failed");
        }
    }
}
