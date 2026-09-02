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
    pub choices: [String; 4],
    pub shop_items: Vec<i32>,
    pub new_shop_item_id: i32,
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
            choices: [String::new(), String::new(), String::new(), String::new()],
            shop_items: Vec::new(),
            new_shop_item_id: 1,
            raw_params: Vec::new(),
        }
    }
}

impl EventCommandDialogState {
    /// Mutable access to `raw_params[i]`, growing the vector with zeros as
    /// needed. Used by every Maniac editor arm (bespoke or generic) instead
    /// of the fixed `param0..5` fields, since several Maniac commands need
    /// more than six parameters (e.g. `ShowStringPicture`'s 23).
    pub fn param_mut(&mut self, i: usize) -> &mut i32 {
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
        self.choices = [String::new(), String::new(), String::new(), String::new()];
        self.shop_items = Vec::new();
        self.new_shop_item_id = 1;
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
        self.new_shop_item_id = 1;

        if cmd.code == 10140 {
            let parts: Vec<&str> = cmd.string.split('/').collect();
            self.choices = [
                parts.first().copied().unwrap_or("").to_string(),
                parts.get(1).copied().unwrap_or("").to_string(),
                parts.get(2).copied().unwrap_or("").to_string(),
                parts.get(3).copied().unwrap_or("").to_string(),
            ];
        } else {
            self.choices = [String::new(), String::new(), String::new(), String::new()];
        }

        if cmd.code == 10720 {
            self.shop_items = if cmd.parameters.len() > 2 {
                cmd.parameters[2..].to_vec()
            } else {
                Vec::new()
            };
        } else {
            self.shop_items = Vec::new();
        }

        if cmd.code == 11110 || cmd.code == 11120 {
            if cmd.parameters.len() >= 5 {
                self.param0 = cmd.parameters[0];
                self.param1 = cmd.parameters.get(2).copied().unwrap_or(0);
                self.param2 = cmd.parameters.get(3).copied().unwrap_or(0);
                self.param3 = cmd.parameters.get(6).copied().unwrap_or(0);
                self.param4 = cmd.parameters.get(4).copied().unwrap_or(100);
                self.param5 = cmd.parameters.get(5).copied().unwrap_or(0);
            } else {
                self.param0 = cmd.parameters.first().copied().unwrap_or(1);
                self.param1 = cmd.parameters.get(1).copied().unwrap_or(0);
                self.param2 = cmd.parameters.get(2).copied().unwrap_or(0);
                self.param4 = 100;
            }
        }

        // Always the full vector, never truncated - see `raw_params` doc.
        self.raw_params = cmd.parameters.clone();

        // Auto-detect category from code
        self.category = match cmd.code {
            10110..=10150 | 20110 => CommandCategory::Messages,
            10210..=10330 | 11610 => CommandCategory::Progression,
            10410..=10650 => CommandCategory::Character,
            10810..=10870 | 11310..=11410 => CommandCategory::Movement,
            11010..=11210 | 11510..=11560 | 11710..=11720 => CommandCategory::AudioVisual,
            12010..=12410 | 20140..=20141 | 22010..=22410 | 23310..=23311 => CommandCategory::FlowControl,
            10710..=10740 | 11810..=11960 | 12420 | 12510 | 13110..=13410 | 20710..=20732 => CommandCategory::SystemScenes,
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
                            10610 => "10610: Change Hero Name",
                            10620 => "10620: Change Hero Title",
                            10630 => "10630: Change Hero Graphic",
                            10640 => "10640: Change Hero Face Graphic",
                            10650 => "10650: Change Vehicle Graphic",
                            10810 => "10810: Transfer Player (Teleport)",
                            10820 => "10820: Memorize Location",
                            10830 => "10830: Recall to Location",
                            10840 => "10840: Enter/Exit Vehicle",
                            10850 => "10850: Set Vehicle Location",
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
                            11140 => "11140: Show Battle Animation",
                            11210 => "11210: Show Battle Animation",
                            11610 => "11610: Key Input Processing",
                            11710 => "11710: Change Map Chipset",
                            11720 => "11720: Change Parallax Background",
                            11810 => "11810: Set Teleport Target",
                            11820 => "11820: Change Teleport Access",
                            11830 => "11830: Set Escape Target",
                            11840 => "11840: Change Escape Access",
                            11910 => "11910: Open Save Menu",
                            11930 => "11930: Change Save Access",
                            11950 => "11950: Open Main Menu",
                            11960 => "11960: Change Main Menu Access",
                            12010 => "12010: Conditional Branch",
                            12110 => "12110: Label",
                            12120 => "12120: Jump to Label",
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
                                    ui.selectable_value(&mut self.selected_code, 11610, "Key Input Processing");
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
                                    ui.selectable_value(&mut self.selected_code, 10610, "Change Hero Name");
                                    ui.selectable_value(&mut self.selected_code, 10620, "Change Hero Title");
                                    ui.selectable_value(&mut self.selected_code, 10630, "Change Hero Graphic");
                                    ui.selectable_value(&mut self.selected_code, 10640, "Change Hero Face Graphic");
                                }
                                CommandCategory::Movement => {
                                    ui.selectable_value(&mut self.selected_code, 10810, "Transfer Player (Teleport)");
                                    ui.selectable_value(&mut self.selected_code, 10820, "Memorize Location");
                                    ui.selectable_value(&mut self.selected_code, 10830, "Recall to Location");
                                    ui.selectable_value(&mut self.selected_code, 10840, "Enter/Exit Vehicle");
                                    ui.selectable_value(&mut self.selected_code, 10850, "Set Vehicle Location");
                                    ui.selectable_value(&mut self.selected_code, 10860, "Set Event Location");
                                    ui.selectable_value(&mut self.selected_code, 10650, "Change Vehicle Graphic");
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
                                    ui.selectable_value(&mut self.selected_code, 11710, "Change Map Chipset");
                                    ui.selectable_value(&mut self.selected_code, 11720, "Change Parallax Background");
                                }
                                CommandCategory::FlowControl => {
                                    ui.selectable_value(&mut self.selected_code, 12010, "Conditional Branch");
                                    ui.selectable_value(&mut self.selected_code, 12110, "Label");
                                    ui.selectable_value(&mut self.selected_code, 12120, "Jump to Label");
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
                                    ui.selectable_value(&mut self.selected_code, 11820, "Change Teleport Access");
                                    ui.selectable_value(&mut self.selected_code, 11840, "Change Escape Access");
                                    ui.selectable_value(&mut self.selected_code, 11910, "Open Save Menu");
                                    ui.selectable_value(&mut self.selected_code, 11930, "Change Save Access");
                                    ui.selectable_value(&mut self.selected_code, 11950, "Open Main Menu");
                                    ui.selectable_value(&mut self.selected_code, 11960, "Change Main Menu Access");
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
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Insert Code:");
                                    if ui.small_button("\\c[n] Color").on_hover_text("Insert Color code \\c[0..7]").clicked() {
                                        self.string_val.push_str("\\c[1]");
                                    }
                                    if ui.small_button("\\v[n] Var").on_hover_text("Insert Variable code \\v[1]").clicked() {
                                        self.string_val.push_str("\\v[1]");
                                    }
                                    if ui.small_button("\\n[n] Name").on_hover_text("Insert Hero Name \\n[1]").clicked() {
                                        self.string_val.push_str("\\n[1]");
                                    }
                                    if ui.small_button("\\. Wait 0.25s").on_hover_text("Pause 0.25 seconds").clicked() {
                                        self.string_val.push_str("\\.");
                                    }
                                    if ui.small_button("\\| Wait 1s").on_hover_text("Pause 1.0 second").clicked() {
                                        self.string_val.push_str("\\|");
                                    }
                                    if ui.small_button("\\! Keypress").on_hover_text("Wait for player button press").clicked() {
                                        self.string_val.push_str("\\!");
                                    }
                                    if ui.small_button("\\$ Gold").on_hover_text("Display Current Gold Window").clicked() {
                                        self.string_val.push_str("\\$");
                                    }
                                    if ui.small_button("\\> Fast").on_hover_text("Show remaining message characters instantly").clicked() {
                                        self.string_val.push_str("\\>");
                                    }
                                    if ui.small_button("\\< Normal").on_hover_text("Resume normal typewriter speed").clicked() {
                                        self.string_val.push_str("\\<");
                                    }
                                });
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
                                ui.heading("Choice Options (Up to 4)");
                                egui::Grid::new("choice_options_grid")
                                    .num_columns(2)
                                    .spacing([12.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label("Choice 1:");
                                        ui.text_edit_singleline(&mut self.choices[0]);
                                        ui.end_row();

                                        ui.label("Choice 2:");
                                        ui.text_edit_singleline(&mut self.choices[1]);
                                        ui.end_row();

                                        ui.label("Choice 3:");
                                        ui.text_edit_singleline(&mut self.choices[2]);
                                        ui.end_row();

                                        ui.label("Choice 4:");
                                        ui.text_edit_singleline(&mut self.choices[3]);
                                        ui.end_row();
                                    });

                                ui.separator();
                                ui.label("Cancel Action:");
                                ui.horizontal_wrapped(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Disallow Cancel");
                                    ui.radio_value(&mut self.param0, 1, "Choice 1");
                                    ui.radio_value(&mut self.param0, 2, "Choice 2");
                                    ui.radio_value(&mut self.param0, 3, "Choice 3");
                                    ui.radio_value(&mut self.param0, 4, "Choice 4");
                                    ui.radio_value(&mut self.param0, 5, "Cancel Branch");
                                });
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
                            10450 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Slot:");
                                    egui::ComboBox::from_id_salt("cmd_equip_slot")
                                        .selected_text(match self.param4 {
                                            0 => "Weapon", 1 => "Shield", 2 => "Armor", 3 => "Helmet", 4 => "Accessory",
                                            _ => "Weapon",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param4, 0, "Weapon");
                                            ui.selectable_value(&mut self.param4, 1, "Shield");
                                            ui.selectable_value(&mut self.param4, 2, "Armor");
                                            ui.selectable_value(&mut self.param4, 3, "Helmet");
                                            ui.selectable_value(&mut self.param4, 4, "Accessory");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param0, 0, "Equip Item");
                                    ui.radio_value(&mut self.param0, 1, "Unequip / Remove");
                                });
                                if self.param0 == 0 {
                                    ui.horizontal(|ui| {
                                        ui.label("Item ID:");
                                        ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                    });
                                }
                            }
                            10610 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("New Name:");
                                    ui.text_edit_singleline(&mut self.string_val);
                                });
                            }
                            10620 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("New Title:");
                                    ui.text_edit_singleline(&mut self.string_val);
                                });
                            }
                            10630 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("CharSet Graphic:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_charset_combo", &mut self.string_val, "CharSet", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Graphic Index:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(0..=7));
                                    let mut trans = self.param3 != 0;
                                    if ui.checkbox(&mut trans, "Translucent").changed() {
                                        self.param3 = if trans { 1 } else { 0 };
                                    }
                                });
                            }
                            10640 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("FaceSet Graphic:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_faceset_combo", &mut self.string_val, "FaceSet", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Face Index:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(0..=15));
                                });
                            }
                            10650 => {
                                ui.horizontal(|ui| {
                                    ui.label("Vehicle:");
                                    egui::ComboBox::from_id_salt("cmd_veh_combo")
                                        .selected_text(match self.param1 {
                                            0 => "Skiff / Small Boat", 1 => "Ship", 2 => "Airship", _ => "Vehicle",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param1, 0, "Skiff / Small Boat");
                                            ui.selectable_value(&mut self.param1, 1, "Ship");
                                            ui.selectable_value(&mut self.param1, 2, "Airship");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("CharSet Graphic:");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_veh_charset_combo", &mut self.string_val, "CharSet", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Graphic Index:");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(0..=7));
                                });
                            }
                            10820 => {
                                ui.label("Memorize Current Location to Variables:");
                                ui.horizontal(|ui| {
                                    ui.label("Map ID Var:"); ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                    ui.label("X Var:"); ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                    ui.label("Y Var:"); ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                });
                            }
                            10830 => {
                                ui.label("Recall / Teleport from Variables:");
                                ui.horizontal(|ui| {
                                    ui.label("Map ID Var:"); ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                    ui.label("X Var:"); ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                    ui.label("Y Var:"); ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                });
                            }
                            10840 => {
                                ui.label("Board / Dismount Vehicle at current location");
                            }
                            10850 => {
                                ui.horizontal(|ui| {
                                    ui.label("Vehicle:");
                                    egui::ComboBox::from_id_salt("cmd_set_veh_combo")
                                        .selected_text(match self.param0 {
                                            0 => "Skiff / Small Boat", 1 => "Ship", 2 => "Airship", _ => "Vehicle",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param0, 0, "Skiff / Small Boat");
                                            ui.selectable_value(&mut self.param0, 1, "Ship");
                                            ui.selectable_value(&mut self.param0, 2, "Airship");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Target Map ID:"); ui.add(egui::DragValue::new(&mut self.param1).range(1..=9999));
                                    ui.label("X:"); ui.add(egui::DragValue::new(&mut self.param2).range(0..=500));
                                    ui.label("Y:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=500));
                                });
                            }
                            10860 => {
                                ui.horizontal(|ui| {
                                    ui.label("Target Event:");
                                    egui::ComboBox::from_id_salt("cmd_ev_loc_target")
                                        .selected_text(match self.param0 {
                                            10001 => "Player / Party Leader",
                                            10005 => "This Event",
                                            _ => "Specific Event ID",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param0, 10001, "Player / Party Leader");
                                            ui.selectable_value(&mut self.param0, 10005, "This Event");
                                            ui.selectable_value(&mut self.param0, 1, "Specific Event ID");
                                        });
                                    if self.param0 != 10001 && self.param0 != 10005 {
                                        ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000).prefix("#"));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Position Mode:");
                                    ui.radio_value(&mut self.param1, 0, "Coordinates");
                                    ui.radio_value(&mut self.param1, 1, "From Variables");
                                    ui.radio_value(&mut self.param1, 2, "Swap with Event");
                                });
                                ui.horizontal(|ui| {
                                    if self.param1 == 2 {
                                        ui.label("Swap with Event ID:");
                                        ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                    } else if self.param1 == 1 {
                                        ui.label("X Var ID:"); ui.add(egui::DragValue::new(&mut self.param2).range(1..=5000));
                                        ui.label("Y Var ID:"); ui.add(egui::DragValue::new(&mut self.param3).range(1..=5000));
                                    } else {
                                        ui.label("X Coord:"); ui.add(egui::DragValue::new(&mut self.param2).range(0..=500));
                                        ui.label("Y Coord:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=500));
                                    }
                                });
                            }
                            11010 => {
                                ui.horizontal(|ui| {
                                    ui.label("Screen Transition Effect:");
                                    egui::ComboBox::from_id_salt("cmd_trans_combo")
                                        .selected_text(match self.param0 {
                                            0 => "Fade Out / In", 1 => "Random Blocks", 2 => "Wipe Down",
                                            3 => "Wipe Up", 4 => "Curtain Open/Close", 5 => "Horizontal Stripes",
                                            _ => "Default Transition",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param0, 0, "Fade Out / In");
                                            ui.selectable_value(&mut self.param0, 1, "Random Blocks");
                                            ui.selectable_value(&mut self.param0, 2, "Wipe Down");
                                            ui.selectable_value(&mut self.param0, 3, "Wipe Up");
                                            ui.selectable_value(&mut self.param0, 4, "Curtain Open/Close");
                                            ui.selectable_value(&mut self.param0, 5, "Horizontal Stripes");
                                        });
                                });
                            }
                            11040 => {
                                ui.heading("Flash Screen FX");
                                ui.horizontal(|ui| {
                                    let r = (self.param0.clamp(0, 31) * 255 / 31) as u8;
                                    let g = (self.param1.clamp(0, 31) * 255 / 31) as u8;
                                    let b = (self.param2.clamp(0, 31) * 255 / 31) as u8;
                                    let alpha = (self.param3.clamp(0, 31) * 255 / 31) as u8;
                                    let (swatch_rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 32.0), egui::Sense::hover());
                                    ui.painter().rect_filled(swatch_rect, 4.0, egui::Color32::from_rgba_unmultiplied(r, g, b, alpha.max(40)));
                                    ui.painter().rect_stroke(swatch_rect, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::StrokeKind::Outside);

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Red:"); ui.add(egui::DragValue::new(&mut self.param0).range(0..=31));
                                            ui.label("Green:"); ui.add(egui::DragValue::new(&mut self.param1).range(0..=31));
                                            ui.label("Blue:"); ui.add(egui::DragValue::new(&mut self.param2).range(0..=31));
                                            ui.label("Power:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=31));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Duration (tenths):");
                                            ui.add(egui::DragValue::new(&mut self.param4).range(1..=200));
                                            ui.label(format!("({:.1}s)", self.param4 as f32 / 10.0));
                                            let mut wait = self.param5 != 0;
                                            if ui.checkbox(&mut wait, "Wait for Completion").changed() {
                                                self.param5 = if wait { 1 } else { 0 };
                                            }
                                        });
                                    });
                                });
                            }
                            11050 => {
                                ui.horizontal(|ui| {
                                    ui.label("Strength (1..9):"); ui.add(egui::DragValue::new(&mut self.param0).range(1..=9));
                                    ui.label("Speed (1..9):"); ui.add(egui::DragValue::new(&mut self.param1).range(1..=9));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Duration (tenths of sec):");
                                    ui.add(egui::DragValue::new(&mut self.param2).range(1..=200));
                                    let mut wait = self.param3 != 0;
                                    if ui.checkbox(&mut wait, "Wait for Completion").changed() {
                                        self.param3 = if wait { 1 } else { 0 };
                                    }
                                });
                            }
                            11210 => {
                                ui.horizontal(|ui| {
                                    ui.label("Battle Animation ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Target:");
                                    egui::ComboBox::from_id_salt("cmd_anim_target")
                                        .selected_text(match self.param1 {
                                            10001 => "Player / Party Leader",
                                            10005 => "This Event",
                                            _ => "Specific Event ID",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param1, 10001, "Player / Party Leader");
                                            ui.selectable_value(&mut self.param1, 10005, "This Event");
                                            ui.selectable_value(&mut self.param1, 1, "Specific Event ID");
                                        });
                                    if self.param1 != 10001 && self.param1 != 10005 {
                                        ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000).prefix("#"));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    let mut wait = self.param2 != 0;
                                    if ui.checkbox(&mut wait, "Wait for Completion").changed() {
                                        self.param2 = if wait { 1 } else { 0 };
                                    }
                                });
                            }
                            11520 => {
                                ui.horizontal(|ui| {
                                    ui.label("Fade Out Duration (Seconds):");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=20));
                                });
                            }
                            11610 => {
                                ui.horizontal(|ui| {
                                    ui.label("Store Pressed Key in Variable ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    let mut wait = self.param1 != 0;
                                    if ui.checkbox(&mut wait, "Wait for Key Press").changed() {
                                        self.param1 = if wait { 1 } else { 0 };
                                    }
                                });
                            }
                            11710 => {
                                ui.horizontal(|ui| {
                                    ui.label("Switch Map Chipset to ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                });
                            }
                            11720 => {
                                ui.horizontal(|ui| {
                                    ui.label("Parallax Background (Panorama):");
                                    let mut dummy_dirty = false;
                                    crate::widgets::resource_dropdown::resource_combo_box(ui, "cmd_panorama_combo", &mut self.string_val, "Panorama", None, &mut dummy_dirty, None);
                                });
                                ui.horizontal(|ui| {
                                    let mut lx = self.param0 != 0;
                                    if ui.checkbox(&mut lx, "Loop Horizontally (X)").changed() { self.param0 = if lx { 1 } else { 0 }; }
                                    let mut ly = self.param1 != 0;
                                    if ui.checkbox(&mut ly, "Loop Vertically (Y)").changed() { self.param1 = if ly { 1 } else { 0 }; }
                                });
                            }
                            10720 => {
                                ui.horizontal(|ui| {
                                    ui.label("Shop Type:");
                                    ui.radio_value(&mut self.param0, 0, "Standard (Buy & Sell)");
                                    ui.radio_value(&mut self.param0, 1, "Buy Only");
                                    ui.radio_value(&mut self.param0, 2, "Sell Only");
                                });

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.heading("Sold Items (Goods List)");
                                    ui.label("Item ID:");
                                    ui.add(egui::DragValue::new(&mut self.new_shop_item_id).range(1..=5000));
                                    if ui.button("➕ Add Item").clicked() {
                                        self.shop_items.push(self.new_shop_item_id);
                                    }
                                });

                                if self.shop_items.is_empty() {
                                    ui.label("(No specific items added - shop inventory empty or default)");
                                } else {
                                    let mut remove_idx = None;
                                    egui::ScrollArea::vertical()
                                        .id_salt("shop_goods_scroll")
                                        .max_height(100.0)
                                        .show(ui, |ui| {
                                            for (idx, &item_id) in self.shop_items.iter().enumerate() {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("#{}: Item ID {}", idx + 1, item_id));
                                                    if ui.small_button("🗑").clicked() {
                                                        remove_idx = Some(idx);
                                                    }
                                                });
                                            }
                                        });
                                    if let Some(idx) = remove_idx {
                                        self.shop_items.remove(idx);
                                    }
                                }
                            }
                            10730 => {
                                ui.horizontal(|ui| {
                                    ui.label("Inn Type:");
                                    ui.radio_value(&mut self.param0, 0, "Standard Inn");
                                    ui.radio_value(&mut self.param0, 1, "Custom Messages");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Cost (Gold):");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(0..=99999));
                                });
                            }
                            10740 => {
                                ui.horizontal(|ui| {
                                    ui.label("Hero / Actor ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                    let mut allow_def = self.param1 != 0;
                                    if ui.checkbox(&mut allow_def, "Allow Default Name").changed() {
                                        self.param1 = if allow_def { 1 } else { 0 };
                                    }
                                });
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
                            11330 => {
                                ui.horizontal(|ui| {
                                    ui.label("Target Character:");
                                    egui::ComboBox::from_id_salt("cmd_mr_target")
                                        .selected_text(match self.param0 {
                                            10001 => "Player / Party Leader",
                                            10005 => "This Event",
                                            _ => "Specific Event ID",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.param0, 10001, "Player / Party Leader");
                                            ui.selectable_value(&mut self.param0, 10005, "This Event");
                                            ui.selectable_value(&mut self.param0, 1, "Specific Event ID");
                                        });
                                    if self.param0 != 10001 && self.param0 != 10005 {
                                        ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000).prefix("#"));
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Move Frequency:");
                                    ui.add(egui::Slider::new(&mut self.param1, 1..=8));
                                });

                                ui.horizontal(|ui| {
                                    let mut rep = self.param2 != 0;
                                    if ui.checkbox(&mut rep, "🔁 Repeat Route").changed() {
                                        self.param2 = if rep { 1 } else { 0 };
                                    }
                                    let mut skip = self.param3 != 0;
                                    if ui.checkbox(&mut skip, "⏩ Skip if Blocked").changed() {
                                        self.param3 = if skip { 1 } else { 0 };
                                    }
                                });

                                ui.separator();
                                ui.label(format!("Movement Steps ({})", self.raw_params.len().saturating_sub(5)));

                                // Quick Step Palette
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button("⬆ Up").clicked() { self.raw_params.push(0); }
                                    if ui.button("⬇ Down").clicked() { self.raw_params.push(2); }
                                    if ui.button("⬅ Left").clicked() { self.raw_params.push(3); }
                                    if ui.button("➡ Right").clicked() { self.raw_params.push(1); }
                                    if ui.button("⏳ Wait").clicked() { self.raw_params.push(23); }
                                    if ui.button("🚶 Forward").clicked() { self.raw_params.push(11); }
                                    if ui.button("🎯 Toward Hero").clicked() { self.raw_params.push(9); }
                                    if ui.button("🏃 Away Hero").clicked() { self.raw_params.push(10); }
                                    if ui.button("🧹 Clear").clicked() { self.raw_params.truncate(5); }
                                });

                                egui::ScrollArea::vertical()
                                    .id_salt("cmd_mr_steps_scroll")
                                    .max_height(140.0)
                                    .show(ui, |ui| {
                                        let mut remove_step = None;
                                        let start_idx = 5.min(self.raw_params.len());
                                        for i in start_idx..self.raw_params.len() {
                                            ui.horizontal(|ui| {
                                                let cmd_id = self.raw_params[i];
                                                let dummy_cmd = lcf_core::MoveCommand { code: cmd_id, ..Default::default() };
                                                ui.label(format!("#{:02}: {}", i - start_idx + 1, crate::dialogs::move_route_dialog::move_command_label(&dummy_cmd)));
                                                if ui.small_button("✕").clicked() {
                                                    remove_step = Some(i);
                                                }
                                            });
                                        }
                                        if let Some(i) = remove_step {
                                            self.raw_params.remove(i);
                                        }
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
                                ui.heading("Screen Color Tint");
                                ui.horizontal(|ui| {
                                    let r = (((self.param0.clamp(-31, 31) + 31) as f32 / 62.0) * 255.0) as u8;
                                    let g = (((self.param1.clamp(-31, 31) + 31) as f32 / 62.0) * 255.0) as u8;
                                    let b = (((self.param2.clamp(-31, 31) + 31) as f32 / 62.0) * 255.0) as u8;
                                    let (swatch_rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 32.0), egui::Sense::hover());
                                    ui.painter().rect_filled(swatch_rect, 4.0, egui::Color32::from_rgb(r, g, b));
                                    ui.painter().rect_stroke(swatch_rect, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::StrokeKind::Outside);

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Red:"); ui.add(egui::DragValue::new(&mut self.param0).range(-31..=31));
                                            ui.label("Green:"); ui.add(egui::DragValue::new(&mut self.param1).range(-31..=31));
                                            ui.label("Blue:"); ui.add(egui::DragValue::new(&mut self.param2).range(-31..=31));
                                            ui.label("Chroma:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=31));
                                        });
                                    });
                                });
                            }
                            11070 => {
                                ui.heading("Weather Effects");
                                ui.horizontal(|ui| {
                                    ui.label("Effect Type:");
                                    ui.radio_value(&mut self.param0, 0, "None / Clear");
                                    ui.radio_value(&mut self.param0, 1, "🌧 Rain");
                                    ui.radio_value(&mut self.param0, 2, "❄ Snow");
                                    ui.radio_value(&mut self.param0, 3, "🌪 Sandstorm");
                                });
                                if self.param0 != 0 {
                                    ui.horizontal(|ui| {
                                        ui.label("Severity:");
                                        ui.radio_value(&mut self.param1, 0, "Low");
                                        ui.radio_value(&mut self.param1, 1, "Medium");
                                        ui.radio_value(&mut self.param1, 2, "High");
                                    });
                                }
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
                                    let mut is_fixed = self.param3 != 0;
                                    if ui.checkbox(&mut is_fixed, "Pin to Map (Scrolls with Map)").changed() {
                                        self.param3 = if is_fixed { 1 } else { 0 };
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Magnification %:");
                                    if self.param4 == 0 { self.param4 = 100; }
                                    ui.add(egui::DragValue::new(&mut self.param4).range(10..=400));
                                    ui.label("Transparency %:");
                                    ui.add(egui::DragValue::new(&mut self.param5).range(0..=100));
                                });
                                ui.group(|ui| {
                                    ui.label("Color Tint (-31..31):");
                                    ui.horizontal(|ui| {
                                        let r = self.param_mut(7);
                                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "R:");
                                        ui.add(egui::DragValue::new(r).range(-31..=31));
                                        let g = self.param_mut(8);
                                        ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "G:");
                                        ui.add(egui::DragValue::new(g).range(-31..=31));
                                        let b = self.param_mut(9);
                                        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "B:");
                                        ui.add(egui::DragValue::new(b).range(-31..=31));
                                        let sat = self.param_mut(10);
                                        ui.label("Chroma:");
                                        ui.add(egui::DragValue::new(sat).range(0..=31));
                                    });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Effect:");
                                    let effect_mode = self.param_mut(11);
                                    ui.radio_value(effect_mode, 0, "None");
                                    ui.radio_value(effect_mode, 1, "Rotate");
                                    ui.radio_value(effect_mode, 2, "Wave");
                                    if *effect_mode != 0 {
                                        ui.label("Speed:");
                                        let speed = self.param_mut(12);
                                        ui.add(egui::DragValue::new(speed).range(1..=10));
                                    }
                                });
                                if self.selected_code == 11120 {
                                    ui.horizontal(|ui| {
                                        ui.label("Move Duration (0.1s):");
                                        let dur = self.param_mut(13);
                                        if *dur == 0 { *dur = 10; }
                                        ui.add(egui::DragValue::new(dur).range(1..=1000));
                                        let wait = self.param_mut(14);
                                        let mut wait_bool = *wait != 0;
                                        if ui.checkbox(&mut wait_bool, "Wait for Completion").changed() {
                                            *wait = if wait_bool { 1 } else { 0 };
                                        }
                                    });
                                }
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
                            12210 => {
                                ui.label("Loop");
                                ui.colored_label(egui::Color32::GRAY, "Begins a loop structure. Commands between Loop and Repeat Above will continuously repeat until a Break Loop command is encountered.");
                            }
                            12220 => {
                                ui.label("Break Loop");
                                ui.colored_label(egui::Color32::GRAY, "Breaks out of the innermost enclosing loop and jumps to the command following Repeat Above.");
                            }
                            12310 => {
                                ui.label("Exit Event Processing");
                                ui.colored_label(egui::Color32::GRAY, "Immediately terminates execution of the current event page.");
                            }
                            12320 => {
                                ui.label("Erase Event");
                                ui.colored_label(egui::Color32::GRAY, "Temporarily removes this event from the current map until the player exits and re-enters the map.");
                            }
                            12420 => {
                                ui.label("Game Over");
                                ui.colored_label(egui::Color32::GRAY, "Immediately halts gameplay and transitions to the Game Over screen.");
                            }
                            12510 => {
                                ui.label("Return to Title Screen");
                                ui.colored_label(egui::Color32::GRAY, "Immediately ends the current game session and returns to the Title Screen.");
                            }
                            11910 => {
                                ui.label("Open Save Menu");
                                ui.colored_label(egui::Color32::GRAY, "Opens the standard in-game Save Menu, allowing the player to save their progress.");
                            }
                            11930 => {
                                ui.label("Change Save Access:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Enable (Allow)");
                                    ui.radio_value(&mut self.param0, 1, "Disable (Forbid)");
                                });
                            }
                            11950 => {
                                ui.label("Open Main Menu");
                                ui.colored_label(egui::Color32::GRAY, "Opens the standard in-game Main Menu (Items, Skills, Equipment, Status).");
                            }
                            11960 => {
                                ui.label("Change Main Menu Access:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Enable (Allow)");
                                    ui.radio_value(&mut self.param0, 1, "Disable (Forbid)");
                                });
                            }
                            11820 => {
                                ui.label("Change Teleport Access:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Enable (Allow)");
                                    ui.radio_value(&mut self.param0, 1, "Disable (Forbid)");
                                });
                            }
                            11840 => {
                                ui.label("Change Escape Access:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.param0, 0, "Enable (Allow)");
                                    ui.radio_value(&mut self.param0, 1, "Disable (Forbid)");
                                });
                            }
                            12110 => {
                                ui.label("Label Number:");
                                ui.add(egui::DragValue::new(&mut self.param0).range(1..=1000));
                            }
                            12120 => {
                                ui.label("Jump to Label Number:");
                                ui.add(egui::DragValue::new(&mut self.param0).range(1..=1000));
                            }
                            12410 => {
                                ui.label("Comment:");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            20110 => {
                                ui.label("Show Message (Continuation line):");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            22410 => {
                                ui.label("Comment (Continuation line):");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            20710 => {
                                ui.label("When [Victory]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the party wins the battle.");
                            }
                            20711 => {
                                ui.label("When [Escape]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the party successfully flees the battle.");
                            }
                            20712 => {
                                ui.label("When [Defeat]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the party is defeated in battle.");
                            }
                            20713 => {
                                ui.label("End Battle Processing");
                                ui.colored_label(egui::Color32::GRAY, "Marks the conclusion of battle branch handling.");
                            }
                            20720 => {
                                ui.label("When [Transaction]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the player completes a transaction in the shop.");
                            }
                            20721 => {
                                ui.label("When [Cancel]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the player exits the shop without buying.");
                            }
                            20722 => {
                                ui.label("End Shop Processing");
                                ui.colored_label(egui::Color32::GRAY, "Marks the conclusion of shop branch handling.");
                            }
                            20730 => {
                                ui.label("When [Stay / Rest]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the player pays and rests at the inn.");
                            }
                            20731 => {
                                ui.label("When [Cancel]");
                                ui.colored_label(egui::Color32::GRAY, "Executes if the player declines to stay at the inn.");
                            }
                            20732 => {
                                ui.label("End Inn Processing");
                                ui.colored_label(egui::Color32::GRAY, "Marks the conclusion of inn branch handling.");
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
                        let cmd = self.to_event_command();
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

    pub fn to_event_command(&mut self) -> EventCommandInfo {
        let params: Vec<i32>;
        match self.selected_code {
            10120 => {
                params = vec![self.param0, self.param1];
            }
            10140 => {
                let active_choices: Vec<String> = self.choices.iter().filter(|c| !c.trim().is_empty()).cloned().collect();
                if !active_choices.is_empty() {
                    self.string_val = active_choices.join("/");
                }
                params = vec![self.param0];
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
            10450 => {
                params = vec![0, self.param1, self.param0, self.param2, self.param4];
            }
            10610 | 10620 => {
                params = vec![self.param1];
            }
            10630 => {
                params = vec![self.param1, self.param2, self.param3];
            }
            10640 | 10650 => {
                params = vec![self.param1, self.param2];
            }
            10810 => {
                params = vec![0, self.param1, self.param2, self.param3, 0];
            }
            10820 | 10830 => {
                params = vec![self.param0, self.param1, self.param2];
            }
            10840 => {
                params = vec![];
            }
            10850 => {
                params = vec![self.param0, self.param1, self.param2, self.param3];
            }
            10860 => {
                params = vec![self.param0, self.param1, self.param2, self.param3, 0];
            }
            11010 => {
                params = vec![self.param0];
            }
            11040 => {
                params = vec![self.param0, self.param1, self.param2, self.param3, self.param4, self.param5];
            }
            11050 => {
                params = vec![self.param0, self.param1, self.param2, self.param3];
            }
            11210 => {
                params = vec![self.param0, self.param1, self.param2];
            }
            11410 => {
                params = vec![self.param0];
            }
            11510 | 11550 => {
                params = vec![self.param1, self.param2, 50];
            }
            11520 => {
                params = vec![self.param0];
            }
            11030 => {
                params = vec![self.param0, self.param1, self.param2, self.param3];
            }
            11070 => {
                params = vec![self.param0];
            }
            11110 | 11120 => {
                let mut p = vec![
                    self.param0.max(1),
                    0, // Constant position mode
                    self.param1, // X
                    self.param2, // Y
                    if self.param4 == 0 { 100 } else { self.param4 }, // Magnification
                    self.param5, // Transparency
                    self.param3, // Fixed to map
                    self.raw_params.get(7).copied().unwrap_or(0), // Red
                    self.raw_params.get(8).copied().unwrap_or(0), // Green
                    self.raw_params.get(9).copied().unwrap_or(0), // Blue
                    self.raw_params.get(10).copied().unwrap_or(0), // Saturation
                    self.raw_params.get(11).copied().unwrap_or(0), // Effect Mode
                    self.raw_params.get(12).copied().unwrap_or(0), // Effect Power
                ];
                if self.selected_code == 11120 {
                    p.push(self.raw_params.get(13).copied().unwrap_or(10)); // Duration
                    p.push(self.raw_params.get(14).copied().unwrap_or(0)); // Wait
                }
                params = p;
            }
            11130 => {
                params = vec![self.param0];
            }
            11610 => {
                params = vec![self.param0, self.param1];
            }
            11710 => {
                params = vec![self.param0];
            }
            11720 => {
                params = vec![self.param0, self.param1, 0, 0];
            }
            10720 => {
                let mut p = vec![self.param0, 0];
                p.extend_from_slice(&self.shop_items);
                params = p;
            }
            10730 => {
                params = vec![self.param0, self.param1];
            }
            10740 => {
                params = vec![self.param0, self.param1];
            }
            12010 => {
                params = vec![self.param0, self.param1, self.param2, self.param3, self.param4];
            }
            12330 => {
                params = vec![self.param0];
            }
            11820 | 11840 | 11930 | 11960 | 12110 | 12120 => {
                params = vec![self.param0];
            }
            12210 | 12220 | 12310 | 12320 | 12410 | 12420 | 12510 | 11910 | 11950 | 20110 | 22410 | 20710..=20732 | 23310..=23311 => {
                params = vec![];
            }
            10710 => {
                params = vec![0, self.param1];
            }
            11330 => {
                while self.raw_params.len() < 5 {
                    self.raw_params.push(0);
                }
                self.raw_params[0] = self.param0;
                self.raw_params[1] = self.param1;
                self.raw_params[2] = self.param2;
                self.raw_params[3] = self.param3;
                self.raw_params[4] = (self.raw_params.len() - 5) as i32;
                params = self.raw_params.clone();
            }
            _ => {
                params = self.raw_params.clone();
            }
        }

        EventCommandInfo {
            code: self.selected_code,
            indent: self.indent,
            string: self.string_val.clone(),
            parameters: params,
        }
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

