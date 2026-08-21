use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use crate::error::LcfError;
use crate::generated::ldb_gen::Database;
use crate::reader::LcfReader;
use crate::reader_util::ReaderUtil;
use crate::setup::Setup;
use crate::types::EngineVersion;
use crate::writer::LcfWriter;
use crate::xml::{XmlReader, XmlWriter};

pub const LDB_HEADER: &str = "LcfDataBase";

pub struct LdbReader;

impl LdbReader {
    pub fn prepare_save(db: &mut Database) {
        db.system.save_count += 1;
        if !db.classes.is_empty()
            || !db.battleranimations.is_empty()
            || db.actors.iter().any(|a| a.class_id > 0 || !a.battle_commands.is_empty())
        {
            db.system.ldb_id = 2003;
            if db.version < 2 {
                db.version = 2;
            }
        }
    }


    pub fn load<P: AsRef<Path>>(path: P, encoding: &str) -> Result<Database, LcfError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::load_from_reader(&mut reader, encoding)
    }

    pub fn load_from_reader<R: Read + Seek>(stream: &mut R, encoding: &str) -> Result<Database, LcfError> {
        let mut reader = LcfReader::new(stream, encoding);
        let header_len = reader.read_int()? as usize;
        let header = reader.read_raw_string(header_len)?;
        if header != LDB_HEADER {
            return Err(LcfError::InvalidHeader {
                expected: LDB_HEADER,
                found: header,
            });
        }
        let mut db = Database::read_lcf(&mut reader)?;
        let engine = ReaderUtil::get_engine_version(&db);
        for actor in &mut db.actors {
            Setup::actor(actor, engine.is_2k3());
        }
        Ok(db)
    }

    pub fn save<P: AsRef<Path>>(path: P, db: &Database, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_to_writer(&mut writer, db, engine, encoding)
    }

    pub fn save_to_writer<W: Write + Seek>(stream: &mut W, db: &Database, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let actual_engine = if engine.is_2k3() || ReaderUtil::get_engine_version(db).is_2k3() {
            EngineVersion::Engine2003
        } else {
            engine
        };
        let mut writer = LcfWriter::new(stream, actual_engine, encoding);
        writer.write_int(LDB_HEADER.len() as i32)?;
        writer.write_bytes(LDB_HEADER.as_bytes())?;
        db.write_lcf(&mut writer)?;
        Ok(())
    }

    pub fn save_xml<P: AsRef<Path>>(path: P, db: &Database, engine: EngineVersion) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_xml_to_writer(&mut writer, db, engine)
    }

    pub fn save_xml_to_writer<W: Write>(stream: &mut W, db: &Database, engine: EngineVersion) -> Result<(), LcfError> {
        let actual_engine = if engine.is_2k3() || ReaderUtil::get_engine_version(db).is_2k3() {
            EngineVersion::Engine2003
        } else {
            engine
        };
        let mut writer = XmlWriter::new(stream, actual_engine)?;
        writer.begin_element("LDB")?;
        db.write_xml(&mut writer)?;
        writer.end_element("LDB")?;
        Ok(())
    }

    pub fn load_xml<P: AsRef<Path>>(path: P) -> Result<Database, LcfError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::load_xml_from_reader(reader)
    }

    pub fn load_xml_from_reader<R: BufRead>(stream: R) -> Result<Database, LcfError> {
        let mut reader = XmlReader::new(stream);
        reader.expect_root()?; // <LDB>
        // Engine version isn't known until the parsed data is available
        // (unlike binary loading, XML carries no header flag for it), so
        // parse once with is_2k3=false for field defaults - every field a
        // well-formed export writes is overwritten regardless, and this
        // only affects defaults for fields a foreign/hand-edited XML omits.
        let mut db = Database::read_xml(&mut reader, false)?;
        reader.consume_wrapper_end()?; // </LDB>

        // Mirror LdbReader::load's post-load fixup (see `load_from_reader`
        // above): infer the real engine version now that data is present,
        // and run the same per-actor setup pass.
        let engine = ReaderUtil::get_engine_version(&db);
        for actor in &mut db.actors {
            Setup::actor(actor, engine.is_2k3());
        }
        Ok(db)
    }
}

