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
    pub search_query: String,
}

impl Default for AnimationsView {
    fn default() -> Self {
        Self {
            selected_idx: 0,
            is_playing: true,
            scrub_frame: 0,
            fps: 15.0,
            show_target: true,
            search_query: String::new(),
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
                    frames: Vec::new(),
                    timings: Vec::new(),
                });
                *dirty = true;
            }
            return;
        }

        ui.columns(2, |cols| {
            // Master list
            cols[0].group(|ui| {
                ui.horizontal_wrapped(|ui| {
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
                            frames: Vec::new(),
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
                    if ui.add_enabled(animations.len() > 1 && self.selected_idx < animations.len(), egui::Button::new("🗑 Del").small()).clicked() {
                        animations.remove(self.selected_idx);
                        for (i, entry) in animations.iter_mut().enumerate() {
                            entry.id = (i + 1) as i32;
                        }
                        if self.selected_idx >= animations.len() {
                            self.selected_idx = animations.len().saturating_sub(1);
                        }
                        *dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Filter animations...").desired_width(120.0));
                    if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                        self.search_query.clear();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("anim_master_scroll")
                    .max_height(450.0)
                    .show(ui, |ui| {
                        let q = self.search_query.trim().to_lowercase();
                        for (idx, a) in animations.iter().enumerate() {
                            if !q.is_empty() && !a.name.to_lowercase().contains(&q) && !a.id.to_string().contains(&q) {
                                continue;
                            }
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

                                // Sync frames count
                                if a.frames.len() != a.frame_count {
                                    a.frames.resize_with(a.frame_count, || crate::lcf_bridge::AnimationFrameInfo::default());
                                    for (i, f) in a.frames.iter_mut().enumerate() {
                                        f.id = (i + 1) as i32;
                                    }
                                }

                                // Draw animation frame cells
                                if let Some(proj) = project_path {
                                    let tex_opt = cache.get_or_load(ui.ctx(), proj, "Battle", &a.animation_name)
                                        .or_else(|| cache.get_or_load(ui.ctx(), proj, "Battle2", &a.animation_name));

                                    if let Some(tex) = tex_opt {
                                        let center = match a.position {
                                            0 => egui::pos2(stage_rect.center().x, stage_rect.min.y + 40.0),
                                            2 => egui::pos2(stage_rect.center().x, stage_rect.max.y - 40.0),
                                            _ => stage_rect.center(),
                                        };

                                        let frame_opt = a.frames.get(active_frame);
                                        let has_cells = frame_opt.map_or(false, |f| !f.cells.is_empty());

                                        if !has_cells {
                                            let cell_idx = active_frame % 25;
                                            let cell_col = (cell_idx % 5) as f32;
                                            let cell_row = (cell_idx / 5) as f32;

                                            let u0 = cell_col / 5.0;
                                            let u1 = (cell_col + 1.0) / 5.0;
                                            let v0 = cell_row / 5.0;
                                            let v1 = (cell_row + 1.0) / 5.0;

                                            let cell_sz = if a.large { 128.0 } else { 96.0 };
                                            let draw_rect = egui::Rect::from_center_size(center, egui::vec2(cell_sz, cell_sz));
                                            painter.image(tex.id(), draw_rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::WHITE);
                                        } else if let Some(frame) = frame_opt {
                                            for cell in &frame.cells {
                                                if !cell.valid { continue; }
                                                let cell_idx = (cell.cell_id.max(0) as usize) % 25;
                                                let cell_col = (cell_idx % 5) as f32;
                                                let cell_row = (cell_idx / 5) as f32;

                                                let u0 = cell_col / 5.0;
                                                let u1 = (cell_col + 1.0) / 5.0;
                                                let v0 = cell_row / 5.0;
                                                let v1 = (cell_row + 1.0) / 5.0;

                                                let base_sz = if a.large { 128.0 } else { 96.0 };
                                                let zoom_scale = (cell.zoom.max(10) as f32) / 100.0;
                                                let cell_sz = base_sz * zoom_scale;
                                                let cell_pos = egui::pos2(center.x + cell.x as f32, center.y + cell.y as f32);
                                                let draw_rect = egui::Rect::from_center_size(cell_pos, egui::vec2(cell_sz, cell_sz));
                                                let alpha = (255 * (100 - cell.transparency.clamp(0, 100)) / 100) as u8;
                                                painter.image(tex.id(), draw_rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
                                            }
                                        }
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

                                // 2. Frame Cell Composer
                                if active_frame < a.frames.len() {
                                    let mut copy_prev = false;
                                    let mut add_cell = false;
                                    let mut clear_cells = false;
                                    let mut center_all = false;
                                    let mut reset_zoom = false;

                                    let cell_count = a.frames[active_frame].cells.len();
                                    ui.horizontal_wrapped(|ui| {
                                        ui.heading(format!("🎞 Frame {} Cell Composer ({})", active_frame + 1, cell_count));
                                        if ui.button("➕ Add Cell").clicked() {
                                            add_cell = true;
                                        }
                                        if active_frame > 0 && ui.button("📋 Copy Prev Frame").clicked() {
                                            copy_prev = true;
                                        }
                                        if cell_count > 0 && ui.button("🎯 Center All").on_hover_text("Align all cells to center (X=0, Y=0)").clicked() {
                                            center_all = true;
                                        }
                                        if cell_count > 0 && ui.button("🔄 Reset Zoom").on_hover_text("Reset scale to 100% and opacity to 100%").clicked() {
                                            reset_zoom = true;
                                        }
                                        if cell_count > 0 && ui.button("🗑 Clear").clicked() {
                                            clear_cells = true;
                                        }
                                    });

                                    if copy_prev {
                                        let prev_cells = a.frames[active_frame - 1].cells.clone();
                                        a.frames[active_frame].cells = prev_cells;
                                        *dirty = true;
                                    }
                                    if add_cell {
                                        let new_id = (a.frames[active_frame].cells.len() + 1) as i32;
                                        a.frames[active_frame].cells.push(crate::lcf_bridge::AnimationCellInfo {
                                            id: new_id,
                                            valid: true,
                                            cell_id: (active_frame % 25) as i32,
                                            x: 0,
                                            y: 0,
                                            zoom: 100,
                                            transparency: 0,
                                        });
                                        *dirty = true;
                                    }
                                    if center_all {
                                        for cell in &mut a.frames[active_frame].cells {
                                            cell.x = 0;
                                            cell.y = 0;
                                        }
                                        *dirty = true;
                                    }
                                    if reset_zoom {
                                        for cell in &mut a.frames[active_frame].cells {
                                            cell.zoom = 100;
                                            cell.transparency = 0;
                                        }
                                        *dirty = true;
                                    }
                                    if clear_cells {
                                        a.frames[active_frame].cells.clear();
                                        *dirty = true;
                                    }

                                    let frame = &mut a.frames[active_frame];
                                    if frame.cells.is_empty() {
                                        ui.colored_label(egui::Color32::GRAY, "No bespoke cells placed on this frame (plays default cell sequentially).");
                                    } else {
                                        let mut cell_to_del = None;
                                        egui::Grid::new("anim_frame_cells_grid")
                                            .num_columns(7)
                                            .spacing([6.0, 4.0])
                                            .show(ui, |ui| {
                                                ui.label("#");
                                                ui.label("Sub-Cell (0..49)");
                                                ui.label("X Offset");
                                                ui.label("Y Offset");
                                                ui.label("Zoom %");
                                                ui.label("Transp %");
                                                ui.label("");
                                                ui.end_row();

                                                for (c_idx, cell) in frame.cells.iter_mut().enumerate() {
                                                    ui.label(format!("#{}", c_idx + 1));
                                                    if ui.add(egui::DragValue::new(&mut cell.cell_id).range(0..=49)).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut cell.x).range(-200..=200)).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut cell.y).range(-200..=200)).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut cell.zoom).range(10..=300).suffix("%")).changed() { *dirty = true; }
                                                    if ui.add(egui::DragValue::new(&mut cell.transparency).range(0..=100).suffix("%")).changed() { *dirty = true; }
                                                    if ui.small_button("🗑").clicked() {
                                                        cell_to_del = Some(c_idx);
                                                    }
                                                    ui.end_row();
                                                }
                                            });

                                        if let Some(del_idx) = cell_to_del {
                                            frame.cells.remove(del_idx);
                                            *dirty = true;
                                        }
                                    }
                                }

                                ui.separator();

                                // 3. Sound Effects & Screen Flash Timings Table
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


