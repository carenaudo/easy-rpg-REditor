use eframe::egui;
use crate::lcf_bridge::{item_type_label, ItemInfo};

pub fn show_item_form(ui: &mut egui::Ui, item: &mut ItemInfo, dirty: &mut bool) {
    let is_dark = ui.visuals().dark_mode;
    let type_name = item_type_label(item.item_type);
    let badge_color = crate::theme::colors::item_type(item.item_type, is_dark);
    let gold_col = crate::theme::colors::gold(is_dark);
    let uses_col = crate::theme::colors::info(is_dark);

    ui.horizontal_wrapped(|ui| {
        ui.heading(format!("📦 {:04}: {}", item.id, item.name));
        ui.separator();
        ui.colored_label(badge_color, format!("🏷 {}", type_name));
        ui.colored_label(gold_col, format!("💰 {} G", item.price));
        if item.uses > 1 {
            ui.colored_label(uses_col, format!("⏳ {} Uses", item.uses));
        }
    });
    ui.separator();

    let avail_width = ui.available_width();
    let num_cols = if avail_width > 800.0 { 2 } else { 1 };

    egui::ScrollArea::vertical()
        .id_salt("item_editor_scroll")
        .show(ui, |ui| {
            ui.columns(num_cols, |cols| {
                // Column 1: General Info & Core Attributes
                cols[0].group(|ui| {
                    ui.heading("General Properties");
                    egui::Grid::new("item_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut item.name).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Item Type:");
                            egui::ComboBox::from_id_salt("item_type_combo")
                                .selected_text(type_name)
                                .show_ui(ui, |ui| {
                                    for t in 0..=10 {
                                        if ui.selectable_value(&mut item.item_type, t, item_type_label(t)).clicked() {
                                            *dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Price (Gold):");
                            if ui.add(egui::DragValue::new(&mut item.price).range(0..=999999)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Max Uses:");
                            if ui.add(egui::DragValue::new(&mut item.uses).range(0..=255)).on_hover_text("0 or 1 = consumable / single use, >1 = limited uses").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Description:");
                            if ui.add(egui::TextEdit::singleline(&mut item.description).desired_width(260.0)).changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();

                    // Equipment Modifiers (Weapons, Armors, Accessories)
                    if (1..=5).contains(&item.item_type) {
                        ui.heading("⚔ Equipment Stats");
                        egui::Grid::new("item_equip_stats_grid")
                            .num_columns(4)
                            .spacing([12.0, 6.0])
                            .show(ui, |ui| {
                                ui.label("Attack:");
                                if ui.add(egui::DragValue::new(&mut item.atk_points1).range(-999..=999)).changed() { *dirty = true; }
                                ui.label("Defense:");
                                if ui.add(egui::DragValue::new(&mut item.def_points1).range(-999..=999)).changed() { *dirty = true; }
                                ui.end_row();

                                ui.label("Spirit:");
                                if ui.add(egui::DragValue::new(&mut item.spi_points1).range(-999..=999)).changed() { *dirty = true; }
                                ui.label("Agility:");
                                if ui.add(egui::DragValue::new(&mut item.agi_points1).range(-999..=999)).changed() { *dirty = true; }
                                ui.end_row();

                                ui.label("Max HP:");
                                if ui.add(egui::DragValue::new(&mut item.max_hp_points).range(-9999..=9999)).changed() { *dirty = true; }
                                ui.label("Max SP:");
                                if ui.add(egui::DragValue::new(&mut item.max_sp_points).range(-9999..=9999)).changed() { *dirty = true; }
                                ui.end_row();
                            });
                    }

                    // Recovery Effects (Medicine, Usable items)
                    if item.item_type == 0 || item.item_type == 6 || item.item_type == 7 {
                        ui.heading("💚 Recovery Effects");
                        egui::Grid::new("item_recovery_grid")
                            .num_columns(2)
                            .spacing([12.0, 6.0])
                            .show(ui, |ui| {
                                ui.label("HP Recovery Rate (%):");
                                if ui.add(egui::DragValue::new(&mut item.recover_hp_rate).range(0..=100)).changed() { *dirty = true; }
                                ui.end_row();

                                ui.label("HP Flat Recovery:");
                                if ui.add(egui::DragValue::new(&mut item.recover_hp).range(0..=99999)).changed() { *dirty = true; }
                                ui.end_row();

                                ui.label("SP Recovery Rate (%):");
                                if ui.add(egui::DragValue::new(&mut item.recover_sp_rate).range(0..=100)).changed() { *dirty = true; }
                                ui.end_row();

                                ui.label("SP Flat Recovery:");
                                if ui.add(egui::DragValue::new(&mut item.recover_sp).range(0..=9999)).changed() { *dirty = true; }
                                ui.end_row();
                            });
                    }
                });

                // Column 2: Usability, Triggers & Special Traits
                cols[1].group(|ui| {
                    ui.heading("🎯 Usability & Scope");
                    egui::Grid::new("item_usage_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Usable in Menu:");
                            if ui.checkbox(&mut item.occasion_field1, "Allowed").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Usable in Battle:");
                            if ui.checkbox(&mut item.occasion_battle, "Allowed").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Target Scope:");
                            let scope_text = if item.entire_party { "👥 Entire Party" } else { "👤 Single Ally" };
                            if ui.checkbox(&mut item.entire_party, scope_text).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Battle Animation ID:");
                            if ui.add(egui::DragValue::new(&mut item.animation_id).range(0..=500)).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Trigger Switch ID:");
                            if ui.add(egui::DragValue::new(&mut item.switch_id).range(0..=5000)).on_hover_text("Turn this switch ON when item is used").changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Teach/Cast Skill ID:");
                            if ui.add(egui::DragValue::new(&mut item.skill_id).range(0..=5000)).on_hover_text("Skill ID learned (Book) or cast on use").changed() { *dirty = true; }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.heading("✨ Special Traits & Flags");
                    ui.vertical(|ui| {
                        if item.item_type == 1 {
                            // Weapon specific
                            if ui.checkbox(&mut item.two_handed, "Two-Handed Weapon").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.dual_attack, "Attack Twice per Turn").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.attack_all, "Attack All Enemies").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.preemptive, "Preemptive Strike Bonus").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.ignore_evasion, "Ignore Enemy Evasion").changed() { *dirty = true; }
                        }
                        if (2..=5).contains(&item.item_type) {
                            // Armor / Accessory specific
                            if ui.checkbox(&mut item.prevent_critical, "Prevent Critical Hits").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.raise_evasion, "Raise Evasion Rate").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.half_sp_cost, "Half SP Consumption").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.no_terrain_damage, "Prevent Terrain Damage (Lava/Swamp)").changed() { *dirty = true; }
                            if ui.checkbox(&mut item.cursed, "Cursed (Cannot be unequipped)").changed() { *dirty = true; }
                        }
                    });
                });
            });
        });
}

