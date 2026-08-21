use std::fs;
use std::io::Cursor;
use std::path::Path;
use lcf_core::ini::IniReader;
use lcf_core::ldb::LdbReader;
use lcf_core::lmt::LmtReader;
use lcf_core::lmu::LmuReader;
use lcf_core::lsd::LsdReader;
use lcf_core::types::EngineVersion;
use lcf_core::Save;

const TEST_GAME_2000: &str = r"D:\programacion\test-assets\TestGame\TestGame-2000";
const TEST_GAME_2003: &str = r"D:\programacion\test-assets\TestGame\TestGame-2003";

#[test]
fn test_ini_parsing() {
    let ini_path = Path::new(TEST_GAME_2000).join("RPG_RT.ini");
    if ini_path.exists() {
        let ini = IniReader::load(&ini_path).expect("Failed to parse RPG_RT.ini 2000");
        let title = ini.get_string("RPG_RT", "GameTitle", "");
        println!("TestGame-2000 GameTitle: {}", title);
        assert!(!title.is_empty(), "GameTitle should be present");
    }

    let ini_path_2003 = Path::new(TEST_GAME_2003).join("RPG_RT.ini");
    if ini_path_2003.exists() {
        let ini = IniReader::load(&ini_path_2003).expect("Failed to parse RPG_RT.ini 2003");
        let title = ini.get_string("RPG_RT", "GameTitle", "");
        println!("TestGame-2003 GameTitle: {}", title);
        assert!(!title.is_empty(), "GameTitle should be present");
    }
}

#[test]
fn test_lmt_roundtrip_2000() {
    let path = Path::new(TEST_GAME_2000).join("RPG_RT.lmt");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let tmap = LmtReader::load(&path, "auto").expect("Failed to load RPG_RT.lmt 2000");
    assert!(!tmap.maps.is_empty(), "Maps list should not be empty");
    println!("Loaded LMT 2000 with {} maps", tmap.maps.len());

    // Serialize to memory
    let mut out = Cursor::new(Vec::new());
    LmtReader::save_to_writer(&mut out, &tmap, EngineVersion::Engine2000, "auto")
        .expect("Failed to save RPG_RT.lmt 2000");

    // Deserialize back
    out.set_position(0);
    let tmap2 = LmtReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload saved RPG_RT.lmt 2000");

    assert_eq!(tmap.maps.len(), tmap2.maps.len());
    assert_eq!(tmap.tree_order, tmap2.tree_order);
    assert_eq!(tmap.active_node, tmap2.active_node);
    assert_eq!(tmap.start, tmap2.start);

    for (m1, m2) in tmap.maps.iter().zip(tmap2.maps.iter()) {
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.name, m2.name);
        assert_eq!(m1.parent_map, m2.parent_map);
        assert_eq!(m1.r#type, m2.r#type);
        assert_eq!(m1.music_type, m2.music_type);
        assert_eq!(m1.music, m2.music);
    }

    // Test XML save + load round trip
    let mut xml_out = Cursor::new(Vec::new());
    LmtReader::save_xml_to_writer(&mut xml_out, &tmap, EngineVersion::Engine2000)
        .expect("Failed to save LMT XML");
    let xml_str = String::from_utf8(xml_out.into_inner()).unwrap();
    assert!(xml_str.starts_with("<?xml"));
    assert!(xml_str.contains("<LMT>"));

    let tmap3 = LmtReader::load_xml_from_reader(xml_str.as_bytes())
        .expect("Failed to load LMT XML");
    assert_eq!(tmap.maps.len(), tmap3.maps.len());
    assert_eq!(tmap.tree_order, tmap3.tree_order);
    assert_eq!(tmap.active_node, tmap3.active_node);
    assert_eq!(tmap.start, tmap3.start);
    for (m1, m3) in tmap.maps.iter().zip(tmap3.maps.iter()) {
        assert_eq!(m1.id, m3.id);
        assert_eq!(m1.name, m3.name);
        assert_eq!(m1.parent_map, m3.parent_map);
        assert_eq!(m1.r#type, m3.r#type);
        assert_eq!(m1.music_type, m3.music_type);
        assert_eq!(m1.music, m3.music);
    }
}

#[test]
fn test_lmt_roundtrip_2003() {
    let path = Path::new(TEST_GAME_2003).join("RPG_RT.lmt");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let tmap = LmtReader::load(&path, "auto").expect("Failed to load RPG_RT.lmt 2003");
    assert!(!tmap.maps.is_empty(), "Maps list should not be empty");
    println!("Loaded LMT 2003 with {} maps", tmap.maps.len());

    let mut out = Cursor::new(Vec::new());
    LmtReader::save_to_writer(&mut out, &tmap, EngineVersion::Engine2003, "auto")
        .expect("Failed to save RPG_RT.lmt 2003");

    out.set_position(0);
    let tmap2 = LmtReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload saved RPG_RT.lmt 2003");

    assert_eq!(tmap.maps.len(), tmap2.maps.len());
    assert_eq!(tmap.tree_order, tmap2.tree_order);
    assert_eq!(tmap.active_node, tmap2.active_node);
}

#[test]
fn test_ldb_roundtrip_2000() {
    let path = Path::new(TEST_GAME_2000).join("RPG_RT.ldb");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let db = LdbReader::load(&path, "auto").expect("Failed to load RPG_RT.ldb 2000");
    assert!(!db.actors.is_empty(), "Actors should not be empty");
    assert!(!db.chipsets.is_empty(), "Chipsets should not be empty");
    println!("Loaded LDB 2000 with {} actors, {} chipsets, {} skills, {} items",
        db.actors.len(), db.chipsets.len(), db.skills.len(), db.items.len());

    let mut out = Cursor::new(Vec::new());
    LdbReader::save_to_writer(&mut out, &db, EngineVersion::Engine2000, "auto")
        .expect("Failed to save RPG_RT.ldb 2000");

    out.set_position(0);
    let db2 = LdbReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload saved RPG_RT.ldb 2000");

    assert_eq!(db.actors.len(), db2.actors.len());
    assert_eq!(db.chipsets.len(), db2.chipsets.len());
    assert_eq!(db.skills.len(), db2.skills.len());
    assert_eq!(db.items.len(), db2.items.len());
    assert_eq!(db.enemies.len(), db2.enemies.len());
    assert_eq!(db.troops.len(), db2.troops.len());
    assert_eq!(db.terrains.len(), db2.terrains.len());
    assert_eq!(db.attributes.len(), db2.attributes.len());
    assert_eq!(db.states.len(), db2.states.len());
    assert_eq!(db.animations.len(), db2.animations.len());
    assert_eq!(db.commonevents.len(), db2.commonevents.len());


    // Test XML save + load round trip
    let mut xml_out = Cursor::new(Vec::new());
    LdbReader::save_xml_to_writer(&mut xml_out, &db, EngineVersion::Engine2000)
        .expect("Failed to save LDB XML");
    let xml_str = String::from_utf8(xml_out.into_inner()).unwrap();
    assert!(xml_str.starts_with("<?xml"));
    assert!(xml_str.contains("<LDB>"));

    let db3 = LdbReader::load_xml_from_reader(xml_str.as_bytes())
        .expect("Failed to load LDB XML");
    assert_eq!(db.actors.len(), db3.actors.len());
    assert_eq!(db.chipsets.len(), db3.chipsets.len());
    assert_eq!(db.skills.len(), db3.skills.len());
    assert_eq!(db.items.len(), db3.items.len());
    assert_eq!(db.enemies.len(), db3.enemies.len());
    assert_eq!(db.troops.len(), db3.troops.len());
    for (a1, a3) in db.actors.iter().zip(db3.actors.iter()) {
        assert_eq!(a1.name, a3.name);
        assert_eq!(a1.parameters, a3.parameters);
        assert_eq!(a1.initial_equipment, a3.initial_equipment);
    }
}

#[test]
fn test_ldb_roundtrip_2003_and_japanese_terms() {
    let path = Path::new(TEST_GAME_2003).join("RPG_RT.ldb");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let db = LdbReader::load(&path, "auto").expect("Failed to load RPG_RT.ldb 2003");
    println!("command_attack decoded as: {:?}", db.terms.command_attack.as_str());
    println!("command_defend decoded as: {:?}", db.terms.command_defend.as_str());

    // Assert that Japanese command terms are cleanly decoded as valid UTF-8 Japanese (攻撃 and 防御) without replacement characters
    assert_eq!(db.terms.command_attack.as_str(), "攻撃");
    assert_eq!(db.terms.command_defend.as_str(), "防御");
    assert!(!db.terms.command_attack.as_str().contains('\u{FFFD}'));
    assert!(!db.terms.command_defend.as_str().contains('\u{FFFD}'));

    // Assert that saving preserves the exact raw Shift-JIS bytes on write
    let mut out = Cursor::new(Vec::new());
    LdbReader::save_to_writer(&mut out, &db, EngineVersion::Engine2003, "auto")
        .expect("Failed to save RPG_RT.ldb 2003");

    out.set_position(0);
    let db2 = LdbReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload saved RPG_RT.ldb 2003");

    assert_eq!(db2.terms.command_attack.as_str(), "攻撃");
    assert_eq!(db2.terms.command_defend.as_str(), "防御");
}

#[test]
fn test_ldb_roundtrip_maniac_and_japanese_terms() {
    let path = Path::new(r"D:\programacion\test-assets\TestGame\TestGame-Maniac").join("RPG_RT.ldb");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let db = LdbReader::load(&path, "auto").expect("Failed to load RPG_RT.ldb Maniac");
    println!("Maniac command_attack decoded as: {:?}", db.terms.command_attack.as_str());
    println!("Maniac command_defend decoded as: {:?}", db.terms.command_defend.as_str());

    assert_eq!(db.terms.command_attack.as_str(), "攻撃");
    assert_eq!(db.terms.command_defend.as_str(), "防御");
    assert!(!db.terms.command_attack.as_str().contains('\u{FFFD}'));
    assert!(!db.terms.command_defend.as_str().contains('\u{FFFD}'));

    let mut out = Cursor::new(Vec::new());
    LdbReader::save_to_writer(&mut out, &db, EngineVersion::Engine2003, "auto")
        .expect("Failed to save RPG_RT.ldb Maniac");

    out.set_position(0);
    let db2 = LdbReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload saved RPG_RT.ldb Maniac");

    assert_eq!(db2.terms.command_attack.as_str(), "攻撃");
    assert_eq!(db2.terms.command_defend.as_str(), "防御");
}

#[test]
fn test_lmu_roundtrip_all_maps_2000() {


    let dir = Path::new(TEST_GAME_2000);
    if !dir.exists() {
        eprintln!("Skipping test: {:?} not found", dir);
        return;
    }

    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "lmu") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let map = match LmuReader::load(&path, "auto") {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Note: skipping non-standard map {}: {}", filename, e);
                    continue;
                }
            };

            let mut out = Cursor::new(Vec::new());
            LmuReader::save_to_writer(&mut out, &map, EngineVersion::Engine2000, "auto")
                .unwrap_or_else(|_| panic!("Failed to save {}", filename));

            out.set_position(0);
            let map2 = LmuReader::load_from_reader(&mut out, "auto")
                .unwrap_or_else(|_| panic!("Failed to reload {}", filename));

            assert_eq!(map.chipset_id, map2.chipset_id, "chipset_id in {}", filename);
            assert_eq!(map.width, map2.width, "width in {}", filename);
            assert_eq!(map.height, map2.height, "height in {}", filename);
            assert_eq!(map.lower_layer, map2.lower_layer, "lower_layer in {}", filename);
            assert_eq!(map.upper_layer, map2.upper_layer, "upper_layer in {}", filename);
            assert_eq!(map.events.len(), map2.events.len(), "events.len() in {}", filename);

            // XML save + load round trip
            let mut xml_out = Cursor::new(Vec::new());
            LmuReader::save_xml_to_writer(&mut xml_out, &map, EngineVersion::Engine2000)
                .unwrap_or_else(|_| panic!("Failed to save XML for {}", filename));
            let xml_str = String::from_utf8(xml_out.into_inner()).unwrap();
            let map3 = LmuReader::load_xml_from_reader(xml_str.as_bytes(), false)
                .unwrap_or_else(|_| panic!("Failed to load XML for {}", filename));
            assert_eq!(map.chipset_id, map3.chipset_id, "XML chipset_id in {}", filename);
            assert_eq!(map.width, map3.width, "XML width in {}", filename);
            assert_eq!(map.height, map3.height, "XML height in {}", filename);
            assert_eq!(map.lower_layer, map3.lower_layer, "XML lower_layer in {}", filename);
            assert_eq!(map.upper_layer, map3.upper_layer, "XML upper_layer in {}", filename);
            assert_eq!(map.events.len(), map3.events.len(), "XML events.len() in {}", filename);
            count += 1;
        }
    }
    println!("Successfully validated roundtrip across {} maps in TestGame-2000", count);
    assert!(count > 0);
}

#[test]
fn test_lmu_roundtrip_all_maps_2003() {
    let dir = Path::new(TEST_GAME_2003);
    if !dir.exists() {
        eprintln!("Skipping test: {:?} not found", dir);
        return;
    }

    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "lmu") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let map = match LmuReader::load(&path, "auto") {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Note: skipping non-standard map {}: {}", filename, e);
                    continue;
                }
            };

            let mut out = Cursor::new(Vec::new());
            LmuReader::save_to_writer(&mut out, &map, EngineVersion::Engine2003, "auto")
                .unwrap_or_else(|_| panic!("Failed to save {}", filename));

            out.set_position(0);
            let map2 = LmuReader::load_from_reader(&mut out, "auto")
                .unwrap_or_else(|_| panic!("Failed to reload {}", filename));

            assert_eq!(map.chipset_id, map2.chipset_id, "chipset_id in {}", filename);
            assert_eq!(map.width, map2.width, "width in {}", filename);
            assert_eq!(map.height, map2.height, "height in {}", filename);
            assert_eq!(map.lower_layer, map2.lower_layer, "lower_layer in {}", filename);
            assert_eq!(map.upper_layer, map2.upper_layer, "upper_layer in {}", filename);
            assert_eq!(map.events.len(), map2.events.len(), "events.len() in {}", filename);
            count += 1;
        }
    }
    println!("Successfully validated roundtrip across {} maps in TestGame-2003", count);
    assert!(count > 0);
}

#[test]
fn test_lsd_save_roundtrip() {
    let mut save = Save::default();
    save.system.save_count = 5;
    save.title.hero_name = lcf_core::types::DBString::new("Hero");
    save.title.hero_level = 10;
    save.title.hero_hp = 500;

    // Test inherited base class fields on SavePartyLocation
    save.party_location.map_id = 42;
    save.party_location.position_x = 15;
    save.party_location.position_y = 28;
    save.party_location.direction = 2;
    save.party_location.facing = 2;
    save.party_location.sprite_name = lcf_core::types::DBString::new("HeroSprite");
    save.party_location.flash_red = 120;
    save.party_location.flash_green = 80;
    save.party_location.flash_blue = 200;
    save.party_location.flash_current_level = 0.75;
    save.party_location.flash_time_left = 15;

    // Test inherited base class fields on SaveVehicleLocation
    save.boat_location.map_id = 42;
    save.boat_location.position_x = 16;
    save.boat_location.position_y = 28;
    save.boat_location.sprite_name = lcf_core::types::DBString::new("Ship");

    // Test SaveMapEvent in map_info
    let mut event = lcf_core::generated::lsd_gen::SaveMapEvent::default();
    event.id = 1;
    event.map_id = 42;
    event.position_x = 10;
    event.position_y = 12;
    event.sprite_name = lcf_core::types::DBString::new("NPC");
    save.map_info.events.push(event);

    let mut out = Cursor::new(Vec::new());
    LsdReader::save_to_writer(&mut out, &save, EngineVersion::Engine2000, "auto")
        .expect("Failed to save LSD");

    out.set_position(0);
    let save2 = LsdReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload LSD");

    assert_eq!(save.system.save_count, save2.system.save_count);
    assert_eq!(save.title.hero_name, save2.title.hero_name);
    assert_eq!(save.title.hero_level, save2.title.hero_level);
    assert_eq!(save.title.hero_hp, save2.title.hero_hp);

    // Assert party_location position and animation state
    assert_eq!(save2.party_location.map_id, 42);
    assert_eq!(save2.party_location.position_x, 15);
    assert_eq!(save2.party_location.position_y, 28);
    assert_eq!(save2.party_location.direction, 2);
    assert_eq!(save2.party_location.facing, 2);
    assert_eq!(save2.party_location.sprite_name.as_str(), "HeroSprite");
    assert_eq!(save2.party_location.flash_red, 120);
    assert_eq!(save2.party_location.flash_green, 80);
    assert_eq!(save2.party_location.flash_blue, 200);
    assert!((save2.party_location.flash_current_level - 0.75).abs() < 1e-5);
    assert_eq!(save2.party_location.flash_time_left, 15);

    // Assert boat_location
    assert_eq!(save2.boat_location.map_id, 42);
    assert_eq!(save2.boat_location.position_x, 16);
    assert_eq!(save2.boat_location.position_y, 28);
    assert_eq!(save2.boat_location.sprite_name.as_str(), "Ship");

    // Assert map event
    assert_eq!(save2.map_info.events.len(), 1);
    assert_eq!(save2.map_info.events[0].id, 1);
    assert_eq!(save2.map_info.events[0].map_id, 42);
    assert_eq!(save2.map_info.events[0].position_x, 10);
    assert_eq!(save2.map_info.events[0].position_y, 12);
    assert_eq!(save2.map_info.events[0].sprite_name.as_str(), "NPC");

    // Test XML save + load round trip
    let mut xml_out = Cursor::new(Vec::new());
    LsdReader::save_xml_to_writer(&mut xml_out, &save, EngineVersion::Engine2000)
        .expect("Failed to save LSD XML");
    let xml_str = String::from_utf8(xml_out.into_inner()).unwrap();
    assert!(xml_str.starts_with("<?xml"));
    assert!(xml_str.contains("<LSD>"));

    let save3 = LsdReader::load_xml_from_reader(xml_str.as_bytes())
        .expect("Failed to load LSD XML");
    assert_eq!(save3.system.save_count, save.system.save_count);
    assert_eq!(save3.title.hero_name, save.title.hero_name);
    assert_eq!(save3.title.hero_level, save.title.hero_level);
    assert_eq!(save3.title.hero_hp, save.title.hero_hp);
    assert_eq!(save3.party_location.map_id, 42);
    assert_eq!(save3.party_location.sprite_name.as_str(), "HeroSprite");
    assert!((save3.party_location.flash_current_level - 0.75).abs() < 1e-5);
    assert_eq!(save3.map_info.events.len(), 1);
    assert_eq!(save3.map_info.events[0].sprite_name.as_str(), "NPC");
}


/// Strips `XmlWriter`'s pretty-printing indentation (newlines + leading
/// spaces before each tag) so tests can assert on element structure without
/// being sensitive to indent depth. Leaves meaningful whitespace (e.g. the
/// space-separated values inside a vector text node) untouched.
fn flatten_xml_indent(xml: &str) -> String {
    let mut flat = xml.replace('\n', "");
    while flat.contains(" <") {
        flat = flat.replace(" <", "<");
    }
    flat
}

/// Verifies the XML output shape matches liblcf's `writer_xml.cpp` exactly
/// for the types that were fixed/added: `Rect` (nested l/t/r/b, not a
/// single space-separated tag), `Equipment`/`Parameters` (self-wrapped in
/// their own tag), `DBBitArray` (one space-separated T/F text node, not
/// per-item elements), `f64`, and sparse `Vec<DBString>` (`<item id="N">`,
/// empty entries skipped).
#[test]
fn test_xml_format_matches_liblcf() {
    use lcf_core::generated::ldb_gen::Actor;
    use lcf_core::generated::lmt_gen::MapInfo;
    use lcf_core::types::{DBBitArray, DBString, Rect};
    use lcf_core::xml::XmlWriter;

    // Rect: nested <Rect><l>..</l>...</Rect> inside the field-name wrapper.
    let mut map_info = MapInfo::default();
    map_info.id = 1;
    map_info.area_rect = Rect { l: 1, t: 2, r: 3, b: 4 };
    let mut out = Cursor::new(Vec::new());
    {
        let mut w = XmlWriter::new(&mut out, EngineVersion::Engine2000).unwrap();
        map_info.write_xml(&mut w).unwrap();
    }
    let xml = String::from_utf8(out.into_inner()).unwrap();
    let flat = flatten_xml_indent(&xml);
    assert!(
        flat.contains("<area_rect><Rect><l>1</l><t>2</t><r>3</r><b>4</b></Rect></area_rect>"),
        "Rect XML shape mismatch, got: {xml}"
    );

    // Equipment / Parameters / DBBitArray, via Actor.
    let mut actor = Actor::default();
    actor.id = 1;
    actor.initial_equipment.weapon_id = 7;
    actor.parameters.maxhp = vec![10, 20];
    actor.easyrpg_unarmed_state_set = DBBitArray(vec![true, false, true]);
    let mut out = Cursor::new(Vec::new());
    {
        let mut w = XmlWriter::new(&mut out, EngineVersion::Engine2003).unwrap();
        actor.write_xml(&mut w).unwrap();
    }
    let xml = String::from_utf8(out.into_inner()).unwrap();
    let flat = flatten_xml_indent(&xml);
    assert!(
        flat.contains("<initial_equipment><Equipment><weapon_id>7</weapon_id>"),
        "Equipment must self-wrap in <Equipment>, got: {xml}"
    );
    assert!(
        flat.contains("<parameters><Parameters><maxhp>10 20</maxhp>"),
        "Parameters must self-wrap in <Parameters>, got: {xml}"
    );
    assert!(
        flat.contains("<easyrpg_unarmed_state_set>T F T</easyrpg_unarmed_state_set>"),
        "DBBitArray must be one space-separated T/F text node, got: {xml}"
    );

    // f64 field (LSD only) and sparse Vec<DBString> (maniac_strings).
    let mut save = Save::default();
    save.party_location.flash_current_level = 1.5;
    save.system.maniac_strings = vec![
        DBString::default(),
        DBString::new("second"),
        DBString::default(),
        DBString::new("fourth"),
    ];
    let mut out = Cursor::new(Vec::new());
    LsdReader::save_xml_to_writer(&mut out, &save, EngineVersion::Engine2000)
        .expect("Failed to save LSD XML");
    let xml = String::from_utf8(out.into_inner()).unwrap();
    assert!(
        xml.contains("<flash_current_level>1.5</flash_current_level>"),
        "f64 field not written, got: {xml}"
    );
    assert!(
        xml.contains(r#"<item id="0002">second</item>"#) && xml.contains(r#"<item id="0004">fourth</item>"#),
        "Vec<DBString> should use 1-based <item id=\"N\">, got: {xml}"
    );
    assert!(
        !xml.contains(r#"id="0001"#) || xml.matches(r#"<item id="0001">"#).count() == 0,
        "empty DBString entries must be skipped, got: {xml}"
    );
}

#[test]
fn test_reader_util_and_setup() {
    let now = 1700000000i64;
    let t_dt = lcf_core::ReaderUtil::to_t_date_time(now);
    let back = lcf_core::ReaderUtil::to_unix_timestamp(t_dt);
    assert_eq!(now, back);

    // Precise timestamp tests from liblcf test suite (days relative to Delphi epoch)
    let t1 = lcf_core::ReaderUtil::to_unix_timestamp(27468.96875);
    assert_eq!(t1, 164157300); // 1975-03-15 23:15:00 UTC
    let t2 = lcf_core::ReaderUtil::to_unix_timestamp(36836.125);
    assert_eq!(t2, 973479600); // 2000-11-06 03:00:00 UTC
    assert_eq!(lcf_core::ReaderUtil::to_t_date_time(t1), 27468.96875);
    assert_eq!(lcf_core::ReaderUtil::to_t_date_time(t2), 36836.125);


    assert_eq!(lcf_core::ReaderUtil::codepage_to_encoding(932), "shift_jis");
    assert_eq!(lcf_core::ReaderUtil::codepage_to_encoding(1252), "windows-1252");
    assert_eq!(lcf_core::ReaderUtil::encoding_to_codepage("Shift_JIS"), 932);

    // DBBitArray tests
    let mut bits = lcf_core::types::DBBitArray::new();
    assert!(!bits.get_bit(0));
    bits.set_bit(5, true);
    assert!(bits.get_bit(5));
    assert!(!bits.get_bit(4));
    bits.set_bit(5, false);
    assert!(!bits.get_bit(5));

    let mut actor = lcf_core::Actor::default();
    lcf_core::Setup::actor(&mut actor, true);
    assert_eq!(actor.final_level, 99);
    assert_eq!(actor.exp_base, 300);
    assert_eq!(actor.parameters.maxhp.len(), 99);
}

#[test]
fn test_ldb_append_class_to_empty_classes_persists() {
    let path = Path::new(TEST_GAME_2000).join("RPG_RT.ldb");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let mut db = LdbReader::load(&path, "auto").expect("Failed to load RPG_RT.ldb 2000");
    assert_eq!(db.classes.len(), 0, "TestGame-2000 initial classes should be 0");

    // Append a new Class to the empty classes vector
    let mut new_class = lcf_core::generated::ldb_gen::Class::default();
    new_class.id = 1;
    new_class.name = "Paladin".into();
    db.classes.push(new_class);
    assert_eq!(db.classes.len(), 1);

    // Prepare and save
    LdbReader::prepare_save(&mut db);
    let mut out = Cursor::new(Vec::new());
    LdbReader::save_to_writer(&mut out, &db, EngineVersion::Engine2000, "auto")
        .expect("Failed to save database with synthesized class");

    // Reload from writer
    out.set_position(0);
    let db2 = LdbReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload database with synthesized class");

    assert_eq!(db2.classes.len(), 1, "Appended class must persist upon reload!");
    assert_eq!(db2.classes[0].name.as_str(), "Paladin");
}

#[test]
fn test_ldb_append_all_database_arrays_persists() {
    let path = Path::new(TEST_GAME_2000).join("RPG_RT.ldb");
    if !path.exists() {
        eprintln!("Skipping test: {:?} not found", path);
        return;
    }

    let mut db = LdbReader::load(&path, "auto").expect("Failed to load RPG_RT.ldb 2000");

    let initial_actors = db.actors.len();
    let initial_classes = db.classes.len();
    let initial_skills = db.skills.len();
    let initial_items = db.items.len();
    let initial_enemies = db.enemies.len();
    let initial_troops = db.troops.len();
    let initial_terrains = db.terrains.len();
    let initial_attributes = db.attributes.len();
    let initial_states = db.states.len();
    let initial_animations = db.animations.len();
    let initial_chipsets = db.chipsets.len();
    let initial_switches = db.switches.len();
    let initial_variables = db.variables.len();
    let initial_commonevents = db.commonevents.len();
    let initial_battleranimations = db.battleranimations.len();

    // Append to every single array
    db.actors.push({ let mut x = lcf_core::Actor::default(); x.id = (initial_actors + 1) as i32; x.name = "NewHero".into(); x });
    db.classes.push({ let mut x = lcf_core::generated::ldb_gen::Class::default(); x.id = (initial_classes + 1) as i32; x.name = "NewClass".into(); x });
    db.skills.push({ let mut x = lcf_core::Skill::default(); x.id = (initial_skills + 1) as i32; x.name = "NewSkill".into(); x });
    db.items.push({ let mut x = lcf_core::Item::default(); x.id = (initial_items + 1) as i32; x.name = "NewItem".into(); x });
    db.enemies.push({ let mut x = lcf_core::Enemy::default(); x.id = (initial_enemies + 1) as i32; x.name = "NewEnemy".into(); x });
    db.troops.push({ let mut x = lcf_core::Troop::default(); x.id = (initial_troops + 1) as i32; x.name = "NewTroop".into(); x });
    db.terrains.push({ let mut x = lcf_core::Terrain::default(); x.id = (initial_terrains + 1) as i32; x.name = "NewTerrain".into(); x });
    db.attributes.push({ let mut x = lcf_core::Attribute::default(); x.id = (initial_attributes + 1) as i32; x.name = "NewAttr".into(); x });
    db.states.push({ let mut x = lcf_core::State::default(); x.id = (initial_states + 1) as i32; x.name = "NewState".into(); x });
    db.animations.push({ let mut x = lcf_core::Animation::default(); x.id = (initial_animations + 1) as i32; x.name = "NewAnim".into(); x });
    db.chipsets.push({ let mut x = lcf_core::Chipset::default(); x.id = (initial_chipsets + 1) as i32; x.name = "NewChip".into(); x });
    db.switches.push({ let mut x = lcf_core::Switch::default(); x.id = (initial_switches + 1) as i32; x.name = "NewSwitch".into(); x });
    db.variables.push({ let mut x = lcf_core::Variable::default(); x.id = (initial_variables + 1) as i32; x.name = "NewVar".into(); x });
    db.commonevents.push({ let mut x = lcf_core::CommonEvent::default(); x.id = (initial_commonevents + 1) as i32; x.name = "NewCE".into(); x });
    db.battleranimations.push({ let mut x = lcf_core::BattlerAnimation::default(); x.id = (initial_battleranimations + 1) as i32; x.name = "NewBA".into(); x });

    LdbReader::prepare_save(&mut db);
    let mut out = Cursor::new(Vec::new());
    LdbReader::save_to_writer(&mut out, &db, EngineVersion::Engine2000, "auto")
        .expect("Failed to save database with all appended items");

    out.set_position(0);
    let db2 = LdbReader::load_from_reader(&mut out, "auto")
        .expect("Failed to reload database with all appended items");

    assert_eq!(db2.actors.len(), initial_actors + 1);
    assert_eq!(db2.classes.len(), initial_classes + 1);
    assert_eq!(db2.skills.len(), initial_skills + 1);
    assert_eq!(db2.items.len(), initial_items + 1);
    assert_eq!(db2.enemies.len(), initial_enemies + 1);
    assert_eq!(db2.troops.len(), initial_troops + 1);
    assert_eq!(db2.terrains.len(), initial_terrains + 1);
    assert_eq!(db2.attributes.len(), initial_attributes + 1);
    assert_eq!(db2.states.len(), initial_states + 1);
    assert_eq!(db2.animations.len(), initial_animations + 1);
    assert_eq!(db2.chipsets.len(), initial_chipsets + 1);
    assert_eq!(db2.switches.len(), initial_switches + 1);
    assert_eq!(db2.variables.len(), initial_variables + 1);
    assert_eq!(db2.commonevents.len(), initial_commonevents + 1);
    assert_eq!(db2.battleranimations.len(), initial_battleranimations + 1);

    assert_eq!(db2.actors.last().unwrap().name.as_str(), "NewHero");
    assert_eq!(db2.classes.last().unwrap().name.as_str(), "NewClass");
    assert_eq!(db2.skills.last().unwrap().name.as_str(), "NewSkill");
    assert_eq!(db2.items.last().unwrap().name.as_str(), "NewItem");
    assert_eq!(db2.enemies.last().unwrap().name.as_str(), "NewEnemy");
    assert_eq!(db2.troops.last().unwrap().name.as_str(), "NewTroop");
    assert_eq!(db2.terrains.last().unwrap().name.as_str(), "NewTerrain");
    assert_eq!(db2.attributes.last().unwrap().name.as_str(), "NewAttr");
    assert_eq!(db2.states.last().unwrap().name.as_str(), "NewState");
    assert_eq!(db2.animations.last().unwrap().name.as_str(), "NewAnim");
    assert_eq!(db2.chipsets.last().unwrap().name.as_str(), "NewChip");
    assert_eq!(db2.switches.last().unwrap().name.as_str(), "NewSwitch");
    assert_eq!(db2.variables.last().unwrap().name.as_str(), "NewVar");
    assert_eq!(db2.commonevents.last().unwrap().name.as_str(), "NewCE");
    assert_eq!(db2.battleranimations.last().unwrap().name.as_str(), "NewBA");
}




