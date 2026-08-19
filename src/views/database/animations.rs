use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::{AnimationInfo, AnimationTimingInfo};
use crate::widgets::asset_viewer::AssetPreviewCache;

pub struct AnimationsView {
    pub selected_idx: usize,
    pub is_playing: bool,
    pub scrub_frame: usize,
    pub fps: f32,
    pub show_target: bool,
}

impl Default for AnimationsView {
    fn default() -> Self {
        Self {
            selected_idx: 0,
            is_playing: true,
            scrub_frame: 0,
            fps: 15.0,
            show_target: true,
        }
    }
}

impl AnimationsView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        animations: &mut Vec<AnimationInfo>,
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        if animations.is_empty() {
            ui.label("No animations in database.");
            if ui.button("+ Add Animation").clicked() {
                animations.push(AnimationInfo {
                    id: 1,
                    name: "Hit 1".to_string(),
                    animation_name: "Hit1".to_string(),
                    large: false,
                    scope: 0,
                    position: 1,
                    frame_count: 5,
                    timings: Vec::new(),
                });
                *dirty = true;
            }
            return;
        }

        ui.columns(2, |cols| {
            // Master list
            cols[0].group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Battle Animations");
                    if ui.small_button("+ Add").clicked() {
                        let new_id = (animations.len() + 1) as i32;
                        animations.push(AnimationInfo {
                            id: new_id,
                            name: format!("Animation {:04}", new_id),
                            animation_name: String::new(),
                            large: false,
                            scope: 0,
                            position: 1,
                            frame_count: 5,
                            timings: Vec::new(),
                        });
                        self.selected_idx = animations.len() - 1;
                        *dirty = true;
                    }
                    if ui.small_button("📄 Duplicate").clicked() && self.selected_idx < animations.len() {
                        let mut copy = animations[self.selected_idx].clone();
                        copy.id = (animations.len() + 1) as i32;
                        copy.name = format!("{} (Copy)", copy.name);
                        animations.push(copy);
                        self.selected_idx = animations.len() - 1;
                        *dirty = true;
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("anim_master_scroll")
                    .max_height(450.0)
                    .show(ui, |ui| {
                        for (idx, a) in animations.iter().enumerate() {
                            let label = format!("{:04}: {}", a.id, a.name);
                            if ui.selectable_label(self.selected_idx == idx, label).clicked() {
                                self.selected_idx = idx;
                            }
                        }
                    });
            });

            // Detail view
            cols[1].group(|ui| {
                if let Some(a) = animations.get_mut(self.selected_idx) {
                    ui.heading(format!("Edit Animation #{:04}: {}", a.id, a.name));

                    egui::ScrollArea::vertical()
                        .id_salt("anim_detail_scroll")
                        .max_height(550.0)
                        .show(ui, |ui| {
                            egui::Grid::new("anim_general_grid")
                                .num_columns(2)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Name:");
                                    if ui.text_edit_singleline(&mut a.name).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Battle Graphic:");
                                    ui.horizontal(|ui| {
                                        let anim_text = if a.animation_name.is_empty() { "(None)".to_string() } else { a.animation_name.clone() };
                                        if ui.button(format!("⚡ {}", anim_text)).clicked() {
                                            if let Some(proj) = project_path {
                                                picker.open(proj, "Battle", &a.animation_name, 0);
                                            }
                                        }
                                        if !a.animation_name.is_empty() && ui.small_button("✕").clicked() {
                                            a.animation_name.clear();
                                            *dirty = true;
                                        }
                                    });
                                    ui.end_row();

                                    ui.label("Scope:");
                                    egui::ComboBox::from_id_salt("anim_scope_combo")
                                        .selected_text(if a.scope == 1 { "All Targets" } else { "Single Target" })
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_value(&mut a.scope, 0, "Single Target").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut a.scope, 1, "All Targets").clicked() { *dirty = true; }
                                        });
                                    ui.end_row();

                                    ui.label("Screen Position:");
                                    egui::ComboBox::from_id_salt("anim_pos_combo")
                                        .selected_text(match a.position {
                                            0 => "Top",
                                            1 => "Center",
                                            2 => "Bottom",
                                            _ => "Screen Center",
                                        })
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_value(&mut a.position, 0, "Top").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut a.position, 1, "Center").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut a.position, 2, "Bottom").clicked() { *dirty = true; }
                                            if ui.selectable_value(&mut a.position, 3, "Screen Center").clicked() { *dirty = true; }
                                        });
                                    ui.end_row();

                                    ui.label("Frame Count:");
                                    if ui.add(egui::DragValue::new(&mut a.frame_count).range(1..=100)).changed() { *dirty = true; }
                                    ui.end_row();

                                    ui.label("Cell Resolution:");
                                    if ui.checkbox(&mut a.large, "Large Cells (128×128)").changed() { *dirty = true; }
                                    ui.end_row();
                                });

                            ui.separator();

                            if !a.animation_name.is_empty() {
                                let total_frames = if a.frame_count > 0 { a.frame_count } else { 25 };

                                // 1. Live Animation Playback Stage
                                ui.heading("🎬 Live Animation Preview");
                                ui.horizontal_wrapped(|ui| {
                                    let play_btn_text = if self.is_playing { "⏸ Pause" } else { "▶ Play" };
                                    if ui.button(play_btn_text).clicked() {
                                        self.is_playing = !self.is_playing;
                                    }
                                    if ui.small_button("⏮ Reset").clicked() {
                                        self.scrub_frame = 0;
                                    }
                                    ui.checkbox(&mut self.show_target, "🎯 Dummy Target");
                                    ui.label("Speed:");
                                    ui.add(egui::Slider::new(&mut self.fps, 4.0..=30.0).suffix(" fps"));
                                });

                                // Frame scrubber
                                let active_frame = if self.is_playing {
                                    let t = ui.input(|i| i.time);
                                    let f = ((t * self.fps as f64) as usize) % total_frames;
                                    self.scrub_frame = f;
                                    ui.ctx().request_repaint();
                                    f
                                } else {
                                    self.scrub_frame.min(total_frames.saturating_sub(1))
                                };

                                ui.horizontal(|ui| {
                                    ui.label(format!("Frame {}/{}", active_frame + 1, total_frames));
                                    let mut scrub = active_frame;
                                    if ui.add(egui::Slider::new(&mut scrub, 0..=(total_frames.saturating_sub(1)))).changed() {
                                        self.scrub_frame = scrub;
                                        self.is_playing = false;
                                    }
                                });

                                // Check if active frame has a flash timing
                                let active_flash = a.timings.iter().find(|t| t.frame == (active_frame + 1) as i32 && t.flash_scope > 0);

                                // Stage rendering canvas
                                let stage_sz = egui::vec2(220.0, 180.0);
                                let (stage_rect, _) = ui.allocate_exact_size(stage_sz, egui::Sense::hover());
                                let painter = ui.painter_at(stage_rect);

                                // Stage Background (themed vignette battlefield)
                                let is_dark = ui.visuals().dark_mode;
                                let stage_bg = if is_dark {
                                    egui::Color32::from_rgb(14, 18, 24)
                                } else {
                                    egui::Color32::from_rgb(240, 243, 246)
                                };
                                painter.rect_filled(stage_rect, 6.0, stage_bg);
                                painter.rect_stroke(stage_rect, 6.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Outside);

                                // Target crosshair / Dummy silhouette
                                if self.show_target {
                                    let center = match a.position {
                                        0 => egui::pos2(stage_rect.center().x, stage_rect.min.y + 40.0),
                                        2 => egui::pos2(stage_rect.center().x, stage_rect.max.y - 40.0),
                                        _ => stage_rect.center(),
                                    };
                                    let cross_col = if is_dark {
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 45)
                                    };
                                    let dummy_col = if is_dark {
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100)
                                    };
                                    painter.circle_stroke(center, 24.0, egui::Stroke::new(1.0, cross_col));
                                    painter.line_segment([egui::pos2(center.x - 30.0, center.y), egui::pos2(center.x + 30.0, center.y)], egui::Stroke::new(1.0, cross_col));
                                    painter.line_segment([egui::pos2(center.x, center.y - 30.0), egui::pos2(center.x, center.y + 30.0)], egui::Stroke::new(1.0, cross_col));
                                    painter.text(egui::pos2(center.x, center.y + 4.0), egui::Align2::CENTER_CENTER, "👾", egui::FontId::proportional(22.0), dummy_col);
                                }

                                // Draw animation frame
                                if let Some(proj) = project_path {
                                    let tex_opt = cache.get_or_load(ui.ctx(), proj, "Battle", &a.animation_name)
                                        .or_else(|| cache.get_or_load(ui.ctx(), proj, "Battle2", &a.animation_name));

                                    if let Some(tex) = tex_opt {
                                        let cell_idx = active_frame % 25;
                                        let cell_col = (cell_idx % 5) as f32;
                                        let cell_row = (cell_idx / 5) as f32;

                                        let u0 = cell_col / 5.0;
                                        let u1 = (cell_col + 1.0) / 5.0;
                                        let v0 = cell_row / 5.0;
                                        let v1 = (cell_row + 1.0) / 5.0;

                                        let cell_sz = if a.large { 128.0 } else { 96.0 };
                                        let center = match a.position {
                                            0 => egui::pos2(stage_rect.center().x, stage_rect.min.y + 40.0),
                                            2 => egui::pos2(stage_rect.center().x, stage_rect.max.y - 40.0),
                                            _ => stage_rect.center(),
                                        };
                                        let draw_rect = egui::Rect::from_center_size(center, egui::vec2(cell_sz, cell_sz));
                                        painter.image(tex.id(), draw_rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::WHITE);
                                    }
                                }

                                // Screen Flash Overlay if active
                                if let Some(flash) = active_flash {
                                    let r = (flash.flash_red * 255 / 31).clamp(0, 255) as u8;
                                    let g = (flash.flash_green * 255 / 31).clamp(0, 255) as u8;
                                    let b = (flash.flash_blue * 255 / 31).clamp(0, 255) as u8;
                                    let alpha = (flash.flash_power * 180 / 31).clamp(20, 200) as u8;
                                    painter.rect_filled(stage_rect, 6.0, egui::Color32::from_rgba_unmultiplied(r, g, b, alpha));
                                }

                                ui.separator();

                                // 2. Sound Effects & Screen Flash Timings Table
                                ui.horizontal(|ui| {
                                    ui.heading(format!("⚡ Sound & Flash Timing Cues ({})", a.timings.len()));
                                    if ui.button("➕ Add Timing").clicked() {
                                        a.timings.push(AnimationTimingInfo {
                                            id: (a.timings.len() + 1) as i32,
                                            frame: (active_frame + 1) as i32,
                                            se_name: "Blow1".to_string(),
                                            flash_scope: 1,
                                            flash_red: 31,
                                            flash_green: 31,
                                            flash_blue: 31,
                                            flash_power: 31,
                                            screen_shake: 0,
                                        });
                                        *dirty = true;
                                    }
                                });

                                if a.timings.is_empty() {
                                    ui.label("No sound or flash cues defined for this animation.");
                                } else {
                                    let mut to_delete = None;
                                    egui::Grid::new("anim_timings_grid")
                                        .num_columns(6)
                                        .spacing([8.0, 4.0])
                                        .show(ui, |ui| {
                                            ui.label("Frame");
                                            ui.label("Sound (SE)");
                                            ui.label("Flash Target");
                                            ui.label("RGB / Power");
                                            ui.label("Shake");
                                            ui.label("");
                                            ui.end_row();

                                            for (idx, t) in a.timings.iter_mut().enumerate() {
                                                if ui.add(egui::DragValue::new(&mut t.frame).range(1..=100)).changed() { *dirty = true; }
                                                if ui.text_edit_singleline(&mut t.se_name).changed() { *dirty = true; }

                                                egui::ComboBox::from_id_salt(format!("flash_scope_{}", idx))
                                                    .selected_text(match t.flash_scope {
                                                        0 => "None",
                                                        1 => "Target",
                                                        _ => "Screen",
                                                    })
                                                    .show_ui(ui, |ui| {
                                                        if ui.selectable_value(&mut t.flash_scope, 0, "None").clicked() { *dirty = true; }
                                                        if ui.selectable_value(&mut t.flash_scope, 1, "Target").clicked() { *dirty = true; }
                                                        if ui.selectable_value(&mut t.flash_scope, 2, "Screen").clicked() { *dirty = true; }
                                                    });

                                                ui.horizontal(|ui| {
                                                    if ui.add(egui::DragValue::new(&mut t.flash_red).range(0..=31).prefix("R:")).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut t.flash_green).range(0..=31).prefix("G:")).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut t.flash_blue).range(0..=31).prefix("B:")).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut t.flash_power).range(0..=31).prefix("P:")).changed() { *dirty = true; }
                                                });

                                                egui::ComboBox::from_id_salt(format!("shake_{}", idx))
                                                    .selected_text(match t.screen_shake {
                                                        0 => "None",
                                                        1 => "Target",
                                                        _ => "Screen",
                                                    })
                                                    .show_ui(ui, |ui| {
                                                        if ui.selectable_value(&mut t.screen_shake, 0, "None").clicked() { *dirty = true; }
                                                        if ui.selectable_value(&mut t.screen_shake, 1, "Target").clicked() { *dirty = true; }
                                                        if ui.selectable_value(&mut t.screen_shake, 2, "Screen").clicked() { *dirty = true; }
                                                    });

                                                if ui.small_button("🗑").clicked() {
                                                    to_delete = Some(idx);
                                                }
                                                ui.end_row();
                                            }
                                        });

                                    if let Some(del_idx) = to_delete {
                                        a.timings.remove(del_idx);
                                        *dirty = true;
                                    }
                                }
                            }
                        });
                }
            });
        });
    }
}


