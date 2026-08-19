use eframe::egui;
use crate::lcf_bridge::{ClassInfo, SkillInfo};
use crate::views::database::actors::{generate_growth_curve, GrowthCurvePreset, StatCurveType};

pub struct ClassViewState {
    pub active_stat_curve: StatCurveType,
    pub curve_v1: i32,
    pub curve_vmax: i32,
    pub curve_preset: GrowthCurvePreset,
}

impl Default for ClassViewState {
    fn default() -> Self {
        Self {
            active_stat_curve: StatCurveType::MaxHp,
            curve_v1: 500,
            curve_vmax: 5000,
            curve_preset: GrowthCurvePreset::Linear,
        }
    }
}

pub fn show_class_form(
    ui: &mut egui::Ui,
    class: &mut ClassInfo,
    skills: &[SkillInfo],
    states: &[crate::lcf_bridge::StateInfo],
    attributes: &[crate::lcf_bridge::AttributeInfo],
    view_state: &mut ClassViewState,
    dirty: &mut bool,
) {
    let is_dark = ui.visuals().dark_mode;
    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("🛡 {:04}: {}", class.id, class.name));
        ui.separator();
        if class.two_weapon {
            let col = crate::theme::colors::stat_atk(is_dark);
            ui.colored_label(col, "⚔ Dual Wield");
        }
        if class.auto_battle {
            let col = crate::theme::colors::info(is_dark);
            ui.colored_label(col, "🤖 Auto Battle");
        }
        if class.super_guard {
            let col = crate::theme::colors::warning(is_dark);
            ui.colored_label(col, "🛡 Super Guard");
        }
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 900.0 { 2 } else { 1 };

    egui::ScrollArea::vertical()
        .id_salt("class_editor_scroll")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: General Info, EXP Curve & Traits
                cols[0].group(|ui| {
                    ui.heading("General Properties & Traits");
                    egui::Grid::new("class_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut class.name).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Base EXP:");
                            if ui.add(egui::DragValue::new(&mut class.exp_base).range(1..=10000)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("EXP Inflation:");
                            if ui.add(egui::DragValue::new(&mut class.exp_inflation).range(1..=10000)).changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Special Class Traits");
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut class.two_weapon, "Dual Wield (Two Weapons instead of Shield)").changed() { *dirty = true; }
                        if ui.checkbox(&mut class.lock_equipment, "Lock Equipment (Cannot change gear in menu)").changed() { *dirty = true; }
                        if ui.checkbox(&mut class.auto_battle, "Auto Battle (AI commands in combat)").changed() { *dirty = true; }
                        if ui.checkbox(&mut class.super_guard, "Super Guard (Guard takes 1/4 damage)").changed() { *dirty = true; }
                    });

                    ui.separator();
                    // Skill Learning Table
                    ui.horizontal(|ui| {
                        ui.heading("📖 Learned Skills");
                        if ui.button("➕ Add Skill").clicked() {
                            class.skills.push((1, skills.first().map(|s| s.id).unwrap_or(1)));
                            *dirty = true;
                        }
                    });

                    if class.skills.is_empty() {
                        ui.label("No skills assigned to this class.");
                    } else {
                        let mut to_delete = None;
                        for (idx, (lvl, skill_id)) in class.skills.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{}: Level", idx + 1));
                                if ui.add(egui::DragValue::new(lvl).range(1..=99)).changed() { *dirty = true; }

                                egui::ComboBox::from_id_salt(format!("class_skill_combo_{}", idx))
                                    .selected_text(
                                        skills.iter().find(|s| s.id == *skill_id)
                                            .map(|s| format!("{:03}: {}", s.id, s.name))
                                            .unwrap_or_else(|| format!("Skill ID {}", skill_id))
                                    )
                                    .show_ui(ui, |ui| {
                                        for s in skills {
                                            if ui.selectable_value(skill_id, s.id, format!("{:03}: {}", s.id, s.name)).clicked() {
                                                *dirty = true;
                                            }
                                        }
                                    });

                                if ui.small_button("🗑").clicked() {
                                    to_delete = Some(idx);
                                }
                            });
                        }

                        if let Some(del_idx) = to_delete {
                            class.skills.remove(del_idx);
                            *dirty = true;
                        }
                    }
                });

                // Column 2: Stat Growth Studio (Interactive Canvas)
                cols[1].group(|ui| {
                    ui.heading("Class Stat Growth Studio");
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::MaxHp, "Max HP");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::MaxSp, "Max SP");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Attack, "Attack");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Defense, "Defense");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Spirit, "Spirit");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Agility, "Agility");
                    });

                    let current_curve: &mut Vec<i16> = match view_state.active_stat_curve {
                        StatCurveType::MaxHp => &mut class.param_maxhp,
                        StatCurveType::MaxSp => &mut class.param_maxsp,
                        StatCurveType::Attack => &mut class.param_attack,
                        StatCurveType::Defense => &mut class.param_defense,
                        StatCurveType::Spirit => &mut class.param_spirit,
                        StatCurveType::Agility => &mut class.param_agility,
                    };

                    if current_curve.len() < 99 {
                        let default_val = match view_state.active_stat_curve {
                            StatCurveType::MaxHp => 500,
                            StatCurveType::MaxSp => 100,
                            _ => 25,
                        };
                        current_curve.resize(99, default_val);
                    }

                    // Curve Formula Toolbar
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Lv 1:");
                            ui.add(egui::DragValue::new(&mut view_state.curve_v1).range(1..=9999));
                            ui.label("Lv 99:");
                            ui.add(egui::DragValue::new(&mut view_state.curve_vmax).range(1..=9999));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::EarlyBloomer, "🌱 Early");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::Linear, "📏 Linear");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::LateBloomer, "🚀 Late");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::SCurve, "〰 S-Curve");

                            if ui.button("⚡ Apply Preset").clicked() {
                                *current_curve = generate_growth_curve(view_state.curve_v1, view_state.curve_vmax, 99, view_state.curve_preset);
                                *dirty = true;
                            }
                        });
                    });

                    // Interactive Stat Curve Plot
                    let plot_width = ui.available_width().max(280.0);
                    let plot_size = egui::vec2(plot_width, 180.0);
                    let (rect, response) = ui.allocate_exact_size(plot_size, egui::Sense::click_and_drag());
                    let painter = ui.painter_at(rect);

                    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
                    painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                    let grid_line_color = crate::theme::colors::grid_line(is_dark);

                    // Grid lines
                    for i in 1..4 {
                        let gx = rect.min.x + (rect.width() * (i as f32 / 4.0));
                        let gy = rect.min.y + (rect.height() * (i as f32 / 4.0));
                        painter.line_segment([egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)], egui::Stroke::new(0.5, grid_line_color));
                        painter.line_segment([egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)], egui::Stroke::new(0.5, grid_line_color));
                    }

                    let max_val = current_curve.iter().copied().max().unwrap_or(100).max(1) as f32;
                    let count = current_curve.len().max(1);

                    let mut points = Vec::with_capacity(count);
                    for (idx, &val) in current_curve.iter().enumerate() {
                        let px = rect.min.x + (idx as f32 / (count.max(2) - 1) as f32) * (rect.width() - 24.0) + 12.0;
                        let py = rect.max.y - (val as f32 / max_val) * (rect.height() - 24.0) - 12.0;
                        points.push(egui::pos2(px, py));
                    }

                    // Fill area under curve
                    let fill_poly_col = if is_dark {
                        egui::Color32::from_rgba_unmultiplied(200, 140, 60, 30)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(217, 119, 6, 30)
                    };
                    if points.len() >= 2 {
                        let mut mesh_points = Vec::new();
                        for pt in &points {
                            mesh_points.push(*pt);
                        }
                        mesh_points.push(egui::pos2(rect.max.x - 12.0, rect.max.y - 12.0));
                        mesh_points.push(egui::pos2(rect.min.x + 12.0, rect.max.y - 12.0));
                        painter.add(egui::Shape::convex_polygon(mesh_points, fill_poly_col, egui::Stroke::NONE));
                    }

                    // Line segments
                    let line_stroke_col = crate::theme::colors::stat_atk(is_dark);
                    for w in points.windows(2) {
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.5, line_stroke_col));
                    }

                    // Drag-and-drop sculpting
                    if response.dragged() || response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let rel_x = (pos.x - rect.min.x - 12.0) / (rect.width() - 24.0);
                            let lvl_idx = ((rel_x * (count - 1) as f32).round() as usize).clamp(0, count - 1);
                            let rel_y = (rect.max.y - 12.0 - pos.y) / (rect.height() - 24.0);
                            let new_val = (rel_y.clamp(0.0, 1.0) * max_val).round() as i32;
                            if let Some(val) = current_curve.get_mut(lvl_idx) {
                                *val = new_val.clamp(1, 32767) as i16;
                                *dirty = true;
                            }
                        }
                    }

                    // Hover tooltip
                    if let Some(hover_pos) = response.hover_pos() {
                        if rect.contains(hover_pos) {
                            let rel_x = (hover_pos.x - rect.min.x - 12.0) / (rect.width() - 24.0);
                            let hover_lvl = ((rel_x * (count - 1) as f32).round() as usize).clamp(0, count - 1);
                            if let Some(&val) = current_curve.get(hover_lvl) {
                                let pt = points[hover_lvl];
                                painter.circle_filled(pt, 5.0, egui::Color32::from_rgb(255, 220, 0));
                                response.on_hover_text(format!("Level {}: {} (Drag to modify)", hover_lvl + 1, val));
                            }
                        }
                    }
                });
            });

            ui.separator();
            crate::views::database::actors::render_resistance_tables(ui, &mut class.state_ranks, &mut class.attribute_ranks, states, attributes, dirty);
        });
}

