use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StructRow {
    #[serde(rename = "Type")]
    file_type: String,
    #[serde(rename = "Structure")]
    structure: String,
    #[serde(rename = "Base", default)]
    base: String,
    #[serde(rename = "Index available?", default)]
    index_available: String,
}

#[derive(Debug, Deserialize)]
struct FieldRow {
    #[serde(rename = "Structure")]
    structure: String,
    #[serde(rename = "Field")]
    field: String,
    #[serde(rename = "Size Field?", default)]
    size_field: String,
    #[serde(rename = "Type")]
    field_type: String,
    #[serde(rename = "Index", default)]
    index: String,
    #[serde(rename = "Default Value", default)]
    default_value: String,
    #[serde(rename = "PersistIfDefault", default)]
    persist_if_default: String,
    #[serde(rename = "Is2k3", default)]
    is_2k3: String,
    #[serde(rename = "Comment", default)]
    _comment: String,
}

#[derive(Debug, Deserialize)]
struct EnumRow {
    #[serde(rename = "Structure")]
    structure: String,
    #[serde(rename = "Entry")]
    entry: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "Index")]
    index: String,
}

#[derive(Debug, Clone)]
struct FieldDef {
    field: String,
    rust_name: String,
    rust_type: String,
    chunk_id: Option<u32>,
    default_val: String,
    persist_if_default: bool,
    is_2k3: bool,
    is_size_field: bool,
    is_flags: bool,
}


#[derive(Debug, Clone)]
struct StructDef {
    file_type: String,
    name: String,
    base: Option<String>,
    has_id: bool,
    fields: Vec<FieldDef>,
}

fn sanitize_field_name(name: &str) -> String {
    let s = name.to_string();
    match s.as_str() {
        "type" => "r#type".to_string(),
        "match" => "r#match".to_string(),
        "loop" => "r#loop".to_string(),
        "box" => "r#box".to_string(),
        "move" => "r#move".to_string(),
        "use" => "r#use".to_string(),
        "ref" => "r#ref".to_string(),
        "fn" => "r#fn".to_string(),
        _ => s,
    }
}

fn parse_hex_or_dec(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

fn map_type_to_rust(csv_type: &str) -> String {
    let mut t = csv_type.trim();
    if t.contains(':') && !t.starts_with("Array<") && !t.starts_with("Vector<") {
        t = t.split(':').next().unwrap().trim();
    }
    if t == "Int8" {
        "i8".to_string()
    } else if t == "Int32" || t == "DatabaseVersion" || t == "EmptyBlock" {
        "i32".to_string()
    } else if t == "Int16" {
        "i16".to_string()
    } else if t == "UInt8" {
        "u8".to_string()
    } else if t == "UInt32" {
        "u32".to_string()
    } else if t == "Double" {
        "f64".to_string()
    } else if t == "Boolean" {
        "bool".to_string()
    } else if t == "String" || t == "DBString" {
        "DBString".to_string()
    } else if t.starts_with("Ref<") || t.starts_with("Enum<") || t.ends_with("_Flags") {
        "i32".to_string()
    } else if t.starts_with("Array<") && t.ends_with('>') {
        let mut inner = &t[6..t.len() - 1];
        if inner.contains(':') {
            inner = inner.split(':').next().unwrap().trim();
        }
        format!("Vec<{}>", inner)
    } else if t.starts_with("Vector<") && t.ends_with('>') {
        let mut inner = &t[7..t.len() - 1];
        if inner.contains(':') {
            inner = inner.split(':').next().unwrap().trim();
        }
        let rust_inner = map_type_to_rust(inner);
        format!("Vec<{}>", rust_inner)
    } else if t == "DBBitArray" || t == "BitArray" {
        "DBBitArray".to_string()
    } else {
        t.to_string()
    }
}

fn generate_default_expr(f: &FieldDef) -> String {
    let t = f.rust_type.as_str();
    let val = f.default_val.as_str();

    if t == "bool" {
        if val.eq_ignore_ascii_case("true") {
            "true".to_string()
        } else {
            "false".to_string()
        }
    } else if t == "i32" || t == "i16" || t == "u8" || t == "u32" || t == "i8" {
        if val.contains('|') {
            let parts: Vec<&str> = val.split('|').collect();
            let v2k = parts[0].trim().parse::<i32>().unwrap_or(0);
            let v2k3 = parts[1].trim().parse::<i32>().unwrap_or(0);
            format!("if _is_2k3 {{ {} }} else {{ {} }}", v2k3, v2k)
        } else if !val.is_empty() && val.parse::<f64>().is_ok() {
            let n = val.parse::<i32>().unwrap_or(0);
            format!("{}", n)
        } else {
            "0".to_string()
        }
    } else if t == "f64" {
        if !val.is_empty() && val.parse::<f64>().is_ok() {
            format!("{:.1}", val.parse::<f64>().unwrap())
        } else {
            "0.0".to_string()
        }
    } else if t == "DBString" {
        if val == "\"\"\"(OFF)\"\"\"" || val == "(OFF)" {
            "DBString::new(\"(OFF)\")".to_string()
        } else if !val.is_empty() && val != "\"\"" {
            let clean = val.trim_matches('"');
            format!("DBString::new(\"{}\")", clean)
        } else {
            "DBString::default()".to_string()
        }
    } else if t.starts_with("Vec<") {
        if val.contains("[1]*162") {
            "vec![1; 162]".to_string()
        } else if val.contains("[15]*162") {
            "vec![15; 162]".to_string()
        } else if val.contains("[31]+[15]*143") {
            "{\n            let mut v = vec![15; 144];\n            v[0] = 31;\n            v\n        }".to_string()
        } else {
            "Vec::new()".to_string()
        }
    } else {
        format!("{}::default()", t)
    }
}

/// Types hand-written in `types.rs` (liblcf calls these `RawStruct`s) whose
/// `read_xml_fields`/`write_xml` take no `id`/`is_2k3` - they have neither
/// an id attribute nor engine-conditional defaults. Every other struct
/// referenced as a field type is codegen-emitted and takes
/// `(reader, id, is_2k3)` / uses `default_for_engine(is_2k3)`.
fn is_special_raw_name(name: &str) -> bool {
    matches!(
        name,
        "Parameters" | "Equipment" | "Rect" | "Music" | "Sound" | "MoveCommand" | "EventCommand" | "TreeMap"
    )
}

fn generate_struct_code(s: &StructDef, all_structs: &HashMap<String, StructDef>) -> String {
    let is_special_raw = is_special_raw_name(s.name.as_str());

    if is_special_raw {
        return String::new();
    }

    let mut code = String::new();

    // Struct definition
    code.push_str(&format!("#[derive(Clone, PartialEq, Debug)]\npub struct {} {{\n", s.name));
    if s.has_id {
        code.push_str("    pub id: i32,\n");
    }
    for f in &s.fields {
        if f.is_size_field {
            continue;
        }
        code.push_str(&format!("    pub {}: {},\n", f.rust_name, f.rust_type));
    }
    code.push_str("}\n\n");

    // Default impl
    code.push_str(&format!("impl Default for {} {{\n    fn default() -> Self {{\n        Self::default_for_engine(false)\n    }}\n}}\n\n", s.name));

    // default_for_engine impl
    code.push_str(&format!("impl {} {{\n    pub fn default_for_engine(_is_2k3: bool) -> Self {{\n        Self {{\n", s.name));
    if s.has_id {
        code.push_str("            id: 0,\n");
    }
    for f in &s.fields {
        if f.is_size_field {
            continue;
        }
        let def_expr = generate_default_expr(f);
        code.push_str(&format!("            {}: {},\n", f.rust_name, def_expr));
    }
    code.push_str("        }\n    }\n\n");

    // read_lcf
    code.push_str("    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, crate::error::LcfError> {\n");
    code.push_str("        let mut obj = Self::default();\n");
    code.push_str("        while let Some(chunk) = reader.read_chunk_header()? {\n");
    code.push_str("            let start_off = reader.tell()?;\n");
    code.push_str("            match chunk.id {\n");

    for f in &s.fields {
        if f.is_size_field {
            if let Some(cid) = f.chunk_id {
                code.push_str(&format!("                0x{:02X} => {{ let _ = reader.read_int()?; }}\n", cid));
            }
            continue;
        }

        if let Some(cid) = f.chunk_id {
            let t = f.rust_type.as_str();
            code.push_str(&format!("                0x{:02X} => {{\n", cid));
            if f.is_flags {
                code.push_str(&format!("                    obj.{} = reader.read_flags(chunk.length)?;\n", f.rust_name));
            } else if t == "i32" || t == "i8" {
                code.push_str(&format!("                    obj.{} = reader.read_int()? as {};\n", f.rust_name, t));
            } else if t == "i16" {

                code.push_str(&format!("                    obj.{} = reader.read_i16()?;\n", f.rust_name));
            } else if t == "u8" {
                code.push_str(&format!("                    obj.{} = reader.read_u8()?;\n", f.rust_name));
            } else if t == "u32" {
                code.push_str(&format!("                    obj.{} = reader.read_u32_le()?;\n", f.rust_name));
            } else if t == "f64" {
                code.push_str(&format!("                    obj.{} = reader.read_f64()?;\n", f.rust_name));
            } else if t == "bool" {
                code.push_str(&format!("                    obj.{} = reader.read_bool()?;\n", f.rust_name));
            } else if t == "DBString" {
                code.push_str(&format!("                    obj.{} = reader.read_string(chunk.length)?;\n", f.rust_name));
            } else if t == "Rect" {
                code.push_str(&format!("                    obj.{} = reader.read_rect()?;\n", f.rust_name));
            } else if t == "Parameters" {
                code.push_str(&format!("                    obj.{} = crate::types::Parameters::read_lcf(reader, chunk.length)?;\n", f.rust_name));
            } else if t == "Equipment" {
                code.push_str(&format!("                    obj.{} = crate::types::Equipment::read_lcf(reader, chunk.length)?;\n", f.rust_name));
            } else if t == "MoveRoute" {
                code.push_str(&format!("                    obj.{} = MoveRoute::read_lcf(reader)?;\n", f.rust_name));
            } else if t == "EventPageCondition" {
                code.push_str(&format!("                    obj.{} = EventPageCondition::read_lcf(reader)?;\n", f.rust_name));
            } else if t == "Vec<u8>" {
                code.push_str(&format!("                    obj.{} = reader.read_vector_u8(chunk.length)?;\n", f.rust_name));
            } else if t == "Vec<i16>" {
                code.push_str(&format!("                    obj.{} = reader.read_vector_i16(chunk.length)?;\n", f.rust_name));
            } else if t == "Vec<i32>" {
                code.push_str(&format!("                    obj.{} = reader.read_vector_i32(chunk.length)?;\n", f.rust_name));
            } else if t == "Vec<u32>" {
                code.push_str(&format!("                    let items = chunk.length / 4;\n                    obj.{}.clear();\n                    for _ in 0..items {{ obj.{}.push(reader.read_u32_le()?); }}\n", f.rust_name, f.rust_name));
            } else if t == "Vec<bool>" {
                code.push_str(&format!("                    obj.{} = reader.read_vector_bool(chunk.length)?;\n", f.rust_name));
            } else if t == "DBBitArray" {
                code.push_str(&format!("                    obj.{} = reader.read_bit_array(chunk.length)?;\n", f.rust_name));
            } else if t.starts_with("Vec<") {
                let inner = &t[4..t.len() - 1];
                if inner == "EventCommand" {
                    code.push_str(&format!("                    let end_pos = start_off + chunk.length as u64;\n                    obj.{}.clear();\n                    loop {{\n                        if reader.tell()? >= end_pos {{ break; }}\n                        let cmd = crate::types::EventCommand::read_lcf(reader)?;\n                        if cmd.code == 0 {{\n                            for _ in 0..3 {{ if reader.tell()? < end_pos {{ let _ = reader.read_u8()?; }} }}\n                            break;\n                        }}\n                        obj.{}.push(cmd);\n                    }}\n", f.rust_name, f.rust_name));
                } else if inner == "MoveCommand" {
                    code.push_str(&format!("                    let end_pos = start_off + chunk.length as u64;\n                    obj.{}.clear();\n                    while reader.tell()? < end_pos {{\n                        obj.{}.push(crate::types::MoveCommand::read_lcf(reader)?);\n                    }}\n", f.rust_name, f.rust_name));
                } else if inner == "DBString" {
                    code.push_str(&format!("                    let count = reader.read_int()?;\n                    obj.{}.clear();\n                    for _ in 0..count {{\n                        let len = reader.read_int()? as usize;\n                        obj.{}.push(reader.read_string(len)?);\n                    }}\n", f.rust_name, f.rust_name));
                } else if inner == "i32" || inner == "u8" || inner == "i16" || inner == "u32" || inner == "bool" {
                    code.push_str(&format!("                    obj.{} = reader.read_vector_{}(chunk.length)?;\n", f.rust_name, inner));
                } else {
                    let inner_has_id = all_structs.get(inner).map_or(false, |s_def| s_def.has_id);
                    if inner_has_id {
                        code.push_str(&format!("                    let count = reader.read_int()?;\n                    obj.{}.clear();\n                    for _ in 0..count {{\n                        let id = reader.read_int()?;\n                        let mut item = {}::read_lcf(reader)?;\n                        item.id = id;\n                        obj.{}.push(item);\n                    }}\n", f.rust_name, inner, f.rust_name));
                    } else {
                        code.push_str(&format!("                    let count = reader.read_int()?;\n                    obj.{}.clear();\n                    for _ in 0..count {{\n                        let item = {}::read_lcf(reader)?;\n                        obj.{}.push(item);\n                    }}\n", f.rust_name, inner, f.rust_name));
                    }
                }
            } else if all_structs.contains_key(t) {
                code.push_str(&format!("                    obj.{} = {}::read_lcf(reader)?;\n", f.rust_name, t));
            } else {
                code.push_str("                    reader.skip(chunk.length)?;\n");
            }
            code.push_str("                }\n");
        }
    }

    code.push_str("                _ => reader.skip(chunk.length)?,\n");
    code.push_str("            }\n");
    code.push_str("            let read_bytes = (reader.tell()? - start_off) as usize;\n");
    code.push_str("            if read_bytes < chunk.length {\n");
    code.push_str("                reader.skip(chunk.length - read_bytes)?;\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("        Ok(obj)\n");
    code.push_str("    }\n\n");

    // write_lcf
    code.push_str("    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), crate::error::LcfError> {\n");
    code.push_str("        let _def = Self::default_for_engine(writer.is_2k3());\n");

    for f in &s.fields {
        if f.is_size_field {
            continue;
        }

        if let Some(cid) = f.chunk_id {
            let t = f.rust_type.as_str();

            let emit_field = |statement: String, cond: Option<String>| -> String {
                let mut out = String::new();
                if f.is_2k3 {
                    if let Some(c) = cond {
                        out.push_str(&format!("        if writer.is_2k3() && ({}) {{\n            {}\n        }}\n", c, statement));
                    } else {
                        out.push_str(&format!("        if writer.is_2k3() {{\n            {}\n        }}\n", statement));
                    }
                } else if let Some(c) = cond {
                    out.push_str(&format!("        if {} {{\n            {}\n        }}\n", c, statement));
                } else {
                    out.push_str(&format!("        {}\n", statement));
                }
                out
            };

            if f.persist_if_default {
                if f.is_flags {
                    code.push_str(&emit_field(format!("writer.write_flags_chunk(0x{:02X}, self.{}, writer.is_2k3())?;", cid, f.rust_name), None));
                } else if t == "i32" || t == "i8" {
                    code.push_str(&emit_field(format!("writer.write_int_chunk(0x{:02X}, self.{} as i32)?;", cid, f.rust_name), None));
                } else if t == "bool" {
                    code.push_str(&emit_field(format!("writer.write_bool_chunk(0x{:02X}, self.{})?;", cid, f.rust_name), None));
                } else if t == "DBString" {
                    code.push_str(&emit_field(format!("writer.write_dbstring_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), None));
                } else if t == "Rect" {
                    code.push_str(&emit_field(format!("writer.write_rect_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), None));
                } else if t == "Vec<u8>" {
                    code.push_str(&emit_field(format!("writer.write_vector_u8_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), None));
                } else if t == "Vec<i16>" {
                    code.push_str(&emit_field(format!("writer.write_vector_i16_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), None));
                } else if t == "Parameters" || t == "Equipment" || t == "MoveRoute" || t == "EventPageCondition" {
                    code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| self.{}.write_lcf(w))?;", cid, f.rust_name), None));
                } else if t.starts_with("Vec<") {
                    let inner = &t[4..t.len() - 1];
                    if inner == "EventCommand" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            for cmd in &self.{} {{ cmd.write_lcf(w)?; }}\n            for _ in 0..4 {{ w.write_int(0)?; }}\n            Ok(())\n        }})?;", cid, f.rust_name), None));
                    } else if inner == "MoveCommand" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            for cmd in &self.{} {{ cmd.write_lcf(w)?; }}\n            Ok(())\n        }})?;", cid, f.rust_name), None));
                    } else if inner == "DBString" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for s in &self.{} {{\n                if let Some(raw) = s.raw_bytes() {{\n                    w.write_int(raw.len() as i32)?;\n                    w.write_bytes(raw)?;\n                }} else {{\n                    let enc = w.encoder().encode(s.as_str());\n                    w.write_int(enc.len() as i32)?;\n                    w.write_bytes(&enc)?;\n                }}\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), None));
                    } else if inner == "i32" || inner == "u8" || inner == "i16" || inner == "u32" || inner == "bool" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            for &item in &self.{} {{ w.write_int(item as i32)?; }}\n            Ok(())\n        }})?;", cid, f.rust_name), None));
                    } else {
                        let inner_has_id = all_structs.get(inner).map_or(false, |s_def| s_def.has_id);
                        if inner_has_id {
                            code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for item in &self.{} {{\n                w.write_int(item.id)?;\n                item.write_lcf(w)?;\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), None));
                        } else {
                            code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for item in &self.{} {{\n                item.write_lcf(w)?;\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), None));
                        }
                    }
                } else if all_structs.contains_key(t) {
                    code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| self.{}.write_lcf(w))?;", cid, f.rust_name), None));
                }
            } else {
                if f.is_flags {
                    code.push_str(&emit_field(format!("writer.write_flags_chunk(0x{:02X}, self.{}, writer.is_2k3())?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "i32" || t == "i16" || t == "u8" || t == "u32" || t == "i8" {
                    code.push_str(&emit_field(format!("writer.write_int_chunk(0x{:02X}, self.{} as i32)?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "f64" {

                    code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| w.write_f64(self.{}))?;", cid, f.rust_name), Some(format!("(self.{} - _def.{}).abs() > f64::EPSILON", f.rust_name, f.rust_name))));
                } else if t == "bool" {
                    code.push_str(&emit_field(format!("writer.write_bool_chunk(0x{:02X}, self.{})?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "DBString" {
                    code.push_str(&emit_field(format!("writer.write_dbstring_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "Vec<u8>" {
                    code.push_str(&emit_field(format!("writer.write_vector_u8_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "Vec<i16>" {
                    code.push_str(&emit_field(format!("writer.write_vector_i16_chunk(0x{:02X}, &self.{})?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t == "Vec<u32>" {
                    code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            for &x in &self.{} {{ w.write_u32_le(x)?; }}\n            Ok(())\n        }})?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                } else if t.starts_with("Vec<") {
                    let inner = &t[4..t.len() - 1];
                    let inner_has_id = all_structs.get(inner).map_or(false, |s_def| s_def.has_id);
                    if inner_has_id {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for item in &self.{} {{\n                w.write_int(item.id)?;\n                item.write_lcf(w)?;\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), Some(format!("!self.{}.is_empty()", f.rust_name))));
                    } else if inner == "DBString" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for s in &self.{} {{\n                if let Some(raw) = s.raw_bytes() {{\n                    w.write_int(raw.len() as i32)?;\n                    w.write_bytes(raw)?;\n                }} else {{\n                    let enc = w.encoder().encode(s.as_str());\n                    w.write_int(enc.len() as i32)?;\n                    w.write_bytes(&enc)?;\n                }}\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), Some(format!("!self.{}.is_empty()", f.rust_name))));
                    } else if inner == "i32" || inner == "u8" || inner == "i16" || inner == "u32" || inner == "bool" {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            for &item in &self.{} {{ w.write_int(item as i32)?; }}\n            Ok(())\n        }})?;", cid, f.rust_name), Some(format!("!self.{}.is_empty()", f.rust_name))));
                    } else {
                        code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| {{\n            w.write_int(self.{}.len() as i32)?;\n            for item in &self.{} {{\n                item.write_lcf(w)?;\n            }}\n            Ok(())\n        }})?;", cid, f.rust_name, f.rust_name), Some(format!("!self.{}.is_empty()", f.rust_name))));
                    }
                } else if all_structs.contains_key(t) {
                    code.push_str(&emit_field(format!("writer.write_sub_chunk(0x{:02X}, |w| self.{}.write_lcf(w))?;", cid, f.rust_name), Some(format!("self.{} != _def.{}", f.rust_name, f.rust_name))));
                }
            }
        }
    }



    if s.name != "Database" && s.name != "Save" {
        code.push_str("        writer.write_int(0)?;\n");
    }
    code.push_str("        Ok(())\n");
    code.push_str("    }\n");

    // write_xml
    code.push_str("    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), crate::error::LcfError> {\n");
    if s.has_id {
        code.push_str(&format!("        writer.begin_element_with_id(\"{}\", self.id)?;\n", s.name));
    } else {
        code.push_str(&format!("        writer.begin_element(\"{}\")?;\n", s.name));
    }

    for f in &s.fields {
        if f.is_size_field {
            continue;
        }
        let t = f.rust_type.as_str();
        if t == "i32" || t == "i16" || t == "u8" || t == "u32" || t == "i8" {
            code.push_str(&format!("        writer.write_node_int(\"{}\", self.{} as i32)?;\n", f.field, f.rust_name));
        } else if t == "f64" {
            code.push_str(&format!("        writer.write_node_f64(\"{}\", self.{})?;\n", f.field, f.rust_name));
        } else if t == "bool" {
            code.push_str(&format!("        writer.write_node_bool(\"{}\", self.{})?;\n", f.field, f.rust_name));
        } else if t == "DBString" {
            code.push_str(&format!("        writer.write_node_dbstring(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "Rect" {
            code.push_str(&format!("        writer.write_node_rect(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "DBBitArray" {
            code.push_str(&format!("        writer.write_node_vector_bool(\"{}\", &self.{}.0)?;\n", f.field, f.rust_name));
        } else if t == "Vec<i16>" {
            code.push_str(&format!("        writer.write_node_vector_i16(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "Vec<u8>" {
            code.push_str(&format!("        writer.write_node_vector_u8(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "Vec<i32>" {
            code.push_str(&format!("        writer.write_node_vector_i32(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "Vec<u32>" {
            code.push_str(&format!("        writer.write_node_vector_u32(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t == "Vec<bool>" {
            code.push_str(&format!("        writer.write_node_vector_bool(\"{}\", &self.{})?;\n", f.field, f.rust_name));
        } else if t.starts_with("Vec<") {
            let inner = &t[4..t.len() - 1];
            if inner == "DBString" {
                // liblcf skips empty entries and writes non-empty ones as
                // <item id="N"> with 1-based id = original index+1
                // (dbstring_struct.cpp: WriteXml/DbStringVectorXmlHandler).
                code.push_str(&format!(
                    "        writer.begin_element(\"{}\")?;\n        for (idx, item) in self.{}.iter().enumerate() {{ if !item.as_str().is_empty() {{ writer.begin_element_with_id(\"item\", (idx + 1) as i32)?; writer.write_node_dbstring_value(item)?; writer.end_element(\"item\")?; }} }}\n        writer.end_element(\"{}\")?;\n",
                    f.field, f.rust_name, f.field
                ));
            } else if all_structs.contains_key(inner) {
                code.push_str(&format!("        writer.begin_element(\"{}\")?;\n        for item in &self.{} {{ item.write_xml(writer)?; }}\n        writer.end_element(\"{}\")?;\n", f.field, f.rust_name, f.field));
            }
        } else if all_structs.contains_key(t) {
            code.push_str(&format!("        writer.begin_element(\"{}\")?;\n        self.{}.write_xml(writer)?;\n        writer.end_element(\"{}\")?;\n", f.field, f.rust_name, f.field));
        }
    }

    code.push_str(&format!("        writer.end_element(\"{}\")?;\n", s.name));
    code.push_str("        Ok(())\n");
    code.push_str("    }\n");

    // read_xml: top-level, consumes this struct's own start tag (and thus
    // its id attribute) before delegating to read_xml_fields. Used by
    // format facades (Database/TreeMap/Map/Save) once the format-root tag
    // (<LDB>/<LMT>/<LMU>/<LSD>) has been consumed, and by any field that
    // nests a singular non-special-raw struct.
    code.push_str("    pub fn read_xml<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>, is_2k3: bool) -> Result<Self, crate::error::LcfError> {\n");
    code.push_str("        match reader.next_child()? {\n");
    code.push_str("            Some(tag) => Self::read_xml_fields(reader, tag.id.unwrap_or(0), is_2k3),\n");
    code.push_str("            None => Ok(Self::default_for_engine(is_2k3)),\n");
    code.push_str("        }\n");
    code.push_str("    }\n");

    // read_xml_fields: assumes this struct's own start tag was already
    // consumed by the caller (mirrors write_xml, which always emits its
    // own tag) - the mirror mechanism used for nested/repeated struct
    // fields, where the wrapping field-name tag (and, for Vec<Struct>,
    // each item's own tag) is consumed by the parent's dispatch loop.
    code.push_str("    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>, id: i32, is_2k3: bool) -> Result<Self, crate::error::LcfError> {\n");
    code.push_str("        let mut obj = Self::default_for_engine(is_2k3);\n");
    if s.has_id {
        code.push_str("        obj.id = id;\n");
    }
    code.push_str("        loop {\n");
    code.push_str("            match reader.next_child()? {\n");
    code.push_str("                None => break,\n");
    code.push_str("                Some(tag) => match tag.name.as_str() {\n");

    for f in &s.fields {
        if f.is_size_field {
            continue;
        }
        let t = f.rust_type.as_str();
        if t == "i32" || t == "i8" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_int()? as {},\n", f.field, f.rust_name, t));
        } else if t == "i16" || t == "u8" || t == "u32" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_int()? as {},\n", f.field, f.rust_name, t));
        } else if t == "f64" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_f64()?,\n", f.field, f.rust_name));
        } else if t == "bool" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_bool()?,\n", f.field, f.rust_name));
        } else if t == "DBString" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_dbstring()?,\n", f.field, f.rust_name));
        } else if t == "DBBitArray" {
            code.push_str(&format!("                    \"{}\" => obj.{} = crate::types::DBBitArray(reader.read_node_vector_bool()?),\n", f.field, f.rust_name));
        } else if t == "Vec<i16>" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_vector_i16()?,\n", f.field, f.rust_name));
        } else if t == "Vec<u8>" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_vector_u8()?,\n", f.field, f.rust_name));
        } else if t == "Vec<i32>" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_vector_i32()?,\n", f.field, f.rust_name));
        } else if t == "Vec<u32>" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_vector_u32()?,\n", f.field, f.rust_name));
        } else if t == "Vec<bool>" {
            code.push_str(&format!("                    \"{}\" => obj.{} = reader.read_node_vector_bool()?,\n", f.field, f.rust_name));
        } else if t == "Rect" || all_structs.contains_key(t) {
            // Singular struct-typed field: field-name wrap around exactly
            // one instance of the type's own self-tag (mirrors
            // XmlWriter::write_node_rect / the all_structs write_xml branch).
            let read_call = if is_special_raw_name(t) {
                format!("{}::read_xml_fields(reader)?", t)
            } else {
                format!("{}::read_xml_fields(reader, inner.id.unwrap_or(0), is_2k3)?", t)
            };
            code.push_str(&format!(
                "                    \"{field}\" => obj.{name} = match reader.next_child()? {{ Some(inner) => {{ let v = {call}; reader.consume_wrapper_end()?; v }}, None => {ty}::default() }},\n",
                field = f.field, name = f.rust_name, call = read_call, ty = t
            ));
        } else if t.starts_with("Vec<") {
            let inner = &t[4..t.len() - 1];
            if inner == "DBString" {
                // Sparse: <item id="N">value</item>, empty entries omitted
                // (mirrors dbstring_struct.cpp's resize(id) reconstruction).
                code.push_str(&format!(
                    "                    \"{field}\" => {{\n                        let mut v: Vec<crate::types::DBString> = Vec::new();\n                        loop {{\n                            match reader.next_child()? {{\n                                None => break,\n                                Some(item_tag) => {{\n                                    let text = reader.read_node_string()?;\n                                    let id = item_tag.id.unwrap_or(0) as usize;\n                                    if id >= 1 {{\n                                        if id > v.len() {{ v.resize(id, crate::types::DBString::default()); }}\n                                        v[id - 1] = crate::types::DBString::new(text);\n                                    }}\n                                }}\n                            }}\n                        }}\n                        obj.{name} = v;\n                    }},\n",
                    field = f.field, name = f.rust_name
                ));
            } else if all_structs.contains_key(inner) {
                let item_read_call = if is_special_raw_name(inner) {
                    format!("{}::read_xml_fields(reader)?", inner)
                } else {
                    format!("{}::read_xml_fields(reader, item_tag.id.unwrap_or(0), is_2k3)?", inner)
                };
                code.push_str(&format!(
                    "                    \"{field}\" => {{\n                        loop {{\n                            match reader.next_child()? {{\n                                None => break,\n                                Some(item_tag) => obj.{name}.push({call}),\n                            }}\n                        }}\n                    }},\n",
                    field = f.field, name = f.rust_name, call = item_read_call
                ));
            } else {
                code.push_str(&format!("                    \"{}\" => reader.skip_to_end()?,\n", f.field));
            }
        } else {
            code.push_str(&format!("                    \"{}\" => reader.skip_to_end()?,\n", f.field));
        }
    }

    code.push_str("                    _ => reader.skip_to_end()?,\n");
    code.push_str("                },\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("        Ok(obj)\n");
    code.push_str("    }\n");

    code.push_str("}\n\n");
    code
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csv_dir = PathBuf::from(r"d:\programacion\liblcf\generator\csv");
    let out_dir = PathBuf::from(r"d:\programacion\easy-rpg REditor\crates\lcf-core\src\generated");
    fs::create_dir_all(&out_dir)?;

    println!("Reading schema from {:?}...", csv_dir);

    // Read enums
    let mut enums: BTreeMap<String, Vec<(String, i32)>> = BTreeMap::new();
    for filename in &["enums.csv", "enums_easyrpg.csv"] {
        let path = csv_dir.join(filename);
        if !path.exists() {
            continue;
        }
        let file = File::open(&path)?;
        let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(BufReader::new(file));
        for result in rdr.deserialize() {
            let row: EnumRow = result?;
            let enum_name = format!("{}_{}", row.structure.trim(), row.entry.trim());
            let idx = row.index.trim().parse::<i32>().unwrap_or(0);
            enums.entry(enum_name).or_default().push((row.value.trim().to_string(), idx));
        }
    }

    // Read structs
    let mut structs: BTreeMap<String, StructDef> = BTreeMap::new();
    for filename in &["structs.csv", "structs_easyrpg.csv"] {
        let path = csv_dir.join(filename);
        if !path.exists() {
            continue;
        }
        let file = File::open(&path)?;
        let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(BufReader::new(file));
        for result in rdr.deserialize() {
            let row: StructRow = result?;
            let has_id = row.index_available.trim() == "1";
            let base = if row.base.trim().is_empty() {
                None
            } else {
                Some(row.base.trim().to_string())
            };
            structs.insert(
                row.structure.clone(),
                StructDef {
                    file_type: row.file_type.trim().to_lowercase(),
                    name: row.structure,
                    base,
                    has_id,
                    fields: Vec::new(),
                },
            );
        }
    }

    // Read fields
    for filename in &["fields.csv", "fields_easyrpg.csv"] {
        let path = csv_dir.join(filename);
        if !path.exists() {
            continue;
        }
        let file = File::open(&path)?;
        let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(BufReader::new(file));
        for result in rdr.deserialize() {
            let row: FieldRow = result?;
            if let Some(s) = structs.get_mut(&row.structure) {
                let chunk_id = parse_hex_or_dec(&row.index);
                let persist = row.persist_if_default.trim() == "1";
                let is_2k3 = row.is_2k3.trim() == "1";
                let is_size = row.size_field.trim() == "t" || row.size_field.trim() == "s";
                let is_flags = row.field_type.trim().ends_with("_Flags") || row.field.trim() == "flags" || row.field.trim() == "special_flags";
                let rust_name = sanitize_field_name(&row.field);
                let rust_type = map_type_to_rust(&row.field_type);
                s.fields.push(FieldDef {
                    field: row.field,
                    rust_name,
                    rust_type,
                    chunk_id,
                    default_val: row.default_value.trim().to_string(),
                    persist_if_default: persist,
                    is_2k3,
                    is_size_field: is_size,
                    is_flags,
                });

            }
        }
    }

    // Flatten C++ inheritance (e.g. SavePartyLocation, SaveVehicleLocation, SaveMapEvent inherit from SaveMapEventBase)
    let base_fields_map: HashMap<String, Vec<FieldDef>> = structs
        .iter()
        .map(|(k, v)| (k.clone(), v.fields.clone()))
        .collect();

    for s in structs.values_mut() {
        if let Some(base_name) = &s.base {
            if let Some(base_fields) = base_fields_map.get(base_name) {
                let mut combined = base_fields.clone();
                combined.extend(s.fields.drain(..));
                // Sort fields by chunk_id so they appear in chunk order
                combined.sort_by_key(|f| f.chunk_id.unwrap_or(u32::MAX));
                s.fields = combined;
            }
        }
    }

    println!("Parsed {} structs and {} enums (with base-class inheritance flattened).", structs.len(), enums.len());

    let all_structs: HashMap<String, StructDef> = structs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    // Generate enums.rs
    let mut enums_code = String::from("// Generated by lcf-codegen. DO NOT EDIT.\n#![allow(non_camel_case_types, dead_code, non_snake_case)]\n\n");
    for (name, variants) in &enums {
        enums_code.push_str(&format!("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n#[repr(i32)]\npub enum {} {{\n", name));
        for (v_name, v_idx) in variants {
            let var_name = heck::AsPascalCase(v_name).to_string();
            let var_name = if var_name == "Self" {
                "Self_".to_string()
            } else if var_name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                format!("Val{}", var_name)
            } else {
                var_name
            };
            enums_code.push_str(&format!("    {} = {},\n", var_name, v_idx));
        }
        enums_code.push_str("}\n\n");
    }
    fs::write(out_dir.join("enums.rs"), enums_code)?;

    // Group structs by file_type: ldb, lmt, lmu, lsd
    let mut ldb_code = String::from("// Generated by lcf-codegen. DO NOT EDIT.\n#![allow(non_camel_case_types, dead_code, non_snake_case, unused_variables, unused_imports, clippy::all)]\nuse crate::types::*;\nuse crate::generated::lmu_gen::*;\nuse crate::generated::lmt_gen::*;\n\n");
    let mut lmt_code = String::from("// Generated by lcf-codegen. DO NOT EDIT.\n#![allow(non_camel_case_types, dead_code, non_snake_case, unused_variables, unused_imports, clippy::all)]\nuse crate::types::*;\nuse crate::generated::lmu_gen::*;\n\n");
    let mut lmu_code = String::from("// Generated by lcf-codegen. DO NOT EDIT.\n#![allow(non_camel_case_types, dead_code, non_snake_case, unused_variables, unused_imports, clippy::all)]\nuse crate::types::*;\n\n");
    let mut lsd_code = String::from("// Generated by lcf-codegen. DO NOT EDIT.\n#![allow(non_camel_case_types, dead_code, non_snake_case, unused_variables, unused_imports, clippy::all)]\nuse crate::types::*;\nuse crate::generated::lmu_gen::*;\nuse crate::generated::ldb_gen::*;\nuse crate::generated::lmt_gen::*;\n\n");

    for (_, s) in &structs {
        let code = generate_struct_code(s, &all_structs);
        match s.file_type.as_str() {
            "ldb" => ldb_code.push_str(&code),
            "lmt" => lmt_code.push_str(&code),
            "lmu" => lmu_code.push_str(&code),
            "lsd" => lsd_code.push_str(&code),
            _ => ldb_code.push_str(&code),
        }
    }

    fs::write(out_dir.join("ldb_gen.rs"), ldb_code)?;
    fs::write(out_dir.join("lmt_gen.rs"), lmt_code)?;
    fs::write(out_dir.join("lmu_gen.rs"), lmu_code)?;
    fs::write(out_dir.join("lsd_gen.rs"), lsd_code)?;

    // Generate mod.rs
    let mod_code = r#"// Generated by lcf-codegen. DO NOT EDIT.

pub mod enums;
pub mod ldb_gen;
pub mod lmt_gen;
pub mod lmu_gen;
pub mod lsd_gen;

pub use enums::*;
pub use ldb_gen::*;
pub use lmt_gen::*;
pub use lmu_gen::*;
pub use lsd_gen::*;
"#;
    fs::write(out_dir.join("mod.rs"), mod_code)?;

    println!("Full codegen with flattened inheritance completed successfully.");
    Ok(())
}
