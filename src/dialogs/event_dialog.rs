use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::dialogs::event_command_dialog::EventCommandDialogState;
use crate::lcf_bridge::{
    event_command_label, event_layer_label, event_move_type_label, event_trigger_label,
    EventConditionInfo, EventInfo, EventPageInfo,
};
use crate::widgets::asset_viewer::AssetPreviewCache;

pub struct EventDialogState {
    pub is_open: bool,
    pub event: EventInfo,
    pub selected_page: usize,
    pub selected_command: Option<usize>,
    pub copied_page: Option<EventPageInfo>,
    pub cmd_dialog: EventCommandDialogState,
    pub asset_picker: AssetPickerState,
    pub command_search: String,
}

impl Default for EventDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            event: EventInfo::default(),
            selected_page: 0,
            selected_command: None,
            copied_page: None,
            cmd_dialog: EventCommandDialogState::default(),
            asset_picker: AssetPickerState::default(),
            command_search: String::new(),
        }
    }
}

impl EventDialogState {
    pub fn open(&mut self, event: &EventInfo) {
        self.is_open = true;
        self.event = event.clone();
        if self.event.pages.is_empty() {
            self.event.pages.push(EventPageInfo {
                id: 1,
                character_name: String::new(),
                character_index: 0,
                character_direction: 2,
                character_pattern: 1,
                translucent: false,
                move_type: 0,
                move_frequency: 3,
                trigger: 0,
                layer: 1,
                overlap_forbidden: false,
                animation_type: 0,
                move_speed: 3,
                condition: EventConditionInfo::default(),
                commands: Vec::new(),
            });
        }
        self.selected_page = 0;
        self.selected_command = None;
    }

    /// Shows the multi-page event dialog. Returns Some(EventInfo) if saved.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project_path: &str,
        cache: &mut AssetPreviewCache,
    ) -> Option<EventInfo> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut is_open = self.is_open;

        // Sub-dialogs
        if let Some((idx_opt, cmd)) = self.cmd_dialog.show(ctx) {
            if let Some(page) = self.event.pages.get_mut(self.selected_page) {
                if let Some(idx) = idx_opt {
                    if idx < page.commands.len() {
                        page.commands[idx] = cmd;
                    }
                } else {
                    let insert_pos = self.selected_command.map(|i| i + 1).unwrap_or(page.commands.len());
                    page.commands.insert(insert_pos, cmd);
                    self.selected_command = Some(insert_pos);
                }
            }
        }

        if let Some((graphic_file, sub_idx)) = self.asset_picker.show(ctx, project_path, cache) {
            if let Some(page) = self.event.pages.get_mut(self.selected_page) {
                page.character_name = graphic_file;
                page.character_index = sub_idx;
            }
        }

        egui::Window::new(format!("Event Editor - ID: {:04}", self.event.id))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([720.0, 580.0])
            .show(ctx, |ui| {
                // Header: Name and Position
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.event.name);
                    ui.separator();
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut self.event.x).range(0..=500));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.event.y).range(0..=500));
                });

                ui.separator();

                // Page tabs row
                ui.horizontal(|ui| {
                    for i in 0..self.event.pages.len() {
                        let label = format!("Page {}", i + 1);
                        if ui.selectable_label(self.selected_page == i, label).clicked() {
                            self.selected_page = i;
                            self.selected_command = None;
                        }
                    }

                    if ui.button("+ New Page").clicked() {
                        let new_id = (self.event.pages.len() + 1) as i32;
                        self.event.pages.push(EventPageInfo {
                            id: new_id,
                            layer: 1,
                            move_frequency: 3,
                            move_speed: 3,
                            ..Default::default()
                        });
                        self.selected_page = self.event.pages.len() - 1;
                        self.selected_command = None;
                    }

                    if ui.button("Copy Page").clicked() {
                        if let Some(page) = self.event.pages.get(self.selected_page) {
                            self.copied_page = Some(page.clone());
                        }
                    }

                    if ui.add_enabled(self.copied_page.is_some(), egui::Button::new("Paste Page")).clicked() {
                        if let Some(copied) = &self.copied_page {
                            let mut new_p = copied.clone();
                            new_p.id = (self.event.pages.len() + 1) as i32;
                            self.event.pages.push(new_p);
                            self.selected_page = self.event.pages.len() - 1;
                            self.selected_command = None;
                        }
                    }

                    if ui.add_enabled(self.event.pages.len() > 1, egui::Button::new("Delete Page")).clicked() {
                        self.event.pages.remove(self.selected_page);
                        if self.selected_page >= self.event.pages.len() && self.selected_page > 0 {
                            self.selected_page -= 1;
                        }
                        self.selected_command = None;
                    }
                });

                ui.separator();

                if let Some(page) = self.event.pages.get_mut(self.selected_page) {
                    ui.columns(2, |cols| {
                        // Left Column: Preconditions, Graphic, Movement, Trigger
                        cols[0].group(|ui| {
                            ui.heading("Page Preconditions");
                            egui::Grid::new("page_precond_grid")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    ui.checkbox(&mut page.condition.switch1_flag, "Switch 1:");
                                    ui.add_enabled(page.condition.switch1_flag, egui::DragValue::new(&mut page.condition.switch1_id).range(1..=5000));
                                    ui.end_row();

                                    ui.checkbox(&mut page.condition.switch2_flag, "Switch 2:");
                                    ui.add_enabled(page.condition.switch2_flag, egui::DragValue::new(&mut page.condition.switch2_id).range(1..=5000));
                                    ui.end_row();

                                    ui.checkbox(&mut page.condition.var_flag, "Variable:");
                                    ui.horizontal(|ui| {
                                        ui.add_enabled(page.condition.var_flag, egui::DragValue::new(&mut page.condition.var_id).range(1..=5000));
                                        egui::ComboBox::from_id_salt("var_comp_op")
                                            .selected_text(match page.condition.var_compare_op {
                                                0 => "==",
                                                1 => ">=",
                                                2 => "<=",
                                                3 => ">",
                                                4 => "<",
                                                _ => "!=",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut page.condition.var_compare_op, 0, "== (Equal)");
                                                ui.selectable_value(&mut page.condition.var_compare_op, 1, ">= (Greater/Equal)");
                                                ui.selectable_value(&mut page.condition.var_compare_op, 2, "<= (Less/Equal)");
                                                ui.selectable_value(&mut page.condition.var_compare_op, 3, "> (Greater)");
                                                ui.selectable_value(&mut page.condition.var_compare_op, 4, "< (Less)");
                                                ui.selectable_value(&mut page.condition.var_compare_op, 5, "!= (Not Equal)");
                                            });
                                        ui.add_enabled(page.condition.var_flag, egui::DragValue::new(&mut page.condition.var_value));
                                    });
                                    ui.end_row();

                                    ui.checkbox(&mut page.condition.item_flag, "Item in Inventory:");
                                    ui.add_enabled(page.condition.item_flag, egui::DragValue::new(&mut page.condition.item_id).range(1..=5000));
                                    ui.end_row();

                                    ui.checkbox(&mut page.condition.actor_flag, "Hero in Party:");
                                    ui.add_enabled(page.condition.actor_flag, egui::DragValue::new(&mut page.condition.actor_id).range(1..=5000));
                                    ui.end_row();

                                    ui.checkbox(&mut page.condition.timer_flag, "Timer Remaining:");
                                    ui.horizontal(|ui| {
                                        let mut mins = page.condition.timer_sec / 60;
                                        let mut secs = page.condition.timer_sec % 60;
                                        if ui.add_enabled(page.condition.timer_flag, egui::DragValue::new(&mut mins).range(0..=99).suffix("m")).changed() {
                                            page.condition.timer_sec = mins * 60 + secs;
                                        }
                                        if ui.add_enabled(page.condition.timer_flag, egui::DragValue::new(&mut secs).range(0..=59).suffix("s")).changed() {
                                            page.condition.timer_sec = mins * 60 + secs;
                                        }
                                    });
                                    ui.end_row();
                                });
                        });

                        cols[0].group(|ui| {
                            ui.heading("Graphic & Behavior");
                            ui.horizontal(|ui| {
                                ui.label("CharSet:");
                                if ui.button(if page.character_name.is_empty() { "(None)" } else { &page.character_name }).clicked() {
                                    self.asset_picker.open(project_path, "CharSet", &page.character_name, page.character_index);
                                }
                                if !page.character_name.is_empty() {
                                    ui.label(format!("Idx #{}", page.character_index));
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut page.translucent, "Translucent");
                                ui.checkbox(&mut page.overlap_forbidden, "Forbid Overlap");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Animation:");
                                egui::ComboBox::from_id_salt(format!("page_anim_{}", page.id))
                                    .selected_text(match page.animation_type {
                                        0 => "Normal Walking",
                                        1 => "Continuous Walk in Place",
                                        2 => "Fixed Graphic / Spin",
                                        _ => "Stepping Animation",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut page.animation_type, 0, "Normal Walking");
                                        ui.selectable_value(&mut page.animation_type, 1, "Continuous Walk in Place");
                                        ui.selectable_value(&mut page.animation_type, 2, "Fixed Graphic / Spin");
                                        ui.selectable_value(&mut page.animation_type, 3, "Stepping Animation");
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Trigger:");
                                egui::ComboBox::from_id_salt(format!("page_trigger_{}", page.id))
                                    .selected_text(event_trigger_label(page.trigger))
                                    .show_ui(ui, |ui| {
                                        for t in 0..=4 {
                                            ui.selectable_value(&mut page.trigger, t, event_trigger_label(t));
                                        }
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Layer:");
                                egui::ComboBox::from_id_salt(format!("page_layer_{}", page.id))
                                    .selected_text(event_layer_label(page.layer))
                                    .show_ui(ui, |ui| {
                                        for l in 0..=2 {
                                            ui.selectable_value(&mut page.layer, l, event_layer_label(l));
                                        }
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Move Type:");
                                egui::ComboBox::from_id_salt(format!("page_movetype_{}", page.id))
                                    .selected_text(event_move_type_label(page.move_type))
                                    .show_ui(ui, |ui| {
                                        for m in 0..=6 {
                                            ui.selectable_value(&mut page.move_type, m, event_move_type_label(m));
                                        }
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Speed:");
                                ui.add(egui::Slider::new(&mut page.move_speed, 1..=6));
                                ui.label("Freq:");
                                ui.add(egui::Slider::new(&mut page.move_frequency, 1..=8));
                            });
                        });

                        // Right Column: Event Commands List
                        cols[1].group(|ui| {
                            ui.heading("Event Commands");
                            ui.horizontal(|ui| {
                                ui.label("🔍");
                                ui.add(egui::TextEdit::singleline(&mut self.command_search).hint_text("Filter script / code...").desired_width(180.0));
                                if !self.command_search.is_empty() && ui.small_button("✕").clicked() {
                                    self.command_search.clear();
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.button("+ Add").clicked() {
                                    let indent = self.selected_command.and_then(|i| page.commands.get(i)).map(|c| c.indent).unwrap_or(0);
                                    self.cmd_dialog.open_new(indent);
                                }
                                if ui.add_enabled(self.selected_command.is_some(), egui::Button::new("Edit")).clicked() {
                                    if let Some(idx) = self.selected_command {
                                        if let Some(cmd) = page.commands.get(idx) {
                                            self.cmd_dialog.open_edit(idx, cmd);
                                        }
                                    }
                                }
                                if ui.add_enabled(self.selected_command.is_some(), egui::Button::new("▲")).clicked() {
                                    if let Some(idx) = self.selected_command {
                                        if idx > 0 {
                                            page.commands.swap(idx, idx - 1);
                                            self.selected_command = Some(idx - 1);
                                        }
                                    }
                                }
                                if ui.add_enabled(self.selected_command.is_some(), egui::Button::new("▼")).clicked() {
                                    if let Some(idx) = self.selected_command {
                                        if idx + 1 < page.commands.len() {
                                            page.commands.swap(idx, idx + 1);
                                            self.selected_command = Some(idx + 1);
                                        }
                                    }
                                }
                                if ui.add_enabled(self.selected_command.is_some(), egui::Button::new("Delete")).clicked() {
                                    if let Some(idx) = self.selected_command {
                                        page.commands.remove(idx);
                                        if idx >= page.commands.len() && idx > 0 {
                                            self.selected_command = Some(idx - 1);
                                        } else if page.commands.is_empty() {
                                            self.selected_command = None;
                                        }
                                    }
                                }
                            });

                            egui::ScrollArea::vertical()
                                .id_salt("event_dialog_cmd_list")
                                .max_height(340.0)
                                .show(ui, |ui| {
                                    if page.commands.is_empty() {
                                        ui.colored_label(egui::Color32::GRAY, "(Empty command list - click '+ Add' above)");
                                    }
                                    let q = self.command_search.trim().to_lowercase();
                                    for (c_idx, cmd) in page.commands.iter().enumerate() {
                                        let label = event_command_label(cmd);
                                        let color = crate::lcf_bridge::event_command_color(cmd.code, ui.visuals().dark_mode);
                                        let indent = "|  ".repeat(cmd.indent.max(0) as usize);

                                        let matches_filter = !q.is_empty() && (label.to_lowercase().contains(&q) || cmd.string.to_lowercase().contains(&q) || cmd.code.to_string().contains(&q));
                                        let prefix = if matches_filter { "🎯 " } else { "" };
                                        let display = format!("{}{}{}", prefix, indent, label);

                                        let selected = self.selected_command == Some(c_idx);
                                        let mut rich = egui::RichText::new(display).color(color).monospace();
                                        if matches_filter {
                                            rich = rich.strong().underline();
                                        }
                                        let resp = ui.selectable_label(selected, rich);
                                        if resp.clicked() {
                                            self.selected_command = Some(c_idx);
                                        }
                                        if resp.double_clicked() {
                                            self.cmd_dialog.open_edit(c_idx, cmd);
                                        }
                                    }
                                });
                        });
                    });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save Event").clicked() {
                        self.event.page_count = self.event.pages.len() as i32;
                        if let Some(first) = self.event.pages.first() {
                            self.event.trigger = event_trigger_label(first.trigger).to_string();
                            self.event.graphic = first.character_name.clone();
                        }
                        result = Some(self.event.clone());
                        self.is_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }

        result
    }
}
