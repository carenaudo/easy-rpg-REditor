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
        }
    }
}

impl AssetPickerState {
    pub fn open(&mut self, project_path: &str, category: &str, current_file: &str, current_index: i32) {
        self.category = category.to_string();
        self.selected_file = current_file.to_string();
        self.selected_index = current_index;
        self.search_filter.clear();
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
                                cols[1].label("💡 Click any face below to select it (4×4 grid):");
                                let disp_size = egui::vec2(192.0, 192.0);
                                let (rect, resp) = cols[1].allocate_exact_size(disp_size, egui::Sense::click());
                                let painter = cols[1].painter_at(rect);
                                draw_checkerboard(&painter, rect, 8.0, is_dark);
                                painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                                let cell_w = disp_size.x / 4.0;
                                let cell_h = disp_size.y / 4.0;

                                // Draw grid lines
                                for i in 0..=4 {
                                    let x = rect.min.x + i as f32 * cell_w;
                                    painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                    let y = rect.min.y + i as f32 * cell_h;
                                    painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], egui::Stroke::new(1.0, grid_stroke_color));
                                }

                                // Highlight selected face
                                let sel_idx = self.selected_index.clamp(0, 15) as usize;
                                let sel_c = sel_idx % 4;
                                let sel_r = sel_idx / 4;
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
                                        self.selected_index = row * 4 + col;
                                    }
                                }
                            } else if self.category == "CharSet" {
                                cols[1].label("💡 Click any character below to select it (4×2 grid):");
                                let disp_size = egui::vec2(216.0, 192.0); // 288x256 scaled down proportionally (3/4)
                                let (rect, resp) = cols[1].allocate_exact_size(disp_size, egui::Sense::click());
                                let painter = cols[1].painter_at(rect);
                                draw_checkerboard(&painter, rect, 8.0, is_dark);
                                painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                                let cell_w = disp_size.x / 4.0;
                                let cell_h = disp_size.y / 2.0;

                                // Draw grid lines
                                for i in 0..=4 {
                                    let x = rect.min.x + i as f32 * cell_w;
                                    painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], egui::Stroke::new(1.0, grid_stroke_color));
                                }
                                for i in 0..=2 {
                                    let y = rect.min.y + i as f32 * cell_h;
                                    painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], egui::Stroke::new(1.0, grid_stroke_color));
                                }

                                // Highlight selected character
                                let sel_idx = self.selected_index.clamp(0, 7) as usize;
                                let sel_c = sel_idx % 4;
                                let sel_r = sel_idx / 4;
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
                                        self.selected_index = row * 4 + col;
                                    }
                                }
                            } else {
                                let (rect, _) = cols[1].allocate_exact_size(egui::vec2(220.0, 180.0), egui::Sense::hover());
                                let painter = cols[1].painter_at(rect);
                                draw_checkerboard(&painter, rect, 8.0, is_dark);
                                painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                            }
                        } else if self.category == "Music" || self.category == "Sound" || self.category == "Title" {
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

