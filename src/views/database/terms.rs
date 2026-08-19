use eframe::egui;
use crate::lcf_bridge::TermsInfo;

fn term_field(ui: &mut egui::Ui, label: &str, val: &mut String, dirty: &mut bool) {
    ui.label(label);
    let resp = ui.text_edit_singleline(val);
    if resp.changed() {
        *dirty = true;
    }
    ui.end_row();
}

pub fn show_terms_form(ui: &mut egui::Ui, terms: &mut TermsInfo, dirty: &mut bool) {
    ui.heading("Vocabulary & Terms (RPG_RT.ldb)");
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("terms_form_scroll")
        .show(ui, |ui| {
        ui.collapsing("Menu & Interface Terms", |ui| {
            egui::Grid::new("terms_menu_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field(ui, "New Game:", &mut terms.new_game, dirty);
                    term_field(ui, "Load Game:", &mut terms.load_game, dirty);
                    term_field(ui, "Exit Game:", &mut terms.exit_game, dirty);
                    term_field(ui, "Status:", &mut terms.status, dirty);
                    term_field(ui, "Equipment:", &mut terms.menu_equipment, dirty);
                    term_field(ui, "Save:", &mut terms.menu_save, dirty);
                    term_field(ui, "Quit:", &mut terms.menu_quit, dirty);
                    term_field(ui, "Gold:", &mut terms.gold, dirty);
                    term_field(ui, "Level:", &mut terms.level, dirty);
                    term_field(ui, "HP:", &mut terms.health_points, dirty);
                    term_field(ui, "SP:", &mut terms.spirit_points, dirty);
                    term_field(ui, "Attack:", &mut terms.attack, dirty);
                    term_field(ui, "Defense:", &mut terms.defense, dirty);
                    term_field(ui, "Spirit:", &mut terms.spirit, dirty);
                    term_field(ui, "Agility:", &mut terms.agility, dirty);
                    term_field(ui, "Weapon:", &mut terms.weapon, dirty);
                    term_field(ui, "Shield:", &mut terms.shield, dirty);
                    term_field(ui, "Armor:", &mut terms.armor, dirty);
                    term_field(ui, "Helmet:", &mut terms.helmet, dirty);
                    term_field(ui, "Accessory:", &mut terms.accessory, dirty);
                });
        });

        ui.separator();
        ui.collapsing("System Prompts & Abbreviations", |ui| {
            egui::Grid::new("terms_system_prompts_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field(ui, "Row Selection:", &mut terms.row, dirty);
                    term_field(ui, "Order Selection:", &mut terms.order, dirty);
                    term_field(ui, "Wait Mode ON:", &mut terms.wait_on, dirty);
                    term_field(ui, "Wait Mode OFF:", &mut terms.wait_off, dirty);
                    term_field(ui, "Level Abbr:", &mut terms.lvl_short, dirty);
                    term_field(ui, "HP Abbr:", &mut terms.hp_short, dirty);
                    term_field(ui, "SP Abbr:", &mut terms.sp_short, dirty);
                    term_field(ui, "EXP Abbr:", &mut terms.exp_short, dirty);
                    term_field(ui, "SP Cost:", &mut terms.sp_cost, dirty);
                    term_field(ui, "Possessed Items:", &mut terms.possessed_items, dirty);
                    term_field(ui, "Equipped Items:", &mut terms.equipped_items, dirty);
                    term_field(ui, "File Prompt:", &mut terms.file, dirty);
                    term_field(ui, "Yes:", &mut terms.yes, dirty);
                    term_field(ui, "No:", &mut terms.no, dirty);
                    term_field(ui, "Save Message:", &mut terms.save_game_message, dirty);
                    term_field(ui, "Load Message:", &mut terms.load_game_message, dirty);
                    term_field(ui, "Exit Message:", &mut terms.exit_game_message, dirty);
                });
        });

        ui.separator();
        ui.collapsing("Battle Commands & Status", |ui| {
            egui::Grid::new("terms_battle_cmd_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field(ui, "Attack Command:", &mut terms.command_attack, dirty);
                    term_field(ui, "Defend Command:", &mut terms.command_defend, dirty);
                    term_field(ui, "Item Command:", &mut terms.command_item, dirty);
                    term_field(ui, "Skill Command:", &mut terms.command_skill, dirty);
                    term_field(ui, "Auto Battle:", &mut terms.battle_auto, dirty);
                    term_field(ui, "Escape:", &mut terms.battle_escape, dirty);
                    term_field(ui, "Fight:", &mut terms.battle_fight, dirty);
                    term_field(ui, "Normal Status:", &mut terms.normal_status, dirty);
                });
        });

        ui.separator();
        ui.collapsing("Battle Messages & Combat", |ui| {
            egui::Grid::new("terms_battle_msg_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field(ui, "Encounter:", &mut terms.encounter, dirty);
                    term_field(ui, "Special Combat:", &mut terms.special_combat, dirty);
                    term_field(ui, "Attacking:", &mut terms.attacking, dirty);
                    term_field(ui, "Defending:", &mut terms.defending, dirty);
                    term_field(ui, "Observing:", &mut terms.observing, dirty);
                    term_field(ui, "Focus / Charge:", &mut terms.focus, dirty);
                    term_field(ui, "Autodestruction:", &mut terms.autodestruction, dirty);
                    term_field(ui, "Escape Success:", &mut terms.escape_success, dirty);
                    term_field(ui, "Escape Failure:", &mut terms.escape_failure, dirty);
                    term_field(ui, "Enemy Escape:", &mut terms.enemy_escape, dirty);
                    term_field(ui, "Enemy Transform:", &mut terms.enemy_transform, dirty);
                    term_field(ui, "Enemy Damaged:", &mut terms.enemy_damaged, dirty);
                    term_field(ui, "Enemy Undamaged:", &mut terms.enemy_undamaged, dirty);
                    term_field(ui, "Actor Damaged:", &mut terms.actor_damaged, dirty);
                    term_field(ui, "Actor Undamaged:", &mut terms.actor_undamaged, dirty);
                    term_field(ui, "Enemy Critical:", &mut terms.enemy_critical, dirty);
                    term_field(ui, "Actor Critical:", &mut terms.actor_critical, dirty);
                    term_field(ui, "Level Up:", &mut terms.level_up, dirty);
                    term_field(ui, "Skill Learned:", &mut terms.skill_learned, dirty);
                    term_field(ui, "Miss:", &mut terms.miss, dirty);
                    term_field(ui, "Dodge:", &mut terms.dodge, dirty);
                    term_field(ui, "Use Item:", &mut terms.use_item, dirty);
                    term_field(ui, "HP Recovery:", &mut terms.hp_recovery, dirty);
                    term_field(ui, "Parameter Increase:", &mut terms.parameter_increase, dirty);
                    term_field(ui, "Parameter Decrease:", &mut terms.parameter_decrease, dirty);
                    term_field(ui, "Resistance Increase:", &mut terms.resistance_increase, dirty);
                    term_field(ui, "Resistance Decrease:", &mut terms.resistance_decrease, dirty);
                    term_field(ui, "Enemy HP Absorbed:", &mut terms.enemy_hp_absorbed, dirty);
                    term_field(ui, "Actor HP Absorbed:", &mut terms.actor_hp_absorbed, dirty);
                    term_field(ui, "Skill Failure A:", &mut terms.skill_failure_a, dirty);
                    term_field(ui, "Skill Failure B:", &mut terms.skill_failure_b, dirty);
                    term_field(ui, "Skill Failure C:", &mut terms.skill_failure_c, dirty);
                    term_field(ui, "Victory:", &mut terms.victory, dirty);
                    term_field(ui, "Defeat:", &mut terms.defeat, dirty);
                    term_field(ui, "EXP Received:", &mut terms.exp_received, dirty);
                    term_field(ui, "Gold Received A:", &mut terms.gold_recieved_a, dirty);
                    term_field(ui, "Gold Received B:", &mut terms.gold_recieved_b, dirty);
                    term_field(ui, "Item Received:", &mut terms.item_recieved, dirty);
                });
        });

        ui.separator();
        ui.collapsing("Shop & Inn Dialogues", |ui| {
            egui::Grid::new("terms_shop_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    term_field(ui, "Shop Greeting:", &mut terms.shop_greeting1, dirty);
                    term_field(ui, "Shop Re-greeting:", &mut terms.shop_regreeting1, dirty);
                    term_field(ui, "Shop Buy Prompt:", &mut terms.shop_buy1, dirty);
                    term_field(ui, "Shop Sell Prompt:", &mut terms.shop_sell1, dirty);
                    term_field(ui, "Shop Leave:", &mut terms.shop_leave1, dirty);
                    term_field(ui, "Shop Buy Select:", &mut terms.shop_buy_select1, dirty);
                    term_field(ui, "Shop Buy Number:", &mut terms.shop_buy_number1, dirty);
                    term_field(ui, "Shop Purchased:", &mut terms.shop_purchased1, dirty);
                    term_field(ui, "Shop Sell Select:", &mut terms.shop_sell_select1, dirty);
                    term_field(ui, "Shop Sell Number:", &mut terms.shop_sell_number1, dirty);
                    term_field(ui, "Shop Sold:", &mut terms.shop_sold1, dirty);
                    term_field(ui, "Inn Greeting:", &mut terms.inn_a_greeting_1, dirty);
                    term_field(ui, "Inn Accept:", &mut terms.inn_a_accept, dirty);
                    term_field(ui, "Inn Cancel:", &mut terms.inn_a_cancel, dirty);
                });
        });
    });
}
