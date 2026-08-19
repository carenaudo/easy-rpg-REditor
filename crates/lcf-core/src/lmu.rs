use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use crate::error::LcfError;
use crate::generated::lmu_gen::Map;
use crate::reader::LcfReader;
use crate::types::EngineVersion;
use crate::writer::LcfWriter;
use crate::xml::XmlWriter;

pub const LMU_HEADER: &str = "LcfMapUnit";

pub struct LmuReader;

impl LmuReader {
    pub fn load<P: AsRef<Path>>(path: P, encoding: &str) -> Result<Map, LcfError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::load_from_reader(&mut reader, encoding)
    }

    pub fn load_from_reader<R: Read + Seek>(stream: &mut R, encoding: &str) -> Result<Map, LcfError> {
        let mut reader = LcfReader::new(stream, encoding);
        let header_len = reader.read_int()? as usize;
        let header = reader.read_raw_string(header_len)?;
        if header != LMU_HEADER {
            return Err(LcfError::InvalidHeader {
                expected: LMU_HEADER,
                found: header,
            });
        }
        let map = Map::read_lcf(&mut reader)?;
        Ok(map)
    }

    pub fn save<P: AsRef<Path>>(path: P, map: &Map, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_to_writer(&mut writer, map, engine, encoding)
    }

    pub fn save_to_writer<W: Write + Seek>(stream: &mut W, map: &Map, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let mut writer = LcfWriter::new(stream, engine, encoding);
        writer.write_int(LMU_HEADER.len() as i32)?;
        writer.write_bytes(LMU_HEADER.as_bytes())?;
        map.write_lcf(&mut writer)?;
        Ok(())
    }

    pub fn save_xml<P: AsRef<Path>>(path: P, map: &Map, engine: EngineVersion) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_xml_to_writer(&mut writer, map, engine)
    }

    pub fn save_xml_to_writer<W: Write>(stream: &mut W, map: &Map, engine: EngineVersion) -> Result<(), LcfError> {
        let mut writer = XmlWriter::new(stream, engine)?;
        writer.begin_element("LMU")?;
        map.write_xml(&mut writer)?;
        writer.end_element("LMU")?;
        Ok(())
    }
}
