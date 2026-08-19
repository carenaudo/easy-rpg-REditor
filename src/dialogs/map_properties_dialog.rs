use eframe::egui;
use crate::lcf_bridge::{self, AnchorOrigin, ChipsetInfo, MapPropertiesInfo, TroopInfo};

pub struct MapPropertiesDialogState {
    pub is_open: bool,
    pub map_id: i32,
    pub props: MapPropertiesInfo,
    pub available_chipsets: Vec<ChipsetInfo>,
    pub available_troops: Vec<TroopInfo>,
    pub anchor: AnchorOrigin,
    pub status_message: Option<Result<String, String>>,
}

impl Default for MapPropertiesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            map_id: 0,
            props: MapPropertiesInfo::default(),
            available_chipsets: Vec::new(),
            available_troops: Vec::new(),
            anchor: AnchorOrigin::TopLeft,
            status_message: None,
        }
    }
}

impl MapPropertiesDialogState {
    pub fn open(&mut self, project_path: &str, map_id: i32) {
        self.map_id = map_id;
        self.is_open = true;
        self.status_message = None;
        self.available_chipsets = lcf_bridge::get_chipsets(project_path);
        self.available_troops = lcf_bridge::get_troops(project_path);
        if let Ok(p) = lcf_bridge::get_map_properties(project_path, map_id) {
            self.props = p;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, project_path: Option<&str>) -> Option<bool> {
        if !self.is_open {
            return None;
        }

        let mut saved = None;
        let mut is_open = self.is_open;

        egui::Window::new(format!("Map Properties - Map #{:04}", self.map_id))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([580.0, 560.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("map_properties_scroll")
                    .show(ui, |ui| {
                        ui.group(|ui| {
                            ui.heading("General Properties");
                            egui::Grid::new("map_props_general_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Map Name:");
                                    ui.text_edit_singleline(&mut self.props.name);
                                    ui.end_row();

                                    ui.label("Tileset (ChipSet):");
                                    egui::ComboBox::from_id_salt("map_chipset_combo")
                                        .selected_text(
                                            self.available_chipsets
                                                .iter()
                                                .find(|c| c.id == self.props.chipset_id)
                                                .map(|c| format!("{:04}: {}", c.id, c.name))
                                                .unwrap_or_else(|| format!("ID {}", self.props.chipset_id)),
                                        )
                                        .show_ui(ui, |ui| {
                                            for cs in &self.available_chipsets {
                                                ui.selectable_value(
                                                    &mut self.props.chipset_id,
                                                    cs.id,
                                                    format!("{:04}: {}", cs.id, cs.name),
                                                );
                                            }
                                        });
                                    ui.end_row();

                                    ui.label("Loop / Scroll Wrap:");
                                    egui::ComboBox::from_id_salt("map_scroll_combo")
                                        .selected_text(match self.props.scroll_type {
                                            0 => "None",
                                            1 => "Vertical",
                                            2 => "Horizontal",
                                            3 => "Both (World Map)",
                                            _ => "Custom",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.props.scroll_type, 0, "None");
                                            ui.selectable_value(&mut self.props.scroll_type, 1, "Vertical");
                                            ui.selectable_value(&mut self.props.scroll_type, 2, "Horizontal");
                                            ui.selectable_value(&mut self.props.scroll_type, 3, "Both (World Map)");
                                        });
                                    ui.end_row();
                                });
                        });

                        ui.separator();

                        ui.group(|ui| {
                            ui.heading("Dimensions & Resize Anchor");
                            ui.horizontal(|ui| {
                                ui.label("Width (20..500):");
                                ui.add(egui::DragValue::new(&mut self.props.width).range(20..=500));
                                ui.separator();
                                ui.label("Height (15..500):");
                                ui.add(egui::DragValue::new(&mut self.props.height).range(15..=500));
                            });

                            ui.label("Resize Anchor Origin (crop/expand alignment):");
                            egui::Grid::new("anchor_grid")
                                .num_columns(3)
                                .spacing([6.0, 6.0])
                                .show(ui, |ui| {
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::TopLeft, "↖ Top-Left");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::TopCenter, "↑ Top");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::TopRight, "↗ Top-Right");
                                    ui.end_row();

                                    ui.radio_value(&mut self.anchor, AnchorOrigin::CenterLeft, "← Left");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::Center, "• Center");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::CenterRight, "→ Right");
                                    ui.end_row();

                                    ui.radio_value(&mut self.anchor, AnchorOrigin::BottomLeft, "↙ Bottom-Left");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::BottomCenter, "↓ Bottom");
                                    ui.radio_value(&mut self.anchor, AnchorOrigin::BottomRight, "↘ Bottom-Right");
                                    ui.end_row();
                                });
                        });

                        ui.separator();

                        ui.group(|ui| {
                            ui.heading("Audio & Panorama Background");
                            egui::Grid::new("map_props_audio_bg_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Music Setting:");
                                    egui::ComboBox::from_id_salt("map_music_type_combo")
                                        .selected_text(match self.props.music_type {
                                            0 => "Inherit Parent",
                                            1 => "Specific BGM",
                                            _ => "No Music",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.props.music_type, 0, "Inherit Parent");
                                            ui.selectable_value(&mut self.props.music_type, 1, "Specific BGM");
                                            ui.selectable_value(&mut self.props.music_type, 2, "No Music");
                                        });
                                    ui.end_row();

                                    if self.props.music_type == 1 {
                                        ui.label("BGM Name:");
                                        let mut dummy_dirty = false;
                                        crate::widgets::resource_dropdown::resource_combo_box(ui, "map_props_bgm_combo", &mut self.props.music_name, "Music", project_path, &mut dummy_dirty);
                                        ui.end_row();
                                    }

                                    ui.label("Panorama Graphic:");
                                    let mut dummy_dirty2 = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "map_props_panorama_combo", &mut self.props.parallax_name, "Panorama", project_path, &mut dummy_dirty2);
                                    ui.end_row();

                                    ui.label("Panorama Loops:");
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.props.parallax_loop_x, "Loop Horizontal");
                                        ui.checkbox(&mut self.props.parallax_loop_y, "Loop Vertical");
                                    });
                                    ui.end_row();

                                    ui.label("Auto-scroll Speed X:");
                                    ui.add(egui::DragValue::new(&mut self.props.parallax_sx).range(-32..=32));
                                    ui.end_row();

                                    ui.label("Auto-scroll Speed Y:");
                                    ui.add(egui::DragValue::new(&mut self.props.parallax_sy).range(-32..=32));
                                    ui.end_row();
                                });
                        });

                        ui.separator();

                        ui.group(|ui| {
                            ui.heading("Restrictions & Special Conditions");
                            egui::Grid::new("map_props_restrictions_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    let perm_label = |v: i32| match v {
                                        0 => "Inherit Parent",
                                        1 => "Allow",
                                        _ => "Disallow",
                                    };

                                    ui.label("Teleport Skill/Item:");
                                    egui::ComboBox::from_id_salt("teleport_perm")
                                        .selected_text(perm_label(self.props.teleport))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.props.teleport, 0, "Inherit Parent");
                                            ui.selectable_value(&mut self.props.teleport, 1, "Allow");
                                            ui.selectable_value(&mut self.props.teleport, 2, "Disallow");
                                        });
                                    ui.end_row();

                                    ui.label("Escape Skill/Item:");
                                    egui::ComboBox::from_id_salt("escape_perm")
                                        .selected_text(perm_label(self.props.escape))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.props.escape, 0, "Inherit Parent");
                                            ui.selectable_value(&mut self.props.escape, 1, "Allow");
                                            ui.selectable_value(&mut self.props.escape, 2, "Disallow");
                                        });
                                    ui.end_row();

                                    ui.label("Save Menu Access:");
                                    egui::ComboBox::from_id_salt("save_perm")
                                        .selected_text(perm_label(self.props.save))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.props.save, 0, "Inherit Parent");
                                            ui.selectable_value(&mut self.props.save, 1, "Allow");
                                            ui.selectable_value(&mut self.props.save, 2, "Disallow");
                                        });
                                    ui.end_row();
                                });
                        });

                        ui.separator();

                        ui.group(|ui| {
                            ui.heading("Random Encounters");
                            ui.horizontal(|ui| {
                                ui.label("Average Steps per Encounter:");
                                ui.add(egui::DragValue::new(&mut self.props.encounter_steps).range(5..=1000));
                            });

                            ui.label("Troops:");
                            let mut remove_idx = None;
                            for (idx, troop_id) in self.props.encounters.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Slot #{}:", idx + 1));
                                    egui::ComboBox::from_id_salt(format!("map_encounter_troop_{}", idx))
                                        .selected_text(
                                            self.available_troops
                                                .iter()
                                                .find(|t| t.id == *troop_id)
                                                .map(|t| format!("{:04}: {}", t.id, t.name))
                                                .unwrap_or_else(|| format!("Troop ID {}", troop_id)),
                                        )
                                        .show_ui(ui, |ui| {
                                            for t in &self.available_troops {
                                                ui.selectable_value(troop_id, t.id, format!("{:04}: {}", t.id, t.name));
                                            }
                                        });

                                    if ui.button("🗑").clicked() {
                                        remove_idx = Some(idx);
                                    }
                                });
                            }
                            if let Some(idx) = remove_idx {
                                self.props.encounters.remove(idx);
                            }
                            if ui.button("+ Add Troop Encounter").clicked() {
                                let first_troop_id = self.available_troops.first().map(|t| t.id).unwrap_or(1);
                                self.props.encounters.push(first_troop_id);
                            }
                        });
                    });

                ui.separator();

                if let Some(msg) = &self.status_message {
                    let is_dark = ui.visuals().dark_mode;
                    match msg {
                        Ok(t) => { ui.colored_label(crate::theme::colors::success(is_dark), t); }
                        Err(e) => { ui.colored_label(crate::theme::colors::danger(is_dark), e); }
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("Apply & Save Properties").clicked() {
                        if let Some(path) = project_path {
                            match lcf_bridge::save_map_properties(path, self.map_id, &self.props, self.anchor) {
                                Ok(()) => {
                                    self.status_message = Some(Ok("Map properties saved.".to_string()));
                                    saved = Some(true);
                                    self.is_open = false;
                                }
                                Err(e) => self.status_message = Some(Err(e)),
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }

        saved
    }
}
