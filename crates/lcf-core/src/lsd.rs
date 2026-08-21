use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use crate::error::LcfError;
use crate::generated::lsd_gen::Save;
use crate::reader::LcfReader;
use crate::reader_util::ReaderUtil;
use crate::types::EngineVersion;
use crate::writer::LcfWriter;
use crate::xml::{XmlReader, XmlWriter};

pub const LSD_HEADER: &str = "LcfSaveData";

pub struct LsdReader;

impl LsdReader {
    pub fn prepare_save(save: &mut Save, version: i32, codepage: i32) {
        save.system.save_count += 1;
        save.title.timestamp = ReaderUtil::generate_timestamp();
        save.easyrpg_data.version = version;
        save.easyrpg_data.codepage = codepage;
    }

    pub fn load<P: AsRef<Path>>(path: P, encoding: &str) -> Result<Save, LcfError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::load_from_reader(&mut reader, encoding)
    }

    pub fn load_from_reader<R: Read + Seek>(stream: &mut R, encoding: &str) -> Result<Save, LcfError> {
        let mut reader = LcfReader::new(stream, encoding);
        let header_len = reader.read_int()? as usize;
        let header = reader.read_raw_string(header_len)?;
        if header != LSD_HEADER {
            return Err(LcfError::InvalidHeader {
                expected: LSD_HEADER,
                found: header,
            });
        }
        let pos_after_header = reader.tell()?;
        let mut save = Save::read_lcf(&mut reader)?;

        // If save specifies a custom codepage in easyrpg_data, reload strings with it
        if save.easyrpg_data.codepage > 0 {
            let cp_enc = ReaderUtil::codepage_to_encoding(save.easyrpg_data.codepage);
            let s = reader.into_inner();
            s.seek(SeekFrom::Start(pos_after_header))?;
            let mut reader2 = LcfReader::new(s, cp_enc);
            save = Save::read_lcf(&mut reader2)?;
        }

        Ok(save)
    }

    pub fn save<P: AsRef<Path>>(path: P, save: &Save, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_to_writer(&mut writer, save, engine, encoding)
    }

    pub fn save_to_writer<W: Write + Seek>(stream: &mut W, save: &Save, engine: EngineVersion, encoding: &str) -> Result<(), LcfError> {
        let actual_encoding = if save.easyrpg_data.codepage > 0 {
            ReaderUtil::codepage_to_encoding(save.easyrpg_data.codepage)
        } else {
            encoding
        };
        let mut writer = LcfWriter::new(stream, engine, actual_encoding);
        writer.write_int(LSD_HEADER.len() as i32)?;
        writer.write_bytes(LSD_HEADER.as_bytes())?;
        save.write_lcf(&mut writer)?;
        Ok(())
    }

    pub fn save_xml<P: AsRef<Path>>(path: P, save: &Save, engine: EngineVersion) -> Result<(), LcfError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::save_xml_to_writer(&mut writer, save, engine)
    }

    pub fn save_xml_to_writer<W: Write>(stream: &mut W, save: &Save, engine: EngineVersion) -> Result<(), LcfError> {
        let mut writer = XmlWriter::new(stream, engine)?;
        writer.begin_element("LSD")?;
        save.write_xml(&mut writer)?;
        writer.end_element("LSD")?;
        Ok(())
    }

    pub fn load_xml<P: AsRef<Path>>(path: P) -> Result<Save, LcfError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::load_xml_from_reader(reader)
    }

    pub fn load_xml_from_reader<R: BufRead>(stream: R) -> Result<Save, LcfError> {
        let mut reader = XmlReader::new(stream);
        reader.expect_root()?; // <LSD>
        // Like LDB, engine version isn't known upfront; is_2k3 only
        // affects defaults for fields a well-formed export never omits.
        let save = Save::read_xml(&mut reader, false)?;
        reader.consume_wrapper_end()?; // </LSD>
        Ok(save)
    }
}
