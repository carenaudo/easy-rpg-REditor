use eframe::egui;
use image::RgbaImage;
use crate::tilemap::{self, PALETTE_COLS};

#[derive(Clone, Debug, PartialEq)]
pub struct TileBrush {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<i32>,
}

impl Default for TileBrush {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            tiles: vec![5000],
        }
    }
}

pub struct MapPalette {
    pub selected_tile_id: i32,
    pub brush: TileBrush,
    pub drag_start: Option<(usize, usize)>,
    pub drag_end: Option<(usize, usize)>,
    lower_texture: Option<egui::TextureHandle>,
    lower_tile_ids: Vec<i32>,
    upper_texture: Option<egui::TextureHandle>,
    upper_tile_ids: Vec<i32>,
    palette_zoom: f32,
}

impl Default for MapPalette {
    fn default() -> Self {
        Self {
            selected_tile_id: 5000,
            brush: TileBrush::default(),
            drag_start: None,
            drag_end: None,
            lower_texture: None,
            lower_tile_ids: Vec::new(),
            upper_texture: None,
            upper_tile_ids: Vec::new(),
            palette_zoom: 1.5,
        }
    }
}

impl MapPalette {
    pub fn set_single_tile(&mut self, id: i32, is_upper: bool) {
        self.selected_tile_id = id;
        self.brush = TileBrush {
            width: 1,
            height: 1,
            tiles: vec![id],
        };
        let tile_ids = if is_upper { &self.upper_tile_ids } else { &self.lower_tile_ids };
        if let Some(idx) = tile_ids.iter().position(|&t| t == id) {
            let col = idx % PALETTE_COLS;
            let row = idx / PALETTE_COLS;
            self.drag_start = Some((col, row));
            self.drag_end = Some((col, row));
        } else {
            self.drag_start = None;
            self.drag_end = None;
        }
    }

    fn update_brush_for_layer(&mut self, is_upper: bool) {
        if let (Some((c1, r1)), Some((c2, r2))) = (self.drag_start, self.drag_end) {
            let min_c = c1.min(c2).min(PALETTE_COLS - 1);
            let max_c = c1.max(c2).min(PALETTE_COLS - 1);
            let min_r = r1.min(r2);
            let max_r = r2.max(r1);

            let w = max_c - min_c + 1;
            let h = max_r - min_r + 1;
            let mut tiles = Vec::with_capacity(w * h);

            let tile_ids = if is_upper { &self.upper_tile_ids } else { &self.lower_tile_ids };
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    let idx = r * PALETTE_COLS + c;
                    let id = tile_ids.get(idx).copied().unwrap_or(0);
                    tiles.push(id);
                }
            }

            self.selected_tile_id = tiles.first().copied().unwrap_or(5000);
            self.brush = TileBrush {
                width: w,
                height: h,
                tiles,
            };
        }
    }

    pub fn reload_chipset(&mut self, ctx: &egui::Context, chipset: &RgbaImage) {
        // Lower palette
        let (lower_img, lower_ids) = tilemap::render_palette_image(chipset, false);
        let size = [lower_img.width() as usize, lower_img.height() as usize];
        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &lower_img);
        self.lower_texture = Some(ctx.load_texture("palette_lower", color_img, egui::TextureOptions::NEAREST));
        self.lower_tile_ids = lower_ids;

        // Upper palette
        let (upper_img, upper_ids) = tilemap::render_palette_image(chipset, true);
        let size = [upper_img.width() as usize, upper_img.height() as usize];
        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &upper_img);
        self.upper_texture = Some(ctx.load_texture("palette_upper", color_img, egui::TextureOptions::NEAREST));
        self.upper_tile_ids = upper_ids;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, is_upper: bool) {
        // Pane head: compact uppercase title, right-aligned zoom steppers.
        ui.horizontal(|ui| {
            ui.strong(rust_i18n::t!("pane.palette").to_uppercase());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("+").on_hover_text("Zoom in").clicked() {
                    self.palette_zoom = (self.palette_zoom + 0.25).min(3.0);
                }
                if ui.small_button("−").on_hover_text("Zoom out").clicked() {
                    self.palette_zoom = (self.palette_zoom - 0.25).max(1.0);
                }
            });
        });
        ui.horizontal(|ui| {
            let is_dark = ui.visuals().dark_mode;
            let brush_desc = if self.brush.width > 1 || self.brush.height > 1 {
                format!("Brush: {}×{} (Selected: #{})", self.brush.width, self.brush.height, self.selected_tile_id)
            } else {
                format!("Selected: #{}", self.selected_tile_id)
            };
            ui.colored_label(crate::theme::colors::muted(is_dark), brush_desc);
        });

        ui.separator();

        let tex_id = if is_upper {
            self.upper_texture.as_ref().map(|t| t.id())
        } else {
            self.lower_texture.as_ref().map(|t| t.id())
        };

        if let Some(texture_id) = tex_id {
            let tile_ids_len = if is_upper { self.upper_tile_ids.len() } else { self.lower_tile_ids.len() };
            let tile_px = 16.0 * self.palette_zoom;
            let display_w = PALETTE_COLS as f32 * tile_px;
            let rows = (tile_ids_len + PALETTE_COLS - 1) / PALETTE_COLS;
            let display_h = rows as f32 * tile_px;

            egui::ScrollArea::vertical()
                .id_salt("map_palette_canvas_scroll")
                .show(ui, |ui| {
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(display_w, display_h), egui::Sense::click_and_drag());

                    // Paint texture
                    ui.painter().image(
                        texture_id,
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                    // Click & drag handling for multi-tile selection
                    if resp.drag_started() || resp.clicked() {
                        if let Some(mouse_pos) = resp.interact_pointer_pos() {
                            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, display_w - 1.0);
                            let rel_y = (mouse_pos.y - rect.min.y).clamp(0.0, display_h - 1.0);
                            let col = (rel_x / tile_px) as usize;
                            let row = (rel_y / tile_px) as usize;
                            self.drag_start = Some((col, row));
                            self.drag_end = Some((col, row));
                            self.update_brush_for_layer(is_upper);
                        }
                    } else if resp.dragged() {
                        if let Some(mouse_pos) = resp.interact_pointer_pos() {
                            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, display_w - 1.0);
                            let rel_y = (mouse_pos.y - rect.min.y).clamp(0.0, display_h - 1.0);
                            let col = (rel_x / tile_px) as usize;
                            let row = (rel_y / tile_px) as usize;
                            self.drag_end = Some((col, row));
                            self.update_brush_for_layer(is_upper);
                        }
                    }

                    // Highlight selected brush rectangle
                    if let (Some((c1, r1)), Some((c2, r2))) = (self.drag_start, self.drag_end) {
                        let min_c = c1.min(c2) as f32;
                        let max_c = c1.max(c2) as f32;
                        let min_r = r1.min(r2) as f32;
                        let max_r = r2.max(r1) as f32;
                        let brush_rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + min_c * tile_px, rect.min.y + min_r * tile_px),
                            egui::pos2(rect.min.x + (max_c + 1.0) * tile_px, rect.min.y + (max_r + 1.0) * tile_px),
                        );
                        ui.painter().rect_stroke(
                            brush_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                            egui::StrokeKind::Outside,
                        );
                    }
                });
        } else {
            ui.colored_label(egui::Color32::GRAY, "(Select a map to load chipset)");
        }
    }
}
