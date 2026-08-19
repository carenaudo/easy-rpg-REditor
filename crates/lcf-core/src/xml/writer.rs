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

    pub fn write_node_string(&mut self, name: &str, val: &str) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>", name)?;
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
        write!(self.writer, "</{}>", name)?;
        self.new_line()?;
        Ok(())
    }

    pub fn write_node_dbstring(&mut self, name: &str, val: &DBString) -> Result<(), LcfError> {
        self.write_node_string(name, val.as_str())
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

    pub fn write_node_rect(&mut self, name: &str, rect: &Rect) -> Result<(), LcfError> {
        self.new_line()?;
        self.do_indent()?;
        write!(self.writer, "<{}>{} {} {} {}</{}>", name, rect.l, rect.t, rect.r, rect.b, name)?;
        self.new_line()?;
        Ok(())
    }
}
