mod lcf_bridge;

use eframe::{egui, epi};
use image::load_from_memory;
use rfd::FileDialog;

struct EditorApp {
    project_path: Option<String>,
    maps: Vec<String>,
    selected_map: Option<usize>,
    chipset_texture: Option<egui::TextureHandle>,
}

impl epi::App for EditorApp {
    fn name(&self) -> &str {
        "EasyRPG Editor"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            if ui.button("Open Project").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    let path_str = path.display().to_string();
                    self.project_path = Some(path_str.clone());
                    self.maps = lcf_bridge::load_project(&path_str);
                }
            }
        });

        egui::SidePanel::left("map_list").show(ctx, |ui| {
            ui.heading("Maps");
            for (i, map) in self.maps.iter().enumerate() {
                if ui.selectable_label(self.selected_map == Some(i), map).clicked() {
                    self.selected_map = Some(i);
                    if let Some(proj) = &self.project_path {
                        let bytes = lcf_bridge::get_map_chipset(proj, (i + 1) as i32);
                        if !bytes.is_empty() {
                            if let Ok(img) = load_from_memory(&bytes) {
                                let size = [img.width() as usize, img.height() as usize];
                                let rgba = img.to_rgba8();
                                let tex = frame.tex_allocator().alloc_srgba_premultiplied(size, &rgba);
                                self.chipset_texture = Some(tex);
                            }
                        }
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.chipset_texture {
                ui.image(tex, tex.size_vec2());
            } else {
                ui.label("No map selected");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let app = EditorApp {
        project_path: None,
        maps: vec![],
        selected_map: None,
        chipset_texture: None,
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native("EasyRPG Editor", options, Box::new(|_cc| Box::new(app)))
}
