use std::io::{Read, Seek, Write};
use crate::error::LcfError;
use crate::reader::LcfReader;
use crate::types::{DBString, Music, Rect};
use crate::writer::LcfWriter;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Encounter {
    pub id: i32,
    pub troop_id: i32,
}

impl Encounter {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut enc = Encounter::default();
        while let Some(chunk) = reader.read_chunk_header()? {
            let start = reader.tell()?;
            match chunk.id {
                0x01 => enc.troop_id = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(enc)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if self.troop_id != 0 {
            writer.write_int_chunk(0x01, self.troop_id)?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Start {
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

impl Start {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut start = Start::default();
        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => start.party_map_id = reader.read_int()?,
                0x02 => start.party_x = reader.read_int()?,
                0x03 => start.party_y = reader.read_int()?,
                0x0B => start.boat_map_id = reader.read_int()?,
                0x0C => start.boat_x = reader.read_int()?,
                0x0D => start.boat_y = reader.read_int()?,
                0x15 => start.ship_map_id = reader.read_int()?,
                0x16 => start.ship_x = reader.read_int()?,
                0x17 => start.ship_y = reader.read_int()?,
                0x1F => start.airship_map_id = reader.read_int()?,
                0x20 => start.airship_x = reader.read_int()?,
                0x21 => start.airship_y = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(start)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if self.party_map_id != 0 { writer.write_int_chunk(0x01, self.party_map_id)?; }
        if self.party_x != 0 { writer.write_int_chunk(0x02, self.party_x)?; }
        if self.party_y != 0 { writer.write_int_chunk(0x03, self.party_y)?; }
        if self.boat_map_id != 0 { writer.write_int_chunk(0x0B, self.boat_map_id)?; }
        if self.boat_x != 0 { writer.write_int_chunk(0x0C, self.boat_x)?; }
        if self.boat_y != 0 { writer.write_int_chunk(0x0D, self.boat_y)?; }
        if self.ship_map_id != 0 { writer.write_int_chunk(0x15, self.ship_map_id)?; }
        if self.ship_x != 0 { writer.write_int_chunk(0x16, self.ship_x)?; }
        if self.ship_y != 0 { writer.write_int_chunk(0x17, self.ship_y)?; }
        if self.airship_map_id != 0 { writer.write_int_chunk(0x1F, self.airship_map_id)?; }
        if self.airship_x != 0 { writer.write_int_chunk(0x20, self.airship_x)?; }
        if self.airship_y != 0 { writer.write_int_chunk(0x21, self.airship_y)?; }
        writer.write_int(0)?;
        Ok(())
    }
}

impl Music {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut m = Music {
            name: DBString::new("(OFF)"),
            fadein: 0,
            volume: 100,
            tempo: 100,
            balance: 50,
        };
        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => m.name = reader.read_string(chunk.length)?,
                0x02 => m.fadein = reader.read_int()?,
                0x03 => m.volume = reader.read_int()?,
                0x04 => m.tempo = reader.read_int()?,
                0x05 => m.balance = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(m)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        // Chunk 0x01: PersistIfDefault = 1
        writer.write_dbstring_chunk(0x01, &self.name)?;
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
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapInfo {
    pub id: i32,
    pub name: DBString,
    pub parent_map: i32,
    pub indentation: i32,
    pub r#type: i32,
    pub scrollbar_x: i32,
    pub scrollbar_y: i32,
    pub expanded_node: bool,
    pub music_type: i32,
    pub music: Music,
    pub background_type: i32,
    pub background_name: DBString,
    pub teleport: i32,
    pub escape: i32,
    pub save: i32,
    pub encounters: Vec<Encounter>,
    pub encounter_steps: i32,
    pub area_rect: Rect,
}

impl MapInfo {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut info = MapInfo {
            id: 0,
            name: DBString::default(),
            parent_map: 0,
            indentation: 0,
            r#type: -1,
            scrollbar_x: 0,
            scrollbar_y: 0,
            expanded_node: false,
            music_type: 0,
            music: Music::default(),
            background_type: 0,
            background_name: DBString::default(),
            teleport: 0,
            escape: 0,
            save: 0,
            encounters: Vec::new(),
            encounter_steps: 25,
            area_rect: Rect::default(),
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => info.name = reader.read_string(chunk.length)?,
                0x02 => info.parent_map = reader.read_int()?,
                0x03 => info.indentation = reader.read_int()?,
                0x04 => info.r#type = reader.read_int()?,
                0x05 => info.scrollbar_x = reader.read_int()?,
                0x06 => info.scrollbar_y = reader.read_int()?,
                0x07 => info.expanded_node = reader.read_bool()?,
                0x0B => info.music_type = reader.read_int()?,
                0x0C => info.music = Music::read_lcf(reader)?,
                0x15 => info.background_type = reader.read_int()?,
                0x16 => info.background_name = reader.read_string(chunk.length)?,
                0x1F => info.teleport = reader.read_int()?,
                0x20 => info.escape = reader.read_int()?,
                0x21 => info.save = reader.read_int()?,
                0x29 => {
                    let count = reader.read_int()?;
                    info.encounters.clear();
                    for _ in 0..count {
                        let enc_id = reader.read_int()?;
                        let mut enc = Encounter::read_lcf(reader)?;
                        enc.id = enc_id;
                        info.encounters.push(enc);
                    }
                }
                0x2C => info.encounter_steps = reader.read_int()?,
                0x33 => info.area_rect = reader.read_rect()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(info)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if self.parent_map != 0 {
            writer.write_int_chunk(0x02, self.parent_map)?;
        }
        if self.indentation != 0 {
            writer.write_int_chunk(0x03, self.indentation)?;
        }
        if self.r#type != -1 {
            writer.write_int_chunk(0x04, self.r#type)?;
        }
        if self.scrollbar_x != 0 {
            writer.write_int_chunk(0x05, self.scrollbar_x)?;
        }
        if self.scrollbar_y != 0 {
            writer.write_int_chunk(0x06, self.scrollbar_y)?;
        }
        if self.expanded_node {
            writer.write_bool_chunk(0x07, self.expanded_node)?;
        }
        // music_type: PersistIfDefault = 1
        writer.write_int_chunk(0x0B, self.music_type)?;
        // music: PersistIfDefault = 1
        writer.write_sub_chunk(0x0C, |w| self.music.write_lcf(w))?;
        // background_type: PersistIfDefault = 1
        writer.write_int_chunk(0x15, self.background_type)?;
        if !self.background_name.is_empty() {
            writer.write_dbstring_chunk(0x16, &self.background_name)?;
        }
        // teleport: PersistIfDefault = 1
        writer.write_int_chunk(0x1F, self.teleport)?;
        // escape: PersistIfDefault = 1
        writer.write_int_chunk(0x20, self.escape)?;
        // save: PersistIfDefault = 1
        writer.write_int_chunk(0x21, self.save)?;
        // encounters: PersistIfDefault = 1
        writer.write_sub_chunk(0x29, |w| {
            w.write_int(self.encounters.len() as i32)?;
            for enc_item in &self.encounters {
                w.write_int(enc_item.id)?;
                enc_item.write_lcf(w)?;
            }
            Ok(())
        })?;
        if self.encounter_steps != 25 {
            writer.write_int_chunk(0x2C, self.encounter_steps)?;
        }
        // area_rect: PersistIfDefault = 1
        writer.write_rect_chunk(0x33, &self.area_rect)?;
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TreeMap {
    pub header: String,
    pub maps: Vec<MapInfo>,
    pub tree_order: Vec<i32>,
    pub active_node: i32,
    pub start: Start,
}

impl TreeMap {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut tmap = TreeMap::default();
        let count = reader.read_int()?;
        tmap.maps.clear();
        for _ in 0..count {
            let id = reader.read_int()?;
            let mut info = MapInfo::read_lcf(reader)?;
            info.id = id;
            tmap.maps.push(info);
        }

        let order_count = reader.read_int()?;
        tmap.tree_order.clear();
        for _ in 0..order_count {
            tmap.tree_order.push(reader.read_int()?);
        }

        tmap.active_node = reader.read_int()?;
        tmap.start = Start::read_lcf(reader)?;
        Ok(tmap)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.maps.len() as i32)?;
        for map_info in &self.maps {
            writer.write_int(map_info.id)?;
            map_info.write_lcf(writer)?;
        }

        writer.write_int(self.tree_order.len() as i32)?;
        for &node in &self.tree_order {
            writer.write_int(node)?;
        }

        writer.write_int(self.active_node)?;
        self.start.write_lcf(writer)?;
        Ok(())
    }
}
