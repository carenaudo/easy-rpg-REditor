use eframe::egui;
use crate::dialogs::event_command_dialog::EventCommandDialogState;
use crate::lcf_bridge::{event_command_label, EnemyInfo, TerrainInfo, TroopInfo, TroopMemberInfo, TroopPageConditionInfo, TroopPageInfo};
use crate::widgets::asset_viewer::AssetPreviewCache;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BattlePerspective {
    FrontView2000,
    SideView2003,
}

pub struct TroopViewState {
    pub perspective: BattlePerspective,
    pub selected_member_idx: Option<usize>,
    pub is_dragging: bool,
    pub background_name: String,
    pub active_page_idx: usize,
    pub selected_cmd_idx: Option<usize>,
}

impl Default for TroopViewState {
    fn default() -> Self {
        Self {
            perspective: BattlePerspective::FrontView2000,
            selected_member_idx: None,
            is_dragging: false,
            background_name: "Grassland".to_string(),
            active_page_idx: 0,
            selected_cmd_idx: None,
        }
    }
}

pub fn show_troop_form(
    ui: &mut egui::Ui,
    troop: &mut TroopInfo,
    project_path: Option<&str>,
    enemies: &[EnemyInfo],
    terrains: &[TerrainInfo],
    cache: &mut AssetPreviewCache,
    state: &mut TroopViewState,
    cmd_dialog: &mut EventCommandDialogState,
    dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.heading(format!("🐺 {:04}: {}", troop.id, troop.name));
        ui.separator();

        ui.label("Name:");
        let name_edit = ui.text_edit_singleline(&mut troop.name);
        if name_edit.changed() {
            *dirty = true;
        }

        ui.separator();
        ui.label("Perspective:");
        ui.selectable_value(&mut state.perspective, BattlePerspective::FrontView2000, "⚔ Front-View (2000)");
        ui.selectable_value(&mut state.perspective, BattlePerspective::SideView2003, "🛡 Side-View (2003)");
    });

    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 1000.0 { 2 } else { 1 };

    ui.columns(num_cols, |cols| {
        // Left Column: Interactive 2D Battlefield Canvas
        cols[0].group(|ui| {
            let is_dark = ui.visuals().dark_mode;
            ui.horizontal(|ui| {
                ui.heading("Battlefield Stage");
                let hint_col = crate::theme::colors::muted(is_dark);
                ui.label(egui::RichText::new("(Drag monsters to reposition)").italics().color(hint_col));
            });

            let canvas_width = ui.available_width().max(360.0);
            let canvas_size = egui::vec2(canvas_width, 280.0);
            let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            // 1. Draw Battlefield Background
            let pers_id = match state.perspective {
                BattlePerspective::FrontView2000 => 0,
                BattlePerspective::SideView2003 => 1,
            };
            let bg_color = crate::theme::colors::troop_stage_bg(pers_id, is_dark);
            painter.rect_filled(rect, 4.0, bg_color);
            painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

            // Ground perspective lines
            let ground_y = rect.min.y + rect.height() * 0.72;
            let guide_line_col = crate::theme::colors::grid_line(is_dark);
            painter.line_segment(
                [egui::pos2(rect.min.x, ground_y), egui::pos2(rect.max.x, ground_y)],
                egui::Stroke::new(1.0, guide_line_col),
            );

            // Side-View 2003 party slot guides on the right
            if state.perspective == BattlePerspective::SideView2003 {
                let party_x = rect.max.x - 60.0;
                let slot_line_col = if is_dark {
                    egui::Color32::from_rgba_unmultiplied(100, 200, 255, 50)
                } else {
                    egui::Color32::from_rgba_unmultiplied(37, 99, 235, 60)
                };
                painter.line_segment(
                    [egui::pos2(party_x, rect.min.y + 20.0), egui::pos2(party_x, rect.max.y - 20.0)],
                    egui::Stroke::new(1.0, slot_line_col),
                );

                for slot in 0..4 {
                    let slot_y = rect.min.y + 50.0 + (slot as f32) * 50.0;
                    let circle_bg = if is_dark {
                        egui::Color32::from_rgb(45, 90, 150)
                    } else {
                        egui::Color32::from_rgb(200, 220, 245)
                    };
                    let text_fg = if is_dark {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(15, 23, 42)
                    };
                    painter.circle_filled(egui::pos2(party_x + 20.0, slot_y), 14.0, circle_bg);
                    painter.text(
                        egui::pos2(party_x + 20.0, slot_y),
                        egui::Align2::CENTER_CENTER,
                        format!("H{}", slot + 1),
                        egui::FontId::proportional(11.0),
                        text_fg,
                    );
                }
            } else {
                // Front-View 2000 status HUD preview bar at bottom
                let hud_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.max.y - 40.0),
                    rect.max,
                );
                let hud_bg = if is_dark {
                    egui::Color32::from_rgba_unmultiplied(10, 16, 24, 200)
                } else {
                    egui::Color32::from_rgba_unmultiplied(220, 228, 238, 220)
                };
                let hud_fg = if is_dark {
                    egui::Color32::from_rgb(130, 150, 170)
                } else {
                    egui::Color32::from_rgb(50, 65, 85)
                };
                painter.rect_filled(hud_rect, 0.0, hud_bg);
                painter.text(
                    hud_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Party Status HUD Area",
                    egui::FontId::proportional(12.0),
                    hud_fg,
                );
            }

            // 2. Draw & Interact with Troop Monster Members
            let scale_x = rect.width() / 320.0;
            let scale_y = rect.height() / 240.0;

            let pointer_pos = response.interact_pointer_pos();

            if response.drag_started() {
                if let Some(pos) = pointer_pos {
                    state.selected_member_idx = None;
                    for (i, member) in troop.members.iter().enumerate().rev() {
                        let mx = rect.min.x + (member.x as f32) * scale_x;
                        let my = rect.min.y + (member.y as f32) * scale_y;
                        let m_rect = egui::Rect::from_center_size(egui::pos2(mx, my), egui::vec2(54.0, 54.0));
                        if m_rect.contains(pos) {
                            state.selected_member_idx = Some(i);
                            state.is_dragging = true;
                            break;
                        }
                    }
                }
            }

            if response.dragged() && state.is_dragging {
                if let (Some(sel_idx), Some(pos)) = (state.selected_member_idx, pointer_pos) {
                    if let Some(member) = troop.members.get_mut(sel_idx) {
                        let new_x = ((pos.x - rect.min.x) / scale_x).round() as i32;
                        let new_y = ((pos.y - rect.min.y) / scale_y).round() as i32;
                        member.x = new_x.clamp(16, 304);
                        member.y = new_y.clamp(16, 224);
                        *dirty = true;
                    }
                }
            }

            if response.drag_stopped() {
                state.is_dragging = false;
            }

            // Render Monster Sprites
            for (i, member) in troop.members.iter().enumerate() {
                let mx = rect.min.x + (member.x as f32) * scale_x;
                let my = rect.min.y + (member.y as f32) * scale_y;
                let center = egui::pos2(mx, my);

                let enemy_opt = enemies.iter().find(|e| e.id == member.enemy_id);
                let enemy_name = enemy_opt.map(|e| e.name.as_str()).unwrap_or("Unknown");
                let battler_name = enemy_opt.map(|e| e.battler_name.as_str()).unwrap_or("");

                let is_selected = state.selected_member_idx == Some(i);

                let mut drawn_image = false;
                if let Some(proj) = project_path {
                    if !battler_name.is_empty() {
                        if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "Monster", battler_name) {
                            let tex_sz = tex.size_vec2();
                            let render_sz = egui::vec2(tex_sz.x * scale_x * 0.85, tex_sz.y * scale_y * 0.85);
                            let img_rect = egui::Rect::from_center_size(center, render_sz);
                            painter.image(tex.id(), img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                            drawn_image = true;
                        }
                    }
                }

                if !drawn_image {
                    let box_rect = egui::Rect::from_center_size(center, egui::vec2(40.0, 40.0));
                    painter.rect_filled(box_rect, 4.0, egui::Color32::from_rgb(180, 50, 50));
                    painter.text(center, egui::Align2::CENTER_CENTER, format!("{}", i + 1), egui::FontId::proportional(14.0), egui::Color32::WHITE);
                }

                // Selection highlight
                if is_selected {
                    let sel_rect = egui::Rect::from_center_size(center, egui::vec2(58.0, 58.0));
                    painter.rect_stroke(sel_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 0)), egui::StrokeKind::Outside);
                }

                // Label tag
                let tag_pos = egui::pos2(center.x, center.y + 26.0);
                painter.text(
                    tag_pos,
                    egui::Align2::CENTER_TOP,
                    format!("{}. {}", i + 1, enemy_name),
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(220, 230, 245),
                );
            }

            // Stage Alignment Tools
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("🎯 Center All").clicked() {
                    let total = troop.members.len();
                    for (i, m) in troop.members.iter_mut().enumerate() {
                        let spacing = 240.0 / (total as f32 + 1.0);
                        m.x = (40.0 + (i as f32 + 1.0) * spacing) as i32;
                        m.y = 120;
                    }
                    *dirty = true;
                }

                if ui.button("↔ Disperse").clicked() {
                    let total = troop.members.len();
                    if total > 1 {
                        let step = 200 / (total - 1) as i32;
                        for (i, m) in troop.members.iter_mut().enumerate() {
                            m.x = 60 + (i as i32) * step;
                        }
                        *dirty = true;
                    }
                }

                if ui.button("⬇ Align Ground").clicked() {
                    for m in troop.members.iter_mut() {
                        m.y = 150;
                    }
                    *dirty = true;
                }
            });

            ui.separator();
            ui.collapsing("🌍 Battle Terrain Restrictions", |ui| {
                if troop.terrain_set.len() < terrains.len() {
                    troop.terrain_set.resize(terrains.len(), true);
                }
                ui.horizontal_wrapped(|ui| {
                    for (t_idx, t) in terrains.iter().enumerate() {
                        let is_allowed = troop.terrain_set.get_mut(t_idx).unwrap();
                        if ui.checkbox(is_allowed, format!("{:02}: {}", t.id, t.name)).changed() {
                            *dirty = true;
                        }
                    }
                });
            });
        });

        // Right Column: Monster Members & Event Pages
        let right_col = &mut cols[1];
        right_col.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Troop Members ({})", troop.members.len()));
                if ui.small_button("➕ Add Monster").clicked() {
                    let new_enemy_id = enemies.first().map(|e| e.id).unwrap_or(1);
                    troop.members.push(TroopMemberInfo {
                        enemy_id: new_enemy_id,
                        x: 160,
                        y: 120,
                        invisible: false,
                    });
                    state.selected_member_idx = Some(troop.members.len() - 1);
                    *dirty = true;
                }
                if let Some(sel) = state.selected_member_idx {
                    if sel < troop.members.len() && ui.small_button("🗑 Remove").clicked() {
                        troop.members.remove(sel);
                        state.selected_member_idx = None;
                        *dirty = true;
                    }
                }
            });

            egui::ScrollArea::vertical()
                .id_salt("troop_members_table_scroll")
                .max_height(140.0)
                .show(ui, |ui| {
                    egui::Grid::new("troop_members_edit_grid")
                        .num_columns(6)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Slot");
                            ui.label("Monster");
                            ui.label("X");
                            ui.label("Y");
                            ui.label("Hidden");
                            ui.label("");
                            ui.end_row();

                            let mut remove_idx = None;
                            for (i, member) in troop.members.iter_mut().enumerate() {
                                let is_sel = state.selected_member_idx == Some(i);
                                if ui.selectable_label(is_sel, format!("#{}", i + 1)).clicked() {
                                    state.selected_member_idx = Some(i);
                                }

                                egui::ComboBox::from_id_salt(format!("member_enemy_{}", i))
                                    .selected_text(
                                        enemies.iter().find(|e| e.id == member.enemy_id)
                                            .map(|e| format!("{:03}: {}", e.id, e.name))
                                            .unwrap_or_else(|| format!("ID {}", member.enemy_id))
                                    )
                                    .show_ui(ui, |ui| {
                                        for e in enemies {
                                            if ui.selectable_value(&mut member.enemy_id, e.id, format!("{:03}: {}", e.id, e.name)).clicked() {
                                                *dirty = true;
                                            }
                                        }
                                    });

                                let x_edit = ui.add(egui::DragValue::new(&mut member.x).range(0..=320));
                                if x_edit.changed() { *dirty = true; }

                                let y_edit = ui.add(egui::DragValue::new(&mut member.y).range(0..=240));
                                if y_edit.changed() { *dirty = true; }

                                if ui.checkbox(&mut member.invisible, "").changed() { *dirty = true; }

                                if ui.small_button("🗑").clicked() {
                                    remove_idx = Some(i);
                                }

                                ui.end_row();
                            }

                            if let Some(idx) = remove_idx {
                                troop.members.remove(idx);
                                if state.selected_member_idx == Some(idx) {
                                    state.selected_member_idx = None;
                                }
                                *dirty = true;
                            }
                        });
                });

            ui.separator();

            // Tabbed Battle Event Pages
            ui.horizontal(|ui| {
                ui.heading("Battle Events");
                if ui.small_button("➕ New Page").clicked() {
                    let new_id = (troop.pages.len() + 1) as i32;
                    troop.pages.push(TroopPageInfo {
                        id: new_id,
                        condition: TroopPageConditionInfo::default(),
                        commands: Vec::new(),
                    });
                    state.active_page_idx = troop.pages.len() - 1;
                    *dirty = true;
                }
            });

            if troop.pages.is_empty() {
                troop.pages.push(TroopPageInfo {
                    id: 1,
                    condition: TroopPageConditionInfo::default(),
                    commands: Vec::new(),
                });
            }

            ui.horizontal(|ui| {
                for (p_idx, _) in troop.pages.iter().enumerate() {
                    let label = format!("Page {}", p_idx + 1);
                    if ui.selectable_label(state.active_page_idx == p_idx, label).clicked() {
                        state.active_page_idx = p_idx;
                        state.selected_cmd_idx = None;
                    }
                }
            });

            if let Some(page) = troop.pages.get_mut(state.active_page_idx) {
                ui.collapsing("⚙ Page Trigger Conditions", |ui| {
                    egui::Grid::new("troop_page_cond_grid")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            let mut switch_a_flag = (page.condition.flags & 1) != 0;
                            if ui.checkbox(&mut switch_a_flag, "Switch A ON:").changed() {
                                if switch_a_flag { page.condition.flags |= 1; } else { page.condition.flags &= !1; }
                                *dirty = true;
                            }
                            if ui.add(egui::DragValue::new(&mut page.condition.switch_a_id).range(1..=5000)).changed() { *dirty = true; }
                            ui.end_row();

                            let mut turn_flag = (page.condition.flags & 4) != 0;
                            if ui.checkbox(&mut turn_flag, "Turn Count:").changed() {
                                if turn_flag { page.condition.flags |= 4; } else { page.condition.flags &= !4; }
                                *dirty = true;
                            }
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(&mut page.condition.turn_a).range(0..=255)).changed() { *dirty = true; }
                                ui.label("+");
                                if ui.add(egui::DragValue::new(&mut page.condition.turn_b).range(0..=255)).changed() { *dirty = true; }
                                ui.label("× X");
                            });
                            ui.end_row();

                            let mut enemy_hp_flag = (page.condition.flags & 16) != 0;
                            if ui.checkbox(&mut enemy_hp_flag, "Enemy HP Range:").changed() {
                                if enemy_hp_flag { page.condition.flags |= 16; } else { page.condition.flags &= !16; }
                                *dirty = true;
                            }
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(&mut page.condition.enemy_hp_min).range(0..=100).suffix("%")).changed() { *dirty = true; }
                                ui.label("–");
                                if ui.add(egui::DragValue::new(&mut page.condition.enemy_hp_max).range(0..=100).suffix("%")).changed() { *dirty = true; }
                            });
                            ui.end_row();
                        });
                });

                ui.horizontal(|ui| {
                    if ui.small_button("➕ Add Command").clicked() {
                        cmd_dialog.open_new(0);
                    }
                });

                egui::ScrollArea::vertical()
                    .id_salt("troop_page_cmds_scroll")
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let is_dark = ui.visuals().dark_mode;
                        if page.commands.is_empty() {
                            ui.colored_label(crate::theme::colors::muted(is_dark), "(No battle events on this page)");
                        }
                        for (c_idx, cmd) in page.commands.iter().enumerate() {
                            let label = event_command_label(cmd);
                            let color = crate::lcf_bridge::event_command_color(cmd.code, is_dark);
                            let is_sel = state.selected_cmd_idx == Some(c_idx);
                            let rich = egui::RichText::new(label).color(color).monospace();
                            if ui.selectable_label(is_sel, rich).clicked() {
                                state.selected_cmd_idx = Some(c_idx);
                            }
                        }
                    });
            }
        });
    });
}
