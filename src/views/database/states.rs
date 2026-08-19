use eframe::egui;
use crate::lcf_bridge::StateInfo;

pub struct StatesView {
    pub selected_idx: usize,
}

impl Default for StatesView {
    fn default() -> Self {
        Self { selected_idx: 0 }
    }
}

impl StatesView {
    pub fn show(&mut self, ui: &mut egui::Ui, states: &mut Vec<StateInfo>, dirty: &mut bool) {
        if states.is_empty() {
            ui.label("No states/conditions in database.");
            if ui.button("➕ Add State").clicked() {
                states.push(StateInfo {
                    id: 1,
                    name: "Poison".to_string(),
                    a_rate: 100,
                    b_rate: 80,
                    c_rate: 60,
                    d_rate: 30,
                    e_rate: 0,
                    ..Default::default()
                });
                *dirty = true;
            }
            return;
        }

        ui.columns(2, |cols| {
            // Master list (Left column)
            cols[0].group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("States / Conditions");
                    if ui.small_button("➕ Add").clicked() {
                        let new_id = (states.len() + 1) as i32;
                        states.push(StateInfo {
                            id: new_id,
                            name: format!("State {:04}", new_id),
                            a_rate: 100,
                            b_rate: 80,
                            c_rate: 60,
                            d_rate: 30,
                            e_rate: 0,
                            ..Default::default()
                        });
                        self.selected_idx = states.len() - 1;
                        *dirty = true;
                    }
                    if ui.small_button("📄 Duplicate").clicked() && self.selected_idx < states.len() {
                        let mut copy = states[self.selected_idx].clone();
                        copy.id = (states.len() + 1) as i32;
                        copy.name = format!("{} (Copy)", copy.name);
                        states.push(copy);
                        self.selected_idx = states.len() - 1;
                        *dirty = true;
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("states_master_scroll")
                    .max_height(550.0)
                    .show(ui, |ui| {
                        for (idx, s) in states.iter().enumerate() {
                            let label = format!("{:04}: {}", s.id, s.name);
                            if ui.selectable_label(self.selected_idx == idx, label).clicked() {
                                self.selected_idx = idx;
                            }
                        }
                    });
            });

            // Detail view (Right column)
            cols[1].group(|ui| {
                if let Some(s) = states.get_mut(self.selected_idx) {
                    ui.horizontal_wrapped(|ui| {
                        ui.heading(format!("Edit Condition #{:04}: {}", s.id, s.name));
                        ui.separator();
                        let is_dark = ui.visuals().dark_mode;
                        let badge_color = if s.state_type == 1 {
                            crate::theme::colors::danger(is_dark)
                        } else {
                            crate::theme::colors::category_logic(is_dark)
                        };
                        ui.colored_label(badge_color, if s.state_type == 1 { "💀 Death / KO" } else { "💫 Normal Condition" });
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("state_detail_scroll")
                        .max_height(550.0)
                        .show(ui, |ui| {
                            // General Properties
                            ui.heading("General Properties");
                            egui::Grid::new("state_general_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Name:");
                                    if ui.text_edit_singleline(&mut s.name).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Type:");
                                    egui::ComboBox::from_id_salt("state_type_combo")
                                        .selected_text(if s.state_type == 1 { "Death / KO" } else { "Normal Condition" })
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_value(&mut s.state_type, 0, "Normal Condition").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut s.state_type, 1, "Death / KO").clicked() { *dirty = true; }
                                        });
                                    ui.end_row();

                                    ui.label("Color / Priority:");
                                    ui.horizontal(|ui| {
                                        ui.label("Priority (0..100):");
                                        if ui.add(egui::DragValue::new(&mut s.priority).range(0..=100)).changed() { *dirty = true; }
                                    });
                                    ui.end_row();

                                    ui.label("Restriction:");
                                    egui::ComboBox::from_id_salt("state_restriction_combo")
                                        .selected_text(match s.restriction {
                                            0 => "None (Can act normally)",
                                            1 => "Cannot act (Turn skip)",
                                            2 => "Attack enemies randomly",
                                            3 => "Attack allies randomly",
                                            _ => "Custom",
                                        })
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_value(&mut s.restriction, 0, "None (Can act normally)").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut s.restriction, 1, "Cannot act (Turn skip)").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut s.restriction, 2, "Attack enemies randomly").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut s.restriction, 3, "Attack allies randomly").clicked() { *dirty = true; }
                                        });
                                    ui.end_row();
                                });

                            ui.separator();

                            // Susceptibility Probabilities (A..E Rates)
                            ui.heading("🎯 Susceptibility Rates (%)");
                            egui::Grid::new("state_rates_grid")
                                .num_columns(5)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Rank A:");
                                    ui.label("Rank B:");
                                    ui.label("Rank C:");
                                    ui.label("Rank D:");
                                    ui.label("Rank E:");
                                    ui.end_row();

                                    if ui.add(egui::DragValue::new(&mut s.a_rate).range(0..=100)).changed() { *dirty = true; }
                                    if ui.add(egui::DragValue::new(&mut s.b_rate).range(0..=100)).changed() { *dirty = true; }
                                    if ui.add(egui::DragValue::new(&mut s.c_rate).range(0..=100)).changed() { *dirty = true; }
                                    if ui.add(egui::DragValue::new(&mut s.d_rate).range(0..=100)).changed() { *dirty = true; }
                                    if ui.add(egui::DragValue::new(&mut s.e_rate).range(0..=100)).changed() { *dirty = true; }
                                    ui.end_row();
                                });

                            ui.separator();

                            // Recovery Conditions
                            ui.heading("⏳ Recovery & Removal Conditions");
                            egui::Grid::new("state_recovery_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Hold Turn Count:");
                                    if ui.add(egui::DragValue::new(&mut s.hold_turn).range(0..=100)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Auto-Release Prob (%):");
                                    if ui.add(egui::DragValue::new(&mut s.auto_release_prob).range(0..=100)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Release on Physical Damage (%):");
                                    if ui.add(egui::DragValue::new(&mut s.release_by_damage).range(0..=100)).on_hover_text("Chance to recover immediately when struck in battle").changed() { *dirty = true; }
                                    ui.end_row();
                                });

                            ui.separator();

                            // Combat Modifiers
                            ui.heading("⚡ Combat Modifiers & Effects");
                            ui.horizontal_wrapped(|ui| {
                                if ui.checkbox(&mut s.affect_attack, "Halve ATK").changed() { *dirty = true; }
                                if ui.checkbox(&mut s.affect_defense, "Halve DEF").changed() { *dirty = true; }
                                if ui.checkbox(&mut s.affect_spirit, "Halve SPI").changed() { *dirty = true; }
                                if ui.checkbox(&mut s.affect_agility, "Halve AGI").changed() { *dirty = true; }
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Hit Penalty (%):");
                                if ui.add(egui::DragValue::new(&mut s.reduce_hit_ratio).range(0..=100)).changed() { *dirty = true; }
                                if ui.checkbox(&mut s.avoid_attacks, "Evade Physical Attacks").changed() { *dirty = true; }
                                if ui.checkbox(&mut s.reflect_magic, "Reflect Magic").changed() { *dirty = true; }
                                if ui.checkbox(&mut s.cursed, "Cursed Equipment").changed() { *dirty = true; }
                            });

                            ui.separator();

                            // HP / SP Drain / Damage over time
                            ui.heading("💚 HP & SP Drain (Turn / Map Steps)");
                            egui::Grid::new("state_drain_grid")
                                .num_columns(4)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("HP Turn Drain (% / Flat):");
                                    if ui.add(egui::DragValue::new(&mut s.hp_change_val).range(0..=9999)).changed() { *dirty = true; }
                                    ui.label("HP Map Drain (Steps):");
                                    if ui.add(egui::DragValue::new(&mut s.hp_change_map_steps).range(0..=255)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("SP Turn Drain (% / Flat):");
                                    if ui.add(egui::DragValue::new(&mut s.sp_change_val).range(0..=9999)).changed() { *dirty = true; }
                                    ui.label("SP Map Drain (Steps):");
                                    if ui.add(egui::DragValue::new(&mut s.sp_change_map_steps).range(0..=255)).changed() { *dirty = true; }
                                    ui.end_row();
                                });

                            ui.separator();

                            // Battle Messages
                            ui.heading("💬 Battle Messages");
                            egui::Grid::new("state_messages_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Hero Inflicted:");
                                    if ui.text_edit_singleline(&mut s.message_actor).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Enemy Inflicted:");
                                    if ui.text_edit_singleline(&mut s.message_enemy).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Already Inflicted:");
                                    if ui.text_edit_singleline(&mut s.message_already).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Recovery Message:");
                                    if ui.text_edit_singleline(&mut s.message_recovery).changed() { *dirty = true; }
                                    ui.end_row();
                                });
                        });
                }
            });
        });
    }
}

