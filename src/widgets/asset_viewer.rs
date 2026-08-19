use eframe::egui;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::app_state::AppPersistentData;

#[derive(Default)]
pub struct AssetPreviewCache {
    textures: HashMap<String, egui::TextureHandle>,
}

impl AssetPreviewCache {
    fn find_file_in_dir(dir: &Path, category: &str, file_name: &str) -> Option<PathBuf> {
        let cat_dir = dir.join(category);
        let extensions = ["png", "xyz", "bmp", "jpg", "jpeg", "PNG", "XYZ", "BMP"];
        for ext in &extensions {
            let p = cat_dir.join(format!("{}.{}", file_name, ext));
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    pub fn get_or_load(
        &mut self,
        ctx: &egui::Context,
        project_path: &str,
        category: &str,
        file_name: &str,
    ) -> Option<egui::TextureHandle> {
        if file_name.is_empty() {
            return None;
        }

        let key = format!("{}/{}/{}", project_path, category, file_name);
        if let Some(tex) = self.textures.get(&key) {
            return Some(tex.clone());
        }

        // 1. Check Project Directory
        let mut path_opt = Self::find_file_in_dir(Path::new(project_path), category, file_name);

        // 2. Fallback to RTP Directory
        if path_opt.is_none() {
            let config = AppPersistentData::load();
            if let Some(rtp_dir) = config.get_effective_rtp_path() {
                path_opt = Self::find_file_in_dir(&rtp_dir, category, file_name);
            }
        }

        let path = path_opt?;
        let bytes = std::fs::read(&path).ok()?;
        let img = crate::tilemap::decode_rpg_image(&bytes).ok()?;

        let size = [img.width() as usize, img.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img);
        let tex = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::NEAREST);
        self.textures.insert(key, tex.clone());
        Some(tex)
    }
}

/// Helper function to draw a checkerboard background for transparency preview matching theme
pub fn draw_checkerboard(painter: &egui::Painter, rect: egui::Rect, cell_size: f32, is_dark: bool) {
    let (c1, c2) = crate::theme::colors::checkerboard(is_dark);
    let cols = (rect.width() / cell_size).ceil() as usize;
    let rows = (rect.height() / cell_size).ceil() as usize;
    for r in 0..rows {
        for c in 0..cols {
            let color = if (r + c) % 2 == 0 { c1 } else { c2 };
            let min = egui::pos2(
                (rect.min.x + c as f32 * cell_size).min(rect.max.x),
                (rect.min.y + r as f32 * cell_size).min(rect.max.y),
            );
            let max = egui::pos2(
                (min.x + cell_size).min(rect.max.x),
                (min.y + cell_size).min(rect.max.y),
            );
            painter.rect_filled(egui::Rect::from_min_max(min, max), 0.0, color);
        }
    }
}
