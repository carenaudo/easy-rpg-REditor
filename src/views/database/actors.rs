use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::{ActorInfo, AttributeInfo, ClassInfo, ItemInfo, SkillInfo, StateInfo};
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StatCurveType {
    MaxHp,
    MaxSp,
    Attack,
    Defense,
    Spirit,
    Agility,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GrowthCurvePreset {
    EarlyBloomer,
    Linear,
    LateBloomer,
    SCurve,
}

pub struct ActorViewState {
    pub active_stat_curve: StatCurveType,
    pub curve_v1: i32,
    pub curve_vmax: i32,
    pub curve_preset: GrowthCurvePreset,
    pub dragging_level_idx: Option<usize>,
}

impl Default for ActorViewState {
    fn default() -> Self {
        Self {
            active_stat_curve: StatCurveType::MaxHp,
            curve_v1: 500,
            curve_vmax: 5000,
            curve_preset: GrowthCurvePreset::Linear,
            dragging_level_idx: None,
        }
    }
}

pub fn generate_growth_curve(v1: i32, vmax: i32, max_lvl: i32, preset: GrowthCurvePreset) -> Vec<i16> {
    let count = (max_lvl.clamp(1, 99)) as usize;
    let mut result = Vec::with_capacity(count);

    for lvl in 1..=count {
        if count <= 1 {
            result.push(v1 as i16);
            continue;
        }

        let t = (lvl - 1) as f32 / (count - 1) as f32; // 0.0 to 1.0
        let factor = match preset {
            GrowthCurvePreset::Linear => t,
            GrowthCurvePreset::EarlyBloomer => t.powf(0.55), // concave down (fast early growth)
            GrowthCurvePreset::LateBloomer => t.powf(1.85),  // concave up (fast late growth)
            GrowthCurvePreset::SCurve => 3.0 * t * t - 2.0 * t * t * t,
        };

        let val = (v1 as f32 + (vmax - v1) as f32 * factor).round() as i32;
        result.push(val.clamp(1, 32767) as i16);
    }

    result
}

pub fn show_actor_form(
    ui: &mut egui::Ui,
    actor: &mut ActorInfo,
    project_path: Option<&str>,
    items: &[ItemInfo],
    skills: &[SkillInfo],
    classes: &[ClassInfo],
    states: &[StateInfo],
    attributes: &[AttributeInfo],
    picker: &mut AssetPickerState,
    cache: &mut AssetPreviewCache,
    view_state: &mut ActorViewState,
    dirty: &mut bool,
) {
    let is_dark = ui.visuals().dark_mode;
    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("👤 {:04}: {}", actor.id, actor.name));
        ui.separator();
        let lvl_col = crate::theme::colors::stat_sp(is_dark);
        ui.colored_label(lvl_col, format!("Level {} – {}", actor.initial_level, actor.final_level));
        if actor.two_weapon {
            let col = crate::theme::colors::stat_atk(is_dark);
            ui.colored_label(col, "⚔ Dual Wield");
        }
        if actor.auto_battle {
            let col = crate::theme::colors::info(is_dark);
            ui.colored_label(col, "🤖 Auto Battle");
        }
        if actor.super_guard {
            let col = crate::theme::colors::warning(is_dark);
            ui.colored_label(col, "🛡 Super Guard");
        }
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 1200.0 { 3 } else { 2 };

    egui::ScrollArea::vertical()
        .id_salt("actor_editor_scroll")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: Identity, Visuals & Equipment
                cols[0].group(|ui| {
                    ui.heading("General Details");
                    egui::Grid::new("actor_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            let name_edit = ui.text_edit_singleline(&mut actor.name);
                            if name_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Title:");
                            let title_edit = ui.text_edit_singleline(&mut actor.title);
                            if title_edit.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Class (2003):");
                            egui::ComboBox::from_id_salt("actor_class_combo")
                                .selected_text(
                                    if actor.class_id == 0 {
                                        "(None)".to_string()
                                    } else {
                                        classes.iter().find(|c| c.id == actor.class_id).map(|c| format!("{:03}: {}", c.id, c.name)).unwrap_or_else(|| format!("Class {}", actor.class_id))
                                    }
                                )
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut actor.class_id, 0, "(None)").clicked() { *dirty = true; }
                                    for c in classes {
                                        if ui.selectable_value(&mut actor.class_id, c.id, format!("{:03}: {}", c.id, c.name)).clicked() {
                                            *dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Initial Level:");
                            let init_lvl = ui.add(egui::DragValue::new(&mut actor.initial_level).range(1..=99));
                            if init_lvl.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Final Level:");
                            let fin_lvl = ui.add(egui::DragValue::new(&mut actor.final_level).range(1..=99));
                            if fin_lvl.changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Battler Animation ID:");
                            if ui.add(egui::DragValue::new(&mut actor.battler_animation).range(0..=500)).on_hover_text("RPG2003 Battle Animation / Sprite layout").changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Graphics & FaceSet");
                    ui.horizontal(|ui| {
                        // Character sprite preview with live walking animation and checkerboard
                        ui.vertical(|ui| {
                            ui.label("CharSet:");
                            let btn_text = if actor.character_name.is_empty() { "(None)".to_string() } else { format!("{} #{}", actor.character_name, actor.character_index) };
                            if ui.button(btn_text).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "CharSet", &actor.character_name, actor.character_index);
                                }
                            }

                            let frame_sz = egui::vec2(48.0, 64.0);
                            let (rect, resp) = ui.allocate_exact_size(frame_sz, egui::Sense::hover());
                            let painter = ui.painter_at(rect);
                            draw_checkerboard(&painter, rect, 8.0, is_dark);
                            painter.rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                            if let Some(proj) = project_path {
                                if !actor.character_name.is_empty() {
                                    if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "CharSet", &actor.character_name) {
                                        let step = (ui.input(|i| i.time * 4.0).floor() as usize) % 4;
                                        let anim_frame = match step { 0 => 0, 1 => 1, 2 => 2, _ => 1 };
                                        let dir_cycle = (ui.input(|i| (i.time * 4.0 / 4.0).floor() as usize)) % 4;
                                        let dir = match dir_cycle { 0 => 0, 1 => 1, 2 => 3, _ => 2 };

                                        let char_idx = (actor.character_index.clamp(0, 7)) as usize;
                                        let char_col = char_idx % 4;
                                        let char_row = char_idx / 4;

                                        let u0 = (char_col as f32 * 72.0 + anim_frame as f32 * 24.0) / 288.0;
                                        let u1 = u0 + (24.0 / 288.0);
                                        let v0 = (char_row as f32 * 128.0 + dir as f32 * 32.0) / 256.0;
                                        let v1 = v0 + (32.0 / 256.0);

                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::WHITE);
                                        ui.ctx().request_repaint();
                                    }
                                }
                            }
                            resp.on_hover_text("Live animated walking preview (rotates 360° every full step cycle)");
                        });

                        ui.separator();

                        // FaceSet preview
                        ui.vertical(|ui| {
                            ui.label("FaceSet:");
                            let face_text = if actor.face_name.is_empty() { "(None)".to_string() } else { format!("{} #{}", actor.face_name, actor.face_index) };
                            if ui.button(face_text).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "FaceSet", &actor.face_name, actor.face_index);
                                }
                            }

                            let face_sz = egui::vec2(64.0, 64.0);
                            let (rect, _) = ui.allocate_exact_size(face_sz, egui::Sense::hover());
                            let painter = ui.painter_at(rect);
                            draw_checkerboard(&painter, rect, 8.0, is_dark);
                            painter.rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                            if let Some(proj) = project_path {
                                if !actor.face_name.is_empty() {
                                    if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "FaceSet", &actor.face_name) {
                                        let face_idx = (actor.face_index.clamp(0, 15)) as usize;
                                        let col = face_idx % 4;
                                        let row = face_idx / 4;
                                        let u0 = col as f32 / 4.0;
                                        let u1 = (col + 1) as f32 / 4.0;
                                        let v0 = row as f32 / 4.0;
                                        let v1 = (row + 1) as f32 / 4.0;

                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::WHITE);
                                    }
                                }
                            }
                        });
                    });

                    ui.separator();
                    ui.heading("Starting Equipment");
                    egui::Grid::new("actor_equip_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            let equip_slot = |ui: &mut egui::Ui, label: &str, eq_id: &mut i32, dirty: &mut bool| {
                                ui.label(label);
                                egui::ComboBox::from_id_salt(format!("actor_{}", label))
                                    .selected_text(
                                        if *eq_id == 0 {
                                            "(None)".to_string()
                                        } else {
                                            items.iter().find(|i| i.id == *eq_id).map(|i| format!("{:03}: {}", i.id, i.name)).unwrap_or_else(|| format!("ID {}", eq_id))
                                        }
                                    )
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_value(eq_id, 0, "(None)").clicked() { *dirty = true; }
                                        for item in items {
                                            if ui.selectable_value(eq_id, item.id, format!("{:03}: {}", item.id, item.name)).clicked() {
                                                *dirty = true;
                                            }
                                        }
                                    });
                                ui.end_row();
                            };

                            equip_slot(ui, "Weapon:", &mut actor.weapon_id, dirty);
                            equip_slot(ui, "Shield:", &mut actor.shield_id, dirty);
                            equip_slot(ui, "Armor:", &mut actor.armor_id, dirty);
                            equip_slot(ui, "Helmet:", &mut actor.helmet_id, dirty);
                            equip_slot(ui, "Accessory:", &mut actor.accessory_id, dirty);
                        });

                    ui.separator();
                    ui.heading("Special Combat Traits");
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut actor.two_weapon, "Dual Wield (Two Weapons)").changed() { *dirty = true; }
                        if ui.checkbox(&mut actor.lock_equipment, "Lock Equipment").changed() { *dirty = true; }
                        if ui.checkbox(&mut actor.auto_battle, "Auto Battle (AI controlled)").changed() { *dirty = true; }
                        if ui.checkbox(&mut actor.super_guard, "Super Guard (Quarter Damage)").changed() { *dirty = true; }
                    });
                });

                // Column 2: Stat Growth Studio (Interactive Canvas)
                cols[1].group(|ui| {
                    ui.heading("Stat Growth Studio");
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::MaxHp, "Max HP");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::MaxSp, "Max SP");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Attack, "Attack");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Defense, "Defense");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Spirit, "Spirit");
                        ui.selectable_value(&mut view_state.active_stat_curve, StatCurveType::Agility, "Agility");
                    });

                    let current_curve: &mut Vec<i16> = match view_state.active_stat_curve {
                        StatCurveType::MaxHp => &mut actor.param_maxhp,
                        StatCurveType::MaxSp => &mut actor.param_maxsp,
                        StatCurveType::Attack => &mut actor.param_attack,
                        StatCurveType::Defense => &mut actor.param_defense,
                        StatCurveType::Spirit => &mut actor.param_spirit,
                        StatCurveType::Agility => &mut actor.param_agility,
                    };

                    if current_curve.len() < actor.final_level as usize {
                        let default_val = match view_state.active_stat_curve {
                            StatCurveType::MaxHp => 500,
                            StatCurveType::MaxSp => 100,
                            _ => 20,
                        };
                        current_curve.resize(actor.final_level as usize, default_val);
                    }

                    // Curve Formula Toolbar
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Lv 1:");
                            ui.add(egui::DragValue::new(&mut view_state.curve_v1).range(1..=9999));

                            ui.label("Lv Max:");
                            ui.add(egui::DragValue::new(&mut view_state.curve_vmax).range(1..=9999));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::EarlyBloomer, "🌱 Early");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::Linear, "📏 Linear");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::LateBloomer, "🚀 Late");
                            ui.selectable_value(&mut view_state.curve_preset, GrowthCurvePreset::SCurve, "〰 S-Curve");

                            if ui.button("⚡ Apply Preset").clicked() {
                                *current_curve = generate_growth_curve(view_state.curve_v1, view_state.curve_vmax, actor.final_level, view_state.curve_preset);
                                *dirty = true;
                            }
                        });
                    });

                    // Interactive Stat Curve Canvas
                    let plot_width = ui.available_width().max(280.0);
                    let plot_size = egui::vec2(plot_width, 180.0);
                    let (rect, response) = ui.allocate_exact_size(plot_size, egui::Sense::click_and_drag());
                    let painter = ui.painter_at(rect);

                    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
                    painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                    let grid_line_color = crate::theme::colors::grid_line(is_dark);

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

                    let fill_poly_col = if is_dark {
                        egui::Color32::from_rgba_unmultiplied(60, 140, 220, 30)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(37, 99, 235, 30)
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

                    let line_stroke_col = crate::theme::colors::stat_sp(is_dark);
                    for w in points.windows(2) {
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.5, line_stroke_col));
                    }

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

                    if let Some(hover_pos) = response.hover_pos() {
                        if rect.contains(hover_pos) {
                            let rel_x = (hover_pos.x - rect.min.x - 12.0) / (rect.width() - 24.0);
                            let hover_lvl = ((rel_x * (count - 1) as f32).round() as usize).clamp(0, count - 1);
                            if let Some(&val) = current_curve.get(hover_lvl) {
                                let pt = points[hover_lvl];
                                let dot_col = if is_dark { egui::Color32::from_rgb(255, 220, 0) } else { egui::Color32::from_rgb(217, 119, 6) };
                                painter.circle_filled(pt, 5.0, dot_col);
                                response.on_hover_text(format!("Level {}: {} (Drag to modify)", hover_lvl + 1, val));
                            }
                        }
                    }

                    if num_cols == 2 {
                        ui.separator();
                        render_skills_table(ui, actor, skills, dirty);
                    }
                });

                // Column 3: Learned Skills & Resistance Matrices (or Column 2 if 2 cols)
                if num_cols == 3 {
                    cols[2].group(|ui| {
                        render_skills_table(ui, actor, skills, dirty);
                    });
                }
            });

            ui.separator();
            render_resistance_tables(ui, &mut actor.state_ranks, &mut actor.attribute_ranks, states, attributes, dirty);
        });
}

pub fn render_resistance_tables(
    ui: &mut egui::Ui,
    state_ranks: &mut Vec<u8>,
    attr_ranks: &mut Vec<u8>,
    states: &[StateInfo],
    attributes: &[AttributeInfo],
    dirty: &mut bool,
) {
    let is_dark = ui.visuals().dark_mode;
    let col_a = crate::theme::colors::rank_a(is_dark);
    let col_b = crate::theme::colors::rank_b(is_dark);
    let col_c = crate::theme::colors::rank_c(is_dark);
    let col_d = crate::theme::colors::rank_d(is_dark);
    let col_e = crate::theme::colors::rank_e(is_dark);

    ui.group(|ui| {
        ui.heading("🛡 State & Attribute Resistance Matrices");
        ui.columns(2, |cols| {
            // Left: State Resistances
            cols[0].group(|ui| {
                ui.heading(format!("States Susceptibility ({})", states.len()));
                if states.is_empty() {
                    ui.label("(No states defined)");
                } else {
                    if state_ranks.len() < states.len() {
                        state_ranks.resize(states.len(), 2); // Default rank C (index 2)
                    }
                    egui::Grid::new("state_ranks_grid")
                        .num_columns(6)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("State");
                            ui.colored_label(col_a, "A (100%)");
                            ui.colored_label(col_b, "B (80%)");
                            ui.colored_label(col_c, "C (60%)");
                            ui.colored_label(col_d, "D (30%)");
                            ui.colored_label(col_e, "E (0%)");
                            ui.end_row();

                            for (idx, state) in states.iter().enumerate() {
                                ui.label(format!("{:02}: {}", state.id, state.name));
                                let current_rank = state_ranks.get_mut(idx).unwrap();
                                for r in 0..=4 {
                                    if ui.selectable_label(*current_rank == r, match r { 0 => "A", 1 => "B", 2 => "C", 3 => "D", _ => "E" }).clicked() {
                                        *current_rank = r;
                                        *dirty = true;
                                    }
                                }
                                ui.end_row();
                            }
                        });
                }
            });

            // Right: Attribute / Element Resistances
            cols[1].group(|ui| {
                ui.heading(format!("Attribute / Element Resistances ({})", attributes.len()));
                if attributes.is_empty() {
                    ui.label("(No attributes defined)");
                } else {
                    if attr_ranks.len() < attributes.len() {
                        attr_ranks.resize(attributes.len(), 2); // Default rank C (index 2)
                    }
                    egui::Grid::new("attr_ranks_grid")
                        .num_columns(6)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Element");
                            ui.colored_label(col_a, "A (200%)");
                            ui.colored_label(col_b, "B (150%)");
                            ui.colored_label(col_c, "C (100%)");
                            ui.colored_label(col_d, "D (50%)");
                            ui.colored_label(col_e, "E (0%)");
                            ui.end_row();

                            for (idx, attr) in attributes.iter().enumerate() {
                                ui.label(format!("{:02}: {}", attr.id, attr.name));
                                let current_rank = attr_ranks.get_mut(idx).unwrap();
                                for r in 0..=4 {
                                    if ui.selectable_label(*current_rank == r, match r { 0 => "A", 1 => "B", 2 => "C", 3 => "D", _ => "E" }).clicked() {
                                        *current_rank = r;
                                        *dirty = true;
                                    }
                                }
                                ui.end_row();
                            }
                        });
                }
            });
        });
    });
}

fn render_skills_table(
    ui: &mut egui::Ui,
    actor: &mut ActorInfo,
    skills: &[SkillInfo],
    dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.heading(format!("Skill Learning Table ({})", actor.skills.len()));
        if ui.small_button("➕ Add Skill").clicked() {
            let sid = skills.first().map(|s| s.id).unwrap_or(1);
            actor.skills.push((1, sid));
            *dirty = true;
        }
    });

    egui::ScrollArea::vertical()
        .id_salt("actor_skills_scroll_area")
        .show(ui, |ui| {
            let mut remove_idx = None;
            for (i, (lvl, skill_id)) in actor.skills.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("Lv:");
                    let lvl_edit = ui.add(egui::DragValue::new(lvl).range(1..=99));
                    if lvl_edit.changed() { *dirty = true; }

                    egui::ComboBox::from_id_salt(format!("actor_skill_combo_{}", i))
                        .selected_text(
                            skills.iter().find(|s| s.id == *skill_id)
                                .map(|s| format!("{:03}: {}", s.id, s.name))
                                .unwrap_or_else(|| format!("ID {}", skill_id))
                        )
                        .show_ui(ui, |ui| {
                            for s in skills {
                                if ui.selectable_value(skill_id, s.id, format!("{:03}: {}", s.id, s.name)).clicked() {
                                    *dirty = true;
                                }
                            }
                        });

                    if ui.small_button("🗑").clicked() {
                        remove_idx = Some(i);
                    }
                });
                ui.add_space(2.0);
            }

            if let Some(idx) = remove_idx {
                actor.skills.remove(idx);
                *dirty = true;
            }
        });
}
