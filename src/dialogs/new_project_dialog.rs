use eframe::egui;
use std::fs;
use std::path::PathBuf;
use lcf_core::ldb::LdbReader;
use lcf_core::lmt::LmtReader;
use lcf_core::lmu::LmuReader;
use lcf_core::types::{DBString, EngineVersion, TreeMap as LmtTreeMap};
use lcf_core::generated::ldb_gen::{Database as LdbDatabase, Chipset as LdbChipset, Actor as LdbActor};
use lcf_core::generated::lmt_gen::MapInfo as LmtMapInfo;
use lcf_core::generated::lmu_gen::Map as LmuMap;

pub struct NewProjectDialogState {
    pub is_open: bool,
    pub project_title: String,
    pub destination_dir: String,
    pub is_2003: bool,
    /// Maniac Patch is a RPG Maker 2003-only engine extension; when set,
    /// `create_project` writes an `EasyRPG.ini` declaring `[Patch] Maniac=1`
    /// (the same authoritative signal `lcf_bridge::detect_maniac_patch`
    /// checks for), so the new project is recognized as Maniac immediately.
    pub is_maniac: bool,
    pub status_message: Option<Result<String, String>>,
}

impl Default for NewProjectDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            project_title: "My New RPG".to_string(),
            destination_dir: String::new(),
            is_2003: false,
            is_maniac: false,
            status_message: None,
        }
    }
}

impl NewProjectDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.status_message = None;
    }

    pub fn create_project(&self) -> Result<PathBuf, String> {
        if self.destination_dir.trim().is_empty() {
            return Err("Destination folder cannot be empty.".to_string());
        }

        let base_path = PathBuf::from(&self.destination_dir);
        let project_dir = if base_path.file_name().map(|n| n.to_string_lossy().to_string()) == Some(self.project_title.clone()) {
            base_path
        } else {
            base_path.join(&self.project_title)
        };

        fs::create_dir_all(&project_dir).map_err(|e| format!("Failed to create project folder: {}", e))?;

        // Standard Asset Folders
        let mut folders = vec![
            "Backdrop", "Battle", "Battle2", "CharSet", "ChipSet", "FaceSet",
            "GameOver", "Monster", "Movie", "Music", "Panorama", "Picture", "Sound", "System", "Title",
        ];
        if self.is_2003 {
            folders.push("System2");
            folders.push("Frame");
            folders.push("BattleCharSet");
            folders.push("BattleWeapon");
        }
        for f in &folders {
            let _ = fs::create_dir_all(project_dir.join(f));
        }

        let engine = if self.is_2003 { EngineVersion::Engine2003 } else { EngineVersion::Engine2000 };

        // 1. Create Default LDB
        let mut db = LdbDatabase::default_for_engine(self.is_2003);

        let mut default_actor = LdbActor::default();
        default_actor.id = 1;
        default_actor.name = DBString::new("Hero".to_string());
        db.actors.push(default_actor);

        let mut default_chipset = LdbChipset::default();
        default_chipset.id = 1;
        default_chipset.name = DBString::new("World".to_string());
        db.chipsets.push(default_chipset);

        let ldb_path = project_dir.join("RPG_RT.ldb");
        LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| format!("Failed to save RPG_RT.ldb: {}", e))?;

        // RPG_RT.ini
        let ini_content = format!("[RPG_RT]\r\nGameTitle={}\r\nMapEditMode=2\r\nMapEditZoom=0\r\n", self.project_title);
        let _ = fs::write(project_dir.join("RPG_RT.ini"), ini_content);

        // EasyRPG.ini - declares Maniac Patch support so
        // lcf_bridge::detect_maniac_patch recognizes this project
        // immediately (Maniac is 2003-only, mirrors real Maniac-patched
        // games such as the TestGame-Maniac fixture).
        if self.is_2003 && self.is_maniac {
            let easyrpg_ini = "[Game]\r\nEngine=rpg2k3e\r\n\r\n[Patch]\r\nManiac=1\r\n";
            let _ = fs::write(project_dir.join("EasyRPG.ini"), easyrpg_ini);
        }

        // 2. Create Default LMT
        let mut tree = LmtTreeMap::default();
        let mut map_info = LmtMapInfo::default();
        map_info.id = 1;
        map_info.name = DBString::new("Map0001".to_string());
        map_info.parent_map = 0;
        map_info.r#type = 1;
        tree.maps.push(map_info);
        tree.tree_order.push(1);
        tree.start.party_map_id = 1;
        tree.start.party_x = 10;
        tree.start.party_y = 7;

        let lmt_path = project_dir.join("RPG_RT.lmt");
        LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| format!("Failed to save RPG_RT.lmt: {}", e))?;

        // 3. Create Default Map0001.lmu
        let mut map = LmuMap::default();
        map.chipset_id = 1;
        map.width = 20;
        map.height = 15;
        let total = (map.width * map.height) as usize;
        map.lower_layer = vec![4000; total];
        map.upper_layer = vec![10000; total];

        let lmu_path = project_dir.join("Map0001.lmu");
        LmuReader::save(&lmu_path, &map, engine, "auto").map_err(|e| format!("Failed to save Map0001.lmu: {}", e))?;

        Ok(project_dir)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.is_open {
            return None;
        }

        let mut created_path = None;
        let mut is_open = self.is_open;

        egui::Window::new(format!("✨ {}", rust_i18n::t!("new_proj.title")))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                egui::Grid::new("new_project_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Engine Version:");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.is_2003, false, "🎮 RPG Maker 2000");
                            ui.radio_value(&mut self.is_2003, true, "⚔ RPG Maker 2003");
                        });
                        ui.end_row();

                        if !self.is_2003 {
                            self.is_maniac = false;
                        }
                        ui.label("Engine Extension:");
                        ui.add_enabled_ui(self.is_2003, |ui| {
                            ui.checkbox(&mut self.is_maniac, "🔧 Enable Maniac Patch")
                                .on_hover_text("Maniac Patch is a RPG Maker 2003-only engine extension. Writes an EasyRPG.ini declaring [Patch] Maniac=1 so the editor recognizes this project as Maniac immediately.")
                                .on_disabled_hover_text("Maniac Patch requires RPG Maker 2003.");
                        });
                        ui.end_row();

                        ui.label(rust_i18n::t!("new_proj.project_title"));
                        ui.text_edit_singleline(&mut self.project_title);
                        ui.end_row();

                        ui.label(rust_i18n::t!("new_proj.destination_folder"));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.destination_dir);
                            if ui.button(rust_i18n::t!("new_proj.browse")).clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    self.destination_dir = dir.to_string_lossy().to_string();
                                }
                            }
                        });
                        ui.end_row();
                    });

                ui.separator();

                if let Some(msg) = &self.status_message {
                    let is_dark = ui.visuals().dark_mode;
                    match msg {
                        Ok(t) => { ui.colored_label(crate::theme::colors::success(is_dark), t); }
                        Err(e) => { ui.colored_label(crate::theme::colors::danger(is_dark), e); }
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button(rust_i18n::t!("new_proj.create_button")).clicked() {
                        match self.create_project() {
                            Ok(path) => {
                                created_path = Some(path.to_string_lossy().to_string());
                                self.is_open = false;
                            }
                            Err(e) => {
                                self.status_message = Some(Err(e));
                            }
                        }
                    }
                    if ui.button(rust_i18n::t!("new_proj.cancel")).clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }

        created_path
    }
}
