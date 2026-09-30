use crate::manager::ModManager;
use crate::model::{InstalledMod, ModKind};
use eframe::egui;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey},
};
use std::fs;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct OverlayMod {
    name: String,
    version: Option<String>,
    kind: ModKind,
    state: String,
    detail: String,
    ok: bool,
}

pub struct OverlayApp {
    manager: Option<ModManager>,
    _hotkeys: Option<GlobalHotKeyManager>,
    hotkey_id: u32,
    visible: bool,
    last_refresh: Instant,
    mods: Vec<OverlayMod>,
    loader_status: String,
    loader_ok: bool,
}
impl OverlayApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_visible: bool) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgba_premultiplied(10, 10, 15, 245);
        visuals.window_fill = egui::Color32::from_rgba_premultiplied(14, 14, 21, 248);
        cc.egui_ctx.set_visuals(visuals);

        let hotkey = HotKey::new(None, Code::F8);
        let hotkey_id = hotkey.id();
        let hotkeys = GlobalHotKeyManager::new().ok();
        if let Some(manager) = hotkeys.as_ref() {
            let _ = manager.register(hotkey);
        }

        let mut app = Self {
            manager: ModManager::detect().ok(),
            _hotkeys: hotkeys,
            hotkey_id,
            visible: start_visible,
            last_refresh: Instant::now() - Duration::from_secs(5),
            mods: Vec::new(),
            loader_status: "Checking loader…".into(),
            loader_ok: true,
        };
        app.refresh();
        app
    }
    fn refresh(&mut self) {
        let Some(manager) = self.manager.as_ref() else {
            self.loader_status = "Minecraft Dungeons II not detected".into();
            self.loader_ok = false;
            self.mods.clear();
            return;
        };

        let vanilla = manager.vanilla_mode();
        let log_path = manager.paths.win64.join("ue4ss/UE4SS.log");
        let log = fs::read_to_string(log_path).unwrap_or_default();
        let crashed_early = log.contains("Locating KismetStringLibrary CDO")
            && !log.contains("[Lua]")
            && !log.contains("Starting Lua mod");

        if vanilla {
            self.loader_status = "Vanilla mode · all managed mods bypassed".into();
            self.loader_ok = true;
        } else if crashed_early {
            self.loader_status =
                "UE4SS failed during UE 5.6 initialization · PAK mods remain usable".into();
            self.loader_ok = false;
        } else if manager.ue4ss_installed() {
            self.loader_status = "Modded mode · UE4SS detected".into();
            self.loader_ok = true;
        } else {
            self.loader_status = "Modded mode · PAK loader only".into();
            self.loader_ok = true;
        }

        let installed = manager.scan().unwrap_or_default();
        self.mods = installed
            .iter()
            .map(|item| self.describe_mod(manager, item, vanilla, crashed_early))
            .collect();
        self.last_refresh = Instant::now();
    }
    fn describe_mod(
        &self,
        manager: &ModManager,
        item: &InstalledMod,
        vanilla: bool,
        crashed_early: bool,
    ) -> OverlayMod {
        let (state, detail, ok) = if vanilla {
            ("Bypassed", "Vanilla mode is active".to_string(), true)
        } else if !item.enabled {
            (
                "Disabled",
                "Disabled in Obsidian Mods Manager".to_string(),
                true,
            )
        } else if item.kind == ModKind::Pak {
            (
                "Active",
                "PAK/IoStore override is installed".to_string(),
                true,
            )
        } else {
            let status_path = manager
                .paths
                .ue4ss_mods
                .join("ObsidianRuntime/status")
                .join(format!("{}.status", item.id));
            let report = fs::read_to_string(status_path).unwrap_or_default();
            if report.trim().eq_ignore_ascii_case("loaded") {
                ("Active", "Startup health report received".to_string(), true)
            } else if crashed_early {
                (
                    "Failed",
                    "UE4SS stopped before this mod could start".to_string(),
                    false,
                )
            } else {
                (
                    "Enabled",
                    "Waiting for a startup health report".to_string(),
                    true,
                )
            }
        };

        OverlayMod {
            name: item.name.clone(),
            version: item.version.clone(),
            kind: item.kind.clone(),
            state: state.into(),
            detail,
            ok,
        }
    }
}
impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey_id && event.state == HotKeyState::Pressed {
                self.visible = !self.visible;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.visible));
                if self.visible {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.refresh();
                }
            }
        }

        if self.last_refresh.elapsed() >= Duration::from_secs(1) {
            self.refresh();
        }
        ctx.request_repaint_after(Duration::from_millis(150));

        if !self.visible {
            return;
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgba_premultiplied(9, 9, 14, 246))
                    .inner_margin(24.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OBSIDIAN").size(24.0).strong());
                    ui.label(
                        egui::RichText::new("MODS")
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::from_rgb(166, 120, 255)),
                    );
                    ui.separator();
                    ui.label(egui::RichText::new("F8 to close").weak());
                });
                ui.add_space(8.0);
                let status_color = if self.loader_ok {
                    egui::Color32::from_rgb(112, 218, 143)
                } else {
                    egui::Color32::from_rgb(244, 111, 118)
                };
                ui.horizontal(|ui| {
                    ui.colored_label(status_color, "●");
                    ui.label(&self.loader_status);
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading("Managed mods");
                if self.mods.is_empty() {
                    ui.label(egui::RichText::new("No managed mods installed.").weak());
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for item in &self.mods {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let dot = if item.ok { "●" } else { "●" };
                                    let color = if item.ok {
                                        egui::Color32::from_rgb(112, 218, 143)
                                    } else {
                                        egui::Color32::from_rgb(244, 111, 118)
                                    };
                                    ui.colored_label(color, dot);
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&item.name).size(16.0).strong(),
                                            );
                                            ui.small(item.kind.label());
                                            if let Some(version) = &item.version {
                                                ui.small(format!("v{version}"));
                                            }
                                        });
                                        ui.small(&item.detail);
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(&item.state)
                                                    .color(color)
                                                    .strong(),
                                            );
                                        },
                                    );
                                });
                            });
                            ui.add_space(6.0);
                        }
                    });
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Obsidian Mods Manager {}",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Refresh").clicked() {
                            self.refresh();
                        }
                    });
                });
            });
    }
}

pub fn run(start_visible: bool) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([560.0, 360.0])
            .with_title("Obsidian Mods Overlay")
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            .with_visible(start_visible),
        ..Default::default()
    };

    eframe::run_native(
        "Obsidian Mods Overlay",
        options,
        Box::new(move |cc| Ok(Box::new(OverlayApp::new(cc, start_visible)))),
    )
}
