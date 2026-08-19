use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::{ActorInfo, SystemInfo};
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

pub fn show_system_form(
    ui: &mut egui::Ui,
    sys: &mut SystemInfo,
    project_path: Option<&str>,
    picker: &mut AssetPickerState,
    cache: &mut AssetPreviewCache,
    actors: &[ActorInfo],
    dirty: &mut bool,
) {
    let is_dark = ui.visuals().dark_mode;
    ui.horizontal_wrapped(|ui| {
        ui.heading("⚙ System Configuration");
        ui.separator();
        let engine_col = crate::theme::colors::info(is_dark);
        let save_col = crate::theme::colors::muted(is_dark);
        ui.colored_label(
            engine_col,
            if sys.ldb_id == 2003 { "🎮 RPG Maker 2003" } else { "🎮 RPG Maker 2000" },
        );
        ui.colored_label(save_col, format!("💾 Save Count: {}", sys.save_count));
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 900.0 { 2 } else { 1 };

    egui::ScrollArea::vertical()
        .id_salt("system_scroll_area")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: System Graphics & Starting Party
                cols[0].group(|ui| {
                    ui.heading("🖼 System Graphics & Windowskin");
                    egui::Grid::new("system_graphics_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("System (Windowskin):");
                            let sys_btn = if sys.system_name.is_empty() { "(None)".to_string() } else { format!("🖼 {}", sys.system_name) };
                            if ui.button(sys_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "System", &sys.system_name, 0);
                                }
                            }
                            ui.end_row();

                            if sys.ldb_id == 2003 {
                                ui.label("System2 (Battle UI):");
                                let sys2_btn = if sys.system2_name.is_empty() { "(None)".to_string() } else { format!("🖼 {}", sys.system2_name) };
                                if ui.button(sys2_btn).clicked() {
                                    if let Some(proj) = project_path {
                                        picker.open(proj, "System2", &sys.system2_name, 0);
                                    }
                                }
                                ui.end_row();
                            }

                            ui.label("Title Screen Graphic:");
                            let title_btn = if sys.title_name.is_empty() { "(None)".to_string() } else { format!("🖼 {}", sys.title_name) };
                            if ui.button(title_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "Title", &sys.title_name, 0);
                                }
                            }
                            ui.end_row();

                            ui.label("Game Over Graphic:");
                            let go_btn = if sys.gameover_name.is_empty() { "(None)".to_string() } else { format!("🖼 {}", sys.gameover_name) };
                            if ui.button(go_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "GameOver", &sys.gameover_name, 0);
                                }
                            }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("⛵ Vehicle Graphics");
                    egui::Grid::new("system_vehicles_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Boat Graphic:");
                            let b_btn = if sys.boat_name.is_empty() { "(None)".to_string() } else { format!("⛵ {}", sys.boat_name) };
                            if ui.button(b_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "CharSet", &sys.boat_name, 0);
                                }
                            }
                            ui.end_row();

                            ui.label("Ship Graphic:");
                            let s_btn = if sys.ship_name.is_empty() { "(None)".to_string() } else { format!("🚢 {}", sys.ship_name) };
                            if ui.button(s_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "CharSet", &sys.ship_name, 0);
                                }
                            }
                            ui.end_row();

                            ui.label("Airship Graphic:");
                            let a_btn = if sys.airship_name.is_empty() { "(None)".to_string() } else { format!("🛸 {}", sys.airship_name) };
                            if ui.button(a_btn).clicked() {
                                if let Some(proj) = project_path {
                                    picker.open(proj, "CharSet", &sys.airship_name, 0);
                                }
                            }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("👥 Starting Party Members");
                        if ui.button("➕ Add Member").clicked() {
                            let first_actor_id = actors.first().map(|a| a.id as i16).unwrap_or(1);
                            sys.party.push(first_actor_id);
                            *dirty = true;
                        }
                    });

                    if sys.party.is_empty() {
                        ui.label("No starting heroes assigned.");
                    } else {
                        let mut to_delete = None;
                        for (idx, member_id) in sys.party.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Slot #{}:", idx + 1));
                                egui::ComboBox::from_id_salt(format!("party_slot_{}", idx))
                                    .selected_text(
                                        actors.iter().find(|a| a.id == *member_id as i32)
                                            .map(|a| format!("{:03}: {}", a.id, a.name))
                                            .unwrap_or_else(|| format!("Actor ID {}", member_id))
                                    )
                                    .show_ui(ui, |ui| {
                                        for a in actors {
                                            if ui.selectable_value(member_id, a.id as i16, format!("{:03}: {}", a.id, a.name)).clicked() {
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
                            sys.party.remove(del_idx);
                            *dirty = true;
                        }
                    }
                });

                // Column 2: Transitions, Font & BGM
                cols[1].group(|ui| {
                    ui.heading("🎬 Screen Transitions & Font");
                    egui::Grid::new("system_transitions_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            let transition_label = |t: i32| match t {
                                0 => "Fade Out/In",
                                1 => "Blocks / Mosaic",
                                2 => "Wipe Down",
                                3 => "Wipe Up",
                                4 => "Curtains Close/Open",
                                5 => "Horizontal Blinds",
                                6 => "Vertical Blinds",
                                7 => "Spiral In/Out",
                                _ => "Instant / None",
                            };

                            ui.label("Map Teleport Out:");
                            egui::ComboBox::from_id_salt("trans_out_combo")
                                .selected_text(transition_label(sys.transition_out))
                                .show_ui(ui, |ui| {
                                    for t in 0..=8 {
                                        if ui.selectable_value(&mut sys.transition_out, t, transition_label(t)).clicked() { *dirty = true; }
                                    }
                                });
                            ui.end_row();

                            ui.label("Map Teleport In:");
                            egui::ComboBox::from_id_salt("trans_in_combo")
                                .selected_text(transition_label(sys.transition_in))
                                .show_ui(ui, |ui| {
                                    for t in 0..=8 {
                                        if ui.selectable_value(&mut sys.transition_in, t, transition_label(t)).clicked() { *dirty = true; }
                                    }
                                });
                            ui.end_row();

                            ui.label("Battle Start Fade Out:");
                            egui::ComboBox::from_id_salt("battle_start_out_combo")
                                .selected_text(transition_label(sys.battle_start_fadeout))
                                .show_ui(ui, |ui| {
                                    for t in 0..=8 {
                                        if ui.selectable_value(&mut sys.battle_start_fadeout, t, transition_label(t)).clicked() { *dirty = true; }
                                    }
                                });
                            ui.end_row();

                            ui.label("Battle End Fade In:");
                            egui::ComboBox::from_id_salt("battle_end_in_combo")
                                .selected_text(transition_label(sys.battle_end_fadein))
                                .show_ui(ui, |ui| {
                                    for t in 0..=8 {
                                        if ui.selectable_value(&mut sys.battle_end_fadein, t, transition_label(t)).clicked() { *dirty = true; }
                                    }
                                });
                            ui.end_row();

                            ui.label("System Font:");
                            egui::ComboBox::from_id_salt("system_font_combo")
                                .selected_text(if sys.font_id == 1 { "Font 1 (MS Mincho)" } else { "Font 0 (MS Gothic)" })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut sys.font_id, 0, "Font 0 (MS Gothic)").clicked() { *dirty = true; }
                                    if ui.selectable_value(&mut sys.font_id, 1, "Font 1 (MS Mincho)").clicked() { *dirty = true; }
                                });
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("🎵 Background Music (BGM & ME)");
                    egui::Grid::new("system_music_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Title Screen BGM:");
                            crate::widgets::resource_dropdown::resource_combo_box(ui, "sys_bgm_title", &mut sys.title_music_name, "Music", project_path, dirty);
                            ui.end_row();

                            ui.label("Battle BGM:");
                            crate::widgets::resource_dropdown::resource_combo_box(ui, "sys_bgm_battle", &mut sys.battle_music_name, "Music", project_path, dirty);
                            ui.end_row();

                            ui.label("Victory ME:");
                            crate::widgets::resource_dropdown::resource_combo_box(ui, "sys_me_victory", &mut sys.victory_music_name, "Music", project_path, dirty);
                            ui.end_row();

                            ui.label("Game Over ME:");
                            crate::widgets::resource_dropdown::resource_combo_box(ui, "sys_me_gameover", &mut sys.gameover_music_name, "Music", project_path, dirty);
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("Preview");
                    let mut render_preview_box = |ui: &mut egui::Ui, title: &str, category: &str, file_name: &str, target_w: f32, target_h: f32| {
                        ui.label(title);
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(target_w, target_h), egui::Sense::hover());
                        let painter = ui.painter_at(rect);
                        draw_checkerboard(&painter, rect, 8.0, is_dark);
                        painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                        let mut drawn = false;
                        if let Some(proj) = project_path {
                            if !file_name.is_empty() {
                                if let Some(tex) = cache.get_or_load(ui.ctx(), proj, category, file_name) {
                                    painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                    drawn = true;
                                }
                            }
                        }

                        if !drawn {
                            painter.text(rect.center(), egui::Align2::CENTER_CENTER, format!("(No {})", title), egui::FontId::proportional(11.0), egui::Color32::GRAY);
                        }
                        ui.add_space(4.0);
                    };

                    render_preview_box(ui, "Windowskin (System):", "System", &sys.system_name, 160.0, 80.0);
                    if !sys.title_name.is_empty() {
                        render_preview_box(ui, "Title Screen:", "Title", &sys.title_name, 160.0, 120.0);
                    }
                });
            });
        });
}
