use eframe::egui;
use std::fs;
use std::path::Path;
use crate::audio::AudioPlayer;
use crate::widgets::asset_viewer::AssetPreviewCache;

pub struct ResourceManagerDialogState {
    pub is_open: bool,
    pub active_category_idx: usize,
    pub selected_file: Option<String>,
    pub status_message: Option<Result<String, String>>,
}

pub const RESOURCE_CATEGORIES: &[&str] = &[
    "Backdrop",
    "Battle",
    "Battle2",
    "BattleCharSet",
    "BattleWeapon",
    "CharSet",
    "ChipSet",
    "FaceSet",
    "Frame",
    "GameOver",
    "Monster",
    "Movie",
    "Music",
    "Panorama",
    "Picture",
    "Sound",
    "System",
    "System2",
    "Title",
];

impl Default for ResourceManagerDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_category_idx: 5, // default CharSet
            selected_file: None,
            status_message: None,
        }
    }
}

impl ResourceManagerDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.status_message = None;
        self.selected_file = None;
    }

    pub fn list_category_files(&self, project_path: &str) -> Vec<String> {
        let cat = RESOURCE_CATEGORIES[self.active_category_idx];
        let dir = Path::new(project_path).join(cat);
        let mut results = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        results.push(fname);
                    }
                }
            }
        }
        results.sort();
        results
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project_path: Option<&str>,
        asset_cache: &mut AssetPreviewCache,
        audio: Option<&AudioPlayer>,
        open_soundfont_dialog: &mut bool,
    ) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;

        egui::Window::new(format!("📁 {}", rust_i18n::t!("res_mgr.title")))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([720.0, 520.0])
            .show(ctx, |ui| {
                let proj = match project_path {
                    Some(p) => p,
                    None => {
                        ui.label("Open a project to manage resources.");
                        return;
                    }
                };

                let cat = RESOURCE_CATEGORIES[self.active_category_idx];
                let files = self.list_category_files(proj);

                ui.columns(2, |cols| {
                    // Left Column: Category Tabs & File List
                    cols[0].vertical(|ui| {
                        ui.label(rust_i18n::t!("res_mgr.asset_type"));
                        egui::ScrollArea::horizontal()
                            .id_salt("res_mgr_cat_scroll")
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (i, &c) in RESOURCE_CATEGORIES.iter().enumerate() {
                                        let selected = self.active_category_idx == i;
                                        if ui.selectable_label(selected, c).clicked() {
                                            self.active_category_idx = i;
                                            self.selected_file = None;
                                            self.status_message = None;
                                            if let Some(a) = audio { a.stop(); }
                                        }
                                    }
                                });
                            });

                        ui.separator();
                        ui.label(rust_i18n::t!("res_mgr.files", count = files.len()));

                        egui::ScrollArea::vertical()
                            .id_salt("res_mgr_files_scroll")
                            .max_height(340.0)
                            .show(ui, |ui| {
                                for f in &files {
                                    let selected = self.selected_file.as_deref() == Some(f.as_str());
                                    if ui.selectable_label(selected, f).clicked() {
                                        self.selected_file = Some(f.clone());
                                        self.status_message = None;
                                        if let Some(a) = audio { a.stop(); }
                                    }
                                }
                            });
                    });

                    // Right Column: Preview & Actions
                    cols[1].vertical(|ui| {
                        ui.heading(rust_i18n::t!("res_mgr.asset_details"));
                        ui.separator();

                        if let Some(fname) = self.selected_file.clone() {
                            let file_path = Path::new(proj).join(cat).join(&fname);
                            let file_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                            let size_kb = (file_size as f64 / 1024.0).max(0.1);

                            ui.label(rust_i18n::t!("res_mgr.name", name = &fname));
                            ui.label(format!("Size: {:.1} KB", size_kb));

                            let stem = Path::new(&fname).file_stem().unwrap_or_default().to_string_lossy().to_string();

                            // Image preview
                            if let Some(tex) = asset_cache.get_or_load(ctx, proj, cat, &stem) {
                                ui.add(egui::Image::from_texture(&tex).max_width(260.0).max_height(240.0));
                            }

                            // Audio preview - only Music/Sound categories are playable.
                            if cat == "Music" || cat == "Sound" {
                                ui.horizontal(|ui| {
                                    if ui.button("▶ Play").clicked() {
                                        if let Some(a) = audio {
                                            match a.play_file(&file_path) {
                                                Ok(()) => {
                                                    self.status_message = None;
                                                }
                                                Err(e) if e == crate::audio::ERR_SOUNDFONT_MISSING => {
                                                    self.status_message = Some(Err("No SoundFont loaded. MIDI playback requires an .sf2 file.".to_string()));
                                                }
                                                Err(e) => {
                                                    self.status_message = Some(Err(e));
                                                }
                                            }
                                        } else {
                                            self.status_message = Some(Err("No audio output device available.".to_string()));
                                        }
                                    }
                                    if ui.button("⏹ Stop").clicked() {
                                        if let Some(a) = audio { a.stop(); }
                                    }
                                    if AudioPlayer::is_midi(&file_path) {
                                        ui.label("🎹 MIDI");
                                    }
                                });
                            }

                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button(format!("📤 Export Asset")).clicked() {
                                    if let Some(dest_path) = rfd::FileDialog::new()
                                        .set_title("Export Asset")
                                        .set_file_name(&fname)
                                        .save_file()
                                    {
                                        if let Err(e) = fs::copy(&file_path, &dest_path) {
                                            self.status_message = Some(Err(format!("Export failed: {}", e)));
                                        } else {
                                            self.status_message = Some(Ok(format!("Asset '{}' exported successfully.", &fname)));
                                        }
                                    }
                                }

                                if ui.button(format!("🗑 {}", rust_i18n::t!("res_mgr.delete"))).clicked() {
                                    if let Err(e) = fs::remove_file(&file_path) {
                                        self.status_message = Some(Err(format!("Failed to delete: {}", e)));
                                    } else {
                                        self.status_message = Some(Ok(rust_i18n::t!("res_mgr.deleted", name = &fname).to_string()));
                                        self.selected_file = None;
                                    }
                                }
                            });
                        } else {
                            let is_dark = ui.visuals().dark_mode;
                            ui.colored_label(crate::theme::colors::muted(is_dark), rust_i18n::t!("res_mgr.select_prompt"));
                        }

                        ui.separator();

                        if ui.button(format!("📥 {}", rust_i18n::t!("res_mgr.import"))).clicked() {
                            if let Some(src_path) = rfd::FileDialog::new()
                                .set_title("Import Asset")
                                .pick_file()
                            {
                                if let Some(file_name) = src_path.file_name() {
                                    let dest_dir = Path::new(proj).join(cat);
                                    let _ = fs::create_dir_all(&dest_dir);
                                    let dest_file = dest_dir.join(file_name);
                                    if let Err(e) = fs::copy(&src_path, &dest_file) {
                                        self.status_message = Some(Err(format!("Import failed: {}", e)));
                                    } else {
                                        let name_str = file_name.to_string_lossy().to_string();
                                        self.status_message = Some(Ok(rust_i18n::t!("res_mgr.imported", name = &name_str).to_string()));
                                        self.selected_file = Some(name_str);
                                    }
                                }
                            }
                        }
                    });
                });

                if let Some(msg) = &self.status_message {
                    let is_dark = ui.visuals().dark_mode;
                    let mut show_setup_button = false;
                    match msg {
                        Ok(t) => { ui.colored_label(crate::theme::colors::success(is_dark), t); }
                        Err(e) => {
                            ui.colored_label(crate::theme::colors::danger(is_dark), e);
                            if e.contains("SoundFont") {
                                show_setup_button = true;
                            }
                        }
                    }
                    if show_setup_button {
                        if ui.button("🎵 Configure SoundFont...").clicked() {
                            *open_soundfont_dialog = true;
                        }
                    }
                }
            });

        if !is_open {
            if let Some(a) = audio { a.stop(); }
            self.is_open = false;
        }
    }
}
