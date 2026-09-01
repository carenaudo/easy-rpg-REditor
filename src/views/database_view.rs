use eframe::egui;
use crate::app_state::EditorAppState;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::dialogs::event_command_dialog::EventCommandDialogState;
use crate::views::database::*;
use crate::widgets::asset_viewer::AssetPreviewCache;

pub struct DatabaseViewState {
    pub selected_actor: usize,
    pub selected_class: usize,
    pub selected_item: usize,
    pub selected_skill: usize,
    pub selected_attribute: usize,
    pub selected_enemy: usize,
    pub selected_troop: usize,
    pub selected_common_event: usize,
    pub selected_common_event_cmd: Option<usize>,
    pub chipsets_view: chipsets::ChipsetsView,
    pub states_view: states::StatesView,
    pub terrains_view: terrains::TerrainsView,
    pub animations_view: animations::AnimationsView,
    pub actor_view_state: actors::ActorViewState,
    pub class_view_state: classes::ClassViewState,
    pub troop_view_state: troops::TroopViewState,
    pub switch_var_view_state: switches_vars::SwitchVarViewState,
    pub asset_picker: AssetPickerState,
    pub cmd_dialog: EventCommandDialogState,
    pub item_filter: String,
    pub resize_dialog_open: bool,
    pub resize_target_count: usize,
}

impl Default for DatabaseViewState {
    fn default() -> Self {
        Self {
            selected_actor: 0,
            selected_class: 0,
            selected_item: 0,
            selected_skill: 0,
            selected_attribute: 0,
            selected_enemy: 0,
            selected_troop: 0,
            selected_common_event: 0,
            selected_common_event_cmd: None,
            chipsets_view: chipsets::ChipsetsView::default(),
            states_view: states::StatesView::default(),
            terrains_view: terrains::TerrainsView::default(),
            animations_view: animations::AnimationsView::default(),
            actor_view_state: actors::ActorViewState::default(),
            class_view_state: classes::ClassViewState::default(),
            troop_view_state: troops::TroopViewState::default(),
            switch_var_view_state: switches_vars::SwitchVarViewState::default(),
            asset_picker: AssetPickerState::default(),
            cmd_dialog: EventCommandDialogState::default(),
            item_filter: String::new(),
            resize_dialog_open: false,
            resize_target_count: 20,
        }
    }
}

impl DatabaseViewState {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        app: &mut EditorAppState,
        asset_cache: &mut AssetPreviewCache,
        audio: Option<&crate::audio::AudioPlayer>,
    ) {
        let proj = app.project_path.clone();

        // Handle Asset Picker Modal Result
        if let Some((graphic_file, sub_idx)) = self.asset_picker.show(ui.ctx(), proj.as_deref().unwrap_or(""), asset_cache) {
            match self.asset_picker.category.as_str() {
                "CharSet" => {
                    if let Some(actor) = app.actors.get_mut(self.selected_actor) {
                        actor.character_name = graphic_file;
                        actor.character_index = sub_idx;
                        app.actors_dirty = true;
                    }
                }
                "FaceSet" => {
                    if let Some(actor) = app.actors.get_mut(self.selected_actor) {
                        actor.face_name = graphic_file;
                        actor.face_index = sub_idx;
                        app.actors_dirty = true;
                    }
                }
                "Monster" => {
                    if let Some(enemy) = app.enemies.get_mut(self.selected_enemy) {
                        enemy.battler_name = graphic_file;
                        app.enemies_dirty = true;
                    }
                }
                _ => {}
            }
        }

        // Handle Command Dialog Modal Result for Common Events & Troops
        if let Some((idx_opt, cmd)) = self.cmd_dialog.show(ui.ctx()) {
            if app.db_category == crate::app_state::DbCategory::CommonEvents {
                if let Some(ce) = app.common_events.get_mut(self.selected_common_event) {
                    if let Some(idx) = idx_opt {
                        if idx < ce.commands.len() {
                            ce.commands[idx] = cmd;
                        }
                    } else {
                        let insert_pos = self.selected_common_event_cmd.map(|i| i + 1).unwrap_or(ce.commands.len());
                        crate::lcf_bridge::insert_event_command_with_scaffolding(&mut ce.commands, insert_pos, cmd);
                        self.selected_common_event_cmd = Some(insert_pos);
                    }
                    app.common_events_dirty = true;
                }
            } else if app.db_category == crate::app_state::DbCategory::Troops {
                if let Some(troop) = app.troops.get_mut(self.selected_troop) {
                    if let Some(page) = troop.pages.get_mut(self.troop_view_state.active_page_idx) {
                        if let Some(idx) = idx_opt {
                            if idx < page.commands.len() {
                                page.commands[idx] = cmd;
                            }
                        } else {
                            let insert_pos = self.troop_view_state.selected_cmd_idx.map(|i| i + 1).unwrap_or(page.commands.len());
                            crate::lcf_bridge::insert_event_command_with_scaffolding(&mut page.commands, insert_pos, cmd);
                            self.troop_view_state.selected_cmd_idx = Some(insert_pos);
                        }
                        app.troops_dirty = true;
                    }
                }
            }
        }

        // Category Save Toolbar
        let (is_dirty, save_msg) = match app.db_category {
            crate::app_state::DbCategory::Actors => (app.actors_dirty, app.actors_save_message.clone()),
            crate::app_state::DbCategory::Classes => (app.classes_dirty, app.classes_save_message.clone()),
            crate::app_state::DbCategory::Items => (app.items_dirty, app.items_save_message.clone()),
            crate::app_state::DbCategory::Skills => (app.skills_dirty, app.skills_save_message.clone()),
            crate::app_state::DbCategory::Attributes => (app.attributes_dirty, app.attributes_save_message.clone()),
            crate::app_state::DbCategory::Enemies => (app.enemies_dirty, app.enemies_save_message.clone()),
            crate::app_state::DbCategory::Troops => (app.troops_dirty, app.troops_save_message.clone()),
            crate::app_state::DbCategory::CommonEvents => (app.common_events_dirty, app.common_events_save_message.clone()),
            crate::app_state::DbCategory::Switches => (app.switches_dirty, app.switches_save_message.clone()),
            crate::app_state::DbCategory::Variables => (app.variables_dirty, app.variables_save_message.clone()),
            crate::app_state::DbCategory::Chipsets => (app.chipsets_dirty, app.chipsets_save_message.clone()),
            crate::app_state::DbCategory::States => (app.states_dirty, app.states_save_message.clone()),
            crate::app_state::DbCategory::Terrains => (app.terrains_dirty, app.terrains_save_message.clone()),
            crate::app_state::DbCategory::Animations => (app.animations_dirty, app.animations_save_message.clone()),
            crate::app_state::DbCategory::Terms => (app.terms_dirty, app.terms_save_message.clone()),
            crate::app_state::DbCategory::System => (app.system_dirty, app.system_save_message.clone()),
            crate::app_state::DbCategory::ManiacStringVariables => (app.maniac_string_variables_dirty, app.maniac_string_variables_save_message.clone()),
        };

        ui.horizontal(|ui| {
            let engine_col = if app.is_2003 { egui::Color32::from_rgb(80, 180, 255) } else { egui::Color32::from_rgb(255, 180, 80) };
            ui.colored_label(engine_col, if app.is_2003 { "🎮 RPG Maker 2003" } else { "🎮 RPG Maker 2000" });

            if app.maniac.detected {
                let mut tooltip = app.maniac.evidence.join("\n");
                tooltip.push_str("\n\nDetected via project data; run Project Health for a full event-command scan.");
                ui.colored_label(egui::Color32::from_rgb(200, 140, 255), "🔧 Maniac Patch")
                    .on_hover_text(tooltip);
            }
            ui.separator();

            ui.add_enabled_ui(is_dirty, |ui| {
                if ui.button(format!("💾 {}", rust_i18n::t!("db.save_changes"))).clicked() {
                    app.save_current_db_category();
                }
                if ui.button(format!("↺ {}", rust_i18n::t!("db.discard_changes"))).clicked() {
                    app.discard_current_db_category();
                }
            });

            if is_dirty {
                ui.colored_label(egui::Color32::from_rgb(255, 190, 40), "● Unsaved Changes (Ctrl+S)");
            }
            if let Some(msg) = &save_msg {
                match msg {
                    Ok(txt) => { ui.colored_label(egui::Color32::from_rgb(80, 220, 80), txt); }
                    Err(txt) => { ui.colored_label(egui::Color32::RED, txt); }
                }
            }
        });

        ui.separator();

        // Specific Single-Panel Views
        match app.db_category {
            crate::app_state::DbCategory::States => {
                self.states_view.show(ui, &mut app.states, &mut app.states_dirty);
                return;
            }
            crate::app_state::DbCategory::Terrains => {
                self.terrains_view.show(ui, &mut app.terrains, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.terrains_dirty, audio);
                return;
            }
            crate::app_state::DbCategory::Animations => {
                self.animations_view.show(ui, &mut app.animations, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.animations_dirty);
                return;
            }
            crate::app_state::DbCategory::System => {
                if let Some(sys) = &mut app.system {
                    system::show_system_form(ui, sys, proj.as_deref(), &mut self.asset_picker, asset_cache, &app.actors, &mut app.system_dirty, audio);
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(No system data loaded)");
                }
                return;
            }
            crate::app_state::DbCategory::Terms => {
                if let Some(terms) = &mut app.terms {
                    terms::show_terms_form(ui, terms, &mut app.terms_dirty);
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(No terms data loaded)");
                }
                return;
            }
            crate::app_state::DbCategory::Switches => {
                switches_vars::show_switches_table(ui, &mut app.switches, &mut self.switch_var_view_state, &mut app.switches_dirty);
                return;
            }
            crate::app_state::DbCategory::Variables => {
                switches_vars::show_variables_table(ui, &mut app.variables, &mut self.switch_var_view_state, &mut app.variables_dirty);
                return;
            }
            crate::app_state::DbCategory::ManiacStringVariables => {
                switches_vars::show_maniac_string_variables_table(ui, &mut app.maniac_string_variables, &mut self.switch_var_view_state, &mut app.maniac_string_variables_dirty);
                return;
            }
            _ => {}
        }

        // Master-Detail Layout (Actors, Classes, Items, Skills, Attributes, Enemies, Troops, CommonEvents, Chipsets)
        let master_width = 240.0f32;

        ui.horizontal_top(|ui| {
            // Master List Column (Left)
            ui.allocate_ui_with_layout(
                egui::vec2(master_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.group(|ui| {
                        ui.set_width(master_width);

                    // Add / Duplicate / Delete / Resize Bar
                    ui.horizontal(|ui| {
                        if ui.small_button("➕ Add").clicked() {
                            match app.db_category {
                                crate::app_state::DbCategory::Actors => {
                                    let new_id = (app.actors.len() + 1) as i32;
                                    app.actors.push(crate::lcf_bridge::ActorInfo { id: new_id, name: format!("Hero {:04}", new_id), ..Default::default() });
                                    self.selected_actor = app.actors.len() - 1;
                                    app.actors_dirty = true;
                                }
                                crate::app_state::DbCategory::Classes => {
                                    let new_id = (app.classes.len() + 1) as i32;
                                    app.classes.push(crate::lcf_bridge::ClassInfo { id: new_id, name: format!("Class {:04}", new_id), ..Default::default() });
                                    self.selected_class = app.classes.len() - 1;
                                    app.classes_dirty = true;
                                }
                                crate::app_state::DbCategory::Items => {
                                    let new_id = (app.items.len() + 1) as i32;
                                    app.items.push(crate::lcf_bridge::ItemInfo { id: new_id, name: format!("Item {:04}", new_id), ..Default::default() });
                                    self.selected_item = app.items.len() - 1;
                                    app.items_dirty = true;
                                }
                                crate::app_state::DbCategory::Skills => {
                                    let new_id = (app.skills.len() + 1) as i32;
                                    app.skills.push(crate::lcf_bridge::SkillInfo { id: new_id, name: format!("Skill {:04}", new_id), ..Default::default() });
                                    self.selected_skill = app.skills.len() - 1;
                                    app.skills_dirty = true;
                                }
                                crate::app_state::DbCategory::Attributes => {
                                    let new_id = (app.attributes.len() + 1) as i32;
                                    app.attributes.push(crate::lcf_bridge::AttributeInfo {
                                        id: new_id,
                                        name: format!("Element {:04}", new_id),
                                        attribute_type: "Physical".to_string(),
                                        a_rate: 300,
                                        b_rate: 200,
                                        c_rate: 100,
                                        d_rate: 50,
                                        e_rate: 0,
                                    });
                                    self.selected_attribute = app.attributes.len() - 1;
                                    app.attributes_dirty = true;
                                }
                                crate::app_state::DbCategory::Enemies => {
                                    let new_id = (app.enemies.len() + 1) as i32;
                                    app.enemies.push(crate::lcf_bridge::EnemyInfo { id: new_id, name: format!("Enemy {:04}", new_id), ..Default::default() });
                                    self.selected_enemy = app.enemies.len() - 1;
                                    app.enemies_dirty = true;
                                }
                                crate::app_state::DbCategory::Troops => {
                                    let new_id = (app.troops.len() + 1) as i32;
                                    app.troops.push(crate::lcf_bridge::TroopInfo { id: new_id, name: format!("Troop {:04}", new_id), ..Default::default() });
                                    self.selected_troop = app.troops.len() - 1;
                                    app.troops_dirty = true;
                                }
                                crate::app_state::DbCategory::CommonEvents => {
                                    let new_id = (app.common_events.len() + 1) as i32;
                                    app.common_events.push(crate::lcf_bridge::CommonEventInfo { id: new_id, name: format!("Common Event {:04}", new_id), ..Default::default() });
                                    self.selected_common_event = app.common_events.len() - 1;
                                    app.common_events_dirty = true;
                                }
                                crate::app_state::DbCategory::Chipsets => {
                                    let new_id = (app.chipsets.len() + 1) as i32;
                                    app.chipsets.push(crate::lcf_bridge::ChipsetInfo {
                                        id: new_id,
                                        name: format!("ChipSet {:04}", new_id),
                                        chipset_name: "World".to_string(),
                                        terrain_data: vec![1; 162],
                                        passable_data_lower: vec![15; 162],
                                        passable_data_upper: vec![15; 144],
                                        animation_type: 0,
                                        animation_speed: 0,
                                    });
                                    self.chipsets_view.selected_idx = app.chipsets.len() - 1;
                                    app.chipsets_dirty = true;
                                }
                                _ => {}
                            }
                        }

                        if ui.small_button("📄 Dup").clicked() {
                            match app.db_category {
                                crate::app_state::DbCategory::Actors => {
                                    if let Some(src) = app.actors.get(self.selected_actor).cloned() {
                                        let new_id = (app.actors.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.actors.push(dup);
                                        self.selected_actor = app.actors.len() - 1;
                                        app.actors_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Classes => {
                                    if let Some(src) = app.classes.get(self.selected_class).cloned() {
                                        let new_id = (app.classes.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.classes.push(dup);
                                        self.selected_class = app.classes.len() - 1;
                                        app.classes_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Items => {
                                    if let Some(src) = app.items.get(self.selected_item).cloned() {
                                        let new_id = (app.items.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.items.push(dup);
                                        self.selected_item = app.items.len() - 1;
                                        app.items_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Skills => {
                                    if let Some(src) = app.skills.get(self.selected_skill).cloned() {
                                        let new_id = (app.skills.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.skills.push(dup);
                                        self.selected_skill = app.skills.len() - 1;
                                        app.skills_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Attributes => {
                                    if let Some(src) = app.attributes.get(self.selected_attribute).cloned() {
                                        let new_id = (app.attributes.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.attributes.push(dup);
                                        self.selected_attribute = app.attributes.len() - 1;
                                        app.attributes_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Enemies => {
                                    if let Some(src) = app.enemies.get(self.selected_enemy).cloned() {
                                        let new_id = (app.enemies.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.enemies.push(dup);
                                        self.selected_enemy = app.enemies.len() - 1;
                                        app.enemies_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Troops => {
                                    if let Some(src) = app.troops.get(self.selected_troop).cloned() {
                                        let new_id = (app.troops.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.troops.push(dup);
                                        self.selected_troop = app.troops.len() - 1;
                                        app.troops_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::CommonEvents => {
                                    if let Some(src) = app.common_events.get(self.selected_common_event).cloned() {
                                        let new_id = (app.common_events.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.common_events.push(dup);
                                        self.selected_common_event = app.common_events.len() - 1;
                                        app.common_events_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Chipsets => {
                                    if let Some(src) = app.chipsets.get(self.chipsets_view.selected_idx).cloned() {
                                        let new_id = (app.chipsets.len() + 1) as i32;
                                        let mut dup = src;
                                        dup.id = new_id;
                                        dup.name = format!("{} (Copy)", dup.name);
                                        app.chipsets.push(dup);
                                        self.chipsets_view.selected_idx = app.chipsets.len() - 1;
                                        app.chipsets_dirty = true;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if ui.small_button("🗑 Del").clicked() {
                            match app.db_category {
                                crate::app_state::DbCategory::Actors => {
                                    if app.actors.len() > 1 && self.selected_actor < app.actors.len() {
                                        app.actors.remove(self.selected_actor);
                                        for (i, a) in app.actors.iter_mut().enumerate() { a.id = (i + 1) as i32; }
                                        if self.selected_actor >= app.actors.len() { self.selected_actor = app.actors.len() - 1; }
                                        app.actors_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Classes => {
                                    if app.classes.len() > 1 && self.selected_class < app.classes.len() {
                                        app.classes.remove(self.selected_class);
                                        for (i, c) in app.classes.iter_mut().enumerate() { c.id = (i + 1) as i32; }
                                        if self.selected_class >= app.classes.len() { self.selected_class = app.classes.len() - 1; }
                                        app.classes_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Items => {
                                    if app.items.len() > 1 && self.selected_item < app.items.len() {
                                        app.items.remove(self.selected_item);
                                        for (i, it) in app.items.iter_mut().enumerate() { it.id = (i + 1) as i32; }
                                        if self.selected_item >= app.items.len() { self.selected_item = app.items.len() - 1; }
                                        app.items_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Skills => {
                                    if app.skills.len() > 1 && self.selected_skill < app.skills.len() {
                                        app.skills.remove(self.selected_skill);
                                        for (i, sk) in app.skills.iter_mut().enumerate() { sk.id = (i + 1) as i32; }
                                        if self.selected_skill >= app.skills.len() { self.selected_skill = app.skills.len() - 1; }
                                        app.skills_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Attributes => {
                                    if app.attributes.len() > 1 && self.selected_attribute < app.attributes.len() {
                                        app.attributes.remove(self.selected_attribute);
                                        for (i, at) in app.attributes.iter_mut().enumerate() { at.id = (i + 1) as i32; }
                                        if self.selected_attribute >= app.attributes.len() { self.selected_attribute = app.attributes.len() - 1; }
                                        app.attributes_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Enemies => {
                                    if app.enemies.len() > 1 && self.selected_enemy < app.enemies.len() {
                                        app.enemies.remove(self.selected_enemy);
                                        for (i, e) in app.enemies.iter_mut().enumerate() { e.id = (i + 1) as i32; }
                                        if self.selected_enemy >= app.enemies.len() { self.selected_enemy = app.enemies.len() - 1; }
                                        app.enemies_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Troops => {
                                    if app.troops.len() > 1 && self.selected_troop < app.troops.len() {
                                        app.troops.remove(self.selected_troop);
                                        for (i, tr) in app.troops.iter_mut().enumerate() { tr.id = (i + 1) as i32; }
                                        if self.selected_troop >= app.troops.len() { self.selected_troop = app.troops.len() - 1; }
                                        app.troops_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::CommonEvents => {
                                    if app.common_events.len() > 1 && self.selected_common_event < app.common_events.len() {
                                        app.common_events.remove(self.selected_common_event);
                                        for (i, ce) in app.common_events.iter_mut().enumerate() { ce.id = (i + 1) as i32; }
                                        if self.selected_common_event >= app.common_events.len() { self.selected_common_event = app.common_events.len() - 1; }
                                        app.common_events_dirty = true;
                                    }
                                }
                                crate::app_state::DbCategory::Chipsets => {
                                    if app.chipsets.len() > 1 && self.chipsets_view.selected_idx < app.chipsets.len() {
                                        app.chipsets.remove(self.chipsets_view.selected_idx);
                                        for (i, cs) in app.chipsets.iter_mut().enumerate() { cs.id = (i + 1) as i32; }
                                        if self.chipsets_view.selected_idx >= app.chipsets.len() { self.chipsets_view.selected_idx = app.chipsets.len() - 1; }
                                        app.chipsets_dirty = true;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if ui.small_button("📏 Max...").on_hover_text("Batch resize database array capacity").clicked() {
                            let current_len = match app.db_category {
                                crate::app_state::DbCategory::Actors => app.actors.len(),
                                crate::app_state::DbCategory::Classes => app.classes.len(),
                                crate::app_state::DbCategory::Items => app.items.len(),
                                crate::app_state::DbCategory::Skills => app.skills.len(),
                                crate::app_state::DbCategory::Attributes => app.attributes.len(),
                                crate::app_state::DbCategory::Enemies => app.enemies.len(),
                                crate::app_state::DbCategory::Troops => app.troops.len(),
                                crate::app_state::DbCategory::CommonEvents => app.common_events.len(),
                                crate::app_state::DbCategory::Chipsets => app.chipsets.len(),
                                _ => 0,
                            };
                            if current_len > 0 {
                                self.resize_target_count = current_len;
                                self.resize_dialog_open = true;
                            }
                        }
                    });

                    // Search Filter
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.add(egui::TextEdit::singleline(&mut self.item_filter).hint_text("Filter...").desired_width(160.0));
                        if !self.item_filter.is_empty() && ui.small_button("✕").clicked() {
                            self.item_filter.clear();
                        }
                    });

                    ui.separator();

                    let filter = self.item_filter.to_lowercase();

                    // Scrollable Items List
                    egui::ScrollArea::vertical()
                        .id_salt("db_master_items_scroll")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                match app.db_category {
                                    crate::app_state::DbCategory::Actors => {
                                        for (i, a) in app.actors.iter().enumerate() {
                                            if !filter.is_empty() && !a.name.to_lowercase().contains(&filter) && !a.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", a.id, a.name);
                                            if ui.selectable_label(self.selected_actor == i, label).clicked() {
                                                self.selected_actor = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Classes => {
                                        for (i, c) in app.classes.iter().enumerate() {
                                            if !filter.is_empty() && !c.name.to_lowercase().contains(&filter) && !c.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", c.id, c.name);
                                            if ui.selectable_label(self.selected_class == i, label).clicked() {
                                                self.selected_class = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Items => {
                                        for (i, item) in app.items.iter().enumerate() {
                                            if !filter.is_empty() && !item.name.to_lowercase().contains(&filter) && !item.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", item.id, item.name);
                                            if ui.selectable_label(self.selected_item == i, label).clicked() {
                                                self.selected_item = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Skills => {
                                        for (i, s) in app.skills.iter().enumerate() {
                                            if !filter.is_empty() && !s.name.to_lowercase().contains(&filter) && !s.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", s.id, s.name);
                                            if ui.selectable_label(self.selected_skill == i, label).clicked() {
                                                self.selected_skill = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Attributes => {
                                        for (i, attr) in app.attributes.iter().enumerate() {
                                            if !filter.is_empty() && !attr.name.to_lowercase().contains(&filter) && !attr.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", attr.id, attr.name);
                                            if ui.selectable_label(self.selected_attribute == i, label).clicked() {
                                                self.selected_attribute = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Enemies => {
                                        for (i, e) in app.enemies.iter().enumerate() {
                                            if !filter.is_empty() && !e.name.to_lowercase().contains(&filter) && !e.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", e.id, e.name);
                                            if ui.selectable_label(self.selected_enemy == i, label).clicked() {
                                                self.selected_enemy = i;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Troops => {
                                        for (i, t) in app.troops.iter().enumerate() {
                                            if !filter.is_empty() && !t.name.to_lowercase().contains(&filter) && !t.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", t.id, t.name);
                                            if ui.selectable_label(self.selected_troop == i, label).clicked() {
                                                self.selected_troop = i;
                                                self.troop_view_state.selected_cmd_idx = None;
                                                self.troop_view_state.active_page_idx = 0;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::CommonEvents => {
                                        for (i, ce) in app.common_events.iter().enumerate() {
                                            if !filter.is_empty() && !ce.name.to_lowercase().contains(&filter) && !ce.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", ce.id, ce.name);
                                            if ui.selectable_label(self.selected_common_event == i, label).clicked() {
                                                self.selected_common_event = i;
                                                self.selected_common_event_cmd = None;
                                            }
                                        }
                                    }
                                    crate::app_state::DbCategory::Chipsets => {
                                        for (i, cs) in app.chipsets.iter().enumerate() {
                                            if !filter.is_empty() && !cs.name.to_lowercase().contains(&filter) && !cs.id.to_string().contains(&filter) {
                                                continue;
                                            }
                                            let label = format!("{:04}: {}", cs.id, cs.name);
                                            if ui.selectable_label(self.chipsets_view.selected_idx == i, label).clicked() {
                                                self.chipsets_view.selected_idx = i;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            });
                        });
                });
            });

            // Active Form / Dashboard Column (Right)
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());

                        match app.db_category {
                            crate::app_state::DbCategory::Actors => {
                                if let Some(actor) = app.actors.get_mut(self.selected_actor) {
                                    actors::show_actor_form(
                                        ui,
                                        actor,
                                        app.is_2003,
                                        proj.as_deref(),
                                        &app.items,
                                        &app.skills,
                                        &app.classes,
                                        &app.states,
                                        &app.attributes,
                                        &mut self.asset_picker,
                                        asset_cache,
                                        &mut self.actor_view_state,
                                        &mut app.actors_dirty,
                                    );
                                }
                            }
                            crate::app_state::DbCategory::Classes => {
                                if let Some(class) = app.classes.get_mut(self.selected_class) {
                                    classes::show_class_form(
                                        ui,
                                        class,
                                        &app.skills,
                                        &app.states,
                                        &app.attributes,
                                        &mut self.class_view_state,
                                        &mut app.classes_dirty,
                                    );
                                }
                            }
                            crate::app_state::DbCategory::Items => {
                                if let Some(item) = app.items.get_mut(self.selected_item) {
                                    items::show_item_form(ui, item, &mut app.items_dirty);
                                }
                            }
                            crate::app_state::DbCategory::Skills => {
                                if let Some(skill) = app.skills.get_mut(self.selected_skill) {
                                    skills::show_skill_form(
                                        ui,
                                        skill,
                                        app.is_2003,
                                        &mut app.skills_dirty,
                                        app.project_path.as_deref(),
                                        audio,
                                    );
                                }
                            }
                    crate::app_state::DbCategory::Attributes => {
                        if let Some(attr) = app.attributes.get_mut(self.selected_attribute) {
                            attributes::show_attribute_form(ui, attr, &mut app.attributes_dirty);
                        }
                    }
                    crate::app_state::DbCategory::Enemies => {
                        if let Some(enemy) = app.enemies.get_mut(self.selected_enemy) {
                            enemies::show_enemy_form(ui, enemy, &app.states, &app.attributes, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.enemies_dirty);
                        }
                    }
                    crate::app_state::DbCategory::Troops => {
                        if let Some(troop) = app.troops.get_mut(self.selected_troop) {
                            troops::show_troop_form(
                                ui,
                                troop,
                                proj.as_deref(),
                                &app.enemies,
                                &app.terrains,
                                asset_cache,
                                &mut self.troop_view_state,
                                &mut self.cmd_dialog,
                                &mut app.troops_dirty,
                            );
                        }
                    }
                    crate::app_state::DbCategory::CommonEvents => {
                        if let Some(ce) = app.common_events.get_mut(self.selected_common_event) {
                            common_events::show_common_event_form(ui, ce, &app.switches, &mut self.cmd_dialog, &mut self.selected_common_event_cmd, &mut app.common_events_dirty);
                        }
                    }
                    crate::app_state::DbCategory::Chipsets => {
                        self.chipsets_view.show(ui, &mut app.chipsets, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.chipsets_dirty);
                    }
                    _ => {}
                }
            });
        });
    });

        // Render Batch Resize Capacity Modal
        if self.resize_dialog_open {
            let mut open = self.resize_dialog_open;
            egui::Window::new("Resize Database Capacity")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Set total number of entries (1..5000):");
                    ui.add(egui::DragValue::new(&mut self.resize_target_count).range(1..=5000));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            let target = self.resize_target_count.clamp(1, 5000);
                            match app.db_category {
                                crate::app_state::DbCategory::Actors => {
                                    while app.actors.len() < target {
                                        let id = (app.actors.len() + 1) as i32;
                                        app.actors.push(crate::lcf_bridge::ActorInfo { id, name: format!("Hero {:04}", id), ..Default::default() });
                                    }
                                    app.actors.truncate(target);
                                    if self.selected_actor >= app.actors.len() { self.selected_actor = app.actors.len() - 1; }
                                    app.actors_dirty = true;
                                }
                                crate::app_state::DbCategory::Classes => {
                                    while app.classes.len() < target {
                                        let id = (app.classes.len() + 1) as i32;
                                        app.classes.push(crate::lcf_bridge::ClassInfo { id, name: format!("Class {:04}", id), ..Default::default() });
                                    }
                                    app.classes.truncate(target);
                                    if self.selected_class >= app.classes.len() { self.selected_class = app.classes.len() - 1; }
                                    app.classes_dirty = true;
                                }
                                crate::app_state::DbCategory::Items => {
                                    while app.items.len() < target {
                                        let id = (app.items.len() + 1) as i32;
                                        app.items.push(crate::lcf_bridge::ItemInfo { id, name: format!("Item {:04}", id), ..Default::default() });
                                    }
                                    app.items.truncate(target);
                                    if self.selected_item >= app.items.len() { self.selected_item = app.items.len() - 1; }
                                    app.items_dirty = true;
                                }
                                crate::app_state::DbCategory::Skills => {
                                    while app.skills.len() < target {
                                        let id = (app.skills.len() + 1) as i32;
                                        app.skills.push(crate::lcf_bridge::SkillInfo { id, name: format!("Skill {:04}", id), ..Default::default() });
                                    }
                                    app.skills.truncate(target);
                                    if self.selected_skill >= app.skills.len() { self.selected_skill = app.skills.len() - 1; }
                                    app.skills_dirty = true;
                                }
                                crate::app_state::DbCategory::Attributes => {
                                    while app.attributes.len() < target {
                                        let id = (app.attributes.len() + 1) as i32;
                                        app.attributes.push(crate::lcf_bridge::AttributeInfo {
                                            id,
                                            name: format!("Element {:04}", id),
                                            attribute_type: "Physical".to_string(),
                                            a_rate: 300,
                                            b_rate: 200,
                                            c_rate: 100,
                                            d_rate: 50,
                                            e_rate: 0,
                                        });
                                    }
                                    app.attributes.truncate(target);
                                    if self.selected_attribute >= app.attributes.len() { self.selected_attribute = app.attributes.len() - 1; }
                                    app.attributes_dirty = true;
                                }
                                crate::app_state::DbCategory::Enemies => {
                                    while app.enemies.len() < target {
                                        let id = (app.enemies.len() + 1) as i32;
                                        app.enemies.push(crate::lcf_bridge::EnemyInfo { id, name: format!("Enemy {:04}", id), ..Default::default() });
                                    }
                                    app.enemies.truncate(target);
                                    if self.selected_enemy >= app.enemies.len() { self.selected_enemy = app.enemies.len() - 1; }
                                    app.enemies_dirty = true;
                                }
                                crate::app_state::DbCategory::Troops => {
                                    while app.troops.len() < target {
                                        let id = (app.troops.len() + 1) as i32;
                                        app.troops.push(crate::lcf_bridge::TroopInfo { id, name: format!("Troop {:04}", id), ..Default::default() });
                                    }
                                    app.troops.truncate(target);
                                    if self.selected_troop >= app.troops.len() { self.selected_troop = app.troops.len() - 1; }
                                    app.troops_dirty = true;
                                }
                                crate::app_state::DbCategory::CommonEvents => {
                                    while app.common_events.len() < target {
                                        let id = (app.common_events.len() + 1) as i32;
                                        app.common_events.push(crate::lcf_bridge::CommonEventInfo { id, name: format!("Common Event {:04}", id), ..Default::default() });
                                    }
                                    app.common_events.truncate(target);
                                    if self.selected_common_event >= app.common_events.len() { self.selected_common_event = app.common_events.len() - 1; }
                                    app.common_events_dirty = true;
                                }
                                crate::app_state::DbCategory::Chipsets => {
                                    while app.chipsets.len() < target {
                                        let id = (app.chipsets.len() + 1) as i32;
                                        app.chipsets.push(crate::lcf_bridge::ChipsetInfo {
                                            id,
                                            name: format!("ChipSet {:04}", id),
                                            chipset_name: "World".to_string(),
                                            terrain_data: vec![1; 162],
                                            passable_data_lower: vec![15; 162],
                                            passable_data_upper: vec![15; 144],
                                            animation_type: 0,
                                            animation_speed: 0,
                                        });
                                    }
                                    app.chipsets.truncate(target);
                                    if self.chipsets_view.selected_idx >= app.chipsets.len() { self.chipsets_view.selected_idx = app.chipsets.len() - 1; }
                                    app.chipsets_dirty = true;
                                }
                                _ => {}
                            }
                            self.resize_dialog_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.resize_dialog_open = false;
                        }
                    });
                });
            self.resize_dialog_open = open;
        }
}
}

