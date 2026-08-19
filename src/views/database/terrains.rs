use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::TerrainInfo;
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

pub struct TerrainsView {
    pub selected_idx: usize,
}

impl Default for TerrainsView {
    fn default() -> Self {
        Self { selected_idx: 0 }
    }
}

impl TerrainsView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        terrains: &mut Vec<TerrainInfo>,
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        if terrains.is_empty() {
            ui.label("No terrains in database.");
            if ui.button("+ Add Terrain").clicked() {
                terrains.push(TerrainInfo {
                    id: 1,
                    name: "Plains".to_string(),
                    damage: 0,
                    encounter_rate: 100,
                    background_name: "Plains".to_string(),
                    boat_pass: false,
                    ship_pass: false,
                    airship_pass: true,
                    airship_land: true,
                    bush_depth: 0,
                    footstep_name: String::new(),
                });
                *dirty = true;
            }
            return;
        }

        ui.columns(2, |cols| {
            // Master list
            cols[0].group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Terrains");
                    if ui.small_button("+ Add").clicked() {
                        let new_id = (terrains.len() + 1) as i32;
                        terrains.push(TerrainInfo {
                            id: new_id,
                            name: format!("Terrain {:04}", new_id),
                            damage: 0,
                            encounter_rate: 100,
                            background_name: String::new(),
                            boat_pass: false,
                            ship_pass: false,
                            airship_pass: true,
                            airship_land: true,
                            bush_depth: 0,
                            footstep_name: String::new(),
                        });
                        self.selected_idx = terrains.len() - 1;
                        *dirty = true;
                    }
                    if ui.small_button("📄 Duplicate").clicked() && self.selected_idx < terrains.len() {
                        let mut copy = terrains[self.selected_idx].clone();
                        copy.id = (terrains.len() + 1) as i32;
                        copy.name = format!("{} (Copy)", copy.name);
                        terrains.push(copy);
                        self.selected_idx = terrains.len() - 1;
                        *dirty = true;
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("terrains_master_scroll")
                    .max_height(450.0)
                    .show(ui, |ui| {
                        for (idx, t) in terrains.iter().enumerate() {
                            let label = format!("{:04}: {}", t.id, t.name);
                            if ui.selectable_label(self.selected_idx == idx, label).clicked() {
                                self.selected_idx = idx;
                            }
                        }
                    });
            });

            // Detail view
            cols[1].group(|ui| {
                if let Some(t) = terrains.get_mut(self.selected_idx) {
                    ui.heading(format!("Edit Terrain #{:04}: {}", t.id, t.name));

                    egui::ScrollArea::vertical()
                        .id_salt("terrain_detail_scroll")
                        .max_height(550.0)
                        .show(ui, |ui| {
                            egui::Grid::new("terrain_general_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Name:");
                                    if ui.text_edit_singleline(&mut t.name).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Damage on Step (HP):");
                                    if ui.add(egui::DragValue::new(&mut t.damage).range(0..=1000)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Encounter Rate (%):");
                                    if ui.add(egui::DragValue::new(&mut t.encounter_rate).range(0..=1000)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Bush Transparency Depth:");
                                    egui::ComboBox::from_id_salt("terrain_bush_depth")
                                        .selected_text(match t.bush_depth {
                                            0 => "None (Normal)",
                                            1 => "1/3 Depth (Low)",
                                            2 => "1/2 Depth (Medium)",
                                            _ => "Full Depth (Deep)",
                                        })
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_value(&mut t.bush_depth, 0, "None (Normal)").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut t.bush_depth, 1, "1/3 Depth (Low)").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut t.bush_depth, 2, "1/2 Depth (Medium)").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut t.bush_depth, 3, "Full Depth (Deep)").clicked() { *dirty = true; }
                                        });
                                    ui.end_row();

                                    ui.label("Footstep SE:");
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "terrain_footstep_se", &mut t.footstep_name, "Sound", project_path, dirty);
                                    ui.end_row();

                                    ui.label("Battle Backdrop:");
                                    ui.horizontal(|ui| {
                                        let bg_text = if t.background_name.is_empty() { "(None)".to_string() } else { t.background_name.clone() };
                                        if ui.button(format!("🖼 {}", bg_text)).clicked() {
                                            if let Some(proj) = project_path {
                                                picker.open(proj, "Backdrop", &t.background_name, 0);
                                            }
                                        }
                                        if !t.background_name.is_empty() && ui.small_button("✕").clicked() {
                                            t.background_name.clear();
                                            *dirty = true;
                                        }
                                    });
                                    ui.end_row();
                                });

                            // Battle Backdrop Preview Box
                            if !t.background_name.is_empty() {
                                ui.add_space(6.0);
                                ui.label("Backdrop Live Preview:");
                                let is_dark = ui.visuals().dark_mode;
                                let disp_size = egui::vec2(240.0, 120.0);
                                let (rect, _) = ui.allocate_exact_size(disp_size, egui::Sense::hover());
                                let painter = ui.painter_at(rect);
                                draw_checkerboard(&painter, rect, 8.0, is_dark);
                                painter.rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                                if let Some(proj) = project_path {
                                    if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "Backdrop", &t.background_name) {
                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                    }
                                }
                            }

                            ui.separator();
                            ui.heading("Vehicle & Passage Restrictions");
                            ui.horizontal_wrapped(|ui| {
                                if ui.checkbox(&mut t.boat_pass, "⛵ Boat Passable").changed() { *dirty = true; }
                                if ui.checkbox(&mut t.ship_pass, "🚢 Ship Passable").changed() { *dirty = true; }
                                if ui.checkbox(&mut t.airship_pass, "🛸 Airship Passable").changed() { *dirty = true; }
                                if ui.checkbox(&mut t.airship_land, "🛬 Airship Can Land").changed() { *dirty = true; }
                            });
                        });
                }
            });
        });
    }
}

