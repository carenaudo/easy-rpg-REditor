use eframe::egui;
use rfd::FileDialog;
use crate::lcf_bridge;

pub struct XmlIoDialogState {
    pub is_open: bool,
    pub status_message: Option<Result<String, String>>,
}

impl Default for XmlIoDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            status_message: None,
        }
    }
}

impl XmlIoDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.status_message = None;
    }

    pub fn show(&mut self, ctx: &egui::Context, project_path: Option<&str>, active_map_id: Option<i32>) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;

        egui::Window::new("XML Import / Export")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([460.0, 320.0])
            .show(ctx, |ui| {
                if let Some(proj) = project_path {
                    ui.label(format!("Project: {}", proj));
                    ui.separator();

                    ui.heading("Export XML");
                    ui.horizontal(|ui| {
                        if ui.button("Export Database to XML (LDB)").clicked() {
                            if let Some(path) = FileDialog::new().set_file_name("RPG_RT.edb").add_filter("XML", &["xml", "edb"]).save_file() {
                                match lcf_bridge::export_database_to_xml(proj, &path) {
                                    Ok(()) => self.status_message = Some(Ok(format!("Database exported to {:?}", path.file_name().unwrap()))),
                                    Err(e) => self.status_message = Some(Err(format!("Export failed: {e}"))),
                                }
                            }
                        }

                        if ui.button("Export Map Tree to XML (LMT)").clicked() {
                            if let Some(path) = FileDialog::new().set_file_name("RPG_RT.emt").add_filter("XML", &["xml", "emt"]).save_file() {
                                match lcf_bridge::export_tree_to_xml(proj, &path) {
                                    Ok(()) => self.status_message = Some(Ok(format!("Map tree exported to {:?}", path.file_name().unwrap()))),
                                    Err(e) => self.status_message = Some(Err(format!("Export failed: {e}"))),
                                }
                            }
                        }
                    });

                    if let Some(map_id) = active_map_id {
                        ui.horizontal(|ui| {
                            if ui.button(format!("Export Current Map #{:04} to XML (LMU)", map_id)).clicked() {
                                let default_name = format!("Map{:04}.emu", map_id);
                                if let Some(path) = FileDialog::new().set_file_name(&default_name).add_filter("XML", &["xml", "emu"]).save_file() {
                                    match lcf_bridge::export_map_to_xml(proj, map_id, &path) {
                                        Ok(()) => self.status_message = Some(Ok(format!("Map exported to {:?}", path.file_name().unwrap()))),
                                        Err(e) => self.status_message = Some(Err(format!("Export failed: {e}"))),
                                    }
                                }
                            }

                            if ui.button("Export Save01.lsd to XML (LSD)").clicked() {
                                if let Some(path) = FileDialog::new().set_file_name("Save01.esd").add_filter("XML", &["xml", "esd"]).save_file() {
                                    match lcf_bridge::export_save_to_xml(proj, "Save01.lsd", &path) {
                                        Ok(()) => self.status_message = Some(Ok(format!("Save exported to {:?}", path.file_name().unwrap()))),
                                        Err(e) => self.status_message = Some(Err(format!("Export failed: {e}"))),
                                    }
                                }
                            }
                        });
                    }

                    ui.separator();
                    ui.heading("Format Information");
                    ui.label("XML files exported from EasyRPG REditor use liblcf-compatible XML tags (LDB, LMT, LMU, LSD), enabling easy inspection and version control diffing.");

                    let is_dark = ui.visuals().dark_mode;
                    if let Some(msg) = &self.status_message {
                        ui.separator();
                        match msg {
                            Ok(text) => { ui.colored_label(crate::theme::colors::success(is_dark), text); }
                            Err(text) => { ui.colored_label(crate::theme::colors::danger(is_dark), text); }
                        }
                    }
                } else {
                    let is_dark = ui.visuals().dark_mode;
                    ui.colored_label(crate::theme::colors::warning(is_dark), "No project currently open.");
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
            });

        if !is_open {
            self.is_open = false;
        }
    }
}
