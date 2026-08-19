use eframe::egui;
use std::collections::BTreeSet;
use std::path::Path;
use crate::app_state::AppPersistentData;

pub fn list_available_resources(category: &str, project_path: Option<&str>) -> Vec<String> {
    let mut files = BTreeSet::new();

    let valid_exts = [
        "png", "xyz", "bmp", "PNG", "XYZ", "BMP",
        "mid", "wav", "mp3", "ogg", "wma", "MID", "WAV", "MP3", "OGG", "WMA"
    ];

    // 1. Scan Project directory
    if let Some(proj) = project_path {
        let dir = Path::new(proj).join(category);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if valid_exts.contains(&ext) {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                files.insert(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan RTP directory
    let cfg = AppPersistentData::load();
    if let Some(rtp) = cfg.get_effective_rtp_path() {
        let dir = rtp.join(category);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if valid_exts.contains(&ext) {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                files.insert(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    files.into_iter().collect()
}

pub fn resource_combo_box(
    ui: &mut egui::Ui,
    id_salt: &str,
    current_value: &mut String,
    category: &str,
    project_path: Option<&str>,
    dirty: &mut bool,
) {
    let items = list_available_resources(category, project_path);
    let is_audio = category == "Music" || category == "Sound";

    ui.horizontal(|ui| {
        let display_text = if current_value.is_empty() {
            "(None)".to_string()
        } else {
            current_value.clone()
        };

        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(display_text)
            .show_ui(ui, |ui| {
                if ui.selectable_value(current_value, String::new(), "(None)").clicked() {
                    *dirty = true;
                }
                for item in &items {
                    if ui.selectable_value(current_value, item.clone(), item).clicked() {
                        *dirty = true;
                    }
                }
            });

        if is_audio && !current_value.is_empty() {
            let is_playing_id = ui.make_persistent_id(format!("{}_playing", id_salt));
            let mut is_playing = ui.ctx().data_mut(|d| d.get_temp::<bool>(is_playing_id).unwrap_or(false));

            let btn_text = if is_playing { "⏹ Stop" } else { "▶ Play" };
            if ui.small_button(btn_text).clicked() {
                is_playing = !is_playing;
                ui.ctx().data_mut(|d| d.insert_temp(is_playing_id, is_playing));
            }
        }
    });
}
