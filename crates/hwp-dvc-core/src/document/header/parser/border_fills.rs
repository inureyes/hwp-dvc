//! `<hh:borderFills>` — cell-border decorations keyed by `id`.

use std::io::BufRead;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::{Border, BorderFill, LineType};
use crate::document::header::HeaderTables;
use crate::error::{DvcError, DvcResult};

use super::common::{attr_bool, attr_string, attr_u32, local_name, parse_width_mm, skip};

pub(super) fn parse<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"borderFill" => {
                let mut bf = BorderFill {
                    id: attr_u32(e.attributes(), b"id")?,
                    three_d: attr_bool(e.attributes(), b"threeD")?,
                    shadow: attr_bool(e.attributes(), b"shadow")?,
                    center_line: attr_string(e.attributes(), b"centerLine")?,
                    break_cell_separate_line: attr_bool(e.attributes(), b"breakCellSeparateLine")?,
                    ..Default::default()
                };
                parse_border_fill_body(reader, &mut bf)?;
                tables.border_fills.insert(bf.id, bf);
            }
            Event::End(ref e) if local_name(e.name()) == b"borderFills" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <borderFills>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_border_fill_body<B: BufRead>(
    reader: &mut Reader<B>,
    bf: &mut BorderFill,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name());
                let is_start = matches!(ev, Event::Start(_));
                match name {
                    b"leftBorder" => bf.left = parse_border(e.attributes())?,
                    b"rightBorder" => bf.right = parse_border(e.attributes())?,
                    b"topBorder" => bf.top = parse_border(e.attributes())?,
                    b"bottomBorder" => bf.bottom = parse_border(e.attributes())?,
                    b"diagonal" => bf.diagonal = parse_border(e.attributes())?,
                    b"fillBrush" => bf.has_fill_brush = true,
                    _ => {}
                }
                if is_start {
                    skip(reader, e)?;
                }
            }
            Event::End(e) if local_name(e.name()) == b"borderFill" => return Ok(()),
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <borderFill>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_border(attrs: Attributes<'_>) -> DvcResult<Border> {
    let type_str = attr_string(attrs.clone(), b"type")?;
    let width_str = attr_string(attrs.clone(), b"width")?;
    let color = attr_string(attrs, b"color")?;
    Ok(Border {
        line_type: LineType::parse(&type_str),
        width_mm: parse_width_mm(&width_str),
        color,
    })
}
