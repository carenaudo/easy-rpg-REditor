use std::io::{Seek, SeekFrom, Write};
use crate::encoder::LcfEncoder;
use crate::error::LcfError;
use crate::types::{DBBitArray, DBString, EngineVersion, Rect};

pub struct LcfWriter<W: Write + Seek> {
    stream: W,
    encoder: LcfEncoder,
    engine: EngineVersion,
    offset: u64,
}

impl<W: Write + Seek> LcfWriter<W> {
    pub fn new(stream: W, engine: EngineVersion, encoding: &str) -> Self {
        Self {
            stream,
            encoder: LcfEncoder::new(encoding),
            engine,
            offset: 0,
        }
    }

    pub fn engine(&self) -> EngineVersion {
        self.engine
    }

    pub fn is_2k3(&self) -> bool {
        self.engine.is_2k3()
    }

    pub fn encoder(&self) -> &LcfEncoder {
        &self.encoder
    }

    pub fn encoding_name(&self) -> &str {
        self.encoder.encoding_name()
    }

    pub fn tell(&mut self) -> Result<u64, LcfError> {
        Ok(self.offset)
    }

    pub fn seek(&mut self, pos: u64) -> Result<(), LcfError> {
        self.stream.seek(SeekFrom::Start(pos))?;
        self.offset = pos;
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), LcfError> {
        if !bytes.is_empty() {
            self.stream.write_all(bytes)?;
            self.offset += bytes.len() as u64;
        }
        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<(), LcfError> {
        self.write_bytes(&[val])
    }

    pub fn write_i8(&mut self, val: i8) -> Result<(), LcfError> {
        self.write_u8(val as u8)
    }

    pub fn write_i16(&mut self, val: i16) -> Result<(), LcfError> {
        self.write_bytes(&val.to_le_bytes())
    }

    pub fn write_u32_le(&mut self, val: u32) -> Result<(), LcfError> {
        self.write_bytes(&val.to_le_bytes())
    }

    pub fn write_i32_le(&mut self, val: i32) -> Result<(), LcfError> {
        self.write_bytes(&val.to_le_bytes())
    }

    pub fn write_f64(&mut self, val: f64) -> Result<(), LcfError> {
        self.write_bytes(&val.to_le_bytes())
    }

    /// Writes a 7-bit compressed integer (BER / varint format).
    pub fn write_int(&mut self, val: i32) -> Result<(), LcfError> {
        let value = val as u32;
        for i in (0..=28).rev().step_by(7) {
            if value >= (1u32 << i) || i == 0 {
                let b = (((value >> i) & 0x7F) | if i > 0 { 0x80 } else { 0 }) as u8;
                self.write_u8(b)?;
            }
        }
        Ok(())
    }

    /// Writes a 64-bit compressed integer.
    pub fn write_u64(&mut self, value: u64) -> Result<(), LcfError> {
        for i in (0..=56).rev().step_by(7) {
            if value >= (1u64 << i) || i == 0 {
                let b = (((value >> i) & 0x7F) | if i > 0 { 0x80 } else { 0 }) as u8;
                self.write_u8(b)?;
            }
        }
        Ok(())
    }

    pub fn write_bool(&mut self, val: bool) -> Result<(), LcfError> {
        self.write_u8(if val { 1 } else { 0 })
    }

    pub fn write_string(&mut self, text: &str) -> Result<(), LcfError> {
        let bytes = self.encoder.encode(text);
        self.write_bytes(&bytes)
    }

    pub fn write_dbstring(&mut self, text: &DBString) -> Result<(), LcfError> {
        if let Some(raw) = text.raw_bytes() {
            self.write_bytes(raw)
        } else {
            self.write_string(text.as_str())
        }
    }


    pub fn write_rect(&mut self, rect: &Rect) -> Result<(), LcfError> {
        self.write_i32_le(rect.l)?;
        self.write_i32_le(rect.t)?;
        self.write_i32_le(rect.r)?;
        self.write_i32_le(rect.b)?;
        Ok(())
    }

    pub fn write_vector_u8(&mut self, vec: &[u8]) -> Result<(), LcfError> {
        self.write_bytes(vec)
    }

    pub fn write_vector_i16(&mut self, vec: &[i16]) -> Result<(), LcfError> {
        for &val in vec {
            self.write_i16(val)?;
        }
        Ok(())
    }

    pub fn write_vector_i32(&mut self, vec: &[i32]) -> Result<(), LcfError> {
        for &val in vec {
            self.write_i32_le(val)?;
        }
        Ok(())
    }

    pub fn write_vector_bool(&mut self, vec: &[bool]) -> Result<(), LcfError> {
        for &val in vec {
            self.write_u8(if val { 1 } else { 0 })?;
        }
        Ok(())
    }

    pub fn write_bit_array(&mut self, bits: &DBBitArray) -> Result<(), LcfError> {
        self.write_vector_bool(&bits.0)
    }

    pub fn write_chunk(&mut self, id: u32, payload: &[u8]) -> Result<(), LcfError> {
        self.write_int(id as i32)?;
        self.write_int(payload.len() as i32)?;
        self.write_bytes(payload)?;
        Ok(())
    }

    pub fn write_int_chunk(&mut self, id: u32, val: i32) -> Result<(), LcfError> {
        let mut buf = Vec::new();
        {
            let mut sub_w = LcfWriter::new(std::io::Cursor::new(&mut buf), self.engine, self.encoder.encoding_name());
            sub_w.write_int(val)?;
        }
        self.write_chunk(id, &buf)
    }

    pub fn write_bool_chunk(&mut self, id: u32, val: bool) -> Result<(), LcfError> {
        self.write_int_chunk(id, if val { 1 } else { 0 })
    }

    pub fn write_flags_chunk(&mut self, id: u32, val: i32, is_2k3_2bytes: bool) -> Result<(), LcfError> {
        if is_2k3_2bytes || val > 0xFF {
            let bytes = [val as u8, (val >> 8) as u8];
            self.write_chunk(id, &bytes)
        } else {
            let bytes = [val as u8];
            self.write_chunk(id, &bytes)
        }
    }


    pub fn write_string_chunk(&mut self, id: u32, text: &str) -> Result<(), LcfError> {
        let bytes = self.encoder.encode(text);
        self.write_chunk(id, &bytes)
    }

    pub fn write_dbstring_chunk(&mut self, id: u32, text: &DBString) -> Result<(), LcfError> {
        if let Some(raw) = text.raw_bytes() {
            self.write_chunk(id, raw)
        } else {
            self.write_string_chunk(id, text.as_str())
        }
    }



    pub fn write_rect_chunk(&mut self, id: u32, rect: &Rect) -> Result<(), LcfError> {
        let mut buf = Vec::new();
        {
            let mut sub_w = LcfWriter::new(std::io::Cursor::new(&mut buf), self.engine, self.encoder.encoding_name());
            sub_w.write_rect(rect)?;
        }
        self.write_chunk(id, &buf)
    }

    pub fn write_vector_u8_chunk(&mut self, id: u32, vec: &[u8]) -> Result<(), LcfError> {
        self.write_chunk(id, vec)
    }

    pub fn write_vector_i16_chunk(&mut self, id: u32, vec: &[i16]) -> Result<(), LcfError> {
        let mut buf = Vec::new();
        {
            let mut sub_w = LcfWriter::new(std::io::Cursor::new(&mut buf), self.engine, self.encoder.encoding_name());
            sub_w.write_vector_i16(vec)?;
        }
        self.write_chunk(id, &buf)
    }

    pub fn write_sub_chunk<F>(&mut self, id: u32, f: F) -> Result<(), LcfError>
    where
        F: FnOnce(&mut LcfWriter<std::io::Cursor<Vec<u8>>>) -> Result<(), LcfError>,
    {
        let encoding_name = self.encoder.encoding_name().to_string();
        let engine = self.engine;
        let mut sub_w = LcfWriter::new(std::io::Cursor::new(Vec::new()), engine, &encoding_name);
        f(&mut sub_w)?;
        let buf = sub_w.stream.into_inner();
        self.write_chunk(id, &buf)?;
        Ok(())
    }

    /// Calculate the byte length of a compressed integer.
    pub fn int_size(mut val: u32) -> usize {
        let mut count = 0;
        loop {
            val >>= 7;
            count += 1;
            if val == 0 {
                break;
            }
        }
        count
    }
}
