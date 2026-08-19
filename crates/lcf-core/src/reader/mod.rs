use std::io::{Read, Seek, SeekFrom};
use crate::encoder::LcfEncoder;
use crate::error::LcfError;
use crate::types::{DBBitArray, DBString, Rect};

pub struct LcfReader<R: Read + Seek> {
    stream: R,
    encoder: LcfEncoder,
    offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkHeader {
    pub id: u32,
    pub length: usize,
}

impl<R: Read + Seek> LcfReader<R> {
    pub fn new(stream: R, encoding: &str) -> Self {
        Self {
            stream,
            encoder: LcfEncoder::new(encoding),
            offset: 0,
        }
    }

    pub fn encoder(&self) -> &LcfEncoder {
        &self.encoder
    }

    pub fn set_encoding(&mut self, encoding: &str) {
        self.encoder = LcfEncoder::new(encoding);
    }

    pub fn into_inner(self) -> R {
        self.stream
    }


    pub fn tell(&mut self) -> Result<u64, LcfError> {
        Ok(self.offset)
    }

    pub fn seek(&mut self, pos: u64) -> Result<(), LcfError> {
        self.stream.seek(SeekFrom::Start(pos))?;
        self.offset = pos;
        Ok(())
    }

    pub fn skip(&mut self, bytes: usize) -> Result<(), LcfError> {
        self.stream.seek(SeekFrom::Current(bytes as i64))?;
        self.offset += bytes as u64;
        Ok(())
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, LcfError> {
        let mut buf = vec![0u8; len];
        if len > 0 {
            self.stream.read_exact(&mut buf).map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    LcfError::UnexpectedEof { offset: self.offset }
                } else {
                    LcfError::Io(e)
                }
            })?;
            self.offset += len as u64;
        }
        Ok(buf)
    }

    pub fn read_u8(&mut self) -> Result<u8, LcfError> {
        let mut buf = [0u8; 1];
        self.stream.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LcfError::UnexpectedEof { offset: self.offset }
            } else {
                LcfError::Io(e)
            }
        })?;
        self.offset += 1;
        Ok(buf[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, LcfError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_i16(&mut self) -> Result<i16, LcfError> {
        let mut buf = [0u8; 2];
        self.stream.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LcfError::UnexpectedEof { offset: self.offset }
            } else {
                LcfError::Io(e)
            }
        })?;
        self.offset += 2;
        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, LcfError> {
        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LcfError::UnexpectedEof { offset: self.offset }
            } else {
                LcfError::Io(e)
            }
        })?;
        self.offset += 4;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, LcfError> {
        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LcfError::UnexpectedEof { offset: self.offset }
            } else {
                LcfError::Io(e)
            }
        })?;
        self.offset += 4;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64, LcfError> {
        let mut buf = [0u8; 8];
        self.stream.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LcfError::UnexpectedEof { offset: self.offset }
            } else {
                LcfError::Io(e)
            }
        })?;
        self.offset += 8;
        Ok(f64::from_le_bytes(buf))
    }

    /// Reads a 7-bit compressed integer (BER / varint format).
    pub fn read_int(&mut self) -> Result<i32, LcfError> {
        let mut value: i32 = 0;
        let mut loops = 0;
        loop {
            value <<= 7;
            let temp = self.read_u8()?;
            value |= (temp & 0x7F) as i32;
            loops += 1;
            if (temp & 0x80) == 0 {
                break;
            }
            if loops > 5 {
                return Err(LcfError::InvalidVarint { offset: self.offset });
            }
        }
        Ok(value)
    }

    /// Reads a 64-bit compressed integer.
    pub fn read_u64(&mut self) -> Result<u64, LcfError> {
        let mut value: u64 = 0;
        let mut loops = 0;
        loop {
            value <<= 7;
            let temp = self.read_u8()?;
            value |= (temp & 0x7F) as u64;
            loops += 1;
            if (temp & 0x80) == 0 {
                break;
            }
            if loops > 9 {
                return Err(LcfError::InvalidVarint { offset: self.offset });
            }
        }
        Ok(value)
    }

    pub fn read_bool(&mut self) -> Result<bool, LcfError> {
        Ok(self.read_int()? > 0)
    }

    pub fn read_flags(&mut self, len: usize) -> Result<i32, LcfError> {
        let bytes = self.read_bytes(len)?;
        let mut val = 0i32;
        for (i, &b) in bytes.iter().enumerate() {
            val |= (b as i32) << (i * 8);
        }
        Ok(val)
    }


    pub fn read_string(&mut self, len: usize) -> Result<DBString, LcfError> {
        let bytes = self.read_bytes(len)?;
        let decoded = self.encoder.decode(&bytes);
        Ok(DBString::from_raw(bytes, decoded))
    }

    pub fn read_raw_string(&mut self, len: usize) -> Result<String, LcfError> {
        let bytes = self.read_bytes(len)?;
        Ok(self.encoder.decode(&bytes))
    }

    pub fn read_rect(&mut self) -> Result<Rect, LcfError> {
        let l = self.read_i32_le()?;
        let t = self.read_i32_le()?;
        let r = self.read_i32_le()?;
        let b = self.read_i32_le()?;
        Ok(Rect { l, t, r, b })
    }

    pub fn read_vector_u8(&mut self, len: usize) -> Result<Vec<u8>, LcfError> {
        self.read_bytes(len)
    }

    pub fn read_vector_i16(&mut self, len: usize) -> Result<Vec<i16>, LcfError> {
        let items = len / 2;
        let mut result = Vec::with_capacity(items);
        for _ in 0..items {
            result.push(self.read_i16()?);
        }
        if len % 2 != 0 {
            self.skip(1)?;
            result.push(0);
        }
        Ok(result)
    }

    pub fn read_vector_i32(&mut self, len: usize) -> Result<Vec<i32>, LcfError> {
        let items = len / 4;
        let mut result = Vec::with_capacity(items);
        for _ in 0..items {
            result.push(self.read_i32_le()?);
        }
        if len % 4 != 0 {
            self.skip(len % 4)?;
            result.push(0);
        }
        Ok(result)
    }

    pub fn read_vector_bool(&mut self, len: usize) -> Result<Vec<bool>, LcfError> {
        let bytes = self.read_bytes(len)?;
        Ok(bytes.into_iter().map(|b| b > 0).collect())
    }

    pub fn read_bit_array(&mut self, len: usize) -> Result<DBBitArray, LcfError> {
        let bools = self.read_vector_bool(len)?;
        Ok(DBBitArray(bools))
    }

    pub fn read_chunk_header(&mut self) -> Result<Option<ChunkHeader>, LcfError> {
        let id = match self.read_int() {
            Ok(0) => return Ok(None),
            Ok(id) => id as u32,
            Err(LcfError::UnexpectedEof { .. }) => return Ok(None),
            Err(e) => return Err(e),
        };
        let length = self.read_int()? as usize;
        Ok(Some(ChunkHeader { id, length }))
    }
}
