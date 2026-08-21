use std::io::BufRead;
use quick_xml::events::Event;
use quick_xml::Reader;
use crate::error::LcfError;
use crate::types::DBString;

/// A child element's opening tag: its name and, if present, its `id="NNNN"`
/// attribute (liblcf writes struct ids as an XML attribute, not a child
/// node - see `XmlWriter::begin_element_with_id`).
#[derive(Debug, Clone)]
pub struct XmlTag {
    pub name: String,
    pub id: Option<i32>,
}

/// Mirror of `XmlWriter`: a small cursor over a `quick_xml` event stream.
///
/// The contract every reader method here follows (matching how
/// `XmlWriter`'s writers always finish by emitting their own closing tag):
/// every method that reads "one element's content" fully consumes that
/// element, including its matching end tag, before returning. This lets
/// callers loop `next_child()` to walk a container's children without
/// tracking nesting themselves - quick_xml's well-formedness guarantees do
/// that for us, as long as every child is fully drained before asking for
/// the next one.
pub struct XmlReader<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
    /// Set by `next_child`/`expect_root` when the tag just returned was a
    /// self-closing `<x/>` (no separate End event will ever arrive for it).
    /// The very next content-consuming call (`read_text_to_end`,
    /// `skip_to_end`, or another `next_child` used to walk that element's
    /// own children) must observe this and treat the element as having no
    /// content, without consuming whatever real event comes next in the
    /// stream (which belongs to a sibling or the parent's own end).
    pending_empty: bool,
}

impl<R: BufRead> XmlReader<R> {
    pub fn new(reader: R) -> Self {
        let mut r = Reader::from_reader(reader);
        r.config_mut().trim_text(true);
        Self {
            reader: r,
            buf: Vec::new(),
            pending_empty: false,
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

    fn take_pending_empty(&mut self) -> bool {
        if self.pending_empty {
            self.pending_empty = false;
            true
        } else {
            false
        }
    }

    fn tag_from_start(e: &quick_xml::events::BytesStart) -> Result<XmlTag, LcfError> {
        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
        let mut id = None;
        for attr in e.attributes() {
            let attr = attr.map_err(|e| LcfError::XmlError(e.to_string()))?;
            if attr.key.as_ref() == b"id" {
                let val = attr.unescape_value().map_err(|e| LcfError::XmlError(e.to_string()))?;
                id = val.parse::<i32>().ok();
            }
        }
        Ok(XmlTag { name, id })
    }

    /// Reads the root element's opening tag (e.g. `<LDB>`), expecting a
    /// document-level `Start`. Used once, at the top of each format's
    /// `load_xml`.
    pub fn expect_root(&mut self) -> Result<XmlTag, LcfError> {
        loop {
            match self.next_event()? {
                Event::Start(e) => return Self::tag_from_start(&e),
                Event::Empty(e) => {
                    let tag = Self::tag_from_start(&e)?;
                    self.pending_empty = true;
                    return Ok(tag);
                }
                Event::Decl(_) | Event::Comment(_) | Event::PI(_) | Event::DocType(_) => continue,
                Event::Eof => return Err(LcfError::XmlError("unexpected end of file: no root element".into())),
                _ => continue,
            }
        }
    }

    /// Reads forward until the next child `Start`/`Empty` tag is found and
    /// returns it (leaving the cursor positioned right after it, ready to
    /// read that element's own content). If an `End` tag is found first -
    /// closing whatever container the caller is iterating, or if the
    /// element the caller just entered turned out to be self-closing
    /// (`<x/>`, tracked via `pending_empty`) - returns `None` instead.
    pub fn next_child(&mut self) -> Result<Option<XmlTag>, LcfError> {
        if self.take_pending_empty() {
            return Ok(None);
        }
        loop {
            match self.next_event()? {
                Event::Start(e) => return Ok(Some(Self::tag_from_start(&e)?)),
                Event::Empty(e) => {
                    let tag = Self::tag_from_start(&e)?;
                    self.pending_empty = true;
                    return Ok(Some(tag));
                }
                Event::End(_) => return Ok(None),
                Event::Text(_) | Event::Comment(_) | Event::CData(_) | Event::PI(_) => continue,
                Event::Eof => return Err(LcfError::XmlError("unexpected end of file while reading children".into())),
                _ => continue,
            }
        }
    }

    /// Consumes the text content and matching end tag of the element whose
    /// start was just returned by `next_child`/`expect_root`, returning the
    /// (unescaped) text - or `""` for an empty element (`<x/>` or `<x></x>`).
    pub fn read_text_to_end(&mut self) -> Result<String, LcfError> {
        if self.take_pending_empty() {
            return Ok(String::new());
        }
        let mut text = String::new();
        loop {
            match self.next_event()? {
                Event::Text(t) => {
                    let unescaped = t.unescape().map_err(|e| LcfError::XmlError(e.to_string()))?;
                    text.push_str(&unescaped);
                }
                Event::CData(t) => {
                    text.push_str(&String::from_utf8_lossy(&t.into_inner()));
                }
                Event::End(_) => return Ok(text),
                Event::Comment(_) | Event::PI(_) => continue,
                Event::Eof => return Err(LcfError::XmlError("unexpected end of file while reading text content".into())),
                other => return Err(LcfError::XmlError(format!("unexpected event while reading text content: {other:?}"))),
            }
        }
    }

    /// Skips an entire element (any nesting) whose start was just returned
    /// by `next_child`, up to and including its end tag. Used for unknown
    /// tags so a foreign/newer XML file with extra fields doesn't error.
    pub fn skip_to_end(&mut self) -> Result<(), LcfError> {
        if self.take_pending_empty() {
            return Ok(());
        }
        let mut depth = 0u32;
        loop {
            match self.next_event()? {
                Event::Start(_) => depth += 1,
                Event::Empty(_) => {}
                Event::End(_) => {
                    if depth == 0 {
                        return Ok(());
                    }
                    depth -= 1;
                }
                Event::Eof => return Err(LcfError::XmlError("unexpected end of file while skipping element".into())),
                _ => {}
            }
        }
    }

    /// Consumes exactly the closing tag of a field-name wrapper after its
    /// singular struct-typed content (a `RawStruct` like `Rect`/`Equipment`,
    /// or a nested generated struct) has fully consumed its own tag+fields.
    /// E.g. after `<area_rect><Rect>...</Rect>` has been read up through
    /// `</Rect>`, this consumes the trailing `</area_rect>`.
    pub fn consume_wrapper_end(&mut self) -> Result<(), LcfError> {
        match self.next_child()? {
            None => Ok(()),
            Some(tag) => Err(LcfError::XmlError(format!(
                "expected closing tag but found unexpected element <{}>", tag.name
            ))),
        }
    }

    // -- primitive value readers, each assuming the element's start was
    //    just consumed by next_child/expect_root, and fully consuming
    //    through that element's own end tag --

    pub fn read_node_int(&mut self) -> Result<i32, LcfError> {
        Ok(self.read_text_to_end()?.trim().parse().unwrap_or(0))
    }

    pub fn read_node_f64(&mut self) -> Result<f64, LcfError> {
        Ok(self.read_text_to_end()?.trim().parse().unwrap_or(0.0))
    }

    pub fn read_node_bool(&mut self) -> Result<bool, LcfError> {
        Ok(self.read_text_to_end()?.trim() == "T")
    }

    pub fn read_node_string(&mut self) -> Result<String, LcfError> {
        self.read_text_to_end()
    }

    pub fn read_node_dbstring(&mut self) -> Result<DBString, LcfError> {
        Ok(DBString::new(self.read_text_to_end()?))
    }

    pub fn read_node_vector_i16(&mut self) -> Result<Vec<i16>, LcfError> {
        Ok(self.read_text_to_end()?.split_whitespace().filter_map(|s| s.parse().ok()).collect())
    }

    pub fn read_node_vector_u8(&mut self) -> Result<Vec<u8>, LcfError> {
        Ok(self.read_text_to_end()?.split_whitespace().filter_map(|s| s.parse().ok()).collect())
    }

    pub fn read_node_vector_i32(&mut self) -> Result<Vec<i32>, LcfError> {
        Ok(self.read_text_to_end()?.split_whitespace().filter_map(|s| s.parse().ok()).collect())
    }

    pub fn read_node_vector_u32(&mut self) -> Result<Vec<u32>, LcfError> {
        Ok(self.read_text_to_end()?.split_whitespace().filter_map(|s| s.parse().ok()).collect())
    }

    pub fn read_node_vector_bool(&mut self) -> Result<Vec<bool>, LcfError> {
        Ok(self.read_text_to_end()?.split_whitespace().map(|s| s == "T").collect())
    }
}
