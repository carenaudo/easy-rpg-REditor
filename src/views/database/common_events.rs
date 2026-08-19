use eframe::egui;
use crate::dialogs::event_command_dialog::EventCommandDialogState;
use crate::lcf_bridge::{event_command_label, CommonEventInfo, SwitchInfo};

pub fn show_common_event_form(
    ui: &mut egui::Ui,
    ce: &mut CommonEventInfo,
    switches: &[SwitchInfo],
    cmd_dialog: &mut EventCommandDialogState,
    selected_cmd: &mut Option<usize>,
    dirty: &mut bool,
) {
    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("📜 {:04}: {}", ce.id, ce.name));
        ui.separator();

        ui.label("Name:");
        let name_edit = ui.text_edit_singleline(&mut ce.name);
        if name_edit.changed() {
            *dirty = true;
        }

        ui.separator();
        ui.label("Trigger:");
        egui::ComboBox::from_id_salt(format!("ce_trigger_{}", ce.id))
            .selected_text(match ce.trigger {
                0 => "Call (Explicit)",
                1 => "AutoStart (Map)",
                2 => "Parallel Process (Map)",
                _ => "Call",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut ce.trigger, 0, "Call (Explicit)").clicked() { *dirty = true; }
                if ui.selectable_value(&mut ce.trigger, 1, "AutoStart (Map)").clicked() { *dirty = true; }
                if ui.selectable_value(&mut ce.trigger, 2, "Parallel Process (Map)").clicked() { *dirty = true; }
            });

        ui.separator();
        let cb = ui.checkbox(&mut ce.switch_flag, "Condition Switch:");
        if cb.changed() { *dirty = true; }
        if ce.switch_flag {
            egui::ComboBox::from_id_salt(format!("ce_sw_combo_{}", ce.id))
                .selected_text(
                    switches.iter().find(|s| s.id == ce.switch_id)
                        .map(|s| format!("{:04}: {}", s.id, s.name))
                        .unwrap_or_else(|| format!("Switch ID {}", ce.switch_id))
                )
                .show_ui(ui, |ui| {
                    for s in switches {
                        if ui.selectable_value(&mut ce.switch_id, s.id, format!("{:04}: {}", s.id, s.name)).clicked() {
                            *dirty = true;
                        }
                    }
                });
        }
    });

    ui.separator();

    // Event Commands Scripting Studio
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading(format!("Event Commands ({})", ce.commands.len()));

            ui.separator();

            if ui.button("➕ Add Command").clicked() {
                let indent = selected_cmd.and_then(|i| ce.commands.get(i)).map(|c| c.indent).unwrap_or(0);
                cmd_dialog.open_new(indent);
            }

            if ui.add_enabled(selected_cmd.is_some(), egui::Button::new("✏ Edit")).clicked() {
                if let Some(idx) = *selected_cmd {
                    if let Some(cmd) = ce.commands.get(idx) {
                        cmd_dialog.open_edit(idx, cmd);
                    }
                }
            }

            if ui.add_enabled(selected_cmd.is_some(), egui::Button::new("🗑 Delete")).clicked() {
                if let Some(idx) = *selected_cmd {
                    ce.commands.remove(idx);
                    *dirty = true;
                    if idx >= ce.commands.len() && idx > 0 {
                        *selected_cmd = Some(idx - 1);
                    } else if ce.commands.is_empty() {
                        *selected_cmd = None;
                    }
                }
            }

            ui.separator();

            // Move Up / Down
            let can_up = selected_cmd.map(|i| i > 0).unwrap_or(false);
            if ui.add_enabled(can_up, egui::Button::new("⬆ Move Up")).clicked() {
                if let Some(idx) = *selected_cmd {
                    ce.commands.swap(idx, idx - 1);
                    *selected_cmd = Some(idx - 1);
                    *dirty = true;
                }
            }

            let can_down = selected_cmd.map(|i| i + 1 < ce.commands.len()).unwrap_or(false);
            if ui.add_enabled(can_down, egui::Button::new("⬇ Move Down")).clicked() {
                if let Some(idx) = *selected_cmd {
                    ce.commands.swap(idx, idx + 1);
                    *selected_cmd = Some(idx + 1);
                    *dirty = true;
                }
            }
        });

        ui.add_space(4.0);

        // Full-height scrollable script canvas
        egui::ScrollArea::vertical()
            .id_salt("ce_commands_scroll_area")
            .show(ui, |ui| {
                if ce.commands.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "(Empty command sequence — click '➕ Add Command' to insert)");
                }

                for (c_idx, cmd) in ce.commands.iter().enumerate() {
                    let label = event_command_label(cmd);
                    let color = crate::lcf_bridge::event_command_color(cmd.code, ui.visuals().dark_mode);
                    let indent_str = "  │ ".repeat(cmd.indent.max(0) as usize);

                    let selected = *selected_cmd == Some(c_idx);

                    ui.horizontal(|ui| {
                        // Line number badge
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 115, 135),
                            format!("{:03}:", c_idx + 1),
                        );

                        // Tree indent + Command Text
                        let text = format!("{}> {}", indent_str, label);
                        let rich = egui::RichText::new(text).color(color).monospace();
                        let resp = ui.selectable_label(selected, rich);

                        if resp.clicked() {
                            *selected_cmd = Some(c_idx);
                        }
                        if resp.double_clicked() {
                            cmd_dialog.open_edit(c_idx, cmd);
                        }
                    });
                }
            });
    });
}

