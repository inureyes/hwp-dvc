//! `<hh:charProperties>` — character-shape records keyed by `id`.

use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::CharShape;
use crate::document::header::HeaderTables;
use crate::error::{DvcError, DvcResult};

use super::common::{
    attr_bool, attr_string, attr_u32, local_name, parse_lang_tuple_i32, parse_lang_tuple_u32, skip,
};

pub(super) fn parse<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"charPr" => {
                let mut cs = CharShape {
                    id: attr_u32(e.attributes(), b"id")?,
                    height: attr_u32(e.attributes(), b"height")?,
                    text_color: attr_string(e.attributes(), b"textColor")?,
                    shade_color: attr_string(e.attributes(), b"shadeColor")?,
                    use_font_space: attr_bool(e.attributes(), b"useFontSpace")?,
                    use_kerning: attr_bool(e.attributes(), b"useKerning")?,
                    sym_mark: attr_string(e.attributes(), b"symMark")?,
                    border_fill_id_ref: attr_u32(e.attributes(), b"borderFillIDRef")?,
                    ..Default::default()
                };
                parse_char_pr_body(reader, &mut cs)?;
                tables.char_shapes.insert(cs.id, cs);
            }
            Event::End(ref e) if local_name(e.name()) == b"charProperties" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <charProperties>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_char_pr_body<B: BufRead>(reader: &mut Reader<B>, cs: &mut CharShape) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name());
                let is_start = matches!(ev, Event::Start(_));
                match name {
                    b"fontRef" => cs.font_ref = parse_lang_tuple_u32(e.attributes())?,
                    b"ratio" => cs.ratio = parse_lang_tuple_u32(e.attributes())?,
                    b"spacing" => cs.spacing = parse_lang_tuple_i32(e.attributes())?,
                    b"relSz" => cs.rel_sz = parse_lang_tuple_u32(e.attributes())?,
                    b"offset" => cs.offset = parse_lang_tuple_i32(e.attributes())?,
                    b"bold" => cs.bold = true,
                    b"italic" => cs.italic = true,
                    b"underline" => {
                        // Present iff underline is active (reference uses
                        // existence as the boolean); type/shape/color are
                        // ignored for the header-table surface.
                        cs.underline = true;
                    }
                    b"strikeout" => {
                        let shape = attr_string(e.attributes(), b"shape")?;
                        cs.strikeout = shape != "NONE" && !shape.is_empty();
                    }
                    b"outline" => {
                        let t = attr_string(e.attributes(), b"type")?;
                        cs.outline = t != "NONE" && !t.is_empty();
                    }
                    b"emboss" => cs.emboss = true,
                    b"engrave" => cs.engrave = true,
                    b"shadow" => {
                        let t = attr_string(e.attributes(), b"type")?;
                        cs.shadow = t != "NONE" && !t.is_empty();
                    }
                    b"supscript" => cs.supscript = true,
                    b"subscript" => cs.subscript = true,
                    _ => {}
                }
                if is_start {
                    skip(reader, e)?;
                }
            }
            Event::End(e) if local_name(e.name()) == b"charPr" => return Ok(()),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <charPr>".into())),
            _ => {}
        }
        buf.clear();
    }
}
