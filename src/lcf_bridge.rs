use std::fs;
use std::path::Path;
use eframe::egui;
use lcf_core::ldb::LdbReader;
use lcf_core::lmt::LmtReader;
use lcf_core::lmu::LmuReader;
use lcf_core::lsd::LsdReader;
use lcf_core::types::{DBBitArray, DBString, EngineVersion, EventCommand as LcfEventCommand, Sound as LdbSound};
use lcf_core::generated::lmu_gen::{Event as LmuEvent, EventPage as LmuEventPage, Map as LmuMap};
use lcf_core::generated::lmt_gen::{Encounter as LmtEncounter, MapInfo as LmtMapInfo};
use lcf_core::generated::ldb_gen::{
    Chipset as LdbChipset, State as LdbState, Terrain as LdbTerrain,
    AnimationTiming as LdbAnimationTiming, AnimationFrame as LdbAnimationFrame,
    TroopPage as LdbTroopPage, TroopPageCondition as LdbTroopPageCondition, TroopMember as LdbTroopMember,
    Learning as LdbLearning, EnemyAction as LdbEnemyAction,
};

#[derive(Clone, Debug, Default)]
pub struct MapLayers {
    pub width: i32,
    pub height: i32,
    pub lower_layer: Vec<i32>,
    pub upper_layer: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventCommandInfo {
    pub code: i32,
    pub indent: i32,
    pub string: String,
    pub parameters: Vec<i32>,
}

impl From<&LcfEventCommand> for EventCommandInfo {
    fn from(cmd: &LcfEventCommand) -> Self {
        Self {
            code: cmd.code,
            indent: cmd.indent,
            string: cmd.string.0.clone(),
            parameters: cmd.parameters.clone(),
        }
    }
}

impl From<&EventCommandInfo> for LcfEventCommand {
    fn from(cmd: &EventCommandInfo) -> Self {
        Self {
            code: cmd.code,
            indent: cmd.indent,
            string: DBString::new(cmd.string.clone()),
            parameters: cmd.parameters.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventConditionInfo {
    pub switch1_flag: bool,
    pub switch1_id: i32,
    pub switch2_flag: bool,
    pub switch2_id: i32,
    pub var_flag: bool,
    pub var_id: i32,
    pub var_value: i32,
    pub var_compare_op: i32,
    pub item_flag: bool,
    pub item_id: i32,
    pub actor_flag: bool,
    pub actor_id: i32,
    pub timer_flag: bool,
    pub timer_sec: i32,
}

#[derive(Clone, Debug, Default)]
pub struct EventPageInfo {
    pub id: i32,
    pub character_name: String,
    pub character_index: i32,
    pub character_direction: i32,
    pub character_pattern: i32,
    pub translucent: bool,
    pub move_type: i32,
    pub move_frequency: i32,
    pub trigger: i32,
    pub layer: i32,
    pub overlap_forbidden: bool,
    pub animation_type: i32,
    pub move_speed: i32,
    pub condition: EventConditionInfo,
    pub commands: Vec<EventCommandInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct EventInfo {
    pub id: i32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub page_count: i32,
    pub trigger: String,
    pub graphic: String,
    pub pages: Vec<EventPageInfo>,
}

pub fn event_trigger_label(trigger: i32) -> &'static str {
    match trigger {
        0 => "Action Button",
        1 => "Player Touch",
        2 => "Event Touch",
        3 => "Autostart",
        4 => "Parallel",
        _ => "Unknown",
    }
}

pub fn event_layer_label(layer: i32) -> &'static str {
    match layer {
        0 => "Below Characters",
        1 => "Same as Characters",
        2 => "Above Characters",
        _ => "Unknown",
    }
}

pub fn event_move_type_label(move_type: i32) -> &'static str {
    match move_type {
        0 => "Stationary",
        1 => "Random",
        2 => "Step Left-Right",
        3 => "Step Up-Down",
        4 => "Towards Hero",
        5 => "Away from Hero",
        6 => "Custom Route",
        _ => "Unknown",
    }
}

pub fn event_command_label(cmd: &EventCommandInfo) -> String {
    let prefix = "  ".repeat(cmd.indent.max(0) as usize);
    let desc = match cmd.code {
        10110 => format!("Show Message: \"{}\"", cmd.string),
        10120 => "Message Options".to_string(),
        10130 => format!("Show Choices: \"{}\"", cmd.string),
        20130 => format!("When [Choice {}]", cmd.parameters.first().copied().unwrap_or(0) + 1),
        20131 => "When Cancel".to_string(),
        20132 => "End Choices".to_string(),
        10140 => format!("Input Number (Var #{})", cmd.parameters.first().copied().unwrap_or(0)),
        10210 => {
            let op = match cmd.parameters.get(3).copied().unwrap_or(0) {
                0 => "ON",
                1 => "OFF",
                _ => "TOGGLE",
            };
            format!("Control Switches: [#{}] = {}", cmd.parameters.get(1).copied().unwrap_or(0), op)
        }
        10220 => {
            let var_id = cmd.parameters.get(1).copied().unwrap_or(0);
            let op = match cmd.parameters.get(3).copied().unwrap_or(0) {
                0 => "=",
                1 => "+=",
                2 => "-=",
                3 => "*=",
                4 => "/=",
                5 => "%=",
                _ => "=",
            };
            format!("Control Variables: [#{}] {} Val", var_id, op)
        }
        10310 => "Change Gold".to_string(),
        10320 => "Change Items".to_string(),
        10330 => "Change Party Members".to_string(),
        10340 => "Change EXP".to_string(),
        10350 => "Change Level".to_string(),
        10360 => "Change Parameters".to_string(),
        10370 => "Change Skills".to_string(),
        10380 => "Change Equipment".to_string(),
        10390 => "Change HP".to_string(),
        10400 => "Change SP".to_string(),
        10410 => "Change Condition / State".to_string(),
        10420 => "Recover All".to_string(),
        10610 => format!(
            "Transfer Player -> Map #{:04} ({}, {})",
            cmd.parameters.get(1).copied().unwrap_or(0),
            cmd.parameters.get(2).copied().unwrap_or(0),
            cmd.parameters.get(3).copied().unwrap_or(0)
        ),
        10630 => "Set Event Location".to_string(),
        10710 => "Erase / Show Screen".to_string(),
        10720 => "Tint Screen".to_string(),
        10730 => "Flash Screen".to_string(),
        10740 => "Shake Screen".to_string(),
        10750 => "Pan Screen".to_string(),
        10760 => "Weather Effects".to_string(),
        10810 => format!("Show Picture: \"{}\"", cmd.string),
        10820 => "Move Picture".to_string(),
        10830 => "Erase Picture".to_string(),
        10910 => "Show Battle Animation".to_string(),
        11010 => "Flash Event".to_string(),
        11020 => "Set Move Route".to_string(),
        11030 => format!("Wait: {:.1}s", cmd.parameters.first().copied().unwrap_or(0) as f32 / 10.0),
        11110 => format!("Play BGM: \"{}\"", cmd.string),
        11120 => "Fade Out BGM".to_string(),
        11140 => format!("Play SE: \"{}\"", cmd.string),
        11210 => "Key Input Processing".to_string(),
        11310 => "Change Chipset".to_string(),
        11320 => "Change Parallax Background".to_string(),
        11410 => "Teleport Target".to_string(),
        11420 => "Escape Target".to_string(),
        11430 => "Open Save Menu".to_string(),
        11440 => "Open Main Menu".to_string(),
        11510 => "Conditional Branch".to_string(),
        21510 => "Else".to_string(),
        21511 => "End Branch".to_string(),
        11520 => "Loop".to_string(),
        21520 => "End Loop".to_string(),
        11530 => "Break Loop".to_string(),
        11540 => "Exit Event Processing".to_string(),
        11550 => "Erase Event".to_string(),
        11560 => "Call Event".to_string(),
        11570 => format!("// Comment: {}", cmd.string),
        11610 => "Game Over".to_string(),
        11620 => "Return to Title Screen".to_string(),
        11710 => "Battle Processing".to_string(),
        11720 => "Shop Processing".to_string(),
        11730 => "Inn Processing".to_string(),
        11740 => "Hero Name Input".to_string(),
        _ => {
            if !cmd.string.is_empty() {
                format!("Command #{} ({})", cmd.code, cmd.string)
            } else if !cmd.parameters.is_empty() {
                format!("Command #{} {:?}", cmd.code, cmd.parameters)
            } else {
                format!("Command #{}", cmd.code)
            }
        }
    };
    format!("{prefix}◆ {desc}")
}

#[derive(Clone, Debug, Default)]
pub struct MapEntry {
    pub id: i32,
    pub name: String,
}

pub fn event_command_color(code: i32, is_dark: bool) -> egui::Color32 {
    crate::theme::colors::event_command(code, is_dark)
}

#[derive(Clone, Debug, Default)]
pub struct ProjectInfo {
    pub valid: bool,
    pub maps: Vec<MapEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct Passability {
    pub lower: Vec<u8>,
    pub upper: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct ActorInfo {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub character_name: String,
    pub character_index: i32,
    pub face_name: String,
    pub face_index: i32,
    pub class_id: i32,
    pub class_name: String,
    pub initial_level: i32,
    pub final_level: i32,
    pub two_weapon: bool,
    pub lock_equipment: bool,
    pub auto_battle: bool,
    pub super_guard: bool,
    pub battler_animation: i32,
    pub weapon_id: i32,
    pub shield_id: i32,
    pub armor_id: i32,
    pub helmet_id: i32,
    pub accessory_id: i32,
    pub param_maxhp: Vec<i16>,
    pub param_maxsp: Vec<i16>,
    pub param_attack: Vec<i16>,
    pub param_defense: Vec<i16>,
    pub param_spirit: Vec<i16>,
    pub param_agility: Vec<i16>,
    pub skills: Vec<(i32, i32)>, // (level, skill_id)
    pub state_ranks: Vec<u8>,
    pub attribute_ranks: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub item_type: i32,
    pub price: i32,
    pub uses: i32,
    pub atk_points1: i32,
    pub def_points1: i32,
    pub spi_points1: i32,
    pub agi_points1: i32,
    pub two_handed: bool,
    pub sp_cost: i32,
    pub hit: i32,
    pub critical_hit: i32,
    pub animation_id: i32,
    pub preemptive: bool,
    pub dual_attack: bool,
    pub attack_all: bool,
    pub ignore_evasion: bool,
    pub prevent_critical: bool,
    pub raise_evasion: bool,
    pub half_sp_cost: bool,
    pub no_terrain_damage: bool,
    pub cursed: bool,
    pub entire_party: bool,
    pub recover_hp_rate: i32,
    pub recover_hp: i32,
    pub recover_sp_rate: i32,
    pub recover_sp: i32,
    pub occasion_field1: bool,
    pub occasion_battle: bool,
    pub max_hp_points: i32,
    pub max_sp_points: i32,
    pub skill_id: i32,
    pub switch_id: i32,
}

pub fn item_type_label(r#type: i32) -> &'static str {
    match r#type {
        0 => "Normal",
        1 => "Weapon",
        2 => "Shield",
        3 => "Armor",
        4 => "Helmet",
        5 => "Accessory",
        6 => "Medicine",
        7 => "Book",
        8 => "Material",
        9 => "Special",
        10 => "Switch",
        _ => "Unknown",
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub skill_type: i32,
    pub sp_type: i32,
    pub sp_percent: i32,
    pub sp_cost: i32,
    pub scope: i32,
    pub switch_id: i32,
    pub animation_id: i32,
    pub occasion_field: bool,
    pub occasion_battle: bool,
    pub reverse_state_effect: bool,
    pub physical_rate: i32,
    pub magical_rate: i32,
    pub variance: i32,
    pub power: i32,
    pub hit: i32,
    pub affect_hp: bool,
    pub affect_sp: bool,
    pub affect_attack: bool,
    pub affect_defense: bool,
    pub affect_spirit: bool,
    pub affect_agility: bool,
    pub absorb_damage: bool,
    pub ignore_defense: bool,
}

pub fn skill_type_label(r#type: i32) -> &'static str {
    match r#type {
        0 => "Normal",
        1 => "Teleport",
        2 => "Escape",
        3 => "Switch",
        4 => "Subskill",
        _ => "Unknown",
    }
}

pub fn skill_scope_label(scope: i32) -> &'static str {
    match scope {
        0 => "Enemy",
        1 => "All Enemies",
        2 => "Self",
        3 => "Ally",
        4 => "Party",
        _ => "Unknown",
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnemyActionInfo {
    pub id: i32,
    pub kind: i32, // 0: Basic, 1: Skill, 2: Morph/Transform
    pub basic: i32, // 0: Attack, 1: Double, 2: Defend, 3: Observe, 4: Charge, 5: Self-Destruct, 6: Escape, 7: Do Nothing
    pub skill_id: i32,
    pub enemy_id: i32,
    pub condition_type: i32, // 0: Always, 1: Switch ON, 2: Turn A+B*X, 3: Actor Count, 4: HP A..B%, 5: SP A..B%, 6: Party Lvl, 7: Party Fatigue
    pub condition_param1: i32,
    pub condition_param2: i32,
    pub switch_id: i32,
    pub switch_on: bool,
    pub switch_on_id: i32,
    pub switch_off: bool,
    pub switch_off_id: i32,
    pub rating: i32, // 1..100 (default 50)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnemyInfo {
    pub id: i32,
    pub name: String,
    pub battler_name: String,
    pub battler_hue: i32,
    pub max_hp: i32,
    pub max_sp: i32,
    pub attack: i32,
    pub defense: i32,
    pub spirit: i32,
    pub agility: i32,
    pub exp: i32,
    pub gold: i32,
    pub drop_id: i32,
    pub drop_prob: i32,
    pub critical_hit: bool,
    pub critical_hit_chance: i32,
    pub miss: bool,
    pub levitate: bool,
    pub transparent: bool,
    pub actions: Vec<EnemyActionInfo>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TroopMemberInfo {
    pub enemy_id: i32,
    pub x: i32,
    pub y: i32,
    pub invisible: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TroopPageConditionInfo {
    pub flags: i32,
    pub switch_a_id: i32,
    pub switch_b_id: i32,
    pub variable_id: i32,
    pub variable_value: i32,
    pub turn_a: i32,
    pub turn_b: i32,
    pub fatigue_min: i32,
    pub fatigue_max: i32,
    pub enemy_id: i32,
    pub enemy_hp_min: i32,
    pub enemy_hp_max: i32,
    pub actor_id: i32,
    pub actor_hp_min: i32,
    pub actor_hp_max: i32,
    pub turn_enemy_id: i32,
    pub turn_enemy_a: i32,
    pub turn_enemy_b: i32,
    pub turn_actor_id: i32,
    pub turn_actor_a: i32,
    pub turn_actor_b: i32,
    pub command_actor_id: i32,
    pub command_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TroopPageInfo {
    pub id: i32,
    pub condition: TroopPageConditionInfo,
    pub commands: Vec<EventCommandInfo>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TroopInfo {
    pub id: i32,
    pub name: String,
    pub auto_alignment: bool,
    pub appear_randomly: bool,
    pub terrain_set: Vec<bool>,
    pub members: Vec<TroopMemberInfo>,
    pub pages: Vec<TroopPageInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct CommonEventInfo {
    pub id: i32,
    pub name: String,
    pub trigger: i32,
    pub switch_flag: bool,
    pub switch_id: i32,
    pub commands: Vec<EventCommandInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct SwitchInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct VariableInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassInfo {
    pub id: i32,
    pub name: String,
    pub two_weapon: bool,
    pub lock_equipment: bool,
    pub auto_battle: bool,
    pub super_guard: bool,
    pub exp_base: i32,
    pub exp_inflation: i32,
    pub exp_correction: i32,
    pub battler_animation: i32,
    pub param_maxhp: Vec<i16>,
    pub param_maxsp: Vec<i16>,
    pub param_attack: Vec<i16>,
    pub param_defense: Vec<i16>,
    pub param_spirit: Vec<i16>,
    pub param_agility: Vec<i16>,
    pub skills: Vec<(i32, i32)>, // (level, skill_id)
    pub state_ranks: Vec<u8>,
    pub attribute_ranks: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct AttributeInfo {
    pub id: i32,
    pub name: String,
    pub attribute_type: String,
    pub a_rate: i32,
    pub b_rate: i32,
    pub c_rate: i32,
    pub d_rate: i32,
    pub e_rate: i32,
}

pub fn attribute_type_label(r#type: i32) -> &'static str {
    match r#type {
        0 => "Physical",
        1 => "Magical",
        _ => "Unknown",
    }
}

#[derive(Clone, Debug, Default)]
pub struct SystemInfo {
    pub ldb_id: i32,
    pub boat_name: String,
    pub ship_name: String,
    pub airship_name: String,
    pub title_name: String,
    pub gameover_name: String,
    pub system_name: String,
    pub system2_name: String,
    pub party: Vec<i16>,
    pub title_music_name: String,
    pub battle_music_name: String,
    pub victory_music_name: String,
    pub gameover_music_name: String,
    pub cursor_sound_name: String,
    pub decision_sound_name: String,
    pub cancel_sound_name: String,
    pub buzzer_sound_name: String,
    pub transition_out: i32,
    pub transition_in: i32,
    pub battle_start_fadeout: i32,
    pub battle_start_fadein: i32,
    pub battle_end_fadeout: i32,
    pub battle_end_fadein: i32,
    pub font_id: i32,
    pub save_count: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TermsInfo {
    // Menu & General
    pub new_game: String,
    pub load_game: String,
    pub exit_game: String,
    pub status: String,
    pub menu_equipment: String,
    pub menu_save: String,
    pub menu_quit: String,
    pub row: String,
    pub order: String,
    pub wait_on: String,
    pub wait_off: String,
    pub level: String,
    pub health_points: String,
    pub spirit_points: String,
    pub normal_status: String,
    pub exp_short: String,
    pub lvl_short: String,
    pub hp_short: String,
    pub sp_short: String,
    pub sp_cost: String,
    pub attack: String,
    pub defense: String,
    pub spirit: String,
    pub agility: String,
    pub weapon: String,
    pub shield: String,
    pub armor: String,
    pub helmet: String,
    pub accessory: String,
    pub command_attack: String,
    pub command_defend: String,
    pub command_item: String,
    pub command_skill: String,
    pub battle_auto: String,
    pub battle_escape: String,
    pub battle_fight: String,
    pub gold: String,
    pub possessed_items: String,
    pub equipped_items: String,
    pub save_game_message: String,
    pub load_game_message: String,
    pub exit_game_message: String,
    pub file: String,
    pub yes: String,
    pub no: String,

    // Battle Messages
    pub encounter: String,
    pub special_combat: String,
    pub escape_success: String,
    pub escape_failure: String,
    pub victory: String,
    pub defeat: String,
    pub exp_received: String,
    pub gold_recieved_a: String,
    pub gold_recieved_b: String,
    pub item_recieved: String,
    pub attacking: String,
    pub enemy_critical: String,
    pub actor_critical: String,
    pub defending: String,
    pub observing: String,
    pub focus: String,
    pub autodestruction: String,
    pub enemy_escape: String,
    pub enemy_transform: String,
    pub enemy_damaged: String,
    pub enemy_undamaged: String,
    pub actor_damaged: String,
    pub actor_undamaged: String,
    pub skill_failure_a: String,
    pub skill_failure_b: String,
    pub skill_failure_c: String,
    pub dodge: String,
    pub use_item: String,
    pub hp_recovery: String,
    pub parameter_increase: String,
    pub parameter_decrease: String,
    pub enemy_hp_absorbed: String,
    pub actor_hp_absorbed: String,
    pub resistance_increase: String,
    pub resistance_decrease: String,
    pub level_up: String,
    pub skill_learned: String,
    pub battle_start: String,
    pub miss: String,

    // Shop 1
    pub shop_greeting1: String,
    pub shop_regreeting1: String,
    pub shop_buy1: String,
    pub shop_sell1: String,
    pub shop_leave1: String,
    pub shop_buy_select1: String,
    pub shop_buy_number1: String,
    pub shop_purchased1: String,
    pub shop_sell_select1: String,
    pub shop_sell_number1: String,
    pub shop_sold1: String,

    // Shop 2
    pub shop_greeting2: String,
    pub shop_regreeting2: String,
    pub shop_buy2: String,
    pub shop_sell2: String,
    pub shop_leave2: String,
    pub shop_buy_select2: String,
    pub shop_buy_number2: String,
    pub shop_purchased2: String,
    pub shop_sell_select2: String,
    pub shop_sell_number2: String,
    pub shop_sold2: String,

    // Shop 3
    pub shop_greeting3: String,
    pub shop_regreeting3: String,
    pub shop_buy3: String,
    pub shop_sell3: String,
    pub shop_leave3: String,
    pub shop_buy_select3: String,
    pub shop_buy_number3: String,
    pub shop_purchased3: String,
    pub shop_sell_select3: String,
    pub shop_sell_number3: String,
    pub shop_sold3: String,

    // Inn A
    pub inn_a_greeting_1: String,
    pub inn_a_greeting_2: String,
    pub inn_a_greeting_3: String,
    pub inn_a_accept: String,
    pub inn_a_cancel: String,

    // Inn B
    pub inn_b_greeting_1: String,
    pub inn_b_greeting_2: String,
    pub inn_b_greeting_3: String,
    pub inn_b_accept: String,
    pub inn_b_cancel: String,

    // Maniac Patch Terms
    pub maniac_item_received_a: String,
    pub maniac_level_up_a: String,
    pub maniac_level_up_b: String,
    pub maniac_level_up_c: String,
    pub maniac_exp_received_a: String,
    pub maniac_skill_learned_a: String,

    // EasyRPG Extended Terms
    pub easyrpg_item_number_separator: String,
    pub easyrpg_skill_cost_separator: String,
    pub easyrpg_equipment_arrow: String,
    pub easyrpg_status_scene_name: String,
    pub easyrpg_status_scene_class: String,
    pub easyrpg_status_scene_title: String,
    pub easyrpg_status_scene_condition: String,
    pub easyrpg_status_scene_front: String,
    pub easyrpg_status_scene_back: String,
    pub easyrpg_order_scene_confirm: String,
    pub easyrpg_order_scene_redo: String,
    pub easyrpg_battle2k3_double_attack: String,
    pub easyrpg_battle2k3_defend: String,
    pub easyrpg_battle2k3_observe: String,
    pub easyrpg_battle2k3_charge: String,
    pub easyrpg_battle2k3_selfdestruct: String,
    pub easyrpg_battle2k3_escape: String,
    pub easyrpg_battle2k3_special_combat_back: String,
    pub easyrpg_battle2k3_skill: String,
    pub easyrpg_battle2k3_item: String,
}

#[derive(Clone, Debug, Default)]
pub struct SavePartyMember {
    pub id: i32,
    pub name: String,
    pub level: i32,
    pub current_hp: i32,
    pub current_sp: i32,
}

#[derive(Clone, Debug, Default)]
pub struct SaveSlotInfo {
    pub file_name: String,
    pub hero_name: String,
    pub hero_level: i32,
    pub hero_hp: i32,
    pub timestamp: String,
    pub map_id: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub gold: i32,
    pub party: Vec<SavePartyMember>,
    pub inventory: Vec<(i32, i32)>, // item_id, count
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapTreeItem {
    pub id: i32,
    pub name: String,
    pub parent_map: i32,
    pub indentation: i32,
    pub expanded_node: bool,
    pub music_type: i32,
    pub music_name: String,
    pub background_type: i32,
    pub background_name: String,
    pub teleport: i32,
    pub escape: i32,
    pub save: i32,
    pub encounter_steps: i32,
    pub encounters: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartPointInfo {
    pub party_map_id: i32,
    pub party_x: i32,
    pub party_y: i32,
    pub boat_map_id: i32,
    pub boat_x: i32,
    pub boat_y: i32,
    pub ship_map_id: i32,
    pub ship_x: i32,
    pub ship_y: i32,
    pub airship_map_id: i32,
    pub airship_x: i32,
    pub airship_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorOrigin {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapPropertiesInfo {
    pub id: i32,
    pub name: String,
    pub parent_map: i32,
    pub chipset_id: i32,
    pub width: i32,
    pub height: i32,
    pub scroll_type: i32, // 0 = None, 1 = Vertical, 2 = Horizontal, 3 = Both
    pub parallax_name: String,
    pub parallax_loop_x: bool,
    pub parallax_loop_y: bool,
    pub parallax_sx: i32,
    pub parallax_sy: i32,
    pub music_type: i32,
    pub music_name: String,
    pub background_type: i32,
    pub background_name: String,
    pub teleport: i32,
    pub escape: i32,
    pub save: i32,
    pub encounter_steps: i32,
    pub encounters: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChipsetInfo {
    pub id: i32,
    pub name: String,
    pub chipset_name: String,
    pub terrain_data: Vec<i16>,
    pub passable_data_lower: Vec<u8>,
    pub passable_data_upper: Vec<u8>,
    pub animation_type: i32,
    pub animation_speed: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateInfo {
    pub id: i32,
    pub name: String,
    pub state_type: i32,
    pub color: i32,
    pub priority: i32,
    pub restriction: i32,
    pub a_rate: i32,
    pub b_rate: i32,
    pub c_rate: i32,
    pub d_rate: i32,
    pub e_rate: i32,
    pub hold_turn: i32,
    pub auto_release_prob: i32,
    pub release_by_damage: i32,
    pub affect_attack: bool,
    pub affect_defense: bool,
    pub affect_spirit: bool,
    pub affect_agility: bool,
    pub reduce_hit_ratio: i32,
    pub avoid_attacks: bool,
    pub reflect_magic: bool,
    pub cursed: bool,
    pub hp_change_type: i32,
    pub hp_change_val: i32,
    pub hp_change_max: i32,
    pub hp_change_map_steps: i32,
    pub hp_change_map_val: i32,
    pub sp_change_type: i32,
    pub sp_change_val: i32,
    pub sp_change_max: i32,
    pub sp_change_map_steps: i32,
    pub sp_change_map_val: i32,
    pub message_actor: String,
    pub message_enemy: String,
    pub message_already: String,
    pub message_affected: String,
    pub message_recovery: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TerrainInfo {
    pub id: i32,
    pub name: String,
    pub damage: i32,
    pub encounter_rate: i32,
    pub background_name: String,
    pub boat_pass: bool,
    pub ship_pass: bool,
    pub airship_pass: bool,
    pub airship_land: bool,
    pub bush_depth: i32,
    pub footstep_name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationTimingInfo {
    pub id: i32,
    pub frame: i32,
    pub se_name: String,
    pub flash_scope: i32,
    pub flash_red: i32,
    pub flash_green: i32,
    pub flash_blue: i32,
    pub flash_power: i32,
    pub screen_shake: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationInfo {
    pub id: i32,
    pub name: String,
    pub animation_name: String,
    pub large: bool,
    pub scope: i32,
    pub position: i32,
    pub frame_count: usize,
    pub timings: Vec<AnimationTimingInfo>,
}

fn map_filename(map_id: i32) -> String {
    format!("Map{:04}.lmu", map_id)
}

fn format_unix_timestamp(unix: i64) -> String {
    if unix <= 0 {
        return "unknown".to_string();
    }
    let days = unix.div_euclid(86400);
    let secs_of_day = unix.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    let hh = secs_of_day / 3600;
    let mm = (secs_of_day % 3600) / 60;
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn engine_version_for(db: &lcf_core::Database) -> EngineVersion {
    if db.system.ldb_id == 2003 || db.version >= 2 {
        EngineVersion::Engine2003
    } else {
        EngineVersion::Engine2000
    }
}

fn backup_file_once(path: &Path) -> Result<(), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bak");
    let backup_path = path.with_extension(format!("{ext}.bak"));
    if !backup_path.exists() {
        fs::copy(path, &backup_path).map_err(|e| format!("backup failed: {e}"))?;
    }
    Ok(())
}

fn backup_ldb_once(ldb_path: &Path) -> Result<(), String> {
    backup_file_once(ldb_path)
}

// ---------------------------------------------------------------------------
// Maps & Layers
// ---------------------------------------------------------------------------

pub fn load_project(path: &str) -> ProjectInfo {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let treemap = match LmtReader::load(&lmt_path, "auto") {
        Ok(t) => t,
        Err(_) => return ProjectInfo { valid: false, maps: Vec::new() },
    };

    let mut maps = Vec::new();
    for map_info in treemap.maps {
        if (map_info.r#type == 1 || map_info.r#type == -1) && map_info.id > 0 {
            maps.push(MapEntry {
                id: map_info.id,
                name: map_info.name.0,
            });
        }
    }

    ProjectInfo {
        valid: true,
        maps,
    }
}

pub fn is_project_2003(path: &str) -> bool {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    if let Ok(db) = LdbReader::load(&ldb_path, "auto") {
        lcf_core::reader_util::ReaderUtil::get_engine_version(&db).is_2k3()
    } else {
        false
    }
}

pub fn get_map_chipset(path: &str, map_id: i32) -> Vec<u8> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let map = match LmuReader::load(&map_path, "auto") {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let chipset_idx = (map.chipset_id - 1) as usize;
    if chipset_idx >= db.chipsets.len() {
        return Vec::new();
    }

    let chipset_name = &db.chipsets[chipset_idx].chipset_name.0;
    if chipset_name.is_empty() {
        return Vec::new();
    }

    let img_path = Path::new(path).join("ChipSet").join(format!("{}.png", chipset_name));
    fs::read(img_path).unwrap_or_default()
}

pub fn get_map_layers(path: &str, map_id: i32) -> MapLayers {
    let map_path = Path::new(path).join(map_filename(map_id));
    let map = match LmuReader::load(&map_path, "auto") {
        Ok(m) => m,
        Err(_) => return MapLayers::default(),
    };

    MapLayers {
        width: map.width,
        height: map.height,
        lower_layer: map.lower_layer.into_iter().map(|v| v as i32).collect(),
        upper_layer: map.upper_layer.into_iter().map(|v| v as i32).collect(),
    }
}

pub fn save_map_layers(path: &str, map_id: i32, lower: &[i32], upper: &[i32]) -> Result<(), String> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let mut map = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;

    backup_file_once(&map_path)?;

    map.lower_layer = lower.iter().map(|&v| v as i16).collect();
    map.upper_layer = upper.iter().map(|&v| v as i16).collect();

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };

    LmuReader::save(&map_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_chipset_passability(path: &str, map_id: i32) -> Passability {
    let map_path = Path::new(path).join(map_filename(map_id));
    let map = match LmuReader::load(&map_path, "auto") {
        Ok(m) => m,
        Err(_) => return Passability::default(),
    };

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Passability::default(),
    };

    let chipset_idx = (map.chipset_id - 1) as usize;
    if chipset_idx >= db.chipsets.len() {
        return Passability::default();
    }

    let cs = &db.chipsets[chipset_idx];
    Passability {
        lower: cs.passable_data_lower.clone(),
        upper: cs.passable_data_upper.clone(),
    }
}

// ---------------------------------------------------------------------------
// Events (Simple & Full)
// ---------------------------------------------------------------------------

pub fn get_map_events(path: &str, map_id: i32) -> Vec<EventInfo> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let map = match LmuReader::load(&map_path, "auto") {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    map.events
        .into_iter()
        .map(|e| {
            let page_count = e.pages.len() as i32;
            let (trigger, graphic) = if let Some(first_page) = e.pages.first() {
                (
                    event_trigger_label(first_page.trigger).to_string(),
                    first_page.character_name.0.clone(),
                )
            } else {
                (String::new(), String::new())
            };

            let pages = e
                .pages
                .into_iter()
                .map(|p| {
                    let f = p.condition.flags;
                    let cond = EventConditionInfo {
                        switch1_flag: (f & 0x01) != 0,
                        switch1_id: p.condition.switch_a_id,
                        switch2_flag: (f & 0x02) != 0,
                        switch2_id: p.condition.switch_b_id,
                        var_flag: (f & 0x04) != 0,
                        var_id: p.condition.variable_id,
                        var_value: p.condition.variable_value,
                        var_compare_op: p.condition.compare_operator,
                        item_flag: (f & 0x08) != 0,
                        item_id: p.condition.item_id,
                        actor_flag: (f & 0x10) != 0,
                        actor_id: p.condition.actor_id,
                        timer_flag: (f & 0x20) != 0,
                        timer_sec: p.condition.timer_sec,
                    };

                    let commands = p.event_commands.iter().map(EventCommandInfo::from).collect();

                    EventPageInfo {
                        id: p.id,
                        character_name: p.character_name.0,
                        character_index: p.character_index,
                        character_direction: p.character_direction,
                        character_pattern: p.character_pattern,
                        translucent: p.translucent,
                        move_type: p.move_type,
                        move_frequency: p.move_frequency,
                        trigger: p.trigger,
                        layer: p.layer,
                        overlap_forbidden: p.overlap_forbidden,
                        animation_type: p.animation_type,
                        move_speed: p.move_speed,
                        condition: cond,
                        commands,
                    }
                })
                .collect();

            EventInfo {
                id: e.id,
                name: e.name.0,
                x: e.x,
                y: e.y,
                page_count,
                trigger,
                graphic,
                pages,
            }
        })
        .collect()
}

pub fn save_map_events(path: &str, map_id: i32, events: &[EventInfo]) -> Result<(), String> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let mut map = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;

    backup_file_once(&map_path)?;

    for edit in events {
        if let Some(event) = map.events.iter_mut().find(|e| e.id == edit.id) {
            event.name = edit.name.clone().into();
            event.x = edit.x;
            event.y = edit.y;
        }
    }

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };

    LmuReader::save(&map_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_map_events_full(path: &str, map_id: i32, events: &[EventInfo]) -> Result<(), String> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let mut map = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;

    backup_file_once(&map_path)?;

    map.events = events
        .iter()
        .map(|e| {
            let mut lmu_e = LmuEvent::default();
            lmu_e.id = e.id;
            lmu_e.name = DBString::new(e.name.clone());
            lmu_e.x = e.x;
            lmu_e.y = e.y;

            lmu_e.pages = e
                .pages
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let mut page = LmuEventPage::default();
                    page.id = (i + 1) as i32;
                    page.character_name = DBString::new(p.character_name.clone());
                    page.character_index = p.character_index;
                    page.character_direction = p.character_direction;
                    page.character_pattern = p.character_pattern;
                    page.translucent = p.translucent;
                    page.move_type = p.move_type;
                    page.move_frequency = p.move_frequency;
                    page.trigger = p.trigger;
                    page.layer = p.layer;
                    page.overlap_forbidden = p.overlap_forbidden;
                    page.animation_type = p.animation_type;
                    page.move_speed = p.move_speed;

                    let mut flags = 0i32;
                    if p.condition.switch1_flag { flags |= 0x01; }
                    if p.condition.switch2_flag { flags |= 0x02; }
                    if p.condition.var_flag { flags |= 0x04; }
                    if p.condition.item_flag { flags |= 0x08; }
                    if p.condition.actor_flag { flags |= 0x10; }
                    if p.condition.timer_flag { flags |= 0x20; }
                    page.condition.flags = flags;
                    page.condition.switch_a_id = p.condition.switch1_id;
                    page.condition.switch_b_id = p.condition.switch2_id;
                    page.condition.variable_id = p.condition.var_id;
                    page.condition.variable_value = p.condition.var_value;
                    page.condition.compare_operator = p.condition.var_compare_op;
                    page.condition.item_id = p.condition.item_id;
                    page.condition.actor_id = p.condition.actor_id;
                    page.condition.timer_sec = p.condition.timer_sec;

                    page.event_commands = p.commands.iter().map(LcfEventCommand::from).collect();

                    page
                })
                .collect();

            lmu_e
        })
        .collect();

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };

    LmuReader::save(&map_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Database Entities
// ---------------------------------------------------------------------------

pub fn get_actors(path: &str) -> Vec<ActorInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.actors
        .into_iter()
        .map(|actor| {
            let class_name = db
                .classes
                .iter()
                .find(|c| c.id == actor.class_id)
                .map(|c| c.name.0.clone())
                .unwrap_or_default();

            ActorInfo {
                id: actor.id,
                name: actor.name.0,
                title: actor.title.0,
                character_name: actor.character_name.0,
                character_index: actor.character_index,
                face_name: actor.face_name.0,
                face_index: actor.face_index,
                class_id: actor.class_id,
                class_name,
                initial_level: actor.initial_level,
                final_level: actor.final_level,
                two_weapon: actor.two_weapon,
                lock_equipment: actor.lock_equipment,
                auto_battle: actor.auto_battle,
                super_guard: actor.super_guard,
                battler_animation: actor.battler_animation,
                weapon_id: actor.initial_equipment.weapon_id,
                shield_id: actor.initial_equipment.shield_id,
                armor_id: actor.initial_equipment.armor_id,
                helmet_id: actor.initial_equipment.helmet_id,
                accessory_id: actor.initial_equipment.accessory_id,
                param_maxhp: actor.parameters.maxhp.clone(),
                param_maxsp: actor.parameters.maxsp.clone(),
                param_attack: actor.parameters.attack.clone(),
                param_defense: actor.parameters.defense.clone(),
                param_spirit: actor.parameters.spirit.clone(),
                param_agility: actor.parameters.agility.clone(),
                skills: actor.skills.iter().map(|s| (s.level, s.skill_id)).collect(),
                state_ranks: actor.state_ranks.clone(),
                attribute_ranks: actor.attribute_ranks.clone(),
            }
        })
        .collect()
}

pub fn save_actors(path: &str, actors: &[ActorInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in actors {
        if let Some(actor) = db.actors.iter_mut().find(|a| a.id == edit.id) {
            actor.name = edit.name.clone().into();
            actor.title = edit.title.clone().into();
            actor.character_name = edit.character_name.clone().into();
            actor.character_index = edit.character_index;
            actor.face_name = edit.face_name.clone().into();
            actor.face_index = edit.face_index;
            actor.class_id = edit.class_id;
            actor.initial_level = edit.initial_level;
            actor.final_level = edit.final_level;
            actor.two_weapon = edit.two_weapon;
            actor.lock_equipment = edit.lock_equipment;
            actor.auto_battle = edit.auto_battle;
            actor.super_guard = edit.super_guard;
            actor.battler_animation = edit.battler_animation;
            actor.initial_equipment.weapon_id = edit.weapon_id;
            actor.initial_equipment.shield_id = edit.shield_id;
            actor.initial_equipment.armor_id = edit.armor_id;
            actor.initial_equipment.helmet_id = edit.helmet_id;
            actor.initial_equipment.accessory_id = edit.accessory_id;
            if !edit.param_maxhp.is_empty() {
                actor.parameters.maxhp = edit.param_maxhp.clone();
            }
            if !edit.param_maxsp.is_empty() {
                actor.parameters.maxsp = edit.param_maxsp.clone();
            }
            if !edit.param_attack.is_empty() {
                actor.parameters.attack = edit.param_attack.clone();
            }
            if !edit.param_defense.is_empty() {
                actor.parameters.defense = edit.param_defense.clone();
            }
            if !edit.param_spirit.is_empty() {
                actor.parameters.spirit = edit.param_spirit.clone();
            }
            if !edit.param_agility.is_empty() {
                actor.parameters.agility = edit.param_agility.clone();
            }
            actor.skills = edit.skills.iter().enumerate().map(|(i, &(lvl, sid))| {
                LdbLearning {
                    id: (i + 1) as i32,
                    level: lvl,
                    skill_id: sid,
                }
            }).collect();
            actor.state_ranks = edit.state_ranks.clone();
            actor.attribute_ranks = edit.attribute_ranks.clone();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_classes(path: &str) -> Vec<ClassInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.classes
        .into_iter()
        .map(|class| {
            ClassInfo {
                id: class.id,
                name: class.name.0,
                two_weapon: class.two_weapon,
                lock_equipment: class.lock_equipment,
                auto_battle: class.auto_battle,
                super_guard: class.super_guard,
                exp_base: class.exp_base,
                exp_inflation: class.exp_inflation,
                exp_correction: class.exp_correction,
                battler_animation: class.battler_animation,
                param_maxhp: class.parameters.maxhp.clone(),
                param_maxsp: class.parameters.maxsp.clone(),
                param_attack: class.parameters.attack.clone(),
                param_defense: class.parameters.defense.clone(),
                param_spirit: class.parameters.spirit.clone(),
                param_agility: class.parameters.agility.clone(),
                skills: class.skills.iter().map(|s| (s.level, s.skill_id)).collect(),
                state_ranks: class.state_ranks.clone(),
                attribute_ranks: class.attribute_ranks.clone(),
            }
        })
        .collect()
}

pub fn save_classes(path: &str, classes: &[ClassInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in classes {
        if let Some(class) = db.classes.iter_mut().find(|c| c.id == edit.id) {
            class.name = edit.name.clone().into();
            class.two_weapon = edit.two_weapon;
            class.lock_equipment = edit.lock_equipment;
            class.auto_battle = edit.auto_battle;
            class.super_guard = edit.super_guard;
            class.exp_base = edit.exp_base;
            class.exp_inflation = edit.exp_inflation;
            class.exp_correction = edit.exp_correction;
            class.battler_animation = edit.battler_animation;
            if !edit.param_maxhp.is_empty() {
                class.parameters.maxhp = edit.param_maxhp.clone();
            }
            if !edit.param_maxsp.is_empty() {
                class.parameters.maxsp = edit.param_maxsp.clone();
            }
            if !edit.param_attack.is_empty() {
                class.parameters.attack = edit.param_attack.clone();
            }
            if !edit.param_defense.is_empty() {
                class.parameters.defense = edit.param_defense.clone();
            }
            if !edit.param_spirit.is_empty() {
                class.parameters.spirit = edit.param_spirit.clone();
            }
            if !edit.param_agility.is_empty() {
                class.parameters.agility = edit.param_agility.clone();
            }
            class.skills = edit.skills.iter().enumerate().map(|(i, &(lvl, sid))| {
                LdbLearning {
                    id: (i + 1) as i32,
                    level: lvl,
                    skill_id: sid,
                }
            }).collect();
            class.state_ranks = edit.state_ranks.clone();
            class.attribute_ranks = edit.attribute_ranks.clone();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_items(path: &str) -> Vec<ItemInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.items
        .into_iter()
        .map(|item| ItemInfo {
            id: item.id,
            name: item.name.0,
            description: item.description.0,
            item_type: item.r#type,
            price: item.price,
            uses: item.uses,
            atk_points1: item.atk_points1,
            def_points1: item.def_points1,
            spi_points1: item.spi_points1,
            agi_points1: item.agi_points1,
            two_handed: item.two_handed,
            sp_cost: item.sp_cost,
            hit: item.hit,
            critical_hit: item.critical_hit,
            animation_id: item.animation_id,
            preemptive: item.preemptive,
            dual_attack: item.dual_attack,
            attack_all: item.attack_all,
            ignore_evasion: item.ignore_evasion,
            prevent_critical: item.prevent_critical,
            raise_evasion: item.raise_evasion,
            half_sp_cost: item.half_sp_cost,
            no_terrain_damage: item.no_terrain_damage,
            cursed: item.cursed,
            entire_party: item.entire_party,
            recover_hp_rate: item.recover_hp_rate,
            recover_hp: item.recover_hp,
            recover_sp_rate: item.recover_sp_rate,
            recover_sp: item.recover_sp,
            occasion_field1: item.occasion_field1,
            occasion_battle: item.occasion_battle,
            max_hp_points: item.max_hp_points,
            max_sp_points: item.max_sp_points,
            skill_id: item.skill_id,
            switch_id: item.switch_id,
        })
        .collect()
}

pub fn save_items(path: &str, items: &[ItemInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in items {
        if let Some(item) = db.items.iter_mut().find(|i| i.id == edit.id) {
            item.name = edit.name.clone().into();
            item.description = edit.description.clone().into();
            item.r#type = edit.item_type;
            item.price = edit.price;
            item.uses = edit.uses;
            item.atk_points1 = edit.atk_points1;
            item.def_points1 = edit.def_points1;
            item.spi_points1 = edit.spi_points1;
            item.agi_points1 = edit.agi_points1;
            item.two_handed = edit.two_handed;
            item.sp_cost = edit.sp_cost;
            item.hit = edit.hit;
            item.critical_hit = edit.critical_hit;
            item.animation_id = edit.animation_id;
            item.preemptive = edit.preemptive;
            item.dual_attack = edit.dual_attack;
            item.attack_all = edit.attack_all;
            item.ignore_evasion = edit.ignore_evasion;
            item.prevent_critical = edit.prevent_critical;
            item.raise_evasion = edit.raise_evasion;
            item.half_sp_cost = edit.half_sp_cost;
            item.no_terrain_damage = edit.no_terrain_damage;
            item.cursed = edit.cursed;
            item.entire_party = edit.entire_party;
            item.recover_hp_rate = edit.recover_hp_rate;
            item.recover_hp = edit.recover_hp;
            item.recover_sp_rate = edit.recover_sp_rate;
            item.recover_sp = edit.recover_sp;
            item.occasion_field1 = edit.occasion_field1;
            item.occasion_battle = edit.occasion_battle;
            item.max_hp_points = edit.max_hp_points;
            item.max_sp_points = edit.max_sp_points;
            item.skill_id = edit.skill_id;
            item.switch_id = edit.switch_id;
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_skills(path: &str) -> Vec<SkillInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.skills
        .into_iter()
        .map(|skill| SkillInfo {
            id: skill.id,
            name: skill.name.0,
            description: skill.description.0,
            skill_type: skill.r#type,
            sp_type: skill.sp_type,
            sp_percent: skill.sp_percent,
            sp_cost: skill.sp_cost,
            scope: skill.scope,
            switch_id: skill.switch_id,
            animation_id: skill.animation_id,
            occasion_field: skill.occasion_field,
            occasion_battle: skill.occasion_battle,
            reverse_state_effect: skill.reverse_state_effect,
            physical_rate: skill.physical_rate,
            magical_rate: skill.magical_rate,
            variance: skill.variance,
            power: skill.power,
            hit: skill.hit,
            affect_hp: skill.affect_hp,
            affect_sp: skill.affect_sp,
            affect_attack: skill.affect_attack,
            affect_defense: skill.affect_defense,
            affect_spirit: skill.affect_spirit,
            affect_agility: skill.affect_agility,
            absorb_damage: skill.absorb_damage,
            ignore_defense: skill.ignore_defense,
        })
        .collect()
}

pub fn save_skills(path: &str, skills: &[SkillInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in skills {
        if let Some(skill) = db.skills.iter_mut().find(|s| s.id == edit.id) {
            skill.name = edit.name.clone().into();
            skill.description = edit.description.clone().into();
            skill.r#type = edit.skill_type;
            skill.sp_type = edit.sp_type;
            skill.sp_percent = edit.sp_percent;
            skill.sp_cost = edit.sp_cost;
            skill.scope = edit.scope;
            skill.switch_id = edit.switch_id;
            skill.animation_id = edit.animation_id;
            skill.occasion_field = edit.occasion_field;
            skill.occasion_battle = edit.occasion_battle;
            skill.reverse_state_effect = edit.reverse_state_effect;
            skill.physical_rate = edit.physical_rate;
            skill.magical_rate = edit.magical_rate;
            skill.variance = edit.variance;
            skill.power = edit.power;
            skill.hit = edit.hit;
            skill.affect_hp = edit.affect_hp;
            skill.affect_sp = edit.affect_sp;
            skill.affect_attack = edit.affect_attack;
            skill.affect_defense = edit.affect_defense;
            skill.affect_spirit = edit.affect_spirit;
            skill.affect_agility = edit.affect_agility;
            skill.absorb_damage = edit.absorb_damage;
            skill.ignore_defense = edit.ignore_defense;
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_attributes(path: &str) -> Vec<AttributeInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.attributes
        .into_iter()
        .map(|attr| AttributeInfo {
            id: attr.id,
            name: attr.name.0,
            attribute_type: attribute_type_label(attr.r#type).to_string(),
            a_rate: attr.a_rate,
            b_rate: attr.b_rate,
            c_rate: attr.c_rate,
            d_rate: attr.d_rate,
            e_rate: attr.e_rate,
        })
        .collect()
}

pub fn save_attributes(path: &str, attributes: &[AttributeInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in attributes {
        if let Some(attr) = db.attributes.iter_mut().find(|a| a.id == edit.id) {
            attr.name = edit.name.clone().into();
            attr.a_rate = edit.a_rate;
            attr.b_rate = edit.b_rate;
            attr.c_rate = edit.c_rate;
            attr.d_rate = edit.d_rate;
            attr.e_rate = edit.e_rate;
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_enemies(path: &str) -> Vec<EnemyInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.enemies
        .into_iter()
        .map(|e| EnemyInfo {
            id: e.id,
            name: e.name.0,
            battler_name: e.battler_name.0,
            battler_hue: e.battler_hue,
            max_hp: e.max_hp,
            max_sp: e.max_sp,
            attack: e.attack,
            defense: e.defense,
            spirit: e.spirit,
            agility: e.agility,
            exp: e.exp,
            gold: e.gold,
            drop_id: e.drop_id,
            drop_prob: e.drop_prob,
            critical_hit: e.critical_hit,
            critical_hit_chance: e.critical_hit_chance,
            miss: e.miss,
            levitate: e.levitate,
            transparent: e.transparent,
            actions: e.actions.into_iter().map(|a| EnemyActionInfo {
                id: a.id,
                kind: a.kind,
                basic: a.basic,
                skill_id: a.skill_id,
                enemy_id: a.enemy_id,
                condition_type: a.condition_type,
                condition_param1: a.condition_param1,
                condition_param2: a.condition_param2,
                switch_id: a.switch_id,
                switch_on: a.switch_on,
                switch_on_id: a.switch_on_id,
                switch_off: a.switch_off,
                switch_off_id: a.switch_off_id,
                rating: a.rating,
            }).collect(),
        })
        .collect()
}

pub fn save_enemies(path: &str, enemies: &[EnemyInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in enemies {
        if let Some(e) = db.enemies.iter_mut().find(|e| e.id == edit.id) {
            e.name = edit.name.clone().into();
            e.battler_name = edit.battler_name.clone().into();
            e.battler_hue = edit.battler_hue;
            e.max_hp = edit.max_hp;
            e.max_sp = edit.max_sp;
            e.attack = edit.attack;
            e.defense = edit.defense;
            e.spirit = edit.spirit;
            e.agility = edit.agility;
            e.exp = edit.exp;
            e.gold = edit.gold;
            e.drop_id = edit.drop_id;
            e.drop_prob = edit.drop_prob;
            e.critical_hit = edit.critical_hit;
            e.critical_hit_chance = edit.critical_hit_chance;
            e.miss = edit.miss;
            e.levitate = edit.levitate;
            e.transparent = edit.transparent;
            e.actions = edit.actions.iter().enumerate().map(|(i, a)| LdbEnemyAction {
                id: (i + 1) as i32,
                kind: a.kind,
                basic: a.basic,
                skill_id: a.skill_id,
                enemy_id: a.enemy_id,
                condition_type: a.condition_type,
                condition_param1: a.condition_param1,
                condition_param2: a.condition_param2,
                switch_id: a.switch_id,
                switch_on: a.switch_on,
                switch_on_id: a.switch_on_id,
                switch_off: a.switch_off,
                switch_off_id: a.switch_off_id,
                rating: a.rating,
            }).collect();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_troops(path: &str) -> Vec<TroopInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.troops
        .into_iter()
        .map(|t| TroopInfo {
            id: t.id,
            name: t.name.0,
            auto_alignment: t.auto_alignment,
            appear_randomly: t.appear_randomly,
            terrain_set: t.terrain_set.0.clone(),
            members: t
                .members
                .into_iter()
                .map(|m| TroopMemberInfo {
                    enemy_id: m.enemy_id,
                    x: m.x,
                    y: m.y,
                    invisible: m.invisible,
                })
                .collect(),
            pages: t
                .pages
                .into_iter()
                .map(|p| TroopPageInfo {
                    id: p.id,
                    condition: TroopPageConditionInfo {
                        flags: p.condition.flags,
                        switch_a_id: p.condition.switch_a_id,
                        switch_b_id: p.condition.switch_b_id,
                        variable_id: p.condition.variable_id,
                        variable_value: p.condition.variable_value,
                        turn_a: p.condition.turn_a,
                        turn_b: p.condition.turn_b,
                        fatigue_min: p.condition.fatigue_min,
                        fatigue_max: p.condition.fatigue_max,
                        enemy_id: p.condition.enemy_id,
                        enemy_hp_min: p.condition.enemy_hp_min,
                        enemy_hp_max: p.condition.enemy_hp_max,
                        actor_id: p.condition.actor_id,
                        actor_hp_min: p.condition.actor_hp_min,
                        actor_hp_max: p.condition.actor_hp_max,
                        turn_enemy_id: p.condition.turn_enemy_id,
                        turn_enemy_a: p.condition.turn_enemy_a,
                        turn_enemy_b: p.condition.turn_enemy_b,
                        turn_actor_id: p.condition.turn_actor_id,
                        turn_actor_a: p.condition.turn_actor_a,
                        turn_actor_b: p.condition.turn_actor_b,
                        command_actor_id: p.condition.command_actor_id,
                        command_id: p.condition.command_id,
                    },
                    commands: p.event_commands.iter().map(EventCommandInfo::from).collect(),
                })
                .collect(),
        })
        .collect()
}

pub fn save_troops(path: &str, troops: &[TroopInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in troops {
        if let Some(t) = db.troops.iter_mut().find(|t| t.id == edit.id) {
            t.name = edit.name.clone().into();
            t.auto_alignment = edit.auto_alignment;
            t.appear_randomly = edit.appear_randomly;
            t.terrain_set = DBBitArray(edit.terrain_set.clone());
            t.members = edit.members.iter().enumerate().map(|(i, m)| {
                LdbTroopMember {
                    id: (i + 1) as i32,
                    enemy_id: m.enemy_id,
                    x: m.x,
                    y: m.y,
                    invisible: m.invisible,
                }
            }).collect();
            t.pages = edit.pages.iter().enumerate().map(|(i, p)| {
                LdbTroopPage {
                    id: (i + 1) as i32,
                    condition: LdbTroopPageCondition {
                        flags: p.condition.flags,
                        switch_a_id: p.condition.switch_a_id,
                        switch_b_id: p.condition.switch_b_id,
                        variable_id: p.condition.variable_id,
                        variable_value: p.condition.variable_value,
                        turn_a: p.condition.turn_a,
                        turn_b: p.condition.turn_b,
                        fatigue_min: p.condition.fatigue_min,
                        fatigue_max: p.condition.fatigue_max,
                        enemy_id: p.condition.enemy_id,
                        enemy_hp_min: p.condition.enemy_hp_min,
                        enemy_hp_max: p.condition.enemy_hp_max,
                        actor_id: p.condition.actor_id,
                        actor_hp_min: p.condition.actor_hp_min,
                        actor_hp_max: p.condition.actor_hp_max,
                        turn_enemy_id: p.condition.turn_enemy_id,
                        turn_enemy_a: p.condition.turn_enemy_a,
                        turn_enemy_b: p.condition.turn_enemy_b,
                        turn_actor_id: p.condition.turn_actor_id,
                        turn_actor_a: p.condition.turn_actor_a,
                        turn_actor_b: p.condition.turn_actor_b,
                        command_actor_id: p.condition.command_actor_id,
                        command_id: p.condition.command_id,
                    },
                    event_commands: p.commands.iter().map(LcfEventCommand::from).collect(),
                }
            }).collect();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_common_events(path: &str) -> Vec<CommonEventInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.commonevents
        .into_iter()
        .map(|ce| CommonEventInfo {
            id: ce.id,
            name: ce.name.0,
            trigger: ce.trigger,
            switch_flag: ce.switch_flag,
            switch_id: ce.switch_id,
            commands: ce.event_commands.iter().map(EventCommandInfo::from).collect(),
        })
        .collect()
}

pub fn save_common_events(path: &str, events: &[CommonEventInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in events {
        if let Some(ce) = db.commonevents.iter_mut().find(|c| c.id == edit.id) {
            ce.name = edit.name.clone().into();
            ce.trigger = edit.trigger;
            ce.switch_flag = edit.switch_flag;
            ce.switch_id = edit.switch_id;
            ce.event_commands = edit.commands.iter().map(LcfEventCommand::from).collect();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_switches(path: &str) -> Vec<SwitchInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.switches
        .into_iter()
        .map(|s| SwitchInfo {
            id: s.id,
            name: s.name.0,
        })
        .collect()
}

pub fn save_switches(path: &str, switches: &[SwitchInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in switches {
        if let Some(s) = db.switches.iter_mut().find(|s| s.id == edit.id) {
            s.name = edit.name.clone().into();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_variables(path: &str) -> Vec<VariableInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    db.variables
        .into_iter()
        .map(|v| VariableInfo {
            id: v.id,
            name: v.name.0,
        })
        .collect()
}

pub fn save_variables(path: &str, variables: &[VariableInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    for edit in variables {
        if let Some(v) = db.variables.iter_mut().find(|v| v.id == edit.id) {
            v.name = edit.name.clone().into();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_system(path: &str) -> Option<SystemInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = LdbReader::load(&ldb_path, "auto").ok()?;
    let s = &db.system;
    Some(SystemInfo {
        ldb_id: s.ldb_id,
        boat_name: s.boat_name.0.clone(),
        ship_name: s.ship_name.0.clone(),
        airship_name: s.airship_name.0.clone(),
        title_name: s.title_name.0.clone(),
        gameover_name: s.gameover_name.0.clone(),
        system_name: s.system_name.0.clone(),
        system2_name: s.system2_name.0.clone(),
        party: s.party.clone(),
        title_music_name: s.title_music.name.0.clone(),
        battle_music_name: s.battle_music.name.0.clone(),
        victory_music_name: s.battle_end_music.name.0.clone(),
        gameover_music_name: s.gameover_music.name.0.clone(),
        cursor_sound_name: s.cursor_se.name.0.clone(),
        decision_sound_name: s.decision_se.name.0.clone(),
        cancel_sound_name: s.cancel_se.name.0.clone(),
        buzzer_sound_name: s.buzzer_se.name.0.clone(),
        transition_out: s.transition_out,
        transition_in: s.transition_in,
        battle_start_fadeout: s.battle_start_fadeout,
        battle_start_fadein: s.battle_start_fadein,
        battle_end_fadeout: s.battle_end_fadeout,
        battle_end_fadein: s.battle_end_fadein,
        font_id: s.font_id,
        save_count: s.save_count,
    })
}

pub fn save_system(path: &str, system: &SystemInfo) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    let s = &mut db.system;
    s.boat_name = DBString::new(system.boat_name.clone());
    s.ship_name = DBString::new(system.ship_name.clone());
    s.airship_name = DBString::new(system.airship_name.clone());
    s.title_name = DBString::new(system.title_name.clone());
    s.gameover_name = DBString::new(system.gameover_name.clone());
    s.system_name = DBString::new(system.system_name.clone());
    s.system2_name = DBString::new(system.system2_name.clone());
    s.party = system.party.clone();
    s.title_music.name = DBString::new(system.title_music_name.clone());
    s.battle_music.name = DBString::new(system.battle_music_name.clone());
    s.battle_end_music.name = DBString::new(system.victory_music_name.clone());
    s.gameover_music.name = DBString::new(system.gameover_music_name.clone());
    s.cursor_se.name = DBString::new(system.cursor_sound_name.clone());
    s.decision_se.name = DBString::new(system.decision_sound_name.clone());
    s.cancel_se.name = DBString::new(system.cancel_sound_name.clone());
    s.buzzer_se.name = DBString::new(system.buzzer_sound_name.clone());
    s.transition_out = system.transition_out;
    s.transition_in = system.transition_in;
    s.battle_start_fadeout = system.battle_start_fadeout;
    s.battle_start_fadein = system.battle_start_fadein;
    s.battle_end_fadeout = system.battle_end_fadeout;
    s.battle_end_fadein = system.battle_end_fadein;
    s.font_id = system.font_id;

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_terms(path: &str) -> Option<TermsInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = LdbReader::load(&ldb_path, "auto").ok()?;
    let t = &db.terms;
    Some(TermsInfo {
        new_game: t.new_game.0.clone(),
        load_game: t.load_game.0.clone(),
        exit_game: t.exit_game.0.clone(),
        status: t.status.0.clone(),
        menu_equipment: t.menu_equipment.0.clone(),
        menu_save: t.menu_save.0.clone(),
        menu_quit: t.menu_quit.0.clone(),
        row: t.row.0.clone(),
        order: t.order.0.clone(),
        wait_on: t.wait_on.0.clone(),
        wait_off: t.wait_off.0.clone(),
        level: t.level.0.clone(),
        health_points: t.health_points.0.clone(),
        spirit_points: t.spirit_points.0.clone(),
        normal_status: t.normal_status.0.clone(),
        exp_short: t.exp_short.0.clone(),
        lvl_short: t.lvl_short.0.clone(),
        hp_short: t.hp_short.0.clone(),
        sp_short: t.sp_short.0.clone(),
        sp_cost: t.sp_cost.0.clone(),
        attack: t.attack.0.clone(),
        defense: t.defense.0.clone(),
        spirit: t.spirit.0.clone(),
        agility: t.agility.0.clone(),
        weapon: t.weapon.0.clone(),
        shield: t.shield.0.clone(),
        armor: t.armor.0.clone(),
        helmet: t.helmet.0.clone(),
        accessory: t.accessory.0.clone(),
        command_attack: t.command_attack.0.clone(),
        command_defend: t.command_defend.0.clone(),
        command_item: t.command_item.0.clone(),
        command_skill: t.command_skill.0.clone(),
        battle_auto: t.battle_auto.0.clone(),
        battle_escape: t.battle_escape.0.clone(),
        battle_fight: t.battle_fight.0.clone(),
        gold: t.gold.0.clone(),
        possessed_items: t.possessed_items.0.clone(),
        equipped_items: t.equipped_items.0.clone(),
        save_game_message: t.save_game_message.0.clone(),
        load_game_message: t.load_game_message.0.clone(),
        exit_game_message: t.exit_game_message.0.clone(),
        file: t.file.0.clone(),
        yes: t.yes.0.clone(),
        no: t.no.0.clone(),
        encounter: t.encounter.0.clone(),
        special_combat: t.special_combat.0.clone(),
        escape_success: t.escape_success.0.clone(),
        escape_failure: t.escape_failure.0.clone(),
        victory: t.victory.0.clone(),
        defeat: t.defeat.0.clone(),
        exp_received: t.exp_received.0.clone(),
        gold_recieved_a: t.gold_recieved_a.0.clone(),
        gold_recieved_b: t.gold_recieved_b.0.clone(),
        item_recieved: t.item_recieved.0.clone(),
        attacking: t.attacking.0.clone(),
        enemy_critical: t.enemy_critical.0.clone(),
        actor_critical: t.actor_critical.0.clone(),
        defending: t.defending.0.clone(),
        observing: t.observing.0.clone(),
        focus: t.focus.0.clone(),
        autodestruction: t.autodestruction.0.clone(),
        enemy_escape: t.enemy_escape.0.clone(),
        enemy_transform: t.enemy_transform.0.clone(),
        enemy_damaged: t.enemy_damaged.0.clone(),
        enemy_undamaged: t.enemy_undamaged.0.clone(),
        actor_damaged: t.actor_damaged.0.clone(),
        actor_undamaged: t.actor_undamaged.0.clone(),
        skill_failure_a: t.skill_failure_a.0.clone(),
        skill_failure_b: t.skill_failure_b.0.clone(),
        skill_failure_c: t.skill_failure_c.0.clone(),
        dodge: t.dodge.0.clone(),
        use_item: t.use_item.0.clone(),
        hp_recovery: t.hp_recovery.0.clone(),
        parameter_increase: t.parameter_increase.0.clone(),
        parameter_decrease: t.parameter_decrease.0.clone(),
        enemy_hp_absorbed: t.enemy_hp_absorbed.0.clone(),
        actor_hp_absorbed: t.actor_hp_absorbed.0.clone(),
        resistance_increase: t.resistance_increase.0.clone(),
        resistance_decrease: t.resistance_decrease.0.clone(),
        level_up: t.level_up.0.clone(),
        skill_learned: t.skill_learned.0.clone(),
        battle_start: t.battle_start.0.clone(),
        miss: t.miss.0.clone(),

        // Shop 1
        shop_greeting1: t.shop_greeting1.0.clone(),
        shop_regreeting1: t.shop_regreeting1.0.clone(),
        shop_buy1: t.shop_buy1.0.clone(),
        shop_sell1: t.shop_sell1.0.clone(),
        shop_leave1: t.shop_leave1.0.clone(),
        shop_buy_select1: t.shop_buy_select1.0.clone(),
        shop_buy_number1: t.shop_buy_number1.0.clone(),
        shop_purchased1: t.shop_purchased1.0.clone(),
        shop_sell_select1: t.shop_sell_select1.0.clone(),
        shop_sell_number1: t.shop_sell_number1.0.clone(),
        shop_sold1: t.shop_sold1.0.clone(),

        // Shop 2
        shop_greeting2: t.shop_greeting2.0.clone(),
        shop_regreeting2: t.shop_regreeting2.0.clone(),
        shop_buy2: t.shop_buy2.0.clone(),
        shop_sell2: t.shop_sell2.0.clone(),
        shop_leave2: t.shop_leave2.0.clone(),
        shop_buy_select2: t.shop_buy_select2.0.clone(),
        shop_buy_number2: t.shop_buy_number2.0.clone(),
        shop_purchased2: t.shop_purchased2.0.clone(),
        shop_sell_select2: t.shop_sell_select2.0.clone(),
        shop_sell_number2: t.shop_sell_number2.0.clone(),
        shop_sold2: t.shop_sold2.0.clone(),

        // Shop 3
        shop_greeting3: t.shop_greeting3.0.clone(),
        shop_regreeting3: t.shop_regreeting3.0.clone(),
        shop_buy3: t.shop_buy3.0.clone(),
        shop_sell3: t.shop_sell3.0.clone(),
        shop_leave3: t.shop_leave3.0.clone(),
        shop_buy_select3: t.shop_buy_select3.0.clone(),
        shop_buy_number3: t.shop_buy_number3.0.clone(),
        shop_purchased3: t.shop_purchased3.0.clone(),
        shop_sell_select3: t.shop_sell_select3.0.clone(),
        shop_sell_number3: t.shop_sell_number3.0.clone(),
        shop_sold3: t.shop_sold3.0.clone(),

        // Inn A
        inn_a_greeting_1: t.inn_a_greeting_1.0.clone(),
        inn_a_greeting_2: t.inn_a_greeting_2.0.clone(),
        inn_a_greeting_3: t.inn_a_greeting_3.0.clone(),
        inn_a_accept: t.inn_a_accept.0.clone(),
        inn_a_cancel: t.inn_a_cancel.0.clone(),

        // Inn B
        inn_b_greeting_1: t.inn_b_greeting_1.0.clone(),
        inn_b_greeting_2: t.inn_b_greeting_2.0.clone(),
        inn_b_greeting_3: t.inn_b_greeting_3.0.clone(),
        inn_b_accept: t.inn_b_accept.0.clone(),
        inn_b_cancel: t.inn_b_cancel.0.clone(),

        // Maniac Patch Terms
        maniac_item_received_a: t.maniac_item_received_a.0.clone(),
        maniac_level_up_a: t.maniac_level_up_a.0.clone(),
        maniac_level_up_b: t.maniac_level_up_b.0.clone(),
        maniac_level_up_c: t.maniac_level_up_c.0.clone(),
        maniac_exp_received_a: t.maniac_exp_received_a.0.clone(),
        maniac_skill_learned_a: t.maniac_skill_learned_a.0.clone(),

        // EasyRPG Extended Terms
        easyrpg_item_number_separator: t.easyrpg_item_number_separator.0.clone(),
        easyrpg_skill_cost_separator: t.easyrpg_skill_cost_separator.0.clone(),
        easyrpg_equipment_arrow: t.easyrpg_equipment_arrow.0.clone(),
        easyrpg_status_scene_name: t.easyrpg_status_scene_name.0.clone(),
        easyrpg_status_scene_class: t.easyrpg_status_scene_class.0.clone(),
        easyrpg_status_scene_title: t.easyrpg_status_scene_title.0.clone(),
        easyrpg_status_scene_condition: t.easyrpg_status_scene_condition.0.clone(),
        easyrpg_status_scene_front: t.easyrpg_status_scene_front.0.clone(),
        easyrpg_status_scene_back: t.easyrpg_status_scene_back.0.clone(),
        easyrpg_order_scene_confirm: t.easyrpg_order_scene_confirm.0.clone(),
        easyrpg_order_scene_redo: t.easyrpg_order_scene_redo.0.clone(),
        easyrpg_battle2k3_double_attack: t.easyrpg_battle2k3_double_attack.0.clone(),
        easyrpg_battle2k3_defend: t.easyrpg_battle2k3_defend.0.clone(),
        easyrpg_battle2k3_observe: t.easyrpg_battle2k3_observe.0.clone(),
        easyrpg_battle2k3_charge: t.easyrpg_battle2k3_charge.0.clone(),
        easyrpg_battle2k3_selfdestruct: t.easyrpg_battle2k3_selfdestruct.0.clone(),
        easyrpg_battle2k3_escape: t.easyrpg_battle2k3_escape.0.clone(),
        easyrpg_battle2k3_special_combat_back: t.easyrpg_battle2k3_special_combat_back.0.clone(),
        easyrpg_battle2k3_skill: t.easyrpg_battle2k3_skill.0.clone(),
        easyrpg_battle2k3_item: t.easyrpg_battle2k3_item.0.clone(),
    })
}

pub fn save_terms(path: &str, terms: &TermsInfo) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_ldb_once(&ldb_path)?;

    let t = &mut db.terms;
    t.new_game = terms.new_game.clone().into();
    t.load_game = terms.load_game.clone().into();
    t.exit_game = terms.exit_game.clone().into();
    t.status = terms.status.clone().into();
    t.menu_equipment = terms.menu_equipment.clone().into();
    t.menu_save = terms.menu_save.clone().into();
    t.menu_quit = terms.menu_quit.clone().into();
    t.row = terms.row.clone().into();
    t.order = terms.order.clone().into();
    t.wait_on = terms.wait_on.clone().into();
    t.wait_off = terms.wait_off.clone().into();
    t.level = terms.level.clone().into();
    t.health_points = terms.health_points.clone().into();
    t.spirit_points = terms.spirit_points.clone().into();
    t.normal_status = terms.normal_status.clone().into();
    t.exp_short = terms.exp_short.clone().into();
    t.lvl_short = terms.lvl_short.clone().into();
    t.hp_short = terms.hp_short.clone().into();
    t.sp_short = terms.sp_short.clone().into();
    t.sp_cost = terms.sp_cost.clone().into();
    t.attack = terms.attack.clone().into();
    t.defense = terms.defense.clone().into();
    t.spirit = terms.spirit.clone().into();
    t.agility = terms.agility.clone().into();
    t.weapon = terms.weapon.clone().into();
    t.shield = terms.shield.clone().into();
    t.armor = terms.armor.clone().into();
    t.helmet = terms.helmet.clone().into();
    t.accessory = terms.accessory.clone().into();
    t.command_attack = terms.command_attack.clone().into();
    t.command_defend = terms.command_defend.clone().into();
    t.command_item = terms.command_item.clone().into();
    t.command_skill = terms.command_skill.clone().into();
    t.battle_auto = terms.battle_auto.clone().into();
    t.battle_escape = terms.battle_escape.clone().into();
    t.battle_fight = terms.battle_fight.clone().into();
    t.gold = terms.gold.clone().into();
    t.possessed_items = terms.possessed_items.clone().into();
    t.equipped_items = terms.equipped_items.clone().into();
    t.save_game_message = terms.save_game_message.clone().into();
    t.load_game_message = terms.load_game_message.clone().into();
    t.exit_game_message = terms.exit_game_message.clone().into();
    t.file = terms.file.clone().into();
    t.yes = terms.yes.clone().into();
    t.no = terms.no.clone().into();
    t.encounter = terms.encounter.clone().into();
    t.special_combat = terms.special_combat.clone().into();
    t.escape_success = terms.escape_success.clone().into();
    t.escape_failure = terms.escape_failure.clone().into();
    t.victory = terms.victory.clone().into();
    t.defeat = terms.defeat.clone().into();
    t.exp_received = terms.exp_received.clone().into();
    t.gold_recieved_a = terms.gold_recieved_a.clone().into();
    t.gold_recieved_b = terms.gold_recieved_b.clone().into();
    t.item_recieved = terms.item_recieved.clone().into();
    t.attacking = terms.attacking.clone().into();
    t.enemy_critical = terms.enemy_critical.clone().into();
    t.actor_critical = terms.actor_critical.clone().into();
    t.defending = terms.defending.clone().into();
    t.observing = terms.observing.clone().into();
    t.focus = terms.focus.clone().into();
    t.autodestruction = terms.autodestruction.clone().into();
    t.enemy_escape = terms.enemy_escape.clone().into();
    t.enemy_transform = terms.enemy_transform.clone().into();
    t.enemy_damaged = terms.enemy_damaged.clone().into();
    t.enemy_undamaged = terms.enemy_undamaged.clone().into();
    t.actor_damaged = terms.actor_damaged.clone().into();
    t.actor_undamaged = terms.actor_undamaged.clone().into();
    t.skill_failure_a = terms.skill_failure_a.clone().into();
    t.skill_failure_b = terms.skill_failure_b.clone().into();
    t.skill_failure_c = terms.skill_failure_c.clone().into();
    t.dodge = terms.dodge.clone().into();
    t.use_item = terms.use_item.clone().into();
    t.hp_recovery = terms.hp_recovery.clone().into();
    t.parameter_increase = terms.parameter_increase.clone().into();
    t.parameter_decrease = terms.parameter_decrease.clone().into();
    t.enemy_hp_absorbed = terms.enemy_hp_absorbed.clone().into();
    t.actor_hp_absorbed = terms.actor_hp_absorbed.clone().into();
    t.resistance_increase = terms.resistance_increase.clone().into();
    t.resistance_decrease = terms.resistance_decrease.clone().into();
    t.level_up = terms.level_up.clone().into();
    t.skill_learned = terms.skill_learned.clone().into();
    t.battle_start = terms.battle_start.clone().into();
    t.miss = terms.miss.clone().into();

    // Shop 1
    t.shop_greeting1 = terms.shop_greeting1.clone().into();
    t.shop_regreeting1 = terms.shop_regreeting1.clone().into();
    t.shop_buy1 = terms.shop_buy1.clone().into();
    t.shop_sell1 = terms.shop_sell1.clone().into();
    t.shop_leave1 = terms.shop_leave1.clone().into();
    t.shop_buy_select1 = terms.shop_buy_select1.clone().into();
    t.shop_buy_number1 = terms.shop_buy_number1.clone().into();
    t.shop_purchased1 = terms.shop_purchased1.clone().into();
    t.shop_sell_select1 = terms.shop_sell_select1.clone().into();
    t.shop_sell_number1 = terms.shop_sell_number1.clone().into();
    t.shop_sold1 = terms.shop_sold1.clone().into();

    // Shop 2
    t.shop_greeting2 = terms.shop_greeting2.clone().into();
    t.shop_regreeting2 = terms.shop_regreeting2.clone().into();
    t.shop_buy2 = terms.shop_buy2.clone().into();
    t.shop_sell2 = terms.shop_sell2.clone().into();
    t.shop_leave2 = terms.shop_leave2.clone().into();
    t.shop_buy_select2 = terms.shop_buy_select2.clone().into();
    t.shop_buy_number2 = terms.shop_buy_number2.clone().into();
    t.shop_purchased2 = terms.shop_purchased2.clone().into();
    t.shop_sell_select2 = terms.shop_sell_select2.clone().into();
    t.shop_sell_number2 = terms.shop_sell_number2.clone().into();
    t.shop_sold2 = terms.shop_sold2.clone().into();

    // Shop 3
    t.shop_greeting3 = terms.shop_greeting3.clone().into();
    t.shop_regreeting3 = terms.shop_regreeting3.clone().into();
    t.shop_buy3 = terms.shop_buy3.clone().into();
    t.shop_sell3 = terms.shop_sell3.clone().into();
    t.shop_leave3 = terms.shop_leave3.clone().into();
    t.shop_buy_select3 = terms.shop_buy_select3.clone().into();
    t.shop_buy_number3 = terms.shop_buy_number3.clone().into();
    t.shop_purchased3 = terms.shop_purchased3.clone().into();
    t.shop_sell_select3 = terms.shop_sell_select3.clone().into();
    t.shop_sell_number3 = terms.shop_sell_number3.clone().into();
    t.shop_sold3 = terms.shop_sold3.clone().into();

    // Inn A
    t.inn_a_greeting_1 = terms.inn_a_greeting_1.clone().into();
    t.inn_a_greeting_2 = terms.inn_a_greeting_2.clone().into();
    t.inn_a_greeting_3 = terms.inn_a_greeting_3.clone().into();
    t.inn_a_accept = terms.inn_a_accept.clone().into();
    t.inn_a_cancel = terms.inn_a_cancel.clone().into();

    // Inn B
    t.inn_b_greeting_1 = terms.inn_b_greeting_1.clone().into();
    t.inn_b_greeting_2 = terms.inn_b_greeting_2.clone().into();
    t.inn_b_greeting_3 = terms.inn_b_greeting_3.clone().into();
    t.inn_b_accept = terms.inn_b_accept.clone().into();
    t.inn_b_cancel = terms.inn_b_cancel.clone().into();

    // Maniac Patch Terms
    t.maniac_item_received_a = terms.maniac_item_received_a.clone().into();
    t.maniac_level_up_a = terms.maniac_level_up_a.clone().into();
    t.maniac_level_up_b = terms.maniac_level_up_b.clone().into();
    t.maniac_level_up_c = terms.maniac_level_up_c.clone().into();
    t.maniac_exp_received_a = terms.maniac_exp_received_a.clone().into();
    t.maniac_skill_learned_a = terms.maniac_skill_learned_a.clone().into();

    // EasyRPG Extended Terms
    t.easyrpg_item_number_separator = terms.easyrpg_item_number_separator.clone().into();
    t.easyrpg_skill_cost_separator = terms.easyrpg_skill_cost_separator.clone().into();
    t.easyrpg_equipment_arrow = terms.easyrpg_equipment_arrow.clone().into();
    t.easyrpg_status_scene_name = terms.easyrpg_status_scene_name.clone().into();
    t.easyrpg_status_scene_class = terms.easyrpg_status_scene_class.clone().into();
    t.easyrpg_status_scene_title = terms.easyrpg_status_scene_title.clone().into();
    t.easyrpg_status_scene_condition = terms.easyrpg_status_scene_condition.clone().into();
    t.easyrpg_status_scene_front = terms.easyrpg_status_scene_front.clone().into();
    t.easyrpg_status_scene_back = terms.easyrpg_status_scene_back.clone().into();
    t.easyrpg_order_scene_confirm = terms.easyrpg_order_scene_confirm.clone().into();
    t.easyrpg_order_scene_redo = terms.easyrpg_order_scene_redo.clone().into();
    t.easyrpg_battle2k3_double_attack = terms.easyrpg_battle2k3_double_attack.clone().into();
    t.easyrpg_battle2k3_defend = terms.easyrpg_battle2k3_defend.clone().into();
    t.easyrpg_battle2k3_observe = terms.easyrpg_battle2k3_observe.clone().into();
    t.easyrpg_battle2k3_charge = terms.easyrpg_battle2k3_charge.clone().into();
    t.easyrpg_battle2k3_selfdestruct = terms.easyrpg_battle2k3_selfdestruct.clone().into();
    t.easyrpg_battle2k3_escape = terms.easyrpg_battle2k3_escape.clone().into();
    t.easyrpg_battle2k3_special_combat_back = terms.easyrpg_battle2k3_special_combat_back.clone().into();
    t.easyrpg_battle2k3_skill = terms.easyrpg_battle2k3_skill.clone().into();
    t.easyrpg_battle2k3_item = terms.easyrpg_battle2k3_item.clone().into();

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Save File Management
// ---------------------------------------------------------------------------

fn load_save_slot_info(dir: &str, file_name: String) -> SaveSlotInfo {
    let lsd_path = Path::new(dir).join(&file_name);
    match LsdReader::load(&lsd_path, "auto") {
        Ok(save) => {
            let party = save
                .actors
                .iter()
                .map(|a| SavePartyMember {
                    id: a.id,
                    name: a.name.0.clone(),
                    level: a.level,
                    current_hp: a.current_hp,
                    current_sp: a.current_sp,
                })
                .collect();

            let mut inventory = Vec::new();
            for (i, &id) in save.inventory.item_ids.iter().enumerate() {
                let count = save.inventory.item_counts.get(i).copied().unwrap_or(1) as i32;
                inventory.push((id as i32, count));
            }

            SaveSlotInfo {
                file_name,
                hero_name: save.title.hero_name.0.clone(),
                hero_level: save.title.hero_level,
                hero_hp: save.title.hero_hp,
                timestamp: format_unix_timestamp(lcf_core::ReaderUtil::to_unix_timestamp(save.title.timestamp)),
                map_id: save.party_location.map_id,
                position_x: save.party_location.position_x,
                position_y: save.party_location.position_y,
                gold: save.inventory.gold,
                party,
                inventory,
                error: None,
            }
        }
        Err(e) => SaveSlotInfo {
            file_name,
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

pub fn list_saves(path: &str) -> Vec<SaveSlotInfo> {
    let entries = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.starts_with("save") && name.ends_with(".lsd")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    files
        .into_iter()
        .map(|entry| load_save_slot_info(path, entry.file_name().to_string_lossy().to_string()))
        .collect()
}

pub fn reload_save_slot(path: &str, file_name: &str) -> SaveSlotInfo {
    load_save_slot_info(path, file_name.to_string())
}

pub fn save_save_slot(path: &str, file_name: &str, info: &SaveSlotInfo) -> Result<(), String> {
    let lsd_path = Path::new(path).join(file_name);
    let mut save = LsdReader::load(&lsd_path, "auto").map_err(|e| e.to_string())?;

    backup_file_once(&lsd_path)?;

    // Update hero & party
    save.title.hero_name = DBString::new(info.hero_name.clone());
    save.title.hero_level = info.hero_level;
    save.title.hero_hp = info.hero_hp;
    save.party_location.map_id = info.map_id;
    save.party_location.position_x = info.position_x;
    save.party_location.position_y = info.position_y;
    save.inventory.gold = info.gold;

    for edit in &info.party {
        if let Some(actor) = save.actors.iter_mut().find(|a| a.id == edit.id) {
            actor.name = edit.name.clone().into();
            actor.level = edit.level;
            actor.current_hp = edit.current_hp;
            actor.current_sp = edit.current_sp;
        }
    }

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };

    LsdReader::save(&lsd_path, &save, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// XML Import / Export
// ---------------------------------------------------------------------------

pub fn export_database_to_xml(path: &str, dest_xml_path: &Path) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    let engine = engine_version_for(&db);
    LdbReader::save_xml(dest_xml_path, &db, engine).map_err(|e| e.to_string())
}

pub fn export_map_to_xml(path: &str, map_id: i32, dest_xml_path: &Path) -> Result<(), String> {
    let map_path = Path::new(path).join(map_filename(map_id));
    let map = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmuReader::save_xml(dest_xml_path, &map, engine).map_err(|e| e.to_string())
}

pub fn export_tree_to_xml(path: &str, dest_xml_path: &Path) -> Result<(), String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save_xml(dest_xml_path, &tree, engine).map_err(|e| e.to_string())
}

pub fn export_save_to_xml(path: &str, save_file: &str, dest_xml_path: &Path) -> Result<(), String> {
    let lsd_path = Path::new(path).join(save_file);
    let save = LsdReader::load(&lsd_path, "auto").map_err(|e| e.to_string())?;
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LsdReader::save_xml(dest_xml_path, &save, engine).map_err(|e| e.to_string())
}

pub fn resize_map_layers(
    old_lower: &[i32],
    old_upper: &[i32],
    old_w: i32,
    old_h: i32,
    new_w: i32,
    new_h: i32,
    anchor: AnchorOrigin,
) -> (Vec<i32>, Vec<i32>) {
    let off_x = match anchor {
        AnchorOrigin::TopLeft | AnchorOrigin::CenterLeft | AnchorOrigin::BottomLeft => 0,
        AnchorOrigin::TopCenter | AnchorOrigin::Center | AnchorOrigin::BottomCenter => (new_w - old_w) / 2,
        AnchorOrigin::TopRight | AnchorOrigin::CenterRight | AnchorOrigin::BottomRight => new_w - old_w,
    };
    let off_y = match anchor {
        AnchorOrigin::TopLeft | AnchorOrigin::TopCenter | AnchorOrigin::TopRight => 0,
        AnchorOrigin::CenterLeft | AnchorOrigin::Center | AnchorOrigin::CenterRight => (new_h - old_h) / 2,
        AnchorOrigin::BottomLeft | AnchorOrigin::BottomCenter | AnchorOrigin::BottomRight => new_h - old_h,
    };

    let total = (new_w * new_h) as usize;
    let mut new_lower = vec![4000; total];
    let mut new_upper = vec![10000; total];

    for ny in 0..new_h {
        for nx in 0..new_w {
            let ox = nx - off_x;
            let oy = ny - off_y;
            let n_idx = (ny * new_w + nx) as usize;
            if ox >= 0 && ox < old_w && oy >= 0 && oy < old_h {
                let o_idx = (oy * old_w + ox) as usize;
                if o_idx < old_lower.len() {
                    new_lower[n_idx] = old_lower[o_idx];
                }
                if o_idx < old_upper.len() {
                    new_upper[n_idx] = old_upper[o_idx];
                }
            }
        }
    }

    (new_lower, new_upper)
}

pub fn get_map_tree(path: &str) -> Vec<MapTreeItem> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let tree = match LmtReader::load(&lmt_path, "auto") {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    tree.maps
        .into_iter()
        .map(|m| MapTreeItem {
            id: m.id,
            name: m.name.0,
            parent_map: m.parent_map,
            indentation: m.indentation,
            expanded_node: m.expanded_node,
            music_type: m.music_type,
            music_name: m.music.name.0,
            background_type: m.background_type,
            background_name: m.background_name.0,
            teleport: m.teleport,
            escape: m.escape,
            save: m.save,
            encounter_steps: m.encounter_steps,
            encounters: m.encounters.into_iter().map(|e| e.troop_id).collect(),
        })
        .collect()
}

pub fn get_start_points(path: &str) -> StartPointInfo {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let tree = match LmtReader::load(&lmt_path, "auto") {
        Ok(t) => t,
        Err(_) => return StartPointInfo::default(),
    };
    StartPointInfo {
        party_map_id: tree.start.party_map_id,
        party_x: tree.start.party_x,
        party_y: tree.start.party_y,
        boat_map_id: tree.start.boat_map_id,
        boat_x: tree.start.boat_x,
        boat_y: tree.start.boat_y,
        ship_map_id: tree.start.ship_map_id,
        ship_x: tree.start.ship_x,
        ship_y: tree.start.ship_y,
        airship_map_id: tree.start.airship_map_id,
        airship_x: tree.start.airship_x,
        airship_y: tree.start.airship_y,
    }
}

pub fn save_start_points(path: &str, start: &StartPointInfo) -> Result<(), String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let mut tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&lmt_path)?;

    tree.start.party_map_id = start.party_map_id;
    tree.start.party_x = start.party_x;
    tree.start.party_y = start.party_y;
    tree.start.boat_map_id = start.boat_map_id;
    tree.start.boat_x = start.boat_x;
    tree.start.boat_y = start.boat_y;
    tree.start.ship_map_id = start.ship_map_id;
    tree.start.ship_x = start.ship_x;
    tree.start.ship_y = start.ship_y;
    tree.start.airship_map_id = start.airship_map_id;
    tree.start.airship_x = start.airship_x;
    tree.start.airship_y = start.airship_y;

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| e.to_string())
}

pub fn get_map_properties(path: &str, map_id: i32) -> Result<MapPropertiesInfo, String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    let map_info = tree.maps.iter().find(|m| m.id == map_id).cloned().unwrap_or_default();

    let map_path = Path::new(path).join(map_filename(map_id));
    let lmu = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;

    Ok(MapPropertiesInfo {
        id: map_id,
        name: map_info.name.0,
        parent_map: map_info.parent_map,
        chipset_id: lmu.chipset_id,
        width: lmu.width,
        height: lmu.height,
        scroll_type: lmu.scroll_type,
        parallax_name: lmu.parallax_name.0,
        parallax_loop_x: lmu.parallax_loop_x,
        parallax_loop_y: lmu.parallax_loop_y,
        parallax_sx: lmu.parallax_sx,
        parallax_sy: lmu.parallax_sy,
        music_type: map_info.music_type,
        music_name: map_info.music.name.0,
        background_type: map_info.background_type,
        background_name: map_info.background_name.0,
        teleport: map_info.teleport,
        escape: map_info.escape,
        save: map_info.save,
        encounter_steps: map_info.encounter_steps,
        encounters: map_info.encounters.iter().map(|e| e.troop_id).collect(),
    })
}

pub fn save_map_properties(
    path: &str,
    map_id: i32,
    props: &MapPropertiesInfo,
    anchor: AnchorOrigin,
) -> Result<(), String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let mut tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&lmt_path)?;

    if let Some(m) = tree.maps.iter_mut().find(|m| m.id == map_id) {
        m.name = DBString::new(props.name.clone());
        m.parent_map = props.parent_map;
        m.music_type = props.music_type;
        m.music.name = DBString::new(props.music_name.clone());
        m.background_type = props.background_type;
        m.background_name = DBString::new(props.background_name.clone());
        m.teleport = props.teleport;
        m.escape = props.escape;
        m.save = props.save;
        m.encounter_steps = props.encounter_steps;
        m.encounters = props.encounters.iter().enumerate().map(|(i, &tid)| {
            LmtEncounter { id: (i + 1) as i32, troop_id: tid }
        }).collect();
    }

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| e.to_string())?;

    let map_path = Path::new(path).join(map_filename(map_id));
    let mut map = LmuReader::load(&map_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&map_path)?;

    map.chipset_id = props.chipset_id;
    map.scroll_type = props.scroll_type;
    map.parallax_name = DBString::new(props.parallax_name.clone());
    map.parallax_loop_x = props.parallax_loop_x;
    map.parallax_loop_y = props.parallax_loop_y;
    map.parallax_sx = props.parallax_sx;
    map.parallax_sy = props.parallax_sy;

    if map.width != props.width || map.height != props.height {
        let old_lower: Vec<i32> = map.lower_layer.iter().map(|&v| v as i32).collect();
        let old_upper: Vec<i32> = map.upper_layer.iter().map(|&v| v as i32).collect();
        let (new_lower, new_upper) = resize_map_layers(
            &old_lower,
            &old_upper,
            map.width,
            map.height,
            props.width,
            props.height,
            anchor,
        );
        map.width = props.width;
        map.height = props.height;
        map.lower_layer = new_lower.into_iter().map(|v| v as i16).collect();
        map.upper_layer = new_upper.into_iter().map(|v| v as i16).collect();
    }

    LmuReader::save(&map_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_new_map(
    path: &str,
    parent_id: i32,
    name: &str,
    width: i32,
    height: i32,
    chipset_id: i32,
) -> Result<i32, String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let mut tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&lmt_path)?;

    let max_id = tree.maps.iter().map(|m| m.id).max().unwrap_or(0);
    let new_id = max_id + 1;

    let parent_indent = tree.maps.iter().find(|m| m.id == parent_id).map(|m| m.indentation).unwrap_or(0);
    let mut map_info = LmtMapInfo::default();
    map_info.id = new_id;
    map_info.name = DBString::new(name.to_string());
    map_info.parent_map = parent_id;
    map_info.indentation = if parent_id > 0 { parent_indent + 1 } else { 0 };
    map_info.r#type = 1;

    tree.maps.push(map_info);
    tree.tree_order.push(new_id);

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| e.to_string())?;

    let map_path = Path::new(path).join(map_filename(new_id));
    let mut map = LmuMap::default();
    map.chipset_id = chipset_id;
    map.width = width;
    map.height = height;
    let total = (width * height) as usize;
    map.lower_layer = vec![4000; total];
    map.upper_layer = vec![10000; total];

    LmuReader::save(&map_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    Ok(new_id)
}

pub fn duplicate_map(path: &str, source_map_id: i32) -> Result<i32, String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let mut tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&lmt_path)?;

    let max_id = tree.maps.iter().map(|m| m.id).max().unwrap_or(0);
    let new_id = max_id + 1;

    let src_info = tree.maps.iter().find(|m| m.id == source_map_id).cloned().ok_or("Source map not found")?;
    let mut new_info = src_info.clone();
    new_info.id = new_id;
    new_info.name = DBString::new(format!("{} (Copy)", src_info.name.0));
    tree.maps.push(new_info);
    tree.tree_order.push(new_id);

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| e.to_string())?;

    let src_path = Path::new(path).join(map_filename(source_map_id));
    let dst_path = Path::new(path).join(map_filename(new_id));
    if src_path.exists() {
        let map = LmuReader::load(&src_path, "auto").map_err(|e| e.to_string())?;
        LmuReader::save(&dst_path, &map, engine, "auto").map_err(|e| e.to_string())?;
    }

    Ok(new_id)
}

pub fn delete_map(path: &str, map_id: i32) -> Result<(), String> {
    let lmt_path = Path::new(path).join("RPG_RT.lmt");
    let mut tree = LmtReader::load(&lmt_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&lmt_path)?;

    tree.maps.retain(|m| m.id != map_id);
    tree.tree_order.retain(|&id| id != map_id);

    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let engine = match LdbReader::load(&ldb_path, "auto") {
        Ok(db) => engine_version_for(&db),
        Err(_) => EngineVersion::Engine2000,
    };
    LmtReader::save(&lmt_path, &tree, engine, "auto").map_err(|e| e.to_string())?;

    let map_path = Path::new(path).join(map_filename(map_id));
    if map_path.exists() {
        let _ = fs::remove_file(map_path);
    }

    Ok(())
}

pub fn get_chipsets(path: &str) -> Vec<ChipsetInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    db.chipsets
        .into_iter()
        .map(|cs| ChipsetInfo {
            id: cs.id,
            name: cs.name.0,
            chipset_name: cs.chipset_name.0,
            terrain_data: cs.terrain_data,
            passable_data_lower: cs.passable_data_lower,
            passable_data_upper: cs.passable_data_upper,
            animation_type: cs.animation_type,
            animation_speed: cs.animation_speed,
        })
        .collect()
}

pub fn save_chipsets(path: &str, chipsets: &[ChipsetInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&ldb_path)?;

    db.chipsets = chipsets
        .iter()
        .map(|cs| LdbChipset {
            id: cs.id,
            name: DBString::new(cs.name.clone()),
            chipset_name: DBString::new(cs.chipset_name.clone()),
            terrain_data: cs.terrain_data.clone(),
            passable_data_lower: cs.passable_data_lower.clone(),
            passable_data_upper: cs.passable_data_upper.clone(),
            animation_type: cs.animation_type,
            animation_speed: cs.animation_speed,
        })
        .collect();

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())
}

pub fn get_states(path: &str) -> Vec<StateInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    db.states
        .into_iter()
        .map(|s| StateInfo {
            id: s.id,
            name: s.name.0,
            state_type: s.r#type,
            color: s.color,
            priority: s.priority,
            restriction: s.restriction,
            a_rate: s.a_rate,
            b_rate: s.b_rate,
            c_rate: s.c_rate,
            d_rate: s.d_rate,
            e_rate: s.e_rate,
            hold_turn: s.hold_turn,
            auto_release_prob: s.auto_release_prob,
            release_by_damage: s.release_by_damage,
            affect_attack: s.affect_attack,
            affect_defense: s.affect_defense,
            affect_spirit: s.affect_spirit,
            affect_agility: s.affect_agility,
            reduce_hit_ratio: s.reduce_hit_ratio,
            avoid_attacks: s.avoid_attacks,
            reflect_magic: s.reflect_magic,
            cursed: s.cursed,
            hp_change_type: s.hp_change_type,
            hp_change_val: s.hp_change_val,
            hp_change_max: s.hp_change_max,
            hp_change_map_steps: s.hp_change_map_steps,
            hp_change_map_val: s.hp_change_map_val,
            sp_change_type: s.sp_change_type,
            sp_change_val: s.sp_change_val,
            sp_change_max: s.sp_change_max,
            sp_change_map_steps: s.sp_change_map_steps,
            sp_change_map_val: s.sp_change_map_val,
            message_actor: s.message_actor.0,
            message_enemy: s.message_enemy.0,
            message_already: s.message_already.0,
            message_affected: s.message_affected.0,
            message_recovery: s.message_recovery.0,
        })
        .collect()
}

pub fn save_states(path: &str, states: &[StateInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&ldb_path)?;

    db.states = states
        .iter()
        .map(|s| {
            let mut ldb_s = LdbState::default();
            ldb_s.id = s.id;
            ldb_s.name = DBString::new(s.name.clone());
            ldb_s.r#type = s.state_type;
            ldb_s.color = s.color;
            ldb_s.priority = s.priority;
            ldb_s.restriction = s.restriction;
            ldb_s.a_rate = s.a_rate;
            ldb_s.b_rate = s.b_rate;
            ldb_s.c_rate = s.c_rate;
            ldb_s.d_rate = s.d_rate;
            ldb_s.e_rate = s.e_rate;
            ldb_s.hold_turn = s.hold_turn;
            ldb_s.auto_release_prob = s.auto_release_prob;
            ldb_s.release_by_damage = s.release_by_damage;
            ldb_s.affect_attack = s.affect_attack;
            ldb_s.affect_defense = s.affect_defense;
            ldb_s.affect_spirit = s.affect_spirit;
            ldb_s.affect_agility = s.affect_agility;
            ldb_s.reduce_hit_ratio = s.reduce_hit_ratio;
            ldb_s.avoid_attacks = s.avoid_attacks;
            ldb_s.reflect_magic = s.reflect_magic;
            ldb_s.cursed = s.cursed;
            ldb_s.hp_change_type = s.hp_change_type;
            ldb_s.hp_change_val = s.hp_change_val;
            ldb_s.hp_change_max = s.hp_change_max;
            ldb_s.hp_change_map_steps = s.hp_change_map_steps;
            ldb_s.hp_change_map_val = s.hp_change_map_val;
            ldb_s.sp_change_type = s.sp_change_type;
            ldb_s.sp_change_val = s.sp_change_val;
            ldb_s.sp_change_max = s.sp_change_max;
            ldb_s.sp_change_map_steps = s.sp_change_map_steps;
            ldb_s.sp_change_map_val = s.sp_change_map_val;
            ldb_s.message_actor = DBString::new(s.message_actor.clone());
            ldb_s.message_enemy = DBString::new(s.message_enemy.clone());
            ldb_s.message_already = DBString::new(s.message_already.clone());
            ldb_s.message_affected = DBString::new(s.message_affected.clone());
            ldb_s.message_recovery = DBString::new(s.message_recovery.clone());
            ldb_s
        })
        .collect();

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())
}

pub fn get_terrains(path: &str) -> Vec<TerrainInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    db.terrains
        .into_iter()
        .map(|t| TerrainInfo {
            id: t.id,
            name: t.name.0,
            damage: t.damage,
            encounter_rate: t.encounter_rate,
            background_name: t.background_name.0,
            boat_pass: t.boat_pass,
            ship_pass: t.ship_pass,
            airship_pass: t.airship_pass,
            airship_land: t.airship_land,
            bush_depth: t.bush_depth,
            footstep_name: t.footstep.name.0,
        })
        .collect()
}

pub fn save_terrains(path: &str, terrains: &[TerrainInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&ldb_path)?;

    db.terrains = terrains
        .iter()
        .map(|t| {
            let mut ldb_t = LdbTerrain::default();
            ldb_t.id = t.id;
            ldb_t.name = DBString::new(t.name.clone());
            ldb_t.damage = t.damage;
            ldb_t.encounter_rate = t.encounter_rate;
            ldb_t.background_name = DBString::new(t.background_name.clone());
            ldb_t.boat_pass = t.boat_pass;
            ldb_t.ship_pass = t.ship_pass;
            ldb_t.airship_pass = t.airship_pass;
            ldb_t.airship_land = t.airship_land;
            ldb_t.bush_depth = t.bush_depth;
            ldb_t.footstep.name = DBString::new(t.footstep_name.clone());
            ldb_t
        })
        .collect();

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())
}

pub fn get_animations(path: &str) -> Vec<AnimationInfo> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let db = match LdbReader::load(&ldb_path, "auto") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    db.animations
        .into_iter()
        .map(|a| AnimationInfo {
            id: a.id,
            name: a.name.0,
            animation_name: a.animation_name.0,
            large: a.large,
            scope: a.scope,
            position: a.position,
            frame_count: a.frames.len(),
            timings: a.timings.into_iter().map(|t| AnimationTimingInfo {
                id: t.id,
                frame: t.frame,
                se_name: t.se.name.0,
                flash_scope: t.flash_scope,
                flash_red: t.flash_red,
                flash_green: t.flash_green,
                flash_blue: t.flash_blue,
                flash_power: t.flash_power,
                screen_shake: t.screen_shake,
            }).collect(),
        })
        .collect()
}

pub fn save_animations(path: &str, animations: &[AnimationInfo]) -> Result<(), String> {
    let ldb_path = Path::new(path).join("RPG_RT.ldb");
    let mut db = LdbReader::load(&ldb_path, "auto").map_err(|e| e.to_string())?;
    backup_file_once(&ldb_path)?;

    for edit in animations {
        if let Some(ldb_a) = db.animations.iter_mut().find(|a| a.id == edit.id) {
            ldb_a.name = DBString::new(edit.name.clone());
            ldb_a.animation_name = DBString::new(edit.animation_name.clone());
            ldb_a.large = edit.large;
            ldb_a.scope = edit.scope;
            ldb_a.position = edit.position;
            
            if ldb_a.frames.len() != edit.frame_count {
                ldb_a.frames.resize_with(edit.frame_count, || LdbAnimationFrame::default());
                for (idx, f) in ldb_a.frames.iter_mut().enumerate() {
                    f.id = (idx + 1) as i32;
                }
            }

            ldb_a.timings = edit.timings.iter().enumerate().map(|(i, t)| {
                let mut se = LdbSound::default();
                se.name = DBString::new(t.se_name.clone());
                se.volume = 100;
                se.tempo = 100;
                se.balance = 50;
                LdbAnimationTiming {
                    id: (i + 1) as i32,
                    frame: t.frame,
                    se,
                    flash_scope: t.flash_scope,
                    flash_red: t.flash_red,
                    flash_green: t.flash_green,
                    flash_blue: t.flash_blue,
                    flash_power: t.flash_power,
                    screen_shake: t.screen_shake,
                }
            }).collect();
        }
    }

    let engine = engine_version_for(&db);
    LdbReader::save(&ldb_path, &db, engine, "auto").map_err(|e| e.to_string())
}
