use eframe::egui;
use crate::lcf_bridge::{self, MapLayers};

pub struct MapTargetPickerState {
    pub is_open: bool,
    pub selected_map_id: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub maps_list: Vec<(i32, String)>,
    pub current_layers: Option<MapLayers>,
}

impl Default for MapTargetPickerState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_map_id: 1,
            target_x: 0,
            target_y: 0,
            maps_list: Vec::new(),
            current_layers: None,
        }
    }
}

impl MapTargetPickerState {
    pub fn open(&mut self, project_path: &str, init_map_id: i32, init_x: i32, init_y: i32) {
        self.is_open = true;
        self.selected_map_id = init_map_id.max(1);
        self.target_x = init_x.max(0);
        self.target_y = init_y.max(0);

        let info = lcf_bridge::load_project(project_path);
        self.maps_list = info.maps.into_iter().map(|m| (m.id, m.name)).collect();
        self.load_map_layers(project_path, self.selected_map_id);
    }

    fn load_map_layers(&mut self, project_path: &str, map_id: i32) {
        let layers = lcf_bridge::get_map_layers(project_path, map_id);
        self.current_layers = if layers.width > 0 && layers.height > 0 {
            Some(layers)
        } else {
            None
        };
    }

    pub fn show(&mut self, ctx: &egui::Context, project_path: &str) -> Option<(i32, i32, i32)> {
        if !self.is_open {
            return None;
        }

        let mut confirmed = None;
        let mut is_open = self.is_open;

        egui::Window::new("Select Target Location")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([720.0, 500.0])
            .show(ctx, |ui| {
                ui.columns(2, |cols| {
                    // Left: Maps List
                    cols[0].group(|ui| {
                        ui.heading("Select Destination Map");
                        egui::ScrollArea::vertical()
                            .id_salt("target_picker_maps_scroll")
                            .max_height(380.0)
                            .show(ui, |ui| {
                                let mut clicked_mid = None;
                                for (map_id, map_name) in &self.maps_list {
                                    let is_sel = *map_id == self.selected_map_id;
                                    let label = format!("{:04}: {}", map_id, map_name);
                                    if ui.selectable_label(is_sel, label).clicked() {
                                        clicked_mid = Some(*map_id);
                                    }
                                }
                                if let Some(mid) = clicked_mid {
                                    self.selected_map_id = mid;
                                    self.load_map_layers(project_path, mid);
                                }
                            });
                    });

                    // Right: Map Coordinate Picker Canvas
                    cols[1].group(|ui| {
                        ui.heading(format!("Map #{:04} - Target ({}, {})", self.selected_map_id, self.target_x, self.target_y));

                        if let Some(layers) = &self.current_layers {
                            let tile_px = 16.0;
                            let canvas_w = layers.width as f32 * tile_px;
                            let canvas_h = layers.height as f32 * tile_px;

                            egui::ScrollArea::both()
                                .id_salt("target_picker_canvas_scroll")
                                .max_height(350.0)
                                .show(ui, |ui| {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(canvas_w, canvas_h),
                                        egui::Sense::click(),
                                    );
                                    let painter = ui.painter().with_clip_rect(rect);

                                    let is_dark = ui.visuals().dark_mode;
                                    let canvas_bg = if is_dark { egui::Color32::from_rgb(30, 40, 30) } else { egui::Color32::from_rgb(235, 245, 238) };
                                    // Background fill
                                    painter.rect_filled(rect, 0.0, canvas_bg);

                                    // Draw grid
                                    let grid_col = crate::theme::colors::grid_line(is_dark);
                                    for x in 0..=layers.width {
                                        let gx = rect.min.x + x as f32 * tile_px;
                                        painter.line_segment(
                                            [egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)],
                                            egui::Stroke::new(1.0, grid_col),
                                        );
                                    }
                                    for y in 0..=layers.height {
                                        let gy = rect.min.y + y as f32 * tile_px;
                                        painter.line_segment(
                                            [egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)],
                                            egui::Stroke::new(1.0, grid_col),
                                        );
                                    }

                                    // Handle click
                                    if resp.clicked() {
                                        if let Some(pos) = resp.interact_pointer_pos() {
                                            let tx = ((pos.x - rect.min.x) / tile_px).floor() as i32;
                                            let ty = ((pos.y - rect.min.y) / tile_px).floor() as i32;
                                            self.target_x = tx.clamp(0, layers.width - 1);
                                            self.target_y = ty.clamp(0, layers.height - 1);
                                        }
                                    }

                                    // Draw Target Crosshair
                                    let target_rect = egui::Rect::from_min_size(
                                        egui::pos2(rect.min.x + self.target_x as f32 * tile_px, rect.min.y + self.target_y as f32 * tile_px),
                                        egui::vec2(tile_px, tile_px),
                                    );
                                    let target_stroke_col = crate::theme::colors::danger(is_dark);
                                    painter.rect_stroke(
                                        target_rect,
                                        0.0,
                                        egui::Stroke::new(2.0, target_stroke_col),
                                        egui::StrokeKind::Outside,
                                    );
                                    painter.text(
                                        target_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "🎯",
                                        egui::FontId::proportional(12.0),
                                        egui::Color32::WHITE,
                                    );
                                });
                        } else {
                            let is_dark = ui.visuals().dark_mode;
                            ui.colored_label(crate::theme::colors::muted(is_dark), "(Select map to load canvas)");
                        }
                    });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        confirmed = Some((self.selected_map_id, self.target_x, self.target_y));
                        self.is_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }

        confirmed
    }
}
