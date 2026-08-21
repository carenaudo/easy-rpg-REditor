use eframe::egui;
use std::path::Path;
use crate::app_state::{AppPersistentData, EditorAppState};

#[derive(Clone, Debug)]
pub struct MissingAssetReport {
    pub category: String,
    pub file_name: String,
    pub referenced_by: String,
}

#[derive(Default)]
pub struct ProjectAnalyzerDialog {
    pub is_open: bool,
    pub is_scanned: bool,
    pub total_maps: usize,
    pub total_events: usize,
    pub total_actors: usize,
    pub total_items: usize,
    pub total_skills: usize,
    pub total_enemies: usize,
    pub total_troops: usize,
    pub total_common_events: usize,
    pub total_switches: usize,
    pub total_variables: usize,
    pub missing_assets: Vec<MissingAssetReport>,
}

impl ProjectAnalyzerDialog {
    pub fn open(&mut self, app: &EditorAppState) {
        self.is_open = true;
        self.run_analysis(app);
    }

    fn file_exists(project_path: Option<&str>, rtp_path: Option<&Path>, category: &str, file_name: &str) -> bool {
        if file_name.trim().is_empty() || file_name == "(None)" {
            return true;
        }

        let extensions = ["png", "xyz", "bmp", "mid", "wav", "mp3", "ogg", "wma"];

        // 1. Check in project folder
        if let Some(proj) = project_path {
            let cat_dir = Path::new(proj).join(category);
            for ext in &extensions {
                if cat_dir.join(format!("{}.{}", file_name, ext)).exists() {
                    return true;
                }
            }
            if cat_dir.join(file_name).exists() {
                return true;
            }
        }

        // 2. Check in RTP folder
        if let Some(rtp) = rtp_path {
            let cat_dir = rtp.join(category);
            for ext in &extensions {
                if cat_dir.join(format!("{}.{}", file_name, ext)).exists() {
                    return true;
                }
            }
            if cat_dir.join(file_name).exists() {
                return true;
            }
        }

        false
    }

    pub fn run_analysis(&mut self, app: &EditorAppState) {
        self.missing_assets.clear();
        self.total_maps = app.maps.len();
        self.total_actors = app.actors.len();
        self.total_items = app.items.len();
        self.total_skills = app.skills.len();
        self.total_enemies = app.enemies.len();
        self.total_troops = app.troops.len();
        self.total_common_events = app.common_events.len();
        self.total_switches = app.switches.len();
        self.total_variables = app.variables.len();

        let proj_path = app.project_path.as_deref();
        let config = AppPersistentData::load();
        let rtp_path = config.get_effective_rtp_path_for(proj_path.is_some().then_some(app.is_2003));

        // 1. Audit Actors
        for actor in &app.actors {
            if !actor.character_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "CharSet", &actor.character_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "CharSet".to_string(),
                    file_name: actor.character_name.clone(),
                    referenced_by: format!("Actor #{:04}: {}", actor.id, actor.name),
                });
            }
            if !actor.face_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "FaceSet", &actor.face_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "FaceSet".to_string(),
                    file_name: actor.face_name.clone(),
                    referenced_by: format!("Actor #{:04}: {}", actor.id, actor.name),
                });
            }
        }

        // 2. Audit Enemies
        for enemy in &app.enemies {
            if !enemy.battler_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "Monster", &enemy.battler_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "Monster".to_string(),
                    file_name: enemy.battler_name.clone(),
                    referenced_by: format!("Enemy #{:04}: {}", enemy.id, enemy.name),
                });
            }
        }

        // 3. Audit Chipsets
        for cs in &app.chipsets {
            if !cs.chipset_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "ChipSet", &cs.chipset_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "ChipSet".to_string(),
                    file_name: cs.chipset_name.clone(),
                    referenced_by: format!("Chipset #{:04}: {}", cs.id, cs.name),
                });
            }
        }

        // 4. Audit System Visuals
        if let Some(sys) = &app.system {
            if !sys.title_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "Title", &sys.title_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "Title".to_string(),
                    file_name: sys.title_name.clone(),
                    referenced_by: "System Database (Title Graphic)".to_string(),
                });
            }
            if !sys.gameover_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "GameOver", &sys.gameover_name) {
                self.missing_assets.push(MissingAssetReport {
                    category: "GameOver".to_string(),
                    file_name: sys.gameover_name.clone(),
                    referenced_by: "System Database (GameOver Graphic)".to_string(),
                });
            }
        }

        // 5. Audit Map Events across project
        let mut event_count = 0;
        if let Some(path) = proj_path {
            for (mid, mname) in &app.maps {
                let events = crate::lcf_bridge::get_map_events(path, *mid);
                event_count += events.len();
                for ev in events {
                    for page in &ev.pages {
                        if !page.character_name.is_empty() && !Self::file_exists(proj_path, rtp_path.as_deref(), "CharSet", &page.character_name) {
                            self.missing_assets.push(MissingAssetReport {
                                category: "CharSet".to_string(),
                                file_name: page.character_name.clone(),
                                referenced_by: format!("Map #{:04} ({}) Event #{:04}: {}", mid, mname, ev.id, ev.name),
                            });
                        }
                    }
                }
            }
        }
        self.total_events = event_count;
        self.is_scanned = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, app: &EditorAppState) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;

        egui::Window::new("🩺 Project Health & Integrity Analyzer")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([680.0, 480.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Project Metrics");
                    ui.separator();
                    if ui.button("🔄 Re-run Scan").clicked() {
                        self.run_analysis(app);
                    }
                });

                // Metrics Dashboard Cards
                ui.horizontal_wrapped(|ui| {
                    ui.group(|ui| {
                        ui.label(format!("🗺 Maps: {}", self.total_maps));
                        ui.label(format!("🎭 Events: {}", self.total_events));
                    });
                    ui.group(|ui| {
                        ui.label(format!("👤 Actors: {}", self.total_actors));
                        ui.label(format!("🐺 Enemies: {}", self.total_enemies));
                    });
                    ui.group(|ui| {
                        ui.label(format!("🗡 Items: {}", self.total_items));
                        ui.label(format!("⚡ Skills: {}", self.total_skills));
                    });
                    ui.group(|ui| {
                        ui.label(format!("📜 Common Events: {}", self.total_common_events));
                        ui.label(format!("🔘 Switches: {}", self.total_switches));
                    });
                });

                ui.separator();
                ui.heading("Asset Integrity Status");

                if self.missing_assets.is_empty() {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(0, 180, 0), "✅ All referenced graphics and sounds exist in project or RTP.");
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(240, 100, 100), format!("⚠️ Found {} missing asset references:", self.missing_assets.len()));
                    });

                    egui::ScrollArea::vertical()
                        .id_salt("missing_assets_scroll")
                        .max_height(240.0)
                        .show(ui, |ui| {
                            egui::Grid::new("missing_assets_grid")
                                .num_columns(3)
                                .spacing([12.0, 6.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Category");
                                    ui.strong("Missing Asset");
                                    ui.strong("Referenced By");
                                    ui.end_row();

                                    for item in &self.missing_assets {
                                        ui.colored_label(egui::Color32::from_rgb(100, 160, 240), &item.category);
                                        ui.colored_label(egui::Color32::from_rgb(255, 120, 120), &item.file_name);
                                        ui.label(&item.referenced_by);
                                        ui.end_row();
                                    }
                                });
                        });
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
            });

        if !is_open {
            self.is_open = false;
        }
    }
}
