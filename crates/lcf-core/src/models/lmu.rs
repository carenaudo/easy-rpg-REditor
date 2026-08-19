use std::io::{Read, Seek, Write};
use crate::error::LcfError;
use crate::reader::LcfReader;
use crate::types::DBString;
use crate::writer::LcfWriter;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MoveCommand {
    pub command_id: i32,
    pub parameter_a: i32,
    pub parameter_b: i32,
    pub parameter_c: i32,
    pub parameter_string: DBString,
}

impl MoveCommand {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut cmd = MoveCommand::default();
        cmd.command_id = reader.read_int()?;
        match cmd.command_id {
            // switch_on (32 / 0x20), switch_off (33 / 0x21)
            32 | 33 => {
                cmd.parameter_a = reader.read_int()?;
            }
            // change_graphic (34 / 0x22)
            34 => {
                let len = reader.read_int()? as usize;
                cmd.parameter_string = reader.read_string(len)?;
                cmd.parameter_a = reader.read_int()?;
            }
            // play_sound_effect (35 / 0x23)
            35 => {
                let len = reader.read_int()? as usize;
                cmd.parameter_string = reader.read_string(len)?;
                cmd.parameter_a = reader.read_int()?;
                cmd.parameter_b = reader.read_int()?;
                cmd.parameter_c = reader.read_int()?;
            }
            _ => {}
        }
        Ok(cmd)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.command_id)?;
        match self.command_id {
            32 | 33 => {
                writer.write_int(self.parameter_a)?;
            }
            34 => {
                let bytes = writer.encoder().encode(&self.parameter_string.0);
                writer.write_int(bytes.len() as i32)?;
                writer.write_bytes(&bytes)?;
                writer.write_int(self.parameter_a)?;
            }
            35 => {
                let bytes = writer.encoder().encode(&self.parameter_string.0);
                writer.write_int(bytes.len() as i32)?;
                writer.write_bytes(&bytes)?;
                writer.write_int(self.parameter_a)?;
                writer.write_int(self.parameter_b)?;
                writer.write_int(self.parameter_c)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MoveRoute {
    pub move_commands: Vec<MoveCommand>,
    pub repeat: bool,
    pub skippable: bool,
}

impl MoveRoute {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut route = MoveRoute {
            move_commands: Vec::new(),
            repeat: true,
            skippable: false,
        };
        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x0B => {
                    let _ = reader.read_int()?;
                }
                0x0C => {
                    let end_pos = start_off + chunk.length as u64;
                    route.move_commands.clear();
                    while reader.tell()? < end_pos {
                        route.move_commands.push(MoveCommand::read_lcf(reader)?);
                    }
                }
                0x15 => route.repeat = reader.read_bool()?,
                0x16 => route.skippable = reader.read_bool()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(route)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        // 0x0B: size field
        writer.write_int_chunk(0x0B, self.move_commands.len() as i32)?;
        // 0x0C: commands payload (PersistIfDefault = 1)
        writer.write_sub_chunk(0x0C, |w| {
            for cmd in &self.move_commands {
                cmd.write_lcf(w)?;
            }
            Ok(())
        })?;
        if !self.repeat {
            writer.write_bool_chunk(0x15, self.repeat)?;
        }
        if self.skippable {
            writer.write_bool_chunk(0x16, self.skippable)?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventCommand {
    pub code: i32,
    pub indent: i32,
    pub string: DBString,
    pub parameters: Vec<i32>,
}

impl EventCommand {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut cmd = EventCommand::default();
        cmd.code = reader.read_int()?;
        if cmd.code != 0 {
            cmd.indent = reader.read_int()?;
            let str_len = reader.read_int()? as usize;
            cmd.string = reader.read_string(str_len)?;
            let param_count = reader.read_int()?;
            cmd.parameters = Vec::with_capacity(param_count.max(0) as usize);
            for _ in 0..param_count {
                cmd.parameters.push(reader.read_int()?);
            }
        }
        Ok(cmd)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        writer.write_int(self.code)?;
        writer.write_int(self.indent)?;
        let bytes = writer.encoder().encode(&self.string.0);
        writer.write_int(bytes.len() as i32)?;
        writer.write_bytes(&bytes)?;
        writer.write_int(self.parameters.len() as i32)?;
        for &p in &self.parameters {
            writer.write_int(p)?;
        }
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventPageCondition {
    pub flags: i32,
    pub switch_a_id: i32,
    pub switch_b_id: i32,
    pub variable_id: i32,
    pub variable_value: i32,
    pub item_id: i32,
    pub actor_id: i32,
    pub timer_sec: i32,
    pub timer2_sec: i32,
    pub compare_operator: i32,
}

impl EventPageCondition {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut cond = EventPageCondition {
            flags: 0,
            switch_a_id: 1,
            switch_b_id: 1,
            variable_id: 1,
            variable_value: 0,
            item_id: 1,
            actor_id: 1,
            timer_sec: 0,
            timer2_sec: 0,
            compare_operator: 1,
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => cond.flags = reader.read_int()?,
                0x02 => cond.switch_a_id = reader.read_int()?,
                0x03 => cond.switch_b_id = reader.read_int()?,
                0x04 => cond.variable_id = reader.read_int()?,
                0x05 => cond.variable_value = reader.read_int()?,
                0x06 => cond.item_id = reader.read_int()?,
                0x07 => cond.actor_id = reader.read_int()?,
                0x08 => cond.timer_sec = reader.read_int()?,
                0x09 => cond.timer2_sec = reader.read_int()?,
                0x0A => cond.compare_operator = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(cond)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        // flags: PersistIfDefault = 1
        writer.write_int_chunk(0x01, self.flags)?;
        if self.switch_a_id != 1 {
            writer.write_int_chunk(0x02, self.switch_a_id)?;
        }
        if self.switch_b_id != 1 {
            writer.write_int_chunk(0x03, self.switch_b_id)?;
        }
        if self.variable_id != 1 {
            writer.write_int_chunk(0x04, self.variable_id)?;
        }
        if self.variable_value != 0 {
            writer.write_int_chunk(0x05, self.variable_value)?;
        }
        if self.item_id != 1 {
            writer.write_int_chunk(0x06, self.item_id)?;
        }
        // actor_id: PersistIfDefault = 1
        writer.write_int_chunk(0x07, self.actor_id)?;
        if self.timer_sec != 0 {
            writer.write_int_chunk(0x08, self.timer_sec)?;
        }
        if writer.is_2k3() {
            if self.timer2_sec != 0 {
                writer.write_int_chunk(0x09, self.timer2_sec)?;
            }
            if self.compare_operator != 1 {
                writer.write_int_chunk(0x0A, self.compare_operator)?;
            }
        }
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventPage {
    pub id: i32,
    pub condition: EventPageCondition,
    pub character_name: DBString,
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
    pub move_route: MoveRoute,
    pub event_commands: Vec<EventCommand>,
}

impl EventPage {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut page = EventPage {
            id: 0,
            condition: EventPageCondition::default(),
            character_name: DBString::default(),
            character_index: 0,
            character_direction: 2,
            character_pattern: 1,
            translucent: false,
            move_type: 1,
            move_frequency: 3,
            trigger: 0,
            layer: 0,
            overlap_forbidden: false,
            animation_type: 0,
            move_speed: 3,
            move_route: MoveRoute::default(),
            event_commands: Vec::new(),
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x02 => page.condition = EventPageCondition::read_lcf(reader)?,
                0x15 => page.character_name = reader.read_string(chunk.length)?,
                0x16 => page.character_index = reader.read_int()?,
                0x17 => page.character_direction = reader.read_int()?,
                0x18 => page.character_pattern = reader.read_int()?,
                0x19 => page.translucent = reader.read_bool()?,
                0x1F => page.move_type = reader.read_int()?,
                0x20 => page.move_frequency = reader.read_int()?,
                0x21 => page.trigger = reader.read_int()?,
                0x22 => page.layer = reader.read_int()?,
                0x23 => page.overlap_forbidden = reader.read_bool()?,
                0x24 => page.animation_type = reader.read_int()?,
                0x25 => page.move_speed = reader.read_int()?,
                0x29 => page.move_route = MoveRoute::read_lcf(reader)?,
                0x33 => {
                    let _ = reader.read_int()?; // size field
                }
                0x34 => {
                    let end_pos = start_off + chunk.length as u64;
                    page.event_commands.clear();
                    loop {
                        if reader.tell()? >= end_pos {
                            break;
                        }
                        let cmd = EventCommand::read_lcf(reader)?;
                        if cmd.code == 0 {
                            for _ in 0..3 {
                                if reader.tell()? < end_pos {
                                    let _ = reader.read_u8()?;
                                }
                            }
                            break;
                        }
                        page.event_commands.push(cmd);
                    }
                }
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(page)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        // 0x02: condition (PersistIfDefault = 1)
        writer.write_sub_chunk(0x02, |w| self.condition.write_lcf(w))?;
        if !self.character_name.is_empty() {
            writer.write_dbstring_chunk(0x15, &self.character_name)?;
        }
        if self.character_index != 0 {
            writer.write_int_chunk(0x16, self.character_index)?;
        }
        // 0x17: character_direction (PersistIfDefault = 1)
        writer.write_int_chunk(0x17, self.character_direction)?;
        if self.character_pattern != 1 {
            writer.write_int_chunk(0x18, self.character_pattern)?;
        }
        // 0x19: translucent (PersistIfDefault = 1)
        writer.write_bool_chunk(0x19, self.translucent)?;
        // 0x1F: move_type (PersistIfDefault = 1)
        writer.write_int_chunk(0x1F, self.move_type)?;
        if self.move_frequency != 3 {
            writer.write_int_chunk(0x20, self.move_frequency)?;
        }
        // 0x21: trigger (PersistIfDefault = 1)
        writer.write_int_chunk(0x21, self.trigger)?;
        // 0x22: layer (PersistIfDefault = 1)
        writer.write_int_chunk(0x22, self.layer)?;
        // 0x23: overlap_forbidden (PersistIfDefault = 1)
        writer.write_bool_chunk(0x23, self.overlap_forbidden)?;
        // 0x24: animation_type (PersistIfDefault = 1)
        writer.write_int_chunk(0x24, self.animation_type)?;
        if self.move_speed != 3 {
            writer.write_int_chunk(0x25, self.move_speed)?;
        }
        // 0x29: move_route (PersistIfDefault = 1)
        writer.write_sub_chunk(0x29, |w| self.move_route.write_lcf(w))?;
        // 0x33: event_commands size (PersistIfDefault = 1)
        writer.write_int_chunk(0x33, self.event_commands.len() as i32)?;
        // 0x34: event_commands array (PersistIfDefault = 1)
        writer.write_sub_chunk(0x34, |w| {
            for cmd in &self.event_commands {
                cmd.write_lcf(w)?;
            }
            for _ in 0..4 {
                w.write_int(0)?;
            }
            Ok(())
        })?;
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Event {
    pub id: i32,
    pub name: DBString,
    pub x: i32,
    pub y: i32,
    pub pages: Vec<EventPage>,
}

impl Event {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut event = Event::default();
        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => event.name = reader.read_string(chunk.length)?,
                0x02 => event.x = reader.read_int()?,
                0x03 => event.y = reader.read_int()?,
                0x05 => {
                    let count = reader.read_int()?;
                    event.pages.clear();
                    for _ in 0..count {
                        let page_id = reader.read_int()?;
                        let mut page = EventPage::read_lcf(reader)?;
                        page.id = page_id;
                        event.pages.push(page);
                    }
                }
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(event)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if !self.name.is_empty() {
            writer.write_dbstring_chunk(0x01, &self.name)?;
        }
        if self.x != 0 {
            writer.write_int_chunk(0x02, self.x)?;
        }
        if self.y != 0 {
            writer.write_int_chunk(0x03, self.y)?;
        }
        // 0x05: pages (PersistIfDefault = 1)
        writer.write_sub_chunk(0x05, |w| {
            w.write_int(self.pages.len() as i32)?;
            for page in &self.pages {
                w.write_int(page.id)?;
                page.write_lcf(w)?;
            }
            Ok(())
        })?;
        writer.write_int(0)?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Map {
    pub header: String,
    pub chipset_id: i32,
    pub width: i32,
    pub height: i32,
    pub scroll_type: i32,
    pub parallax_flag: bool,
    pub parallax_name: DBString,
    pub parallax_loop_x: bool,
    pub parallax_loop_y: bool,
    pub parallax_auto_loop_x: bool,
    pub parallax_sx: i32,
    pub parallax_auto_loop_y: bool,
    pub parallax_sy: i32,
    pub generator_flag: bool,
    pub generator_mode: i32,
    pub top_level: bool,
    pub generator_tiles: i32,
    pub generator_width: i32,
    pub generator_height: i32,
    pub generator_surround: bool,
    pub generator_upper_wall: bool,
    pub generator_floor_b: bool,
    pub generator_floor_c: bool,
    pub generator_extra_b: bool,
    pub generator_extra_c: bool,
    pub generator_x: Vec<u32>,
    pub generator_y: Vec<u32>,
    pub generator_tile_ids: Vec<i16>,
    pub lower_layer: Vec<i16>,
    pub upper_layer: Vec<i16>,
    pub events: Vec<Event>,
    pub save_count_2k3e: i32,
    pub save_count: i32,
}

impl Map {
    pub fn read_lcf<R: Read + Seek>(reader: &mut LcfReader<R>) -> Result<Self, LcfError> {
        let mut map = Map {
            header: String::new(),
            chipset_id: 1,
            width: 20,
            height: 15,
            scroll_type: 0,
            parallax_flag: false,
            parallax_name: DBString::default(),
            parallax_loop_x: false,
            parallax_loop_y: false,
            parallax_auto_loop_x: false,
            parallax_sx: 0,
            parallax_auto_loop_y: false,
            parallax_sy: 0,
            generator_flag: false,
            generator_mode: 0,
            top_level: false,
            generator_tiles: 0,
            generator_width: 4,
            generator_height: 1,
            generator_surround: true,
            generator_upper_wall: true,
            generator_floor_b: true,
            generator_floor_c: true,
            generator_extra_b: true,
            generator_extra_c: true,
            generator_x: Vec::new(),
            generator_y: Vec::new(),
            generator_tile_ids: Vec::new(),
            lower_layer: Vec::new(),
            upper_layer: Vec::new(),
            events: Vec::new(),
            save_count_2k3e: 0,
            save_count: 0,
        };

        while let Some(chunk) = reader.read_chunk_header()? {
            let start_off = reader.tell()?;
            match chunk.id {
                0x01 => map.chipset_id = reader.read_int()?,
                0x02 => map.width = reader.read_int()?,
                0x03 => map.height = reader.read_int()?,
                0x0B => map.scroll_type = reader.read_int()?,
                0x1F => map.parallax_flag = reader.read_bool()?,
                0x20 => map.parallax_name = reader.read_string(chunk.length)?,
                0x21 => map.parallax_loop_x = reader.read_bool()?,
                0x22 => map.parallax_loop_y = reader.read_bool()?,
                0x23 => map.parallax_auto_loop_x = reader.read_bool()?,
                0x24 => map.parallax_sx = reader.read_int()?,
                0x25 => map.parallax_auto_loop_y = reader.read_bool()?,
                0x26 => map.parallax_sy = reader.read_int()?,
                0x28 => map.generator_flag = reader.read_bool()?,
                0x29 => map.generator_mode = reader.read_int()?,
                0x2A => map.top_level = reader.read_bool()?,
                0x30 => map.generator_tiles = reader.read_int()?,
                0x31 => map.generator_width = reader.read_int()?,
                0x32 => map.generator_height = reader.read_int()?,
                0x33 => map.generator_surround = reader.read_bool()?,
                0x34 => map.generator_upper_wall = reader.read_bool()?,
                0x35 => map.generator_floor_b = reader.read_bool()?,
                0x36 => map.generator_floor_c = reader.read_bool()?,
                0x37 => map.generator_extra_b = reader.read_bool()?,
                0x38 => map.generator_extra_c = reader.read_bool()?,
                0x3C => {
                    let items = chunk.length / 4;
                    map.generator_x = Vec::with_capacity(items);
                    for _ in 0..items {
                        map.generator_x.push(reader.read_u32_le()?);
                    }
                }
                0x3D => {
                    let items = chunk.length / 4;
                    map.generator_y = Vec::with_capacity(items);
                    for _ in 0..items {
                        map.generator_y.push(reader.read_u32_le()?);
                    }
                }
                0x3E => map.generator_tile_ids = reader.read_vector_i16(chunk.length)?,
                0x47 => map.lower_layer = reader.read_vector_i16(chunk.length)?,
                0x48 => map.upper_layer = reader.read_vector_i16(chunk.length)?,
                0x51 => {
                    let count = reader.read_int()?;
                    map.events.clear();
                    for _ in 0..count {
                        let ev_id = reader.read_int()?;
                        let mut ev = Event::read_lcf(reader)?;
                        ev.id = ev_id;
                        map.events.push(ev);
                    }
                }
                0x5A => map.save_count_2k3e = reader.read_int()?,
                0x5B => map.save_count = reader.read_int()?,
                _ => reader.skip(chunk.length)?,
            }
            let read_bytes = (reader.tell()? - start_off) as usize;
            if read_bytes < chunk.length {
                reader.skip(chunk.length - read_bytes)?;
            }
        }
        Ok(map)
    }

    pub fn write_lcf<W: Write + Seek>(&self, writer: &mut LcfWriter<W>) -> Result<(), LcfError> {
        if self.chipset_id != 1 {
            writer.write_int_chunk(0x01, self.chipset_id)?;
        }
        if self.width != 20 {
            writer.write_int_chunk(0x02, self.width)?;
        }
        if self.height != 15 {
            writer.write_int_chunk(0x03, self.height)?;
        }
        // 0x0B: scroll_type (PersistIfDefault = 1)
        writer.write_int_chunk(0x0B, self.scroll_type)?;
        if self.parallax_flag {
            writer.write_bool_chunk(0x1F, self.parallax_flag)?;
        }
        if !self.parallax_name.is_empty() {
            writer.write_dbstring_chunk(0x20, &self.parallax_name)?;
        }
        if self.parallax_loop_x {
            writer.write_bool_chunk(0x21, self.parallax_loop_x)?;
        }
        if self.parallax_loop_y {
            writer.write_bool_chunk(0x22, self.parallax_loop_y)?;
        }
        if self.parallax_auto_loop_x {
            writer.write_bool_chunk(0x23, self.parallax_auto_loop_x)?;
        }
        if self.parallax_sx != 0 {
            writer.write_int_chunk(0x24, self.parallax_sx)?;
        }
        if self.parallax_auto_loop_y {
            writer.write_bool_chunk(0x25, self.parallax_auto_loop_y)?;
        }
        if self.parallax_sy != 0 {
            writer.write_int_chunk(0x26, self.parallax_sy)?;
        }
        if self.generator_flag {
            writer.write_bool_chunk(0x28, self.generator_flag)?;
        }
        if self.generator_mode != 0 {
            writer.write_int_chunk(0x29, self.generator_mode)?;
        }
        if self.top_level {
            writer.write_bool_chunk(0x2A, self.top_level)?;
        }
        if self.generator_tiles != 0 {
            writer.write_int_chunk(0x30, self.generator_tiles)?;
        }
        if self.generator_width != 4 {
            writer.write_int_chunk(0x31, self.generator_width)?;
        }
        if self.generator_height != 1 {
            writer.write_int_chunk(0x32, self.generator_height)?;
        }
        if !self.generator_surround {
            writer.write_bool_chunk(0x33, self.generator_surround)?;
        }
        if !self.generator_upper_wall {
            writer.write_bool_chunk(0x34, self.generator_upper_wall)?;
        }
        if !self.generator_floor_b {
            writer.write_bool_chunk(0x35, self.generator_floor_b)?;
        }
        if !self.generator_floor_c {
            writer.write_bool_chunk(0x36, self.generator_floor_c)?;
        }
        if !self.generator_extra_b {
            writer.write_bool_chunk(0x37, self.generator_extra_b)?;
        }
        if !self.generator_extra_c {
            writer.write_bool_chunk(0x38, self.generator_extra_c)?;
        }
        if !self.generator_x.is_empty() {
            writer.write_sub_chunk(0x3C, |w| {
                for &x in &self.generator_x {
                    w.write_u32_le(x)?;
                }
                Ok(())
            })?;
        }
        if !self.generator_y.is_empty() {
            writer.write_sub_chunk(0x3D, |w| {
                for &y in &self.generator_y {
                    w.write_u32_le(y)?;
                }
                Ok(())
            })?;
        }
        if !self.generator_tile_ids.is_empty() {
            writer.write_vector_i16_chunk(0x3E, &self.generator_tile_ids)?;
        }
        // 0x47: lower_layer (PersistIfDefault = 1)
        writer.write_vector_i16_chunk(0x47, &self.lower_layer)?;
        // 0x48: upper_layer (PersistIfDefault = 1)
        writer.write_vector_i16_chunk(0x48, &self.upper_layer)?;
        // 0x51: events (PersistIfDefault = 1)
        writer.write_sub_chunk(0x51, |w| {
            w.write_int(self.events.len() as i32)?;
            for ev in &self.events {
                w.write_int(ev.id)?;
                ev.write_lcf(w)?;
            }
            Ok(())
        })?;
        if writer.is_2k3() && self.save_count_2k3e != 0 {
            writer.write_int_chunk(0x5A, self.save_count_2k3e)?;
        }
        if self.save_count != 0 {
            writer.write_int_chunk(0x5B, self.save_count)?;
        }
        writer.write_int(0)?;
        Ok(())
    }
}
