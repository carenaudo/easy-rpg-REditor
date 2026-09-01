use eframe::egui;
use lcf_core::{MoveCommand, MoveRoute};
use lcf_core::types::DBString;

pub fn move_command_label(cmd: &MoveCommand) -> String {
    match cmd.code {
        0 => "⬆ Move Up".to_string(),
        1 => "➡ Move Right".to_string(),
        2 => "⬇ Move Down".to_string(),
        3 => "⬅ Move Left".to_string(),
        4 => "↗ Move Up-Right".to_string(),
        5 => "↘ Move Down-Right".to_string(),
        6 => "↙ Move Down-Left".to_string(),
        7 => "↖ Move Up-Left".to_string(),
        8 => "🎲 Move Random".to_string(),
        9 => "🎯 Move Towards Hero".to_string(),
        10 => "🏃 Move Away From Hero".to_string(),
        11 => "🚶 Step Forward".to_string(),
        12 => "👀 Face Up".to_string(),
        13 => "👀 Face Right".to_string(),
        14 => "👀 Face Down".to_string(),
        15 => "👀 Face Left".to_string(),
        16 => "🔄 Turn 90° Right".to_string(),
        17 => "🔄 Turn 90° Left".to_string(),
        18 => "🔁 Turn 180°".to_string(),
        19 => "🔀 Turn 90° Random".to_string(),
        20 => "🎲 Face Random Direction".to_string(),
        21 => "👤 Face Hero".to_string(),
        22 => "🙈 Face Away From Hero".to_string(),
        23 => "⏳ Wait".to_string(),
        24 => "🦘 Begin Jump".to_string(),
        25 => "🦘 End Jump".to_string(),
        26 => "🔒 Lock Facing".to_string(),
        27 => "🔓 Unlock Facing".to_string(),
        28 => "⏩ Increase Movement Speed".to_string(),
        29 => "⏪ Decrease Movement Speed".to_string(),
        30 => "⚡ Increase Movement Frequency".to_string(),
        31 => "💤 Decrease Movement Frequency".to_string(),
        32 => format!("⚡ Switch ON [#{:04}]", cmd.parameter_a),
        33 => format!("⚡ Switch OFF [#{:04}]", cmd.parameter_a),
        34 => format!("🖼 Change Graphic [{}, Idx {}]", if cmd.string.0.is_empty() { "(None)" } else { &cmd.string.0 }, cmd.parameter_a),
        35 => format!("🔊 Play Sound [{}]", if cmd.string.0.is_empty() { "(None)" } else { &cmd.string.0 }),
        36 => "👻 Phasing / Through ON".to_string(),
        37 => "👤 Phasing / Through OFF".to_string(),
        38 => "⏹ Stop Animation".to_string(),
        39 => "▶ Start Animation".to_string(),
        40 => "🌫 Increase Transparency".to_string(),
        41 => "💎 Decrease Transparency".to_string(),
        _ => format!("Custom Move Opcode ({})", cmd.code),
    }
}

pub struct MoveRouteDialogState {
    pub is_open: bool,
    pub route: MoveRoute,
    pub selected_step: Option<usize>,
    pub target_char: i32, // 10001 = Player, 10005 = This Event, or Event ID
    pub show_target_selector: bool,
    pub pending_switch_id: i32,
    pub pending_graphic_name: String,
    pub pending_graphic_idx: i32,
    pub pending_sound_name: String,
}

impl Default for MoveRouteDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            route: MoveRoute {
                move_commands: Vec::new(),
                repeat: true,
                skippable: false,
            },
            selected_step: None,
            target_char: 10005, // This Event
            show_target_selector: false,
            pending_switch_id: 1,
            pending_graphic_name: String::new(),
            pending_graphic_idx: 0,
            pending_sound_name: String::new(),
        }
    }
}

impl MoveRouteDialogState {
    pub fn open_for_event_page(&mut self, route: &MoveRoute) {
        self.is_open = true;
        self.route = route.clone();
        self.selected_step = None;
        self.show_target_selector = false;
    }

    pub fn open_for_event_command(&mut self, route: &MoveRoute, target: i32) {
        self.is_open = true;
        self.route = route.clone();
        self.target_char = target;
        self.selected_step = None;
        self.show_target_selector = true;
    }

    pub fn push_cmd(&mut self, code: i32) {
        let cmd = MoveCommand {
            code,
            parameter_a: 0,
            parameter_b: 0,
            parameter_c: 0,
            string: DBString::default(),
        };
        if let Some(idx) = self.selected_step {
            let insert_pos = (idx + 1).min(self.route.move_commands.len());
            self.route.move_commands.insert(insert_pos, cmd);
            self.selected_step = Some(insert_pos);
        } else {
            self.route.move_commands.push(cmd);
            self.selected_step = Some(self.route.move_commands.len() - 1);
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<(MoveRoute, Option<i32>)> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut is_open = self.is_open;

        egui::Window::new("🚶 Custom Movement Route Editor")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([720.0, 560.0])
            .show(ctx, |ui| {
                if self.show_target_selector {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🎯 Target Character:");
                            egui::ComboBox::from_id_salt("mr_target_char_combo")
                                .selected_text(match self.target_char {
                                    10001 => "Player / Party Leader",
                                    10005 => "This Event",
                                    _ => "Specific Event ID",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.target_char, 10001, "Player / Party Leader");
                                    ui.selectable_value(&mut self.target_char, 10005, "This Event");
                                    ui.selectable_value(&mut self.target_char, 1, "Specific Event ID");
                                });
                            if self.target_char != 10001 && self.target_char != 10005 {
                                ui.add(egui::DragValue::new(&mut self.target_char).range(1..=5000).prefix("#"));
                            }
                        });
                    });
                    ui.separator();
                }

                ui.columns(2, |cols| {
                    // Left Column: Movement Step Sequence List
                    cols[0].group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(format!("Movement Steps ({})", self.route.move_commands.len()));
                            if ui.small_button("🧹 Clear").clicked() {
                                self.route.move_commands.clear();
                                self.selected_step = None;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.route.repeat, "🔁 Repeat Route");
                            ui.checkbox(&mut self.route.skippable, "⏩ Skip if Blocked");
                        });

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .id_salt("mr_step_list_scroll")
                            .max_height(360.0)
                            .show(ui, |ui| {
                                for (idx, cmd) in self.route.move_commands.iter().enumerate() {
                                    let is_selected = self.selected_step == Some(idx);
                                    let label = format!("{:03}: {}", idx + 1, move_command_label(cmd));
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        self.selected_step = Some(idx);
                                    }
                                }
                            });

                        ui.separator();
                        ui.horizontal(|ui| {
                            let has_sel = self.selected_step.is_some();
                            if ui.add_enabled(has_sel, egui::Button::new("▲ Move Up")).clicked() {
                                if let Some(idx) = self.selected_step {
                                    if idx > 0 {
                                        self.route.move_commands.swap(idx, idx - 1);
                                        self.selected_step = Some(idx - 1);
                                    }
                                }
                            }
                            if ui.add_enabled(has_sel, egui::Button::new("▼ Move Down")).clicked() {
                                if let Some(idx) = self.selected_step {
                                    if idx + 1 < self.route.move_commands.len() {
                                        self.route.move_commands.swap(idx, idx + 1);
                                        self.selected_step = Some(idx + 1);
                                    }
                                }
                            }
                            if ui.add_enabled(has_sel, egui::Button::new("🗑 Delete")).clicked() {
                                if let Some(idx) = self.selected_step {
                                    if idx < self.route.move_commands.len() {
                                        self.route.move_commands.remove(idx);
                                        if idx >= self.route.move_commands.len() {
                                            self.selected_step = self.route.move_commands.len().checked_sub(1);
                                        }
                                    }
                                }
                            }
                        });
                    });

                    // Right Column: 4-Category Movement Command Palette
                    cols[1].group(|ui| {
                        ui.heading("Command Palette");

                        egui::ScrollArea::vertical()
                            .id_salt("mr_palette_scroll")
                            .max_height(420.0)
                            .show(ui, |ui| {
                                // 1. Directional Movement
                                ui.collapsing("🚶 Directional Steps", |ui| {
                                    egui::Grid::new("mr_dir_grid").num_columns(3).spacing([6.0, 4.0]).show(ui, |ui| {
                                        if ui.button("↖ Up-Left").clicked() { self.push_cmd(7); }
                                        if ui.button("⬆ Move Up").clicked() { self.push_cmd(0); }
                                        if ui.button("↗ Up-Right").clicked() { self.push_cmd(4); }
                                        ui.end_row();

                                        if ui.button("⬅ Move Left").clicked() { self.push_cmd(3); }
                                        if ui.button("🎲 Random").clicked() { self.push_cmd(8); }
                                        if ui.button("➡ Move Right").clicked() { self.push_cmd(1); }
                                        ui.end_row();

                                        if ui.button("↙ Down-Left").clicked() { self.push_cmd(6); }
                                        if ui.button("⬇ Move Down").clicked() { self.push_cmd(2); }
                                        if ui.button("↘ Down-Right").clicked() { self.push_cmd(5); }
                                        ui.end_row();
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("🎯 Toward Hero").clicked() { self.push_cmd(9); }
                                        if ui.button("🏃 Away Hero").clicked() { self.push_cmd(10); }
                                        if ui.button("🚶 Step Forward").clicked() { self.push_cmd(11); }
                                    });
                                });

                                // 2. Facing & Turning
                                ui.collapsing("👀 Facing & Turning", |ui| {
                                    egui::Grid::new("mr_facing_grid").num_columns(2).spacing([6.0, 4.0]).show(ui, |ui| {
                                        if ui.button("👀 Face Up").clicked() { self.push_cmd(12); }
                                        if ui.button("👀 Face Right").clicked() { self.push_cmd(13); }
                                        ui.end_row();
                                        if ui.button("👀 Face Down").clicked() { self.push_cmd(14); }
                                        if ui.button("👀 Face Left").clicked() { self.push_cmd(15); }
                                        ui.end_row();
                                        if ui.button("🔄 Turn 90° R").clicked() { self.push_cmd(16); }
                                        if ui.button("🔄 Turn 90° L").clicked() { self.push_cmd(17); }
                                        ui.end_row();
                                        if ui.button("🔁 Turn 180°").clicked() { self.push_cmd(18); }
                                        if ui.button("🔀 Turn Random").clicked() { self.push_cmd(19); }
                                        ui.end_row();
                                        if ui.button("🎲 Face Random").clicked() { self.push_cmd(20); }
                                        if ui.button("👤 Face Hero").clicked() { self.push_cmd(21); }
                                        ui.end_row();
                                        if ui.button("🙈 Face Away").clicked() { self.push_cmd(22); }
                                        ui.end_row();
                                    });
                                });

                                // 3. Action & Timing
                                ui.collapsing("⏳ Timing, Jump & Pacing", |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("⏳ Wait").clicked() { self.push_cmd(23); }
                                        if ui.button("🦘 Begin Jump").clicked() { self.push_cmd(24); }
                                        if ui.button("🦘 End Jump").clicked() { self.push_cmd(25); }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("🔒 Lock Facing").clicked() { self.push_cmd(26); }
                                        if ui.button("🔓 Unlock Facing").clicked() { self.push_cmd(27); }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("⏩ Speed Up").clicked() { self.push_cmd(28); }
                                        if ui.button("⏪ Speed Down").clicked() { self.push_cmd(29); }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("⚡ Freq Up").clicked() { self.push_cmd(30); }
                                        if ui.button("💤 Freq Down").clicked() { self.push_cmd(31); }
                                    });
                                });

                                // 4. Switches & Effects
                                ui.collapsing("⚡ Switches, Graphic & Effects", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Switch #:");
                                        ui.add(egui::DragValue::new(&mut self.pending_switch_id).range(1..=5000));
                                        if ui.button("ON").clicked() {
                                            let cmd = MoveCommand {
                                                code: 32,
                                                parameter_a: self.pending_switch_id,
                                                ..Default::default()
                                            };
                                            self.route.move_commands.push(cmd);
                                        }
                                        if ui.button("OFF").clicked() {
                                            let cmd = MoveCommand {
                                                code: 33,
                                                parameter_a: self.pending_switch_id,
                                                ..Default::default()
                                            };
                                            self.route.move_commands.push(cmd);
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Graphic:");
                                        ui.add(egui::TextEdit::singleline(&mut self.pending_graphic_name).desired_width(90.0));
                                        ui.label("Idx:");
                                        ui.add(egui::DragValue::new(&mut self.pending_graphic_idx).range(0..=7));
                                        if ui.button("+ Add").clicked() {
                                            let cmd = MoveCommand {
                                                code: 34,
                                                parameter_a: self.pending_graphic_idx,
                                                string: DBString::new(self.pending_graphic_name.clone()),
                                                ..Default::default()
                                            };
                                            self.route.move_commands.push(cmd);
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Sound:");
                                        ui.add(egui::TextEdit::singleline(&mut self.pending_sound_name).desired_width(120.0));
                                        if ui.button("+ Add").clicked() {
                                            let cmd = MoveCommand {
                                                code: 35,
                                                parameter_a: 100, // volume
                                                parameter_b: 100, // tempo
                                                parameter_c: 50,  // balance
                                                string: DBString::new(self.pending_sound_name.clone()),
                                            };
                                            self.route.move_commands.push(cmd);
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        if ui.button("👻 Phasing ON").clicked() { self.push_cmd(36); }
                                        if ui.button("👤 Phasing OFF").clicked() { self.push_cmd(37); }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("⏹ Stop Anim").clicked() { self.push_cmd(38); }
                                        if ui.button("▶ Start Anim").clicked() { self.push_cmd(39); }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("🌫 Transp +").clicked() { self.push_cmd(40); }
                                        if ui.button("💎 Transp -").clicked() { self.push_cmd(41); }
                                    });
                                });
                            });
                    });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("✔ OK / Apply").clicked() {
                        let target = if self.show_target_selector { Some(self.target_char) } else { None };
                        result = Some((self.route.clone(), target));
                        self.is_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        self.is_open = is_open;
        result
    }
}
