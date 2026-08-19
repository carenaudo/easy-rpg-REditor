use eframe::egui;
use crate::lcf_bridge::{skill_scope_label, skill_type_label, SkillInfo};

pub fn show_skill_form(ui: &mut egui::Ui, skill: &mut SkillInfo, dirty: &mut bool) {
    let type_name = skill_type_label(skill.skill_type);
    let scope_name = skill_scope_label(skill.scope);

    let is_dark = ui.visuals().dark_mode;
    let type_col = crate::theme::colors::info(is_dark);
    let scope_col = crate::theme::colors::stat_spi(is_dark);
    let sp_col = crate::theme::colors::stat_sp(is_dark);

    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("✨ {:04}: {}", skill.id, skill.name));
        ui.separator();
        ui.colored_label(type_col, format!("🏷 {}", type_name));
        ui.colored_label(scope_col, format!("🎯 {}", scope_name));
        let sp_str = if skill.sp_type == 1 {
            format!("🔮 SP Cost: {}%", skill.sp_percent)
        } else {
            format!("🔮 SP Cost: {}", skill.sp_cost)
        };
        ui.colored_label(sp_col, sp_str);
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 800.0 { 2 } else { 1 };

    egui::ScrollArea::vertical()
        .id_salt("skill_editor_scroll")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: General Info & Damage Formulas
                cols[0].group(|ui| {
                    ui.heading("General Properties");
                    egui::Grid::new("skill_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut skill.name).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Skill Type:");
                            egui::ComboBox::from_id_salt("skill_type_combo")
                                .selected_text(type_name)
                                .show_ui(ui, |ui| {
                                    for t in 0..=4 {
                                        if ui.selectable_value(&mut skill.skill_type, t, skill_type_label(t)).clicked() {
                                            *dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Scope / Target:");
                            egui::ComboBox::from_id_salt("skill_scope_combo")
                                .selected_text(scope_name)
                                .show_ui(ui, |ui| {
                                    for s in 0..=4 {
                                        if ui.selectable_value(&mut skill.scope, s, skill_scope_label(s)).clicked() {
                                            *dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("SP Cost Mode:");
                            ui.horizontal(|ui| {
                                if ui.selectable_value(&mut skill.sp_type, 0, "Flat").clicked() { *dirty = true; }
                                if ui.selectable_value(&mut skill.sp_type, 1, "% Max SP").clicked() { *dirty = true; }
                            });
                            ui.end_row();

                            if skill.sp_type == 1 {
                                ui.label("SP Percent (%):");
                                if ui.add(egui::DragValue::new(&mut skill.sp_percent).range(0..=100)).changed() { *dirty = true; }
                                ui.end_row();
                            } else {
                                ui.label("SP Cost:");
                                if ui.add(egui::DragValue::new(&mut skill.sp_cost).range(0..=9999)).changed() { *dirty = true; }
                                ui.end_row();
                            }

                            ui.label("Description:");
                            if ui.add(egui::TextEdit::singleline(&mut skill.description).desired_width(260.0)).changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("⚡ Combat Damage & Power Formula");
                    egui::Grid::new("skill_power_grid")
                        .num_columns(4)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Power / Base:");
                            if ui.add(egui::DragValue::new(&mut skill.power).range(0..=9999)).changed() { *dirty = true; }
                            ui.label("Hit Rate (%):");
                            if ui.add(egui::DragValue::new(&mut skill.hit).range(0..=100)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Physical Rate (%):");
                            if ui.add(egui::DragValue::new(&mut skill.physical_rate).range(0..=1000)).changed() { *dirty = true; }
                            ui.label("Magical Rate (%):");
                            if ui.add(egui::DragValue::new(&mut skill.magical_rate).range(0..=1000)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Variance (%):");
                            if ui.add(egui::DragValue::new(&mut skill.variance).range(0..=100)).changed() { *dirty = true; }
                            ui.label("");
                            ui.label("");
                            ui.end_row();
                        });

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut skill.ignore_defense, "Ignore Defense (Piercing)").changed() { *dirty = true; }
                        if ui.checkbox(&mut skill.absorb_damage, "Absorb Damage (Drain)").changed() { *dirty = true; }
                    });
                });

                // Column 2: Affected Stats, Usability & Triggers
                cols[1].group(|ui| {
                    ui.heading("📊 Affected Parameters");
                    egui::Grid::new("skill_affect_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            if ui.checkbox(&mut skill.affect_hp, "Affects HP").changed() { *dirty = true; }
                            if ui.checkbox(&mut skill.affect_sp, "Affects SP").changed() { *dirty = true; }
                            ui.end_row();

                            if ui.checkbox(&mut skill.affect_attack, "Affects Attack").changed() { *dirty = true; }
                            if ui.checkbox(&mut skill.affect_defense, "Affects Defense").changed() { *dirty = true; }
                            ui.end_row();

                            if ui.checkbox(&mut skill.affect_spirit, "Affects Spirit").changed() { *dirty = true; }
                            if ui.checkbox(&mut skill.affect_agility, "Affects Agility").changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("🎯 Usability & Linkages");
                    egui::Grid::new("skill_link_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Usable in Menu:");
                            if ui.checkbox(&mut skill.occasion_field, "Allowed").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Usable in Battle:");
                            if ui.checkbox(&mut skill.occasion_battle, "Allowed").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Battle Animation ID:");
                            if ui.add(egui::DragValue::new(&mut skill.animation_id).range(0..=500)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Trigger Switch ID:");
                            if ui.add(egui::DragValue::new(&mut skill.switch_id).range(0..=5000)).on_hover_text("Switch turned ON when skill is cast (e.g. Map Skill)").changed() { *dirty = true; }
                            ui.end_row();
                        });

                    if ui.checkbox(&mut skill.reverse_state_effect, "Reverse State Effect (Cures instead of Inflicting)").changed() {
                        *dirty = true;
                    }
                });
            });
        });
}

