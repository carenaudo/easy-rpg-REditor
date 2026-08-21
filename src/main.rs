use eframe::egui;
use image::RgbaImage;
use rfd::FileDialog;
use std::path::Path;

use easy_editor::app_state::{DbCategory, EditorAppState, ViewMode};
use easy_editor::audio::AudioPlayer;
use easy_editor::dialogs::map_properties_dialog::MapPropertiesDialogState;
use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;
use easy_editor::dialogs::project_analyzer_dialog::ProjectAnalyzerDialog;
use easy_editor::dialogs::project_search::{ProjectSearchDialog, SearchJumpTarget};
use easy_editor::dialogs::resource_manager_dialog::ResourceManagerDialogState;
use easy_editor::dialogs::sound_test_dialog::SoundTestDialog;
use easy_editor::dialogs::soundfont_dialog::SoundFontDialog;
use easy_editor::dialogs::xml_io_dialog::{XmlImportKind, XmlIoDialogState};
use easy_editor::lcf_bridge;
use easy_editor::theme::{self, ThemeMode, ThemePalette};
use easy_editor::tilemap;
use easy_editor::views;
use easy_editor::views::database_view::DatabaseViewState;
use easy_editor::views::map_view::{MapDims, MapDrawTool, MapLayerMode, MapViewState};
use easy_editor::views::save_view::SaveViewState;
use easy_editor::widgets::asset_viewer::AssetPreviewCache;
use easy_editor::widgets::map_tree::{MapTreeAction, MapTreeWidget};

rust_i18n::i18n!("locales", fallback = "en");

struct EditorApp {
    state: EditorAppState,
    map_view: MapViewState,
    database_view: DatabaseViewState,
    save_view: SaveViewState,
    map_tree_widget: MapTreeWidget,
    map_props_dialog: MapPropertiesDialogState,
    res_mgr_dialog: ResourceManagerDialogState,
    new_project_dialog: NewProjectDialogState,
    search_dialog: ProjectSearchDialog,
    xml_dialog: XmlIoDialogState,
    sound_test_dialog: SoundTestDialog,
    soundfont_dialog: SoundFontDialog,
    analyzer_dialog: ProjectAnalyzerDialog,
    asset_cache: AssetPreviewCache,
    cached_chipset: Option<RgbaImage>,
    passability: lcf_bridge::Passability,
    open_blocked_message: Option<String>,
    /// `None` when no audio output device is available - playback controls
    /// stay disabled/inert rather than the app failing to start.
    audio: Option<AudioPlayer>,
    /// Shows the Save All / Discard / Cancel choice when the window close
    /// button is clicked while there are unsaved changes.
    show_close_confirm: bool,
    /// Set right before re-issuing `ViewportCommand::Close` from the modal's
    /// Discard/Save-and-close actions, so the close guard below doesn't
    /// intercept that second close request and reopen the same modal in an
    /// infinite loop (discarding doesn't itself clear the dirty flags the
    /// guard checks, so without this the request would just bounce forever).
    force_close: bool,
}

impl EditorApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        let state = EditorAppState::default();
        theme::setup_fonts(&cc.egui_ctx);
        state.config.theme.apply(&cc.egui_ctx);
        rust_i18n::set_locale(&state.config.locale);

        let sf_manager = easy_editor::audio::SoundFontManager::new();
        if let Some(ref sf_path) = state.config.soundfont_path {
            let _ = sf_manager.load(Path::new(sf_path));
        } else if let Some(detected) = easy_editor::audio::SoundFontManager::detect_soundfont(
            None,
            state.config.rtp_path.as_deref(),
            state.config.last_project.as_deref(),
        ) {
            let _ = sf_manager.load(&detected);
        }

        let audio = AudioPlayer::new(sf_manager);

        let mut app = Self {
            state,
            map_view: MapViewState::default(),
            database_view: DatabaseViewState::default(),
            save_view: SaveViewState::default(),
            map_tree_widget: MapTreeWidget::default(),
            map_props_dialog: MapPropertiesDialogState::default(),
            res_mgr_dialog: ResourceManagerDialogState::default(),
            new_project_dialog: NewProjectDialogState::default(),
            search_dialog: ProjectSearchDialog::default(),
            xml_dialog: XmlIoDialogState::default(),
            sound_test_dialog: SoundTestDialog::default(),
            soundfont_dialog: SoundFontDialog::default(),
            analyzer_dialog: ProjectAnalyzerDialog::default(),
            asset_cache: AssetPreviewCache::default(),
            cached_chipset: None,
            passability: lcf_bridge::Passability::default(),
            open_blocked_message: None,
            audio,
            show_close_confirm: false,
            force_close: false,
        };

        if let Some(last_proj) = app.state.config.last_project.clone() {
            if Path::new(&last_proj).exists() {
                app.load_project(last_proj, &cc.egui_ctx);
            }
        }

        app
    }

    fn load_project(&mut self, path: String, ctx: &egui::Context) {
        self.state.load_project_from(path.clone());
        self.map_view.map_dims = None;
        self.map_view.map_texture = None;
        self.map_view.events.clear();
        self.map_view.events_dirty = false;
        self.map_view.map_dirty = false;
        self.map_view.hover_tile = None;
        self.cached_chipset = None;

        if let Some(proj) = &self.state.project_path {
            self.map_view.start_points = lcf_bridge::get_start_points(proj);
        }

        // Auto-select first map if available
        if !self.state.maps.is_empty() {
            self.select_map(0, ctx);
        }
    }

    /// Closes the current project and returns to the "no project loaded"
    /// state, so the developer can load a different project without
    /// restarting the editor. Callers must check for unsaved changes first.
    fn close_current_project(&mut self) {
        self.state.close_project();
        self.map_view.map_dims = None;
        self.map_view.map_texture = None;
        self.map_view.events.clear();
        self.map_view.events_dirty = false;
        self.map_view.map_dirty = false;
        self.map_view.hover_tile = None;
        self.map_view.start_points = Default::default();
        self.map_view.undo_stack.clear();
        self.map_view.save_message = None;
        self.cached_chipset = None;
        self.open_blocked_message = None;
    }

    fn select_map(&mut self, map_idx: usize, ctx: &egui::Context) {
        if map_idx >= self.state.maps.len() {
            return;
        }

        self.state.selected_map = Some(map_idx);
        let (map_id, _) = self.state.maps[map_idx];

        if let Some(proj) = &self.state.project_path {
            let chipset_bytes = lcf_bridge::get_map_chipset(proj, map_id);
            let layers = lcf_bridge::get_map_layers(proj, map_id);
            let events = lcf_bridge::get_map_events(proj, map_id);
            let passability = lcf_bridge::get_chipset_passability(proj, map_id);

            self.map_view.events = events;
            self.map_view.events_dirty = false;
            self.map_view.map_dirty = false;
            self.map_view.save_message = None;
            self.map_view.undo_stack.clear();
            self.map_view.hover_tile = None;
            self.passability = passability;
            self.map_view.map_dims = Some(MapDims {
                width: layers.width,
                height: layers.height,
                lower: layers.lower_layer,
                upper: layers.upper_layer,
            });

            if !chipset_bytes.is_empty() {
                if let Ok(chipset_rgba) = tilemap::decode_chipset(&chipset_bytes) {
                    self.map_view.palette.reload_chipset(ctx, &chipset_rgba);
                    self.map_view.refresh_texture(ctx, &chipset_rgba);
                    self.cached_chipset = Some(chipset_rgba);
                }
            }
        }
    }

    fn launch_playtest(&mut self) {
        if let Some(proj) = &self.state.project_path {
            let candidates = ["Player.exe", "easyrpg.exe", "RPG_RT.exe", "Player", "easyrpg", "RPG_RT"];
            let mut found_exe = None;
            for cand in &candidates {
                let p = Path::new(proj).join(cand);
                if p.exists() {
                    found_exe = Some(p);
                    break;
                }
            }

            let exe = found_exe.unwrap_or_else(|| std::path::PathBuf::from("Player.exe"));
            match std::process::Command::new(exe).current_dir(proj).arg("--test-play").spawn() {
                Ok(_) => {
                    self.open_blocked_message = None;
                }
                Err(e) => {
                    self.open_blocked_message = Some(rust_i18n::t!("dialog.playtest_error", error = e.to_string()).to_string());
                }
            }
        }
    }

    /// Saves whatever the active view/mode considers "current" - the
    /// selected map in Maps mode, or the selected database category in
    /// Database mode. Shared by the Project menu's Save item and the
    /// global Ctrl+S shortcut.
    fn save_current(&mut self) {
        match self.state.view_mode {
            ViewMode::Database => self.state.save_current_db_category(),
            ViewMode::Maps => {
                let map_id = self.state.selected_map.and_then(|i| self.state.maps.get(i)).map(|m| m.0);
                self.map_view.save_current_map(self.state.project_path.as_deref(), map_id);
            }
            ViewMode::Saves => {}
        }
    }

    /// Context strip: app icon, project name, current map, and a dirty dot.
    fn ui_context_strip(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("context_strip").show(ui, |ui| {
            ui.horizontal(|ui| {
                let is_dark = ui.visuals().dark_mode;
                ui.label("⚙");

                let mut text = rust_i18n::t!("app.title").to_string();
                if let Some(proj) = &self.state.project_path {
                    let proj_name = Path::new(proj)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| proj.clone());
                    text.push_str(" — ");
                    text.push_str(&proj_name);
                    if self.state.view_mode == ViewMode::Maps {
                        if let Some((map_id, map_name)) = self.state.selected_map.and_then(|i| self.state.maps.get(i)) {
                            text.push_str(&format!(" · {:04}: {}", map_id, map_name));
                        }
                    }
                }
                ui.colored_label(crate::theme::colors::muted(is_dark), text);

                let dirty = self.state.has_unsaved_changes() || self.map_view.map_dirty || self.map_view.events_dirty;
                if dirty {
                    ui.colored_label(crate::theme::colors::warning(is_dark), "●");
                }
            });
        });
    }

    /// 🌐 language icon popup.
    fn ui_language_popup(&mut self, ui: &mut egui::Ui) {
        let current_loc = rust_i18n::locale();
        let loc_str: &str = &current_loc;
        ui.menu_button("🌐", |ui| {
            for (loc, name) in &[
                ("en", "English"),
                ("es", "Español"),
                ("ja", "日本語"),
                ("de", "Deutsch"),
                ("fr", "Français"),
                ("it", "Italiano"),
                ("pt-BR", "Português (Brasil)"),
                ("zh-CN", "简体中文"),
            ] {
                if ui.selectable_label(loc_str == *loc, *name).clicked() {
                    rust_i18n::set_locale(loc);
                    self.state.config.locale = loc.to_string();
                    self.state.config.save();
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text(rust_i18n::t!("menu.language"));
    }

    /// 🎨 theme icon popup - palette and dark/light/system mode together.
    fn ui_theme_popup(&mut self, ui: &mut egui::Ui) {
        let mut current_palette = self.state.config.theme.palette;
        let mut current_mode = self.state.config.theme.mode;
        let label = format!("🎨 {}", current_palette.name());

        ui.menu_button(label, |ui| {
            let palettes = [
                ThemePalette::Zinc,
                ThemePalette::Slate,
                ThemePalette::Stone,
                ThemePalette::Gray,
                ThemePalette::Neutral,
                ThemePalette::Blue,
                ThemePalette::Violet,
                ThemePalette::Rose,
                ThemePalette::Orange,
                ThemePalette::Green,
                ThemePalette::Yellow,
                ThemePalette::Red,
            ];
            for p in palettes {
                if ui.selectable_value(&mut current_palette, p, p.name()).clicked() {
                    self.state.config.theme.palette = p;
                    self.state.config.theme.apply(ui.ctx());
                    self.state.config.save();
                }
            }

            ui.separator();

            if ui.selectable_value(&mut current_mode, ThemeMode::Dark, "🌙 Dark").clicked() {
                self.state.config.theme.mode = ThemeMode::Dark;
                self.state.config.theme.apply(ui.ctx());
                self.state.config.save();
            }
            if ui.selectable_value(&mut current_mode, ThemeMode::Light, "☀️ Light").clicked() {
                self.state.config.theme.mode = ThemeMode::Light;
                self.state.config.theme.apply(ui.ctx());
                self.state.config.save();
            }
            if ui.selectable_value(&mut current_mode, ThemeMode::System, "💻 System").clicked() {
                self.state.config.theme.mode = ThemeMode::System;
                self.state.config.theme.apply(ui.ctx());
                self.state.config.save();
            }
        })
        .response
        .on_hover_text(format!("{}: {}", rust_i18n::t!("menu.theme"), current_palette.name()));
    }

    /// 📦 RTP folder icon popup.
    fn ui_rtp_popup(&mut self, ui: &mut egui::Ui) {
        let project_version = self.state.project_path.is_some().then_some(self.state.is_2003);
        let rtp_display = self
            .state
            .config
            .get_effective_rtp_path_for(project_version)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "(Not Configured)".to_string());

        ui.menu_button("📦", |ui| {
            ui.label(format!("RTP Folder:\n{}", rtp_display));
            ui.separator();
            if ui.button(rust_i18n::t!("menu.rtp")).clicked() {
                if let Some(folder) = FileDialog::new().set_title("Select EasyRPG / RPG Maker RTP Folder").pick_folder() {
                    self.state.config.rtp_path = Some(folder.to_string_lossy().to_string());
                    self.state.config.save();
                }
                ui.close();
            }
        })
        .response
        .on_hover_text(format!("RTP Folder:\n{}\n\nClick to change RTP directory.", rtp_display));
    }

    /// ⚙ preferences icon popup - a compact overview of the three settings
    /// above, per the mockup's fourth icon. Actual changes happen via the
    /// dedicated 🌐/🎨/📦 popups; this just summarizes current state.
    fn ui_preferences_popup(&mut self, ui: &mut egui::Ui) {
        let project_version = self.state.project_path.is_some().then_some(self.state.is_2003);
        let rtp_display = self
            .state
            .config
            .get_effective_rtp_path_for(project_version)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "(Not Configured)".to_string());

        ui.menu_button("⚙", |ui| {
            ui.label(rust_i18n::t!("menu.preferences"));
            ui.separator();
            ui.label(format!("{}: {}", rust_i18n::t!("menu.language"), &*rust_i18n::locale()));
            ui.label(format!(
                "{}: {} / {:?}",
                rust_i18n::t!("menu.theme"),
                self.state.config.theme.palette.name(),
                self.state.config.theme.mode
            ));
            ui.label(format!("RTP: {}", rtp_display));
            ui.separator();
            ui.small("Use the 🌐 🎨 📦 icons to change these settings.");
        })
        .response
        .on_hover_text(rust_i18n::t!("menu.preferences"));
    }

    /// Menu bar: five dropdown menus, the Maps/Database/Saves mode switch,
    /// the Playtest button, and the right-aligned preference icon popups.
    fn ui_menubar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menubar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button(rust_i18n::t!("menu.project"), |ui| {
                    if ui.button(format!("✨ {}", rust_i18n::t!("menu.new_project"))).clicked() {
                        self.new_project_dialog.open();
                        ui.close();
                    }
                    if ui.button(rust_i18n::t!("menu.open_project")).clicked() {
                        if self.state.has_unsaved_changes() || self.map_view.map_dirty || self.map_view.events_dirty {
                            self.open_blocked_message = Some(rust_i18n::t!("dialog.unsaved_prompt").to_string());
                        } else {
                            self.open_blocked_message = None;
                            if let Some(path) = FileDialog::new().pick_folder() {
                                let path_str = path.display().to_string();
                                self.load_project(path_str, ui.ctx());
                            }
                        }
                        ui.close();
                    }
                    if !self.state.config.recent_projects.is_empty() {
                        ui.menu_button(rust_i18n::t!("menu.recent_projects"), |ui| {
                            for p in &self.state.config.recent_projects.clone() {
                                if ui.button(p).clicked() {
                                    if !self.state.has_unsaved_changes() && !self.map_view.map_dirty && !self.map_view.events_dirty {
                                        self.load_project(p.clone(), ui.ctx());
                                    } else {
                                        self.open_blocked_message = Some(rust_i18n::t!("dialog.unsaved_prompt").to_string());
                                    }
                                    ui.close();
                                }
                            }
                        });
                    }
                    ui.separator();
                    let can_save = match self.state.view_mode {
                        ViewMode::Database => self.state.has_unsaved_changes(),
                        ViewMode::Maps => self.map_view.map_dirty || self.map_view.events_dirty,
                        ViewMode::Saves => false,
                    };
                    if ui.add_enabled(can_save, egui::Button::new(format!("💾 {}", rust_i18n::t!("map.save_map")))).clicked() {
                        self.save_current();
                        ui.close();
                    }
                    ui.separator();
                    let has_project = self.state.project_path.is_some();
                    if ui.add_enabled(has_project, egui::Button::new(format!("🗙 {}", rust_i18n::t!("menu.close_project")))).clicked() {
                        if self.state.has_unsaved_changes() || self.map_view.map_dirty || self.map_view.events_dirty {
                            self.open_blocked_message = Some(rust_i18n::t!("dialog.unsaved_prompt").to_string());
                        } else {
                            self.close_current_project();
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("🚪 {}", rust_i18n::t!("menu.exit"))).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });

                ui.menu_button(rust_i18n::t!("menu.edit"), |ui| {
                    let in_maps = self.state.view_mode == ViewMode::Maps;
                    let can_undo = in_maps && self.map_view.undo_stack.can_undo();
                    let can_redo = in_maps && self.map_view.undo_stack.can_redo();
                    if ui.add_enabled(can_undo, egui::Button::new(format!("⟲ {}", rust_i18n::t!("menu.undo")))).clicked() {
                        self.map_view.undo(ui.ctx(), self.cached_chipset.as_ref());
                        ui.close();
                    }
                    if ui.add_enabled(can_redo, egui::Button::new(format!("⟳ {}", rust_i18n::t!("menu.redo")))).clicked() {
                        self.map_view.redo(ui.ctx(), self.cached_chipset.as_ref());
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("🔍 {}", rust_i18n::t!("menu.search"))).clicked() {
                        self.search_dialog.open();
                        ui.close();
                    }
                    if ui.add_enabled(in_maps, egui::Button::new(format!("↔ {}", rust_i18n::t!("menu.shift_map")))).clicked() {
                        self.map_view.show_shift_dialog = true;
                        ui.close();
                    }
                });

                ui.menu_button(rust_i18n::t!("menu.resources"), |ui| {
                    if ui.button(format!("📁 {}", rust_i18n::t!("menu.resources"))).clicked() {
                        self.res_mgr_dialog.open();
                        ui.close();
                    }
                    if ui.button(format!("🎵 {}", rust_i18n::t!("menu.sound_test"))).clicked() {
                        self.sound_test_dialog.open(self.state.project_path.as_deref());
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("📄 {}", rust_i18n::t!("menu.xml_io"))).clicked() {
                        self.xml_dialog.open();
                        ui.close();
                    }
                    if ui.button(format!("📦 {}", rust_i18n::t!("menu.rtp"))).clicked() {
                        if let Some(folder) = FileDialog::new().set_title("Select EasyRPG / RPG Maker RTP Folder").pick_folder() {
                            self.state.config.rtp_path = Some(folder.to_string_lossy().to_string());
                            self.state.config.save();
                        }
                        ui.close();
                    }
                    if ui.button("🎹 SoundFont (.sf2)...").clicked() {
                        self.soundfont_dialog.open();
                        ui.close();
                    }
                });

                ui.menu_button(rust_i18n::t!("menu.tools"), |ui| {
                    if ui.button(format!("🩺 {}", rust_i18n::t!("menu.project_health"))).clicked() {
                        self.analyzer_dialog.open(&self.state);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("▶ {}", rust_i18n::t!("menu.playtest"))).clicked() {
                        self.launch_playtest();
                        ui.close();
                    }
                });

                ui.menu_button(rust_i18n::t!("menu.help"), |ui| {
                    ui.label(rust_i18n::t!("app.title"));
                    ui.label("RPG Maker 2000 & 2003 Game Development Suite");
                });

                ui.separator();

                let maps_title = format!("🗺 {}", rust_i18n::t!("views.maps"));
                let db_title = format!("🗄 {}", rust_i18n::t!("views.database"));
                let saves_title = format!("💾 {}", rust_i18n::t!("views.saves"));
                ui.selectable_value(&mut self.state.view_mode, ViewMode::Maps, maps_title);
                ui.selectable_value(&mut self.state.view_mode, ViewMode::Database, db_title);
                ui.selectable_value(&mut self.state.view_mode, ViewMode::Saves, saves_title);

                if ui.button(format!("▶ {}", rust_i18n::t!("menu.playtest"))).clicked() {
                    self.launch_playtest();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.ui_preferences_popup(ui);
                    self.ui_rtp_popup(ui);
                    self.ui_theme_popup(ui);
                    self.ui_language_popup(ui);
                });
            });

            if let Some(msg) = &self.open_blocked_message {
                let danger_col = crate::theme::colors::danger(ui.visuals().dark_mode);
                ui.colored_label(danger_col, msg);
            }
        });
    }

    /// Map editing toolbar - layer/tool selectors, view toggles, undo/redo,
    /// and the Save Map action. Only shown in Maps mode.
    fn ui_map_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("map_toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

                ui.label(format!("{}:", rust_i18n::t!("status.layer")));
                ui.selectable_value(&mut self.map_view.layer_mode, MapLayerMode::Lower, rust_i18n::t!("map.layer_lower"));
                ui.selectable_value(&mut self.map_view.layer_mode, MapLayerMode::Upper, rust_i18n::t!("map.layer_upper"));
                ui.selectable_value(&mut self.map_view.layer_mode, MapLayerMode::Events, rust_i18n::t!("map.layer_events"));

                ui.separator();

                if self.map_view.layer_mode != MapLayerMode::Events {
                    ui.label(format!("{}:", rust_i18n::t!("status.tool")));
                    ui.selectable_value(&mut self.map_view.draw_tool, MapDrawTool::Pen, format!("✏ {}", rust_i18n::t!("map.tool_pen")));
                    ui.selectable_value(&mut self.map_view.draw_tool, MapDrawTool::Rectangle, format!("▭ {}", rust_i18n::t!("map.tool_rect")));
                    ui.selectable_value(&mut self.map_view.draw_tool, MapDrawTool::Ellipse, format!("⭕ {}", rust_i18n::t!("map.tool_circle")));
                    ui.selectable_value(&mut self.map_view.draw_tool, MapDrawTool::Fill, format!("🪣 {}", rust_i18n::t!("map.tool_fill")));
                    ui.selectable_value(&mut self.map_view.draw_tool, MapDrawTool::Eyedropper, format!("💉 {}", rust_i18n::t!("map.tool_picker")));
                    ui.separator();
                }

                ui.checkbox(&mut self.map_view.show_grid, "▦ Grid");
                ui.checkbox(&mut self.map_view.show_passability, rust_i18n::t!("map.overlay_passability"));
                ui.checkbox(&mut self.map_view.dim_inactive_layers, rust_i18n::t!("map.dim_layers"));

                ui.menu_button("👁", |ui| {
                    ui.label(rust_i18n::t!("map.layer_visibility"));
                    ui.separator();
                    if ui.checkbox(&mut self.map_view.show_lower_layer, "👁 Ground").changed() {
                        if let Some(cs) = &self.cached_chipset {
                            self.map_view.refresh_texture(ui.ctx(), cs);
                        }
                    }
                    if ui.checkbox(&mut self.map_view.show_upper_layer, "👁 Upper").changed() {
                        if let Some(cs) = &self.cached_chipset {
                            self.map_view.refresh_texture(ui.ctx(), cs);
                        }
                    }
                    ui.checkbox(&mut self.map_view.show_events, "👁 Events");
                })
                .response
                .on_hover_text(rust_i18n::t!("map.layer_visibility"));

                ui.separator();

                let can_undo = self.map_view.undo_stack.can_undo();
                let can_redo = self.map_view.undo_stack.can_redo();
                if ui.add_enabled(can_undo, egui::Button::new(format!("⟲ {}", rust_i18n::t!("menu.undo")))).clicked() {
                    self.map_view.undo(ui.ctx(), self.cached_chipset.as_ref());
                }
                if ui.add_enabled(can_redo, egui::Button::new(format!("⟳ {}", rust_i18n::t!("menu.redo")))).clicked() {
                    self.map_view.redo(ui.ctx(), self.cached_chipset.as_ref());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_dark = ui.visuals().dark_mode;
                    let is_dirty = self.map_view.map_dirty || self.map_view.events_dirty;

                    ui.add_enabled_ui(is_dirty, |ui| {
                        if ui.button(rust_i18n::t!("map.save_map")).clicked() {
                            let map_id = self.state.selected_map.and_then(|i| self.state.maps.get(i)).map(|m| m.0);
                            self.map_view.save_current_map(self.state.project_path.as_deref(), map_id);
                        }
                    });
                    if is_dirty {
                        ui.colored_label(crate::theme::colors::warning(is_dark), "● Unsaved Changes");
                    }
                    if let Some(msg) = &self.map_view.save_message {
                        match msg {
                            Ok(txt) => {
                                ui.colored_label(crate::theme::colors::success(is_dark), txt);
                            }
                            Err(txt) => {
                                ui.colored_label(crate::theme::colors::danger(is_dark), txt);
                            }
                        }
                    }
                });
            });
        });
    }

    /// Status bar: cursor/map/layer/tool/tile/event readouts (Maps mode) or
    /// category/entry-count summary (Database mode), plus zoom controls.
    fn ui_status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let is_dark = ui.visuals().dark_mode;
                let muted = crate::theme::colors::muted(is_dark);

                match self.state.view_mode {
                    ViewMode::Maps => {
                        let cursor_txt = match self.map_view.hover_tile {
                            Some((x, y)) => format!("{x}, {y}"),
                            None => "—".to_string(),
                        };
                        ui.colored_label(muted, rust_i18n::t!("status.cursor"));
                        ui.label(cursor_txt);
                        ui.separator();

                        if let Some(dims) = &self.map_view.map_dims {
                            ui.colored_label(muted, rust_i18n::t!("status.map"));
                            ui.label(format!("{} × {}", dims.width, dims.height));
                            ui.separator();
                        }

                        ui.colored_label(muted, rust_i18n::t!("status.layer"));
                        let layer_txt = match self.map_view.layer_mode {
                            MapLayerMode::Lower => rust_i18n::t!("map.layer_lower"),
                            MapLayerMode::Upper => rust_i18n::t!("map.layer_upper"),
                            MapLayerMode::Events => rust_i18n::t!("map.layer_events"),
                        };
                        ui.label(layer_txt);

                        if self.map_view.layer_mode != MapLayerMode::Events {
                            ui.separator();
                            ui.colored_label(muted, rust_i18n::t!("status.tool"));
                            let tool_txt = match self.map_view.draw_tool {
                                MapDrawTool::Pen => rust_i18n::t!("map.tool_pen"),
                                MapDrawTool::Rectangle => rust_i18n::t!("map.tool_rect"),
                                MapDrawTool::Ellipse => rust_i18n::t!("map.tool_circle"),
                                MapDrawTool::Fill => rust_i18n::t!("map.tool_fill"),
                                MapDrawTool::Eyedropper => rust_i18n::t!("map.tool_picker"),
                            };
                            ui.label(tool_txt);
                            ui.separator();
                            ui.colored_label(muted, rust_i18n::t!("status.tile"));
                            ui.label(format!("#{}", self.map_view.palette.selected_tile_id));
                        }

                        ui.separator();
                        ui.colored_label(muted, rust_i18n::t!("status.events"));
                        ui.label(self.map_view.events.len().to_string());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{}%", (self.map_view.zoom * 100.0) as i32));
                            if ui.small_button("+").clicked() {
                                self.map_view.zoom = (self.map_view.zoom + 0.25).min(4.0);
                            }
                            ui.add(egui::Slider::new(&mut self.map_view.zoom, 0.5..=4.0).step_by(0.1).show_value(false));
                            if ui.small_button("−").clicked() {
                                self.map_view.zoom = (self.map_view.zoom - 0.25).max(0.5);
                            }
                            ui.colored_label(muted, rust_i18n::t!("status.zoom"));
                        });
                    }
                    ViewMode::Database => {
                        let (count, dirty) = self.state.current_db_category_status();
                        ui.colored_label(muted, rust_i18n::t!("db.categories"));
                        ui.label(format!("{} — {} entries", self.state.db_category.label(), count));
                        if dirty {
                            ui.separator();
                            ui.colored_label(crate::theme::colors::warning(is_dark), "● Unsaved Changes");
                        }
                    }
                    ViewMode::Saves => {
                        ui.colored_label(muted, rust_i18n::t!("views.saves"));
                        ui.label(format!("{} slot(s)", self.state.saves.len()));
                    }
                }
            });
        });
    }

    /// Left dock. In Maps mode this is a resizable two-pane split (map tree
    /// on top, tileset palette below); Database and Saves modes fill it
    /// with their existing single-pane content.
    fn ui_left_dock(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("left_dock")
            .resizable(true)
            .default_size(260.0)
            .min_size(180.0)
            .max_size(450.0)
            .show(ui, |ui| {
                let is_dark = ui.visuals().dark_mode;
                if let Some(err) = &self.state.project_error {
                    let danger_col = crate::theme::colors::danger(is_dark);
                    ui.colored_label(danger_col, err);
                }

                match self.state.view_mode {
                    ViewMode::Database => {
                        ui.heading(format!("📂 {}", rust_i18n::t!("db.categories")));
                        ui.add_space(4.0);

                        let char_col = crate::theme::colors::category_characters(is_dark);
                        ui.colored_label(char_col, "👤 Characters");
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Actors, format!("  Actors ({})", self.state.actors.len()));
                        if self.state.is_2003 {
                            ui.selectable_value(&mut self.state.db_category, DbCategory::Classes, format!("  Classes ({})", self.state.classes.len()));
                        } else if self.state.db_category == DbCategory::Classes {
                            self.state.db_category = DbCategory::Actors;
                        }

                        ui.add_space(4.0);
                        let combat_col = crate::theme::colors::category_combat(is_dark);
                        ui.colored_label(combat_col, "⚔ Combat");
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Enemies, format!("  Enemies ({})", self.state.enemies.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Troops, format!("  Troops ({})", self.state.troops.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Skills, format!("  Skills ({})", self.state.skills.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::States, format!("  States ({})", self.state.states.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Attributes, format!("  Attributes ({})", self.state.attributes.len()));

                        ui.add_space(4.0);
                        let world_col = crate::theme::colors::category_world(is_dark);
                        ui.colored_label(world_col, "🗺 World & Items");
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Items, format!("  Items ({})", self.state.items.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Chipsets, format!("  ChipSets ({})", self.state.chipsets.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Terrains, format!("  Terrains ({})", self.state.terrains.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Animations, format!("  Animations ({})", self.state.animations.len()));

                        ui.add_space(4.0);
                        let logic_col = crate::theme::colors::category_logic(is_dark);
                        ui.colored_label(logic_col, "⚙ Logic & System");
                        ui.selectable_value(&mut self.state.db_category, DbCategory::CommonEvents, format!("  Common Events ({})", self.state.common_events.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Switches, format!("  Switches ({})", self.state.switches.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Variables, format!("  Variables ({})", self.state.variables.len()));
                        ui.selectable_value(&mut self.state.db_category, DbCategory::System, "  System Settings".to_string());
                        ui.selectable_value(&mut self.state.db_category, DbCategory::Terms, "  Terms / Vocabulary".to_string());

                        if self.state.maniac.detected {
                            ui.selectable_value(&mut self.state.db_category, DbCategory::ManiacStringVariables, format!("  🔧 Maniac String Variables ({})", self.state.maniac_string_variables.len()));
                        } else if self.state.db_category == DbCategory::ManiacStringVariables {
                            self.state.db_category = DbCategory::Actors;
                        }
                    }
                    ViewMode::Maps => {
                        egui::Panel::top("map_tree_pane")
                            .resizable(true)
                            .default_size(260.0)
                            .min_size(100.0)
                            .show(ui, |ui| {
                                if let Some(act) = self.map_tree_widget.show(ui, &self.state.map_tree, self.state.selected_map) {
                                    match act {
                                        MapTreeAction::Select(idx) => {
                                            if self.map_view.map_dirty || self.map_view.events_dirty {
                                                self.map_view.save_message = Some(Err("Save or discard map changes before switching maps.".to_string()));
                                            } else {
                                                self.select_map(idx, ui.ctx());
                                            }
                                        }
                                        MapTreeAction::OpenProperties(mid) => {
                                            if let Some(proj) = &self.state.project_path {
                                                self.map_props_dialog.open(proj, mid);
                                            }
                                        }
                                        MapTreeAction::NewMap { parent_id } => {
                                            if let Some(proj) = &self.state.project_path {
                                                if let Ok(new_id) = lcf_bridge::create_new_map(proj, parent_id, "New Map", 20, 15, 1) {
                                                    self.load_project(proj.clone(), ui.ctx());
                                                    if let Some(idx) = self.state.maps.iter().position(|m| m.0 == new_id) {
                                                        self.select_map(idx, ui.ctx());
                                                    }
                                                }
                                            }
                                        }
                                        MapTreeAction::Duplicate(mid) => {
                                            if let Some(proj) = &self.state.project_path {
                                                if let Ok(new_id) = lcf_bridge::duplicate_map(proj, mid) {
                                                    self.load_project(proj.clone(), ui.ctx());
                                                    if let Some(idx) = self.state.maps.iter().position(|m| m.0 == new_id) {
                                                        self.select_map(idx, ui.ctx());
                                                    }
                                                }
                                            }
                                        }
                                        MapTreeAction::Delete(mid) => {
                                            if let Some(proj) = &self.state.project_path {
                                                if lcf_bridge::delete_map(proj, mid).is_ok() {
                                                    self.load_project(proj.clone(), ui.ctx());
                                                }
                                            }
                                        }
                                    }
                                }
                            });

                        if !self.state.chipsets.is_empty() {
                            let active_id = self.map_view.active_chipset_id.unwrap_or(1);
                            let active_name = self.state.chipsets.iter()
                                .find(|c| c.id == active_id)
                                .map(|c| format!("{:04}: {}", c.id, c.name))
                                .unwrap_or_else(|| format!("{:04}: Chipset", active_id));

                            ui.horizontal(|ui| {
                                ui.label("🗺 Chipset:");
                                egui::ComboBox::from_id_salt("map_view_chipset_selector")
                                    .selected_text(active_name)
                                    .show_ui(ui, |ui| {
                                        for cs in &self.state.chipsets {
                                            let label = format!("{:04}: {}", cs.id, cs.name);
                                            if ui.selectable_label(active_id == cs.id, label).clicked() {
                                                self.map_view.active_chipset_id = Some(cs.id);
                                                if let Some(proj) = &self.state.project_path {
                                                    if let Some(_tex) = self.asset_cache.get_or_load(ui.ctx(), proj, "ChipSet", &cs.chipset_name) {
                                                        let path_opt = Path::new(proj).join("ChipSet").join(format!("{}.png", cs.chipset_name));
                                                        if let Ok(bytes) = std::fs::read(&path_opt) {
                                                            if let Ok(rgba) = tilemap::decode_chipset(&bytes) {
                                                                self.map_view.palette.reload_chipset(ui.ctx(), &rgba);
                                                                self.map_view.refresh_texture(ui.ctx(), &rgba);
                                                                self.cached_chipset = Some(rgba);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                            });
                        }

                        let is_upper = self.map_view.layer_mode == views::map_view::MapLayerMode::Upper;
                        self.map_view.palette.show(ui, is_upper);
                    }
                    ViewMode::Saves => {
                        ui.heading("Save Files");
                        ui.label("Manage and edit saved games (.lsd).");
                    }
                }
            });
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // App close guard - intercepted unless force_close was set by the
        // confirmation modal's own re-issued close request (see field doc).
        if ui.ctx().input(|i| i.viewport().close_requested())
            && !self.force_close
            && (self.state.has_unsaved_changes() || self.map_view.map_dirty || self.map_view.events_dirty)
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_close_confirm = true;
        }

        if self.show_close_confirm {
            egui::Modal::new(egui::Id::new("close_confirm_modal")).show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);
                ui.heading(rust_i18n::t!("dialog.unsaved_title"));
                ui.label(rust_i18n::t!("dialog.close_unsaved_prompt"));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(format!("💾 {}", rust_i18n::t!("dialog.save_all_and_close"))).clicked() {
                        let mut errors = self.state.save_all_dirty();
                        if self.map_view.map_dirty || self.map_view.events_dirty {
                            let map_id = self.state.selected_map.and_then(|i| self.state.maps.get(i)).map(|m| m.0);
                            self.map_view.save_current_map(self.state.project_path.as_deref(), map_id);
                            if self.map_view.map_dirty || self.map_view.events_dirty {
                                errors.push("Map failed to save.".to_string());
                            }
                        }
                        if errors.is_empty() {
                            self.show_close_confirm = false;
                            self.force_close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        } else {
                            self.open_blocked_message = Some(format!(
                                "{}: {}",
                                rust_i18n::t!("dialog.save_all_failed"),
                                errors.join("; ")
                            ));
                        }
                    }
                    if ui.button(format!("🗑 {}", rust_i18n::t!("dialog.discard_and_close"))).clicked() {
                        self.show_close_confirm = false;
                        self.force_close = true;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button(rust_i18n::t!("dialog.cancel")).clicked() {
                        self.show_close_confirm = false;
                    }
                });
            });
        }

        // Global Shortcuts
        if ui.input(|i| i.key_pressed(egui::Key::F5)) {
            self.map_view.layer_mode = MapLayerMode::Lower;
        }
        if ui.input(|i| i.key_pressed(egui::Key::F6)) {
            self.map_view.layer_mode = MapLayerMode::Upper;
        }
        if ui.input(|i| i.key_pressed(egui::Key::F7)) {
            self.map_view.layer_mode = MapLayerMode::Events;
        }
        if ui.input(|i| i.key_pressed(egui::Key::F9)) {
            self.launch_playtest();
        }
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.search_dialog.open();
        }
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save_current();
        }
        if self.state.view_mode == ViewMode::Maps {
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
                self.map_view.undo(ui.ctx(), self.cached_chipset.as_ref());
            }
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
                self.map_view.redo(ui.ctx(), self.cached_chipset.as_ref());
            }
        }

        // Global Modals
        if let Some(created_proj) = self.new_project_dialog.show(ui.ctx()) {
            self.load_project(created_proj, ui.ctx());
        }

        self.res_mgr_dialog.show(
            ui.ctx(),
            self.state.project_path.as_deref(),
            &mut self.asset_cache,
            self.audio.as_ref(),
            &mut self.soundfont_dialog.is_open,
        );

        if let Some(saved) = self.map_props_dialog.show(ui.ctx(), self.state.project_path.as_deref(), self.audio.as_ref()) {
            if saved {
                if let Some(sel) = self.state.selected_map {
                    self.select_map(sel, ui.ctx());
                }
            }
        }

        if let Some(target) = self.search_dialog.show(ui.ctx(), &self.state) {
            match target {
                SearchJumpTarget::Map(idx) => {
                    self.state.view_mode = ViewMode::Maps;
                    self.select_map(idx, ui.ctx());
                }
                SearchJumpTarget::Event { map_index, .. } => {
                    self.state.view_mode = ViewMode::Maps;
                    self.select_map(map_index, ui.ctx());
                }
                SearchJumpTarget::DatabaseActor(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Actors;
                    self.database_view.selected_actor = idx;
                }
                SearchJumpTarget::DatabaseItem(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Items;
                    self.database_view.selected_item = idx;
                }
                SearchJumpTarget::DatabaseSkill(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Skills;
                    self.database_view.selected_skill = idx;
                }
                SearchJumpTarget::DatabaseEnemy(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Enemies;
                    self.database_view.selected_enemy = idx;
                }
                SearchJumpTarget::DatabaseTroop(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Troops;
                    self.database_view.selected_troop = idx;
                }
                SearchJumpTarget::DatabaseTerrain(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Terrains;
                    self.database_view.terrains_view.selected_idx = idx;
                }
                SearchJumpTarget::DatabaseChipset(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Chipsets;
                    self.database_view.chipsets_view.selected_idx = idx;
                }
                SearchJumpTarget::DatabaseCommonEvent(idx) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::CommonEvents;
                    self.database_view.selected_common_event = idx;
                }
                SearchJumpTarget::DatabaseSwitch(_) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Switches;
                }
                SearchJumpTarget::DatabaseVariable(_) => {
                    self.state.view_mode = ViewMode::Database;
                    self.state.db_category = DbCategory::Variables;
                }
            }
        }

        let active_map_id = self.state.selected_map.and_then(|i| self.state.maps.get(i)).map(|m| m.0);
        self.xml_dialog.show(ui.ctx(), self.state.project_path.as_deref(), active_map_id);
        if let Some(kind) = self.xml_dialog.take_import() {
            self.asset_cache.clear();
            match kind {
                XmlImportKind::Database | XmlImportKind::Tree => {
                    if let Some(proj) = self.state.project_path.clone() {
                        let prev_selected = self.state.selected_map;
                        self.load_project(proj, ui.ctx());
                        // load_project always selects map 0; restore the
                        // previous selection if it's still valid, matching
                        // how create_new_map/duplicate_map/delete_map do it.
                        if let Some(idx) = prev_selected {
                            if idx < self.state.maps.len() && idx != 0 {
                                self.select_map(idx, ui.ctx());
                            }
                        }
                    }
                }
                XmlImportKind::Map(map_id) => {
                    if let Some(idx) = self.state.maps.iter().position(|m| m.0 == map_id) {
                        self.select_map(idx, ui.ctx());
                    }
                }
                XmlImportKind::Save => {
                    if let Some(proj) = self.state.project_path.clone() {
                        if let Some(slot) = self.state.saves.iter_mut().find(|s| s.info.file_name == "Save01.lsd") {
                            slot.info = lcf_bridge::reload_save_slot(&proj, "Save01.lsd");
                            slot.dirty = false;
                            slot.save_message = None;
                        }
                    }
                }
            }
        }
        self.sound_test_dialog.show(
            ui.ctx(),
            self.state.project_path.as_deref(),
            self.audio.as_ref(),
            &mut self.soundfont_dialog.is_open,
        );
        if let Some(audio_player) = &self.audio {
            self.soundfont_dialog.show(ui.ctx(), &mut self.state, audio_player.soundfont_manager());
        }
        self.analyzer_dialog.show(ui.ctx(), &self.state);

        self.ui_context_strip(ui);
        self.ui_menubar(ui);
        if self.state.view_mode == ViewMode::Maps {
            self.ui_map_toolbar(ui);
        }
        self.ui_status_bar(ui);
        self.ui_left_dock(ui);

        // Central View Area
        egui::CentralPanel::default().show(ui, |ui| {
            if self.state.project_path.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("🎮 EasyRPG REditor");
                    ui.label("RPG Maker 2000 & 2003 Game Development Suite");
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let total_w = 360.0;
                        let offset = (ui.available_width() - total_w).max(0.0) / 2.0;
                        ui.add_space(offset);

                        ui.vertical(|ui| {
                            if ui.add_sized([total_w, 40.0], egui::Button::new("✨ Create New Game (RPG 2000 / 2003)")).clicked() {
                                self.new_project_dialog.open();
                            }
                            ui.add_space(8.0);
                            if ui.add_sized([total_w, 36.0], egui::Button::new("📂 Open Existing Game Project...")).clicked() {
                                if let Some(path) = FileDialog::new().pick_folder() {
                                    let path_str = path.display().to_string();
                                    self.load_project(path_str, ui.ctx());
                                }
                            }

                            if !self.state.config.recent_projects.is_empty() {
                                ui.add_space(16.0);
                                ui.separator();
                                ui.label("Recent Projects:");
                                for p in &self.state.config.recent_projects.clone() {
                                    if ui.button(p).clicked() {
                                        self.load_project(p.clone(), ui.ctx());
                                    }
                                }
                            }
                        });
                    });
                });
                return;
            }

            match self.state.view_mode {
                ViewMode::Maps => {
                    let map_id = self.state.selected_map.and_then(|i| self.state.maps.get(i)).map(|m| m.0);
                    self.map_view.show(
                        ui,
                        self.state.project_path.as_deref(),
                        map_id,
                        self.cached_chipset.as_ref(),
                        &self.passability,
                        &mut self.asset_cache,
                    );
                }
                ViewMode::Database => {
                    self.database_view.show(ui, &mut self.state, &mut self.asset_cache, self.audio.as_ref());
                }
                ViewMode::Saves => {
                    self.save_view.show(ui, self.state.project_path.as_deref(), &mut self.state.saves);
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("EasyRPG REditor")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "EasyRPG REditor",
        native_options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}
