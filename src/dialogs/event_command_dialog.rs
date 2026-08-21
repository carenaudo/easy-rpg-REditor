use eframe::egui;
use crate::lcf_bridge::EventCommandInfo;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommandCategory {
    Messages,
    Progression,
    Character,
    Movement,
    AudioVisual,
    FlowControl,
    SystemScenes,
}

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
        }
    }
}

impl EventCommandDialogState {
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

        // Auto-detect category from code
        self.category = match cmd.code {
            10110..=10140 => CommandCategory::Messages,
            10210..=10330 | 11210 => CommandCategory::Progression,
            10340..=10420 => CommandCategory::Character,
            10610..=10630 | 11020 | 11030 => CommandCategory::Movement,
            10710..=10910 | 11110..=11140 | 11310..=11320 => CommandCategory::AudioVisual,
            11510..=11570 | 20130..=21520 => CommandCategory::FlowControl,
            11410..=11440 | 11610..=11740 => CommandCategory::SystemScenes,
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
                });

                ui.separator();

                // Command selector within chosen category
                ui.horizontal(|ui| {
                    ui.label("Command:");
                    egui::ComboBox::from_id_salt("cmd_type_sub_combo")
                        .selected_text(match self.selected_code {
                            10110 => "10110: Show Message",
                            10120 => "10120: Message Options",
                            10130 => "10130: Show Choices",
                            10140 => "10140: Input Number",
                            10210 => "10210: Control Switches",
                            10220 => "10220: Control Variables",
                            10310 => "10310: Change Gold",
                            10320 => "10320: Change Items",
                            10330 => "10330: Change Party Members",
                            10340 => "10340: Change EXP",
                            10350 => "10350: Change Level",
                            10360 => "10360: Change Parameters",
                            10370 => "10370: Change Skills",
                            10380 => "10380: Change Equipment",
                            10390 => "10390: Change HP",
                            10400 => "10400: Change SP",
                            10410 => "10410: Change Condition / State",
                            10420 => "10420: Recover All",
                            10610 => "10610: Transfer Player (Teleport)",
                            10630 => "10630: Set Event Location",
                            11020 => "11020: Set Move Route",
                            11030 => "11030: Wait",
                            11110 => "11110: Play BGM",
                            11120 => "11120: Fade Out BGM",
                            11140 => "11140: Play Sound Effect (SE)",
                            10710 => "10710: Erase / Show Screen",
                            10720 => "10720: Tint Screen",
                            10730 => "10730: Flash Screen",
                            10740 => "10740: Shake Screen",
                            10760 => "10760: Weather Effects",
                            10810 => "10810: Show Picture",
                            10820 => "10820: Move Picture",
                            10830 => "10830: Erase Picture",
                            10910 => "10910: Show Battle Animation",
                            11510 => "11510: Conditional Branch",
                            11520 => "11520: Loop",
                            11530 => "11530: Break Loop",
                            11540 => "11540: Exit Event Processing",
                            11550 => "11550: Erase Event",
                            11560 => "11560: Call Common Event",
                            11570 => "11570: Comment",
                            11710 => "11710: Battle Processing",
                            11720 => "11720: Shop Processing",
                            11730 => "11730: Inn Processing",
                            11740 => "11740: Hero Name Input",
                            11430 => "11430: Open Save Menu",
                            11440 => "11440: Open Main Menu",
                            11610 => "11610: Game Over",
                            11620 => "11620: Return to Title Screen",
                            _ => "Custom Event Command",
                        })
                        .show_ui(ui, |ui| {
                            match self.category {
                                CommandCategory::Messages => {
                                    ui.selectable_value(&mut self.selected_code, 10110, "Show Message");
                                    ui.selectable_value(&mut self.selected_code, 10120, "Message Options");
                                    ui.selectable_value(&mut self.selected_code, 10130, "Show Choices");
                                    ui.selectable_value(&mut self.selected_code, 10140, "Input Number");
                                }
                                CommandCategory::Progression => {
                                    ui.selectable_value(&mut self.selected_code, 10210, "Control Switches");
                                    ui.selectable_value(&mut self.selected_code, 10220, "Control Variables");
                                    ui.selectable_value(&mut self.selected_code, 10310, "Change Gold");
                                    ui.selectable_value(&mut self.selected_code, 10320, "Change Items");
                                    ui.selectable_value(&mut self.selected_code, 10330, "Change Party Members");
                                }
                                CommandCategory::Character => {
                                    ui.selectable_value(&mut self.selected_code, 10340, "Change EXP");
                                    ui.selectable_value(&mut self.selected_code, 10350, "Change Level");
                                    ui.selectable_value(&mut self.selected_code, 10360, "Change Parameters");
                                    ui.selectable_value(&mut self.selected_code, 10370, "Change Skills");
                                    ui.selectable_value(&mut self.selected_code, 10380, "Change Equipment");
                                    ui.selectable_value(&mut self.selected_code, 10390, "Change HP");
                                    ui.selectable_value(&mut self.selected_code, 10400, "Change SP");
                                    ui.selectable_value(&mut self.selected_code, 10410, "Change Condition");
                                    ui.selectable_value(&mut self.selected_code, 10420, "Recover All");
                                }
                                CommandCategory::Movement => {
                                    ui.selectable_value(&mut self.selected_code, 10610, "Transfer Player (Teleport)");
                                    ui.selectable_value(&mut self.selected_code, 10630, "Set Event Location");
                                    ui.selectable_value(&mut self.selected_code, 11020, "Set Move Route");
                                    ui.selectable_value(&mut self.selected_code, 11030, "Wait");
                                }
                                CommandCategory::AudioVisual => {
                                    ui.selectable_value(&mut self.selected_code, 11110, "Play BGM");
                                    ui.selectable_value(&mut self.selected_code, 11120, "Fade Out BGM");
                                    ui.selectable_value(&mut self.selected_code, 11140, "Play Sound Effect (SE)");
                                    ui.selectable_value(&mut self.selected_code, 10710, "Erase / Show Screen");
                                    ui.selectable_value(&mut self.selected_code, 10720, "Tint Screen");
                                    ui.selectable_value(&mut self.selected_code, 10730, "Flash Screen");
                                    ui.selectable_value(&mut self.selected_code, 10740, "Shake Screen");
                                    ui.selectable_value(&mut self.selected_code, 10760, "Weather Effects");
                                    ui.selectable_value(&mut self.selected_code, 10810, "Show Picture");
                                    ui.selectable_value(&mut self.selected_code, 10820, "Move Picture");
                                    ui.selectable_value(&mut self.selected_code, 10830, "Erase Picture");
                                    ui.selectable_value(&mut self.selected_code, 10910, "Show Battle Animation");
                                }
                                CommandCategory::FlowControl => {
                                    ui.selectable_value(&mut self.selected_code, 11510, "Conditional Branch");
                                    ui.selectable_value(&mut self.selected_code, 11520, "Loop");
                                    ui.selectable_value(&mut self.selected_code, 11530, "Break Loop");
                                    ui.selectable_value(&mut self.selected_code, 11540, "Exit Event Processing");
                                    ui.selectable_value(&mut self.selected_code, 11550, "Erase Event");
                                    ui.selectable_value(&mut self.selected_code, 11560, "Call Common Event");
                                    ui.selectable_value(&mut self.selected_code, 11570, "Comment");
                                }
                                CommandCategory::SystemScenes => {
                                    ui.selectable_value(&mut self.selected_code, 11710, "Battle Processing");
                                    ui.selectable_value(&mut self.selected_code, 11720, "Shop Processing");
                                    ui.selectable_value(&mut self.selected_code, 11730, "Inn Processing");
                                    ui.selectable_value(&mut self.selected_code, 11740, "Hero Name Input");
                                    ui.selectable_value(&mut self.selected_code, 11430, "Open Save Menu");
                                    ui.selectable_value(&mut self.selected_code, 11440, "Open Main Menu");
                                    ui.selectable_value(&mut self.selected_code, 11610, "Game Over");
                                    ui.selectable_value(&mut self.selected_code, 11620, "Return to Title");
                                }
                            }
                        });
                });

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
                            10130 => {
                                ui.label("Choices (slash-separated, e.g. Yes/No/Cancel):");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            10140 => {
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
                                ui.horizontal(|ui| {
                                    ui.label("Variable ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Operation:");
                                    ui.radio_value(&mut self.param3, 0, "Set (=)");
                                    ui.radio_value(&mut self.param3, 1, "Add (+)");
                                    ui.radio_value(&mut self.param3, 2, "Subtract (-)");
                                    ui.radio_value(&mut self.param3, 3, "Multiply (×)");
                                    ui.radio_value(&mut self.param3, 4, "Divide (÷)");
                                    ui.radio_value(&mut self.param3, 5, "Modulo (%)");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Value / Operand:");
                                    ui.add(egui::DragValue::new(&mut self.param2));
                                });
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
                            10340..=10360 => {
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
                            10370 => {
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
                            10390 | 10400 => {
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
                            10410 => {
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
                            10420 => {
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
                            10610 => {
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
                            11030 => {
                                ui.horizontal(|ui| {
                                    ui.label("Duration (tenths of sec):");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=300));
                                    ui.label(format!("({:.1}s)", self.param0 as f32 / 10.0));
                                });
                            }
                            11110 => {
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
                            11140 => {
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
                            10720 => {
                                ui.label("Screen Color Tint:");
                                ui.horizontal(|ui| {
                                    ui.label("Red:"); ui.add(egui::DragValue::new(&mut self.param0).range(-31..=31));
                                    ui.label("Green:"); ui.add(egui::DragValue::new(&mut self.param1).range(-31..=31));
                                    ui.label("Blue:"); ui.add(egui::DragValue::new(&mut self.param2).range(-31..=31));
                                    ui.label("Chroma:"); ui.add(egui::DragValue::new(&mut self.param3).range(0..=31));
                                });
                            }
                            10760 => {
                                ui.horizontal(|ui| {
                                    ui.label("Weather Effect:");
                                    ui.radio_value(&mut self.param0, 0, "None");
                                    ui.radio_value(&mut self.param0, 1, "Rain");
                                    ui.radio_value(&mut self.param0, 2, "Snow");
                                    ui.radio_value(&mut self.param0, 3, "Sandstorm");
                                });
                            }
                            10810 | 10820 => {
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
                            10830 => {
                                ui.horizontal(|ui| {
                                    ui.label("Picture Number to Erase:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=50));
                                });
                            }
                            11560 => {
                                ui.horizontal(|ui| {
                                    ui.label("Call Common Event ID:");
                                    ui.add(egui::DragValue::new(&mut self.param0).range(1..=5000));
                                });
                            }
                            11710 => {
                                ui.horizontal(|ui| {
                                    ui.label("Battle Troop ID:");
                                    ui.add(egui::DragValue::new(&mut self.param1).range(1..=5000));
                                });
                            }
                            11570 => {
                                ui.label("Comment:");
                                ui.text_edit_singleline(&mut self.string_val);
                            }
                            _ => {
                                ui.horizontal(|ui| {
                                    ui.label("Custom Code:");
                                    ui.add(egui::DragValue::new(&mut self.selected_code));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("String argument:");
                                    ui.text_edit_singleline(&mut self.string_val);
                                });
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        let mut params = Vec::new();
                        match self.selected_code {
                            10120 => {
                                params = vec![self.param0, self.param1];
                            }
                            10140 => {
                                params = vec![self.param1, self.param2];
                            }
                            10210 => {
                                params = vec![0, self.param1, self.param1, self.param3];
                            }
                            10220 => {
                                params = vec![0, self.param1, self.param1, self.param3, 0, self.param2];
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
                            10340..=10360 => {
                                params = vec![0, self.param1, self.param0, 0, self.param2];
                            }
                            10370 => {
                                params = vec![0, self.param1, self.param0, self.param2];
                            }
                            10390 | 10400 => {
                                params = vec![0, self.param1, self.param0, 0, self.param2];
                            }
                            10410 => {
                                params = vec![0, self.param1, self.param0, self.param2];
                            }
                            10420 => {
                                params = vec![self.param0, self.param1];
                            }
                            10610 => {
                                params = vec![0, self.param1, self.param2, self.param3, 0];
                            }
                            11030 => {
                                params = vec![self.param0];
                            }
                            11110 | 11140 => {
                                params = vec![self.param1, self.param2, 50];
                            }
                            10720 => {
                                params = vec![self.param0, self.param1, self.param2, self.param3];
                            }
                            10760 => {
                                params = vec![self.param0];
                            }
                            10810 | 10820 => {
                                params = vec![self.param0, self.param1, self.param2];
                            }
                            10830 => {
                                params = vec![self.param0];
                            }
                            11560 => {
                                params = vec![self.param0];
                            }
                            11710 => {
                                params = vec![0, self.param1];
                            }
                            _ => {
                                if self.param0 != 0 || self.param1 != 0 {
                                    params = vec![self.param0, self.param1, self.param2, self.param3];
                                }
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
}

