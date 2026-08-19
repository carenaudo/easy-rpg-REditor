use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct IniReader {
    values: HashMap<String, String>,
}

impl IniReader {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::load_from_reader(BufReader::new(file))
    }

    pub fn load_from_reader<R: Read>(reader: R) -> io::Result<Self> {
        let mut ini = Self::new();
        let buf = BufReader::new(reader);
        let mut current_section = String::new();

        for line in buf.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_lowercase();
            } else if let Some((key, val)) = trimmed.split_once('=') {
                let k = format!("{}={}", current_section, key.trim().to_lowercase());
                let v = val.trim().to_string();
                ini.values.insert(k, v);
            }
        }

        Ok(ini)
    }

    pub fn get(&self, section: &str, name: &str) -> Option<&str> {
        let key = format!("{}={}", section.trim().to_lowercase(), name.trim().to_lowercase());
        self.values.get(&key).map(|s| s.as_str())
    }

    pub fn get_string(&self, section: &str, name: &str, default: &str) -> String {
        self.get(section, name).unwrap_or(default).to_string()
    }

    pub fn get_int(&self, section: &str, name: &str, default: i64) -> i64 {
        self.get(section, name)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, section: &str, name: &str, default: bool) -> bool {
        self.get(section, name)
            .map(|v| match v.to_ascii_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => true,
                "false" | "no" | "off" | "0" => false,
                _ => default,
            })
            .unwrap_or(default)
    }
}
