use eframe::egui;
use crate::lcf_bridge::TermsInfo;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TermsSubTab {
    MenuSystem,
    Battle,
    Shops,
    Inns,
    Extensions,
}

fn term_field(ui: &mut egui::Ui, label: &str, val: &mut String, dirty: &mut bool) {
    ui.label(label);
    let resp = ui.text_edit_singleline(val);
    if resp.changed() {
        *dirty = true;
    }
    ui.end_row();
}

fn term_field_filtered(ui: &mut egui::Ui, label: &str, val: &mut String, dirty: &mut bool, filter: &str) {
    if !filter.is_empty() {
        let f = filter.to_lowercase();
        let match_label = label.to_lowercase().contains(&f);
        let match_val = val.to_lowercase().contains(&f);
        if !match_label && !match_val {
            return;
        }
    }
    term_field(ui, label, val, dirty);
}

pub fn show_terms_form(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool) {
    ui.horizontal(|ui| {
        ui.heading("Vocabulary & Terms (RPG_RT.ldb)");
        ui.separator();
        ui.colored_label(
            ui.visuals().text_color(),
            "Complete RPG Maker 2000 / 2003 & EasyRPG Vocabulary (153 fields)",
        );
    });
    ui.separator();

    let mut current_tab = ui.data_mut(|d| {
        d.get_temp(egui::Id::new("terms_active_subtab"))
            .unwrap_or(TermsSubTab::MenuSystem)
    });

    let mut search_query = ui.data_mut(|d| {
        d.get_temp::<String>(egui::Id::new("terms_search_query"))
            .unwrap_or_default()
    });

    ui.horizontal(|ui| {
        ui.selectable_value(&mut current_tab, TermsSubTab::MenuSystem, "📋 Menu & Interface");
        ui.selectable_value(&mut current_tab, TermsSubTab::Battle, "⚔ Battle & Combat");
        ui.selectable_value(&mut current_tab, TermsSubTab::Shops, "🏪 Shop Dialogues");
        ui.selectable_value(&mut current_tab, TermsSubTab::Inns, "🏨 Inn Dialogues");
        ui.selectable_value(&mut current_tab, TermsSubTab::Extensions, "⚡ Engine Extensions");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !search_query.is_empty() && ui.button("✖").clicked() {
                search_query.clear();
            }
            ui.add(egui::TextEdit::singleline(&mut search_query).hint_text("🔍 Filter terms..."));
        });
    });

    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new("terms_active_subtab"), current_tab);
        d.insert_temp(egui::Id::new("terms_search_query"), search_query.clone());
    });

    ui.separator();

    let filter = search_query.trim();

    egui::ScrollArea::vertical()
        .id_salt("terms_form_scroll")
        .show(ui, |ui| {
            match current_tab {
                TermsSubTab::MenuSystem => {
                    show_menu_system_tab(ui, terms, dirty, filter);
                }
                TermsSubTab::Battle => {
                    show_battle_tab(ui, terms, dirty, filter);
                }
                TermsSubTab::Shops => {
                    show_shops_tab(ui, terms, dirty, filter);
                }
                TermsSubTab::Inns => {
                    show_inns_tab(ui, terms, dirty, filter);
                }
                TermsSubTab::Extensions => {
                    show_extensions_tab(ui, terms, dirty, filter);
                }
            }
        });
}

fn show_menu_system_tab(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool, filter: &str) {
    ui.collapsing("Main Menu & Commands", |ui| {
        egui::Grid::new("terms_menu_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "New Game:", &mut terms.new_game, dirty, filter);
                term_field_filtered(ui, "Load Game:", &mut terms.load_game, dirty, filter);
                term_field_filtered(ui, "Exit Game:", &mut terms.exit_game, dirty, filter);
                term_field_filtered(ui, "Status:", &mut terms.status, dirty, filter);
                term_field_filtered(ui, "Equipment:", &mut terms.menu_equipment, dirty, filter);
                term_field_filtered(ui, "Save:", &mut terms.menu_save, dirty, filter);
                term_field_filtered(ui, "Quit:", &mut terms.menu_quit, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Hero Stats & Labels", |ui| {
        egui::Grid::new("terms_stats_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Gold / Currency:", &mut terms.gold, dirty, filter);
                term_field_filtered(ui, "Level:", &mut terms.level, dirty, filter);
                term_field_filtered(ui, "HP (Health Points):", &mut terms.health_points, dirty, filter);
                term_field_filtered(ui, "SP (Spirit Points):", &mut terms.spirit_points, dirty, filter);
                term_field_filtered(ui, "Normal Status:", &mut terms.normal_status, dirty, filter);
                term_field_filtered(ui, "Attack:", &mut terms.attack, dirty, filter);
                term_field_filtered(ui, "Defense:", &mut terms.defense, dirty, filter);
                term_field_filtered(ui, "Spirit / Mind:", &mut terms.spirit, dirty, filter);
                term_field_filtered(ui, "Agility / Speed:", &mut terms.agility, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Equipment Slot Names", |ui| {
        egui::Grid::new("terms_equip_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Weapon:", &mut terms.weapon, dirty, filter);
                term_field_filtered(ui, "Shield:", &mut terms.shield, dirty, filter);
                term_field_filtered(ui, "Armor:", &mut terms.armor, dirty, filter);
                term_field_filtered(ui, "Helmet:", &mut terms.helmet, dirty, filter);
                term_field_filtered(ui, "Accessory:", &mut terms.accessory, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Abbreviations & Item Quantities", |ui| {
        egui::Grid::new("terms_abbr_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Level Abbreviation (LV):", &mut terms.lvl_short, dirty, filter);
                term_field_filtered(ui, "HP Abbreviation:", &mut terms.hp_short, dirty, filter);
                term_field_filtered(ui, "SP Abbreviation:", &mut terms.sp_short, dirty, filter);
                term_field_filtered(ui, "EXP Abbreviation:", &mut terms.exp_short, dirty, filter);
                term_field_filtered(ui, "SP Cost Label:", &mut terms.sp_cost, dirty, filter);
                term_field_filtered(ui, "Possessed Items Label:", &mut terms.possessed_items, dirty, filter);
                term_field_filtered(ui, "Equipped Items Label:", &mut terms.equipped_items, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("System Settings & File Confirmations", |ui| {
        egui::Grid::new("terms_sys_file_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Formation Row Selection:", &mut terms.row, dirty, filter);
                term_field_filtered(ui, "Formation Order Selection:", &mut terms.order, dirty, filter);
                term_field_filtered(ui, "Wait Mode ON:", &mut terms.wait_on, dirty, filter);
                term_field_filtered(ui, "Wait Mode OFF:", &mut terms.wait_off, dirty, filter);
                term_field_filtered(ui, "File Prompt:", &mut terms.file, dirty, filter);
                term_field_filtered(ui, "Yes Confirmation:", &mut terms.yes, dirty, filter);
                term_field_filtered(ui, "No Confirmation:", &mut terms.no, dirty, filter);
                term_field_filtered(ui, "Save Game Confirmation:", &mut terms.save_game_message, dirty, filter);
                term_field_filtered(ui, "Load Game Confirmation:", &mut terms.load_game_message, dirty, filter);
                term_field_filtered(ui, "Exit Game Confirmation:", &mut terms.exit_game_message, dirty, filter);
            });
    });
}

fn show_battle_tab(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool, filter: &str) {
    ui.collapsing("Battle Commands", |ui| {
        egui::Grid::new("terms_battle_cmd_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Fight Command:", &mut terms.battle_fight, dirty, filter);
                term_field_filtered(ui, "Auto Battle Command:", &mut terms.battle_auto, dirty, filter);
                term_field_filtered(ui, "Escape Command:", &mut terms.battle_escape, dirty, filter);
                term_field_filtered(ui, "Attack Command:", &mut terms.command_attack, dirty, filter);
                term_field_filtered(ui, "Defend Command:", &mut terms.command_defend, dirty, filter);
                term_field_filtered(ui, "Item Command:", &mut terms.command_item, dirty, filter);
                term_field_filtered(ui, "Skill Command:", &mut terms.command_skill, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Combat Initialization & Outcomes", |ui| {
        egui::Grid::new("terms_combat_init_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Encounter Message:", &mut terms.encounter, dirty, filter);
                term_field_filtered(ui, "Special / Preemptive Combat:", &mut terms.special_combat, dirty, filter);
                term_field_filtered(ui, "Battle Start (2003):", &mut terms.battle_start, dirty, filter);
                term_field_filtered(ui, "Escape Success:", &mut terms.escape_success, dirty, filter);
                term_field_filtered(ui, "Escape Failure:", &mut terms.escape_failure, dirty, filter);
                term_field_filtered(ui, "Victory Message:", &mut terms.victory, dirty, filter);
                term_field_filtered(ui, "Defeat Message:", &mut terms.defeat, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Rewards & Spoils", |ui| {
        egui::Grid::new("terms_rewards_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "EXP Received:", &mut terms.exp_received, dirty, filter);
                term_field_filtered(ui, "Gold Received A:", &mut terms.gold_recieved_a, dirty, filter);
                term_field_filtered(ui, "Gold Received B:", &mut terms.gold_recieved_b, dirty, filter);
                term_field_filtered(ui, "Item Received:", &mut terms.item_recieved, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Actions & Maneuvers", |ui| {
        egui::Grid::new("terms_actions_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Attacking:", &mut terms.attacking, dirty, filter);
                term_field_filtered(ui, "Defending:", &mut terms.defending, dirty, filter);
                term_field_filtered(ui, "Observing:", &mut terms.observing, dirty, filter);
                term_field_filtered(ui, "Focus / Charge:", &mut terms.focus, dirty, filter);
                term_field_filtered(ui, "Autodestruction:", &mut terms.autodestruction, dirty, filter);
                term_field_filtered(ui, "Enemy Escape:", &mut terms.enemy_escape, dirty, filter);
                term_field_filtered(ui, "Enemy Transform:", &mut terms.enemy_transform, dirty, filter);
                term_field_filtered(ui, "Use Item:", &mut terms.use_item, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Hits, Dodges & Criticals", |ui| {
        egui::Grid::new("terms_criticals_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Enemy Critical Hit:", &mut terms.enemy_critical, dirty, filter);
                term_field_filtered(ui, "Actor Critical Hit:", &mut terms.actor_critical, dirty, filter);
                term_field_filtered(ui, "Miss (2003):", &mut terms.miss, dirty, filter);
                term_field_filtered(ui, "Dodge (2000):", &mut terms.dodge, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Damage & Recovery", |ui| {
        egui::Grid::new("terms_damage_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Enemy Damaged:", &mut terms.enemy_damaged, dirty, filter);
                term_field_filtered(ui, "Enemy Undamaged:", &mut terms.enemy_undamaged, dirty, filter);
                term_field_filtered(ui, "Actor Damaged:", &mut terms.actor_damaged, dirty, filter);
                term_field_filtered(ui, "Actor Undamaged:", &mut terms.actor_undamaged, dirty, filter);
                term_field_filtered(ui, "HP Recovery:", &mut terms.hp_recovery, dirty, filter);
                term_field_filtered(ui, "Enemy HP Absorbed:", &mut terms.enemy_hp_absorbed, dirty, filter);
                term_field_filtered(ui, "Actor HP Absorbed:", &mut terms.actor_hp_absorbed, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Parameters, Resistance & Growth", |ui| {
        egui::Grid::new("terms_growth_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Parameter Increase:", &mut terms.parameter_increase, dirty, filter);
                term_field_filtered(ui, "Parameter Decrease:", &mut terms.parameter_decrease, dirty, filter);
                term_field_filtered(ui, "Resistance Increase:", &mut terms.resistance_increase, dirty, filter);
                term_field_filtered(ui, "Resistance Decrease:", &mut terms.resistance_decrease, dirty, filter);
                term_field_filtered(ui, "Level Up:", &mut terms.level_up, dirty, filter);
                term_field_filtered(ui, "Skill Learned:", &mut terms.skill_learned, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("Skill Failures", |ui| {
        egui::Grid::new("terms_failure_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Skill Failure A:", &mut terms.skill_failure_a, dirty, filter);
                term_field_filtered(ui, "Skill Failure B:", &mut terms.skill_failure_b, dirty, filter);
                term_field_filtered(ui, "Skill Failure C:", &mut terms.skill_failure_c, dirty, filter);
            });
    });
}

fn show_shops_tab(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool, filter: &str) {
    let mut shop_tab = ui.data_mut(|d| {
        d.get_temp::<usize>(egui::Id::new("terms_active_shop_tab"))
            .unwrap_or(1)
    });

    ui.horizontal(|ui| {
        ui.label("Shopkeeper Persona:");
        ui.selectable_value(&mut shop_tab, 1, "🏪 Shop Pattern 1 (Standard)");
        ui.selectable_value(&mut shop_tab, 2, "🏪 Shop Pattern 2 (Polite)");
        ui.selectable_value(&mut shop_tab, 3, "🏪 Shop Pattern 3 (Casual)");
    });
    ui.data_mut(|d| d.insert_temp(egui::Id::new("terms_active_shop_tab"), shop_tab));
    ui.separator();

    match shop_tab {
        1 => {
            ui.heading("Shop Pattern 1");
            egui::Grid::new("terms_shop1_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field_filtered(ui, "Greeting:", &mut terms.shop_greeting1, dirty, filter);
                    term_field_filtered(ui, "Re-greeting:", &mut terms.shop_regreeting1, dirty, filter);
                    term_field_filtered(ui, "Buy Command:", &mut terms.shop_buy1, dirty, filter);
                    term_field_filtered(ui, "Sell Command:", &mut terms.shop_sell1, dirty, filter);
                    term_field_filtered(ui, "Leave / Exit:", &mut terms.shop_leave1, dirty, filter);
                    term_field_filtered(ui, "Buy Item Prompt:", &mut terms.shop_buy_select1, dirty, filter);
                    term_field_filtered(ui, "Buy Quantity Prompt:", &mut terms.shop_buy_number1, dirty, filter);
                    term_field_filtered(ui, "Purchase Confirmed:", &mut terms.shop_purchased1, dirty, filter);
                    term_field_filtered(ui, "Sell Item Prompt:", &mut terms.shop_sell_select1, dirty, filter);
                    term_field_filtered(ui, "Sell Quantity Prompt:", &mut terms.shop_sell_number1, dirty, filter);
                    term_field_filtered(ui, "Sale Confirmed:", &mut terms.shop_sold1, dirty, filter);
                });
        }
        2 => {
            ui.heading("Shop Pattern 2");
            egui::Grid::new("terms_shop2_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field_filtered(ui, "Greeting:", &mut terms.shop_greeting2, dirty, filter);
                    term_field_filtered(ui, "Re-greeting:", &mut terms.shop_regreeting2, dirty, filter);
                    term_field_filtered(ui, "Buy Command:", &mut terms.shop_buy2, dirty, filter);
                    term_field_filtered(ui, "Sell Command:", &mut terms.shop_sell2, dirty, filter);
                    term_field_filtered(ui, "Leave / Exit:", &mut terms.shop_leave2, dirty, filter);
                    term_field_filtered(ui, "Buy Item Prompt:", &mut terms.shop_buy_select2, dirty, filter);
                    term_field_filtered(ui, "Buy Quantity Prompt:", &mut terms.shop_buy_number2, dirty, filter);
                    term_field_filtered(ui, "Purchase Confirmed:", &mut terms.shop_purchased2, dirty, filter);
                    term_field_filtered(ui, "Sell Item Prompt:", &mut terms.shop_sell_select2, dirty, filter);
                    term_field_filtered(ui, "Sell Quantity Prompt:", &mut terms.shop_sell_number2, dirty, filter);
                    term_field_filtered(ui, "Sale Confirmed:", &mut terms.shop_sold2, dirty, filter);
                });
        }
        _ => {
            ui.heading("Shop Pattern 3");
            egui::Grid::new("terms_shop3_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field_filtered(ui, "Greeting:", &mut terms.shop_greeting3, dirty, filter);
                    term_field_filtered(ui, "Re-greeting:", &mut terms.shop_regreeting3, dirty, filter);
                    term_field_filtered(ui, "Buy Command:", &mut terms.shop_buy3, dirty, filter);
                    term_field_filtered(ui, "Sell Command:", &mut terms.shop_sell3, dirty, filter);
                    term_field_filtered(ui, "Leave / Exit:", &mut terms.shop_leave3, dirty, filter);
                    term_field_filtered(ui, "Buy Item Prompt:", &mut terms.shop_buy_select3, dirty, filter);
                    term_field_filtered(ui, "Buy Quantity Prompt:", &mut terms.shop_buy_number3, dirty, filter);
                    term_field_filtered(ui, "Purchase Confirmed:", &mut terms.shop_purchased3, dirty, filter);
                    term_field_filtered(ui, "Sell Item Prompt:", &mut terms.shop_sell_select3, dirty, filter);
                    term_field_filtered(ui, "Sell Quantity Prompt:", &mut terms.shop_sell_number3, dirty, filter);
                    term_field_filtered(ui, "Sale Confirmed:", &mut terms.shop_sold3, dirty, filter);
                });
        }
    }
}

fn show_inns_tab(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool, filter: &str) {
    let mut inn_tab = ui.data_mut(|d| {
        d.get_temp::<usize>(egui::Id::new("terms_active_inn_tab"))
            .unwrap_or(1)
    });

    ui.horizontal(|ui| {
        ui.label("Inn Keeper Persona:");
        ui.selectable_value(&mut inn_tab, 1, "🏨 Inn Pattern A");
        ui.selectable_value(&mut inn_tab, 2, "🏨 Inn Pattern B");
    });
    ui.data_mut(|d| d.insert_temp(egui::Id::new("terms_active_inn_tab"), inn_tab));
    ui.separator();

    if inn_tab == 1 {
        ui.heading("Inn Pattern A");
        egui::Grid::new("terms_inn_a_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Greeting Line 1:", &mut terms.inn_a_greeting_1, dirty, filter);
                term_field_filtered(ui, "Greeting Line 2 (Price):", &mut terms.inn_a_greeting_2, dirty, filter);
                term_field_filtered(ui, "Greeting Line 3 (Prompt):", &mut terms.inn_a_greeting_3, dirty, filter);
                term_field_filtered(ui, "Stay Confirmed:", &mut terms.inn_a_accept, dirty, filter);
                term_field_filtered(ui, "Stay Cancelled:", &mut terms.inn_a_cancel, dirty, filter);
            });
    } else {
        ui.heading("Inn Pattern B");
        egui::Grid::new("terms_inn_b_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Greeting Line 1:", &mut terms.inn_b_greeting_1, dirty, filter);
                term_field_filtered(ui, "Greeting Line 2 (Price):", &mut terms.inn_b_greeting_2, dirty, filter);
                term_field_filtered(ui, "Greeting Line 3 (Prompt):", &mut terms.inn_b_greeting_3, dirty, filter);
                term_field_filtered(ui, "Stay Confirmed:", &mut terms.inn_b_accept, dirty, filter);
                term_field_filtered(ui, "Stay Cancelled:", &mut terms.inn_b_cancel, dirty, filter);
            });
    }
}

fn show_extensions_tab(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool, filter: &str) {
    ui.collapsing("Maniac Patch Extended Terms", |ui| {
        egui::Grid::new("terms_maniac_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Item Received (Maniac):", &mut terms.maniac_item_received_a, dirty, filter);
                term_field_filtered(ui, "Level Up (Line A):", &mut terms.maniac_level_up_a, dirty, filter);
                term_field_filtered(ui, "Level Up (Line B):", &mut terms.maniac_level_up_b, dirty, filter);
                term_field_filtered(ui, "Level Up (Line C):", &mut terms.maniac_level_up_c, dirty, filter);
                term_field_filtered(ui, "EXP Received (Maniac):", &mut terms.maniac_exp_received_a, dirty, filter);
                term_field_filtered(ui, "Skill Learned (Maniac):", &mut terms.maniac_skill_learned_a, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("EasyRPG Separators & Formatting", |ui| {
        egui::Grid::new("terms_easyrpg_sep_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Item Number Separator (e.g. ×):", &mut terms.easyrpg_item_number_separator, dirty, filter);
                term_field_filtered(ui, "Skill Cost Separator (e.g. /):", &mut terms.easyrpg_skill_cost_separator, dirty, filter);
                term_field_filtered(ui, "Equipment Change Arrow (e.g. →):", &mut terms.easyrpg_equipment_arrow, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("EasyRPG Status Scene Labels", |ui| {
        egui::Grid::new("terms_easyrpg_status_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Status Scene - Name:", &mut terms.easyrpg_status_scene_name, dirty, filter);
                term_field_filtered(ui, "Status Scene - Class:", &mut terms.easyrpg_status_scene_class, dirty, filter);
                term_field_filtered(ui, "Status Scene - Title:", &mut terms.easyrpg_status_scene_title, dirty, filter);
                term_field_filtered(ui, "Status Scene - Condition:", &mut terms.easyrpg_status_scene_condition, dirty, filter);
                term_field_filtered(ui, "Status Scene - Front Row:", &mut terms.easyrpg_status_scene_front, dirty, filter);
                term_field_filtered(ui, "Status Scene - Back Row:", &mut terms.easyrpg_status_scene_back, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("EasyRPG Order Scene", |ui| {
        egui::Grid::new("terms_easyrpg_order_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Party Order Confirm:", &mut terms.easyrpg_order_scene_confirm, dirty, filter);
                term_field_filtered(ui, "Party Order Redo / Reset:", &mut terms.easyrpg_order_scene_redo, dirty, filter);
            });
    });

    ui.separator();
    ui.collapsing("EasyRPG RPG Maker 2003 Battle Extended Terms", |ui| {
        egui::Grid::new("terms_easyrpg_2k3_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                term_field_filtered(ui, "Double Attack Command:", &mut terms.easyrpg_battle2k3_double_attack, dirty, filter);
                term_field_filtered(ui, "Defend Command:", &mut terms.easyrpg_battle2k3_defend, dirty, filter);
                term_field_filtered(ui, "Observe Command:", &mut terms.easyrpg_battle2k3_observe, dirty, filter);
                term_field_filtered(ui, "Charge Command:", &mut terms.easyrpg_battle2k3_charge, dirty, filter);
                term_field_filtered(ui, "Self-Destruct Command:", &mut terms.easyrpg_battle2k3_selfdestruct, dirty, filter);
                term_field_filtered(ui, "Escape Command:", &mut terms.easyrpg_battle2k3_escape, dirty, filter);
                term_field_filtered(ui, "Special Combat Back Attack:", &mut terms.easyrpg_battle2k3_special_combat_back, dirty, filter);
                term_field_filtered(ui, "Skill Command:", &mut terms.easyrpg_battle2k3_skill, dirty, filter);
                term_field_filtered(ui, "Item Command:", &mut terms.easyrpg_battle2k3_item, dirty, filter);
            });
    });
}
