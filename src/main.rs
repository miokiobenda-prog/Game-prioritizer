use std::{fs, path::{Path, PathBuf}, process::Command};

use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Game {
    name: String,
    executable: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct Library {
    games: Vec<Game>,
}

struct BoosterApp {
    library: Library,
    status: String,
}

impl BoosterApp {
    fn config_path() -> PathBuf {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RustGameBooster")
            .join("library.json")
    }

    fn load() -> Self {
        let library = fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self { library, status: "Ready. Add a game to begin.".into() }
    }

    fn save(&mut self) {
        let path = Self::config_path();
        let result = (|| -> Result<(), String> {
            let parent = path.parent().ok_or("Invalid settings path")?;
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            let data = serde_json::to_string_pretty(&self.library).map_err(|e| e.to_string())?;
            fs::write(path, data).map_err(|e| e.to_string())
        })();
        self.status = match result {
            Ok(()) => "Library saved.".into(),
            Err(error) => format!("Could not save library: {error}"),
        };
    }

    fn add_game(&mut self) {
        let selected = rfd::FileDialog::new()
            .add_filter("Windows programs", &["exe"])
            .set_title("Select a game executable")
            .pick_file();
        if let Some(executable) = selected {
            let name = executable.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Game")
                .to_owned();
            if self.library.games.iter().any(|g| g.executable == executable) {
                self.status = "That game is already in your library.".into();
                return;
            }
            self.library.games.push(Game { name, executable });
            self.save();
        }
    }

    fn launch(&mut self, game: &Game) {
        if !game.executable.is_file() {
            self.status = format!("Missing executable: {}", game.executable.display());
            return;
        }
        let directory = game.executable.parent().unwrap_or(Path::new("."));
        let mut command = Command::new(&game.executable);
        command.current_dir(directory);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // HIGH_PRIORITY_CLASS affects only the child process and ends when it exits.
            command.creation_flags(0x0000_0080);
        }
        self.status = match command.spawn() {
            Ok(child) => format!("Boosted launch: {} (PID {}).", game.name, child.id()),
            Err(error) => format!("Could not launch {}: {error}", game.name),
        };
    }
}

impl eframe::App for BoosterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Game Booster");
            ui.label("Launch games at high process priority. Nothing is changed system-wide.");
            ui.add_space(10.0);
            if ui.button("Add game executable").clicked() {
                self.add_game();
            }
            ui.separator();

            let mut remove = None;
            let games = self.library.games.clone();
            for (index, game) in games.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&game.name);
                            ui.small(game.executable.display().to_string());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Remove").clicked() { remove = Some(index); }
                            if ui.button("Boost & launch").clicked() { self.launch(game); }
                        });
                    });
                });
                ui.add_space(4.0);
            }
            if let Some(index) = remove {
                self.library.games.remove(index);
                self.save();
            }
            if self.library.games.is_empty() {
                ui.label("Your game library is empty.");
            }
            ui.separator();
            ui.label(&self.status);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([680.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native("Rust Game Booster", options, Box::new(|_| Ok(Box::new(BoosterApp::load()))))
}
