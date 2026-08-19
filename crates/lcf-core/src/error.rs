use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LcfError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid LCF header: expected '{expected}', found '{found}'")]
    InvalidHeader {
        expected: &'static str,
        found: String,
    },

    #[error("Invalid compressed integer at offset {offset}")]
    InvalidVarint { offset: u64 },

    #[error("Premature end of file at offset {offset}")]
    UnexpectedEof { offset: u64 },

    #[error("Chunk size mismatch in struct '{struct_name}' for chunk {chunk_id}: expected {expected} bytes, processed {actual} bytes")]
    ChunkSizeMismatch {
        struct_name: &'static str,
        chunk_id: u32,
        expected: usize,
        actual: usize,
    },

    #[error("String encoding error: {0}")]
    EncodingError(String),

    #[error("XML parsing error: {0}")]
    XmlError(String),

    #[error("Format error: {0}")]
    FormatError(String),
}
