use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::lcf_bridge::{
    self, ActorInfo, AnimationInfo, AttributeInfo, ChipsetInfo, ClassInfo, CommonEventInfo,
    EnemyInfo, ItemInfo, MapTreeItem, SkillInfo, StateInfo, SwitchInfo, SystemInfo, TermsInfo,
    TerrainInfo, TroopInfo, VariableInfo,
};
use crate::theme::AppTheme;
use crate::views::save_view::SaveSlotView;

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ViewMode {
    Maps,
    Database,
    Saves,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DbCategory {
    Actors,
    Classes,
    Items,
    Skills,
    Attributes,
    Enemies,
    Troops,
    CommonEvents,
    Switches,
    Variables,
    Chipsets,
    States,
    Terrains,
    Animations,
    System,
    Terms,
}

impl DbCategory {
    /// Localized display name, matching the `db.*` locale keys used
    /// throughout the category list and detail views.
    pub fn label(&self) -> String {
        match self {
            DbCategory::Actors => rust_i18n::t!("db.actors").to_string(),
            DbCategory::Classes => rust_i18n::t!("db.classes").to_string(),
            DbCategory::Items => rust_i18n::t!("db.items").to_string(),
            DbCategory::Skills => rust_i18n::t!("db.skills").to_string(),
            DbCategory::Attributes => rust_i18n::t!("db.attributes").to_string(),
            DbCategory::Enemies => rust_i18n::t!("db.enemies").to_string(),
            DbCategory::Troops => rust_i18n::t!("db.troops").to_string(),
            DbCategory::CommonEvents => rust_i18n::t!("db.common_events").to_string(),
            DbCategory::Switches => rust_i18n::t!("db.switches").to_string(),
            DbCategory::Variables => rust_i18n::t!("db.variables").to_string(),
            DbCategory::Chipsets => rust_i18n::t!("db.chipsets").to_string(),
            DbCategory::States => rust_i18n::t!("db.states").to_string(),
            DbCategory::Terrains => rust_i18n::t!("db.terrains").to_string(),
            DbCategory::Animations => rust_i18n::t!("db.animations").to_string(),
            DbCategory::System => rust_i18n::t!("db.system").to_string(),
            DbCategory::Terms => rust_i18n::t!("db.terms").to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppPersistentData {
    pub theme: AppTheme,
    pub locale: String,
    pub recent_projects: Vec<String>,
    pub last_project: Option<String>,
    pub rtp_path: Option<String>,
}

impl Default for AppPersistentData {
    fn default() -> Self {
        let detected = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        let lang = if detected.starts_with("es") {
            "es"
        } else if detected.starts_with("ja") {
            "ja"
        } else if detected.starts_with("de") {
            "de"
        } else if detected.starts_with("fr") {
            "fr"
        } else if detected.starts_with("it") {
            "it"
        } else if detected.starts_with("pt") {
            "pt-BR"
        } else if detected.starts_with("zh") {
            "zh-CN"
        } else {
            "en"
        };

        Self {
            theme: AppTheme::default(),
            locale: lang.to_string(),
            recent_projects: Vec::new(),
            last_project: None,
            rtp_path: Self::detect_standard_rtp_path().map(|p| p.to_string_lossy().to_string()),
        }
    }
}

impl AppPersistentData {
    /// Detects official RPG Maker 2000/2003 / EasyRPG RTP paths via standard environment variables and system paths
    pub fn detect_standard_rtp_path() -> Option<PathBuf> {
        // 1. Check official EasyRPG environment variables
        let env_vars = ["RPG2K3_RTP_PATH", "RPG2K_RTP_PATH", "RPG_RTP_PATH"];
        for var in &env_vars {
            if let Ok(val) = std::env::var(var) {
                for candidate in val.split(';').chain(val.split(':')) {
                    let p = PathBuf::from(candidate.trim());
                    if p.exists() && p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }

        // 2. Check standard Windows Program Files / Common Files installation paths
        let standard_windows_paths = [
            r"C:\Program Files (x86)\Common Files\Enterbrain\RPG2003\RTP",
            r"C:\Program Files (x86)\Common Files\ASCII\RPG2000\RTP",
            r"C:\Program Files\Common Files\Enterbrain\RPG2003\RTP",
            r"C:\Program Files\Common Files\ASCII\RPG2000\RTP",
            r"D:\programacion\RTP",
        ];

        for path_str in &standard_windows_paths {
            let p = PathBuf::from(path_str);
            if p.exists() && p.is_dir() {
                return Some(p);
            }
        }

        None
    }

    pub fn get_effective_rtp_path(&self) -> Option<PathBuf> {
        if let Some(custom) = &self.rtp_path {
            let p = PathBuf::from(custom);
            if p.exists() {
                return Some(p);
            }
        }
        Self::detect_standard_rtp_path()
    }

    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))?;
        let dir = PathBuf::from(home).join(".easy_editor");
        let _ = fs::create_dir_all(&dir);
        Some(dir.join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppPersistentData>(&data) {
                    if cfg.rtp_path.is_none() {
                        cfg.rtp_path = Self::detect_standard_rtp_path().map(|p| p.to_string_lossy().to_string());
                    }
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, data);
            }
        }
    }

    pub fn add_recent(&mut self, path: String) {
        self.recent_projects.retain(|p| p != &path);
        self.recent_projects.insert(0, path.clone());
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }
        self.last_project = Some(path);
        self.save();
    }
}

pub struct EditorAppState {
    pub project_path: Option<String>,
    pub project_error: Option<String>,
    pub maps: Vec<(i32, String)>,
    pub map_tree: Vec<MapTreeItem>,
    pub selected_map: Option<usize>,
    pub view_mode: ViewMode,
    pub db_category: DbCategory,
    pub db_filter: String,

    // Database Tables
    pub actors: Vec<ActorInfo>,
    pub actors_dirty: bool,
    pub actors_save_message: Option<Result<String, String>>,

    pub classes: Vec<ClassInfo>,
    pub classes_dirty: bool,
    pub classes_save_message: Option<Result<String, String>>,

    pub items: Vec<ItemInfo>,
    pub items_dirty: bool,
    pub items_save_message: Option<Result<String, String>>,

    pub skills: Vec<SkillInfo>,
    pub skills_dirty: bool,
    pub skills_save_message: Option<Result<String, String>>,

    pub attributes: Vec<AttributeInfo>,
    pub attributes_dirty: bool,
    pub attributes_save_message: Option<Result<String, String>>,

    pub enemies: Vec<EnemyInfo>,
    pub enemies_dirty: bool,
    pub enemies_save_message: Option<Result<String, String>>,

    pub troops: Vec<TroopInfo>,
    pub troops_dirty: bool,
    pub troops_save_message: Option<Result<String, String>>,

    pub common_events: Vec<CommonEventInfo>,
    pub common_events_dirty: bool,
    pub common_events_save_message: Option<Result<String, String>>,

    pub switches: Vec<SwitchInfo>,
    pub switches_dirty: bool,
    pub switches_save_message: Option<Result<String, String>>,

    pub variables: Vec<VariableInfo>,
    pub variables_dirty: bool,
    pub variables_save_message: Option<Result<String, String>>,

    pub chipsets: Vec<ChipsetInfo>,
    pub chipsets_dirty: bool,
    pub chipsets_save_message: Option<Result<String, String>>,

    pub states: Vec<StateInfo>,
    pub states_dirty: bool,
    pub states_save_message: Option<Result<String, String>>,

    pub terrains: Vec<TerrainInfo>,
    pub terrains_dirty: bool,
    pub terrains_save_message: Option<Result<String, String>>,

    pub animations: Vec<AnimationInfo>,
    pub animations_dirty: bool,
    pub animations_save_message: Option<Result<String, String>>,

    pub system: Option<SystemInfo>,
    pub system_dirty: bool,
    pub system_save_message: Option<Result<String, String>>,

    pub terms: Option<TermsInfo>,
    pub terms_dirty: bool,
    pub terms_save_message: Option<Result<String, String>>,

    // Saves
    pub saves: Vec<SaveSlotView>,

    // App config
    pub config: AppPersistentData,
}

impl Default for EditorAppState {
    fn default() -> Self {
        let config = AppPersistentData::load();
        Self {
            project_path: None,
            project_error: None,
            maps: Vec::new(),
            map_tree: Vec::new(),
            selected_map: None,
            view_mode: ViewMode::Maps,
            db_category: DbCategory::Actors,
            db_filter: String::new(),
            actors: Vec::new(),
            actors_dirty: false,
            actors_save_message: None,
            classes: Vec::new(),
            classes_dirty: false,
            classes_save_message: None,
            items: Vec::new(),
            items_dirty: false,
            items_save_message: None,
            skills: Vec::new(),
            skills_dirty: false,
            skills_save_message: None,
            attributes: Vec::new(),
            attributes_dirty: false,
            attributes_save_message: None,
            enemies: Vec::new(),
            enemies_dirty: false,
            enemies_save_message: None,
            troops: Vec::new(),
            troops_dirty: false,
            troops_save_message: None,
            common_events: Vec::new(),
            common_events_dirty: false,
            common_events_save_message: None,
            switches: Vec::new(),
            switches_dirty: false,
            switches_save_message: None,
            variables: Vec::new(),
            variables_dirty: false,
            variables_save_message: None,
            chipsets: Vec::new(),
            chipsets_dirty: false,
            chipsets_save_message: None,
            states: Vec::new(),
            states_dirty: false,
            states_save_message: None,
            terrains: Vec::new(),
            terrains_dirty: false,
            terrains_save_message: None,
            animations: Vec::new(),
            animations_dirty: false,
            animations_save_message: None,
            system: None,
            system_dirty: false,
            system_save_message: None,
            terms: None,
            terms_dirty: false,
            terms_save_message: None,
            saves: Vec::new(),
            config,
        }
    }
}

impl EditorAppState {
    pub fn has_unsaved_changes(&self) -> bool {
        self.actors_dirty
            || self.classes_dirty
            || self.items_dirty
            || self.skills_dirty
            || self.attributes_dirty
            || self.enemies_dirty
            || self.troops_dirty
            || self.common_events_dirty
            || self.switches_dirty
            || self.variables_dirty
            || self.chipsets_dirty
            || self.states_dirty
            || self.terrains_dirty
            || self.animations_dirty
            || self.system_dirty
            || self.terms_dirty
            || self.saves.iter().any(|s| s.dirty)
    }

    /// Entry count and dirty flag for whichever category is currently
    /// selected - used by the status bar's Database-mode summary.
    pub fn current_db_category_status(&self) -> (usize, bool) {
        match self.db_category {
            DbCategory::Actors => (self.actors.len(), self.actors_dirty),
            DbCategory::Classes => (self.classes.len(), self.classes_dirty),
            DbCategory::Items => (self.items.len(), self.items_dirty),
            DbCategory::Skills => (self.skills.len(), self.skills_dirty),
            DbCategory::Attributes => (self.attributes.len(), self.attributes_dirty),
            DbCategory::Enemies => (self.enemies.len(), self.enemies_dirty),
            DbCategory::Troops => (self.troops.len(), self.troops_dirty),
            DbCategory::CommonEvents => (self.common_events.len(), self.common_events_dirty),
            DbCategory::Switches => (self.switches.len(), self.switches_dirty),
            DbCategory::Variables => (self.variables.len(), self.variables_dirty),
            DbCategory::Chipsets => (self.chipsets.len(), self.chipsets_dirty),
            DbCategory::States => (self.states.len(), self.states_dirty),
            DbCategory::Terrains => (self.terrains.len(), self.terrains_dirty),
            DbCategory::Animations => (self.animations.len(), self.animations_dirty),
            DbCategory::System => (1, self.system_dirty),
            DbCategory::Terms => (1, self.terms_dirty),
        }
    }

    pub fn load_project_from(&mut self, path_str: String) {
        self.project_path = Some(path_str.clone());
        self.selected_map = None;
        self.config.add_recent(path_str.clone());

        let info = lcf_bridge::load_project(&path_str);
        if info.valid {
            self.project_error = None;
            self.maps = info.maps.into_iter().map(|m| (m.id, m.name)).collect();
            self.map_tree = lcf_bridge::get_map_tree(&path_str);
            self.actors = lcf_bridge::get_actors(&path_str);
            self.actors_dirty = false;
            self.actors_save_message = None;
            self.classes = lcf_bridge::get_classes(&path_str);
            self.classes_dirty = false;
            self.classes_save_message = None;
            self.items = lcf_bridge::get_items(&path_str);
            self.items_dirty = false;
            self.items_save_message = None;
            self.skills = lcf_bridge::get_skills(&path_str);
            self.skills_dirty = false;
            self.skills_save_message = None;
            self.attributes = lcf_bridge::get_attributes(&path_str);
            self.attributes_dirty = false;
            self.attributes_save_message = None;
            self.enemies = lcf_bridge::get_enemies(&path_str);
            self.enemies_dirty = false;
            self.enemies_save_message = None;
            self.troops = lcf_bridge::get_troops(&path_str);
            self.troops_dirty = false;
            self.troops_save_message = None;
            self.common_events = lcf_bridge::get_common_events(&path_str);
            self.common_events_dirty = false;
            self.common_events_save_message = None;
            self.switches = lcf_bridge::get_switches(&path_str);
            self.switches_dirty = false;
            self.switches_save_message = None;
            self.variables = lcf_bridge::get_variables(&path_str);
            self.variables_dirty = false;
            self.variables_save_message = None;
            self.chipsets = lcf_bridge::get_chipsets(&path_str);
            self.chipsets_dirty = false;
            self.chipsets_save_message = None;
            self.states = lcf_bridge::get_states(&path_str);
            self.states_dirty = false;
            self.states_save_message = None;
            self.terrains = lcf_bridge::get_terrains(&path_str);
            self.terrains_dirty = false;
            self.terrains_save_message = None;
            self.animations = lcf_bridge::get_animations(&path_str);
            self.animations_dirty = false;
            self.animations_save_message = None;
            self.system = lcf_bridge::get_system(&path_str);
            self.terms = lcf_bridge::get_terms(&path_str);
            self.terms_dirty = false;
            self.terms_save_message = None;
            self.saves = lcf_bridge::list_saves(&path_str)
                .into_iter()
                .map(|info| SaveSlotView { info, dirty: false, save_message: None })
                .collect();
        } else {
            self.project_error = Some("Not a valid RPG Maker project (RPG_RT.lmt not found or unreadable).".to_string());
            self.maps.clear();
            self.map_tree.clear();
            self.actors.clear();
            self.classes.clear();
            self.items.clear();
            self.skills.clear();
            self.attributes.clear();
            self.enemies.clear();
            self.troops.clear();
            self.common_events.clear();
            self.switches.clear();
            self.variables.clear();
            self.system = None;
            self.terms = None;
            self.saves.clear();
        }
    }

    pub fn save_current_db_category(&mut self) {
        let Some(path) = self.project_path.clone() else { return };
        let category = self.db_category;
        self.save_category(&path, category);
    }

    /// Saves every dirty database category plus every dirty save slot,
    /// regardless of which one is currently selected in the UI - used by
    /// the close-confirmation dialog's "Save All & Close" option. Returns
    /// one error string per category/slot that failed to save; an empty
    /// Vec means everything saved cleanly.
    pub fn save_all_dirty(&mut self) -> Vec<String> {
        let Some(path) = self.project_path.clone() else { return Vec::new() };
        let mut errors = Vec::new();

        macro_rules! save_if_dirty {
            ($category:expr, $dirty_field:ident, $label:literal) => {
                if self.$dirty_field {
                    self.save_category(&path, $category);
                    if self.$dirty_field {
                        errors.push(format!("{} failed to save.", $label));
                    }
                }
            };
        }

        save_if_dirty!(DbCategory::Actors, actors_dirty, "Actors");
        save_if_dirty!(DbCategory::Classes, classes_dirty, "Classes");
        save_if_dirty!(DbCategory::Items, items_dirty, "Items");
        save_if_dirty!(DbCategory::Skills, skills_dirty, "Skills");
        save_if_dirty!(DbCategory::Attributes, attributes_dirty, "Attributes");
        save_if_dirty!(DbCategory::Enemies, enemies_dirty, "Enemies");
        save_if_dirty!(DbCategory::Troops, troops_dirty, "Troops");
        save_if_dirty!(DbCategory::CommonEvents, common_events_dirty, "Common Events");
        save_if_dirty!(DbCategory::Switches, switches_dirty, "Switches");
        save_if_dirty!(DbCategory::Variables, variables_dirty, "Variables");
        save_if_dirty!(DbCategory::Chipsets, chipsets_dirty, "Chipsets");
        save_if_dirty!(DbCategory::States, states_dirty, "States");
        save_if_dirty!(DbCategory::Terrains, terrains_dirty, "Terrains");
        save_if_dirty!(DbCategory::Animations, animations_dirty, "Animations");
        save_if_dirty!(DbCategory::System, system_dirty, "System");
        save_if_dirty!(DbCategory::Terms, terms_dirty, "Terms");

        for slot in &mut self.saves {
            if slot.dirty {
                match lcf_bridge::save_save_slot(&path, &slot.info.file_name, &slot.info) {
                    Ok(()) => {
                        slot.save_message = Some(Ok("Saved successfully.".to_string()));
                        slot.dirty = false;
                    }
                    Err(e) => {
                        errors.push(format!("{}: {e}", slot.info.file_name));
                        slot.save_message = Some(Err(e));
                    }
                }
            }
        }

        errors
    }

    fn save_category(&mut self, path: &str, category: DbCategory) {
        match category {
            DbCategory::Actors => {
                match lcf_bridge::save_actors(path, &self.actors) {
                    Ok(()) => {
                        self.actors_save_message = Some(Ok("Actors saved successfully.".to_string()));
                        self.actors_dirty = false;
                    }
                    Err(e) => self.actors_save_message = Some(Err(e)),
                }
            }
            DbCategory::Classes => {
                match lcf_bridge::save_classes(path, &self.classes) {
                    Ok(()) => {
                        self.classes_save_message = Some(Ok("Classes saved successfully.".to_string()));
                        self.classes_dirty = false;
                    }
                    Err(e) => self.classes_save_message = Some(Err(e)),
                }
            }
            DbCategory::Items => {
                match lcf_bridge::save_items(path, &self.items) {
                    Ok(()) => {
                        self.items_save_message = Some(Ok("Items saved successfully.".to_string()));
                        self.items_dirty = false;
                    }
                    Err(e) => self.items_save_message = Some(Err(e)),
                }
            }
            DbCategory::Skills => {
                match lcf_bridge::save_skills(path, &self.skills) {
                    Ok(()) => {
                        self.skills_save_message = Some(Ok("Skills saved successfully.".to_string()));
                        self.skills_dirty = false;
                    }
                    Err(e) => self.skills_save_message = Some(Err(e)),
                }
            }
            DbCategory::Attributes => {
                match lcf_bridge::save_attributes(path, &self.attributes) {
                    Ok(()) => {
                        self.attributes_save_message = Some(Ok("Attributes saved successfully.".to_string()));
                        self.attributes_dirty = false;
                    }
                    Err(e) => self.attributes_save_message = Some(Err(e)),
                }
            }
            DbCategory::Enemies => {
                match lcf_bridge::save_enemies(path, &self.enemies) {
                    Ok(()) => {
                        self.enemies_save_message = Some(Ok("Enemies saved successfully.".to_string()));
                        self.enemies_dirty = false;
                    }
                    Err(e) => self.enemies_save_message = Some(Err(e)),
                }
            }
            DbCategory::Troops => {
                match lcf_bridge::save_troops(path, &self.troops) {
                    Ok(()) => {
                        self.troops_save_message = Some(Ok("Troops saved successfully.".to_string()));
                        self.troops_dirty = false;
                    }
                    Err(e) => self.troops_save_message = Some(Err(e)),
                }
            }
            DbCategory::CommonEvents => {
                match lcf_bridge::save_common_events(path, &self.common_events) {
                    Ok(()) => {
                        self.common_events_save_message = Some(Ok("Common Events saved successfully.".to_string()));
                        self.common_events_dirty = false;
                    }
                    Err(e) => self.common_events_save_message = Some(Err(e)),
                }
            }
            DbCategory::Switches => {
                match lcf_bridge::save_switches(path, &self.switches) {
                    Ok(()) => {
                        self.switches_save_message = Some(Ok("Switches saved successfully.".to_string()));
                        self.switches_dirty = false;
                    }
                    Err(e) => self.switches_save_message = Some(Err(e)),
                }
            }
            DbCategory::Variables => {
                match lcf_bridge::save_variables(path, &self.variables) {
                    Ok(()) => {
                        self.variables_save_message = Some(Ok("Variables saved successfully.".to_string()));
                        self.variables_dirty = false;
                    }
                    Err(e) => self.variables_save_message = Some(Err(e)),
                }
            }
            DbCategory::Chipsets => {
                match lcf_bridge::save_chipsets(path, &self.chipsets) {
                    Ok(()) => {
                        self.chipsets_save_message = Some(Ok("Chipsets saved successfully.".to_string()));
                        self.chipsets_dirty = false;
                    }
                    Err(e) => self.chipsets_save_message = Some(Err(e)),
                }
            }
            DbCategory::States => {
                match lcf_bridge::save_states(path, &self.states) {
                    Ok(()) => {
                        self.states_save_message = Some(Ok("States saved successfully.".to_string()));
                        self.states_dirty = false;
                    }
                    Err(e) => self.states_save_message = Some(Err(e)),
                }
            }
            DbCategory::Terrains => {
                match lcf_bridge::save_terrains(path, &self.terrains) {
                    Ok(()) => {
                        self.terrains_save_message = Some(Ok("Terrains saved successfully.".to_string()));
                        self.terrains_dirty = false;
                    }
                    Err(e) => self.terrains_save_message = Some(Err(e)),
                }
            }
            DbCategory::Animations => {
                match lcf_bridge::save_animations(path, &self.animations) {
                    Ok(()) => {
                        self.animations_save_message = Some(Ok("Animations saved successfully.".to_string()));
                        self.animations_dirty = false;
                    }
                    Err(e) => self.animations_save_message = Some(Err(e)),
                }
            }
            DbCategory::Terms => {
                if let Some(terms) = &self.terms {
                    match lcf_bridge::save_terms(path, terms) {
                        Ok(()) => {
                            self.terms_save_message = Some(Ok("Terms saved successfully.".to_string()));
                            self.terms_dirty = false;
                        }
                        Err(e) => self.terms_save_message = Some(Err(e)),
                    }
                }
            }
            DbCategory::System => {
                if let Some(system) = &self.system {
                    match lcf_bridge::save_system(path, system) {
                        Ok(()) => {
                            self.system_save_message = Some(Ok("System settings saved successfully.".to_string()));
                            self.system_dirty = false;
                        }
                        Err(e) => self.system_save_message = Some(Err(e)),
                    }
                }
            }
        }
    }

    pub fn discard_current_db_category(&mut self) {
        let path = match self.project_path.as_deref() {
            Some(p) => p,
            None => return,
        };

        match self.db_category {
            DbCategory::Actors => {
                self.actors = lcf_bridge::get_actors(path);
                self.actors_dirty = false;
                self.actors_save_message = None;
            }
            DbCategory::Classes => {
                self.classes = lcf_bridge::get_classes(path);
                self.classes_dirty = false;
                self.classes_save_message = None;
            }
            DbCategory::Items => {
                self.items = lcf_bridge::get_items(path);
                self.items_dirty = false;
                self.items_save_message = None;
            }
            DbCategory::Skills => {
                self.skills = lcf_bridge::get_skills(path);
                self.skills_dirty = false;
                self.skills_save_message = None;
            }
            DbCategory::Attributes => {
                self.attributes = lcf_bridge::get_attributes(path);
                self.attributes_dirty = false;
                self.attributes_save_message = None;
            }
            DbCategory::Enemies => {
                self.enemies = lcf_bridge::get_enemies(path);
                self.enemies_dirty = false;
                self.enemies_save_message = None;
            }
            DbCategory::Troops => {
                self.troops = lcf_bridge::get_troops(path);
                self.troops_dirty = false;
                self.troops_save_message = None;
            }
            DbCategory::CommonEvents => {
                self.common_events = lcf_bridge::get_common_events(path);
                self.common_events_dirty = false;
                self.common_events_save_message = None;
            }
            DbCategory::Switches => {
                self.switches = lcf_bridge::get_switches(path);
                self.switches_dirty = false;
                self.switches_save_message = None;
            }
            DbCategory::Variables => {
                self.variables = lcf_bridge::get_variables(path);
                self.variables_dirty = false;
                self.variables_save_message = None;
            }
            DbCategory::Chipsets => {
                self.chipsets = lcf_bridge::get_chipsets(path);
                self.chipsets_dirty = false;
                self.chipsets_save_message = None;
            }
            DbCategory::States => {
                self.states = lcf_bridge::get_states(path);
                self.states_dirty = false;
                self.states_save_message = None;
            }
            DbCategory::Terrains => {
                self.terrains = lcf_bridge::get_terrains(path);
                self.terrains_dirty = false;
                self.terrains_save_message = None;
            }
            DbCategory::Animations => {
                self.animations = lcf_bridge::get_animations(path);
                self.animations_dirty = false;
                self.animations_save_message = None;
            }
            DbCategory::Terms => {
                self.terms = lcf_bridge::get_terms(path);
                self.terms_dirty = false;
                self.terms_save_message = None;
            }
            DbCategory::System => {
                self.system = lcf_bridge::get_system(path);
                self.system_dirty = false;
                self.system_save_message = None;
            }
        }
    }
}
