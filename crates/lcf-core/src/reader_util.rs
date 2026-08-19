use std::time::{SystemTime, UNIX_EPOCH};
use crate::generated::ldb_gen::Database;
use crate::ini::IniReader;
use crate::types::EngineVersion;

pub struct ReaderUtil;

impl ReaderUtil {
    /// Converts a Windows/DOS codepage number to an encoding label.
    pub fn codepage_to_encoding(codepage: i32) -> &'static str {
        match codepage {
            932 => "shift_jis",
            949 => "euc-kr",
            936 => "gbk",
            950 => "big5",
            1250 => "windows-1250",
            1251 => "windows-1251",
            1252 => "windows-1252",
            1253 => "windows-1253",
            1254 => "windows-1254",
            1255 => "windows-1255",
            1256 => "windows-1256",
            1257 => "windows-1257",
            1258 => "windows-1258",
            65001 => "utf-8",
            _ => "windows-1252",
        }
    }

    /// Converts an encoding label to a Windows/DOS codepage number.
    pub fn encoding_to_codepage(encoding: &str) -> i32 {
        let enc = encoding.to_ascii_lowercase();
        if enc.contains("shift") || enc.contains("932") || enc.contains("sjis") {
            932
        } else if enc.contains("949") || enc.contains("korean") || enc.contains("euc-kr") {
            949
        } else if enc.contains("936") || enc.contains("gbk") || enc.contains("gb2312") || enc.contains("gb18030") {
            936
        } else if enc.contains("950") || enc.contains("big5") {
            950
        } else if enc.contains("1250") {
            1250
        } else if enc.contains("1251") {
            1251
        } else if enc.contains("1253") {
            1253
        } else if enc.contains("1254") {
            1254
        } else if enc.contains("1255") {
            1255
        } else if enc.contains("1256") {
            1256
        } else if enc.contains("1257") {
            1257
        } else if enc.contains("1258") {
            1258
        } else if enc.contains("utf-8") || enc.contains("utf8") {
            65001
        } else {
            1252
        }
    }

    /// Determines the engine version (2000 vs 2003) from database properties.
    pub fn get_engine_version(db: &Database) -> EngineVersion {
        if db.system.ldb_id == 2003
            || db.version >= 2
            || !db.classes.is_empty()
            || !db.battleranimations.is_empty()
            || db.actors.iter().any(|a| a.class_id > 0 || !a.battle_commands.is_empty())
        {
            EngineVersion::Engine2003
        } else {
            EngineVersion::Engine2000
        }
    }



    /// Reads encoding declaration from an EasyRPG project configuration (RPG_RT.ini).
    pub fn get_encoding_from_ini(ini: &IniReader) -> Option<String> {
        let enc = ini.get_string("EasyRPG", "Encoding", "");
        if !enc.is_empty() {
            if let Ok(cp) = enc.parse::<i32>() {
                return Some(Self::codepage_to_encoding(cp).to_string());
            }
            return Some(enc);
        }
        None
    }

    /// Auto-detects text encoding by analyzing database strings (Terms, System, Actors, Skills, Items).
    pub fn detect_encoding(db: &Database) -> String {
        let mut sjis_score = 0;
        let mut euc_score = 0;
        let mut gbk_score = 0;

        let check_dbstring = |s: &crate::types::DBString, sjis: &mut i32, euc: &mut i32, gbk: &mut i32| {
            if s.as_str().chars().any(|c| (c as u32) >= 0x3000 && (c as u32) <= 0x9FFF) {
                *sjis += 5;
            }
            let bytes = if let Some(raw) = s.raw_bytes() {
                raw
            } else {
                s.as_str().as_bytes()
            };


            for window in bytes.windows(2) {
                let b1 = window[0];
                let b2 = window[1];
                // Shift-JIS byte range
                if ((0x81..=0x9F).contains(&b1) || (0xE0..=0xEF).contains(&b1))
                    && ((0x40..=0x7E).contains(&b2) || (0x80..=0xFC).contains(&b2))
                {
                    *sjis += 1;
                }
                // EUC-KR byte range
                if (0xA1..=0xFE).contains(&b1) && (0xA1..=0xFE).contains(&b2) {
                    *euc += 1;
                }
                // GBK byte range
                if (0x81..=0xFE).contains(&b1) && (0x40..=0xFE).contains(&b2) && b2 != 0x7F {
                    *gbk += 1;
                }
            }
        };

        // Sample Terms
        let terms = &db.terms;
        for s in &[
            &terms.menu_save, &terms.menu_quit, &terms.new_game, &terms.load_game,
            &terms.exit_game, &terms.status, &terms.row, &terms.order,
            &terms.health_points, &terms.spirit_points, &terms.attack, &terms.defense,
            &terms.spirit, &terms.agility, &terms.weapon, &terms.shield,
            &terms.armor, &terms.helmet, &terms.accessory,
            &terms.command_attack, &terms.command_defend, &terms.command_item, &terms.command_skill,
            &terms.save_game_message, &terms.load_game_message, &terms.exit_game_message,
            &terms.file, &terms.yes, &terms.no,
        ] {

            check_dbstring(s, &mut sjis_score, &mut euc_score, &mut gbk_score);
        }

        // Sample Actors
        for a in &db.actors {
            check_dbstring(&a.name, &mut sjis_score, &mut euc_score, &mut gbk_score);
            check_dbstring(&a.title, &mut sjis_score, &mut euc_score, &mut gbk_score);
        }

        // Sample Skills
        for sk in &db.skills {
            check_dbstring(&sk.name, &mut sjis_score, &mut euc_score, &mut gbk_score);
        }

        // Sample Items
        for item in &db.items {
            check_dbstring(&item.name, &mut sjis_score, &mut euc_score, &mut gbk_score);
        }

        if sjis_score > 0 && sjis_score >= euc_score && sjis_score >= gbk_score {
            "shift_jis".to_string()
        } else if euc_score > 5 && euc_score > sjis_score {
            "euc-kr".to_string()
        } else if gbk_score > 5 && gbk_score > sjis_score {
            "gbk".to_string()
        } else {
            "windows-1252".to_string()
        }
    }

    /// Converts a Unix timestamp to Delphi/OLE TDateTime format.
    /// (25569.0 is the number of days between 1899-12-30 and 1970-01-01).
    pub fn to_t_date_time(unix_time: i64) -> f64 {
        (unix_time as f64) / 86400.0 + 25569.0
    }

    /// Converts a Delphi/OLE TDateTime float to Unix timestamp seconds.
    pub fn to_unix_timestamp(t_date_time: f64) -> i64 {
        ((t_date_time - 25569.0) * 86400.0 + 0.5) as i64
    }

    /// Generates a current timestamp in Delphi TDateTime format.
    pub fn generate_timestamp() -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self::to_t_date_time(now)
    }
}
