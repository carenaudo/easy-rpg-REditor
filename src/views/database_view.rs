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
        }
    }
}

impl DatabaseViewState {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        app: &mut EditorAppState,
        asset_cache: &mut AssetPreviewCache,
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
                        ce.commands.insert(insert_pos, cmd);
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
                            page.commands.insert(insert_pos, cmd);
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
            crate::app_state::DbCategory::System => (false, None),
        };

        ui.horizontal(|ui| {
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
                self.terrains_view.show(ui, &mut app.terrains, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.terrains_dirty);
                return;
            }
            crate::app_state::DbCategory::Animations => {
                self.animations_view.show(ui, &mut app.animations, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.animations_dirty);
                return;
            }
            crate::app_state::DbCategory::System => {
                if let Some(sys) = &mut app.system {
                    system::show_system_form(ui, sys, proj.as_deref(), &mut self.asset_picker, asset_cache, &app.actors, &mut app.system_dirty);
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

                    // Add / Duplicate Bar
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
                                    skills::show_skill_form(ui, skill, &mut app.skills_dirty);
                                }
                            }
                    crate::app_state::DbCategory::Attributes => {
                        if let Some(attr) = app.attributes.get_mut(self.selected_attribute) {
                            attributes::show_attribute_form(ui, attr, &mut app.attributes_dirty);
                        }
                    }
                    crate::app_state::DbCategory::Enemies => {
                        if let Some(enemy) = app.enemies.get_mut(self.selected_enemy) {
                            enemies::show_enemy_form(ui, enemy, proj.as_deref(), &mut self.asset_picker, asset_cache, &mut app.enemies_dirty);
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
}
}

