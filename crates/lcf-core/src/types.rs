use std::fmt;
use std::ops::{Deref, DerefMut};
use crate::error::LcfError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineVersion {
    Engine2000,
    Engine2003,
}

impl Default for EngineVersion {
    fn default() -> Self {
        EngineVersion::Engine2000
    }
}

impl EngineVersion {
    pub fn is_2k3(&self) -> bool {
        matches!(self, EngineVersion::Engine2003)
    }
}

/// A string from the RPG Maker database/project.
#[derive(Clone, Default, Hash)]
pub struct DBString(pub String, pub Option<Vec<u8>>);

impl PartialEq for DBString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DBString {}

impl PartialEq<str> for DBString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for DBString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for DBString {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}


impl DBString {
    pub fn new(s: impl Into<String>) -> Self {
        DBString(s.into(), None)
    }

    pub fn from_raw(raw: Vec<u8>, decoded: String) -> Self {
        DBString(decoded, Some(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn raw_bytes(&self) -> Option<&[u8]> {
        self.1.as_deref()
    }
}

impl From<String> for DBString {
    fn from(s: String) -> Self {
        DBString::new(s)
    }
}

impl From<&str> for DBString {
    fn from(s: &str) -> Self {
        DBString::new(s)
    }
}

impl Deref for DBString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DBString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1 = None;
        &mut self.0
    }
}

impl fmt::Debug for DBString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for DBString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}



/// A dynamic bit-array stored as boolean vector.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DBBitArray(pub Vec<bool>);

impl DBBitArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get_bit(&self, index: usize) -> bool {
        self.0.get(index).copied().unwrap_or(false)
    }

    pub fn set_bit(&mut self, index: usize, value: bool) {
        if index >= self.0.len() {
            self.0.resize(index + 1, false);
        }
        self.0[index] = value;
    }
}


/// Rect bounds structure.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Rect {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

impl Rect {
    /// Self-contained `<Rect><l>..</l><t>..</t><r>..</r><b>..</b></Rect>`,
    /// matching `RawStruct<rpg::Rect>::WriteXml` (`lmt_rect.cpp`). Note this
    /// does not add a field-name wrapper - callers writing a `Rect`-typed
    /// field should use `XmlWriter::write_node_rect(field_name, rect)`
    /// instead, which supplies that wrapper.
    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("Rect")?;
        writer.write_node_int("l", self.l)?;
        writer.write_node_int("t", self.t)?;
        writer.write_node_int("r", self.r)?;
        writer.write_node_int("b", self.b)?;
        writer.end_element("Rect")
    }

    /// Reads `l`/`t`/`r`/`b` fields, assuming this type's own `<Rect>` start
    /// tag was already consumed by the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut rect = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "l" => rect.l = reader.read_node_int()?,
                    "t" => rect.t = reader.read_node_int()?,
                    "r" => rect.r = reader.read_node_int()?,
                    "b" => rect.b = reader.read_node_int()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(rect)
    }
}

/// Equipment IDs for an actor.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Equipment {
    pub weapon_id: i32,
    pub shield_id: i32,
    pub armor_id: i32,
    pub helmet_id: i32,
    pub accessory_id: i32,
}

impl Equipment {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>, length: usize) -> Result<Self, LcfError> {
        let count = length / 2;
        let mut eq = Self::default();
        if count > 0 { eq.weapon_id = reader.read_i16()? as i32; }
        if count > 1 { eq.shield_id = reader.read_i16()? as i32; }
        if count > 2 { eq.armor_id = reader.read_i16()? as i32; }
        if count > 3 { eq.helmet_id = reader.read_i16()? as i32; }
        if count > 4 { eq.accessory_id = reader.read_i16()? as i32; }
        Ok(eq)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_i16(self.weapon_id as i16)?;
        writer.write_i16(self.shield_id as i16)?;
        writer.write_i16(self.armor_id as i16)?;
        writer.write_i16(self.helmet_id as i16)?;
        writer.write_i16(self.accessory_id as i16)?;
        Ok(())
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("Equipment")?;
        writer.write_node_int("weapon_id", self.weapon_id)?;
        writer.write_node_int("shield_id", self.shield_id)?;
        writer.write_node_int("armor_id", self.armor_id)?;
        writer.write_node_int("helmet_id", self.helmet_id)?;
        writer.write_node_int("accessory_id", self.accessory_id)?;
        writer.end_element("Equipment")?;
        Ok(())
    }

    /// Assumes this type's own `<Equipment>` start tag was already consumed
    /// by the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut eq = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "weapon_id" => eq.weapon_id = reader.read_node_int()?,
                    "shield_id" => eq.shield_id = reader.read_node_int()?,
                    "armor_id" => eq.armor_id = reader.read_node_int()?,
                    "helmet_id" => eq.helmet_id = reader.read_node_int()?,
                    "accessory_id" => eq.accessory_id = reader.read_node_int()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(eq)
    }
}

/// Actor stats parameters table across levels.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Parameters {
    pub maxhp: Vec<i16>,
    pub maxsp: Vec<i16>,
    pub attack: Vec<i16>,
    pub defense: Vec<i16>,
    pub spirit: Vec<i16>,
    pub agility: Vec<i16>,
}

impl Parameters {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>, length: usize) -> Result<Self, LcfError> {
        let count = length / (6 * 2);
        let mut p = Self {
            maxhp: Vec::with_capacity(count),
            maxsp: Vec::with_capacity(count),
            attack: Vec::with_capacity(count),
            defense: Vec::with_capacity(count),
            spirit: Vec::with_capacity(count),
            agility: Vec::with_capacity(count),
        };
        for _ in 0..count { p.maxhp.push(reader.read_i16()?); }
        for _ in 0..count { p.maxsp.push(reader.read_i16()?); }
        for _ in 0..count { p.attack.push(reader.read_i16()?); }
        for _ in 0..count { p.defense.push(reader.read_i16()?); }
        for _ in 0..count { p.spirit.push(reader.read_i16()?); }
        for _ in 0..count { p.agility.push(reader.read_i16()?); }
        Ok(p)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        for &val in &self.maxhp { writer.write_i16(val)?; }
        for &val in &self.maxsp { writer.write_i16(val)?; }
        for &val in &self.attack { writer.write_i16(val)?; }
        for &val in &self.defense { writer.write_i16(val)?; }
        for &val in &self.spirit { writer.write_i16(val)?; }
        for &val in &self.agility { writer.write_i16(val)?; }
        Ok(())
    }


    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("Parameters")?;
        writer.write_node_vector_i16("maxhp", &self.maxhp)?;
        writer.write_node_vector_i16("maxsp", &self.maxsp)?;
        writer.write_node_vector_i16("attack", &self.attack)?;
        writer.write_node_vector_i16("defense", &self.defense)?;
        writer.write_node_vector_i16("spirit", &self.spirit)?;
        writer.write_node_vector_i16("agility", &self.agility)?;
        writer.end_element("Parameters")?;
        Ok(())
    }

    /// Assumes this type's own `<Parameters>` start tag was already
    /// consumed by the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut p = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "maxhp" => p.maxhp = reader.read_node_vector_i16()?,
                    "maxsp" => p.maxsp = reader.read_node_vector_i16()?,
                    "attack" => p.attack = reader.read_node_vector_i16()?,
                    "defense" => p.defense = reader.read_node_vector_i16()?,
                    "spirit" => p.spirit = reader.read_node_vector_i16()?,
                    "agility" => p.agility = reader.read_node_vector_i16()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(p)
    }
}

/// RPG Maker Music specification.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Music {
    pub name: DBString,
    pub fadein: i32,
    pub volume: i32,
    pub tempo: i32,
    pub balance: i32,
}

impl Music {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, LcfError> {
        let mut m = Self::default();
        while let Some(chunk) = reader.read_chunk_header()? {
            let start = reader.tell()?;
            match chunk.id {
                0x01 => m.name = reader.read_string(chunk.length)?,
                0x02 => m.fadein = reader.read_int()?,
                0x03 => m.volume = reader.read_int()?,
                0x04 => m.tempo = reader.read_int()?,
                0x05 => m.balance = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(m)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if self.fadein != 0 {
            writer.write_int_chunk(0x02, self.fadein)?;
        }
        if self.volume != 100 {
            writer.write_int_chunk(0x03, self.volume)?;
        }
        if self.tempo != 100 {
            writer.write_int_chunk(0x04, self.tempo)?;
        }
        if self.balance != 50 {
            writer.write_int_chunk(0x05, self.balance)?;
        }
        writer.write_int(0)?;
        Ok(())
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("Music")?;
        writer.write_node_dbstring("name", &self.name)?;
        writer.write_node_int("fadein", self.fadein)?;
        writer.write_node_int("volume", self.volume)?;
        writer.write_node_int("tempo", self.tempo)?;
        writer.write_node_int("balance", self.balance)?;
        writer.end_element("Music")?;
        Ok(())
    }

    /// Assumes this type's own `<Music>` start tag was already consumed by
    /// the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut m = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "name" => m.name = reader.read_node_dbstring()?,
                    "fadein" => m.fadein = reader.read_node_int()?,
                    "volume" => m.volume = reader.read_node_int()?,
                    "tempo" => m.tempo = reader.read_node_int()?,
                    "balance" => m.balance = reader.read_node_int()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(m)
    }
}

/// RPG Maker Sound effect specification.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Sound {
    pub name: DBString,
    pub volume: i32,
    pub tempo: i32,
    pub balance: i32,
}

impl Sound {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, LcfError> {
        let mut s = Self {
            name: DBString::new("(OFF)"),
            volume: 100,
            tempo: 100,
            balance: 50,
        };
        while let Some(chunk) = reader.read_chunk_header()? {
            let start = reader.tell()?;
            match chunk.id {
                0x01 => s.name = reader.read_string(chunk.length)?,
                0x03 => s.volume = reader.read_int()?,
                0x04 => s.tempo = reader.read_int()?,
                0x05 => s.balance = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(s)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_dbstring_chunk(0x01, &self.name)?;
        if self.volume != 100 {
            writer.write_int_chunk(0x03, self.volume)?;
        }
        if self.tempo != 100 {
            writer.write_int_chunk(0x04, self.tempo)?;
        }
        if self.balance != 50 {
            writer.write_int_chunk(0x05, self.balance)?;
        }
        writer.write_int(0)?;
        Ok(())
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("Sound")?;
        writer.write_node_dbstring("name", &self.name)?;
        writer.write_node_int("volume", self.volume)?;
        writer.write_node_int("tempo", self.tempo)?;
        writer.write_node_int("balance", self.balance)?;
        writer.end_element("Sound")?;
        Ok(())
    }

    /// Assumes this type's own `<Sound>` start tag was already consumed by
    /// the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut s = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "name" => s.name = reader.read_node_dbstring()?,
                    "volume" => s.volume = reader.read_node_int()?,
                    "tempo" => s.tempo = reader.read_node_int()?,
                    "balance" => s.balance = reader.read_node_int()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(s)
    }
}

/// An Event Command instruction.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventCommand {
    pub code: i32,
    pub indent: i32,
    pub string: DBString,
    pub parameters: Vec<i32>,
}

impl EventCommand {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, LcfError> {
        let code = reader.read_int()?;
        let indent = reader.read_int()?;
        let str_len = reader.read_int()? as usize;
        let string = reader.read_string(str_len)?;
        let param_count = reader.read_int()? as usize;
        let mut parameters = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            parameters.push(reader.read_int()?);
        }
        Ok(Self {
            code,
            indent,
            string,
            parameters,
        })
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.code)?;
        writer.write_int(self.indent)?;
        if let Some(raw) = self.string.raw_bytes() {
            writer.write_int(raw.len() as i32)?;
            writer.write_bytes(raw)?;
        } else {
            let encoded = writer.encoder().encode(self.string.as_str());
            writer.write_int(encoded.len() as i32)?;
            writer.write_bytes(&encoded)?;
        }
        writer.write_int(self.parameters.len() as i32)?;
        for &p in &self.parameters {
            writer.write_int(p)?;
        }
        Ok(())
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("EventCommand")?;
        writer.write_node_int("code", self.code)?;
        writer.write_node_int("indent", self.indent)?;
        writer.write_node_dbstring("string", &self.string)?;
        writer.write_node_vector_i32("parameters", &self.parameters)?;
        writer.end_element("EventCommand")?;
        Ok(())
    }

    /// Assumes this type's own `<EventCommand>` start tag was already
    /// consumed by the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut c = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "code" => c.code = reader.read_node_int()?,
                    "indent" => c.indent = reader.read_node_int()?,
                    "string" => c.string = reader.read_node_dbstring()?,
                    "parameters" => c.parameters = reader.read_node_vector_i32()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(c)
    }
}

/// A Move Route instruction.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MoveCommand {
    pub code: i32,
    pub parameter_a: i32,
    pub parameter_b: i32,
    pub parameter_c: i32,
    pub string: DBString,
}

impl MoveCommand {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, LcfError> {
        let code = reader.read_int()?;
        let mut cmd = Self {
            code,
            parameter_a: 0,
            parameter_b: 0,
            parameter_c: 0,
            string: DBString::default(),
        };
        match code {
            32 | 33 => {
                cmd.parameter_a = reader.read_int()?;
            }
            34 => {
                let len = reader.read_int()? as usize;
                cmd.string = reader.read_string(len)?;
                cmd.parameter_a = reader.read_int()?;
            }
            35 => {
                let len = reader.read_int()? as usize;
                cmd.string = reader.read_string(len)?;
                cmd.parameter_a = reader.read_int()?;
                cmd.parameter_b = reader.read_int()?;
                cmd.parameter_c = reader.read_int()?;
            }
            _ => {}
        }
        Ok(cmd)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.code)?;
        match self.code {
            32 | 33 => {
                writer.write_int(self.parameter_a)?;
            }
            34 => {
                if let Some(raw) = self.string.raw_bytes() {
                    writer.write_int(raw.len() as i32)?;
                    writer.write_bytes(raw)?;
                } else {
                    let encoded = writer.encoder().encode(self.string.as_str());
                    writer.write_int(encoded.len() as i32)?;
                    writer.write_bytes(&encoded)?;
                }
                writer.write_int(self.parameter_a)?;
            }
            35 => {
                if let Some(raw) = self.string.raw_bytes() {
                    writer.write_int(raw.len() as i32)?;
                    writer.write_bytes(raw)?;
                } else {
                    let encoded = writer.encoder().encode(self.string.as_str());
                    writer.write_int(encoded.len() as i32)?;
                    writer.write_bytes(&encoded)?;
                }
                writer.write_int(self.parameter_a)?;
                writer.write_int(self.parameter_b)?;
                writer.write_int(self.parameter_c)?;
            }
            _ => {}
        }
        Ok(())
    }



    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("MoveCommand")?;
        writer.write_node_int("code", self.code)?;
        writer.write_node_int("parameter_a", self.parameter_a)?;
        writer.write_node_int("parameter_b", self.parameter_b)?;
        writer.write_node_int("parameter_c", self.parameter_c)?;
        writer.write_node_dbstring("string", &self.string)?;
        writer.end_element("MoveCommand")?;
        Ok(())
    }

    /// Assumes this type's own `<MoveCommand>` start tag was already
    /// consumed by the caller (mirrors `write_xml` above).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut c = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "code" => c.code = reader.read_node_int()?,
                    "parameter_a" => c.parameter_a = reader.read_node_int()?,
                    "parameter_b" => c.parameter_b = reader.read_node_int()?,
                    "parameter_c" => c.parameter_c = reader.read_node_int()?,
                    "string" => c.string = reader.read_node_dbstring()?,
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(c)
    }
}

/// Map Tree Map struct holding hierarchy, order, and start location.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct TreeMap {

    pub maps: Vec<crate::generated::lmt_gen::MapInfo>,
    pub tree_order: Vec<i32>,
    pub active_node: i32,
    pub start: crate::generated::lmt_gen::Start,
}

impl TreeMap {
    pub fn read_lcf<R: std::io::Read + std::io::Seek>(reader: &mut crate::reader::LcfReader<R>) -> Result<Self, LcfError> {
        let mut tmap = Self::default();
        let maps_count = reader.read_int()?;
        tmap.maps.reserve(maps_count as usize);
        for _ in 0..maps_count {
            let id = reader.read_int()?;
            let mut map_info = crate::generated::lmt_gen::MapInfo::read_lcf(reader)?;
            map_info.id = id;
            tmap.maps.push(map_info);
        }

        let order_count = reader.read_int()?;
        tmap.tree_order.reserve(order_count as usize);
        for _ in 0..order_count {
            tmap.tree_order.push(reader.read_int()?);
        }

        tmap.active_node = reader.read_int()?;
        tmap.start = crate::generated::lmt_gen::Start::read_lcf(reader)?;
        Ok(tmap)
    }

    pub fn write_lcf<W: std::io::Write + std::io::Seek>(&self, writer: &mut crate::writer::LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.maps.len() as i32)?;
        for map_info in &self.maps {
            writer.write_int(map_info.id)?;
            map_info.write_lcf(writer)?;
        }

        writer.write_int(self.tree_order.len() as i32)?;
        for &id in &self.tree_order {
            writer.write_int(id)?;
        }

        writer.write_int(self.active_node)?;
        self.start.write_lcf(writer)?;
        Ok(())
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut crate::xml::XmlWriter<W>) -> Result<(), LcfError> {
        writer.begin_element("TreeMap")?;
        writer.begin_element("maps")?;
        for map_info in &self.maps {
            map_info.write_xml(writer)?;
        }
        writer.end_element("maps")?;
        writer.write_node_vector_i32("tree_order", &self.tree_order)?;
        writer.write_node_int("active_node", self.active_node)?;
        writer.begin_element("start")?;
        self.start.write_xml(writer)?;
        writer.end_element("start")?;
        writer.end_element("TreeMap")?;
        Ok(())
    }

    /// Reads a `<TreeMap>` element (self-tag included) written by
    /// `write_xml` above, or by upstream liblcf (`lmt_treemap.cpp`).
    /// Assumes the *enclosing* tag (e.g. `<LMT>`) was already consumed by
    /// the caller, and itself consumes `<TreeMap>...</TreeMap>`.
    pub fn read_xml<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        match reader.next_child()? {
            Some(_) => Self::read_xml_fields(reader),
            None => Ok(Self::default()),
        }
    }

    /// Same as `read_xml`, but assumes the `<TreeMap>` start tag itself was
    /// already consumed by the caller (used when `TreeMap` appears nested
    /// as a field, which it currently doesn't, but kept symmetric with the
    /// generated structs' `read_xml`/`read_xml_fields` split).
    pub fn read_xml_fields<R: std::io::BufRead>(reader: &mut crate::xml::XmlReader<R>) -> Result<Self, LcfError> {
        let mut tmap = Self::default();
        loop {
            match reader.next_child()? {
                None => break,
                Some(tag) => match tag.name.as_str() {
                    "maps" => {
                        loop {
                            match reader.next_child()? {
                                None => break,
                                Some(item_tag) => {
                                    let id = item_tag.id.unwrap_or(0);
                                    let info = crate::generated::lmt_gen::MapInfo::read_xml_fields(reader, id, false)?;
                                    tmap.maps.push(info);
                                }
                            }
                        }
                    }
                    "tree_order" => tmap.tree_order = reader.read_node_vector_i32()?,
                    "active_node" => tmap.active_node = reader.read_node_int()?,
                    "start" => {
                        match reader.next_child()? {
                            Some(inner) => {
                                tmap.start = crate::generated::lmt_gen::Start::read_xml_fields(reader, inner.id.unwrap_or(0), false)?;
                                reader.consume_wrapper_end()?;
                            }
                            None => {}
                        }
                    }
                    _ => reader.skip_to_end()?,
                },
            }
        }
        Ok(tmap)
    }
}
