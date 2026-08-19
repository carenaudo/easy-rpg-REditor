use eframe::egui;
use crate::lcf_bridge::{self, SaveSlotInfo};

pub struct SaveSlotView {
    pub info: SaveSlotInfo,
    pub dirty: bool,
    pub save_message: Option<Result<String, String>>,
}

pub struct SaveViewState {
    pub selected_slot: usize,
}

impl Default for SaveViewState {
    fn default() -> Self {
        Self { selected_slot: 0 }
    }
}

impl SaveViewState {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        project_path: Option<&str>,
        saves: &mut [SaveSlotView],
    ) {
        if saves.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No save files found in project directory (Save01.lsd ..).");
            return;
        }

        ui.columns(2, |cols| {
            // Left column: Slot List
            cols[0].group(|ui| {
                ui.heading("Save Slots");
                egui::ScrollArea::vertical()
                    .id_salt("save_slots_list")
                    .max_height(450.0)
                    .show(ui, |ui| {
                    for (i, slot) in saves.iter().enumerate() {
                        let is_sel = self.selected_slot == i;
                        let label = format!("{}: {} (Lv {})", slot.info.file_name, slot.info.hero_name, slot.info.hero_level);
                        if ui.selectable_label(is_sel, label).clicked() {
                            self.selected_slot = i;
                        }
                    }
                });
            });

            // Right column: Slot Editor
            cols[1].group(|ui| {
                if let Some(slot) = saves.get_mut(self.selected_slot) {
                    ui.horizontal(|ui| {
                        ui.heading(&slot.info.file_name);
                        ui.separator();
                        ui.add_enabled_ui(slot.dirty, |ui| {
                            if ui.button("Save Slot").clicked() {
                                if let Some(proj) = project_path {
                                    match lcf_bridge::save_save_slot(proj, &slot.info.file_name, &slot.info) {
                                        Ok(()) => {
                                            slot.save_message = Some(Ok("Saved successfully.".to_string()));
                                            slot.dirty = false;
                                        }
                                        Err(e) => slot.save_message = Some(Err(e)),
                                    }
                                }
                            }
                            if ui.button("Discard").clicked() {
                                if let Some(proj) = project_path {
                                    slot.info = lcf_bridge::reload_save_slot(proj, &slot.info.file_name);
                                    slot.dirty = false;
                                    slot.save_message = None;
                                }
                            }
                        });
                    });

                    let is_dark = ui.visuals().dark_mode;
                    if slot.dirty {
                        ui.colored_label(crate::theme::colors::warning(is_dark), "● Unsaved Changes");
                    }
                    if let Some(msg) = &slot.save_message {
                        match msg {
                            Ok(txt) => { ui.colored_label(crate::theme::colors::success(is_dark), txt); }
                            Err(txt) => { ui.colored_label(crate::theme::colors::danger(is_dark), txt); }
                        }
                    }

                    ui.separator();

                    egui::Grid::new("save_slot_info_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Timestamp:");
                            ui.label(&slot.info.timestamp);
                            ui.end_row();

                            ui.label("Hero Name:");
                            let name_edit = ui.text_edit_singleline(&mut slot.info.hero_name);
                            if name_edit.changed() {
                                slot.dirty = true;
                            }
                            ui.end_row();

                            ui.label("Hero Level:");
                            let lvl_edit = ui.add(egui::DragValue::new(&mut slot.info.hero_level).range(1..=99));
                            if lvl_edit.changed() {
                                slot.dirty = true;
                            }
                            ui.end_row();

                            ui.label("Gold:");
                            let gold_edit = ui.add(egui::DragValue::new(&mut slot.info.gold).range(0..=9999999));
                            if gold_edit.changed() {
                                slot.dirty = true;
                            }
                            ui.end_row();

                            ui.label("Map ID:");
                            let map_edit = ui.add(egui::DragValue::new(&mut slot.info.map_id).range(1..=9999));
                            if map_edit.changed() {
                                slot.dirty = true;
                            }
                            ui.end_row();

                            ui.label("Position (X, Y):");
                            ui.horizontal(|ui| {
                                let x_edit = ui.add(egui::DragValue::new(&mut slot.info.position_x).range(0..=500));
                                let y_edit = ui.add(egui::DragValue::new(&mut slot.info.position_y).range(0..=500));
                                if x_edit.changed() || y_edit.changed() {
                                    slot.dirty = true;
                                }
                            });
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Party Members");
                    egui::Grid::new("save_party_grid")
                        .num_columns(4)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.label("Level");
                            ui.label("HP");
                            ui.label("SP");
                            ui.end_row();

                            for member in &mut slot.info.party {
                                let n_resp = ui.text_edit_singleline(&mut member.name);
                                let l_resp = ui.add(egui::DragValue::new(&mut member.level).range(1..=99));
                                let hp_resp = ui.add(egui::DragValue::new(&mut member.current_hp).range(0..=99999));
                                let sp_resp = ui.add(egui::DragValue::new(&mut member.current_sp).range(0..=9999));
                                if n_resp.changed() || l_resp.changed() || hp_resp.changed() || sp_resp.changed() {
                                    slot.dirty = true;
                                }
                                ui.end_row();
                            }
                        });

                    if !slot.info.inventory.is_empty() {
                        ui.separator();
                        ui.heading(format!("Inventory Items ({})", slot.info.inventory.len()));
                        egui::ScrollArea::vertical()
                            .id_salt("save_inventory_items")
                            .max_height(140.0)
                            .show(ui, |ui| {
                            for (item_id, count) in &slot.info.inventory {
                                ui.label(format!("Item #{:04}: x{}", item_id, count));
                            }
                        });
                    }
                }
            });
        });
    }
}
