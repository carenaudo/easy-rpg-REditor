use eframe::egui;
use std::path::Path;
use crate::app_state::AppPersistentData;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::ChipsetInfo;
use crate::tilemap;
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PassabilityEditMode {
    General, // 〇, ✕, △, ★
    Directional, // 🠅, 🠇, 🠄, 🠆
    Terrain, // Assign Terrain 1..16
}

pub struct ChipsetsView {
    pub selected_idx: usize,
    pub edit_mode: PassabilityEditMode,
    pub active_terrain_id: i16,
    pub active_tab_upper: bool,
    pub zoom_level: usize, // 0: Normal (28px), 1: Large (38px), 2: Extra Large (48px)
    pub cached_chipset_name: String,
    pub lower_palette_tex: Option<egui::TextureHandle>,
    pub upper_palette_tex: Option<egui::TextureHandle>,
}

impl Default for ChipsetsView {
    fn default() -> Self {
        Self {
            selected_idx: 0,
            edit_mode: PassabilityEditMode::General,
            active_terrain_id: 1,
            active_tab_upper: false,
            zoom_level: 1,
            cached_chipset_name: String::new(),
            lower_palette_tex: None,
            upper_palette_tex: None,
        }
    }
}

impl ChipsetsView {
    fn reload_palette_textures(&mut self, ctx: &egui::Context, project_path: Option<&str>, chipset_name: &str) {
        self.cached_chipset_name = chipset_name.to_string();
        self.lower_palette_tex = None;
        self.upper_palette_tex = None;

        if chipset_name.is_empty() {
            return;
        }

        // Find chipset file in Project or RTP
        let mut bytes_opt = None;
        if let Some(proj) = project_path {
            let extensions = ["png", "xyz", "bmp", "PNG", "XYZ", "BMP"];
            for ext in &extensions {
                let p = Path::new(proj).join("ChipSet").join(format!("{}.{}", chipset_name, ext));
                if let Ok(b) = std::fs::read(&p) {
                    bytes_opt = Some(b);
                    break;
                }
            }
        }

        if bytes_opt.is_none() {
            let cfg = AppPersistentData::load();
            if let Some(rtp) = cfg.get_effective_rtp_path() {
                let extensions = ["png", "xyz", "bmp", "PNG", "XYZ", "BMP"];
                for ext in &extensions {
                    let p = rtp.join("ChipSet").join(format!("{}.{}", chipset_name, ext));
                    if let Ok(b) = std::fs::read(&p) {
                        bytes_opt = Some(b);
                        break;
                    }
                }
            }
        }

        if let Some(bytes) = bytes_opt {
            if let Ok(chipset_rgba) = tilemap::decode_chipset(&bytes) {
                // Lower palette
                let (lower_img, _) = tilemap::render_palette_image(&chipset_rgba, false);
                let size_l = [lower_img.width() as usize, lower_img.height() as usize];
                let color_img_l = egui::ColorImage::from_rgba_unmultiplied(size_l, &lower_img);
                self.lower_palette_tex = Some(ctx.load_texture("db_cs_lower", color_img_l, egui::TextureOptions::NEAREST));

                // Upper palette
                let (upper_img, _) = tilemap::render_palette_image(&chipset_rgba, true);
                let size_u = [upper_img.width() as usize, upper_img.height() as usize];
                let color_img_u = egui::ColorImage::from_rgba_unmultiplied(size_u, &upper_img);
                self.upper_palette_tex = Some(ctx.load_texture("db_cs_upper", color_img_u, egui::TextureOptions::NEAREST));
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        chipsets: &mut Vec<ChipsetInfo>,
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        _cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        if chipsets.is_empty() {
            ui.label("No chipsets in database.");
            if ui.button("➕ Add Chipset").clicked() {
                chipsets.push(ChipsetInfo {
                    id: 1,
                    name: "World".to_string(),
                    chipset_name: "World".to_string(),
                    terrain_data: vec![1; 162],
                    passable_data_lower: vec![15; 162],
                    passable_data_upper: vec![15; 144],
                    animation_type: 0,
                    animation_speed: 0,
                });
                *dirty = true;
            }
            return;
        }

        if let Some(cs) = chipsets.get_mut(self.selected_idx) {
            // Check if cached texture needs reload
            if self.cached_chipset_name != cs.chipset_name || self.lower_palette_tex.is_none() {
                self.reload_palette_textures(ui.ctx(), project_path, &cs.chipset_name);
            }

            // Header & Properties Card
            ui.horizontal_wrapped(|ui| {
                ui.heading(format!("🗺 {:04}: {}", cs.id, cs.name));
                ui.separator();

                ui.label("Name:");
                let name_edit = ui.text_edit_singleline(&mut cs.name);
                if name_edit.changed() { *dirty = true; }

                ui.separator();
                ui.label("Graphic:");
                let gname = if cs.chipset_name.is_empty() { "(None)".to_string() } else { cs.chipset_name.clone() };
                if ui.button(format!("🖼 {}", gname)).clicked() {
                    if let Some(proj) = project_path {
                        picker.open(proj, "ChipSet", &cs.chipset_name, 0);
                    }
                }

                ui.separator();
                ui.label("Anim Speed:");
                egui::ComboBox::from_id_salt("cs_anim_speed_combo")
                    .selected_text(match cs.animation_speed {
                        0 => "Normal (24f)",
                        1 => "Fast (16f)",
                        2 => "Slow (32f)",
                        _ => "Custom",
                    })
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut cs.animation_speed, 0, "Normal (24f)").clicked() { *dirty = true; }
                        if ui.selectable_value(&mut cs.animation_speed, 1, "Fast (16f)").clicked() { *dirty = true; }
                        if ui.selectable_value(&mut cs.animation_speed, 2, "Slow (32f)").clicked() { *dirty = true; }
                    });
            });

            ui.separator();

            // Tileset Toolbar: Layer, Mode, Zoom, Batch Actions
            ui.horizontal_wrapped(|ui| {
                ui.label("Layer:");
                ui.selectable_value(&mut self.active_tab_upper, false, "🏞 Lower Layer (162 Tiles)");
                ui.selectable_value(&mut self.active_tab_upper, true, "🏰 Upper Layer (144 Tiles)");

                ui.separator();
                ui.label("Mode:");
                ui.selectable_value(&mut self.edit_mode, PassabilityEditMode::General, "〇 / ✕ / △ / ★");
                ui.selectable_value(&mut self.edit_mode, PassabilityEditMode::Directional, "Arrows 🠅");
                if !self.active_tab_upper {
                    ui.selectable_value(&mut self.edit_mode, PassabilityEditMode::Terrain, "Terrain ID");
                }

                ui.separator();
                ui.label("Zoom:");
                ui.selectable_value(&mut self.zoom_level, 0, "1x (26px)");
                ui.selectable_value(&mut self.zoom_level, 1, "1.5x (36px)");
                ui.selectable_value(&mut self.zoom_level, 2, "2x (48px)");

                ui.separator();
                // Batch presets
                if ui.small_button("✅ Passable All").clicked() {
                    let data = if self.active_tab_upper { &mut cs.passable_data_upper } else { &mut cs.passable_data_lower };
                    for f in data.iter_mut() {
                        *f = 15;
                    }
                    *dirty = true;
                }
                if ui.small_button("❌ Block All").clicked() {
                    let data = if self.active_tab_upper { &mut cs.passable_data_upper } else { &mut cs.passable_data_lower };
                    for f in data.iter_mut() {
                        *f = 0;
                    }
                    *dirty = true;
                }
            });

            if self.edit_mode == PassabilityEditMode::Terrain && !self.active_tab_upper {
                ui.horizontal(|ui| {
                    ui.label("Active Terrain to Paint:");
                    ui.add(egui::DragValue::new(&mut self.active_terrain_id).range(1..=16));
                    ui.label(format!("(Assigning Terrain {})", self.active_terrain_id));
                });
            }

            ui.separator();

            // Interactive Composited Tileset Grid Studio
            let data = if self.active_tab_upper { &mut cs.passable_data_upper } else { &mut cs.passable_data_lower };
            let total_tiles = data.len();

            let cols_count = tilemap::PALETTE_COLS; // 6 columns
            let rows_count = (total_tiles + cols_count - 1) / cols_count;
            let tile_size = match self.zoom_level {
                0 => 26.0,
                1 => 36.0,
                _ => 48.0,
            };

            let palette_tex = if self.active_tab_upper {
                &self.upper_palette_tex
            } else {
                &self.lower_palette_tex
            };

            egui::ScrollArea::vertical()
                .id_salt("chipset_tile_flags_scroll")
                .show(ui, |ui| {
                    for r in 0..rows_count {
                        ui.horizontal(|ui| {
                            for c in 0..cols_count {
                                let idx = r * cols_count + c;
                                if idx < data.len() {
                                    let flag = data[idx];
                                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(tile_size, tile_size), egui::Sense::click());
                                    let painter = ui.painter_at(rect);

                                    let is_dark = ui.visuals().dark_mode;
                                    // 1. Draw Checkerboard and Composited Tile Texture
                                    draw_checkerboard(&painter, rect, 6.0, is_dark);

                                    if let Some(tex) = palette_tex {
                                        let u0 = (c as f32) / (cols_count as f32);
                                        let v0 = (r as f32) / (rows_count as f32);
                                        let u1 = ((c + 1) as f32) / (cols_count as f32);
                                        let v1 = ((r + 1) as f32) / (rows_count as f32);
                                        painter.image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1)), egui::Color32::WHITE);
                                    }

                                    // Subtle tile border
                                    let border_col = if is_dark {
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 35)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 35)
                                    };
                                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, border_col), egui::StrokeKind::Outside);

                                    // 2. High-Contrast Passability Overlay
                                    let (symbol, color, tooltip) = match self.edit_mode {
                                        PassabilityEditMode::General => {
                                            if (flag & 0x20) != 0 {
                                                ("★", egui::Color32::from_rgb(80, 220, 255), "Above Hero (Star ★)")
                                            } else if (flag & 0x10) != 0 {
                                                ("△", egui::Color32::from_rgb(255, 220, 0), "Counter / Bridge (Triangle △)")
                                            } else if (flag & 0x0F) == 0x0F {
                                                ("〇", egui::Color32::from_rgb(80, 255, 80), "Passable (Circle 〇)")
                                            } else if (flag & 0x0F) == 0 {
                                                ("✕", egui::Color32::from_rgb(255, 80, 80), "Blocked (Cross ✕)")
                                            } else {
                                                ("🠅", egui::Color32::from_rgb(255, 160, 60), "Directional Block")
                                            }
                                        }
                                        PassabilityEditMode::Directional => {
                                            let d = flag & 0x0F;
                                            let s = match d {
                                                15 => "ALL",
                                                0 => "NONE",
                                                _ => "DIR",
                                            };
                                            (s, egui::Color32::from_rgb(255, 180, 50), "Directional Passability")
                                        }
                                        PassabilityEditMode::Terrain => {
                                            let tid = cs.terrain_data.get(idx).copied().unwrap_or(1);
                                            let sym = match tid {
                                                1 => "T1", 2 => "T2", 3 => "T3", 4 => "T4",
                                                5 => "T5", 6 => "T6", 7 => "T7", 8 => "T8",
                                                _ => "T+",
                                            };
                                            (sym, egui::Color32::from_rgb(220, 150, 255), "Terrain Assignment")
                                        }
                                    };

                                    // Badge pill for readability
                                    let badge_r = (tile_size * 0.36).min(14.0);
                                    painter.circle_filled(rect.center(), badge_r, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 185));
                                    let font_sz = (tile_size * 0.44).clamp(10.0, 18.0);
                                    painter.text(rect.center(), egui::Align2::CENTER_CENTER, symbol, egui::FontId::proportional(font_sz), color);
                                    let resp = resp.on_hover_text(tooltip);

                                    if resp.clicked() {
                                        match self.edit_mode {
                                            PassabilityEditMode::General => {
                                                data[idx] = match flag {
                                                    15 => 0,
                                                    0 => 16,
                                                    16 => 32,
                                                    _ => 15,
                                                };
                                                *dirty = true;
                                            }
                                            PassabilityEditMode::Directional => {
                                                data[idx] = (flag + 1) & 0x0F;
                                                *dirty = true;
                                            }
                                            PassabilityEditMode::Terrain => {
                                                if idx < cs.terrain_data.len() {
                                                    cs.terrain_data[idx] = self.active_terrain_id;
                                                    *dirty = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                });
        }
    }
}


