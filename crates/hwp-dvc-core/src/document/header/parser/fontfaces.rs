//! `<hh:fontfaces>` — per-language font-id → face-name maps.

use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::{FontFace, FontLang};
use crate::error::{DvcError, DvcResult};

use super::common::{attr_string, attr_u32, local_name, skip};

pub(super) fn parse<B: BufRead>(reader: &mut Reader<B>, out: &mut Vec<FontFace>) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"fontface" => {
                let lang = attr_string(e.attributes(), b"lang")?;
                let mut face = FontFace {
                    lang: FontLang::parse(&lang).unwrap_or(FontLang::Hangul),
                    ..Default::default()
                };
                parse_fontface_body(reader, &mut face)?;
                out.push(face);
            }
            Event::End(ref e) if local_name(e.name()) == b"fontfaces" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <fontfaces>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_fontface_body<B: BufRead>(reader: &mut Reader<B>, face: &mut FontFace) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Empty(ref e) | Event::Start(ref e) if local_name(e.name()) == b"font" => {
                let id = attr_u32(e.attributes(), b"id")?;
                let name = attr_string(e.attributes(), b"face")?;
                face.fonts.insert(id, name);
                if matches!(ev, Event::Start(_)) {
                    skip(reader, e)?;
                }
            }
            Event::End(ref e) if local_name(e.name()) == b"fontface" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <fontface>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}
