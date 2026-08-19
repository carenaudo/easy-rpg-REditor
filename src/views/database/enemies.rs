use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::{EnemyActionInfo, EnemyInfo};
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

pub fn show_enemy_form(
    ui: &mut egui::Ui,
    enemy: &mut EnemyInfo,
    project_path: Option<&str>,
    picker: &mut AssetPickerState,
    cache: &mut AssetPreviewCache,
    dirty: &mut bool,
) {
    let is_dark = ui.visuals().dark_mode;
    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("👹 {:04}: {}", enemy.id, enemy.name));
        ui.separator();

        // Stat Pills Header
        let hp_col = crate::theme::colors::stat_hp(is_dark);
        let sp_col = crate::theme::colors::stat_sp(is_dark);
        let atk_col = crate::theme::colors::stat_atk(is_dark);
        let def_col = crate::theme::colors::stat_def(is_dark);
        let spi_col = crate::theme::colors::stat_spi(is_dark);
        let agi_col = crate::theme::colors::stat_agi(is_dark);

        ui.colored_label(hp_col, format!("HP: {}", enemy.max_hp));
        ui.colored_label(sp_col, format!("SP: {}", enemy.max_sp));
        ui.colored_label(atk_col, format!("ATK: {}", enemy.attack));
        ui.colored_label(def_col, format!("DEF: {}", enemy.defense));
        ui.colored_label(spi_col, format!("SPI: {}", enemy.spirit));
        ui.colored_label(agi_col, format!("AGI: {}", enemy.agility));
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 900.0 { 2 } else { 1 };

    egui::ScrollArea::vertical()
        .id_salt("enemy_editor_scroll")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: Parameters & General Settings
                cols[0].group(|ui| {
                    ui.heading("General Details & Rewards");
                    egui::Grid::new("enemy_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            let name_edit = ui.text_edit_singleline(&mut enemy.name);
                            if name_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Battler Graphic:");
                            ui.horizontal(|ui| {
                                let btn_text = if enemy.battler_name.is_empty() { "(None)".to_string() } else { format!("🖼 {}", enemy.battler_name) };
                                if ui.button(btn_text).clicked() {
                                    if let Some(proj) = project_path {
                                        picker.open(proj, "Monster", &enemy.battler_name, 0);
                                    }
                                }
                            });
                            ui.end_row();

                            ui.label("Battler Hue:");
                            if ui.add(egui::Slider::new(&mut enemy.battler_hue, 0..=360).suffix("°")).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("EXP Yield:");
                            let exp_edit = ui.add(egui::DragValue::new(&mut enemy.exp).range(0..=999999));
                            if exp_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Gold Yield:");
                            let gold_edit = ui.add(egui::DragValue::new(&mut enemy.gold).range(0..=999999));
                            if gold_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Drop Item ID:");
                            let drop_edit = ui.add(egui::DragValue::new(&mut enemy.drop_id).range(0..=5000));
                            if drop_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Drop Probability (%):");
                            let drop_p = ui.add(egui::DragValue::new(&mut enemy.drop_prob).range(0..=100));
                            if drop_p.changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Combat Parameters");
                    egui::Grid::new("enemy_params_grid")
                        .num_columns(4)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Max HP:");
                            let hp_edit = ui.add(egui::DragValue::new(&mut enemy.max_hp).range(1..=999999));
                            if hp_edit.changed() { *dirty = true; }
                            ui.label("Max SP:");
                            let sp_edit = ui.add(egui::DragValue::new(&mut enemy.max_sp).range(0..=99999));
                            if sp_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Attack:");
                            let atk_edit = ui.add(egui::DragValue::new(&mut enemy.attack).range(1..=9999));
                            if atk_edit.changed() { *dirty = true; }
                            ui.label("Defense:");
                            let def_edit = ui.add(egui::DragValue::new(&mut enemy.defense).range(1..=9999));
                            if def_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Spirit:");
                            let spi_edit = ui.add(egui::DragValue::new(&mut enemy.spirit).range(1..=9999));
                            if spi_edit.changed() { *dirty = true; }
                            ui.label("Agility:");
                            let agi_edit = ui.add(egui::DragValue::new(&mut enemy.agility).range(1..=9999));
                            if agi_edit.changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Special Combat Traits");
                    ui.horizontal_wrapped(|ui| {
                        if ui.checkbox(&mut enemy.critical_hit, "Can Crit").changed() { *dirty = true; }
                        if enemy.critical_hit {
                            ui.label("Chance (1/X):");
                            if ui.add(egui::DragValue::new(&mut enemy.critical_hit_chance).range(1..=100)).changed() { *dirty = true; }
                        }
                        if ui.checkbox(&mut enemy.miss, "Can Miss").changed() { *dirty = true; }
                        if ui.checkbox(&mut enemy.levitate, "Levitate/Fly").changed() { *dirty = true; }
                        if ui.checkbox(&mut enemy.transparent, "Transparent").changed() { *dirty = true; }
                    });
                });

                // Column 2: Battler Preview
                let preview_col = if num_cols > 1 { &mut cols[1] } else { &mut cols[0] };
                preview_col.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Battler Preview");
                        if !enemy.battler_name.is_empty() {
                            ui.label(format!("({})", enemy.battler_name));
                        }
                    });

                    let canvas_width = ui.available_width().max(260.0);
                    let card_sz = egui::vec2(canvas_width, 240.0);
                    let (rect, _) = ui.allocate_exact_size(card_sz, egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    draw_checkerboard(&painter, rect, 10.0, is_dark);
                    painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                    let mut drawn = false;
                    if let Some(proj) = project_path {
                        if !enemy.battler_name.is_empty() {
                            if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "Monster", &enemy.battler_name) {
                                let sz = tex.size_vec2();
                                let max_w: f32 = rect.width() - 24.0;
                                let max_h: f32 = rect.height() - 24.0;
                                let scale = (max_w / sz.x.max(1.0)).min(max_h / sz.y.max(1.0)).min(2.0);
                                let fit_w = sz.x * scale;
                                let fit_h = sz.y * scale;
                                let img_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(fit_w, fit_h));
                                painter.image(tex.id(), img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                drawn = true;

                                let dim_col = crate::theme::colors::dim(is_dark);

                                painter.text(
                                    egui::pos2(rect.max.x - 8.0, rect.max.y - 8.0),
                                    egui::Align2::RIGHT_BOTTOM,
                                    format!("{}x{} px", sz.x as i32, sz.y as i32),
                                    egui::FontId::proportional(11.0),
                                    dim_col,
                                );
                            }
                        }
                    }

                    if !drawn {
                        let placeholder_col = crate::theme::colors::muted(is_dark);
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "(No Battler Graphic Assigned)", egui::FontId::proportional(13.0), placeholder_col);
                    }
                });
            });

            ui.separator();

            // Full Enemy AI Behavior Actions Studio
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("🤖 Enemy AI Actions & Behavior");
                    if ui.button("➕ Add Action").clicked() {
                        let new_id = (enemy.actions.len() + 1) as i32;
                        enemy.actions.push(EnemyActionInfo {
                            id: new_id,
                            kind: 0,
                            basic: 0, // Normal attack
                            skill_id: 1,
                            enemy_id: 1,
                            condition_type: 0, // Always
                            condition_param1: 0,
                            condition_param2: 100,
                            switch_id: 1,
                            switch_on: false,
                            switch_on_id: 1,
                            switch_off: false,
                            switch_off_id: 1,
                            rating: 50,
                        });
                        *dirty = true;
                    }
                });

                if enemy.actions.is_empty() {
                    ui.label("No AI actions configured. Enemy will default to standard attack.");
                } else {
                    let mut to_delete = None;
                    for (idx, action) in enemy.actions.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("#{}:", idx + 1));

                            // Action Kind
                            egui::ComboBox::from_id_salt(format!("act_kind_{}", idx))
                                .selected_text(match action.kind {
                                    0 => "⚔ Basic Action",
                                    1 => "✨ Cast Skill",
                                    _ => "🔄 Transform/Morph",
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut action.kind, 0, "⚔ Basic Action").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.kind, 1, "✨ Cast Skill").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.kind, 2, "🔄 Transform/Morph").clicked() { *dirty = true; }
                                });

                            if action.kind == 0 {
                                egui::ComboBox::from_id_salt(format!("act_basic_{}", idx))
                                    .selected_text(match action.basic {
                                        0 => "Normal Attack",
                                        1 => "Double Attack",
                                        2 => "Defend",
                                        3 => "Observe Situation",
                                        4 => "Charge Energy",
                                        5 => "Self-Destruct",
                                        6 => "Escape",
                                        _ => "Do Nothing",
                                    })
                                    .show_ui(ui, |ui| {
                                        for (b, lbl) in &[
                                            (0, "Normal Attack"), (1, "Double Attack"), (2, "Defend"),
                                            (3, "Observe Situation"), (4, "Charge Energy"), (5, "Self-Destruct"),
                                            (6, "Escape"), (7, "Do Nothing"),
                                        ] {
                                            if ui.selectable_value(&mut action.basic, *b, *lbl).clicked() { *dirty = true; }
                                        }
                                    });
                            } else if action.kind == 1 {
                                ui.label("Skill ID:");
                                if ui.add(egui::DragValue::new(&mut action.skill_id).range(1..=5000)).changed() { *dirty = true; }
                            } else {
                                ui.label("Morph to Enemy ID:");
                                if ui.add(egui::DragValue::new(&mut action.enemy_id).range(1..=5000)).changed() { *dirty = true; }
                            }

                            // Condition
                            ui.separator();
                            ui.label("Condition:");
                            egui::ComboBox::from_id_salt(format!("act_cond_{}", idx))
                                .selected_text(match action.condition_type {
                                    0 => "Always (100%)",
                                    1 => "Switch ON",
                                    2 => "Turn A+B*X",
                                    3 => "Actor Count",
                                    4 => "HP Range %",
                                    5 => "SP Range %",
                                    _ => "Other",
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut action.condition_type, 0, "Always (100%)").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.condition_type, 1, "Switch ON").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.condition_type, 2, "Turn A+B*X").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.condition_type, 3, "Actor Count").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.condition_type, 4, "HP Range %").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut action.condition_type, 5, "SP Range %").clicked() { *dirty = true; }
                                });

                            if action.condition_type == 1 {
                                ui.label("Switch #:");
                                if ui.add(egui::DragValue::new(&mut action.switch_id).range(1..=5000)).changed() { *dirty = true; }
                            } else if action.condition_type == 2 {
                                ui.label("Turn A:");
                                if ui.add(egui::DragValue::new(&mut action.condition_param1).range(0..=999)).changed() { *dirty = true; }
                                ui.label("+ B*X:");
                                if ui.add(egui::DragValue::new(&mut action.condition_param2).range(1..=999)).changed() { *dirty = true; }
                            } else if action.condition_type == 4 || action.condition_type == 5 {
                                ui.label("Min %:");
                                if ui.add(egui::DragValue::new(&mut action.condition_param1).range(0..=100)).changed() { *dirty = true; }
                                ui.label("Max %:");
                                if ui.add(egui::DragValue::new(&mut action.condition_param2).range(0..=100)).changed() { *dirty = true; }
                            }

                            // Rating / Priority
                            ui.separator();
                            ui.label("Rating:");
                            if ui.add(egui::Slider::new(&mut action.rating, 1..=100)).changed() { *dirty = true; }

                            if ui.small_button("🗑").clicked() {
                                to_delete = Some(idx);
                            }
                        });
                    }

                    if let Some(del_idx) = to_delete {
                        enemy.actions.remove(del_idx);
                        *dirty = true;
                    }
                }
            });
        });
}


