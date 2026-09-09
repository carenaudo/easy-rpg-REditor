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
        saves: &mut Vec<SaveSlotView>,
        is_2003: bool,
    ) {
        if saves.is_empty() {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::GRAY, "No save files found in project directory (Save01.lsd ..).");
                if let Some(proj) = project_path {
                    if ui.button("➕ Create First Save File (Save01.lsd)").clicked() {
                        if let Ok(new_name) = lcf_bridge::create_blank_save_slot(proj) {
                            let new_slot = lcf_bridge::reload_save_slot(proj, &new_name);
                            saves.push(SaveSlotView {
                                info: new_slot,
                                dirty: false,
                                save_message: Some(Ok("Created new save file.".to_string())),
                            });
                            self.selected_slot = 0;
                        }
                    }
                }
            });
            return;
        }

        ui.columns(2, |cols| {
            // Left column: Slot List
            cols[0].group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Save Slots");
                    if let Some(proj) = project_path {
                        if ui.small_button("➕ New Save").on_hover_text("Create a new blank save file in this project").clicked() {
                            if let Ok(new_name) = lcf_bridge::create_blank_save_slot(proj) {
                                let new_slot = lcf_bridge::reload_save_slot(proj, &new_name);
                                saves.push(SaveSlotView {
                                    info: new_slot,
                                    dirty: false,
                                    save_message: Some(Ok("Created new save file.".to_string())),
                                });
                                self.selected_slot = saves.len().saturating_sub(1);
                            }
                        }
                    }
                });
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
            let mut clone_source = None;
            cols[1].group(|ui| {
                if let Some(slot) = saves.get_mut(self.selected_slot) {
                    ui.horizontal(|ui| {
                        ui.heading(&slot.info.file_name);
                        if project_path.is_some() {
                            if ui.small_button("📄 Clone Slot").on_hover_text("Duplicate this save file into the next free slot").clicked() {
                                clone_source = Some(slot.info.file_name.clone());
                            }
                        }
                        ui.separator();
                        ui.add_enabled_ui(slot.dirty, |ui| {
                            if ui.button("Save Slot").clicked() {
                                if let Some(proj) = project_path {
                                    let engine = if is_2003 {
                                        lcf_bridge::EngineVersion::Engine2003
                                    } else {
                                        lcf_bridge::EngineVersion::Engine2000
                                    };
                                    match lcf_bridge::save_save_slot_with_engine(proj, &slot.info.file_name, &slot.info, engine) {
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
                    ui.horizontal(|ui| {
                        ui.heading("Party Members");
                        if ui.small_button("➕ Add Hero").clicked() {
                            let new_id = (slot.info.party.len() + 1) as i32;
                            slot.info.party.push(crate::lcf_bridge::SavePartyMember {
                                id: new_id,
                                name: format!("Hero {}", new_id),
                                level: 1,
                                current_hp: 100,
                                current_sp: 50,
                            });
                            slot.dirty = true;
                        }
                    });

                    let mut remove_party_idx = None;
                    egui::Grid::new("save_party_grid")
                        .num_columns(6)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("ID");
                            ui.label("Name");
                            ui.label("Level");
                            ui.label("HP");
                            ui.label("SP");
                            ui.label("");
                            ui.end_row();

                            for (idx, member) in slot.info.party.iter_mut().enumerate() {
                                ui.label(format!("#{}", member.id));
                                let n_resp = ui.text_edit_singleline(&mut member.name);
                                let l_resp = ui.add(egui::DragValue::new(&mut member.level).range(1..=99));
                                let hp_resp = ui.add(egui::DragValue::new(&mut member.current_hp).range(0..=99999));
                                let sp_resp = ui.add(egui::DragValue::new(&mut member.current_sp).range(0..=9999));
                                if n_resp.changed() || l_resp.changed() || hp_resp.changed() || sp_resp.changed() {
                                    slot.dirty = true;
                                }
                                if ui.small_button("🗑").clicked() {
                                    remove_party_idx = Some(idx);
                                }
                                ui.end_row();
                            }
                        });

                    if let Some(idx) = remove_party_idx {
                        if slot.info.party.len() > 1 {
                            slot.info.party.remove(idx);
                            slot.dirty = true;
                        }
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading(format!("Inventory Items ({})", slot.info.inventory.len()));
                        if ui.small_button("➕ Add Item").clicked() {
                            slot.info.inventory.push((1, 1));
                            slot.dirty = true;
                        }
                    });

                    let mut remove_item_idx = None;
                    egui::ScrollArea::vertical()
                        .id_salt("save_inventory_items_scroll")
                        .max_height(140.0)
                        .show(ui, |ui| {
                            egui::Grid::new("save_inventory_grid")
                                .num_columns(4)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Item ID");
                                    ui.label("Count");
                                    ui.label("");
                                    ui.label("");
                                    ui.end_row();

                                    for (idx, (item_id, count)) in slot.info.inventory.iter_mut().enumerate() {
                                        if ui.add(egui::DragValue::new(item_id).range(1..=5000).prefix("#")).changed() {
                                            slot.dirty = true;
                                        }
                                        if ui.add(egui::DragValue::new(count).range(1..=99).prefix("x")).changed() {
                                            slot.dirty = true;
                                        }
                                        if ui.small_button("🗑").clicked() {
                                            remove_item_idx = Some(idx);
                                        }
                                        ui.end_row();
                                    }
                                });
                        });

                    if let Some(idx) = remove_item_idx {
                        if idx < slot.info.inventory.len() {
                            slot.info.inventory.remove(idx);
                            slot.dirty = true;
                        }
                    }

                    ui.separator();
                    ui.collapsing(format!("⚡ Save Switches ({})", slot.info.switches.len()), |ui| {
                        if slot.info.switches.is_empty() {
                            ui.label("(No switch states in save)");
                        } else {
                            egui::ScrollArea::vertical()
                                .id_salt("save_switches_scroll")
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    egui::Grid::new("save_switches_grid")
                                        .num_columns(4)
                                        .spacing([8.0, 4.0])
                                        .show(ui, |ui| {
                                            for (idx, sw) in slot.info.switches.iter_mut().enumerate() {
                                                let sw_num = idx + 1;
                                                let label = format!("{:04}: {}", sw_num, if *sw { "ON" } else { "OFF" });
                                                if ui.checkbox(sw, label).changed() {
                                                    slot.dirty = true;
                                                }
                                                if (idx + 1) % 4 == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                });
                        }
                    });

                    ui.separator();
                    ui.collapsing(format!("🔢 Save Variables ({})", slot.info.variables.len()), |ui| {
                        if slot.info.variables.is_empty() {
                            ui.label("(No variable states in save)");
                        } else {
                            egui::ScrollArea::vertical()
                                .id_salt("save_variables_scroll")
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    egui::Grid::new("save_variables_grid")
                                        .num_columns(4)
                                        .spacing([8.0, 4.0])
                                        .show(ui, |ui| {
                                            for (idx, var) in slot.info.variables.iter_mut().enumerate() {
                                                let var_num = idx + 1;
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("{:04}:", var_num));
                                                    if ui.add(egui::DragValue::new(var).range(-9999999..=9999999)).changed() {
                                                        slot.dirty = true;
                                                    }
                                                });
                                                if (idx + 1) % 2 == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                });
                        }
                    });
                }
            });

            if let Some(src_name) = clone_source {
                if let Some(proj) = project_path {
                    if let Ok(cloned_name) = lcf_bridge::clone_save_slot(proj, &src_name) {
                        let cloned_slot = lcf_bridge::reload_save_slot(proj, &cloned_name);
                        saves.push(SaveSlotView {
                            info: cloned_slot,
                            dirty: false,
                            save_message: Some(Ok("Cloned save file successfully.".to_string())),
                        });
                        self.selected_slot = saves.len().saturating_sub(1);
                    }
                }
            }
        });
    }
}
