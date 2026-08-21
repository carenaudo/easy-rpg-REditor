use eframe::egui;
use rfd::FileDialog;
use crate::lcf_bridge;

/// What changed in the open project after a successful import, so the
/// caller (`EditorApp`) knows what to reload. Consumed once via
/// `XmlIoDialogState::take_import()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XmlImportKind {
    /// `RPG_RT.ldb` was replaced - reload the whole project.
    Database,
    /// `RPG_RT.lmt` was replaced - reload the whole project (map list may
    /// have changed).
    Tree,
    /// The given map id's `.lmu` was replaced - reload just that map.
    Map(i32),
    /// A save slot file was replaced - reload just that slot.
    Save,
}

pub struct XmlIoDialogState {
    pub is_open: bool,
    pub status_message: Option<Result<String, String>>,
    /// Set by a successful import; the caller should call `take_import()`
    /// once per frame and reload accordingly.
    pending_import: Option<XmlImportKind>,
}

impl Default for XmlIoDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            status_message: None,
            pending_import: None,
        }
    }
}

impl XmlIoDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.status_message = None;
    }

    /// Returns and clears the most recent successful import, if any -
    /// callers should check this after every `show()` call and reload the
    /// affected project data.
    pub fn take_import(&mut self) -> Option<XmlImportKind> {
        self.pending_import.take()
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
            .default_size([460.0, 420.0])
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
                    ui.heading("Import XML");
                    ui.label("Importing replaces the matching project file in place. The original is backed up once (e.g. RPG_RT.ldb.bak) before it's overwritten.");
                    ui.horizontal(|ui| {
                        if ui.button("Import Database from XML (LDB)").clicked() {
                            if let Some(path) = FileDialog::new().add_filter("XML", &["xml", "edb"]).pick_file() {
                                match lcf_bridge::import_database_from_xml(proj, &path) {
                                    Ok(()) => {
                                        self.status_message = Some(Ok(format!("Database imported from {:?}", path.file_name().unwrap())));
                                        self.pending_import = Some(XmlImportKind::Database);
                                    }
                                    Err(e) => self.status_message = Some(Err(format!("Import failed: {e}"))),
                                }
                            }
                        }

                        if ui.button("Import Map Tree from XML (LMT)").clicked() {
                            if let Some(path) = FileDialog::new().add_filter("XML", &["xml", "emt"]).pick_file() {
                                match lcf_bridge::import_tree_from_xml(proj, &path) {
                                    Ok(()) => {
                                        self.status_message = Some(Ok(format!("Map tree imported from {:?}", path.file_name().unwrap())));
                                        self.pending_import = Some(XmlImportKind::Tree);
                                    }
                                    Err(e) => self.status_message = Some(Err(format!("Import failed: {e}"))),
                                }
                            }
                        }
                    });

                    if let Some(map_id) = active_map_id {
                        ui.horizontal(|ui| {
                            if ui.button(format!("Import Current Map #{:04} from XML (LMU)", map_id)).clicked() {
                                if let Some(path) = FileDialog::new().add_filter("XML", &["xml", "emu"]).pick_file() {
                                    match lcf_bridge::import_map_from_xml(proj, map_id, &path) {
                                        Ok(()) => {
                                            self.status_message = Some(Ok(format!("Map imported from {:?}", path.file_name().unwrap())));
                                            self.pending_import = Some(XmlImportKind::Map(map_id));
                                        }
                                        Err(e) => self.status_message = Some(Err(format!("Import failed: {e}"))),
                                    }
                                }
                            }

                            if ui.button("Import Save01.lsd from XML (LSD)").clicked() {
                                if let Some(path) = FileDialog::new().add_filter("XML", &["xml", "esd"]).pick_file() {
                                    match lcf_bridge::import_save_from_xml(proj, "Save01.lsd", &path) {
                                        Ok(()) => {
                                            self.status_message = Some(Ok(format!("Save imported from {:?}", path.file_name().unwrap())));
                                            self.pending_import = Some(XmlImportKind::Save);
                                        }
                                        Err(e) => self.status_message = Some(Err(format!("Import failed: {e}"))),
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
