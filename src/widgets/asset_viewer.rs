use eframe::egui;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::app_state::AppPersistentData;

pub const MAX_ASSET_CACHE_ENTRIES: usize = 256;

#[derive(Default)]
pub struct AssetPreviewCache {
    textures: HashMap<String, egui::TextureHandle>,
    access_order: Vec<String>,
    rtp_path: Option<PathBuf>,
}

impl AssetPreviewCache {
    /// Drops every cached texture, forcing the next `get_or_load` call for
    /// each asset to re-read it from disk. Call this whenever project
    /// files may have changed out from under the cache - e.g. after an XML
    /// import replaces a chipset/charset reference.
    pub fn clear(&mut self) {
        self.textures.clear();
        self.access_order.clear();
        self.rtp_path = None;
    }

    pub fn set_rtp_path(&mut self, path: Option<PathBuf>) {
        self.rtp_path = path;
    }

    pub fn get_rtp_path(&mut self) -> Option<&Path> {
        if self.rtp_path.is_none() {
            let config = AppPersistentData::load();
            self.rtp_path = config.get_effective_rtp_path();
        }
        self.rtp_path.as_deref()
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    pub fn find_file_in_dir(dir: &Path, category: &str, file_name: &str) -> Option<PathBuf> {
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

    pub fn find_asset_file_with_rtp(project_path: &str, category: &str, file_name: &str, rtp_dir: Option<&Path>) -> Option<PathBuf> {
        if file_name.is_empty() {
            return None;
        }
        // 1. Check Project Directory
        if let Some(p) = Self::find_file_in_dir(Path::new(project_path), category, file_name) {
            return Some(p);
        }
        // 2. Fallback to RTP Directory
        if let Some(rtp) = rtp_dir {
            if let Some(p) = Self::find_file_in_dir(rtp, category, file_name) {
                return Some(p);
            }
        }
        None
    }

    pub fn find_asset_file(project_path: &str, category: &str, file_name: &str) -> Option<PathBuf> {
        let config = AppPersistentData::load();
        Self::find_asset_file_with_rtp(project_path, category, file_name, config.get_effective_rtp_path().as_deref())
    }

    pub fn load_asset_bytes_with_rtp(project_path: &str, category: &str, file_name: &str, rtp_dir: Option<&Path>) -> Option<Vec<u8>> {
        let p = Self::find_asset_file_with_rtp(project_path, category, file_name, rtp_dir)?;
        std::fs::read(p).ok()
    }

    pub fn load_asset_bytes(project_path: &str, category: &str, file_name: &str) -> Option<Vec<u8>> {
        let config = AppPersistentData::load();
        Self::load_asset_bytes_with_rtp(project_path, category, file_name, config.get_effective_rtp_path().as_deref())
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
            // Refresh LRU position
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key);
            return Some(tex.clone());
        }

        let rtp_dir = self.get_rtp_path().map(|p| p.to_path_buf());
        let bytes = Self::load_asset_bytes_with_rtp(project_path, category, file_name, rtp_dir.as_deref())?;
        let is_opaque_bg = category == "Panorama" || category == "Title" || category == "GameOver" || category == "Battle" || category == "Battle2";
        let img = crate::tilemap::decode_rpg_image_with_alpha(&bytes, !is_opaque_bg).ok()?;

        // Enforce cache capacity limit
        while self.textures.len() >= MAX_ASSET_CACHE_ENTRIES && !self.access_order.is_empty() {
            let oldest = self.access_order.remove(0);
            self.textures.remove(&oldest);
        }

        let size = [img.width() as usize, img.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img);
        let tex = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::NEAREST);
        self.textures.insert(key.clone(), tex.clone());
        self.access_order.push(key);
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
