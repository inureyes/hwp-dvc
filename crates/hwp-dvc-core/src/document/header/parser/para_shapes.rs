//! `<hh:paraProperties>` — paragraph-shape records keyed by `id`.
//!
//! The biggest single record in the header. The `<hp:switch>` wrapper
//! around margin + lineSpacing is special-cased because HWPX emits
//! both an `HwpUnitChar`-required case and a default-namespace
//! fallback; the reference C++ always consumes the first branch.

use std::io::BufRead;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::{
    HAlign, HeadingType, LineBreakWord, LineSpacing, LineSpacingType, Margin, ParaShape, VAlign,
};
use crate::document::header::HeaderTables;
use crate::error::{DvcError, DvcResult};

use super::common::{attr_bool, attr_i32, attr_string, attr_u32, local_name, skip};

pub(super) fn parse<B: BufRead>(
    reader: &mut Reader<B>,
    tables: &mut HeaderTables,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"paraPr" => {
                let mut ps = ParaShape {
                    id: attr_u32(e.attributes(), b"id")?,
                    tab_pr_id_ref: attr_u32(e.attributes(), b"tabPrIDRef")?,
                    condense: attr_u32(e.attributes(), b"condense")?,
                    font_line_height: attr_bool(e.attributes(), b"fontLineHeight")?,
                    snap_to_grid: attr_bool(e.attributes(), b"snapToGrid")?,
                    suppress_line_numbers: attr_bool(e.attributes(), b"suppressLineNumbers")?,
                    checked: attr_bool(e.attributes(), b"checked")?,
                    ..Default::default()
                };
                parse_para_pr_body(reader, &mut ps)?;
                tables.para_shapes.insert(ps.id, ps);
            }
            Event::End(ref e) if local_name(e.name()) == b"paraProperties" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <paraProperties>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_para_pr_body<B: BufRead>(reader: &mut Reader<B>, ps: &mut ParaShape) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name()).to_vec();
                let is_start = matches!(ev, Event::Start(_));
                match name.as_slice() {
                    b"align" => {
                        ps.h_align = HAlign::parse(&attr_string(e.attributes(), b"horizontal")?);
                        ps.v_align = VAlign::parse(&attr_string(e.attributes(), b"vertical")?);
                    }
                    b"heading" => {
                        ps.heading_type =
                            HeadingType::parse(&attr_string(e.attributes(), b"type")?);
                        ps.heading_id_ref = attr_u32(e.attributes(), b"idRef")?;
                        ps.heading_level = attr_u32(e.attributes(), b"level")?;
                    }
                    b"breakSetting" => {
                        ps.break_latin_word =
                            LineBreakWord::parse(&attr_string(e.attributes(), b"breakLatinWord")?);
                        ps.break_non_latin_word = LineBreakWord::parse(&attr_string(
                            e.attributes(),
                            b"breakNonLatinWord",
                        )?);
                        ps.widow_orphan = attr_bool(e.attributes(), b"widowOrphan")?;
                        ps.keep_with_next = attr_bool(e.attributes(), b"keepWithNext")?;
                        ps.keep_lines = attr_bool(e.attributes(), b"keepLines")?;
                        ps.page_break_before = attr_bool(e.attributes(), b"pageBreakBefore")?;
                        // `lineWrap` is "BREAK"/"KEEP" in HWPX; treat BREAK as true.
                        ps.line_wrap = attr_string(e.attributes(), b"lineWrap")? == "BREAK";
                    }
                    b"autoSpacing" => {
                        ps.auto_spacing_eng = attr_bool(e.attributes(), b"eAsianEng")?;
                        ps.auto_spacing_num = attr_bool(e.attributes(), b"eAsianNum")?;
                    }
                    b"switch" if is_start => {
                        parse_para_pr_switch(reader, ps)?;
                        buf.clear();
                        continue;
                    }
                    b"margin" if is_start => {
                        ps.margin = parse_margin_body(reader)?;
                        buf.clear();
                        continue;
                    }
                    b"lineSpacing" => {
                        ps.line_spacing = parse_line_spacing(e.attributes())?;
                    }
                    b"border" => {
                        ps.border_fill_id_ref = attr_u32(e.attributes(), b"borderFillIDRef")?;
                        ps.border_offset_left = attr_i32(e.attributes(), b"offsetLeft")?;
                        ps.border_offset_right = attr_i32(e.attributes(), b"offsetRight")?;
                        ps.border_offset_top = attr_i32(e.attributes(), b"offsetTop")?;
                        ps.border_offset_bottom = attr_i32(e.attributes(), b"offsetBottom")?;
                        ps.connect = attr_bool(e.attributes(), b"connect")?;
                        ps.ignore_margin = attr_bool(e.attributes(), b"ignoreMargin")?;
                    }
                    _ => {}
                }
                if is_start {
                    skip(reader, e)?;
                }
            }
            Event::End(e) if local_name(e.name()) == b"paraPr" => return Ok(()),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <paraPr>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// The `<hp:switch>` inside a `<hh:paraPr>` contains the margin +
/// lineSpacing in both "HwpUnitChar-required" and default namespaces.
/// The reference's `RParaShape` consumes the `<hp:case required-namespace=
/// ".../HwpUnitChar">` branch. We take the first `<margin>` and
/// `<lineSpacing>` we encounter — that is the `<hp:case>` branch in
/// every HWPX in-the-wild — and ignore the rest.
fn parse_para_pr_switch<B: BufRead>(reader: &mut Reader<B>, ps: &mut ParaShape) -> DvcResult<()> {
    let mut buf = Vec::new();
    let mut got_margin = false;
    let mut got_line_spacing = false;
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Start(e) => match local_name(e.name()) {
                b"margin" => {
                    if !got_margin {
                        ps.margin = parse_margin_body(reader)?;
                        got_margin = true;
                    } else {
                        skip(reader, e)?;
                    }
                }
                _ => {
                    // `<hp:case>` or `<hp:default>` wrappers, or any
                    // other nested element — fall through so that the
                    // inner <margin>/<lineSpacing> are reachable as
                    // subsequent events.
                }
            },
            Event::Empty(e) if local_name(e.name()) == b"lineSpacing" && !got_line_spacing => {
                ps.line_spacing = parse_line_spacing(e.attributes())?;
                got_line_spacing = true;
            }
            Event::Empty(_) => {}
            Event::End(e) if local_name(e.name()) == b"switch" => return Ok(()),
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <paraPr switch>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_margin_body<B: BufRead>(reader: &mut Reader<B>) -> DvcResult<Margin> {
    let mut buf = Vec::new();
    let mut m = Margin::default();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name());
                let is_start = matches!(ev, Event::Start(_));
                let v = attr_i32(e.attributes(), b"value")?;
                match name {
                    b"intent" | b"indent" => m.indent = v,
                    b"left" => m.left = v,
                    b"right" => m.right = v,
                    b"prev" => m.prev = v,
                    b"next" => m.next = v,
                    _ => {}
                }
                if is_start {
                    skip(reader, e)?;
                }
            }
            Event::End(e) if local_name(e.name()) == b"margin" => return Ok(m),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <margin>".into())),
            _ => {}
        }
        buf.clear();
    }
}

fn parse_line_spacing(attrs: Attributes<'_>) -> DvcResult<LineSpacing> {
    Ok(LineSpacing {
        type_: LineSpacingType::parse(&attr_string(attrs.clone(), b"type")?),
        value: attr_i32(attrs.clone(), b"value")?,
        unit: attr_string(attrs, b"unit")?,
    })
}
