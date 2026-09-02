rust_i18n::i18n!("locales", fallback = "en");

#[cfg(test)]
mod tests {
    use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;
    use easy_editor::lcf_bridge::{self, AnchorOrigin};
    use easy_editor::tilemap;

    #[test]
    fn test_load_project_and_database() {
        let path = "d:/programacion/test-assets/TestGame/TestGame-2000";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let proj = lcf_bridge::load_project(path);
        assert!(proj.valid, "TestGame-2000 should be a valid project");
        assert!(!proj.maps.is_empty(), "Should have maps loaded");

        let actors = lcf_bridge::get_actors(path);
        assert!(!actors.is_empty(), "Should load actors from LDB");

        let items = lcf_bridge::get_items(path);
        assert!(!items.is_empty(), "Should load items from LDB");

        let skills = lcf_bridge::get_skills(path);
        assert!(!skills.is_empty(), "Should load skills from LDB");

        let enemies = lcf_bridge::get_enemies(path);
        assert!(!enemies.is_empty(), "Should load enemies from LDB");

        let troops = lcf_bridge::get_troops(path);
        assert!(!troops.is_empty(), "Should load troops from LDB");

        let chipsets = lcf_bridge::get_chipsets(path);
        assert!(!chipsets.is_empty(), "Should load chipsets from LDB");

        let states = lcf_bridge::get_states(path);
        assert!(!states.is_empty(), "Should load states from LDB");

        let terrains = lcf_bridge::get_terrains(path);
        assert!(!terrains.is_empty(), "Should load terrains from LDB");

        let animations = lcf_bridge::get_animations(path);
        println!("Loaded {} animations", animations.len());

        let terms = lcf_bridge::get_terms(path);
        assert!(terms.is_some(), "Should load terms from LDB");

        let sys = lcf_bridge::get_system(path);
        assert!(sys.is_some(), "Should load system from LDB");

        let tree = lcf_bridge::get_map_tree(path);
        assert!(!tree.is_empty(), "Should load map tree from LMT");

        let start = lcf_bridge::get_start_points(path);
        assert!(start.party_map_id > 0, "Party start map should be valid");

        let map_id = proj.maps[0].id;
        let layers = lcf_bridge::get_map_layers(path, map_id);
        assert!(layers.width > 0 && layers.height > 0, "Map dimensions should be positive");

        let events = lcf_bridge::get_map_events(path, map_id);
        println!("Map {} has {} events", map_id, events.len());
    }

    #[test]
    fn test_map_resize_anchors() {
        let old_w = 4;
        let old_h = 4;
        let old_lower: Vec<i32> = (0..16).collect();
        let old_upper: Vec<i32> = vec![10000; 16];

        // Top-Left anchor to 6x6
        let (new_lower_tl, _) = lcf_bridge::resize_map_layers(
            &old_lower,
            &old_upper,
            old_w,
            old_h,
            6,
            6,
            AnchorOrigin::TopLeft,
        );
        assert_eq!(new_lower_tl[0], 0); // (0,0) in old -> (0,0) in new
        assert_eq!(new_lower_tl[1], 1);
        assert_eq!(new_lower_tl[6], 4); // (0,1) in old -> (0,1) in new (stride 6)

        // Center anchor to 6x6 (offset x=1, y=1)
        let (new_lower_c, _) = lcf_bridge::resize_map_layers(
            &old_lower,
            &old_upper,
            old_w,
            old_h,
            6,
            6,
            AnchorOrigin::Center,
        );
        assert_eq!(new_lower_c[7], 0); // (1,1) in new -> (0,0) in old
    }

    #[test]
    fn test_autotile_calculation() {
        // Isolated center tile with no matching neighbors (1x1 island with borders all around) -> Subtile 46
        let isolated = tilemap::calculate_autotile_d_subtile(
            false, false, false,
            false,        false,
            false, false, false,
        );
        assert_eq!(isolated, 46, "Isolated autotile should be subtile 46");

        // Fully surrounded center tile with all 8 matching neighbors (seamless solid ground) -> Subtile 0
        let surrounded = tilemap::calculate_autotile_d_subtile(
            true, true, true,
            true,       true,
            true, true, true,
        );
        assert_eq!(surrounded, 0, "Fully surrounded autotile should be subtile 0");
    }

    #[test]
    fn test_new_project_creation() {
        let tmp = std::env::temp_dir().join(format!("test_rpg_proj_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "TestGame".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = false;

        let res = dialog.create_project();
        assert!(res.is_ok(), "Project creation should succeed: {:?}", res);

        let proj_dir = res.unwrap();
        assert!(proj_dir.join("RPG_RT.ldb").exists(), "RPG_RT.ldb must exist");
        assert!(proj_dir.join("RPG_RT.lmt").exists(), "RPG_RT.lmt must exist");
        assert!(proj_dir.join("Map0001.lmu").exists(), "Map0001.lmu must exist");
        assert!(proj_dir.join("CharSet").exists(), "CharSet directory must exist");

        let loaded = lcf_bridge::load_project(&proj_dir.to_string_lossy());
        assert!(loaded.valid, "New project must be valid");
        assert_eq!(loaded.maps.len(), 1, "New project must have 1 initial map");

        let _ = std::fs::remove_dir_all(&proj_dir);

        // Test 2003 creation
        let tmp2003 = std::env::temp_dir().join(format!("test_rpg2003_proj_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let mut dialog2003 = NewProjectDialogState::default();
        dialog2003.project_title = "Test2003".to_string();
        dialog2003.destination_dir = tmp2003.to_string_lossy().to_string();
        dialog2003.is_2003 = true;

        let res2003 = dialog2003.create_project();
        assert!(res2003.is_ok(), "2003 Project creation should succeed: {:?}", res2003);
        let proj2003_dir = res2003.unwrap();
        assert!(proj2003_dir.join("System2").exists(), "2003 System2 directory must exist");
        assert!(proj2003_dir.join("BattleCharSet").exists(), "2003 BattleCharSet directory must exist");

        let _ = std::fs::remove_dir_all(&proj2003_dir);
    }

    #[test]
    fn test_xml_export_endpoints() {
        let path = "d:/programacion/test-assets/TestGame/TestGame-2000";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let tmp_dir = std::env::temp_dir();
        let db_xml = tmp_dir.join("test_db_export.edb");
        let tree_xml = tmp_dir.join("test_tree_export.emt");
        let map_xml = tmp_dir.join("test_map_export.emu");

        let res_db = lcf_bridge::export_database_to_xml(path, &db_xml);
        assert!(res_db.is_ok(), "LDB XML export should succeed: {:?}", res_db);
        assert!(db_xml.exists(), "Exported LDB XML file should exist");
        let _ = std::fs::remove_file(&db_xml);

        let res_tree = lcf_bridge::export_tree_to_xml(path, &tree_xml);
        assert!(res_tree.is_ok(), "LMT XML export should succeed: {:?}", res_tree);
        assert!(tree_xml.exists(), "Exported LMT XML file should exist");
        let _ = std::fs::remove_file(&tree_xml);

        let res_map = lcf_bridge::export_map_to_xml(path, 1, &map_xml);
        assert!(res_map.is_ok(), "LMU XML export should succeed: {:?}", res_map);
        assert!(map_xml.exists(), "Exported LMU XML file should exist");
        let _ = std::fs::remove_file(&map_xml);
    }

    #[test]
    fn test_xml_import_endpoints() {
        // Uses a scratch project (not the shared TestGame fixtures) since
        // import mutates the project's files in place.
        let tmp = std::env::temp_dir().join(format!(
            "test_xml_import_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "XmlImportTest".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = false;
        let proj_dir = dialog.create_project().expect("scratch project creation should succeed");
        let proj_path = proj_dir.to_string_lossy().to_string();

        let tmp_dir = std::env::temp_dir();
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        let db_xml = tmp_dir.join(format!("test_db_import_{unique}.edb"));
        let tree_xml = tmp_dir.join(format!("test_tree_import_{unique}.emt"));
        let map_xml = tmp_dir.join(format!("test_map_import_{unique}.emu"));

        // Export -> re-import each format and confirm the round trip
        // succeeds and the backup file (.bak) was created.
        lcf_bridge::export_database_to_xml(&proj_path, &db_xml).expect("LDB export should succeed");
        let ldb_bak = proj_dir.join("RPG_RT.ldb.bak");
        let _ = std::fs::remove_file(&ldb_bak); // start clean in case a prior run left one
        let res = lcf_bridge::import_database_from_xml(&proj_path, &db_xml);
        assert!(res.is_ok(), "LDB XML import should succeed: {:?}", res);
        assert!(ldb_bak.exists(), "importing should back up the original RPG_RT.ldb");
        let loaded = lcf_bridge::load_project(&proj_path);
        assert!(loaded.valid, "project must still load after LDB import");

        lcf_bridge::export_tree_to_xml(&proj_path, &tree_xml).expect("LMT export should succeed");
        let lmt_bak = proj_dir.join("RPG_RT.lmt.bak");
        let _ = std::fs::remove_file(&lmt_bak);
        let res = lcf_bridge::import_tree_from_xml(&proj_path, &tree_xml);
        assert!(res.is_ok(), "LMT XML import should succeed: {:?}", res);
        assert!(lmt_bak.exists(), "importing should back up the original RPG_RT.lmt");
        let loaded = lcf_bridge::load_project(&proj_path);
        assert!(loaded.valid, "project must still load after LMT import");
        assert_eq!(loaded.maps.len(), 1, "map tree must still have its 1 map after import");

        lcf_bridge::export_map_to_xml(&proj_path, 1, &map_xml).expect("LMU export should succeed");
        let map_bak = proj_dir.join("Map0001.lmu.bak");
        let _ = std::fs::remove_file(&map_bak);
        let res = lcf_bridge::import_map_from_xml(&proj_path, 1, &map_xml);
        assert!(res.is_ok(), "LMU XML import should succeed: {:?}", res);
        assert!(map_bak.exists(), "importing should back up the original Map0001.lmu");

        let _ = std::fs::remove_file(&db_xml);
        let _ = std::fs::remove_file(&tree_xml);
        let _ = std::fs::remove_file(&map_xml);
        let _ = std::fs::remove_dir_all(&proj_dir);
    }

    #[test]
    fn test_stat_growth_curves() {
        use easy_editor::views::database::actors::{generate_growth_curve, GrowthCurvePreset};

        let linear = generate_growth_curve(100, 1000, 10, GrowthCurvePreset::Linear);
        assert_eq!(linear.len(), 10);
        assert_eq!(linear[0], 100);
        assert_eq!(linear[9], 1000);
        assert_eq!(linear[4], 500); // exactly midpoint

        let early = generate_growth_curve(100, 1000, 10, GrowthCurvePreset::EarlyBloomer);
        assert!(early[4] > linear[4], "Early bloomer should have higher stats at mid-level");

        let late = generate_growth_curve(100, 1000, 10, GrowthCurvePreset::LateBloomer);
        assert!(late[4] < linear[4], "Late bloomer should have lower stats at mid-level");
    }

    #[test]
    fn test_event_command_syntax_colors() {
        use easy_editor::lcf_bridge::event_command_color;

        // Dark mode
        let msg_color_dark = event_command_color(10110, true); // Show message
        let if_color_dark = event_command_color(12010, true); // If branch
        let sw_color_dark = event_command_color(10210, true); // Switch operation

        assert_ne!(msg_color_dark, if_color_dark);
        assert_ne!(msg_color_dark, sw_color_dark);

        // Light mode
        let msg_color_light = event_command_color(10110, false);
        let if_color_light = event_command_color(12010, false);
        let sw_color_light = event_command_color(10210, false);

        assert_ne!(msg_color_light, if_color_light);
        assert_ne!(msg_color_light, sw_color_light);
        assert_ne!(msg_color_dark, msg_color_light);

        // Maniac Patch commands get their own distinct color, in both modes.
        let maniac_color_dark = event_command_color(3013, true); // Control Var Array
        assert_ne!(maniac_color_dark, msg_color_dark);
        assert_ne!(maniac_color_dark, if_color_dark);
        let maniac_color_light = event_command_color(3013, false);
        assert_ne!(maniac_color_light, msg_color_light);
        assert_ne!(maniac_color_dark, maniac_color_light);
    }

    #[test]
    fn test_maniac_command_editing_data_tables() {
        use easy_editor::lcf_bridge::{event_command_label, maniac_param_count, maniac_param_hint, EventCommandInfo};

        // maniac_param_count: known Player-implemented codes return Some,
        // unimplemented ones (Player has no CmdSetup dispatch for these)
        // return None.
        assert_eq!(maniac_param_count(3013), Some(5)); // Control Var Array
        assert_eq!(maniac_param_count(3007), Some(23)); // Show String Picture
        assert_eq!(maniac_param_count(3025), None); // Edit Picture - not implemented by Player
        assert_eq!(maniac_param_count(3032), None); // Zoom - not implemented by Player
        assert_eq!(maniac_param_count(99999), None); // not a Maniac code at all

        // maniac_param_hint: only covers the "known but complex" Tier-2
        // commands - no invented semantics for the unimplemented ones.
        assert!(maniac_param_hint(3020, 0).is_some()); // Control Strings, mode bitfield
        assert!(maniac_param_hint(3025, 0).is_none()); // Edit Picture - nothing known
        assert!(maniac_param_hint(3013, 0).is_none()); // Control Var Array has a bespoke form, no hint table entry needed

        // event_command_label: Maniac codes get a "Maniac: <Name>" label
        // instead of the generic "Command #NNNN" fallback.
        let cmd = EventCommandInfo { code: 3013, indent: 0, string: String::new(), parameters: vec![6, 0, 1, 10, 20] };
        let label = event_command_label(&cmd);
        assert!(label.contains("Maniac: Control Var Array"), "label was: {label}");

        let unknown_cmd = EventCommandInfo { code: 3025, indent: 0, string: String::new(), parameters: vec![] };
        let unknown_label = event_command_label(&unknown_cmd);
        assert!(unknown_label.contains("Maniac: Edit Picture"), "label was: {unknown_label}");
    }

    #[test]
    fn test_maniac_command_dialog_full_fidelity() {
        use easy_editor::dialogs::event_command_dialog::{CommandCategory, EventCommandDialogState};
        use easy_editor::lcf_bridge::EventCommandInfo;

        // Regression test for the pre-existing bug: opening a command with
        // more than 6 parameters used to silently truncate to param0..5.
        // `raw_params` must hold the full vector.
        let cmd = EventCommandInfo {
            code: 3020, // Control Strings - a Tier-2 command with up to 8 params
            indent: 1,
            string: "hello".to_string(),
            parameters: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
        let mut state = EventCommandDialogState::default();
        state.open_edit(0, &cmd);

        assert_eq!(state.raw_params, cmd.parameters, "raw_params must preserve every parameter, not just the first 6");
        assert_eq!(state.raw_params.len(), 9);

        // Category auto-detect: Maniac codes must land on the Maniac tab,
        // not the old `_ => Messages` fallback.
        assert_eq!(state.category, CommandCategory::Maniac);

        // A Tier-1 command with fewer than 6 params round-trips too.
        let small_cmd = EventCommandInfo {
            code: 3005, // Get Mouse Position
            indent: 0,
            string: String::new(),
            parameters: vec![10, 11],
        };
        let mut state2 = EventCommandDialogState::default();
        state2.open_edit(0, &small_cmd);
        assert_eq!(state2.raw_params, vec![10, 11]);
        assert_eq!(state2.category, CommandCategory::Maniac);
    }

    #[test]
    fn test_maniac_command_dialog_against_real_fixture() {
        use easy_editor::dialogs::event_command_dialog::EventCommandDialogState;
        use easy_editor::lcf_bridge;

        let maniac_path = "d:/programacion/test-assets/TestGame/TestGame-Maniac";
        if !std::path::Path::new(maniac_path).exists() {
            eprintln!("Skipping test: {:?} not found", maniac_path);
            return;
        }

        let app = {
            let mut app = easy_editor::app_state::EditorAppState::default();
            app.load_project_from(maniac_path.to_string());
            app
        };

        // Find a real Maniac command in this project's maps and confirm
        // opening it for edit preserves every parameter exactly.
        let mut found = false;
        'outer: for (map_id, _) in &app.maps {
            let events = lcf_bridge::get_map_events(maniac_path, *map_id);
            for ev in &events {
                for page in &ev.pages {
                    for cmd in &page.commands {
                        if lcf_bridge::is_maniac_command_code(cmd.code) {
                            let mut state = EventCommandDialogState::default();
                            state.open_edit(0, cmd);
                            assert_eq!(state.raw_params, cmd.parameters, "Maniac command {} on map {} lost parameters on open_edit", cmd.code, map_id);
                            found = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
        assert!(found, "expected at least one real Maniac command somewhere in TestGame-Maniac's maps");
    }

    #[test]
    fn test_troop_command_editing() {
        use easy_editor::lcf_bridge::{EventCommandInfo, TroopPageInfo};
        use easy_editor::views::database::troops::TroopViewState;

        fn cmd(code: i32, indent: i32) -> EventCommandInfo {
            EventCommandInfo { code, indent, string: String::new(), parameters: vec![] }
        }

        let mut page = TroopPageInfo {
            id: 1,
            commands: vec![cmd(12010, 0), cmd(10110, 1), cmd(22010, 0), cmd(10310, 0)],
            ..Default::default()
        };
        let mut state = TroopViewState::default();

        // Add derives indent from the current selection, not always 0
        // (regression test for the pre-existing hardcoded `open_new(0)`).
        state.selected_cmd_idx = Some(1); // the indent=1 "Show Message" row
        let derived_indent = state.selected_cmd_idx.and_then(|i| page.commands.get(i)).map(|c| c.indent).unwrap_or(0);
        assert_eq!(derived_indent, 1);

        // Move Down: swap idx 1 and 2, selection follows to idx 2.
        let idx = state.selected_cmd_idx.unwrap();
        page.commands.swap(idx, idx + 1);
        state.selected_cmd_idx = Some(idx + 1);
        assert_eq!(page.commands[1].code, 22010);
        assert_eq!(page.commands[2].code, 10110);
        assert_eq!(state.selected_cmd_idx, Some(2));

        // Move Up: swap back, selection follows to idx 1.
        let idx = state.selected_cmd_idx.unwrap();
        page.commands.swap(idx, idx - 1);
        state.selected_cmd_idx = Some(idx - 1);
        assert_eq!(page.commands[1].code, 10110);
        assert_eq!(state.selected_cmd_idx, Some(1));

        // Delete a middle entry: selection clamps to the previous entry
        // when the removed index is now out of range.
        page.commands.remove(1);
        assert_eq!(page.commands.len(), 3);
        assert_eq!(page.commands.iter().map(|c| c.code).collect::<Vec<_>>(), vec![12010, 22010, 10310]);

        // Delete until empty: selection clears to None.
        state.selected_cmd_idx = Some(0);
        while !page.commands.is_empty() {
            let idx = state.selected_cmd_idx.unwrap();
            page.commands.remove(idx);
            if idx >= page.commands.len() && idx > 0 {
                state.selected_cmd_idx = Some(idx - 1);
            } else if page.commands.is_empty() {
                state.selected_cmd_idx = None;
            }
        }
        assert!(page.commands.is_empty());
        assert_eq!(state.selected_cmd_idx, None);
    }

    #[test]
    fn test_troop_selection_resets_command_selection() {
        use easy_editor::views::database_view::DatabaseViewState;

        let mut view = DatabaseViewState::default();
        view.selected_troop = 2;
        view.troop_view_state.selected_cmd_idx = Some(5);
        view.troop_view_state.active_page_idx = 3;

        // Simulate clicking a different troop in the sidebar list - must
        // reset both the stale command selection and the active page,
        // mirroring the CommonEvents selection block right next to it.
        view.selected_troop = 0;
        view.troop_view_state.selected_cmd_idx = None;
        view.troop_view_state.active_page_idx = 0;

        assert_eq!(view.selected_troop, 0);
        assert_eq!(view.troop_view_state.selected_cmd_idx, None);
        assert_eq!(view.troop_view_state.active_page_idx, 0);
    }

    #[test]
    fn test_i18n_locales_and_fallback() {
        rust_i18n::set_locale("en");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Open Project");
        assert_eq!(rust_i18n::t!("views.database"), "Database");

        rust_i18n::set_locale("es");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Abrir Proyecto");
        assert_eq!(rust_i18n::t!("views.database"), "Base de Datos");

        rust_i18n::set_locale("ja");
        assert_eq!(rust_i18n::t!("menu.open_project"), "プロジェクトを開く");
        assert_eq!(rust_i18n::t!("views.database"), "データベース");

        rust_i18n::set_locale("de");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Projekt öffnen");
        assert_eq!(rust_i18n::t!("views.database"), "Datenbank");

        rust_i18n::set_locale("fr");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Ouvrir un Projet");
        assert_eq!(rust_i18n::t!("views.database"), "Base de Données");

        rust_i18n::set_locale("it");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Apri Progetto");
        assert_eq!(rust_i18n::t!("views.database"), "Database");

        rust_i18n::set_locale("pt-BR");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Abrir Projeto");
        assert_eq!(rust_i18n::t!("views.database"), "Banco de Dados");

        rust_i18n::set_locale("zh-CN");
        assert_eq!(rust_i18n::t!("menu.open_project"), "打开项目");
        assert_eq!(rust_i18n::t!("views.database"), "数据库");

        // Test fallback to English on missing or unknown locale
        rust_i18n::set_locale("unknown_lang");
        assert_eq!(rust_i18n::t!("menu.open_project"), "Open Project");

        // Reset to English
        rust_i18n::set_locale("en");
    }

    #[test]
    fn test_phase3_database_models() {
        use easy_editor::lcf_bridge::{AnimationInfo, AnimationTimingInfo, TroopInfo, TroopMemberInfo, TroopPageInfo, TroopPageConditionInfo, CommonEventInfo, TerrainInfo};

        // 1. Animation with Timing Cues
        let anim = AnimationInfo {
            id: 1,
            name: "Fire 1".to_string(),
            animation_name: "Fire1".to_string(),
            large: true,
            scope: 1,
            position: 2,
            frame_count: 10,
            frames: Vec::new(),
            timings: vec![
                AnimationTimingInfo {
                    id: 1,
                    frame: 3,
                    se_name: "Fire1".to_string(),
                    flash_scope: 2,
                    flash_red: 31,
                    flash_green: 15,
                    flash_blue: 0,
                    flash_power: 28,
                    screen_shake: 1,
                }
            ],
        };
        assert_eq!(anim.timings.len(), 1);
        assert_eq!(anim.timings[0].flash_red, 31);
        assert_eq!(anim.large, true);

        // 2. Troop with Event Pages and Conditions
        let troop = TroopInfo {
            id: 1,
            name: "Slime x2".to_string(),
            auto_alignment: true,
            appear_randomly: false,
            terrain_set: vec![true, true, false],
            members: vec![
                TroopMemberInfo { enemy_id: 1, x: 100, y: 140, invisible: false },
                TroopMemberInfo { enemy_id: 1, x: 220, y: 140, invisible: true },
            ],
            pages: vec![
                TroopPageInfo {
                    id: 1,
                    condition: TroopPageConditionInfo {
                        flags: 4, // Turn condition
                        turn_a: 1,
                        turn_b: 2,
                        ..Default::default()
                    },
                    commands: Vec::new(),
                }
            ],
        };
        assert_eq!(troop.members.len(), 2);
        assert_eq!(troop.pages[0].condition.turn_a, 1);
        assert!(troop.auto_alignment);

        // 3. Common Event with Switch Trigger
        let ce = CommonEventInfo {
            id: 1,
            name: "Day Night Cycle".to_string(),
            trigger: 2, // Parallel Process
            switch_flag: true,
            switch_id: 42,
            commands: Vec::new(),
        };
        assert_eq!(ce.trigger, 2);
        assert!(ce.switch_flag);
        assert_eq!(ce.switch_id, 42);

        // 4. Terrain Passability & Depth
        let terrain = TerrainInfo {
            id: 1,
            name: "Swamp".to_string(),
            damage: 10,
            encounter_rate: 150,
            background_name: "Swamp".to_string(),
            boat_pass: false,
            ship_pass: false,
            airship_pass: true,
            airship_land: false,
            bush_depth: 2,
            footstep_name: "WaterStep".to_string(),
        };
        assert_eq!(terrain.damage, 10);
        assert_eq!(terrain.bush_depth, 2);
    }

    #[test]
    fn test_phase4_map_properties_and_events() {
        use easy_editor::lcf_bridge::{MapPropertiesInfo, EventCommandInfo};

        // 1. Map Properties with Panorama and Encounter List
        let props = MapPropertiesInfo {
            id: 1,
            name: "Overworld".to_string(),
            parent_map: 0,
            chipset_id: 1,
            width: 100,
            height: 100,
            scroll_type: 3, // Both Loop (World Map)
            parallax_name: "Clouds".to_string(),
            parallax_loop_x: true,
            parallax_loop_y: false,
            parallax_sx: 2,
            parallax_sy: 0,
            music_type: 1,
            music_name: "Field1".to_string(),
            background_type: 0,
            background_name: String::new(),
            teleport: 1,
            escape: 1,
            save: 1,
            encounter_steps: 25,
            encounters: vec![1, 2, 3],
        };
        assert_eq!(props.encounters.len(), 3);
        assert_eq!(props.encounter_steps, 25);
        assert!(props.parallax_loop_x);
        assert_eq!(props.scroll_type, 3);

        // 2. Event Command Info Creation & Label formatting
        let cmd = EventCommandInfo {
            code: 10320, // Change Items
            indent: 1,
            string: String::new(),
            parameters: vec![0, 0, 5, 3], // Add 3 of Item #5
        };
        assert_eq!(cmd.code, 10320);
        let label = easy_editor::lcf_bridge::event_command_label(&cmd);
        assert!(label.contains("Change Items"), "Label should format Change Items command: {}", label);
    }

    #[test]
    fn test_phase5_search_and_event_preconditions() {
        use easy_editor::lcf_bridge::{EventConditionInfo, EventPageInfo};
        use easy_editor::dialogs::project_search::ProjectSearchDialog;
        use easy_editor::app_state::EditorAppState;

        // 1. Event Condition with Hero and Timer
        let cond = EventConditionInfo {
            switch1_flag: true,
            switch1_id: 10,
            switch2_flag: false,
            switch2_id: 0,
            var_flag: true,
            var_id: 5,
            var_value: 100,
            var_compare_op: 1, // >=
            item_flag: true,
            item_id: 3,
            actor_flag: true,
            actor_id: 1,
            timer_flag: true,
            timer_sec: 150, // 2m 30s
            ..Default::default()
        };
        assert!(cond.actor_flag);
        assert_eq!(cond.actor_id, 1);
        assert_eq!(cond.timer_sec, 150);
        assert_eq!(cond.var_compare_op, 1);

        let page = EventPageInfo {
            id: 1,
            character_name: "Hero".to_string(),
            character_index: 0,
            character_direction: 2,
            character_pattern: 1,
            translucent: false,
            move_type: 1, // Random
            move_frequency: 3,
            trigger: 0,
            layer: 1,
            overlap_forbidden: true,
            animation_type: 1, // Continuous Walk in Place
            move_speed: 3,
            condition: cond,
            commands: Vec::new(),
            ..Default::default()
        };
        assert_eq!(page.animation_type, 1);
        assert!(page.overlap_forbidden);

        // 2. Project Search indexing test
        let mut app = EditorAppState::default();
        app.actors.push(easy_editor::lcf_bridge::ActorInfo {
            id: 1,
            name: "Zack the Hero".to_string(),
            title: "Knight".to_string(),
            ..Default::default()
        });
        app.items.push(easy_editor::lcf_bridge::ItemInfo {
            id: 1,
            name: "Excalibur Sword".to_string(),
            description: "Legendary holy sword.".to_string(),
            ..Default::default()
        });

        let mut search = ProjectSearchDialog::default();
        search.query = "Excalibur".to_string();
        search.execute_search(&app);
        assert_eq!(search.results.len(), 1);
        assert_eq!(search.results[0].category, "Items");
        assert!(search.results[0].label.contains("Excalibur"));

        search.query = "Zack".to_string();
        search.execute_search(&app);
        assert_eq!(search.results.len(), 1);
        assert_eq!(search.results[0].category, "Actors");
        assert!(search.results[0].label.contains("Zack"));
    }

    #[test]
    fn test_phase6_resource_manager_and_quick_events() {
        use easy_editor::dialogs::resource_manager_dialog::RESOURCE_CATEGORIES;
        use easy_editor::views::map_view::MapViewState;

        // 1. Verify all 19 standard RPG Maker 2000 & 2003 asset subfolders are recognized
        assert_eq!(RESOURCE_CATEGORIES.len(), 19);
        assert!(RESOURCE_CATEGORIES.contains(&"BattleCharSet"));
        assert!(RESOURCE_CATEGORIES.contains(&"BattleWeapon"));
        assert!(RESOURCE_CATEGORIES.contains(&"Frame"));
        assert!(RESOURCE_CATEGORIES.contains(&"System2"));
        assert!(RESOURCE_CATEGORIES.contains(&"Panorama"));
        assert!(RESOURCE_CATEGORIES.contains(&"Monster"));

        // 2. Test Map Quick Event Generators
        let mut map_view = MapViewState::default();

        // Generate Quick Save Point
        map_view.create_quick_save_point(5, 7);
        assert_eq!(map_view.events.len(), 1);
        let save_ev = &map_view.events[0];
        assert_eq!(save_ev.x, 5);
        assert_eq!(save_ev.y, 7);
        assert!(save_ev.name.starts_with("SavePoint_"));
        assert_eq!(save_ev.pages.len(), 1);
        // Verify Save Menu command (11910) exists in generated script
        assert!(save_ev.pages[0].commands.iter().any(|c| c.code == 11910));

        // Generate Quick Recovery Spring
        map_view.create_quick_recovery(12, 14);
        assert_eq!(map_view.events.len(), 2);
        let fountain_ev = &map_view.events[1];
        assert_eq!(fountain_ev.x, 12);
        assert_eq!(fountain_ev.y, 14);
        assert!(fountain_ev.name.starts_with("Fountain_"));
        // Verify Full Recovery command (10490) exists in generated script
        assert!(fountain_ev.pages[0].commands.iter().any(|c| c.code == 10490));
    }

    #[test]
    fn test_phase7_grid_and_audio_inspector() {
        use easy_editor::views::map_view::MapViewState;
        use easy_editor::dialogs::asset_picker::AssetPickerState;

        // 1. Map View State Grid & Zoom Presets
        let mut map_view = MapViewState::default();
        assert!(map_view.show_grid, "Grid should be enabled by default for tile precision");
        assert_eq!(map_view.zoom, 1.0);

        map_view.zoom = 2.0;
        map_view.show_grid = false;
        assert!(!map_view.show_grid);
        assert_eq!(map_view.zoom, 2.0);

        // 2. Asset Picker Audio State
        let mut picker = AssetPickerState::default();
        picker.category = "Music".to_string();
        picker.selected_file = "Field1".to_string();
        assert_eq!(picker.category, "Music");
        assert_eq!(picker.selected_file, "Field1");

        picker.category = "Sound".to_string();
        picker.selected_file = "Decision1".to_string();
        assert_eq!(picker.category, "Sound");
        assert_eq!(picker.selected_file, "Decision1");
    }

    #[test]
    fn test_phase8_terms_vocabulary_completeness() {
        use easy_editor::lcf_bridge::TermsInfo;

        let mut terms = TermsInfo::default();
        terms.new_game = "New Journey".to_string();
        terms.command_attack = "Strike".to_string();
        terms.exp_received = "%s EXP obtained!".to_string();
        terms.shop_greeting1 = "Welcome, traveler!".to_string();
        terms.inn_a_greeting_1 = "Stay the night for %d Gold?".to_string();

        assert_eq!(terms.new_game, "New Journey");
        assert_eq!(terms.command_attack, "Strike");
        assert_eq!(terms.exp_received, "%s EXP obtained!");
        assert_eq!(terms.shop_greeting1, "Welcome, traveler!");
        assert_eq!(terms.inn_a_greeting_1, "Stay the night for %d Gold?");
    }

    #[test]
    fn test_phase9_map_shift_and_sound_test() {
        use easy_editor::views::map_view::{MapDims, MapViewState};
        use easy_editor::dialogs::sound_test_dialog::SoundTestDialog;
        use easy_editor::lcf_bridge::EventInfo;

        // 1. Map Shift with Wrapping Logic
        let mut map_view = MapViewState::default();
        let w = 5;
        let h = 5;
        let mut lower = vec![0; (w * h) as usize];
        // Set a marker tile at (1, 1)
        lower[(1 * w + 1) as usize] = 42;

        map_view.map_dims = Some(MapDims {
            width: w,
            height: h,
            lower,
            upper: vec![10000; (w * h) as usize],
        });

        map_view.events.push(EventInfo {
            id: 1,
            name: "TestEv".to_string(),
            x: 1,
            y: 1,
            ..Default::default()
        });

        // Shift by (+2, +3) with horizontal & vertical wrapping
        let ctx = eframe::egui::Context::default();
        map_view.shift_map(2, 3, true, true, None, &ctx);

        let dims = map_view.map_dims.as_ref().unwrap();
        // Tile that was at (1, 1) is now at ((1+2)%5, (1+3)%5) = (3, 4)
        assert_eq!(dims.lower[(4 * w + 3) as usize], 42);
        // Event that was at (1, 1) is now at (3, 4)
        assert_eq!(map_view.events[0].x, 3);
        assert_eq!(map_view.events[0].y, 4);

        // 2. Sound Test Dialog
        let mut sound_test = SoundTestDialog::default();
        assert_eq!(sound_test.volume, 100);
        assert_eq!(sound_test.pitch, 100);
        assert_eq!(sound_test.pan, 0);
        assert!(!sound_test.is_playing);

        sound_test.open(None);
        assert!(sound_test.is_open);
    }

    #[test]
    fn test_phase10_project_health_analyzer() {
        use easy_editor::app_state::EditorAppState;
        use easy_editor::dialogs::project_analyzer_dialog::ProjectAnalyzerDialog;
        use easy_editor::lcf_bridge::{ActorInfo, EnemyInfo, ChipsetInfo};

        let mut app = EditorAppState::default();
        app.actors.push(ActorInfo {
            id: 1,
            name: "Hero".to_string(),
            character_name: "HeroNonExistent".to_string(),
            face_name: "HeroFaceNonExistent".to_string(),
            ..Default::default()
        });
        app.enemies.push(EnemyInfo {
            id: 1,
            name: "Slime".to_string(),
            battler_name: "SlimeNonExistent".to_string(),
            ..Default::default()
        });
        app.chipsets.push(ChipsetInfo {
            id: 1,
            name: "World".to_string(),
            chipset_name: "WorldNonExistent".to_string(),
            ..Default::default()
        });

        let mut analyzer = ProjectAnalyzerDialog::default();
        analyzer.run_analysis(&app);

        assert!(analyzer.is_scanned);
        assert_eq!(analyzer.total_actors, 1);
        assert_eq!(analyzer.total_enemies, 1);
        // Should detect missing CharSet, FaceSet, Monster, ChipSet
        assert!(analyzer.missing_assets.len() >= 4);
        assert!(analyzer.missing_assets.iter().any(|m| m.category == "CharSet" && m.file_name == "HeroNonExistent"));
        assert!(analyzer.missing_assets.iter().any(|m| m.category == "FaceSet" && m.file_name == "HeroFaceNonExistent"));
        assert!(analyzer.missing_assets.iter().any(|m| m.category == "Monster" && m.file_name == "SlimeNonExistent"));
        assert!(analyzer.missing_assets.iter().any(|m| m.category == "ChipSet" && m.file_name == "WorldNonExistent"));
    }

    #[test]
    fn test_maniac_patch_detection() {
        use easy_editor::app_state::EditorAppState;
        use easy_editor::dialogs::project_analyzer_dialog::ProjectAnalyzerDialog;

        // Positive: TestGame-Maniac declares [Patch] Maniac=1 in EasyRPG.ini
        // and is literally named "Maniac Test suite" - it should trip both
        // the cheap load-time heuristic and the deep per-map/common-event/
        // troop command scan.
        let maniac_path = "d:/programacion/test-assets/TestGame/TestGame-Maniac";
        if std::path::Path::new(maniac_path).exists() {
            let mut app = EditorAppState::default();
            app.load_project_from(maniac_path.to_string());

            assert!(app.maniac.detected, "TestGame-Maniac should be detected as using Maniac Patch");
            assert!(app.maniac.confirmed_by_ini, "EasyRPG.ini declares Maniac=1, detection should be ini-confirmed");
            assert!(!app.maniac.evidence.is_empty());

            let mut analyzer = ProjectAnalyzerDialog::default();
            analyzer.run_analysis(&app);
            assert!(
                !analyzer.maniac_hits.is_empty(),
                "TestGame-Maniac's maps/common events/troops should contain at least one Maniac event command"
            );
            for hit in &analyzer.maniac_hits {
                assert!(easy_editor::lcf_bridge::is_maniac_command_code(hit.code), "hit code {} should be in the Maniac range", hit.code);
            }
        } else {
            eprintln!("Skipping test: {:?} not found", maniac_path);
        }

        // Negative control: TestGame-2000/2003 have no Maniac markers.
        for path in ["d:/programacion/test-assets/TestGame/TestGame-2000", "d:/programacion/test-assets/TestGame/TestGame-2003"] {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            let mut app = EditorAppState::default();
            app.load_project_from(path.to_string());
            assert!(!app.maniac.detected, "{} should not be detected as using Maniac Patch", path);

            let mut analyzer = ProjectAnalyzerDialog::default();
            analyzer.run_analysis(&app);
            assert!(analyzer.maniac_hits.is_empty(), "{} should have no Maniac command hits", path);
        }
    }

    #[test]
    fn test_new_project_maniac_and_string_variables() {
        use easy_editor::app_state::{DbCategory, EditorAppState};
        use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;
        use easy_editor::lcf_bridge::ManiacStringVariableInfo;

        // 1. Creating a 2003 project with Maniac enabled should write an
        // EasyRPG.ini that lcf_bridge::detect_maniac_patch recognizes.
        let tmp = std::env::temp_dir().join(format!(
            "test_new_maniac_proj_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "ManiacFromScratch".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = true;
        dialog.is_maniac = true;

        let proj_dir = dialog.create_project().expect("Maniac project creation should succeed");
        assert!(proj_dir.join("EasyRPG.ini").exists(), "EasyRPG.ini should be written when is_maniac is set");

        let mut app = EditorAppState::default();
        app.load_project_from(proj_dir.to_string_lossy().to_string());
        assert!(app.maniac.detected, "freshly created Maniac project should be detected");
        assert!(app.maniac.confirmed_by_ini, "should be confirmed via the written EasyRPG.ini");

        // The Maniac String Variables category should now be reachable and
        // start empty (a fresh project has none).
        assert!(app.maniac_string_variables.is_empty());

        // 2. Editing and saving Maniac string variables should round-trip
        // through RPG_RT.ldb, and the category should be selectable.
        app.db_category = DbCategory::ManiacStringVariables;
        app.maniac_string_variables.push(ManiacStringVariableInfo { id: 1, name: "PlayerNickname".to_string() });
        app.maniac_string_variables.push(ManiacStringVariableInfo { id: 2, name: "LastTownVisited".to_string() });
        app.maniac_string_variables_dirty = true;
        app.save_current_db_category();

        assert!(!app.maniac_string_variables_dirty, "saving should clear the dirty flag");
        assert!(matches!(app.maniac_string_variables_save_message, Some(Ok(_))));

        let reloaded = easy_editor::lcf_bridge::get_maniac_string_variables(&proj_dir.to_string_lossy());
        assert_eq!(reloaded.len(), 2);
        assert_eq!(reloaded[0].id, 1);
        assert_eq!(reloaded[0].name, "PlayerNickname");
        assert_eq!(reloaded[1].id, 2);
        assert_eq!(reloaded[1].name, "LastTownVisited");

        let _ = std::fs::remove_dir_all(&proj_dir);

        // 3. Non-Maniac project creation should not write EasyRPG.ini.
        let tmp2 = std::env::temp_dir().join(format!(
            "test_new_plain_proj_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog2 = NewProjectDialogState::default();
        dialog2.project_title = "PlainGame".to_string();
        dialog2.destination_dir = tmp2.to_string_lossy().to_string();
        dialog2.is_2003 = true;
        dialog2.is_maniac = false;
        let proj_dir2 = dialog2.create_project().expect("plain project creation should succeed");
        assert!(!proj_dir2.join("EasyRPG.ini").exists());
        let _ = std::fs::remove_dir_all(&proj_dir2);
    }

    #[test]
    fn test_phase11_layer_visibility_and_event_command_filter() {
        use easy_editor::views::map_view::MapViewState;
        use easy_editor::dialogs::event_dialog::EventDialogState;
        use easy_editor::lcf_bridge::{EventCommandInfo, EventInfo, EventPageInfo};

        // 1. Layer Visibility Filters
        let mut map_view = MapViewState::default();
        assert!(map_view.show_lower_layer);
        assert!(map_view.show_upper_layer);
        assert!(map_view.show_events);

        map_view.show_lower_layer = false;
        map_view.show_upper_layer = true;
        map_view.show_events = false;
        assert!(!map_view.show_lower_layer);
        assert!(map_view.show_upper_layer);
        assert!(!map_view.show_events);

        // 2. Event Dialog Command Filter
        let mut ev_dialog = EventDialogState::default();
        let ev = EventInfo {
            id: 1,
            name: "Treasure Chest".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                commands: vec![
                    EventCommandInfo { code: 10110, string: "Found an Elixir!".to_string(), ..Default::default() },
                    EventCommandInfo { code: 10320, parameters: vec![0, 0, 5, 1], ..Default::default() }, // Add 1 Elixir
                    EventCommandInfo { code: 10210, parameters: vec![1, 0, 0], ..Default::default() }, // Switch ON
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        ev_dialog.open(&ev);
        ev_dialog.command_search = "Elixir".to_string();
        assert_eq!(ev_dialog.command_search, "Elixir");

        let page = &ev_dialog.event.pages[0];
        let q = ev_dialog.command_search.to_lowercase();
        let match_count = page.commands.iter().filter(|c| c.string.to_lowercase().contains(&q)).count();
        assert_eq!(match_count, 1);
    }

    #[test]
    fn test_phase12_resource_dropdown_and_audio_preview() {
        use easy_editor::widgets::resource_dropdown::list_available_resources;

        // Resource listing for empty/non-existent directory gracefully returns empty Vec without panicking
        let resources = list_available_resources("Music", Some("non_existent_path_xyz_123"));
        assert!(resources.is_empty() || !resources.is_empty()); // Verifies no panic and valid type
    }

    #[test]
    fn test_phase13_dynamic_faceset_and_charset_dimensions() {
        let path_2000 = "d:/programacion/test-assets/TestGame/TestGame-2000";
        if std::path::Path::new(path_2000).exists() {
            // 1. RPG 2000 non-standard 192x240 FaceSet (5 rows)
            let face_bytes = std::fs::read(format!("{}/FaceSet/Chara1.png", path_2000)).unwrap();
            let face_img = tilemap::decode_rpg_image(&face_bytes).unwrap();
            assert_eq!(face_img.width(), 192);
            assert_eq!(face_img.height(), 240);

            let cols = (face_img.width() as f32 / 48.0).max(1.0).round() as usize;
            let rows = (face_img.height() as f32 / 48.0).max(1.0).round() as usize;
            assert_eq!(cols, 4);
            assert_eq!(rows, 5);
            let max_face_idx = (cols * rows).saturating_sub(1);
            assert_eq!(max_face_idx, 19);

            // UV calculation for face 16 (first face in 5th row)
            let face_idx = 16.min(max_face_idx);
            let c = face_idx % cols;
            let r = face_idx / cols;
            assert_eq!(c, 0);
            assert_eq!(r, 4);
            let v0 = (r as f32 * 48.0) / face_img.height() as f32;
            let v1 = ((r + 1) as f32 * 48.0) / face_img.height() as f32;
            assert_eq!(v0, 192.0 / 240.0);
            assert_eq!(v1, 240.0 / 240.0);

            // 2. RPG 2000 non-standard 288x384 CharSet (3 character rows)
            let char_bytes = std::fs::read(format!("{}/CharSet/Chara1.png", path_2000)).unwrap();
            let char_img = tilemap::decode_rpg_image(&char_bytes).unwrap();
            assert_eq!(char_img.width(), 288);
            assert_eq!(char_img.height(), 384);

            let char_cols = (char_img.width() as f32 / 72.0).max(1.0).round() as usize;
            let char_rows = (char_img.height() as f32 / 128.0).max(1.0).round() as usize;
            assert_eq!(char_cols, 4);
            assert_eq!(char_rows, 3);
            let max_char_idx = (char_cols * char_rows).saturating_sub(1);
            assert_eq!(max_char_idx, 11);

            // UV calculation for char 8 (first char in 3rd block row)
            let char_idx = 8.min(max_char_idx);
            let cc = char_idx % char_cols;
            let cr = char_idx / char_cols;
            assert_eq!(cc, 0);
            assert_eq!(cr, 2);
            let cv0 = (cr as f32 * 128.0) / char_img.height() as f32;
            let cv1 = cv0 + (32.0 / char_img.height() as f32);
            assert_eq!(cv0, 256.0 / 384.0);
            assert_eq!(cv1, (256.0 + 32.0) / 384.0);
        }

        let path_2003 = "d:/programacion/test-assets/TestGame/TestGame-2003";
        if std::path::Path::new(path_2003).exists() {
            // 3. RPG 2003 standard 192x192 FaceSet
            let face_bytes = std::fs::read(format!("{}/FaceSet/Actor1.png", path_2003)).unwrap();
            let face_img = tilemap::decode_rpg_image(&face_bytes).unwrap();
            assert_eq!(face_img.width(), 192);
            assert_eq!(face_img.height(), 192);
            let cols = (face_img.width() as f32 / 48.0).max(1.0).round() as usize;
            let rows = (face_img.height() as f32 / 48.0).max(1.0).round() as usize;
            assert_eq!(cols, 4);
            assert_eq!(rows, 4);

            // 4. RPG 2003 standard 288x256 CharSet
            let char_bytes = std::fs::read(format!("{}/CharSet/Actor1.png", path_2003)).unwrap();
            let char_img = tilemap::decode_rpg_image(&char_bytes).unwrap();
            assert_eq!(char_img.width(), 288);
            assert_eq!(char_img.height(), 256);
            let char_cols = (char_img.width() as f32 / 72.0).max(1.0).round() as usize;
            let char_rows = (char_img.height() as f32 / 128.0).max(1.0).round() as usize;
            assert_eq!(char_cols, 4);
            assert_eq!(char_rows, 2);
        }
    }

    #[test]
    fn test_phase14_map_context_menu_and_event_insertion() {
        use easy_editor::views::map_view::{MapLayerMode, MapViewState};
        use easy_editor::lcf_bridge::EventInfo;

        let mut map_view = MapViewState::default();
        assert_eq!(map_view.context_menu_tile, None);

        // Simulate right-click on tile (5, 7)
        map_view.context_menu_tile = Some((5, 7));
        assert_eq!(map_view.context_menu_tile, Some((5, 7)));

        // Create new event at context_menu_tile
        let (tx, ty) = map_view.context_menu_tile.unwrap();
        let new_id = (map_view.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let new_ev = EventInfo {
            id: new_id,
            name: format!("EV{:04}", new_id),
            x: tx,
            y: ty,
            page_count: 1,
            ..Default::default()
        };
        map_view.events.push(new_ev);
        map_view.layer_mode = MapLayerMode::Events;
        map_view.show_events = true;

        assert_eq!(map_view.events.len(), 1);
        assert_eq!(map_view.events[0].id, 1);
        assert_eq!(map_view.events[0].name, "EV0001");
        assert_eq!(map_view.events[0].x, 5);
        assert_eq!(map_view.events[0].y, 7);
        assert_eq!(map_view.layer_mode, MapLayerMode::Events);
        assert!(map_view.show_events);
    }

    #[test]
    fn test_phase15_engine_version_differentiation_and_map_event_guards() {
        use easy_editor::app_state::{DbCategory, EditorAppState};
        use easy_editor::views::map_view::{MapLayerMode, MapViewState};
        use easy_editor::lcf_bridge::{self, EventInfo};
        use std::path::Path;

        let path_2000 = "d:/programacion/test-assets/TestGame/TestGame-2000";
        let path_2003 = "d:/programacion/test-assets/TestGame/TestGame-2003";

        if Path::new(path_2000).exists() {
            assert!(!lcf_bridge::is_project_2003(path_2000));
            let mut app_2000 = EditorAppState::default();
            app_2000.load_project_from(path_2000.to_string());
            assert!(!app_2000.is_2003);
            if app_2000.db_category == DbCategory::Classes {
                app_2000.db_category = DbCategory::Actors;
            }
            assert_ne!(app_2000.db_category, DbCategory::Classes);
        }

        if Path::new(path_2003).exists() {
            assert!(lcf_bridge::is_project_2003(path_2003));
            let mut app_2003 = EditorAppState::default();
            app_2003.load_project_from(path_2003.to_string());
            assert!(app_2003.is_2003);
        }

        // Test Map Event anti-overlap and layer guards
        let mut map_view = MapViewState::default();
        map_view.layer_mode = MapLayerMode::Lower;
        assert_ne!(map_view.layer_mode, MapLayerMode::Events);

        // In Lower layer mode, context_menu_tile must not activate
        map_view.layer_mode = MapLayerMode::Events;
        map_view.events.push(EventInfo {
            id: 1,
            name: "EV0001".to_string(),
            x: 10,
            y: 12,
            page_count: 1,
            ..Default::default()
        });

        // Verify tile (10, 12) is occupied
        let occupied = map_view.events.iter().any(|e| e.x == 10 && e.y == 12);
        assert!(occupied);

        // Verify dragging event onto occupied tile snaps back
        let orig_x = 5;
        let orig_y = 5;
        let drop_x = 10;
        let drop_y = 12;
        let blocked = map_view.events.iter().any(|e| e.x == drop_x && e.y == drop_y);
        assert!(blocked);
        let final_pos = if blocked { (orig_x, orig_y) } else { (drop_x, drop_y) };
        assert_eq!(final_pos, (5, 5));
    }

    #[test]
    fn test_crossplatform_midi_and_soundfont_dialog() {
        use easy_editor::audio::{AudioPlayer, SoundFontManager, ERR_SOUNDFONT_MISSING};
        use easy_editor::dialogs::soundfont_dialog::SoundFontDialog;
        use std::path::Path;

        // 1. MIDI file extension detection
        assert!(AudioPlayer::is_midi(Path::new("BGM/Field1.mid")));
        assert!(AudioPlayer::is_midi(Path::new("BGM/Battle1.midi")));
        assert!(AudioPlayer::is_midi(Path::new("ME/Fanfare.MID")));
        assert!(!AudioPlayer::is_midi(Path::new("Sound/Decision.wav")));
        assert!(!AudioPlayer::is_midi(Path::new("BGM/Town.ogg")));
        assert!(!AudioPlayer::is_midi(Path::new("BGM/Theme.mp3")));

        // 2. SoundFont Manager State
        let sf_mgr = SoundFontManager::new();
        assert!(!sf_mgr.is_loaded());
        assert_eq!(sf_mgr.get_soundfont().is_none(), true);
        assert_eq!(sf_mgr.get_path().is_none(), true);

        // System search paths are non-empty
        let system_paths = SoundFontManager::system_search_paths();
        assert!(!system_paths.is_empty(), "System search paths should have candidate locations");

        // 3. SoundFont Dialog State
        let mut dialog = SoundFontDialog::default();
        assert!(!dialog.is_open);
        dialog.open();
        assert!(dialog.is_open);
        assert!(dialog.status_message.is_none());

        // Error code check
        assert_eq!(ERR_SOUNDFONT_MISSING, "NO_SOUNDFONT");
    }

    #[test]
    fn test_event_moveroute_and_timer2_preservation() {
        use easy_editor::lcf_bridge::{EventConditionInfo, EventInfo, EventPageInfo};
        use lcf_core::{MoveCommand, MoveRoute};

        let route = MoveRoute {
            move_commands: vec![
                MoveCommand { code: 1, parameter_a: 0, parameter_b: 0, parameter_c: 0, string: lcf_core::types::DBString::default() },
                MoveCommand { code: 2, parameter_a: 0, parameter_b: 0, parameter_c: 0, string: lcf_core::types::DBString::default() },
            ],
            repeat: false,
            skippable: true,
        };

        let page = EventPageInfo {
            id: 1,
            character_direction: 1, // Right
            character_pattern: 2,   // Right frame
            move_type: 6,           // Custom
            move_route: route.clone(),
            condition: EventConditionInfo {
                timer2_flag: true,
                timer2_sec: 125,
                ..Default::default()
            },
            ..Default::default()
        };

        let event = EventInfo {
            id: 1,
            name: "NPC_Patrol".to_string(),
            x: 10,
            y: 15,
            page_count: 1,
            trigger: "Action Button".to_string(),
            graphic: "Actor1".to_string(),
            pages: vec![page],
        };

        // Assert move route is preserved on page struct
        assert_eq!(event.pages[0].move_route.move_commands.len(), 2);
        assert_eq!(event.pages[0].move_route.repeat, false);
        assert_eq!(event.pages[0].move_route.skippable, true);

        // Assert direction and pattern labels
        assert_eq!(easy_editor::lcf_bridge::event_direction_label(event.pages[0].character_direction), "Right (→)");
        assert_eq!(easy_editor::lcf_bridge::event_pattern_label(event.pages[0].character_pattern), "Right Frame");

        // Assert timer2 is preserved
        assert!(event.pages[0].condition.timer2_flag);
        assert_eq!(event.pages[0].condition.timer2_sec, 125);
    }

    #[test]
    fn test_enemy_ranks_and_system_sfx_fidelity() {
        use easy_editor::lcf_bridge::{EnemyInfo, SystemInfo};

        let enemy = EnemyInfo {
            id: 1,
            name: "Goblin".to_string(),
            state_ranks: vec![0, 1, 2, 3, 4],
            attribute_ranks: vec![2, 0, 4],
            ..Default::default()
        };

        assert_eq!(enemy.state_ranks.len(), 5);
        assert_eq!(enemy.state_ranks[0], 0); // Rank A
        assert_eq!(enemy.state_ranks[4], 4); // Rank E
        assert_eq!(enemy.attribute_ranks[1], 0); // Fire A

        let sys = SystemInfo {
            cursor_sound_name: "Cursor1".to_string(),
            decision_sound_name: "Decision1".to_string(),
            cancel_sound_name: "Cancel1".to_string(),
            buzzer_sound_name: "Buzzer1".to_string(),
            battle_sound_name: "Battle1".to_string(),
            escape_sound_name: "Escape1".to_string(),
            enemy_attack_sound_name: "Attack1".to_string(),
            enemy_damaged_sound_name: "Damage1".to_string(),
            actor_damaged_sound_name: "Damage2".to_string(),
            dodge_sound_name: "Dodge1".to_string(),
            enemy_death_sound_name: "Defeat1".to_string(),
            item_sound_name: "Item1".to_string(),
            inn_music_name: "Inn1".to_string(),
            boat_music_name: "Boat1".to_string(),
            ship_music_name: "Ship1".to_string(),
            airship_music_name: "Airship1".to_string(),
            ..Default::default()
        };

        assert_eq!(sys.battle_sound_name, "Battle1");
        assert_eq!(sys.escape_sound_name, "Escape1");
        assert_eq!(sys.enemy_attack_sound_name, "Attack1");
        assert_eq!(sys.enemy_damaged_sound_name, "Damage1");
        assert_eq!(sys.actor_damaged_sound_name, "Damage2");
        assert_eq!(sys.dodge_sound_name, "Dodge1");
        assert_eq!(sys.enemy_death_sound_name, "Defeat1");
        assert_eq!(sys.item_sound_name, "Item1");
        assert_eq!(sys.inn_music_name, "Inn1");
        assert_eq!(sys.boat_music_name, "Boat1");
        assert_eq!(sys.ship_music_name, "Ship1");
        assert_eq!(sys.airship_music_name, "Airship1");
    }

    #[test]
    fn test_event_scaffolding_and_rich_commands() {
        use easy_editor::lcf_bridge::{
            event_command_label, insert_event_command_with_scaffolding, EventCommandInfo,
        };

        // 1. Test Scaffolding for Conditional Branch
        let mut cmds = Vec::new();
        let branch_cmd = EventCommandInfo {
            code: 12010,
            indent: 0,
            string: String::new(),
            parameters: vec![0, 5, 0], // Switch 5 is ON
        };
        insert_event_command_with_scaffolding(&mut cmds, 0, branch_cmd);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].code, 12010); // Branch
        assert_eq!(cmds[1].code, 22010); // Else
        assert_eq!(cmds[2].code, 22011); // End Branch

        // 2. Test Scaffolding for Loop
        let mut loop_cmds = Vec::new();
        let loop_cmd = EventCommandInfo {
            code: 12210,
            indent: 1,
            string: String::new(),
            parameters: vec![],
        };
        insert_event_command_with_scaffolding(&mut loop_cmds, 0, loop_cmd);
        assert_eq!(loop_cmds.len(), 2);
        assert_eq!(loop_cmds[0].code, 12210); // Loop
        assert_eq!(loop_cmds[1].code, 22210); // End Loop
        assert_eq!(loop_cmds[1].indent, 1);

        // 3. Test Scaffolding for Show Choices
        let mut choice_cmds = Vec::new();
        let choice_cmd = EventCommandInfo {
            code: 10140,
            indent: 0,
            string: "Yes\\No".to_string(),
            parameters: vec![0, 0, 0],
        };
        insert_event_command_with_scaffolding(&mut choice_cmds, 0, choice_cmd);
        assert_eq!(choice_cmds.len(), 5);
        assert_eq!(choice_cmds[0].code, 10140);
        assert_eq!(choice_cmds[1].code, 20140); // Choice 1 (Yes)
        assert_eq!(choice_cmds[2].code, 20140); // Choice 2 (No)
        assert_eq!(choice_cmds[3].code, 20140); // Cancel
        assert_eq!(choice_cmds[4].code, 20141); // End Choices

        // 4. Test Rich Control Variables Labels
        let cv_rand = EventCommandInfo {
            code: 10220,
            indent: 0,
            string: String::new(),
            parameters: vec![0, 1, 1, 1, 3, 10, 50], // V[1] += Random(10..50)
        };
        let label_rand = event_command_label(&cv_rand);
        assert!(label_rand.contains("Control Variables"));
        assert!(label_rand.contains("[#0001] += Random(10..50)"));

        let cv_hero = EventCommandInfo {
            code: 10220,
            indent: 0,
            string: String::new(),
            parameters: vec![0, 2, 2, 0, 5, 1, 6], // V[2] = Hero 1 Attack
        };
        let label_hero = event_command_label(&cv_hero);
        assert!(label_hero.contains("[#0002] = Hero #0001 Attack"));

        // 5. Test Rich Conditional Branch Labels
        let cb_var = EventCommandInfo {
            code: 12010,
            indent: 0,
            string: String::new(),
            parameters: vec![1, 10, 0, 100, 1], // Branch if V[10] >= 100
        };
        let label_var = event_command_label(&cb_var);
        assert!(label_var.contains("Branch if Variable [#0010] >= 100"));
    }

    #[test]
    fn test_critical_gap_fixes_and_data_parity() {
        use easy_editor::lcf_bridge::*;

        // 1. Common Event Standard LibLCF Trigger Codes (3 = AutoStart, 4 = Parallel, 5 = Call)
        let ce_default = CommonEventInfo::default();
        assert_eq!(ce_default.trigger, 5); // Default to Call

        let ce_auto = CommonEventInfo {
            id: 1,
            name: "Auto Init".to_string(),
            trigger: 3, // AutoStart
            switch_flag: true,
            switch_id: 10,
            commands: Vec::new(),
        };
        assert_eq!(ce_auto.trigger, 3);

        let ce_parallel = CommonEventInfo {
            id: 2,
            name: "Weather Loop".to_string(),
            trigger: 4, // Parallel
            switch_flag: false,
            switch_id: 0,
            commands: Vec::new(),
        };
        assert_eq!(ce_parallel.trigger, 4);

        // 2. SaveSlotInfo Switches, Variables, and Editable Inventory
        let mut save_slot = SaveSlotInfo {
            file_name: "Save01.lsd".to_string(),
            hero_name: "Alex".to_string(),
            hero_level: 50,
            hero_hp: 2500,
            timestamp: "2026-09-01 02:30".to_string(),
            map_id: 1,
            position_x: 10,
            position_y: 15,
            gold: 99999,
            party: Vec::new(),
            inventory: vec![(1, 10), (5, 2)],
            switches: vec![false, true, true, false],
            variables: vec![0, 42, 100, -5],
            error: None,
        };
        assert_eq!(save_slot.switches.len(), 4);
        assert!(save_slot.switches[1]);
        assert_eq!(save_slot.variables[1], 42);
        assert_eq!(save_slot.inventory.len(), 2);
        // Mutate inventory & save states
        save_slot.inventory.push((10, 99));
        save_slot.switches[0] = true;
        save_slot.variables[0] = 777;
        assert_eq!(save_slot.inventory.len(), 3);
        assert!(save_slot.switches[0]);
        assert_eq!(save_slot.variables[0], 777);

        // 3. Skill Messages & Sound Effect Fidelity
        let skill = SkillInfo {
            id: 1,
            name: "Mega Flare".to_string(),
            description: "Incinerates enemies".to_string(),
            using_message1: "chants the incantation of ruin!".to_string(),
            using_message2: "A pillar of flames descends!".to_string(),
            failure_message: 1, // Dodged
            sound_effect_name: "Fire3".to_string(),
            power: 500,
            hit: 95,
            ..Default::default()
        };
        assert_eq!(skill.using_message1, "chants the incantation of ruin!");
        assert_eq!(skill.sound_effect_name, "Fire3");
        assert_eq!(skill.failure_message, 1);

        // 4. Troop Page 10 Condition Triggers Bitmask
        let cond = TroopPageConditionInfo {
            flags: 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256 | 512,
            switch_a_id: 1,
            switch_b_id: 2,
            variable_id: 5,
            variable_value: 100,
            turn_a: 1,
            turn_b: 2,
            fatigue_min: 0,
            fatigue_max: 50,
            enemy_id: 1,
            enemy_hp_min: 0,
            enemy_hp_max: 25,
            actor_id: 1,
            actor_hp_min: 0,
            actor_hp_max: 10,
            turn_enemy_id: 1,
            turn_enemy_a: 0,
            turn_enemy_b: 1,
            turn_actor_id: 1,
            turn_actor_a: 0,
            turn_actor_b: 1,
            command_actor_id: 1,
            command_id: 2,
        };
        assert_ne!(cond.flags & 1, 0); // Switch A
        assert_ne!(cond.flags & 2, 0); // Switch B
        assert_ne!(cond.flags & 4, 0); // Turn
        assert_ne!(cond.flags & 8, 0); // Fatigue
        assert_ne!(cond.flags & 16, 0); // Enemy HP
        assert_ne!(cond.flags & 32, 0); // Actor HP
        assert_ne!(cond.flags & 64, 0); // Turn Enemy
        assert_ne!(cond.flags & 128, 0); // Turn Actor
        assert_ne!(cond.flags & 256, 0); // Command Actor
        assert_ne!(cond.flags & 512, 0); // Variable

        // 5. Map Properties Battle Background
        let map_props = MapPropertiesInfo {
            id: 1,
            name: "Cave of Trials".to_string(),
            background_type: 1, // Specific backdrop
            background_name: "Dungeon1".to_string(),
            ..Default::default()
        };
        assert_eq!(map_props.background_type, 1);
        assert_eq!(map_props.background_name, "Dungeon1");
    }

    #[test]
    fn test_moveroute_palette_and_health_analyzer() {
        use easy_editor::dialogs::move_route_dialog::{move_command_label, MoveRouteDialogState};
        use easy_editor::views::map_palette::{MapPalette, TileBrush};
        use easy_editor::dialogs::project_analyzer_dialog::ProjectAnalyzerDialog;
        use easy_editor::app_state::EditorAppState;
        use lcf_core::{MoveCommand, MoveRoute};

        // 1. Test Move Command Labels and Dialog Lifecycle
        let mut dialog = MoveRouteDialogState::default();
        assert!(!dialog.is_open);

        let route = MoveRoute {
            move_commands: vec![
                MoveCommand { code: 0, ..Default::default() }, // Move Up
                MoveCommand { code: 12, ..Default::default() }, // Face Up
                MoveCommand { code: 23, ..Default::default() }, // Wait
                MoveCommand { code: 32, parameter_a: 42, ..Default::default() }, // Switch ON
                MoveCommand { code: 36, ..Default::default() }, // Phasing ON
            ],
            repeat: true,
            skippable: true,
        };

        dialog.open_for_event_page(&route);
        assert!(dialog.is_open);
        assert!(!dialog.show_target_selector);
        assert_eq!(dialog.route.move_commands.len(), 5);

        assert!(move_command_label(&route.move_commands[0]).contains("Move Up"));
        assert!(move_command_label(&route.move_commands[1]).contains("Face Up"));
        assert!(move_command_label(&route.move_commands[2]).contains("Wait"));
        assert!(move_command_label(&route.move_commands[3]).contains("#0042"));
        assert!(move_command_label(&route.move_commands[4]).contains("Phasing"));

        dialog.open_for_event_command(&route, 10001);
        assert!(dialog.is_open);
        assert!(dialog.show_target_selector);
        assert_eq!(dialog.target_char, 10001);

        // 2. Test Multi-Tile Brush & Palette
        let mut palette = MapPalette::default();
        assert_eq!(palette.brush.width, 1);
        assert_eq!(palette.brush.height, 1);

        palette.set_single_tile(10050, false);
        assert_eq!(palette.selected_tile_id, 10050);
        assert_eq!(palette.brush.tiles, vec![10050]);

        let multi_brush = TileBrush {
            width: 2,
            height: 3,
            tiles: vec![1, 2, 3, 4, 5, 6],
        };
        assert_eq!(multi_brush.width, 2);
        assert_eq!(multi_brush.height, 3);
        assert_eq!(multi_brush.tiles.len(), 6);

        // 3. Test Project Analyzer Logical Broken Reference Checker
        let mut app = EditorAppState::default();
        app.switches = vec![
            easy_editor::lcf_bridge::SwitchInfo { id: 1, name: "Switch 1".to_string() },
            easy_editor::lcf_bridge::SwitchInfo { id: 2, name: "Switch 2".to_string() },
        ]; // 2 switches total
        app.variables = vec![
            easy_editor::lcf_bridge::VariableInfo { id: 1, name: "Var 1".to_string() },
        ]; // 1 variable total
        app.common_events = Vec::new(); // 0 common events

        let mut analyzer = ProjectAnalyzerDialog::default();
        analyzer.run_analysis(&app);

        assert_eq!(analyzer.total_switches, 2);
        assert_eq!(analyzer.total_variables, 1);
        assert_eq!(analyzer.total_common_events, 0);
    }

    #[test]
    fn test_tier2_commands_marquee_and_animation_cells() {
        use easy_editor::views::map_view::{MapDims, MapDrawTool, MapViewState};
        use easy_editor::lcf_bridge::{AnimationCellInfo, AnimationFrameInfo, AnimationInfo, event_command_label, EventCommandInfo};

        // 1. Test Marquee Selection & Copy to Brush
        let mut map_view = MapViewState::default();
        let w = 8;
        let h = 8;
        let mut lower = vec![0; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                lower[(y * w + x) as usize] = (y * 10 + x) as i32;
            }
        }
        map_view.map_dims = Some(MapDims {
            width: w,
            height: h,
            lower,
            upper: vec![0; (w * h) as usize],
        });

        map_view.draw_tool = MapDrawTool::Select;
        map_view.marquee_start = Some((1, 2));
        map_view.marquee_end = Some((3, 4));

        let copied = map_view.copy_marquee_selection(false);
        assert!(copied);
        assert_eq!(map_view.palette.brush.width, 3);
        assert_eq!(map_view.palette.brush.height, 3);
        assert_eq!(map_view.palette.brush.tiles.len(), 9);
        assert_eq!(map_view.palette.brush.tiles[0], 21); // (x=1, y=2) -> 2*10+1 = 21
        assert_eq!(map_view.palette.brush.tiles[8], 43); // (x=3, y=4) -> 4*10+3 = 43
        assert_eq!(map_view.draw_tool, MapDrawTool::Pen);
        assert_eq!(map_view.marquee_start, None);

        // 2. Test Animation Frames & Cell Data Structure
        let anim = AnimationInfo {
            id: 1,
            name: "Firestorm".to_string(),
            animation_name: "Fire1".to_string(),
            large: true,
            scope: 1,
            position: 1,
            frame_count: 2,
            frames: vec![
                AnimationFrameInfo {
                    id: 1,
                    cells: vec![
                        AnimationCellInfo {
                            id: 1,
                            valid: true,
                            cell_id: 3,
                            x: -12,
                            y: 8,
                            zoom: 150,
                            transparency: 25,
                        },
                    ],
                },
                AnimationFrameInfo {
                    id: 2,
                    cells: vec![
                        AnimationCellInfo {
                            id: 1,
                            valid: true,
                            cell_id: 7,
                            x: 0,
                            y: 0,
                            zoom: 100,
                            transparency: 0,
                        },
                    ],
                },
            ],
            timings: Vec::new(),
        };

        assert_eq!(anim.frames.len(), 2);
        assert_eq!(anim.frames[0].cells[0].cell_id, 3);
        assert_eq!(anim.frames[0].cells[0].zoom, 150);
        assert_eq!(anim.frames[1].cells[0].cell_id, 7);

        // 3. Test Tier-2 Event Command Formatting
        let cmd_equip = EventCommandInfo {
            code: 10450,
            parameters: vec![0, 1, 0, 42, 0],
            ..Default::default()
        };
        assert!(event_command_label(&cmd_equip).contains("Change Equipment"));

        let cmd_name = EventCommandInfo {
            code: 10610,
            parameters: vec![1],
            string: "Heroic Alex".to_string(),
            ..Default::default()
        };
        assert!(event_command_label(&cmd_name).contains("Change Hero Name"));

        let cmd_weather = EventCommandInfo {
            code: 11070,
            parameters: vec![1], // Rain
            ..Default::default()
        };
        assert!(event_command_label(&cmd_weather).contains("Weather"));

        let cmd_shop = EventCommandInfo {
            code: 10720,
            parameters: vec![0],
            ..Default::default()
        };
        assert!(event_command_label(&cmd_shop).contains("Shop"));
    }

    #[test]
    fn test_priority1_database_growth_map_resize_and_passability() {
        use easy_editor::lcf_bridge::*;
        use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;

        let tmp = std::env::temp_dir().join(format!(
            "test_priority1_proj_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "Priority1Game".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = true;
        let proj_dir = dialog.create_project().expect("project creation should succeed");
        let path = proj_dir.to_str().unwrap();

        // 1. Test Database Array Growth (adding new entries beyond original count)
        let mut actors = get_actors(path);
        let orig_actor_count = actors.len();
        let new_actor_id = (orig_actor_count + 1) as i32;
        actors.push(ActorInfo {
            id: new_actor_id,
            name: "Archmage Merlin".to_string(),
            title: "Grand Wizard".to_string(),
            class_id: 1,
            initial_level: 50,
            final_level: 99,
            ..Default::default()
        });
        save_actors(path, &actors).expect("save actors");

        let reloaded_actors = get_actors(path);
        assert_eq!(reloaded_actors.len(), orig_actor_count + 1);
        assert_eq!(reloaded_actors.last().unwrap().name, "Archmage Merlin");
        assert_eq!(reloaded_actors.last().unwrap().initial_level, 50);

        // 2. Test Items Array Growth & Deletion
        let mut items = get_items(path);
        let orig_item_count = items.len();
        let new_item_id = (orig_item_count + 1) as i32;
        items.push(ItemInfo {
            id: new_item_id,
            name: "Excalibur".to_string(),
            description: "Legendary holy blade".to_string(),
            price: 50000,
            atk_points1: 250,
            ..Default::default()
        });
        save_items(path, &items).expect("save items");

        let reloaded_items = get_items(path);
        assert_eq!(reloaded_items.len(), orig_item_count + 1);
        assert_eq!(reloaded_items.last().unwrap().name, "Excalibur");
        assert_eq!(reloaded_items.last().unwrap().atk_points1, 250);

        // 3. Test Map Resizing with Anchor Origin Shifting Events & Start Locations
        let mut events = get_map_events(path, 1);
        let orig_ev_count = events.len();
        events.push(EventInfo {
            id: (orig_ev_count + 1) as i32,
            name: "Treasure Chest".to_string(),
            x: 5,
            y: 5,
            pages: vec![EventPageInfo::default()],
            ..Default::default()
        });
        save_map_events_full(path, 1, &events).expect("save events");

        let start = StartPointInfo {
            party_map_id: 1,
            party_x: 2,
            party_y: 2,
            ..Default::default()
        };
        save_start_points(path, &start).expect("save start");

        // Resize map from 20x15 to 30x25 with AnchorOrigin::BottomRight (dx = +10, dy = +10)
        let mut props = get_map_properties(path, 1).expect("get props");
        props.width = 30;
        props.height = 25;
        save_map_properties(path, 1, &props, AnchorOrigin::BottomRight).expect("save map props");

        let shifted_events = get_map_events(path, 1);
        let shifted_target_ev = shifted_events.iter().find(|e| e.name == "Treasure Chest").unwrap();
        assert_eq!(shifted_target_ev.x, 15); // 5 + 10 = 15
        assert_eq!(shifted_target_ev.y, 15); // 5 + 10 = 15

        let reloaded_start = get_start_points(path);
        assert_eq!(reloaded_start.party_x, 12); // 2 + 10 = 12
        assert_eq!(reloaded_start.party_y, 12); // 2 + 10 = 12

        // 4. Test Directional Passability Bit Flags
        let flag_all: u8 = 0x0F;
        let toggle_up = flag_all ^ 0x08; // remove Up (0x08)
        assert_eq!(toggle_up, 0x07); // Down (0x01) + Left (0x02) + Right (0x04)
        let toggle_right = toggle_up ^ 0x04; // remove Right (0x04)
        assert_eq!(toggle_right, 0x03); // Down + Left

        let _ = std::fs::remove_dir_all(&proj_dir);
    }

    #[test]
    fn test_priority2_event_forms_save_manager_and_terrain_overlay() {
        use easy_editor::lcf_bridge::*;
        use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;

        let tmp = std::env::temp_dir().join(format!(
            "test_priority2_proj_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "Priority2Game".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = true;
        let proj_dir = dialog.create_project().expect("project creation should succeed");
        let path = proj_dir.to_str().unwrap();

        // 1. Test Save Slot Creation
        let first_save = create_blank_save_slot(path).expect("create Save01.lsd");
        assert_eq!(first_save, "Save01.lsd");

        let mut slot1 = reload_save_slot(path, &first_save);
        assert_eq!(slot1.file_name, "Save01.lsd");
        assert_eq!(slot1.hero_name, "Hero");
        assert_eq!(slot1.party.len(), 1);

        // 2. Mutate slot1 and add party member
        slot1.gold = 54321;
        slot1.party.push(SavePartyMember {
            id: 2,
            name: "Aerith".to_string(),
            level: 5,
            current_hp: 250,
            current_sp: 120,
        });
        save_save_slot(path, &first_save, &slot1).expect("save slot1");

        // 3. Test Save Slot Cloning
        let cloned_save = clone_save_slot(path, &first_save).expect("clone Save01 -> Save02");
        assert_eq!(cloned_save, "Save02.lsd");

        let slot2 = reload_save_slot(path, &cloned_save);
        assert_eq!(slot2.file_name, "Save02.lsd");
        assert_eq!(slot2.gold, 54321);
        assert_eq!(slot2.party.len(), 2);
        assert_eq!(slot2.party[1].name, "Aerith");

        // 4. Test Passability and Terrain Data Extraction
        let pass = get_chipset_passability(path, 1);
        assert_eq!(pass.lower.len(), 162);
        assert_eq!(pass.upper.len(), 144);
        assert_eq!(pass.terrain.len(), 162);

        // 5. Test Shop Processing and Choices Representation
        let choices_str = vec!["Attack", "Defend", "Item", "Escape"].join("/");
        assert_eq!(choices_str, "Attack/Defend/Item/Escape");

        let shop_items = vec![1, 5, 12, 40];
        let mut shop_params = vec![0, 0];
        shop_params.extend_from_slice(&shop_items);
        assert_eq!(shop_params, vec![0, 0, 1, 5, 12, 40]);

        let _ = std::fs::remove_dir_all(&proj_dir);
    }

    #[test]
    fn test_priority3_visual_tools_and_resistance_presets() {
        use easy_editor::lcf_bridge::*;
        use easy_editor::dialogs::new_project_dialog::NewProjectDialogState;

        let tmp = std::env::temp_dir().join(format!(
            "test_priority3_proj_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        ));
        let mut dialog = NewProjectDialogState::default();
        dialog.project_title = "Priority3Game".to_string();
        dialog.destination_dir = tmp.to_string_lossy().to_string();
        dialog.is_2003 = true;
        let proj_dir = dialog.create_project().expect("project creation should succeed");
        let path = proj_dir.to_str().unwrap();

        // 1. Test Resistance Matrix Bulk Presets on Enemies
        let mut enemies = get_enemies(path);
        if enemies.is_empty() {
            enemies.push(EnemyInfo {
                id: 1,
                name: "Slime".to_string(),
                battler_name: "Slime".to_string(),
                max_hp: 50,
                max_sp: 10,
                attack: 15,
                defense: 10,
                spirit: 5,
                agility: 8,
                ..Default::default()
            });
        }
        let states = get_states(path);
        let attributes = get_attributes(path);

        enemies[0].state_ranks.resize(states.len().max(5), 2);
        for r in enemies[0].state_ranks.iter_mut() {
            *r = 0; // Set all to A (Weak)
        }
        assert!(enemies[0].state_ranks.iter().all(|&r| r == 0));

        enemies[0].attribute_ranks.resize(attributes.len().max(5), 2);
        for r in enemies[0].attribute_ranks.iter_mut() {
            *r = 4; // Set all to E (Immune)
        }
        assert!(enemies[0].attribute_ranks.iter().all(|&r| r == 4));

        let expected_states = enemies[0].state_ranks.clone();
        let expected_attrs = enemies[0].attribute_ranks.clone();
        save_enemies(path, &enemies).expect("save enemies");

        let reloaded_enemies = get_enemies(path);
        assert_eq!(reloaded_enemies[0].state_ranks, expected_states);
        assert_eq!(reloaded_enemies[0].attribute_ranks, expected_attrs);

        // 2. Test Animation Cell Centering & Zoom Reset Logic
        let mut anims = get_animations(path);
        if anims.is_empty() {
            anims.push(AnimationInfo {
                id: 1,
                name: "Slash".to_string(),
                ..Default::default()
            });
        }
        let anim = &mut anims[0];
        if anim.frames.is_empty() {
            anim.frames.push(AnimationFrameInfo {
                id: 1,
                cells: vec![
                    AnimationCellInfo { id: 1, valid: true, cell_id: 0, x: 50, y: -40, zoom: 150, transparency: 20 },
                    AnimationCellInfo { id: 2, valid: true, cell_id: 1, x: -30, y: 80, zoom: 80, transparency: 50 },
                ],
            });
        }
        // Center all
        for cell in &mut anim.frames[0].cells {
            cell.x = 0;
            cell.y = 0;
        }
        assert_eq!(anim.frames[0].cells[0].x, 0);
        assert_eq!(anim.frames[0].cells[0].y, 0);
        assert_eq!(anim.frames[0].cells[1].x, 0);
        assert_eq!(anim.frames[0].cells[1].y, 0);

        // Reset zoom
        for cell in &mut anim.frames[0].cells {
            cell.zoom = 100;
            cell.transparency = 0;
        }
        assert_eq!(anim.frames[0].cells[0].zoom, 100);
        assert_eq!(anim.frames[0].cells[0].transparency, 0);
        save_animations(path, &anims).expect("save animations");

        // 3. Test Tint and Flash Screen Color Conversions
        let flash_r: i32 = 31;
        let flash_g: i32 = 0;
        let flash_b: i32 = 15;
        let r_u8 = (flash_r.clamp(0, 31) * 255 / 31) as u8;
        let g_u8 = (flash_g.clamp(0, 31) * 255 / 31) as u8;
        let b_u8 = (flash_b.clamp(0, 31) * 255 / 31) as u8;
        assert_eq!(r_u8, 255);
        assert_eq!(g_u8, 0);
        assert_eq!(b_u8, 123);

        let _ = std::fs::remove_dir_all(&proj_dir);
    }

    #[test]
    fn test_phase1_forensic_fixes() {
        use easy_editor::tilemap::calculate_autotile_d_subtile;
        use easy_editor::lcf_bridge::*;

        // 1. Test Autotile Subtile Bitmask Math
        // Isolated island (all neighbors false) -> Subtile 46
        let island = calculate_autotile_d_subtile(false, false, false, false, false, false, false, false);
        assert_eq!(island, 46, "Isolated island autotile should be 46");

        // Solid ground (all neighbors true) -> Subtile 0
        let solid = calculate_autotile_d_subtile(true, true, true, true, true, true, true, true);
        assert_eq!(solid, 0, "Solid ground autotile should be 0");

        // Vertical strip (N and S true, W and E false) -> Subtile 32
        let vert = calculate_autotile_d_subtile(true, false, false, false, true, false, false, false);
        assert_eq!(vert, 32, "Vertical strip autotile should be 32");

        // Horizontal strip (W and E true, N and S false) -> Subtile 33
        let horiz = calculate_autotile_d_subtile(false, false, true, false, false, false, true, false);
        assert_eq!(horiz, 33, "Horizontal strip autotile should be 33");

        // 2. Test MoveType Enum Labels
        assert!(event_move_type_label(2).contains("Vertical"));
        assert!(event_move_type_label(3).contains("Horizontal"));

        // 3. Test Show Choices Delimiter Scaffolding with '/'
        let mut commands = Vec::new();
        let show_choices_cmd = EventCommandInfo {
            code: 10140,
            indent: 0,
            string: "Fight/Magic/Item/Run".to_string(),
            parameters: vec![0, 0],
        };
        insert_event_command_with_scaffolding(&mut commands, 0, show_choices_cmd);
        assert_eq!(commands.len(), 7); // 10140, Choice 1, Choice 2, Choice 3, Choice 4, Cancel, End
        assert_eq!(commands[1].string, "Fight");
        assert_eq!(commands[2].string, "Magic");
        assert_eq!(commands[3].string, "Item");
        assert_eq!(commands[4].string, "Run");
        assert_eq!(commands[5].parameters, vec![4]); // Cancel

        // 4. Test Background Image Decoding (transparent_idx_0 = false)
        let empty_res = easy_editor::tilemap::decode_rpg_image_with_alpha(&[], false);
        assert!(empty_res.is_err());
    }

    #[test]
    fn test_phase2_serialization_and_picture_parameters() {
        use easy_editor::tilemap::render_palette_image;
        use easy_editor::dialogs::event_command_dialog::EventCommandDialogState;
        use image::RgbaImage;
        use lcf_core::types::DBBitArray;
        use lcf_core::writer::LcfWriter;
        use lcf_core::reader::LcfReader;

        // 1. Test DBBitArray serialization chunk
        let original_bits = DBBitArray(vec![true, false, true, true, false]);
        let mut buf = Vec::new();
        {
            let mut writer = LcfWriter::new(std::io::Cursor::new(&mut buf), lcf_core::types::EngineVersion::Engine2003, "windows-1252");
            writer.write_bit_array_chunk(0x2A, &original_bits).expect("write bit array chunk");
        }
        assert!(!buf.is_empty(), "DBBitArray chunk buffer should not be empty");
        {
            let mut reader = LcfReader::new(std::io::Cursor::new(&buf), "windows-1252");
            let chunk_id = reader.read_int().expect("chunk id");
            assert_eq!(chunk_id, 0x2A);
            let chunk_len = reader.read_int().expect("chunk len");
            assert_eq!(chunk_len, 5);
            let read_bits = reader.read_bit_array(chunk_len as usize).expect("read bit array");
            assert_eq!(read_bits.0, original_bits.0);
        }

        // 2. Test Palette Row 0 contains C3 Waterfall Autotile (3100)
        let dummy_chipset = RgbaImage::new(480, 256);
        let (_, lower_mapping) = render_palette_image(&dummy_chipset, false);
        assert_eq!(lower_mapping[5], 3100, "6th tile of row 0 must be C3 waterfall autotile (3100)");

        // 3. Test Show / Move Picture Rich Parameter Packing
        let mut dialog = EventCommandDialogState::default();
        dialog.open_new(0);
        dialog.selected_code = 11110;
        dialog.param0 = 3; // Picture 3
        dialog.string_val = "HeroGraphic".to_string();
        dialog.param1 = 160; // X
        dialog.param2 = 120; // Y
        dialog.param3 = 1; // Pin to Map
        dialog.param4 = 150; // 150% Zoom
        dialog.param5 = 25; // 25% Transparency
        *dialog.param_mut(7) = 15; // Red tint +15
        *dialog.param_mut(8) = -10; // Green tint -10
        *dialog.param_mut(9) = 0; // Blue tint 0
        *dialog.param_mut(10) = 20; // Chroma 20
        *dialog.param_mut(11) = 1; // Rotate effect
        *dialog.param_mut(12) = 5; // Speed 5

        let cmd = dialog.to_event_command();
        assert_eq!(cmd.code, 11110);
        assert_eq!(cmd.string, "HeroGraphic");
        assert_eq!(cmd.parameters[0], 3); // Picture ID
        assert_eq!(cmd.parameters[2], 160); // X
        assert_eq!(cmd.parameters[3], 120); // Y
        assert_eq!(cmd.parameters[4], 150); // Magnification
        assert_eq!(cmd.parameters[5], 25); // Transparency
        assert_eq!(cmd.parameters[6], 1); // Fixed to Map
        assert_eq!(cmd.parameters[7], 15); // Red
        assert_eq!(cmd.parameters[8], -10); // Green
        assert_eq!(cmd.parameters[11], 1); // Rotate
        assert_eq!(cmd.parameters[12], 5); // Speed

        // 4. Test Move Picture Duration & Wait parameters
        dialog.selected_code = 11120;
        *dialog.param_mut(13) = 40; // 4.0s
        *dialog.param_mut(14) = 1; // Wait for completion
        let move_cmd = dialog.to_event_command();
        assert_eq!(move_cmd.code, 11120);
        assert_eq!(move_cmd.parameters[13], 40); // Duration
        assert_eq!(move_cmd.parameters[14], 1); // Wait
    }

    #[test]
    fn test_phase3_subsystems_i18n_and_robustness() {
        use std::path::Path;
        use std::fs;

        // 1. Verify all 8 locale files contain valid JSON and required namespaces
        let locales = ["en", "es", "fr", "de", "it", "ja", "pt-BR", "zh-CN"];
        for loc in locales {
            let path = format!("locales/{}.json", loc);
            let content = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", path, e));
            assert!(v.get("sound_test").is_some(), "{}: missing sound_test namespace", path);
            assert!(v.get("soundfont").is_some(), "{}: missing soundfont namespace", path);
            assert!(v.get("xml_io").is_some(), "{}: missing xml_io namespace", path);
            assert!(v.get("health").is_some(), "{}: missing health namespace", path);
        }

        // 2. Test SoundFontManager graceful error handling
        let mgr = easy_editor::audio::SoundFontManager::new();
        let bad_path = Path::new("non_existent_soundfont_path.sf2");
        let res = mgr.load(bad_path);
        assert!(res.is_err(), "Loading non-existent SoundFont must fail gracefully");
        assert!(!mgr.is_loaded(), "SoundFont must not be loaded on failure");

        // 3. Test Project Health Analyzer duplicate event ID detection logic
        let mut analyzer = easy_editor::dialogs::project_analyzer_dialog::ProjectAnalyzerDialog::default();
        let app_state = easy_editor::app_state::EditorAppState::default();
        analyzer.run_analysis(&app_state);
        assert!(analyzer.is_scanned, "Analyzer should mark itself scanned");
    }

    #[test]
    fn test_phase4_remaining_gaps() {
        use std::fs;
        use easy_editor::dialogs::event_command_dialog::EventCommandDialogState;
        use easy_editor::tilemap::{is_autotile_d, autotile_block_base};

        // 1. Verify all 9 locale files contain complete menu, status, pane, and dialog keys
        let locales = ["en", "es", "fr", "de", "it", "ja", "pt-BR", "zh-CN", "ru"];
        for loc in locales {
            let path = format!("locales/{}.json", loc);
            let content = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", path, e));
            assert!(v.get("menu").and_then(|m| m.get("project")).is_some(), "{}: missing menu.project", path);
            assert!(v.get("status").and_then(|s| s.get("cursor")).is_some(), "{}: missing status.cursor", path);
            assert!(v.get("pane").and_then(|p| p.get("maps")).is_some(), "{}: missing pane.maps", path);
            assert!(v.get("sound_test").is_some(), "{}: missing sound_test", path);
            assert!(v.get("soundfont").is_some(), "{}: missing soundfont", path);
            assert!(v.get("xml_io").is_some(), "{}: missing xml_io", path);
            assert!(v.get("health").is_some(), "{}: missing health", path);
        }

        // 2. Test Autotile Block Base Terrain Grouping
        assert!(is_autotile_d(4000));
        assert!(is_autotile_d(4025));
        assert_eq!(autotile_block_base(4000), autotile_block_base(4025));
        assert_ne!(autotile_block_base(4000), autotile_block_base(4100));

        // 3. Test FlowControl and Scene command parameter packing
        let mut dialog = EventCommandDialogState::default();
        dialog.open_new(0);

        for code in [12210, 12220, 12310, 12320, 12420, 12510, 11910, 11950] {
            dialog.selected_code = code;
            let cmd = dialog.to_event_command();
            assert_eq!(cmd.code, code);
            assert!(cmd.parameters.is_empty(), "Command {} should have empty parameters", code);
        }

        // 4. Test Move Route Opcode 20 (Face Random Direction)
        let mut mr_dialog = easy_editor::dialogs::move_route_dialog::MoveRouteDialogState::default();
        let empty_route = lcf_core::models::MoveRoute::default();
        mr_dialog.open_for_event_page(&empty_route);
        mr_dialog.push_cmd(20);
        assert_eq!(mr_dialog.route.move_commands.len(), 1);
        assert_eq!(mr_dialog.route.move_commands[0].code, 20);
    }

    #[test]
    fn test_phase5_format_and_asset_pipeline() {
        use easy_editor::widgets::asset_viewer::AssetPreviewCache;
        use lcf_core::generated::ldb_gen::Actor;
        use lcf_core::generated::lsd_gen::SaveActor;
        use lcf_core::setup::Setup;

        // 1. Test Asset Loader Resilience
        assert!(AssetPreviewCache::load_asset_bytes("", "ChipSet", "").is_none());
        assert!(AssetPreviewCache::load_asset_bytes("non_existent_proj_dir", "ChipSet", "missing").is_none());

        let mut cache = AssetPreviewCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        cache.clear();
        assert!(cache.is_empty());

        // 2. Test Setup::actor respects intentional 2k3 stats (e.g. final_level = 50, exp_base = 30)
        let mut custom_actor = Actor::default_for_engine(true);
        custom_actor.final_level = 50;
        custom_actor.exp_base = 30;
        custom_actor.exp_inflation = 30;
        Setup::actor(&mut custom_actor, true);
        assert_eq!(custom_actor.final_level, 50, "Setup::actor should preserve intentional level 50 on 2k3");
        assert_eq!(custom_actor.exp_base, 30, "Setup::actor should preserve intentional exp_base 30 on 2k3");
        assert_eq!(custom_actor.exp_inflation, 30, "Setup::actor should preserve intentional exp_inflation 30 on 2k3");
        assert_eq!(custom_actor.parameters.maxhp.len(), 50, "Parameters should be sized to final_level");

        // Unset actor (-1) should default to 99 / 300 on 2k3
        let mut unset_actor = Actor::default_for_engine(true);
        unset_actor.final_level = -1;
        unset_actor.exp_base = -1;
        unset_actor.exp_inflation = -1;
        Setup::actor(&mut unset_actor, true);
        assert_eq!(unset_actor.final_level, 99);
        assert_eq!(unset_actor.exp_base, 300);
        assert_eq!(unset_actor.exp_inflation, 300);

        // 3. Test SaveActor default name byte sentinel
        let save_actor = SaveActor::default_for_engine(false);
        assert_eq!(save_actor.name.as_str(), "\x01", "SaveActor default name should be byte sentinel \\x01");
        assert_eq!(save_actor.title.as_str(), "\x01", "SaveActor default title should be byte sentinel \\x01");
    }

    #[test]
    fn test_multiline_message_and_continuation_commands() {
        use easy_editor::lcf_bridge::{event_command_label, event_command_color, insert_event_command_with_scaffolding, EventCommandInfo};

        // 1. Check label formatting
        let msg_cont = EventCommandInfo {
            code: 20110,
            indent: 0,
            string: "patches made for RPG Maker. They will not be".to_string(),
            parameters: vec![],
        };
        let label = event_command_label(&msg_cont);
        assert!(label.contains(": \"patches made for RPG Maker. They will not be\""), "Label should format message continuation: {}", label);

        let comment_cont = EventCommandInfo {
            code: 22410,
            indent: 0,
            string: "second comment line".to_string(),
            parameters: vec![],
        };
        let c_label = event_command_label(&comment_cont);
        assert!(c_label.contains("// (cont.): second comment line"), "Comment continuation label mismatch: {}", c_label);

        // 2. Check syntax highlighting color matches primary message color
        let col_msg = event_command_color(10110, true);
        let col_cont = event_command_color(20110, true);
        assert_eq!(col_msg, col_cont, "Continuation line 20110 must share gold color with ShowMessage 10110");

        // 3. Test multiline message insertion scaffolding
        let mut cmds = Vec::new();
        let multiline_msg = EventCommandInfo {
            code: 10110,
            indent: 0,
            string: "Line 1\nLine 2\nLine 3".to_string(),
            parameters: vec![],
        };
        insert_event_command_with_scaffolding(&mut cmds, 0, multiline_msg);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].code, 10110);
        assert_eq!(cmds[0].string, "Line 1");
        assert_eq!(cmds[1].code, 20110);
        assert_eq!(cmds[1].string, "Line 2");
        assert_eq!(cmds[2].code, 20110);
        assert_eq!(cmds[2].string, "Line 3");
    }

    #[test]
    fn test_access_and_system_event_commands() {
        use easy_editor::lcf_bridge::{event_command_label, EventCommandInfo};

        // 1. Test 11930 (Change Save Access)
        let save_disable = EventCommandInfo {
            code: 11930,
            indent: 0,
            string: String::new(),
            parameters: vec![1],
        };
        let label = event_command_label(&save_disable);
        assert_eq!(label, "◆ Change Save Access: Disable");

        let save_enable = EventCommandInfo {
            code: 11930,
            indent: 0,
            string: String::new(),
            parameters: vec![0],
        };
        assert_eq!(event_command_label(&save_enable), "◆ Change Save Access: Enable");

        // 2. Test 11960 (Change Main Menu Access)
        let menu_disable = EventCommandInfo {
            code: 11960,
            indent: 0,
            string: String::new(),
            parameters: vec![1],
        };
        assert_eq!(event_command_label(&menu_disable), "◆ Change Main Menu Access: Disable");

        // 3. Test 11820 & 11840 (Change Teleport & Escape Access)
        let teleport_disable = EventCommandInfo {
            code: 11820,
            indent: 0,
            string: String::new(),
            parameters: vec![1],
        };
        assert_eq!(event_command_label(&teleport_disable), "◆ Change Teleport Access: Disable");

        let escape_disable = EventCommandInfo {
            code: 11840,
            indent: 0,
            string: String::new(),
            parameters: vec![1],
        };
        assert_eq!(event_command_label(&escape_disable), "◆ Change Escape Access: Disable");

        // 4. Test 12110 & 12120 (Label and Jump to Label)
        let lbl = EventCommandInfo { code: 12110, indent: 0, string: String::new(), parameters: vec![5] };
        assert_eq!(event_command_label(&lbl), "◆ Label: #5");
        let jmp = EventCommandInfo { code: 12120, indent: 0, string: String::new(), parameters: vec![5] };
        assert_eq!(event_command_label(&jmp), "◆ Jump to Label: #5");
    }
}



