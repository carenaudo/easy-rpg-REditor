use std::io::BufRead;
use quick_xml::events::Event;
use quick_xml::Reader;
use crate::error::LcfError;

pub struct XmlReader<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<R: BufRead> XmlReader<R> {
    pub fn new(reader: R) -> Self {
        let mut r = Reader::from_reader(reader);
        r.config_mut().trim_text(true);
        Self {
            reader: r,
            buf: Vec::new(),
        }
    }

    pub fn inner_mut(&mut self) -> &mut Reader<R> {
        &mut self.reader
    }

    pub fn next_event(&mut self) -> Result<Event<'static>, LcfError> {
        self.buf.clear();
        let event = self.reader.read_event_into(&mut self.buf)
            .map_err(|e| LcfError::XmlError(e.to_string()))?;
        Ok(event.into_owned())
    }
}
