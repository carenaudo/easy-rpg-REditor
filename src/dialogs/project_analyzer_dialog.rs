use eframe::egui;
use std::path::Path;
use crate::app_state::{AppPersistentData, EditorAppState};

#[derive(Clone, Debug)]
pub struct BrokenReferenceReport {
    pub category: String,
    pub source: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct MissingAssetReport {
    pub category: String,
    pub file_name: String,
    pub referenced_by: String,
}

#[derive(Clone, Debug)]
pub struct ManiacCommandHit {
    pub location: String,
    pub code: i32,
    pub name: String,
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
    pub broken_references: Vec<BrokenReferenceReport>,
    /// Every Maniac Patch event command found across maps, common events,
    /// and troop battle events during the last full scan - the deep,
    /// definitive counterpart to `EditorAppState::maniac`'s cheap heuristic.
    pub maniac_hits: Vec<ManiacCommandHit>,
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

        // 5. Audit Map Events across project (also scans for Maniac Patch
        // commands - the one signal EditorAppState::maniac's cheap
        // heuristic can't check without loading every map).
        self.maniac_hits.clear();
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
                        for cmd in &page.commands {
                            if crate::lcf_bridge::is_maniac_command_code(cmd.code) {
                                self.maniac_hits.push(ManiacCommandHit {
                                    location: format!("Map #{:04} ({}) Event #{:04}: {}", mid, mname, ev.id, ev.name),
                                    code: cmd.code,
                                    name: crate::lcf_bridge::maniac_command_name(cmd.code).to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        self.total_events = event_count;

        // 6. Scan Common Events for Maniac Patch commands (already fully
        // loaded in `app`, no extra file reads).
        for ce in &app.common_events {
            for cmd in &ce.commands {
                if crate::lcf_bridge::is_maniac_command_code(cmd.code) {
                    self.maniac_hits.push(ManiacCommandHit {
                        location: format!("Common Event #{:04}: {}", ce.id, ce.name),
                        code: cmd.code,
                        name: crate::lcf_bridge::maniac_command_name(cmd.code).to_string(),
                    });
                }
            }
        }

        // 7. Scan Troop battle events for Maniac Patch commands (also
        // already fully loaded in `app`).
        for troop in &app.troops {
            for page in &troop.pages {
                for cmd in &page.commands {
                    if crate::lcf_bridge::is_maniac_command_code(cmd.code) {
                        self.maniac_hits.push(ManiacCommandHit {
                            location: format!("Troop #{:04} ({}) battle event", troop.id, troop.name),
                            code: cmd.code,
                            name: crate::lcf_bridge::maniac_command_name(cmd.code).to_string(),
                        });
                    }
                }
            }
        }

        // 8. Broken reference check across Maps, Events, and Common Events
        self.broken_references.clear();
        if let Some(path) = proj_path {
            for (mid, mname) in &app.maps {
                let events = crate::lcf_bridge::get_map_events(path, *mid);
                let mut seen_ids = std::collections::HashSet::new();
                for ev in &events {
                    if !seen_ids.insert(ev.id) {
                        self.broken_references.push(BrokenReferenceReport {
                            category: "Duplicate Event ID".to_string(),
                            source: format!("Map #{:04} ({})", mid, mname),
                            description: format!("Duplicate Event ID #{:04} ('{}') found on map", ev.id, ev.name),
                        });
                    }
                    for page in &ev.pages {
                        let cond = &page.condition;
                        if cond.switch1_flag && cond.switch1_id > 0 && cond.switch1_id as usize > self.total_switches {
                            self.broken_references.push(BrokenReferenceReport {
                                category: "Switch".to_string(),
                                source: format!("Map #{:04} ({}) -> Event #{:04} ({}) Page {}", mid, mname, ev.id, ev.name, page.id),
                                description: format!("Precondition references Switch #{:04} (Max in DB: {})", cond.switch1_id, self.total_switches),
                            });
                        }
                        if cond.switch2_flag && cond.switch2_id > 0 && cond.switch2_id as usize > self.total_switches {
                            self.broken_references.push(BrokenReferenceReport {
                                category: "Switch".to_string(),
                                source: format!("Map #{:04} ({}) -> Event #{:04} ({}) Page {}", mid, mname, ev.id, ev.name, page.id),
                                description: format!("Precondition references Switch #{:04} (Max in DB: {})", cond.switch2_id, self.total_switches),
                            });
                        }
                        if cond.var_flag && cond.var_id > 0 && cond.var_id as usize > self.total_variables {
                            self.broken_references.push(BrokenReferenceReport {
                                category: "Variable".to_string(),
                                source: format!("Map #{:04} ({}) -> Event #{:04} ({}) Page {}", mid, mname, ev.id, ev.name, page.id),
                                description: format!("Precondition references Variable #{:04} (Max in DB: {})", cond.var_id, self.total_variables),
                            });
                        }
                        if cond.item_flag && cond.item_id > 0 && cond.item_id as usize > self.total_items {
                            self.broken_references.push(BrokenReferenceReport {
                                category: "Item".to_string(),
                                source: format!("Map #{:04} ({}) -> Event #{:04} ({}) Page {}", mid, mname, ev.id, ev.name, page.id),
                                description: format!("Precondition references Item #{:04} (Max in DB: {})", cond.item_id, self.total_items),
                            });
                        }
                        if cond.actor_flag && cond.actor_id > 0 && cond.actor_id as usize > self.total_actors {
                            self.broken_references.push(BrokenReferenceReport {
                                category: "Actor".to_string(),
                                source: format!("Map #{:04} ({}) -> Event #{:04} ({}) Page {}", mid, mname, ev.id, ev.name, page.id),
                                description: format!("Precondition references Actor #{:04} (Max in DB: {})", cond.actor_id, self.total_actors),
                            });
                        }

                        // Event Commands
                        for cmd in &page.commands {
                            if cmd.code == 10810 { // Teleport
                                if let Some(&target_map) = cmd.parameters.get(1) {
                                    if target_map > 0 && !app.maps.iter().any(|(id, _)| *id == target_map) {
                                        self.broken_references.push(BrokenReferenceReport {
                                            category: "Teleport".to_string(),
                                            source: format!("Map #{:04} -> Event #{:04} Page {}", mid, ev.id, page.id),
                                            description: format!("Teleports to non-existent Map #{:04}", target_map),
                                        });
                                    }
                                }
                            } else if cmd.code == 12110 { // Call Event
                                if cmd.parameters.first().copied().unwrap_or(0) == 0 {
                                    if let Some(&ce_id) = cmd.parameters.get(1) {
                                        if ce_id > 0 && ce_id as usize > self.total_common_events {
                                            self.broken_references.push(BrokenReferenceReport {
                                                category: "Common Event".to_string(),
                                                source: format!("Map #{:04} -> Event #{:04} Page {}", mid, ev.id, page.id),
                                                description: format!("Calls non-existent Common Event #{:04}", ce_id),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

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
                ui.heading("Logical Reference Integrity");

                if self.broken_references.is_empty() {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(0, 180, 0), "✅ No broken switch, variable, teleport, or common event references found.");
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 140, 0), format!("⚠️ Found {} broken logical reference(s):", self.broken_references.len()));
                    });

                    egui::ScrollArea::vertical()
                        .id_salt("broken_refs_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            egui::Grid::new("broken_refs_grid")
                                .num_columns(3)
                                .spacing([12.0, 6.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Type");
                                    ui.strong("Source Location");
                                    ui.strong("Issue");
                                    ui.end_row();

                                    for ref_issue in &self.broken_references {
                                        ui.colored_label(egui::Color32::from_rgb(255, 180, 50), &ref_issue.category);
                                        ui.label(&ref_issue.source);
                                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &ref_issue.description);
                                        ui.end_row();
                                    }
                                });
                        });
                }

                ui.separator();
                ui.heading("Maniac Patch Detection");

                let evidence = &app.maniac.evidence;
                if evidence.is_empty() && self.maniac_hits.is_empty() {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(140, 140, 140), "No Maniac Patch usage detected (heuristic - see below for what was checked).");
                    });
                } else {
                    if !evidence.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(200, 140, 255), "🔧 Maniac Patch signals found:");
                        for line in evidence {
                            ui.label(format!("  • {}", line));
                        }
                    }

                    if !self.maniac_hits.is_empty() {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(200, 140, 255), format!("{} Maniac event command(s) found:", self.maniac_hits.len()));
                        });

                        egui::ScrollArea::vertical()
                            .id_salt("maniac_hits_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                egui::Grid::new("maniac_hits_grid")
                                    .num_columns(3)
                                    .spacing([12.0, 6.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.strong("Location");
                                        ui.strong("Code");
                                        ui.strong("Command");
                                        ui.end_row();

                                        for hit in &self.maniac_hits {
                                            ui.label(&hit.location);
                                            ui.colored_label(egui::Color32::from_rgb(100, 160, 240), hit.code.to_string());
                                            ui.label(&hit.name);
                                            ui.end_row();
                                        }
                                    });
                            });
                    }
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
