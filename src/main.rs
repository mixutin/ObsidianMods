mod manager;
mod model;

use eframe::egui;
use manager::ModManager;
use model::{CatalogMod, InstalledMod, ModKind};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Discover,
    Installed,
}

enum Action {
    Toggle(usize, bool),
    Uninstall(usize),
    Install(PathBuf),
    InstallCatalog(usize),
    Refresh,
    RefreshCatalog,
    Launch,
    OpenGame,
    OpenMods,
}

struct ObsidianApp {
    manager: Option<ModManager>,
    mods: Vec<InstalledMod>,
    catalog: Vec<CatalogMod>,
    view: View,
    status: String,
    pending_remove: Option<usize>,
}

impl ObsidianApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(13, 13, 17);
        visuals.window_fill = egui::Color32::from_rgb(18, 18, 24);
        cc.egui_ctx.set_visuals(visuals);
        let mut app = Self {
            manager: None,
            mods: Vec::new(),
            catalog: Vec::new(),
            view: View::Discover,
            status: "Detecting Minecraft Dungeons II…".into(),
            pending_remove: None,
        };
        match ModManager::detect() {
            Ok(manager) => {
                app.manager = Some(manager);
                app.refresh();
                app.refresh_catalog();
            }
            Err(err) => app.status = format!("Game detection failed: {err}"),
        }
        app
    }

    fn refresh(&mut self) {
        let Some(manager) = &self.manager else { return };
        match manager.scan() {
            Ok(mods) => {
                self.status = format!(
                    "Ready · {} managed mod{}",
                    mods.len(),
                    if mods.len() == 1 { "" } else { "s" }
                );
                self.mods = mods;
            }
            Err(err) => self.status = format!("Refresh failed: {err}"),
        }
    }

    fn refresh_catalog(&mut self) {
        let Some(manager) = &self.manager else { return };
        match manager.fetch_catalog() {
            Ok(catalog) => {
                self.catalog = catalog.mods;
                self.status = format!("Discover ready · {} catalog mods", self.catalog.len());
            }
            Err(err) => {
                self.status = format!("Catalog unavailable: {err}");
            }
        }
    }

    fn install(&mut self, path: &std::path::Path) {
        let Some(manager) = &self.manager else { return };
        match manager.install_archive(path) {
            Ok(items) => {
                self.status = format!("Installed {}", items.join(", "));
                self.refresh();
            }
            Err(err) => self.status = format!("Install failed: {err}"),
        }
    }

    fn install_catalog(&mut self, index: usize) {
        let Some(manager) = &self.manager else { return };
        let Some(item) = self.catalog.get(index).cloned() else {
            return;
        };
        self.status = format!("Downloading {}…", item.name);
        match manager.install_catalog_mod(&item) {
            Ok(items) => {
                self.status = format!("Installed {}", items.join(", "));
                self.refresh();
            }
            Err(err) => self.status = format!("Install failed: {err}"),
        }
    }

    fn perform(&mut self, action: Action) {
        match action {
            Action::Refresh => self.refresh(),
            Action::RefreshCatalog => self.refresh_catalog(),
            Action::Install(path) => self.install(&path),
            Action::InstallCatalog(index) => self.install_catalog(index),
            Action::Launch => {
                if let Some(m) = &self.manager {
                    if let Err(e) = m.launch_game() {
                        self.status = format!("Could not launch game: {e}");
                    }
                }
            }
            Action::OpenGame => {
                if let Some(m) = &self.manager {
                    if let Err(e) = m.open_game_folder() {
                        self.status = format!("Could not open folder: {e}");
                    }
                }
            }
            Action::OpenMods => {
                if let Some(m) = &self.manager {
                    if let Err(e) = m.open_mods_folder() {
                        self.status = format!("Could not open mods folder: {e}");
                    }
                }
            }
            Action::Toggle(index, enabled) => {
                let result = self.manager.as_ref().and_then(|m| {
                    self.mods
                        .get(index)
                        .map(|item| m.set_enabled(item, enabled))
                });
                match result {
                    Some(Ok(())) => {
                        self.status = if enabled {
                            "Mod enabled".into()
                        } else {
                            "Mod disabled".into()
                        };
                        self.refresh();
                    }
                    Some(Err(e)) => self.status = format!("Could not change mod state: {e}"),
                    None => {}
                }
            }
            Action::Uninstall(index) => {
                let result = self
                    .manager
                    .as_ref()
                    .and_then(|m| self.mods.get(index).map(|item| m.uninstall(item)));
                match result {
                    Some(Ok(())) => {
                        self.status = "Mod removed · backup created".into();
                        self.pending_remove = None;
                        self.refresh();
                    }
                    Some(Err(e)) => self.status = format!("Could not remove mod: {e}"),
                    None => {}
                }
            }
        }
    }
}

impl eframe::App for ObsidianApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        for path in dropped {
            self.install(&path);
        }
        let mut action = None;

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("OBSIDIAN").size(24.0).strong());
                ui.label(
                    egui::RichText::new("MODS")
                        .size(24.0)
                        .color(egui::Color32::from_rgb(157, 120, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(egui::RichText::new("Manager · Minecraft Dungeons II").weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("▶ Launch").clicked() {
                        action = Some(Action::Launch);
                    }
                    if ui.button("↻ Refresh").clicked() {
                        action = Some(Action::Refresh);
                    }
                });
            });
            ui.add_space(8.0);
        });

        egui::SidePanel::left("sidebar").resizable(false).default_width(235.0).show(ctx, |ui| {
            ui.add_space(12.0);
            ui.heading("Obsidian");
            ui.add_space(6.0);
            if ui.selectable_label(self.view == View::Discover, "✦ Discover").clicked() {
                self.view = View::Discover;
            }
            if ui.selectable_label(self.view == View::Installed, "☰ Installed").clicked() {
                self.view = View::Installed;
            }
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            ui.heading("Library");
            ui.add_space(8.0);

            if ui.button("＋ Install mod ZIP").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("Mod archive", &["zip"]).pick_file() {
                    action = Some(Action::Install(path));
                }
            }
            if ui.button("📁 Open mod folder").clicked() { action = Some(Action::OpenMods); }
            if ui.button("🎮 Open game folder").clicked() { action = Some(Action::OpenGame); }
            ui.add_space(18.0);
            ui.separator();
            ui.add_space(12.0);

            if let Some(manager) = &self.manager {
                ui.label(egui::RichText::new("Game detected").color(egui::Color32::from_rgb(117, 214, 140)).strong());
                ui.small(manager.paths.game.display().to_string());
                ui.add_space(10.0);
                let ue4ss = manager.ue4ss_installed();
                let color = if ue4ss {
                    egui::Color32::from_rgb(117, 214, 140)
                } else {
                    egui::Color32::from_rgb(235, 178, 82)
                };
                ui.label(egui::RichText::new(if ue4ss { "UE4SS detected" } else { "UE4SS not detected" }).color(color));
            } else {
                ui.colored_label(egui::Color32::from_rgb(235, 105, 105), "Game not detected");
            }

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Drop a .zip anywhere").strong());
            ui.small("Obsidian automatically detects UE4SS Lua mods and Unreal PAK/UTOC/UCAS packages.");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);

            if self.view == View::Discover {
                ui.horizontal(|ui| {
                    ui.heading("Discover");
                    ui.label(
                        egui::RichText::new(format!("{} available", self.catalog.len())).weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("↻ Catalog").clicked() {
                            action = Some(Action::RefreshCatalog);
                        }
                    });
                });
                ui.label(
                    egui::RichText::new(
                        "Verified Obsidian builds. Downloads are SHA-256 checked before install.",
                    )
                    .weak(),
                );
                ui.add_space(10.0);

                if self.catalog.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("Catalog is empty or offline");
                        ui.label("Refresh the catalog when the Obsidian Mods site is reachable.");
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (index, item) in self.catalog.iter().enumerate() {
                                let installed = self.mods.iter().any(|m| {
                                    m.name.eq_ignore_ascii_case(&item.name)
                                        && m.version.as_deref() == Some(item.version.as_str())
                                });
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&item.name)
                                                        .size(18.0)
                                                        .strong(),
                                                );
                                                if item.featured {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(186, 145, 255),
                                                        "FEATURED",
                                                    );
                                                }
                                                ui.colored_label(
                                                    egui::Color32::from_rgb(96, 184, 255),
                                                    item.category.to_uppercase(),
                                                );
                                            });
                                            ui.small(format!(
                                                "v{} · {} · {}",
                                                item.version, item.author, item.loader
                                            ));
                                            ui.label(&item.summary);
                                            if !item.tags.is_empty() {
                                                ui.small(item.tags.join(" · "));
                                            }
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if installed {
                                                    ui.label(
                                                        egui::RichText::new("Installed ✓").color(
                                                            egui::Color32::from_rgb(117, 214, 140),
                                                        ),
                                                    );
                                                } else if ui.button("Install").clicked() {
                                                    action = Some(Action::InstallCatalog(index));
                                                }
                                            },
                                        );
                                    });
                                });
                                ui.add_space(7.0);
                            }
                        });
                }
                return;
            }

            ui.horizontal(|ui| {
                ui.heading("Installed mods");
                ui.label(egui::RichText::new(format!("{} total", self.mods.len())).weak());
            });
            ui.add_space(6.0);

            if self.mods.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.heading("No managed mods found");
                    ui.label("Install a ZIP or drag one into this window.");
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, item) in self.mods.iter().enumerate() {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let mut enabled = item.enabled;
                                    if ui.checkbox(&mut enabled, "").changed() {
                                        action = Some(Action::Toggle(index, enabled));
                                    }
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&item.name).size(17.0).strong(),
                                            );
                                            let color = match item.kind {
                                                ModKind::Ue4ss => {
                                                    egui::Color32::from_rgb(166, 126, 255)
                                                }
                                                ModKind::Pak => {
                                                    egui::Color32::from_rgb(96, 184, 255)
                                                }
                                            };
                                            ui.colored_label(color, item.kind.label());
                                            if let Some(version) = &item.version {
                                                ui.label(
                                                    egui::RichText::new(format!("v{version}"))
                                                        .weak(),
                                                );
                                            }
                                        });
                                        if let Some(author) = &item.author {
                                            ui.small(format!("by {author}"));
                                        }
                                        if let Some(description) = &item.description {
                                            ui.small(description);
                                        } else {
                                            ui.small(if item.enabled {
                                                "Enabled"
                                            } else {
                                                "Disabled"
                                            });
                                        }
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Remove").clicked() {
                                                self.pending_remove = Some(index);
                                            }
                                        },
                                    );
                                });
                            });
                            ui.add_space(7.0);
                        }
                    });
            }
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(157, 120, 255)));
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("Obsidian Mods Manager 0.2.0");
                });
            });
        });

        if let Some(index) = self.pending_remove {
            if let Some(item) = self.mods.get(index) {
                let name = item.name.clone();
                egui::Window::new("Remove mod?")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(format!("Remove {name}?"));
                        ui.label("A backup will be created before anything is deleted.");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.pending_remove = None;
                            }
                            if ui.button("Remove").clicked() {
                                action = Some(Action::Uninstall(index));
                            }
                        });
                    });
            } else {
                self.pending_remove = None;
            }
        }

        if let Some(action) = action {
            self.perform(action);
        }
    }
}

fn register_protocol_handler() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    #[cfg(target_os = "windows")]
    {
        let key = r"HKCU\Software\Classes\obsidianmods";
        let command_key = r"HKCU\Software\Classes\obsidianmods\shell\open\command";
        let command = format!("\"{}\" \"%1\"", exe.display());
        let _ = Command::new("reg.exe")
            .args(["add", key, "/ve", "/d", "URL:Obsidian Mods Protocol", "/f"])
            .status();
        let _ = Command::new("reg.exe")
            .args(["add", key, "/v", "URL Protocol", "/d", "", "/f"])
            .status();
        let _ = Command::new("reg.exe")
            .args(["add", command_key, "/ve", "/d", &command, "/f"])
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data) = dirs::data_local_dir() {
            let apps = data.join("applications");
            let _ = fs::create_dir_all(&apps);
            let desktop = apps.join("obsidian-mods-manager.desktop");
            let exec = exe.display().to_string().replace('"', "\\\"");
            let contents = format!(
                "[Desktop Entry]\nType=Application\nName=Obsidian Mods Manager\nExec=\"{}\" %u\nIcon=applications-games\nTerminal=false\nCategories=Game;Utility;\nMimeType=x-scheme-handler/obsidianmods;\n",
                exec
            );
            let _ = fs::write(desktop, contents);
            let _ = Command::new("xdg-mime")
                .args([
                    "default",
                    "obsidian-mods-manager.desktop",
                    "x-scheme-handler/obsidianmods",
                ])
                .status();
        }
    }
}

fn main() -> eframe::Result<()> {
    register_protocol_handler();
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--list") {
        match ModManager::detect() {
            Ok(manager) => match manager.scan() {
                Ok(mods) => {
                    println!("game={}", manager.paths.game.display());
                    println!("ue4ss={}", manager.ue4ss_installed());
                    for item in mods {
                        println!(
                            "{}\t{}\t{}",
                            if item.enabled { "enabled" } else { "disabled" },
                            item.kind.label(),
                            item.name
                        );
                    }
                }
                Err(err) => eprintln!("scan error: {err}"),
            },
            Err(err) => eprintln!("detection error: {err}"),
        }
        return Ok(());
    }

    if let Some(uri) = args
        .iter()
        .find(|arg| arg.starts_with("obsidianmods://install/"))
    {
        let id = uri.trim_start_matches("obsidianmods://install/");
        match ModManager::detect() {
            Ok(manager) => match manager.fetch_catalog() {
                Ok(catalog) => {
                    if let Some(item) = catalog.mods.iter().find(|item| item.id == id) {
                        match manager.install_catalog_mod(item) {
                            Ok(items) => eprintln!("Installed {}", items.join(", ")),
                            Err(err) => eprintln!("Install failed: {err}"),
                        }
                    } else {
                        eprintln!("Unknown Obsidian mod id: {id}");
                    }
                }
                Err(err) => eprintln!("Catalog unavailable: {err}"),
            },
            Err(err) => eprintln!("Game detection failed: {err}"),
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([860.0, 560.0])
            .with_title("Obsidian Mods Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Obsidian Mods Manager",
        options,
        Box::new(|cc| Ok(Box::new(ObsidianApp::new(cc)))),
    )
}
