use eframe::egui;
use crate::lcf_bridge::{self, EventCommandInfo};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommandCategory {
    Messages,
    Progression,
    Character,
    Movement,
    AudioVisual,
    FlowControl,
    SystemScenes,
    Maniac,
}

/// All 27 assigned Maniac Patch command codes, in the same order used by
/// `lcf_bridge::maniac_command_name` (used to populate the Maniac tab's
/// command picker).
const MANIAC_CODES: [i32; 27] = [
    3001, 3002, 3003, 3004, 3005, 3006, 3007, 3008, 3009, 3010, 3011, 3012, 3013, 3014, 3015,
    3016, 3017, 3018, 3019, 3020, 3021, 3025, 3026, 3027, 3028, 3029, 3032,
];

pub struct EventCommandDialogState {
    pub is_open: bool,
    pub edit_index: Option<usize>,
    pub category: CommandCategory,
    pub selected_code: i32,
    pub indent: i32,
    pub string_val: String,
    pub param0: i32,
    pub param1: i32,
    pub param2: i32,
    pub param3: i32,
    pub param4: i32,
    pub param5: i32,
    pub param6: i32,
    /// Full parameter vector, kept in sync whenever a command is opened and
    /// live-mutated by every Maniac Patch editor arm (bespoke or generic).
    /// This is the actual save-time source of truth for Maniac commands and
    /// for any other code with no bespoke `param0..5` arm - unlike those six
    /// fixed scalar fields, it is never truncated, which is what makes
    /// editing a command with more than 6 parameters (or any unrecognized
    /// code) lossless.
    pub raw_params: Vec<i32>,
}

impl Default for EventCommandDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            edit_index: None,
            category: CommandCategory::Messages,
            selected_code: 10110,
            indent: 0,
            string_val: String::new(),
            param0: 0,
            param1: 0,
            param2: 0,
            param3: 0,
            param4: 0,
            param5: 0,
            param6: 0,
            raw_params: Vec::new(),
        }
    }
}

impl EventCommandDialogState {
    /// Mutable access to `raw_params[i]`, growing the vector with zeros as
    /// needed. Used by every Maniac editor arm (bespoke or generic) instead
    /// of the fixed `param0..5` fields, since several Maniac commands need
    /// more than six parameters (e.g. `ShowStringPicture`'s 23).
    fn param_mut(&mut self, i: usize) -> &mut i32 {
        if self.raw_params.len() <= i {
            self.raw_params.resize(i + 1, 0);
        }
        &mut self.raw_params[i]
    }

    pub fn open_new(&mut self, current_indent: i32) {
        self.is_open = true;
        self.edit_index = None;
        self.category = CommandCategory::Messages;
        self.selected_code = 10110; // Show Message
        self.indent = current_indent;
        self.string_val = String::new();
        self.param0 = 0;
        self.param1 = 0;
        self.param2 = 0;
        self.param3 = 0;
        self.param4 = 0;
        self.param5 = 0;
        self.param6 = 0;
        self.raw_params = Vec::new();
    }

    pub fn open_edit(&mut self, index: usize, cmd: &EventCommandInfo) {
        self.is_open = true;
        self.edit_index = Some(index);
        self.selected_code = cmd.code;
        self.indent = cmd.indent;
        self.string_val = cmd.string.clone();
        self.param0 = cmd.parameters.first().copied().unwrap_or(0);
        self.param1 = cmd.parameters.get(1).copied().unwrap_or(0);
        self.param2 = cmd.parameters.get(2).copied().unwrap_or(0);
        self.param3 = cmd.parameters.get(3).copied().unwrap_or(0);
        self.param4 = cmd.parameters.get(4).copied().unwrap_or(0);
        self.param5 = cmd.parameters.get(5).copied().unwrap_or(0);
        self.param6 = cmd.parameters.get(6).copied().unwrap_or(0);
        // Always the full vector, never truncated - see `raw_params` doc.
        self.raw_params = cmd.parameters.clone();

        // Auto-detect category from code
        self.category = match cmd.code {
            10110..=10150 => CommandCategory::Messages,
            10210..=10330 | 11610 => CommandCategory::Progression,
            10410..=10490 => CommandCategory::Character,
            10810..=10870 | 11310..=11410 => CommandCategory::Movement,
            11010..=11210 | 11510..=11560 | 11710..=11720 => CommandCategory::AudioVisual,
            12010..=12410 | 20140..=22210 => CommandCategory::FlowControl,
            10710..=10740 | 11810..=11960 | 12420 | 12510 => CommandCategory::SystemScenes,
            3001..=3032 => CommandCategory::Maniac,
            _ => CommandCategory::Messages,
        };
    }

    /// Returns Some((index, EventCommandInfo)) when submitted.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<(Option<usize>, EventCommandInfo)> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut is_open = self.is_open;

        egui::Window::new(if self.edit_index.is_some() { "Edit Event Command" } else { "Add Event Command" })
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([540.0, 420.0])
            .show(ctx, |ui| {
                // Category Bar
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.category, CommandCategory::Messages, "💬 Messages");
                    ui.selectable_value(&mut self.category, CommandCategory::Progression, "🔘 Switches & Items");
                    ui.selectable_value(&mut self.category, CommandCategory::Character, "👤 Character & Stats");
                    ui.selectable_value(&mut self.category, CommandCategory::Movement, "🗺 Movement");
                    ui.selectable_value(&mut self.category, CommandCategory::AudioVisual, "🎵 Audio & Screen");
                    ui.selectable_value(&mut self.category, CommandCategory::FlowControl, "🌀 Logic & Flow");
                    ui.selectable_value(&mut self.category, CommandCategory::SystemScenes, "⚔ Scenes & System");
                    ui.selectable_value(&mut self.category, CommandCategory::Maniac, egui::RichText::new("🔧 Maniac Patch").color(egui::Color32::from_rgb(200, 140, 255)));
                });

                ui.separator();

                // Command selector within chosen category
                let code_before_picker = self.selected_code;
                ui.horizontal(|ui| {
                    ui.label("Command:");
                    egui::ComboBox::from_id_salt("cmd_type_sub_combo")
                        .selected_text(if lcf_bridge::is_maniac_command_code(self.selected_code) {
                            format!("{}: Maniac {}", self.selected_code, lcf_bridge::maniac_command_name(self.selected_code))
                        } else {
                            match self.selected_code {
                            10110 => "10110: Show Message",
                            10120 => "10120: Message Options",
                            10140 => "10140: Show Choices",
                            10150 => "10150: Input Number",
                            10210 => "10210: Control Switches",
                            10220 => "10220: Control Variables",
                            10310 => "10310: Change Gold",
                            10320 => "10320: Change Items",
                            10330 => "10330: Change Party Members",
                            10410 => "10410: Change EXP",
                            10420 => "10420: Change Level",
                            10430 => "10430: Change Parameters",
                            10440 => "10440: Change Skills",
                            10450 => "10450: Change Equipment",
                            10460 => "10460: Change HP",
                            10470 => "10470: Change SP",
                            10480 => "10480: Change Condition / State",
                            10490 => "10490: Recover All",
                            10810 => "10810: Transfer Player (Teleport)",
                            10860 => "10860: Set Event Location",
                            11330 => "11330: Set Move Route",
                            11410 => "11410: Wait",
                            11510 => "11510: Play BGM",
                            11520 => "11520: Fade Out BGM",
                            11550 => "11550: Play Sound Effect (SE)",
                            11010 => "11010: Erase / Show Screen",
                            11030 => "11030: Tint Screen",
                            11040 => "11040: Flash Screen",
                            11050 => "11050: Shake Screen",
                            11070 => "11070: Weather Effects",
                            11110 => "11110: Show Picture",
                            11120 => "11120: Move Picture",
                            11130 => "11130: Erase Picture",
                            11210 => "11210: Show Battle Animation",
                            12010 => "12010: Conditional Branch",
                            12210 => "12210: Loop",
                            12220 => "12220: Break Loop",
                            12310 => "12310: Exit Event Processing",
                            12320 => "12320: Erase Event",
                            12330 => "12330: Call Common Event",
                            12410 => "12410: Comment",
                            10710 => "10710: Battle Processing",
                            10720 => "10720: Shop Processing",
                            10730 => "10730: Inn Processing",
                            10740 => "10740: Hero Name Input",
                            11910 => "11910: Open Save Menu",
                            11950 => "11950: Open Main Menu",
                            12420 => "12420: Game Over",
                            12510 => "12510: Return to Title Screen",
                            _ => "Custom Event Command",
                            }.to_string()
                        })
                        .show_ui(ui, |ui| {
                            match self.category {
                                CommandCategory::Messages => {
                                    ui.selectable_value(&mut self.selected_code, 10110, "Show Message");
                                    ui.selectable_value(&mut self.selected_code, 10120, "Message Options");
                                    ui.selectable_value(&mut self.selected_code, 10140, "Show Choices");
                                    ui.selectable_value(&mut self.selected_code, 10150, "Input Number");
                                }
                                CommandCategory::Progression => {
                                    ui.selectable_value(&mut self.selected_code, 10210, "Control Switches");
                                    ui.selectable_value(&mut self.selected_code, 10220, "Control Variables");
                                    ui.selectable_value(&mut self.selected_code, 10310, "Change Gold");
                                    ui.selectable_value(&mut self.selected_code, 10320, "Change Items");
                                    ui.selectable_value(&mut self.selected_code, 10330, "Change Party Members");
                                }
                                CommandCategory::Character => {
                                    ui.selectable_value(&mut self.selected_code, 10410, "Change EXP");
                                    ui.selectable_value(&mut self.selected_code, 10420, "Change Level");
                                    ui.selectable_value(&mut self.selected_code, 10430, "Change Parameters");
                                    ui.selectable_value(&mut self.selected_code, 10440, "Change Skills");
                                    ui.selectable_value(&mut self.selected_code, 10450, "Change Equipment");
                                    ui.selectable_value(&mut self.selected_code, 10460, "Change HP");
                                    ui.selectable_value(&mut self.selected_code, 10470, "Change SP");
                                    ui.selectable_value(&mut self.selected_code, 10480, "Change Condition");
                                    ui.selectable_value(&mut self.selected_code, 10490, "Recover All");
                                }
                                CommandCategory::Movement => {
                                    ui.selectable_value(&mut self.selected_code, 10810, "Transfer Player (Teleport)");
                                    ui.selectable_value(&mut self.selected_code, 10860, "Set Event Location");
                                    ui.selectable_value(&mut self.selected_code, 11330, "Set Move Route");
                                    ui.selectable_value(&mut self.selected_code, 11410, "Wait");
                                }
                                CommandCategory::AudioVisual => {
                                    ui.selectable_value(&mut self.selected_code, 11510, "Play BGM");
                                    ui.selectable_value(&mut self.selected_code, 11520, "Fade Out BGM");
                                    ui.selectable_value(&mut self.selected_code, 11550, "Play Sound Effect (SE)");
                                    ui.selectable_value(&mut self.selected_code, 11010, "Erase / Show Screen");
                                    ui.selectable_value(&mut self.selected_code, 11030, "Tint Screen");
                                    ui.selectable_value(&mut self.selected_code, 11040, "Flash Screen");
                                    ui.selectable_value(&mut self.selected_code, 11050, "Shake Screen");
                                    ui.selectable_value(&mut self.selected_code, 11070, "Weather Effects");
                                    ui.selectable_value(&mut self.selected_code, 11110, "Show Picture");
                                    ui.selectable_value(&mut self.selected_code, 11120, "Move Picture");
                                    ui.selectable_value(&mut self.selected_code, 11130, "Erase Picture");
                                    ui.selectable_value(&mut self.selected_code, 11210, "Show Battle Animation");
                                }
                                CommandCategory::FlowControl => {
                                    ui.selectable_value(&mut self.selected_code, 12010, "Conditional Branch");
                                    ui.selectable_value(&mut self.selected_code, 12210, "Loop");
                                    ui.selectable_value(&mut self.selected_code, 12220, "Break Loop");
                                    ui.selectable_value(&mut self.selected_code, 12310, "Exit Event Processing");
                                    ui.selectable_value(&mut self.selected_code, 12320, "Erase Event");
                                    ui.selectable_value(&mut self.selected_code, 12330, "Call Common Event");
                                    ui.selectable_value(&mut self.selected_code, 12410, "Comment");
                                }
                                CommandCategory::SystemScenes => {
                                    ui.selectable_value(&mut self.selected_code, 10710, "Battle Processing");
                                    ui.selectable_value(&mut self.selected_code, 10720, "Shop Processing");
                                    ui.selectable_value(&mut self.selected_code, 10730, "Inn Processing");
                                    ui.selectable_value(&mut self.selected_code, 10740, "Hero Name Input");
                                    ui.selectable_value(&mut self.selected_code, 11910, "Open Save Menu");
                                    ui.selectable_value(&mut self.selected_code, 11950, "Open Main Menu");
                                    ui.selectable_value(&mut self.selected_code, 12420, "Game Over");
                                    ui.selectable_value(&mut self.selected_code, 12510, "Return to Title");
                                }
                                CommandCategory::Maniac => {
                                    for &code in MANIAC_CODES.iter() {
                                        ui.selectable_value(&mut self.selected_code, code, format!("Maniac {}", lcf_bridge::maniac_command_name(code)));
                                    }
                                }
                            }
                        });
                });

                // If the picker just switched to a Maniac command (or to a
                // different Maniac command), resize `raw_params` to that
                // command's known parameter count so its editor arm has
                // slots to work with - but only on an actual change, so
                // free-form +/- resizing by the generic editor below isn't
                // clobbered every frame.
                if self.selected_code != code_before_picker && lcf_bridge::is_maniac_command_code(self.selected_code) {
                    let count = lcf_bridge::maniac_param_count(self.selected_code).unwrap_or(4);
                    self.raw_params = vec![0; count];
                }

                ui.separator();

                // Detailed Parameter Editor
                egui::ScrollArea::vertical()
                    .id_salt("cmd_params_scroll")
                    .max_height(240.0)
                    .show(ui, |ui| {
                        match self.selected_code {
                            10110 => {
                                ui.label("Message text:");
                                ui.text_edit_multiline(&mut self.string_val);
                            }
                            10120 => {
                                ui.label("Message Options:");
                                ui.horizontal(|ui| {
                                    ui.label("Position:");
                                    ui.radio_value(&mut self.param0, 0, "Top");
                                    ui.radio_value(&mut self.param0, 1, "Center");
                                    ui.radio_value(&mut self.param0, 2, "Bottom");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Window Transparency:");
                                    ui.radio_value(&mut self.param1, 0, "Normal");
                                    ui.radio_value(&mut self.param1, 1, "Transparent");
                                });
                            }
                            10140 => {
                                ui.label("Choices (slash-separated, e.g. Yes/No/Cancel):");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            10150 => {
                                ui.horizontal(|ui| {
                                    ui.label("Store Result in Variable ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Number of Digits:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=6));
                                });
                            }
                            10210 => {
                                ui.horizontal(|ui| {
                                    ui.label("Switch ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param3, 0, "ON");
                                    ui.radio_value(&mut self.param3, 1, "OFF");
                                    ui.radio_value(&mut self.param3, 2, "Toggle");
                                });
                            }
                            10220 => {
                                ui.heading("Target Variable");
                                ui.horizontal_wrapped(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Single");
                                    ui.radio_value(&mut self.param0, 1, "Range");
                                    ui.radio_value(&mut self.param0, 2, "Indirect");
                                });
                                ui.horizontal(|ui| {
                                    if self.param0 == 1 {
                                        ui.label("Start Var ID:");
                                        ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                        ui.label("End Var ID:");
                                        ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                    } else if self.param0 == 2 {
                                        ui.label("Pointer Var ID (V[ID]):");
                                        ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                    } else {
                                        ui.label("Variable ID:");
                                        ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                    }
                                });

                                ui.separator();
                                ui.heading("Operation");
                                ui.horizontal_wrapped(|ui| {
                                    ui.radio_value(&mut self.param3, 0, "Set (=)");
                                    ui.radio_value(&mut self.param3, 1, "Add (+)");
                                    ui.radio_value(&mut self.param3, 2, "Subtract (-)");
                                    ui.radio_value(&mut self.param3, 3, "Multiply (×)");
                                    ui.radio_value(&mut self.param3, 4, "Divide (÷)");
                                    ui.radio_value(&mut self.param3, 5, "Modulo (%)");
                                });

                                ui.separator();
                                ui.heading("Operand");
                                ui.horizontal_wrapped(|ui| {
                                    ui.radio_value(&mut self.param4, 0, "Constant");
                                    ui.radio_value(&mut self.param4, 1, "Variable");
                                    ui.radio_value(&mut self.param4, 2, "Indirect Var");
                                    ui.radio_value(&mut self.param4, 3, "Random");
                                    ui.radio_value(&mut self.param4, 4, "Item");
                                    ui.radio_value(&mut self.param4, 5, "Hero");
                                    ui.radio_value(&mut self.param4, 6, "Character");
                                    ui.radio_value(&mut self.param4, 7, "Other");
                                });

                                match self.param4 {
                                    0 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Constant Value:");
                                            ui.add(egui::DragValue::new(&mut self.param5));
                                        });
                                    }
                                    1 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Source Variable ID:");
                                            ui.add(egui::DragValue::new(&mut self.param5).range(1..=5000));
                                        });
                                    }
                                    2 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Pointer Variable ID (V[ID]):");
                                            ui.add(egui::DragValue::new(&mut self.param5).range(1..=5000));
                                        });
                                    }
                                    3 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Min:");
                                            ui.add(egui::DragValue::new(&mut self.param5));
                                            ui.label("Max:");
                                            ui.add(egui::DragValue::new(&mut self.param6));
                                        });
                                    }
                                    4 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Item ID:");
                                            ui.add(egui::DragValue::new(&mut self.param5).range(1..=5000));
                                            ui.radio_value(&mut self.param6, 0, "In Inventory");
                                            ui.radio_value(&mut self.param6, 1, "Equipped");
                                        });
                                    }
                                    5 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Hero / Actor ID:");
                                            ui.add(egui::DragValue::new(&mut self.param5).range(1..=5000));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Stat / Property:");
                                            egui::ComboBox::from_id_salt("cmd_cv_hero_stat")
                                                .selected_text(match self.param6 {
                                                    0 => "Level", 1 => "EXP", 2 => "HP", 3 => "SP", 4 => "Max HP", 5 => "Max SP",
                                                    6 => "Attack", 7 => "Defense", 8 => "Spirit", 9 => "Agility",
                                                    10 => "Weapon ID", 11 => "Shield ID", 12 => "Armor ID", 13 => "Helmet ID", 14 => "Accessory ID",
                                                    _ => "Level",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param6, 0, "Level");
                                                    ui.selectable_value(&mut self.param6, 1, "EXP");
                                                    ui.selectable_value(&mut self.param6, 2, "HP");
                                                    ui.selectable_value(&mut self.param6, 3, "SP");
                                                    ui.selectable_value(&mut self.param6, 4, "Max HP");
                                                    ui.selectable_value(&mut self.param6, 5, "Max SP");
                                                    ui.selectable_value(&mut self.param6, 6, "Attack");
                                                    ui.selectable_value(&mut self.param6, 7, "Defense");
                                                    ui.selectable_value(&mut self.param6, 8, "Spirit");
                                                    ui.selectable_value(&mut self.param6, 9, "Agility");
                                                    ui.selectable_value(&mut self.param6, 10, "Weapon ID");
                                                    ui.selectable_value(&mut self.param6, 11, "Shield ID");
                                                    ui.selectable_value(&mut self.param6, 12, "Armor ID");
                                                    ui.selectable_value(&mut self.param6, 13, "Helmet ID");
                                                    ui.selectable_value(&mut self.param6, 14, "Accessory ID");
                                                });
                                        });
                                    }
                                    6 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Target Character:");
                                            egui::ComboBox::from_id_salt("cmd_cv_char_target")
                                                .selected_text(match self.param5 {
                                                    10001 => "Player / Party Leader",
                                                    10005 => "This Event",
                                                    _ => "Specific Event ID",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param5, 10001, "Player / Party Leader");
                                                    ui.selectable_value(&mut self.param5, 10005, "This Event");
                                                    ui.selectable_value(&mut self.param5, 1, "Specific Event ID");
                                                });
                                            if self.param5 != 10001 && self.param5 != 10005 {
                                                ui.add(egui::DragValue::new(&mut self.param5).range(1..=5000));
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Property:");
                                            egui::ComboBox::from_id_salt("cmd_cv_char_prop")
                                                .selected_text(match self.param6 {
                                                    0 => "Map ID", 1 => "X Coordinate", 2 => "Y Coordinate",
                                                    3 => "Facing Direction", 4 => "Screen X", 5 => "Screen Y",
                                                    _ => "Map ID",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param6, 0, "Map ID");
                                                    ui.selectable_value(&mut self.param6, 1, "X Coordinate");
                                                    ui.selectable_value(&mut self.param6, 2, "Y Coordinate");
                                                    ui.selectable_value(&mut self.param6, 3, "Facing Direction");
                                                    ui.selectable_value(&mut self.param6, 4, "Screen X");
                                                    ui.selectable_value(&mut self.param6, 5, "Screen Y");
                                                });
                                        });
                                    }
                                    7 => {
                                        ui.horizontal(|ui| {
                                            ui.label("System Property:");
                                            egui::ComboBox::from_id_salt("cmd_cv_other_prop")
                                                .selected_text(match self.param5 {
                                                    0 => "Gold / Money",
                                                    1 => "Timer 1 (Seconds Left)",
                                                    2 => "Party Size",
                                                    3 => "Save Count",
                                                    4 => "Battle Count",
                                                    5 => "Victories Count",
                                                    6 => "Defeats Count",
                                                    7 => "Escapes Count",
                                                    8 => "MIDI Play Position (Ticks)",
                                                    9 => "Timer 2 (Seconds Left)",
                                                    _ => "Gold / Money",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param5, 0, "Gold / Money");
                                                    ui.selectable_value(&mut self.param5, 1, "Timer 1 (Seconds Left)");
                                                    ui.selectable_value(&mut self.param5, 2, "Party Size");
                                                    ui.selectable_value(&mut self.param5, 3, "Save Count");
                                                    ui.selectable_value(&mut self.param5, 4, "Battle Count");
                                                    ui.selectable_value(&mut self.param5, 5, "Victories Count");
                                                    ui.selectable_value(&mut self.param5, 6, "Defeats Count");
                                                    ui.selectable_value(&mut self.param5, 7, "Escapes Count");
                                                    ui.selectable_value(&mut self.param5, 8, "MIDI Play Position (Ticks)");
                                                    ui.selectable_value(&mut self.param5, 9, "Timer 2 (Seconds Left)");
                                                });
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            10310 => {
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Increase");
                                    ui.radio_value(&mut self.param0, 1, "Decrease");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Amount (Gold):");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=999999));
                                });
                            }
                            10320 => {
                                ui.horizontal(|ui| {
                                    ui.label("Item ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Add (+)");
                                    ui.radio_value(&mut self.param0, 1, "Remove (-)");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Quantity:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=99));
                                });
                            }
                            10330 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Party Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Add to Party");
                                    ui.radio_value(&mut self.param0, 1, "Remove from Party");
                                });
                            }
                            10410..=10430 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Increase");
                                    ui.radio_value(&mut self.param0, 1, "Decrease");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Amount:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=999999));
                                });
                            }
                            10440 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Skill ID:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Learn");
                                    ui.radio_value(&mut self.param0, 1, "Forget");
                                });
                            }
                            10460 | 10470 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Recover / Increase");
                                    ui.radio_value(&mut self.param0, 1, "Drain / Decrease");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Amount:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=99999));
                                });
                            }
                            10480 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Condition / State ID:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Inflict");
                                    ui.radio_value(&mut self.param0, 1, "Heal / Remove");
                                });
                            }
                            10490 => {
                                ui.horizontal(|ui| {
                                    ui.label("Target:");
                                    ui.radio_value(&mut self.param0, 0, "Entire Party");
                                    ui.radio_value(&mut self.param0, 1, "Specific Actor");
                                });
                                if self.param0 == 1 {
                                    ui.horizontal(|ui| {
                                        ui.label("Hero / Actor ID:");
                                        ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                    });
                                }
                            }
                            10810 => {
                                ui.horizontal(|ui| {
                                    ui.label("Target Map ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=9999));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("X coordinate:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(0..=500));
                                    ui.label("Y coordinate:");
                                    ui.add(egui::DragValue::new(&mut self.param3).range(0..=500));
                                });
                            }
                            11410 => {
                                ui.horizontal(|ui| {
                                    ui.label("Duration (tenths of sec):");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=300));
                                    ui.label(format!("({:.1}s)", self.param0 as f32 / 10.0));
                                });
                            }
                            11510 => {
                                ui.horizontal(|ui| {
                                    ui.label("BGM Name:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_bgm_combo", &mut self.string_val, "Music", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Volume (%):");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(0..=100));
                                    ui.label("Tempo (%):");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(50..=150));
                                });
                            }
                            11550 => {
                                ui.horizontal(|ui| {
                                    ui.label("Sound (SE) Name:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_se_combo", &mut self.string_val, "Sound", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Volume (%):");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(0..=100));
                                    ui.label("Tempo (%):");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(50..=150));
                                });
                            }
                            11030 => {
                                ui.label("Screen Color Tint:");
                                ui.horizontal(|ui| {
                                    ui.label("Red:"); ui.add(egui::DragValue::new(&mut self.param0).range(-31..=31));
                                    ui.label("Green:"); ui.add(egui::DragValue::new(&mut self.param1).range(-31..=31));
                                    ui.label("Blue:"); ui.add(egui::DragValue::new(&mut self.param2).range(-31..=31));
                                    ui.label("Chroma:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=31));
                                });
                            }
                            11070 => {
                                ui.horizontal(|ui| {
                                    ui.label("Weather Effect:");
                                    ui.radio_value(&mut self.param0, 0, "None");
                                    ui.radio_value(&mut self.param0, 1, "Rain");
                                    ui.radio_value(&mut self.param0, 2, "Snow");
                                    ui.radio_value(&mut self.param0, 3, "Sandstorm");
                                });
                            }
                            11110 | 11120 => {
                                ui.horizontal(|ui| {
                                    ui.label("Picture Number (1..50):");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=50));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Picture Graphic:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_picture_combo", &mut self.string_val, "Picture", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("X:"); ui.add(egui::DragValue::new(&mut self.param1).range(0..=640));
                                    ui.label("Y:"); ui.add(egui::DragValue::new(&mut self.param2).range(0..=480));
                                });
                            }
                            11130 => {
                                ui.horizontal(|ui| {
                                    ui.label("Picture Number to Erase:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=50));
                                });
                            }
                            12330 => {
                                ui.horizontal(|ui| {
                                    ui.label("Call Common Event ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                });
                            }
                            10710 => {
                                ui.horizontal(|ui| {
                                    ui.label("Battle Troop ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                            }
                            12010 => {
                                ui.heading("Branch Condition");
                                ui.horizontal_wrapped(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Switch");
                                    ui.radio_value(&mut self.param0, 1, "Variable");
                                    ui.radio_value(&mut self.param0, 2, "Timer 1");
                                    ui.radio_value(&mut self.param0, 3, "Gold");
                                    ui.radio_value(&mut self.param0, 4, "Item");
                                    ui.radio_value(&mut self.param0, 5, "Hero");
                                    ui.radio_value(&mut self.param0, 6, "Facing");
                                    ui.radio_value(&mut self.param0, 7, "Vehicle");
                                    ui.radio_value(&mut self.param0, 8, "Action Key");
                                    ui.radio_value(&mut self.param0, 9, "BGM Loop");
                                    ui.radio_value(&mut self.param0, 10, "Timer 2");
                                });

                                ui.separator();
                                match self.param0 {
                                    0 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Switch ID:");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                            ui.label("State:");
                                            ui.radio_value(&mut self.param2, 0, "ON");
                                            ui.radio_value(&mut self.param2, 1, "OFF");
                                        });
                                    }
                                    1 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Variable ID:");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Comparison Operator:");
                                            egui::ComboBox::from_id_salt("cmd_cb_var_op")
                                                .selected_text(match self.param4 {
                                                    0 => "Equal to (==)",
                                                    1 => "Greater or Equal (>=)",
                                                    2 => "Less or Equal (<=)",
                                                    3 => "Greater Than (>)",
                                                    4 => "Less Than (<)",
                                                    5 => "Not Equal (!=)",
                                                    _ => "Equal to (==)",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param4, 0, "Equal to (==)");
                                                    ui.selectable_value(&mut self.param4, 1, "Greater or Equal (>=)");
                                                    ui.selectable_value(&mut self.param4, 2, "Less or Equal (<=)");
                                                    ui.selectable_value(&mut self.param4, 3, "Greater Than (>)");
                                                    ui.selectable_value(&mut self.param4, 4, "Less Than (<)");
                                                    ui.selectable_value(&mut self.param4, 5, "Not Equal (!=)");
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Compare with:");
                                            ui.radio_value(&mut self.param2, 0, "Constant Value");
                                            ui.radio_value(&mut self.param2, 1, "Another Variable");
                                        });
                                        ui.horizontal(|ui| {
                                            if self.param2 == 1 {
                                                ui.label("Source Variable ID:");
                                                ui.add(egui::DragValue::new(&mut self.param3).range(1..=5000));
                                            } else {
                                                ui.label("Constant Value:");
                                                ui.add(egui::DragValue::new(&mut self.param3));
                                            }
                                        });
                                    }
                                    2 | 10 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Time (Seconds):");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(0..=99999));
                                            ui.radio_value(&mut self.param2, 0, ">= Remaining");
                                            ui.radio_value(&mut self.param2, 1, "<= Remaining");
                                        });
                                    }
                                    3 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Gold Amount:");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(0..=999999));
                                            ui.radio_value(&mut self.param2, 0, ">= Possessed");
                                            ui.radio_value(&mut self.param2, 1, "<= Possessed");
                                        });
                                    }
                                    4 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Item ID:");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                            ui.radio_value(&mut self.param2, 0, "In Inventory");
                                            ui.radio_value(&mut self.param2, 1, "Not Possessed");
                                        });
                                    }
                                    5 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Hero / Actor ID:");
                                            ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Condition:");
                                            egui::ComboBox::from_id_salt("cmd_cb_hero_type")
                                                .selected_text(match self.param2 {
                                                    0 => "Is in Party",
                                                    1 => "Name Matches",
                                                    2 => "Level >=",
                                                    3 => "HP >=",
                                                    4 => "Knows Skill",
                                                    5 => "Has Item Equipped",
                                                    6 => "Has State / Condition",
                                                    _ => "Is in Party",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param2, 0, "Is in Party");
                                                    ui.selectable_value(&mut self.param2, 1, "Name Matches");
                                                    ui.selectable_value(&mut self.param2, 2, "Level >=");
                                                    ui.selectable_value(&mut self.param2, 3, "HP >=");
                                                    ui.selectable_value(&mut self.param2, 4, "Knows Skill");
                                                    ui.selectable_value(&mut self.param2, 5, "Has Item Equipped");
                                                    ui.selectable_value(&mut self.param2, 6, "Has State / Condition");
                                                });
                                        });
                                        if self.param2 == 1 {
                                            ui.horizontal(|ui| {
                                                ui.label("Expected Name:");
                                                ui.text_edit_singleline(&mut self.string_val);
                                            });
                                        } else if self.param2 >= 2 {
                                            ui.horizontal(|ui| {
                                                let label = match self.param2 {
                                                    2 => "Min Level:",
                                                    3 => "Min HP:",
                                                    4 => "Skill ID:",
                                                    5 => "Equipped Item ID:",
                                                    6 => "State ID:",
                                                    _ => "Value:",
                                                };
                                                ui.label(label);
                                                ui.add(egui::DragValue::new(&mut self.param3).range(1..=99999));
                                            });
                                        }
                                    }
                                    6 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Target Character:");
                                            egui::ComboBox::from_id_salt("cmd_cb_facing_char")
                                                .selected_text(if self.param1 == 10001 { "Player / Party Leader" } else { "Event" })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param1, 10001, "Player / Party Leader");
                                                    ui.selectable_value(&mut self.param1, 10005, "This Event");
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Direction:");
                                            egui::ComboBox::from_id_salt("cmd_cb_facing_dir")
                                                .selected_text(crate::lcf_bridge::event_direction_label(self.param2))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param2, 0, crate::lcf_bridge::event_direction_label(0));
                                                    ui.selectable_value(&mut self.param2, 1, crate::lcf_bridge::event_direction_label(1));
                                                    ui.selectable_value(&mut self.param2, 2, crate::lcf_bridge::event_direction_label(2));
                                                    ui.selectable_value(&mut self.param2, 3, crate::lcf_bridge::event_direction_label(3));
                                                });
                                        });
                                    }
                                    7 => {
                                        ui.horizontal(|ui| {
                                            ui.label("Vehicle:");
                                            egui::ComboBox::from_id_salt("cmd_cb_veh")
                                                .selected_text(match self.param1 {
                                                    0 => "Boat",
                                                    1 => "Ship",
                                                    2 => "Airship",
                                                    _ => "Boat",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.param1, 0, "Boat");
                                                    ui.selectable_value(&mut self.param1, 1, "Ship");
                                                    ui.selectable_value(&mut self.param1, 2, "Airship");
                                                });
                                        });
                                    }
                                    8 => {
                                        ui.label("Branch is taken if the player triggered this event by pressing the Action Key.");
                                    }
                                    9 => {
                                        ui.label("Branch is taken if the current BGM has looped at least once.");
                                    }
                                    _ => {}
                                }
                            }
                            12410 => {
                                ui.label("Comment:");
                                ui.text_edit_singleline(&mut self.string_val);
                            }

                            // ---- Maniac Patch: Tier-1 bespoke forms ----
                            // Fully decoded against EasyRPG Player's actual
                            // interpreter (game_interpreter.cpp /
                            // game_interpreter_battle.cpp, master, checked
                            // 2026-08-21). Edit `raw_params` directly via
                            // `param_mut` rather than the fixed param0..5
                            // fields, since several of these need more than
                            // six slots.
                            3004 => {
                                ui.label("Maniac: End Load Process");
                                ui.colored_label(egui::Color32::GRAY, "No parameters. Resumes execution after a Maniac-triggered Load.");
                            }
                            3005 => {
                                ui.label("Maniac: Get Mouse Position");
                                ui.horizontal(|ui| {
                                    ui.label("Store X in Variable:");
                                    ui.add(egui::DragValue::new(self.param_mut(0)).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Store Y in Variable:");
                                    ui.add(egui::DragValue::new(self.param_mut(1)).range(1..=5000));
                                });
                            }
                            3002 => {
                                ui.label("Maniac: Save");
                                ui.horizontal(|ui| {
                                    ui.label("Save Slot:");
                                    ui.radio_value(self.param_mut(0), 0, "Value");
                                    ui.radio_value(self.param_mut(0), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(1)).range(1..=999));
                                });
                                let mut store_result = *self.param_mut(2) != 0;
                                if ui.checkbox(&mut store_result, "Store result in a variable").changed() {
                                    *self.param_mut(2) = if store_result { 1 } else { 0 };
                                }
                                if store_result {
                                    ui.horizontal(|ui| {
                                        ui.label("Result Variable:");
                                        ui.add(egui::DragValue::new(self.param_mut(3)).range(1..=5000));
                                    });
                                }
                            }
                            3003 => {
                                ui.label("Maniac: Load");
                                ui.horizontal(|ui| {
                                    ui.label("Save Slot:");
                                    ui.radio_value(self.param_mut(0), 0, "Value");
                                    ui.radio_value(self.param_mut(0), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(1)).range(1..=999));
                                });
                                let mut skip_check = *self.param_mut(2) != 0;
                                if ui.checkbox(&mut skip_check, "Skip file-exists check (can crash the game if missing)").changed() {
                                    *self.param_mut(2) = if skip_check { 1 } else { 0 };
                                }
                            }
                            3009 => {
                                ui.label("Maniac: Control Battle (Hooks)");
                                ui.horizontal(|ui| {
                                    ui.label("Hook Type:");
                                    egui::ComboBox::from_id_salt("maniac_control_battle_hook")
                                        .selected_text(maniac_battle_hook_name(*self.param_mut(0)))
                                        .show_ui(ui, |ui| {
                                            for v in 0..=4 {
                                                ui.selectable_value(self.param_mut(0), v, maniac_battle_hook_name(v));
                                            }
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Common Event:");
                                    ui.radio_value(self.param_mut(1), 0, "Value");
                                    ui.radio_value(self.param_mut(1), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(2)).range(0..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Result Variable (base):");
                                    ui.add(egui::DragValue::new(self.param_mut(3)).range(1..=5000));
                                });
                            }
                            3010 => {
                                ui.label("Maniac: Control ATB Gauge");
                                ui.horizontal(|ui| {
                                    ui.label("Target:");
                                    egui::ComboBox::from_id_salt("maniac_atb_target")
                                        .selected_text(maniac_battle_target_name(*self.param_mut(0)))
                                        .show_ui(ui, |ui| {
                                            for v in 0..=4 {
                                                ui.selectable_value(self.param_mut(0), v, maniac_battle_target_name(v));
                                            }
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Target ID:");
                                    ui.radio_value(self.param_mut(1), 0, "Value");
                                    ui.radio_value(self.param_mut(1), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(2)));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(self.param_mut(3), 0, "Set");
                                    ui.radio_value(self.param_mut(3), 1, "Add");
                                    ui.radio_value(self.param_mut(3), 2, "Subtract");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Value Mode:");
                                    ui.radio_value(self.param_mut(4), 0, "Absolute");
                                    ui.radio_value(self.param_mut(4), 1, "Percentage");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Value:");
                                    ui.radio_value(self.param_mut(5), 0, "Value");
                                    ui.radio_value(self.param_mut(5), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(6)));
                                });
                            }
                            3011 => {
                                ui.label("Maniac: Change Battle Command Ex");
                                let mut remove_row = *self.param_mut(0) != 0;
                                if ui.checkbox(&mut remove_row, "Remove one Actor command row").changed() {
                                    *self.param_mut(0) = if remove_row { 1 } else { 0 };
                                }
                                ui.label("Party Command Flags:");
                                let flags_val = *self.param_mut(1);
                                let mut fight_removed = (flags_val & 0x01) != 0;
                                let mut auto_removed = (flags_val & 0x02) != 0;
                                let mut escape_removed = (flags_val & 0x04) != 0;
                                let mut win_added = (flags_val & 0x08) != 0;
                                let mut lose_added = (flags_val & 0x10) != 0;
                                ui.checkbox(&mut fight_removed, "Remove 'Fight'");
                                ui.checkbox(&mut auto_removed, "Remove 'Auto'");
                                ui.checkbox(&mut escape_removed, "Remove 'Escape'");
                                ui.checkbox(&mut win_added, "Add 'Win'");
                                ui.checkbox(&mut lose_added, "Add 'Lose'");
                                let mut new_flags = 0;
                                if fight_removed { new_flags |= 0x01; }
                                if auto_removed { new_flags |= 0x02; }
                                if escape_removed { new_flags |= 0x04; }
                                if win_added { new_flags |= 0x08; }
                                if lose_added { new_flags |= 0x10; }
                                *self.param_mut(1) = new_flags;
                            }
                            3012 => {
                                ui.label("Maniac: Get Battle Info");
                                ui.horizontal(|ui| {
                                    ui.label("Target:");
                                    egui::ComboBox::from_id_salt("maniac_battleinfo_target")
                                        .selected_text(maniac_battle_target_name(*self.param_mut(0)))
                                        .show_ui(ui, |ui| {
                                            for v in 0..=4 {
                                                ui.selectable_value(self.param_mut(0), v, maniac_battle_target_name(v));
                                            }
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Info:");
                                    egui::ComboBox::from_id_salt("maniac_battleinfo_kind")
                                        .selected_text(match *self.param_mut(1) {
                                            0 => "Parameter Buffs",
                                            1 => "States",
                                            2 => "Elements",
                                            3 => "Position / Status",
                                            _ => "Unknown",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(self.param_mut(1), 0, "Parameter Buffs");
                                            ui.selectable_value(self.param_mut(1), 1, "States");
                                            ui.selectable_value(self.param_mut(1), 2, "Elements");
                                            ui.selectable_value(self.param_mut(1), 3, "Position / Status");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Target ID:");
                                    ui.radio_value(self.param_mut(2), 0, "Value");
                                    ui.radio_value(self.param_mut(2), 1, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(3)));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Result Variable (base):");
                                    ui.add(egui::DragValue::new(self.param_mut(4)).range(1..=5000));
                                });
                            }
                            3013 => {
                                ui.label("Maniac: Control Var Array");
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    egui::ComboBox::from_id_salt("maniac_varray_op")
                                        .selected_text(maniac_var_array_op_name(*self.param_mut(0)))
                                        .show_ui(ui, |ui| {
                                            for v in 0..=15 {
                                                ui.selectable_value(self.param_mut(0), v, maniac_var_array_op_name(v));
                                            }
                                        });
                                });
                                let mode_val = *self.param_mut(1);
                                let mut a_is_var = (mode_val & 0x1) != 0;
                                let mut len_is_var = (mode_val & 0x2) != 0;
                                let mut b_is_var = (mode_val & 0x4) != 0;
                                ui.horizontal(|ui| {
                                    ui.label("Target A (start):");
                                    ui.checkbox(&mut a_is_var, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(2)));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Length:");
                                    ui.checkbox(&mut len_is_var, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(3)).range(1..=999));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Target B:");
                                    ui.checkbox(&mut b_is_var, "Variable");
                                    ui.add(egui::DragValue::new(self.param_mut(4)));
                                });
                                let mut new_mode = 0;
                                if a_is_var { new_mode |= 0x1; }
                                if len_is_var { new_mode |= 0x2; }
                                if b_is_var { new_mode |= 0x4; }
                                *self.param_mut(1) = new_mode;
                            }
                            3014 => {
                                ui.label("Maniac: Key Input Proc Ex");
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    egui::ComboBox::from_id_salt("maniac_keyinput_op")
                                        .selected_text(match *self.param_mut(0) {
                                            0 => "Key Range",
                                            1 => "Key Range (with Joypad)",
                                            2 => "Single Key",
                                            _ => "Unknown",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(self.param_mut(0), 0, "Key Range");
                                            ui.selectable_value(self.param_mut(0), 1, "Key Range (with Joypad)");
                                            ui.selectable_value(self.param_mut(0), 2, "Single Key");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Start Variable:");
                                    ui.add(egui::DragValue::new(self.param_mut(1)).range(1..=5000));
                                });
                                if *self.param_mut(0) == 2 {
                                    ui.horizontal(|ui| {
                                        ui.label("Key Code:");
                                        ui.radio_value(self.param_mut(2), 0, "Value");
                                        ui.radio_value(self.param_mut(2), 1, "Variable");
                                        ui.add(egui::DragValue::new(self.param_mut(3)));
                                    });
                                }
                            }
                            3016 => {
                                ui.label("Maniac: Control Global Save");
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    egui::ComboBox::from_id_salt("maniac_globalsave_op")
                                        .selected_text(maniac_global_save_op_name(*self.param_mut(0)))
                                        .show_ui(ui, |ui| {
                                            for v in 0..=5 {
                                                ui.selectable_value(self.param_mut(0), v, maniac_global_save_op_name(v));
                                            }
                                        });
                                });
                                if matches!(*self.param_mut(0), 4 | 5) {
                                    ui.horizontal(|ui| {
                                        ui.label("Data Type:");
                                        ui.radio_value(self.param_mut(2), 0, "Switch");
                                        ui.radio_value(self.param_mut(2), 1, "Variable");
                                    });
                                    let mode_val = *self.param_mut(1);
                                    let mut a_is_var = (mode_val & 0x1) != 0;
                                    let mut b_is_var = (mode_val & 0x2) != 0;
                                    let mut len_is_var = (mode_val & 0x4) != 0;
                                    ui.horizontal(|ui| {
                                        ui.label("Game State Index:");
                                        ui.checkbox(&mut a_is_var, "Variable");
                                        ui.add(egui::DragValue::new(self.param_mut(3)));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Global Save Index:");
                                        ui.checkbox(&mut b_is_var, "Variable");
                                        ui.add(egui::DragValue::new(self.param_mut(4)));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Length:");
                                        ui.checkbox(&mut len_is_var, "Variable");
                                        ui.add(egui::DragValue::new(self.param_mut(5)).range(1..=999));
                                    });
                                    let mut new_mode = 0;
                                    if a_is_var { new_mode |= 0x1; }
                                    if b_is_var { new_mode |= 0x2; }
                                    if len_is_var { new_mode |= 0x4; }
                                    *self.param_mut(1) = new_mode;
                                }
                            }

                            // ---- Maniac Patch: everything else ----
                            // Too complex (deep bitfields, multi-mode
                            // branching) or entirely unimplemented by
                            // EasyRPG Player to build a verified bespoke
                            // form for this pass - see the plan doc. Safe,
                            // lossless generic editor with per-slot hints
                            // wherever `maniac_param_hint` has one.
                            code if lcf_bridge::is_maniac_command_code(code) => {
                                ui.label(format!("Maniac: {} (no dedicated form yet)", lcf_bridge::maniac_command_name(code)));
                                ui.colored_label(egui::Color32::GRAY, "Editing raw parameters. See the Maniac Patch documentation for this command's exact semantics.");
                                self.show_generic_param_list(ui, Some(code));
                            }

                            _ => {
                                ui.horizontal(|ui| {
                                    ui.label("Custom Code:");
                                    ui.add(egui::DragValue::new(&mut self.selected_code));
                                });
                                self.show_generic_param_list(ui, None);
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        // Every arm below now unconditionally assigns
                        // `params` (the old `_` fallback used to leave it
                        // as an empty Vec conditionally - now it always
                        // clones `raw_params`), so no initial value is
                        // needed.
                        let params: Vec<i32>;
                        match self.selected_code {
                            10120 => {
                                params = vec![self.param0, self.param1];
                            }
                            10150 => {
                                params = vec![self.param1, self.param2];
                            }
                            10210 => {
                                params = vec![0, self.param1, self.param1, self.param3];
                            }
                            10220 => {
                                params = vec![self.param0, self.param1, self.param2, self.param3, self.param4, self.param5, self.param6];
                            }
                            10310 => {
                                params = vec![self.param0, 0, self.param2];
                            }
                            10320 => {
                                params = vec![self.param0, 0, self.param1, self.param2];
                            }
                            10330 => {
                                params = vec![self.param0, self.param1];
                            }
                            10410..=10430 => {
                                params = vec![0, self.param1, self.param0, 0, self.param2];
                            }
                            10440 => {
                                params = vec![0, self.param1, self.param0, self.param2];
                            }
                            10460 | 10470 => {
                                params = vec![0, self.param1, self.param0, 0, self.param2];
                            }
                            10480 => {
                                params = vec![0, self.param1, self.param0, self.param2];
                            }
                            10490 => {
                                params = vec![self.param0, self.param1];
                            }
                            10810 => {
                                params = vec![0, self.param1, self.param2, self.param3, 0];
                            }
                            11410 => {
                                params = vec![self.param0];
                            }
                            11510 | 11550 => {
                                params = vec![self.param1, self.param2, 50];
                            }
                            11030 => {
                                params = vec![self.param0, self.param1, self.param2, self.param3];
                            }
                            11070 => {
                                params = vec![self.param0];
                            }
                            11110 | 11120 => {
                                params = vec![self.param0, self.param1, self.param2];
                            }
                            11130 => {
                                params = vec![self.param0];
                            }
                            12010 => {
                                params = vec![self.param0, self.param1, self.param2, self.param3, self.param4];
                            }
                            12330 => {
                                params = vec![self.param0];
                            }
                            10710 => {
                                params = vec![0, self.param1];
                            }
                            // Every Maniac command (Tier-1 bespoke or the
                            // generic Tier-2 editor) and any other
                            // unrecognized code edits `raw_params` directly,
                            // so it's always the full, lossless parameter
                            // vector - no per-command pack arm needed here.
                            _ => {
                                params = self.raw_params.clone();
                            }
                        }

                        let cmd = EventCommandInfo {
                            code: self.selected_code,
                            indent: self.indent,
                            string: self.string_val.clone(),
                            parameters: params,
                        };
                        result = Some((self.edit_index, cmd));
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

    /// Safe, lossless fallback editor: one row per `raw_params` entry (with
    /// an optional per-slot hint via `maniac_param_hint`), +/- buttons to
    /// grow/shrink the vector, and the full string field. Used for every
    /// Maniac command without a bespoke Tier-1 form above, and for any
    /// other unrecognized command code - replaces the old "Custom Code"
    /// fallback that silently truncated/discarded parameters on save.
    fn show_generic_param_list(&mut self, ui: &mut egui::Ui, hint_code: Option<i32>) {
        ui.label(format!("{} parameter(s):", self.raw_params.len()));
        let mut remove_idx = None;
        for i in 0..self.raw_params.len() {
            ui.horizontal(|ui| {
                let hint = hint_code.and_then(|c| lcf_bridge::maniac_param_hint(c, i));
                ui.label(format!("[{}]{}", i, hint.map(|h| format!(" {h}")).unwrap_or_default()));
                ui.add(egui::DragValue::new(&mut self.raw_params[i]));
                if ui.small_button("✕").clicked() {
                    remove_idx = Some(i);
                }
            });
        }
        if let Some(i) = remove_idx {
            self.raw_params.remove(i);
        }
        if ui.button("+ Add Parameter").clicked() {
            self.raw_params.push(0);
        }
        ui.separator();
        ui.label("String argument:");
        ui.text_edit_multiline(&mut self.string_val);
    }
}

/// Name for a `Maniac_ControlBattle` hook type (`ManiacBattleHookType` in
/// `game_interpreter_battle.h`, EasyRPG/Player).
fn maniac_battle_hook_name(v: i32) -> &'static str {
    match v {
        0 => "ATB Increment",
        1 => "Damage Pop",
        2 => "Targeting",
        3 => "Set State",
        4 => "Stat Change",
        _ => "Unknown",
    }
}

/// Battler target selector shared by `Maniac_ControlAtbGauge` and
/// `Maniac_GetBattleInfo` (`target_flags` in `game_interpreter_battle.cpp`).
fn maniac_battle_target_name(v: i32) -> &'static str {
    match v {
        0 => "Actor",
        1 => "Party Member",
        2 => "Entire Party",
        3 => "Troop Member",
        4 => "Entire Troop",
        _ => "Unknown",
    }
}

/// Operation for `Maniac_ControlVarArray` (`op` in `gi.cpp`'s
/// `CommandManiacControlVarArray`, 16 values).
fn maniac_var_array_op_name(v: i32) -> &'static str {
    match v {
        0 => "Copy",
        1 => "Swap",
        2 => "Sort Ascending",
        3 => "Sort Descending",
        4 => "Shuffle",
        5 => "Enumerate",
        6 => "Add",
        7 => "Subtract",
        8 => "Multiply",
        9 => "Divide",
        10 => "Modulo",
        11 => "Bitwise OR",
        12 => "Bitwise AND",
        13 => "Bitwise XOR",
        14 => "Shift Left",
        15 => "Shift Right",
        _ => "Unknown",
    }
}

/// Operation for `Maniac_ControlGlobalSave` (`operation` in `gi.cpp`'s
/// `CommandManiacControlGlobalSave`).
fn maniac_global_save_op_name(v: i32) -> &'static str {
    match v {
        0 => "Open",
        1 => "Close",
        2 => "Save",
        3 => "Save and Close",
        4 => "Copy: Global Save -> Game State",
        5 => "Copy: Game State -> Global Save",
        _ => "Unknown",
    }
}

