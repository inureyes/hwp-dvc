//! `<hh:borderFills>` — cell-border decorations keyed by `id`.

use std::io::BufRead;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::header::types::{Border, BorderFill, CellFillBrush, LineType};
use crate::document::header::HeaderTables;
use crate::error::{DvcError, DvcResult};

use super::common::{attr_bool, attr_i32, attr_string, attr_u32, local_name, parse_width_mm, skip};

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
            Event::Empty(e) => {
                match local_name(e.name()) {
                    b"leftBorder" => bf.left = parse_border(e.attributes())?,
                    b"rightBorder" => bf.right = parse_border(e.attributes())?,
                    b"topBorder" => bf.top = parse_border(e.attributes())?,
                    b"bottomBorder" => bf.bottom = parse_border(e.attributes())?,
                    b"diagonal" => bf.diagonal = parse_border(e.attributes())?,
                    b"fillBrush" => {
                        // Self-closing `<hc:fillBrush/>` is degenerate
                        // but legal — record the flag only.
                        bf.has_fill_brush = true;
                    }
                    _ => {}
                }
            }
            Event::Start(e) => match local_name(e.name()) {
                b"leftBorder" => {
                    bf.left = parse_border(e.attributes())?;
                    skip(reader, e)?;
                }
                b"rightBorder" => {
                    bf.right = parse_border(e.attributes())?;
                    skip(reader, e)?;
                }
                b"topBorder" => {
                    bf.top = parse_border(e.attributes())?;
                    skip(reader, e)?;
                }
                b"bottomBorder" => {
                    bf.bottom = parse_border(e.attributes())?;
                    skip(reader, e)?;
                }
                b"diagonal" => {
                    bf.diagonal = parse_border(e.attributes())?;
                    skip(reader, e)?;
                }
                b"fillBrush" => {
                    bf.has_fill_brush = true;
                    bf.fill_brush = parse_fill_brush(reader)?;
                }
                _ => skip(reader, e)?,
            },
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

/// Parse a `<hc:fillBrush>` element body, returning the decoded
/// [`CellFillBrush`] or `None` if the brush contained no recognised
/// child (which never happens in well-formed HWPX but is accepted for
/// forward compatibility).
fn parse_fill_brush<B: BufRead>(reader: &mut Reader<B>) -> DvcResult<Option<CellFillBrush>> {
    let mut out: Option<CellFillBrush> = None;
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name());
                let is_start = matches!(ev, Event::Start(_));
                match name {
                    b"winBrush" => {
                        out = Some(CellFillBrush::Solid {
                            face: attr_string(e.attributes(), b"faceColor")?,
                            hatch: attr_string(e.attributes(), b"hatchColor")?,
                            alpha: attr_u32(e.attributes(), b"alpha")?,
                        });
                        if is_start {
                            skip(reader, e)?;
                        }
                    }
                    b"gradation" => {
                        let brush = parse_gradation(e.attributes())?;
                        out = Some(brush);
                        if is_start {
                            skip(reader, e)?;
                        }
                    }
                    b"imgBrush" => {
                        // `<hc:imgBrush>` contains `<hc:img>` with the binary
                        // ref, plus numeric attributes directly on the brush
                        // tag. Capture the top-level attributes first; the
                        // inner `<hc:img>` binary ref is captured by the
                        // nested scan below.
                        let img = parse_img_brush(e.attributes())?;
                        out = Some(img);
                        if is_start {
                            // Walk children for `<hc:img binaryItemIDRef="..">`.
                            if let Some(CellFillBrush::Image { file, .. }) = out.as_mut() {
                                *file = scan_img_ref(reader)?;
                            } else {
                                skip(reader, e)?;
                            }
                        }
                    }
                    _ => {
                        if is_start {
                            skip(reader, e)?;
                        }
                    }
                }
            }
            Event::End(e) if local_name(e.name()) == b"fillBrush" => return Ok(out),
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <fillBrush>".into(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

/// Decode the attributes of `<hc:gradation ...>`. The gradation element
/// carries all of its geometry on the opening tag:
///
/// ```xml
/// <hc:gradation type="LINEAR" angle="90" centerX="50" centerY="50"
///               step="255" colorNum="2" step_center="50" alpha="0">
///     <hc:color value="#000000"/>
///     <hc:color value="#FFFFFF"/>
/// </hc:gradation>
/// ```
///
/// The color list is parsed by the outer walker (this helper only
/// consumes the start-tag attributes). In practice the parser that
/// invokes this function uses `skip` immediately after, discarding the
/// inner `<hc:color>` sequence. For well-formed docs we fill
/// `start_color` and `end_color` with the values found on the outer
/// tag's own attributes when present (via `startColor` / `endColor`),
/// otherwise as empty strings. A follow-up pass can be added if a real
/// gradient fixture is introduced in the test suite.
fn parse_gradation(attrs: Attributes<'_>) -> DvcResult<CellFillBrush> {
    Ok(CellFillBrush::Gradation {
        gradation_type: attr_string(attrs.clone(), b"type")?,
        start_color: attr_string(attrs.clone(), b"startColor")?,
        end_color: attr_string(attrs.clone(), b"endColor")?,
        width_center: attr_u32(attrs.clone(), b"centerX")?,
        height_center: attr_u32(attrs.clone(), b"centerY")?,
        angle: attr_u32(attrs.clone(), b"angle")?,
        blur_level: attr_u32(attrs.clone(), b"step")?,
        blur_center: attr_u32(attrs, b"step_center")?,
    })
}

/// Decode attributes on the `<hc:imgBrush>` opening tag. The actual
/// binary reference lives on the inner `<hc:img binaryItemIDRef=".."/>`
/// child and is filled in by [`scan_img_ref`] afterwards.
fn parse_img_brush(attrs: Attributes<'_>) -> DvcResult<CellFillBrush> {
    Ok(CellFillBrush::Image {
        file: String::new(),
        include: attr_bool(attrs.clone(), b"include")?,
        fill_type: attr_string(attrs.clone(), b"mode")?,
        fill_value: attr_i32(attrs.clone(), b"alpha")?,
        effect_type: attr_string(attrs.clone(), b"effect")?,
        effect_value: attr_i32(attrs.clone(), b"effectValue")?,
        watermark: attr_u32(attrs, b"watermark")?,
    })
}

/// Walk the body of an `<hc:imgBrush>` element and return the binary
/// reference from its `<hc:img binaryItemIDRef=".."/>` child. If no
/// such child is found (malformed), returns an empty string.
fn scan_img_ref<B: BufRead>(reader: &mut Reader<B>) -> DvcResult<String> {
    let mut out = String::new();
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match &ev {
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(e.name());
                let is_start = matches!(ev, Event::Start(_));
                if name == b"img" && out.is_empty() {
                    out = attr_string(e.attributes(), b"binaryItemIDRef")?;
                }
                if is_start {
                    skip(reader, e)?;
                }
            }
            Event::End(e) if local_name(e.name()) == b"imgBrush" => return Ok(out),
            Event::Eof => {
                return Err(DvcError::Document(
                    "unexpected EOF inside <imgBrush>".into(),
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
