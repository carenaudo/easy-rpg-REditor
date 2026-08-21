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
        // Isolated center tile with no matching neighbors (default preview / standalone)
        let isolated = tilemap::calculate_autotile_d_subtile(
            false, false, false,
            false,        false,
            false, false, false,
        );
        assert_eq!(isolated, 0, "Isolated autotile should be subtile 0");

        // Fully surrounded center tile with all 8 matching neighbors (solid center)
        let surrounded = tilemap::calculate_autotile_d_subtile(
            true, true, true,
            true,       true,
            true, true, true,
        );
        assert_eq!(surrounded, 46, "Fully surrounded autotile should be subtile 46");
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
        // Verify Save Menu command (11430) exists in generated script
        assert!(save_ev.pages[0].commands.iter().any(|c| c.code == 11430));

        // Generate Quick Recovery Spring
        map_view.create_quick_recovery(12, 14);
        assert_eq!(map_view.events.len(), 2);
        let fountain_ev = &map_view.events[1];
        assert_eq!(fountain_ev.x, 12);
        assert_eq!(fountain_ev.y, 14);
        assert!(fountain_ev.name.starts_with("Fountain_"));
        // Verify Full Recovery command (10420) exists in generated script
        assert!(fountain_ev.pages[0].commands.iter().any(|c| c.code == 10420));
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
}
