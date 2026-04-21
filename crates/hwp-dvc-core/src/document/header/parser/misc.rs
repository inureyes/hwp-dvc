//! Smaller header tables: `<hh:styles>`, `<hh:bullets>`, `<hh:numberings>`.
//! Grouped together because each fits comfortably in under 80 lines.

use std::io::BufRead;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::{Bullet, Numbering, ParaHead, Style};
use crate::document::header::HeaderTables;
use crate::error::{DvcError, DvcResult};

use super::common::{attr_bool, attr_string, attr_u32, local_name, read_text_until_end, skip};

// ---------------------------------------------------------------------------
// <hh:styles>
// ---------------------------------------------------------------------------

pub(super) fn parse_styles<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) if local_name(e.name()) == b"style" => {
                let style = Style {
                    id: attr_u32(e.attributes(), b"id")?,
                    style_type: attr_string(e.attributes(), b"type")?,
                    name: attr_string(e.attributes(), b"name")?,
                    eng_name: attr_string(e.attributes(), b"engName")?,
                    para_pr_id_ref: attr_u32(e.attributes(), b"paraPrIDRef")?,
                    char_pr_id_ref: attr_u32(e.attributes(), b"charPrIDRef")?,
                    next_style_id_ref: attr_u32(e.attributes(), b"nextStyleIDRef")?,
                    lang_id: attr_u32(e.attributes(), b"langID")?,
                    lock_form: attr_bool(e.attributes(), b"lockForm")?,
                };
                if matches!(ev, Event::Start(_)) {
                    skip(reader, e)?;
                }
                tables.styles.insert(style.id, style);
            }
            Event::End(e) if local_name(e.name()) == b"styles" => return Ok(()),
            Event::Start(e) => skip(reader, e)?,
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <styles>".into())),
            _ => {}
        }
        buf.clear();
    }
}

// ---------------------------------------------------------------------------
// <hh:bullets>
// ---------------------------------------------------------------------------

pub(super) fn parse_bullets<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"bullet" => {
                let bullet = Bullet {
                    id: attr_u32(e.attributes(), b"id")?,
                    char: attr_string(e.attributes(), b"char")?,
                    use_image: attr_bool(e.attributes(), b"useImage")?,
                };
                skip(reader, e)?;
                tables.bullets.insert(bullet.id, bullet);
            }
            Event::Empty(ref e) if local_name(e.name()) == b"bullet" => {
                let bullet = Bullet {
                    id: attr_u32(e.attributes(), b"id")?,
                    char: attr_string(e.attributes(), b"char")?,
                    use_image: attr_bool(e.attributes(), b"useImage")?,
                };
                tables.bullets.insert(bullet.id, bullet);
            }
            Event::End(ref e) if local_name(e.name()) == b"bullets" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <bullets>".into())),
            _ => {}
        }
        buf.clear();
    }
}

// ---------------------------------------------------------------------------
// <hh:numberings>
// ---------------------------------------------------------------------------

pub(super) fn parse_numberings<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"numbering" => {
                let mut n = Numbering {
                    id: attr_u32(e.attributes(), b"id")?,
                    start: attr_u32(e.attributes(), b"start")?,
                    para_heads: Vec::new(),
                };
                parse_numbering_body(reader, &mut n)?;
                tables.numberings.insert(n.id, n);
            }
            Event::End(ref e) if local_name(e.name()) == b"numberings" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <numberings>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_numbering_body<B: BufRead>(reader: &mut Reader<B>, n: &mut Numbering) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"paraHead" => {
                let head = parse_para_head_attrs(e.attributes())?;
                let text = read_text_until_end(reader, b"paraHead")?;
                n.para_heads.push(ParaHead {
                    num_format_text: text,
                    ..head
                });
            }
            Event::Empty(ref e) if local_name(e.name()) == b"paraHead" => {
                n.para_heads.push(parse_para_head_attrs(e.attributes())?);
            }
            Event::End(ref e) if local_name(e.name()) == b"numbering" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <numbering>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_para_head_attrs(attrs: Attributes<'_>) -> DvcResult<ParaHead> {
    Ok(ParaHead {
        start: attr_u32(attrs.clone(), b"start")?,
        level: attr_u32(attrs.clone(), b"level")?,
        align: attr_string(attrs.clone(), b"align")?,
        use_inst_width: attr_bool(attrs.clone(), b"useInstWidth")?,
        auto_indent: attr_bool(attrs.clone(), b"autoIndent")?,
        width_adjust: attr_bool(attrs.clone(), b"widthAdjust")?,
        text_offset_type: attr_string(attrs.clone(), b"textOffsetType")?,
        text_offset: attr_u32(attrs.clone(), b"textOffset")?,
        num_format: attr_string(attrs.clone(), b"numFormat")?,
        char_pr_id_ref: attr_u32(attrs.clone(), b"charPrIDRef")?,
        checkable: attr_bool(attrs, b"checkable")?,
        num_format_text: String::new(),
    })
}
