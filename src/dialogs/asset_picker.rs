use eframe::egui;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use crate::app_state::AppPersistentData;
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

pub struct AssetPickerState {
    pub is_open: bool,
    pub category: String,
    pub selected_file: String,
    pub selected_index: i32,
    pub available_files: Vec<String>,
    pub search_filter: String,
    pub auto_apply_standard_poses: bool,
}

impl Default for AssetPickerState {
    fn default() -> Self {
        Self {
            is_open: false,
            category: "CharSet".to_string(),
            selected_file: String::new(),
            selected_index: 0,
            available_files: Vec::new(),
            search_filter: String::new(),
            auto_apply_standard_poses: false,
        }
    }
}

impl AssetPickerState {
    pub fn open(&mut self, project_path: &str, category: &str, current_file: &str, current_index: i32) {
        self.category = category.to_string();
        self.selected_file = current_file.to_string();
        self.selected_index = current_index;
        self.search_filter.clear();
        self.auto_apply_standard_poses = false;
        self.available_files = Self::scan_files(project_path, category);
        self.is_open = true;
    }

    fn scan_dir_for_category(dir: &Path, category: &str, set: &mut BTreeSet<String>) {
        let cat_dir = dir.join(category);
        if let Ok(entries) = fs::read_dir(cat_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if ext == "png" || ext == "xyz" || ext == "bmp" || ext == "mid" || ext == "wav" || ext == "mp3" || ext == "ogg" {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            set.insert(stem.to_string());
                        }
                    }
                }
            }
        }
    }

    fn scan_files(project_path: &str, category: &str) -> Vec<String> {
        let mut set = BTreeSet::new();

        // 1. Scan Project
        Self::scan_dir_for_category(Path::new(project_path), category, &mut set);

        // 2. Scan RTP
        let config = AppPersistentData::load();
        if let Some(rtp_dir) = config.get_effective_rtp_path() {
            Self::scan_dir_for_category(&rtp_dir, category, &mut set);
        }

        set.into_iter().collect()
    }

    /// Shows the modal asset picker dialog. Returns Some((selected_file, selected_index)) when accepted.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project_path: &str,
        cache: &mut AssetPreviewCache,
    ) -> Option<(String, i32)> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut is_open = self.is_open;

        egui::Window::new(format!("📁 Select {}", self.category))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([620.0, 480.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍 Filter:");
                    ui.add(egui::TextEdit::singleline(&mut self.search_filter).hint_text("Search file...").desired_width(200.0));
                    if !self.search_filter.is_empty() && ui.small_button("✕").clicked() {
                        self.search_filter.clear();
                    }
                });
                ui.separator();

                ui.columns(2, |cols| {
                    // Left column: file list
                    cols[0].heading("Available Files");
                    egui::ScrollArea::vertical()
                        .id_salt("asset_picker_file_list")
                        .max_height(340.0)
                        .show(&mut cols[0], |ui| {
                            if ui.selectable_label(self.selected_file.is_empty(), "(None)").clicked() {
                                self.selected_file.clear();
                            }
                            let filter = self.search_filter.to_lowercase();
                            for f in &self.available_files {
                                if !filter.is_empty() && !f.to_lowercase().contains(&filter) {
                                    continue;
                                }
                                if ui.selectable_label(self.selected_file == *f, f).clicked() {
                                    self.selected_file = f.clone();
                                }
                            }
                        });

                    // Right column: preview & interactive grid
                    cols[1].heading("Interactive Preview");
                    if !self.selected_file.is_empty() {
                        let is_dark = cols[1].visuals().dark_mode;
                        cols[1].label(format!("Graphic: {}  (Index #{})", self.selected_file, self.selected_index));
                        if let Some(tex) = cache.get_or_load(ctx, project_path, &self.category, &self.selected_file) {
                            let grid_stroke_color = crate::theme::colors::grid_line(is_dark);
                            let sel_stroke_color = crate::theme::colors::info(is_dark);

                            if self.category == "FaceSet" {
                                let tex_size = tex.size_vec2();
                                let grid_cols = (tex_size.x / 48.0).max(1.0).round() as i32;
                                let grid_rows = (tex_size.y / 48.0).max(1.0).round() as i32;
                                let max_idx = (grid_cols * grid_rows - 1).max(0);

                                cols[1].label(format!("💡 Click any face below to select it ({}×{} grid):", grid_cols, grid_rows));
                                egui::ScrollArea::both()
                                    .id_salt("faceset_picker_scroll")
                                    .max_height(280.0)
                                    .show(&mut cols[1], |ui| {
                                        let cell_w = 48.0;
                                        let cell_h = 48.0;
                                        let disp_size = egui::vec2(grid_cols as f32 * cell_w, grid_rows as f32 * cell_h);
                                        let (rect, resp) = ui.allocate_exact_size(disp_size, egui::Sense::click());
                                        let painter = ui.painter_at(rect);
                                        draw_checkerboard(&painter, rect, 8.0, is_dark);
                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                                        // Draw grid lines
                                        for i in 0..=grid_cols {
                                            let x = rect.min.x + i as f32 * cell_w;
                                            painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                        }
                                        for i in 0..=grid_rows {
                                            let y = rect.min.y + i as f32 * cell_h;
                                            painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], egui::Stroke::new(1.0, grid_stroke_color));
                                        }

                                        // Highlight selected face
                                        let sel_idx = (self.selected_index.clamp(0, max_idx)) as usize;
                                        let sel_c = (sel_idx as i32) % grid_cols;
                                        let sel_r = (sel_idx as i32) / grid_cols;
                                        let sel_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.min.x + sel_c as f32 * cell_w, rect.min.y + sel_r as f32 * cell_h),
                                            egui::vec2(cell_w, cell_h),
                                        );
                                        painter.rect_stroke(sel_rect, 0.0, egui::Stroke::new(3.0, sel_stroke_color), egui::StrokeKind::Inside);

                                        // Click detection
                                        if resp.clicked() {
                                            if let Some(pos) = resp.interact_pointer_pos() {
                                                let rel_x = (pos.x - rect.min.x).clamp(0.0, disp_size.x - 1.0);
                                                let rel_y = (pos.y - rect.min.y).clamp(0.0, disp_size.y - 1.0);
                                                let col = (rel_x / cell_w) as i32;
                                                let row = (rel_y / cell_h) as i32;
                                                self.selected_index = row * grid_cols + col;
                                            }
                                        }
                                    });
                            } else if self.category == "CharSet" {
                                let tex_size = tex.size_vec2();
                                let grid_cols = (tex_size.x / 72.0).max(1.0).round() as i32;
                                let grid_rows = (tex_size.y / 128.0).max(1.0).round() as i32;
                                let max_idx = (grid_cols * grid_rows - 1).max(0);
                                let scale = 0.75;
                                let cell_w = 72.0 * scale;
                                let cell_h = 128.0 * scale;

                                cols[1].label(format!("💡 Click any character below to select it ({}×{} grid):", grid_cols, grid_rows));
                                egui::ScrollArea::both()
                                    .id_salt("charset_picker_scroll")
                                    .max_height(280.0)
                                    .show(&mut cols[1], |ui| {
                                        let disp_size = egui::vec2(grid_cols as f32 * cell_w, grid_rows as f32 * cell_h);
                                        let (rect, resp) = ui.allocate_exact_size(disp_size, egui::Sense::click());
                                        let painter = ui.painter_at(rect);
                                        draw_checkerboard(&painter, rect, 8.0, is_dark);
                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                                        // Draw grid lines
                                        for i in 0..=grid_cols {
                                            let x = rect.min.x + i as f32 * cell_w;
                                            painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                        }
                                        for i in 0..=grid_rows {
                                            let y = rect.min.y + i as f32 * cell_h;
                                            painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], egui::Stroke::new(1.0, grid_stroke_color));
                                        }

                                        // Highlight selected character
                                        let sel_idx = (self.selected_index.clamp(0, max_idx)) as usize;
                                        let sel_c = (sel_idx as i32) % grid_cols;
                                        let sel_r = (sel_idx as i32) / grid_cols;
                                        let sel_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.min.x + sel_c as f32 * cell_w, rect.min.y + sel_r as f32 * cell_h),
                                            egui::vec2(cell_w, cell_h),
                                        );
                                        painter.rect_stroke(sel_rect, 0.0, egui::Stroke::new(3.0, sel_stroke_color), egui::StrokeKind::Inside);

                                        // Click detection
                                        if resp.clicked() {
                                            if let Some(pos) = resp.interact_pointer_pos() {
                                                let rel_x = (pos.x - rect.min.x).clamp(0.0, disp_size.x - 1.0);
                                                let rel_y = (pos.y - rect.min.y).clamp(0.0, disp_size.y - 1.0);
                                                let col = (rel_x / cell_w) as i32;
                                                let row = (rel_y / cell_h) as i32;
                                                self.selected_index = row * grid_cols + col;
                                            }
                                        }
                                    });
                            } else if self.category == "BattleCharSet" {
                                let grid_rows = 8;
                                let cell_w = 40.0;
                                let cell_h = 40.0;
                                let row_w = cell_w * 3.0;

                                let is_sheet_b = self.selected_file.ends_with(" B")
                                    || self.selected_file.ends_with(" B.png")
                                    || self.selected_file.ends_with("_B")
                                    || self.selected_file.ends_with("_B.png");

                                let standard_labels = if is_sheet_b {
                                    [
                                        "0: Walk Left / Approach",
                                        "1: Walk Right / Retreat",
                                        "2: Victory / Cheer",
                                        "3: Item / Custom (Row 3)",
                                        "4: Custom (Row 4)",
                                        "5: Custom (Row 5)",
                                        "6: Custom (Row 6)",
                                        "7: Custom (Row 7)",
                                    ]
                                } else {
                                    [
                                        "0: Skill / Magic (Cast)",
                                        "1: Defend / Guard",
                                        "2: Abnormal Status / Dazed",
                                        "3: Dead / Defeated",
                                        "4: Damage / Low HP",
                                        "5: Right Hand Attack",
                                        "6: Left Hand Attack",
                                        "7: Idle / Ready",
                                    ]
                                };

                                cols[1].horizontal(|ui| {
                                    let anim_step = (ui.input(|i| i.time * 6.0).floor() as usize) % 4;
                                    let anim_col = match anim_step { 0 => 0, 1 => 1, 2 => 2, _ => 1 };
                                    let sel_row = self.selected_index.clamp(0, 7) as usize;

                                    let u_min = anim_col as f32 / 3.0;
                                    let u_max = (anim_col + 1) as f32 / 3.0;
                                    let v_min = sel_row as f32 / 8.0;
                                    let v_max = (sel_row + 1) as f32 / 8.0;
                                    let anim_uv = egui::Rect::from_min_max(egui::pos2(u_min, v_min), egui::pos2(u_max, v_max));

                                    let (prev_rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                                    let p = ui.painter_at(prev_rect);
                                    draw_checkerboard(&p, prev_rect, 6.0, is_dark);
                                    p.image(tex.id(), prev_rect, anim_uv, egui::Color32::WHITE);
                                    p.rect_stroke(prev_rect, 2.0, egui::Stroke::new(1.5, sel_stroke_color), egui::StrokeKind::Outside);

                                    ui.vertical(|ui| {
                                        ui.label(format!("Active Pose: {}", standard_labels.get(sel_row).unwrap_or(&"Custom Pose")));
                                        ui.small("Click any row below to pick pose frame");
                                    });
                                });

                                egui::ScrollArea::vertical()
                                    .id_salt("battlecharset_picker_scroll")
                                    .max_height(250.0)
                                    .show(&mut cols[1], |ui| {
                                        for row in 0..grid_rows {
                                            let is_selected = self.selected_index == row as i32;

                                            ui.horizontal(|ui| {
                                                let disp_size = egui::vec2(row_w, cell_h);
                                                let (rect, resp) = ui.allocate_exact_size(disp_size, egui::Sense::click());
                                                let painter = ui.painter_at(rect);
                                                draw_checkerboard(&painter, rect, 6.0, is_dark);

                                                let v_min = row as f32 / grid_rows as f32;
                                                let v_max = (row + 1) as f32 / grid_rows as f32;
                                                let uv = egui::Rect::from_min_max(egui::pos2(0.0, v_min), egui::pos2(1.0, v_max));
                                                painter.image(tex.id(), rect, uv, egui::Color32::WHITE);

                                                for f in 1..3 {
                                                    let fx = rect.min.x + f as f32 * cell_w;
                                                    painter.line_segment([egui::pos2(fx, rect.min.y), egui::pos2(fx, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                                }

                                                if is_selected {
                                                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.5, sel_stroke_color), egui::StrokeKind::Inside);
                                                }

                                                let label_text = standard_labels.get(row).copied().unwrap_or("Custom Pose");
                                                let text_ui = ui.selectable_label(is_selected, label_text);
                                                if resp.clicked() || text_ui.clicked() {
                                                    self.selected_index = row as i32;
                                                }
                                            });
                                        }
                                    });
                            } else if self.category == "BattleWeapon" {
                                let grid_rows = 8;
                                let cell_w = 40.0;
                                let cell_h = 40.0;
                                let row_w = cell_w * 3.0;

                                cols[1].horizontal(|ui| {
                                    let anim_step = (ui.input(|i| i.time * 6.0).floor() as usize) % 4;
                                    let anim_col = match anim_step { 0 => 0, 1 => 1, 2 => 2, _ => 1 };
                                    let sel_row = self.selected_index.clamp(0, 7) as usize;

                                    let u_min = anim_col as f32 / 3.0;
                                    let u_max = (anim_col + 1) as f32 / 3.0;
                                    let v_min = sel_row as f32 / 8.0;
                                    let v_max = (sel_row + 1) as f32 / 8.0;
                                    let anim_uv = egui::Rect::from_min_max(egui::pos2(u_min, v_min), egui::pos2(u_max, v_max));

                                    let (prev_rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                                    let p = ui.painter_at(prev_rect);
                                    draw_checkerboard(&p, prev_rect, 6.0, is_dark);
                                    p.image(tex.id(), prev_rect, anim_uv, egui::Color32::WHITE);
                                    p.rect_stroke(prev_rect, 2.0, egui::Stroke::new(1.5, sel_stroke_color), egui::StrokeKind::Outside);

                                    ui.vertical(|ui| {
                                        ui.label(format!("Weapon Motion #{}", sel_row));
                                        ui.small("Click any row below to pick weapon motion");
                                    });
                                });

                                egui::ScrollArea::vertical()
                                    .id_salt("battleweapon_picker_scroll")
                                    .max_height(250.0)
                                    .show(&mut cols[1], |ui| {
                                        for row in 0..grid_rows {
                                            let is_selected = self.selected_index == row as i32;

                                            ui.horizontal(|ui| {
                                                let disp_size = egui::vec2(row_w, cell_h);
                                                let (rect, resp) = ui.allocate_exact_size(disp_size, egui::Sense::click());
                                                let painter = ui.painter_at(rect);
                                                draw_checkerboard(&painter, rect, 6.0, is_dark);

                                                let v_min = row as f32 / grid_rows as f32;
                                                let v_max = (row + 1) as f32 / grid_rows as f32;
                                                let uv = egui::Rect::from_min_max(egui::pos2(0.0, v_min), egui::pos2(1.0, v_max));
                                                painter.image(tex.id(), rect, uv, egui::Color32::WHITE);

                                                for f in 1..3 {
                                                    let fx = rect.min.x + f as f32 * cell_w;
                                                    painter.line_segment([egui::pos2(fx, rect.min.y), egui::pos2(fx, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                                }

                                                if is_selected {
                                                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.5, sel_stroke_color), egui::StrokeKind::Inside);
                                                }

                                                let text_ui = ui.selectable_label(is_selected, format!("Weapon Motion #{}", row));
                                                if resp.clicked() || text_ui.clicked() {
                                                    self.selected_index = row as i32;
                                                }
                                            });
                                        }
                                    });
                            } else {
                                let (rect, _) = cols[1].allocate_exact_size(egui::vec2(220.0, 180.0), egui::Sense::hover());
                                let painter = cols[1].painter_at(rect);
                                draw_checkerboard(&painter, rect, 8.0, is_dark);
                                painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                            }
                        } else if self.category == "Music" || self.category == "Sound" {
                            cols[1].group(|ui| {
                                let icon = if self.category == "Music" { "🎵" } else { "🔊" };
                                ui.heading(format!("{} Audio Track", icon));
                                ui.label(format!("Track Name: {}", self.selected_file));
                                ui.label(format!("Category: {}", self.category));
                                ui.separator();
                                ui.label("Supported Engine Formats: .mid, .wav, .mp3, .ogg, .wma");
                                ui.horizontal(|ui| {
                                    let ok_col = crate::theme::colors::success(is_dark);
                                    ui.colored_label(ok_col, "● Ready for Playback");
                                });
                            });
                        } else {
                            cols[1].colored_label(crate::theme::colors::muted(is_dark), "(Preview not available for this format)");
                        }
                    } else {
                        cols[1].colored_label(egui::Color32::GRAY, "No item selected.");
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if self.category == "BattleCharSet" {
                        ui.checkbox(&mut self.auto_apply_standard_poses, "⚡ Auto-populate all 12 Standard 2003 Poses (A: rows 0-7, B: rows 0-3)");
                    }
                    if ui.button("✅ OK").clicked() {
                        result = Some((self.selected_file.clone(), self.selected_index));
                        self.is_open = false;
                    }
                    if ui.button("❌ Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }

        result
    }
}

