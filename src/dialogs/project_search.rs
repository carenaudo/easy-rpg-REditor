use eframe::egui;

#[derive(Clone, Debug)]
pub enum SearchJumpTarget {
    Map(usize),
    Event { map_index: usize, event_id: i32 },
    DatabaseActor(usize),
    DatabaseItem(usize),
    DatabaseSkill(usize),
    DatabaseEnemy(usize),
    DatabaseTroop(usize),
    DatabaseTerrain(usize),
    DatabaseChipset(usize),
    DatabaseCommonEvent(usize),
    DatabaseSwitch(usize),
    DatabaseVariable(usize),
}

#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub category: String,
    pub label: String,
    pub detail: String,
    pub target: SearchJumpTarget,
}

#[derive(Default)]
pub struct ProjectSearchDialog {
    pub is_open: bool,
    pub query: String,
    pub results: Vec<SearchResultItem>,
}

impl ProjectSearchDialog {
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.results.clear();
    }

    pub fn execute_search(&mut self, app: &crate::app_state::EditorAppState) {
        self.results.clear();
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            return;
        }

        // Search Maps & Map Events across the entire project
        for (i, (map_id, map_name)) in app.maps.iter().enumerate() {
            if map_name.to_lowercase().contains(&q) || map_id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Maps".to_string(),
                    label: format!("Map {:04}: {}", map_id, map_name),
                    detail: format!("ID {}", map_id),
                    target: SearchJumpTarget::Map(i),
                });
            }

            if let Some(path) = &app.project_path {
                let map_events = crate::lcf_bridge::get_map_events(path, *map_id);
                for ev in map_events {
                    let name_matches = ev.name.to_lowercase().contains(&q);
                    let mut matched_cmd_text = None;
                    for page in &ev.pages {
                        for cmd in &page.commands {
                            if !cmd.string.is_empty() && cmd.string.to_lowercase().contains(&q) {
                                matched_cmd_text = Some(cmd.string.clone());
                                break;
                            }
                        }
                        if matched_cmd_text.is_some() { break; }
                    }

                    if name_matches || matched_cmd_text.is_some() || ev.id.to_string().contains(&q) {
                        let detail = if let Some(txt) = matched_cmd_text {
                            format!("Map #{} ({}) at ({}, {}) | \"{}\"", map_id, map_name, ev.x, ev.y, txt)
                        } else {
                            format!("Map #{} ({}) at ({}, {})", map_id, map_name, ev.x, ev.y)
                        };
                        self.results.push(SearchResultItem {
                            category: "Events".to_string(),
                            label: format!("Event #{:04}: {}", ev.id, ev.name),
                            detail,
                            target: SearchJumpTarget::Event { map_index: i, event_id: ev.id },
                        });
                    }
                }
            }
        }

        // Search Database Actors
        for (i, actor) in app.actors.iter().enumerate() {
            if actor.name.to_lowercase().contains(&q) || actor.title.to_lowercase().contains(&q) || actor.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Actors".to_string(),
                    label: format!("{:04}: {}", actor.id, actor.name),
                    detail: format!("Title: {}", actor.title),
                    target: SearchJumpTarget::DatabaseActor(i),
                });
            }
        }

        // Search Items
        for (i, item) in app.items.iter().enumerate() {
            if item.name.to_lowercase().contains(&q) || item.description.to_lowercase().contains(&q) || item.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Items".to_string(),
                    label: format!("{:04}: {}", item.id, item.name),
                    detail: item.description.clone(),
                    target: SearchJumpTarget::DatabaseItem(i),
                });
            }
        }

        // Search Skills
        for (i, skill) in app.skills.iter().enumerate() {
            if skill.name.to_lowercase().contains(&q) || skill.description.to_lowercase().contains(&q) || skill.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Skills".to_string(),
                    label: format!("{:04}: {}", skill.id, skill.name),
                    detail: skill.description.clone(),
                    target: SearchJumpTarget::DatabaseSkill(i),
                });
            }
        }

        // Search Enemies
        for (i, enemy) in app.enemies.iter().enumerate() {
            if enemy.name.to_lowercase().contains(&q) || enemy.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Enemies".to_string(),
                    label: format!("{:04}: {}", enemy.id, enemy.name),
                    detail: format!("HP: {}, ATK: {}", enemy.max_hp, enemy.attack),
                    target: SearchJumpTarget::DatabaseEnemy(i),
                });
            }
        }

        // Search Troops
        for (i, troop) in app.troops.iter().enumerate() {
            if troop.name.to_lowercase().contains(&q) || troop.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Troops".to_string(),
                    label: format!("{:04}: {}", troop.id, troop.name),
                    detail: format!("Members: {}", troop.members.len()),
                    target: SearchJumpTarget::DatabaseTroop(i),
                });
            }
        }

        // Search Terrains
        for (i, terrain) in app.terrains.iter().enumerate() {
            if terrain.name.to_lowercase().contains(&q) || terrain.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Terrains".to_string(),
                    label: format!("{:04}: {}", terrain.id, terrain.name),
                    detail: format!("Damage: {} HP", terrain.damage),
                    target: SearchJumpTarget::DatabaseTerrain(i),
                });
            }
        }

        // Search Chipsets
        for (i, cs) in app.chipsets.iter().enumerate() {
            if cs.name.to_lowercase().contains(&q) || cs.chipset_name.to_lowercase().contains(&q) || cs.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Chipsets".to_string(),
                    label: format!("{:04}: {}", cs.id, cs.name),
                    detail: cs.chipset_name.clone(),
                    target: SearchJumpTarget::DatabaseChipset(i),
                });
            }
        }

        // Search Common Events (name + script dialogue)
        for (i, ce) in app.common_events.iter().enumerate() {
            let name_matches = ce.name.to_lowercase().contains(&q);
            let mut matched_cmd_text = None;
            for cmd in &ce.commands {
                if !cmd.string.is_empty() && cmd.string.to_lowercase().contains(&q) {
                    matched_cmd_text = Some(cmd.string.clone());
                    break;
                }
            }

            if name_matches || matched_cmd_text.is_some() || ce.id.to_string().contains(&q) {
                let detail = if let Some(txt) = matched_cmd_text {
                    format!("Trigger: {} | Script: \"{}\"", ce.trigger, txt)
                } else {
                    format!("Trigger: {}", ce.trigger)
                };
                self.results.push(SearchResultItem {
                    category: "Common Events".to_string(),
                    label: format!("{:04}: {}", ce.id, ce.name),
                    detail,
                    target: SearchJumpTarget::DatabaseCommonEvent(i),
                });
            }
        }

        // Search Switches
        for (i, s) in app.switches.iter().enumerate() {
            if s.name.to_lowercase().contains(&q) || s.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Switches".to_string(),
                    label: format!("Switch #{:04}: {}", s.id, s.name),
                    detail: String::new(),
                    target: SearchJumpTarget::DatabaseSwitch(i),
                });
            }
        }

        // Search Variables
        for (i, v) in app.variables.iter().enumerate() {
            if v.name.to_lowercase().contains(&q) || v.id.to_string().contains(&q) {
                self.results.push(SearchResultItem {
                    category: "Variables".to_string(),
                    label: format!("Variable #{:04}: {}", v.id, v.name),
                    detail: String::new(),
                    target: SearchJumpTarget::DatabaseVariable(i),
                });
            }
        }
    }

    /// Shows the search window. Returns Some(SearchJumpTarget) if an item was clicked.
    pub fn show(&mut self, ctx: &egui::Context, app: &crate::app_state::EditorAppState) -> Option<SearchJumpTarget> {
        if !self.is_open {
            return None;
        }

        let mut jump_target = None;
        let mut is_open = self.is_open;

        egui::Window::new("Project-wide Search")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([520.0, 400.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let text_edit = ui.text_edit_singleline(&mut self.query);
                    if text_edit.changed() {
                        self.execute_search(app);
                    }
                    if !self.query.is_empty() && ui.small_button("x").clicked() {
                        self.query.clear();
                        self.results.clear();
                    }
                });

                ui.separator();
                ui.label(format!("Found {} results:", self.results.len()));

                egui::ScrollArea::vertical()
                    .id_salt("search_results_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let is_dark = ui.visuals().dark_mode;
                        let cat_col = crate::theme::colors::info(is_dark);
                        let detail_col = crate::theme::colors::muted(is_dark);
                        for item in &self.results {
                            ui.horizontal(|ui| {
                                ui.colored_label(cat_col, format!("[{}]", item.category));
                                if ui.button(&item.label).clicked() {
                                    jump_target = Some(item.target.clone());
                                    self.is_open = false;
                                }
                                if !item.detail.is_empty() {
                                    ui.colored_label(detail_col, &item.detail);
                                }
                            });
                        }
                    });
            });

        if !is_open {
            self.is_open = false;
        }

        jump_target
    }
}
