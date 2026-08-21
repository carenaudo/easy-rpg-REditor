use std::io::Write;
use crate::error::LcfError;
use crate::types::{DBString, EngineVersion, Rect};

pub struct XmlWriter<W: Write> {
    writer: W,
    indent: usize,
    at_bol: bool,
    engine: EngineVersion,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(mut writer: W, engine: EngineVersion) -> Result<Self, LcfError> {
        writer.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
        Ok(Self {
            writer,
            indent: 0,
            at_bol: true,
            engine,
        })
    }

    pub fn engine(&self) -> EngineVersion {
        self.engine
    }

    fn new_line(&mut self) -> Result<(), LcfError> {
        if !self.at_bol {
            self.writer.write_all(b"\n")?;
            self.at_bol = true;
        }
        Ok(())
    }

    fn do_indent(&mut self) -> Result<(), LcfError> {
        if self.at_bol {
            for _ in 0..self.indent {
                self.writer.write_all(b" ")?;
            }
            self.at_bol = false;
        }
        Ok(())
    }

    pub fn begin_element(&mut self, name: &str) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        self.indent += 1;
        Ok(())
    }

    pub fn begin_element_with_id(&mut self, name: &str, id: i32) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{} id=\"{:04}\">", name, id)?;
        self.indent += 1;
        Ok(())
    }

    pub fn end_element(&mut self, name: &str) -> Result<(), LcfError> {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.do_indent()?;
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_bool(&mut self, name: &str, val: bool) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>{}</{}>", name, if val { "T" } else { "F" }, name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_int(&mut self, name: &str, val: i32) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>{}</{}>", name, val, name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_f64(&mut self, name: &str, val: f64) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>{}</{}>", name, val, name)?;
        self.new_line()?;
        Ok(())
    }

    /// Writes `val`, entity-escaped, with no surrounding tag or indentation
    /// of its own - for use inside an element the caller has already
    /// opened (e.g. a sparse `<item id="N">` in a `Vec<DBString>`).
    fn write_text_escaped(&mut self, val: &str) -> Result<(), LcfError> {
        for ch in val.chars() {
            match ch {
                '<' => self.writer.write_all(b"&lt;")?,
                '>' => self.writer.write_all(b"&gt;")?,
                '&' => self.writer.write_all(b"&amp;")?,
                '"' => self.writer.write_all(b"&quot;")?,
                '\'' => self.writer.write_all(b"&apos;")?,
                _ => {
                    let mut b = [0u8; 4];
                    let s = ch.encode_utf8(&mut b);
                    self.writer.write_all(s.as_bytes())?;
                }
            }
        }
        Ok(())
    }

    pub fn write_node_string(&mut self, name: &str, val: &str) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        self.write_text_escaped(val)?;
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_dbstring(&mut self, name: &str, val: &DBString) -> Result<(), LcfError> {
        self.write_node_string(name, val.as_str())
    }

    /// Writes just the escaped text content of a `DBString`, with no tag of
    /// its own, matching `RawStruct<DBString>::WriteXml` (`dbstring_struct.cpp`)
    /// which is always called inside a tag the caller already opened.
    pub fn write_node_dbstring_value(&mut self, val: &DBString) -> Result<(), LcfError> {
        self.write_text_escaped(val.as_str())
    }


    pub fn write_node_vector_i16(&mut self, name: &str, val: &[i16]) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        let mut first = true;
        for &item in val {
            if !first {
                write!(self.writer, " ")?;
            }
            first = false;
            write!(self.writer, "{}", item)?;
        }
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_vector_u8(&mut self, name: &str, val: &[u8]) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        let mut first = true;
        for &item in val {
            if !first {
                write!(self.writer, " ")?;
            }
            first = false;
            write!(self.writer, "{}", item)?;
        }
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_vector_i32(&mut self, name: &str, val: &[i32]) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        let mut first = true;
        for &item in val {
            if !first {
                write!(self.writer, " ")?;
            }
            first = false;
            write!(self.writer, "{}", item)?;
        }
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_vector_u32(&mut self, name: &str, val: &[u32]) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        let mut first = true;
        for &item in val {
            if !first {
                write!(self.writer, " ")?;
            }
            first = false;
            write!(self.writer, "{}", item)?;
        }
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    /// Writes a `bool` (or `DBBitArray`) vector as a single space-separated
    /// `T`/`F` text node, matching liblcf's `XmlWriter::WriteVector` over
    /// `std::vector<bool>` (see `writer_xml.cpp`), not per-item elements.
    pub fn write_node_vector_bool(&mut self, name: &str, val: &[bool]) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
        let mut first = true;
        for &item in val {
            if !first {
                write!(self.writer, " ")?;
            }
            first = false;
            write!(self.writer, "{}", if item { "T" } else { "F" })?;
        }
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    /// Writes a `Rect`-typed field as `<field><Rect><l>../t../r../b..</Rect></field>`,
    /// matching liblcf's double-wrap: the field-name tag from
    /// `TypedField::WriteXml`, wrapping `RawStruct<rpg::Rect>::WriteXml`'s
    /// own `<Rect>` tag with nested `l`/`t`/`r`/`b` elements (`lmt_rect.cpp`).
    pub fn write_node_rect(&mut self, name: &str, rect: &Rect) -> Result<(), LcfError> {
        self.begin_element(name)?;
        self.begin_element("Rect")?;
        self.write_node_int("l", rect.l)?;
        self.write_node_int("t", rect.t)?;
        self.write_node_int("r", rect.r)?;
        self.write_node_int("b", rect.b)?;
        self.end_element("Rect")?;
        self.end_element(name)?;
        Ok(())
    }
}
