use eframe::egui;
use image::RgbaImage;
use crate::tilemap::{self, PALETTE_COLS};

pub struct MapPalette {
    pub selected_tile_id: i32,
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
            lower_texture: None,
            lower_tile_ids: Vec::new(),
            upper_texture: None,
            upper_tile_ids: Vec::new(),
            palette_zoom: 1.5,
        }
    }
}

impl MapPalette {
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
            ui.colored_label(crate::theme::colors::muted(is_dark), format!("Selected: #{}", self.selected_tile_id));
        });

        ui.separator();

        let (tex, tile_ids) = if is_upper {
            (&self.upper_texture, &self.upper_tile_ids)
        } else {
            (&self.lower_texture, &self.lower_tile_ids)
        };

        if let Some(texture) = tex {
            let tile_px = 16.0 * self.palette_zoom;
            let display_w = PALETTE_COLS as f32 * tile_px;
            let rows = (tile_ids.len() + PALETTE_COLS - 1) / PALETTE_COLS;
            let display_h = rows as f32 * tile_px;

            egui::ScrollArea::vertical()
                .id_salt("map_palette_canvas_scroll")
                .show(ui, |ui| {
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(display_w, display_h), egui::Sense::click_and_drag());

                    // Paint texture
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                    // Click handling
                    if resp.clicked() || resp.dragged() {
                        if let Some(mouse_pos) = resp.interact_pointer_pos() {
                            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, display_w - 1.0);
                            let rel_y = (mouse_pos.y - rect.min.y).clamp(0.0, display_h - 1.0);
                            let col = (rel_x / tile_px) as usize;
                            let row = (rel_y / tile_px) as usize;
                            let idx = row * PALETTE_COLS + col;
                            if let Some(&tile_id) = tile_ids.get(idx) {
                                self.selected_tile_id = tile_id;
                            }
                        }
                    }

                    // Highlight selected tile
                    if let Some(idx) = tile_ids.iter().position(|&id| id == self.selected_tile_id) {
                        let col = (idx % PALETTE_COLS) as f32;
                        let row = (idx / PALETTE_COLS) as f32;
                        let tile_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + col * tile_px, rect.min.y + row * tile_px),
                            egui::vec2(tile_px, tile_px),
                        );
                        ui.painter().rect_stroke(
                            tile_rect,
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
