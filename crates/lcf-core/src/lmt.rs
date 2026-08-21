use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use crate::error::LcfError;
use crate::reader::LcfReader;
use crate::types::{EngineVersion, TreeMap};
use crate::writer::LcfWriter;
use crate::xml::{XmlReader, XmlWriter};

pub const LMT_HEADER: &str = "LcfMapTree";

pub struct LmtReader;

impl LmtReader {
    pub fn load<P: AsRef<Path>>(path: P, encoding: &str) -> Result<TreeMap, LcfError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::load_from_reader(&mut reader, encoding)
    }

    pub fn load_from_reader<R: Read + Seek>(stream: &mut R, encoding: &str) -> Result<TreeMap, LcfError> {
        let mut reader = LcfReader::new(stream, encoding);
        let header_len = reader.read_int()? as usize;
        let header = reader.read_raw_string(header_len)?;
        if header != LMT_HEADER {
            return Err(LcfError::InvalidHeader {
                expected: LMT_HEADER,
                found: header,
            });
        }
        let tmap = TreeMap::read_lcf(&mut reader)?;
        Ok(tmap)
    }

    pub fn save<P: AsRef<Path>>(path: P, tmap: &TreeMap, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_to_writer(&mut writer, tmap, engine, encoding)
    }

    pub fn save_to_writer<W: Write + Seek>(stream: &mut W, tmap: &TreeMap, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let mut writer = LcfWriter::new(stream, engine, encoding);
        writer.write_int(LMT_HEADER.len() as i32)?;
        writer.write_bytes(LMT_HEADER.as_bytes())?;
        tmap.write_lcf(&mut writer)?;
        Ok(())
    }

    pub fn save_xml<P: AsRef<Path>>(path: P, tmap: &TreeMap, engine: EngineVersion) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_xml_to_writer(&mut writer, tmap, engine)
    }

    pub fn save_xml_to_writer<W: Write>(stream: &mut W, tmap: &TreeMap, engine: EngineVersion) -> Result<(), LcfError> {
        let mut writer = XmlWriter::new(stream, engine)?;
        writer.begin_element("LMT")?;
        tmap.write_xml(&mut writer)?;
        writer.end_element("LMT")?;
        Ok(())
    }

    pub fn load_xml<P: AsRef<Path>>(path: P) -> Result<TreeMap, LcfError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::load_xml_from_reader(reader)
    }

    pub fn load_xml_from_reader<R: BufRead>(stream: R) -> Result<TreeMap, LcfError> {
        let mut reader = XmlReader::new(stream);
        reader.expect_root()?; // <LMT>
        let tmap = TreeMap::read_xml(&mut reader)?;
        reader.consume_wrapper_end()?; // </LMT>
        Ok(tmap)
    }
}
