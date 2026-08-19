use std::io::{Read, Seek, Write};
use crate::error::LcfError;
use crate::reader::LcfReader;
use crate::types::{DBString, Parameters};
use crate::writer::LcfWriter;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Equipment {
    pub weapon_id: i16,
    pub shield_id: i16,
    pub armor_id: i16,
    pub helmet_id: i16,
    pub accessory_id: i16,
}

impl Equipment {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>, length: usize) -> Result<Self, LcfError> {
        if length != 10 {
            reader.skip(length)?;
            return Ok(Equipment::default());
        }
        Ok(Equipment {
            weapon_id: reader.read_i16()?,
            shield_id: reader.read_i16()?,
            armor_id: reader.read_i16()?,
            helmet_id: reader.read_i16()?,
            accessory_id: reader.read_i16()?,
        })
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_i16(self.weapon_id)?;
        writer.write_i16(self.shield_id)?;
        writer.write_i16(self.armor_id)?;
        writer.write_i16(self.helmet_id)?;
        writer.write_i16(self.accessory_id)?;
        Ok(())
    }
}

impl Parameters {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>, length: usize) -> Result<Self, LcfError> {
        let n = length / 6;
        Ok(Parameters {
            maxhp: reader.read_vector_i16(n)?,
            maxsp: reader.read_vector_i16(n)?,
            attack: reader.read_vector_i16(n)?,
            defense: reader.read_vector_i16(n)?,
            spirit: reader.read_vector_i16(n)?,
            agility: reader.read_vector_i16(n)?,
        })
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_vector_i16(&self.maxhp)?;
        writer.write_vector_i16(&self.maxsp)?;
        writer.write_vector_i16(&self.attack)?;
        writer.write_vector_i16(&self.defense)?;
        writer.write_vector_i16(&self.spirit)?;
        writer.write_vector_i16(&self.agility)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Learning {
    pub id: i32,
    pub level: i32,
    pub skill_id: i32,
}

impl Learning {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut learn = Learning {
            id: 0,
            level: 1,
            skill_id: 1,
        };
        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => learn.level = reader.read_int()?,
                0x02 => learn.skill_id = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(learn)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if self.level != 1 {
            writer.write_int_chunk(0x01, self.level)?;
        }
        if self.skill_id != 1 {
            writer.write_int_chunk(0x02, self.skill_id)?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Actor {
    pub id: i32,
    pub name: DBString,
    pub title: DBString,
    pub character_name: DBString,
    pub character_index: i32,
    pub transparent: bool,
    pub initial_level: i32,
    pub final_level: i32,
    pub critical_hit: bool,
    pub critical_hit_chance: i32,
    pub face_name: DBString,
    pub face_index: i32,
    pub two_weapon: bool,
    pub lock_equipment: bool,
    pub auto_battle: bool,
    pub super_guard: bool,
    pub parameters: Parameters,
    pub exp_base: i32,
    pub exp_inflation: i32,
    pub exp_correction: i32,
    pub initial_equipment: Equipment,
    pub unarmed_animation: i32,
    pub class_id: i32,
    pub battle_x: i32,
    pub battle_y: i32,
    pub battler_animation: i32,
    pub skills: Vec<Learning>,
    pub rename_skill: bool,
    pub skill_name: DBString,
    pub state_ranks: Vec<u8>,
    pub attribute_ranks: Vec<u8>,
    pub battle_commands: Vec<i32>,
}

impl Actor {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut actor = Actor {
            id: 0,
            name: DBString::default(),
            title: DBString::default(),
            character_name: DBString::default(),
            character_index: 0,
            transparent: false,
            initial_level: 1,
            final_level: 50,
            critical_hit: true,
            critical_hit_chance: 30,
            face_name: DBString::default(),
            face_index: 0,
            two_weapon: false,
            lock_equipment: false,
            auto_battle: false,
            super_guard: false,
            parameters: Parameters::default(),
            exp_base: 30,
            exp_inflation: 30,
            exp_correction: 0,
            initial_equipment: Equipment::default(),
            unarmed_animation: 1,
            class_id: 0,
            battle_x: 220,
            battle_y: 120,
            battler_animation: 1,
            skills: Vec::new(),
            rename_skill: false,
            skill_name: DBString::default(),
            state_ranks: Vec::new(),
            attribute_ranks: Vec::new(),
            battle_commands: Vec::new(),
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => actor.name = reader.read_string(chunk.length)?,
                0x02 => actor.title = reader.read_string(chunk.length)?,
                0x03 => actor.character_name = reader.read_string(chunk.length)?,
                0x04 => actor.character_index = reader.read_int()?,
                0x05 => actor.transparent = reader.read_bool()?,
                0x07 => actor.initial_level = reader.read_int()?,
                0x08 => actor.final_level = reader.read_int()?,
                0x09 => actor.critical_hit = reader.read_bool()?,
                0x0A => actor.critical_hit_chance = reader.read_int()?,
                0x0F => actor.face_name = reader.read_string(chunk.length)?,
                0x10 => actor.face_index = reader.read_int()?,
                0x15 => actor.two_weapon = reader.read_bool()?,
                0x16 => actor.lock_equipment = reader.read_bool()?,
                0x17 => actor.auto_battle = reader.read_bool()?,
                0x18 => actor.super_guard = reader.read_bool()?,
                0x1F => actor.parameters = Parameters::read_lcf(reader, chunk.length)?,
                0x29 => actor.exp_base = reader.read_int()?,
                0x2A => actor.exp_inflation = reader.read_int()?,
                0x2B => actor.exp_correction = reader.read_int()?,
                0x33 => actor.initial_equipment = Equipment::read_lcf(reader, chunk.length)?,
                0x38 => actor.unarmed_animation = reader.read_int()?,
                0x39 => actor.class_id = reader.read_int()?,
                0x3B => actor.battle_x = reader.read_int()?,
                0x3C => actor.battle_y = reader.read_int()?,
                0x3E => actor.battler_animation = reader.read_int()?,
                0x3F => {
                    let count = reader.read_int()?;
                    actor.skills.clear();
                    for _ in 0..count {
                        let l_id = reader.read_int()?;
                        let mut l = Learning::read_lcf(reader)?;
                        l.id = l_id;
                        actor.skills.push(l);
                    }
                }
                0x42 => actor.rename_skill = reader.read_bool()?,
                0x43 => actor.skill_name = reader.read_string(chunk.length)?,
                0x47 => {
                    let _ = reader.read_int()?;
                }
                0x48 => actor.state_ranks = reader.read_vector_u8(chunk.length)?,
                0x49 => {
                    let _ = reader.read_int()?;
                }
                0x4A => actor.attribute_ranks = reader.read_vector_u8(chunk.length)?,
                0x50 => {
                    let items = chunk.length / 4;
                    actor.battle_commands.clear();
                    for _ in 0..items {
                        actor.battle_commands.push(reader.read_i32_le()?);
                    }
                }
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(actor)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if !self.title.is_empty() {
            writer.write_dbstring_chunk(0x02, &self.title)?;
        }
        if !self.character_name.is_empty() {
            writer.write_dbstring_chunk(0x03, &self.character_name)?;
        }
        if self.character_index != 0 {
            writer.write_int_chunk(0x04, self.character_index)?;
        }
        if self.transparent {
            writer.write_bool_chunk(0x05, self.transparent)?;
        }
        if self.initial_level != 1 {
            writer.write_int_chunk(0x07, self.initial_level)?;
        }
        let def_final = if writer.is_2k3() { 99 } else { 50 };
        if self.final_level != def_final {
            writer.write_int_chunk(0x08, self.final_level)?;
        }
        if !self.critical_hit {
            writer.write_bool_chunk(0x09, self.critical_hit)?;
        }
        if self.critical_hit_chance != 30 {
            writer.write_int_chunk(0x0A, self.critical_hit_chance)?;
        }
        if !self.face_name.is_empty() {
            writer.write_dbstring_chunk(0x0F, &self.face_name)?;
        }
        if self.face_index != 0 {
            writer.write_int_chunk(0x10, self.face_index)?;
        }
        if self.two_weapon {
            writer.write_bool_chunk(0x15, self.two_weapon)?;
        }
        if self.lock_equipment {
            writer.write_bool_chunk(0x16, self.lock_equipment)?;
        }
        if self.auto_battle {
            writer.write_bool_chunk(0x17, self.auto_battle)?;
        }
        if self.super_guard {
            writer.write_bool_chunk(0x18, self.super_guard)?;
        }
        // 0x1F: parameters (PersistIfDefault = 1)
        writer.write_sub_chunk(0x1F, |w| self.parameters.write_lcf(w))?;
        let def_exp = if writer.is_2k3() { 300 } else { 30 };
        if self.exp_base != def_exp {
            writer.write_int_chunk(0x29, self.exp_base)?;
        }
        if self.exp_inflation != def_exp {
            writer.write_int_chunk(0x2A, self.exp_inflation)?;
        }
        if self.exp_correction != 0 {
            writer.write_int_chunk(0x2B, self.exp_correction)?;
        }
        // 0x33: initial_equipment (PersistIfDefault = 1)
        writer.write_sub_chunk(0x33, |w| self.initial_equipment.write_lcf(w))?;
        if self.unarmed_animation != 1 {
            writer.write_int_chunk(0x38, self.unarmed_animation)?;
        }
        if writer.is_2k3() {
            if self.class_id != 0 {
                writer.write_int_chunk(0x39, self.class_id)?;
            }
            if self.battle_x != 220 {
                writer.write_int_chunk(0x3B, self.battle_x)?;
            }
            if self.battle_y != 120 {
                writer.write_int_chunk(0x3C, self.battle_y)?;
            }
            if self.battler_animation != 1 {
                writer.write_int_chunk(0x3E, self.battler_animation)?;
            }
        }
        // 0x3F: skills (PersistIfDefault = 1)
        writer.write_sub_chunk(0x3F, |w| {
            w.write_int(self.skills.len() as i32)?;
            for s in &self.skills {
                w.write_int(s.id)?;
                s.write_lcf(w)?;
            }
            Ok(())
        })?;
        if self.rename_skill {
            writer.write_bool_chunk(0x42, self.rename_skill)?;
        }
        if !self.skill_name.is_empty() {
            writer.write_dbstring_chunk(0x43, &self.skill_name)?;
        }
        // 0x48: state_ranks (PersistIfDefault = 1)
        writer.write_vector_u8_chunk(0x48, &self.state_ranks)?;
        // 0x4A: attribute_ranks (PersistIfDefault = 1)
        writer.write_vector_u8_chunk(0x4A, &self.attribute_ranks)?;
        if writer.is_2k3() {
            // 0x50: battle_commands (PersistIfDefault = 1)
            writer.write_sub_chunk(0x50, |w| {
                for &cmd in &self.battle_commands {
                    w.write_i32_le(cmd)?;
                }
                Ok(())
            })?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Class {
    pub id: i32,
    pub name: DBString,
    pub two_weapon: bool,
    pub lock_equipment: bool,
    pub auto_battle: bool,
    pub super_guard: bool,
    pub parameters: Parameters,
    pub exp_base: i32,
    pub exp_inflation: i32,
    pub exp_correction: i32,
    pub battler_animation: i32,
    pub skills: Vec<Learning>,
    pub state_ranks: Vec<u8>,
    pub attribute_ranks: Vec<u8>,
    pub battle_commands: Vec<i32>,
}

impl Class {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut cl = Class {
            id: 0,
            name: DBString::default(),
            two_weapon: false,
            lock_equipment: false,
            auto_battle: false,
            super_guard: false,
            parameters: Parameters::default(),
            exp_base: 300,
            exp_inflation: 300,
            exp_correction: 0,
            battler_animation: 1,
            skills: Vec::new(),
            state_ranks: Vec::new(),
            attribute_ranks: Vec::new(),
            battle_commands: Vec::new(),
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => cl.name = reader.read_string(chunk.length)?,
                0x15 => cl.two_weapon = reader.read_bool()?,
                0x16 => cl.lock_equipment = reader.read_bool()?,
                0x17 => cl.auto_battle = reader.read_bool()?,
                0x18 => cl.super_guard = reader.read_bool()?,
                0x1F => cl.parameters = Parameters::read_lcf(reader, chunk.length)?,
                0x29 => cl.exp_base = reader.read_int()?,
                0x2A => cl.exp_inflation = reader.read_int()?,
                0x2B => cl.exp_correction = reader.read_int()?,
                0x3E => cl.battler_animation = reader.read_int()?,
                0x3F => {
                    let count = reader.read_int()?;
                    cl.skills.clear();
                    for _ in 0..count {
                        let l_id = reader.read_int()?;
                        let mut l = Learning::read_lcf(reader)?;
                        l.id = l_id;
                        cl.skills.push(l);
                    }
                }
                0x47 => {
                    let _ = reader.read_int()?;
                }
                0x48 => cl.state_ranks = reader.read_vector_u8(chunk.length)?,
                0x49 => {
                    let _ = reader.read_int()?;
                }
                0x4A => cl.attribute_ranks = reader.read_vector_u8(chunk.length)?,
                0x50 => {
                    let items = chunk.length / 4;
                    cl.battle_commands.clear();
                    for _ in 0..items {
                        cl.battle_commands.push(reader.read_i32_le()?);
                    }
                }
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(cl)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if self.two_weapon {
            writer.write_bool_chunk(0x15, self.two_weapon)?;
        }
        if self.lock_equipment {
            writer.write_bool_chunk(0x16, self.lock_equipment)?;
        }
        if self.auto_battle {
            writer.write_bool_chunk(0x17, self.auto_battle)?;
        }
        if self.super_guard {
            writer.write_bool_chunk(0x18, self.super_guard)?;
        }
        // 0x1F: parameters (PersistIfDefault = 1)
        writer.write_sub_chunk(0x1F, |w| self.parameters.write_lcf(w))?;
        if self.exp_base != 300 {
            writer.write_int_chunk(0x29, self.exp_base)?;
        }
        if self.exp_inflation != 300 {
            writer.write_int_chunk(0x2A, self.exp_inflation)?;
        }
        if self.exp_correction != 0 {
            writer.write_int_chunk(0x2B, self.exp_correction)?;
        }
        if self.battler_animation != 1 {
            writer.write_int_chunk(0x3E, self.battler_animation)?;
        }
        // 0x3F: skills (PersistIfDefault = 1)
        writer.write_sub_chunk(0x3F, |w| {
            w.write_int(self.skills.len() as i32)?;
            for s in &self.skills {
                w.write_int(s.id)?;
                s.write_lcf(w)?;
            }
            Ok(())
        })?;
        // 0x48: state_ranks (PersistIfDefault = 1)
        writer.write_vector_u8_chunk(0x48, &self.state_ranks)?;
        // 0x4A: attribute_ranks (PersistIfDefault = 1)
        writer.write_vector_u8_chunk(0x4A, &self.attribute_ranks)?;
        // 0x50: battle_commands (PersistIfDefault = 1)
        writer.write_sub_chunk(0x50, |w| {
            for &cmd in &self.battle_commands {
                w.write_i32_le(cmd)?;
            }
            Ok(())
        })?;
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Chipset {
    pub id: i32,
    pub name: DBString,
    pub chipset_name: DBString,
    pub terrain_data: Vec<i16>,
    pub passable_data_lower: Vec<u8>,
    pub passable_data_upper: Vec<u8>,
    pub animation_type: i32,
    pub animation_speed: i32,
}

impl Chipset {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut cs = Chipset {
            id: 0,
            name: DBString::default(),
            chipset_name: DBString::default(),
            terrain_data: vec![1; 162],
            passable_data_lower: vec![15; 162],
            passable_data_upper: {
                let mut v = vec![15; 144];
                v[0] = 31;
                v
            },
            animation_type: 0,
            animation_speed: 0,
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => cs.name = reader.read_string(chunk.length)?,
                0x02 => cs.chipset_name = reader.read_string(chunk.length)?,
                0x03 => cs.terrain_data = reader.read_vector_i16(chunk.length)?,
                0x04 => cs.passable_data_lower = reader.read_vector_u8(chunk.length)?,
                0x05 => cs.passable_data_upper = reader.read_vector_u8(chunk.length)?,
                0x0B => cs.animation_type = reader.read_int()?,
                0x0C => cs.animation_speed = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(cs)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if !self.chipset_name.is_empty() {
            writer.write_dbstring_chunk(0x02, &self.chipset_name)?;
        }
        let def_terrain = vec![1i16; 162];
        if self.terrain_data != def_terrain {
            writer.write_vector_i16_chunk(0x03, &self.terrain_data)?;
        }
        let def_lower = vec![15u8; 162];
        if self.passable_data_lower != def_lower {
            writer.write_vector_u8_chunk(0x04, &self.passable_data_lower)?;
        }
        let def_upper = {
            let mut v = vec![15u8; 144];
            v[0] = 31;
            v
        };
        if self.passable_data_upper != def_upper {
            writer.write_vector_u8_chunk(0x05, &self.passable_data_upper)?;
        }
        if self.animation_type != 0 {
            writer.write_int_chunk(0x0B, self.animation_type)?;
        }
        if self.animation_speed != 0 {
            writer.write_int_chunk(0x0C, self.animation_speed)?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Database {
    pub header: String,
    pub actors: Vec<Actor>,
    pub chipsets: Vec<Chipset>,
    pub classes: Vec<Class>,
    pub database_version: i32,
    pub raw_chunks: Vec<(u32, Vec<u8>)>,
}

impl Database {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut db = Database::default();

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x0B => {
                    let count = reader.read_int()?;
                    db.actors.clear();
                    for _ in 0..count {
                        let id = reader.read_int()?;
                        let mut actor = Actor::read_lcf(reader)?;
                        actor.id = id;
                        db.actors.push(actor);
                    }
                }
                0x14 => {
                    let count = reader.read_int()?;
                    db.chipsets.clear();
                    for _ in 0..count {
                        let id = reader.read_int()?;
                        let mut cs = Chipset::read_lcf(reader)?;
                        cs.id = id;
                        db.chipsets.push(cs);
                    }
                }
                0x1A => db.database_version = reader.read_int()?,
                0x1F => {
                    let count = reader.read_int()?;
                    db.classes.clear();
                    for _ in 0..count {
                        let id = reader.read_int()?;
                        let mut cl = Class::read_lcf(reader)?;
                        cl.id = id;
                        db.classes.push(cl);
                    }
                }
                _ => {
                    let bytes = reader.read_bytes(chunk.length)?;
                    db.raw_chunks.push((chunk.id, bytes));
                }
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(db)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        // 0x0B: actors (PersistIfDefault = 1)
        writer.write_sub_chunk(0x0B, |w| {
            w.write_int(self.actors.len() as i32)?;
            for a in &self.actors {
                w.write_int(a.id)?;
                a.write_lcf(w)?;
            }
            Ok(())
        })?;

        // 0x14: chipsets (PersistIfDefault = 1)
        writer.write_sub_chunk(0x14, |w| {
            w.write_int(self.chipsets.len() as i32)?;
            for cs in &self.chipsets {
                w.write_int(cs.id)?;
                cs.write_lcf(w)?;
            }
            Ok(())
        })?;

        if self.database_version != 0 {
            writer.write_int_chunk(0x1A, self.database_version)?;
        }

        if !self.classes.is_empty() {
            writer.write_sub_chunk(0x1F, |w| {
                w.write_int(self.classes.len() as i32)?;
                for cl in &self.classes {
                    w.write_int(cl.id)?;
                    cl.write_lcf(w)?;
                }
                Ok(())
            })?;
        }

        // Write any raw chunks that were preserved
        for (id, payload) in &self.raw_chunks {
            if *id != 0x0B && *id != 0x14 && *id != 0x1A && *id != 0x1F {
                writer.write_chunk(*id, payload)?;
            }
        }

        // Note: rpg::Database does NOT write trailing 0x00 chunk at root level!
        Ok(())
    }
}
