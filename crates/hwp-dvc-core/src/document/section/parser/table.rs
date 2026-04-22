//! `<hp:tbl>`, `<hp:tr>`, `<hp:tc>` walker.
//!
//! A table carries top-level attributes (`id`, `borderFillIDRef`,
//! `rowCnt`, `colCnt`) and a sequence of `<hp:tr>` rows. Each row is
//! a sequence of `<hp:tc>` cells. Each cell has a `<hp:subList>`
//! containing more paragraphs; those paragraphs can themselves
//! contain tables, which is exactly how table nesting is expressed.
//!
//! This walker increments `nesting_depth` when descending from a cell
//! into its paragraphs: the parent table's depth + 1 becomes the
//! depth reported by any `<hp:tbl>` found inside. The top-level
//! section walker starts at depth 0, so a table directly in a body
//! paragraph reports depth 0, a table inside its cell reports depth
//! 1, and so on.

use std::io::BufRead;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::document::section::types::{Cell, Row, Table};
use crate::error::{DvcError, DvcResult};

use super::common::{attr_bool01, attr_i32, attr_string, attr_u32, local_name};
use super::paragraph::{parse_paragraph, skip};

/// Parse an `<hp:tbl>` element whose start tag has already been
/// consumed from the reader. `parent_depth` is the nesting depth the
/// enclosing paragraph was walked at. The returned table's
/// `nesting_depth == parent_depth` (i.e., a table directly in a
/// body-level paragraph is depth 0).
///
/// Beyond the minimal identity attributes, this walker also collects
/// the sizing / position / margin / caption fields the standard-mode
/// validator needs (issue #41). Attribute collection is tolerant of
/// omission: writers may legally leave default values off and the
/// defaults here (`0` for integers, empty string for enums) match the
/// "unset" semantics the validator relies on.
pub(super) fn parse_table<B: BufRead>(
    reader: &mut Reader<B>,
    start: &BytesStart<'_>,
    parent_depth: u32,
) -> DvcResult<Table> {
    let mut table = Table {
        id: attr_u32(start.attributes(), b"id")?,
        border_fill_id_ref: attr_u32(start.attributes(), b"borderFillIDRef")?,
        row_cnt: attr_u32(start.attributes(), b"rowCnt")?,
        col_cnt: attr_u32(start.attributes(), b"colCnt")?,
        cell_spacing: attr_u32(start.attributes(), b"cellSpacing")?,
        text_wrap: attr_string(start.attributes(), b"textWrap")?,
        text_flow: attr_string(start.attributes(), b"textFlow")?,
        numbering_type: attr_string(start.attributes(), b"numberingType")?,
        lock: attr_u32(start.attributes(), b"lock")?,
        no_adjust: attr_u32(start.attributes(), b"noAdjust")?,
        nesting_depth: parent_depth,
        ..Table::default()
    };

    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"tr" => {
                let row = parse_row(reader, parent_depth)?;
                table.rows.push(row);
            }
            Event::Empty(ref e) if local_name(e.name()) == b"tr" => {
                // Row with no cells — degenerate but legal.
                let _ = e; // attributes unused for an empty row
                table.rows.push(Row::default());
            }
            Event::Empty(ref e) => {
                collect_table_metadata(&mut table, e)?;
            }
            Event::Start(ref e) => match local_name(e.name()) {
                // Caption lives as a child element whose attributes
                // carry side/size/etc. We only read its attributes and
                // then skip its body — the inner paragraphs are not
                // needed by the standard-mode checker.
                b"caption" => {
                    collect_caption_metadata(&mut table, e)?;
                    skip(reader, e)?;
                }
                _ => skip(reader, e)?,
            },
            Event::End(ref e) if local_name(e.name()) == b"tbl" => return Ok(table),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <tbl>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Copy attributes off a direct `<hp:tbl>` child that is always
/// emitted as a self-closing element (`<hp:sz .../>`, `<hp:pos .../>`,
/// `<hp:outMargin .../>`). Unknown elements are silently ignored so
/// writer-specific additions do not break parsing.
fn collect_table_metadata(table: &mut Table, e: &BytesStart<'_>) -> DvcResult<()> {
    match local_name(e.name()) {
        b"sz" => {
            table.width = attr_u32(e.attributes(), b"width")?;
            table.height = attr_u32(e.attributes(), b"height")?;
            table.size_protect = attr_bool01(e.attributes(), b"protect")?;
        }
        b"pos" => {
            table.treat_as_char = attr_bool01(e.attributes(), b"treatAsChar")?;
            table.flow_with_text = attr_bool01(e.attributes(), b"flowWithText")?;
            table.allow_overlap = attr_bool01(e.attributes(), b"allowOverlap")?;
            table.hold_anchor_and_so = attr_bool01(e.attributes(), b"holdAnchorAndSO")?;
            table.affect_l_spacing = attr_bool01(e.attributes(), b"affectLSpacing")?;
            table.horz_rel_to = attr_string(e.attributes(), b"horzRelTo")?;
            table.vert_rel_to = attr_string(e.attributes(), b"vertRelTo")?;
            table.horz_align = attr_string(e.attributes(), b"horzAlign")?;
            table.vert_align = attr_string(e.attributes(), b"vertAlign")?;
            table.horz_offset = attr_i32(e.attributes(), b"horzOffset")?;
            table.vert_offset = attr_i32(e.attributes(), b"vertOffset")?;
        }
        b"outMargin" => {
            table.out_margin_left = attr_u32(e.attributes(), b"left")?;
            table.out_margin_right = attr_u32(e.attributes(), b"right")?;
            table.out_margin_top = attr_u32(e.attributes(), b"top")?;
            table.out_margin_bottom = attr_u32(e.attributes(), b"bottom")?;
        }
        _ => {}
    }
    Ok(())
}

/// Copy caption attributes off a `<hp:caption>` element. The reference
/// OWPML writer emits `side`, `sz`, `gap`, `fullSz`, and `lineWrap` on
/// the caption container itself. Missing attributes default to their
/// zero equivalents, consistent with the rest of the table walker.
fn collect_caption_metadata(table: &mut Table, e: &BytesStart<'_>) -> DvcResult<()> {
    table.has_caption = true;
    table.caption_side = attr_string(e.attributes(), b"side")?;
    table.caption_size = attr_u32(e.attributes(), b"sz")?;
    table.caption_spacing = attr_i32(e.attributes(), b"gap")?;
    table.caption_full_size = attr_bool01(e.attributes(), b"fullSz")?;
    table.caption_line_wrap = attr_bool01(e.attributes(), b"lineWrap")?;
    Ok(())
}

/// Parse an `<hp:tr>` body up to its closing tag.
fn parse_row<B: BufRead>(reader: &mut Reader<B>, parent_depth: u32) -> DvcResult<Row> {
    let mut row = Row::default();
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"tc" => {
                let cell = parse_cell(reader, e, parent_depth)?;
                row.cells.push(cell);
            }
            Event::End(ref e) if local_name(e.name()) == b"tr" => return Ok(row),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <tr>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Parse an `<hp:tc>` cell body.
///
/// A cell has exactly one `<hp:subList>` (a paragraph container) and
/// one `<hp:cellAddr>` (row/col coordinates), plus assorted sizing
/// children we ignore. The subList is walked at `parent_depth + 1`
/// so tables nested inside get the right depth.
fn parse_cell<B: BufRead>(
    reader: &mut Reader<B>,
    start: &BytesStart<'_>,
    parent_depth: u32,
) -> DvcResult<Cell> {
    let mut cell = Cell {
        row: 0,
        col: 0,
        border_fill_id_ref: attr_u32(start.attributes(), b"borderFillIDRef")?,
        paragraphs: Vec::new(),
    };

    let child_depth = parent_depth.saturating_add(1);
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) => match local_name(e.name()) {
                b"subList" => {
                    parse_sub_list(reader, &mut cell.paragraphs, child_depth)?;
                }
                _ => skip(reader, e)?,
            },
            Event::Empty(ref e) if local_name(e.name()) == b"cellAddr" => {
                cell.col = attr_u32(e.attributes(), b"colAddr")?;
                cell.row = attr_u32(e.attributes(), b"rowAddr")?;
            }
            Event::End(ref e) if local_name(e.name()) == b"tc" => return Ok(cell),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <tc>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Parse an `<hp:subList>` — a flat paragraph container inside a cell.
fn parse_sub_list<B: BufRead>(
    reader: &mut Reader<B>,
    out: &mut Vec<crate::document::section::types::Paragraph>,
    depth: u32,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    // `<hp:secPr>` never appears inside a table cell in a well-formed
    // HWPX — it is section-scoped and sits in the first run of the
    // first top-level paragraph. A per-cell throwaway sink keeps the
    // `parse_paragraph` signature uniform while discarding any
    // spurious match.
    let mut discard_outline_ref: Option<u32> = None;
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"p" => {
                let para = parse_paragraph(reader, e, depth, &mut discard_outline_ref)?;
                out.push(para);
            }
            Event::Empty(ref e) if local_name(e.name()) == b"p" => {
                out.push(crate::document::section::types::Paragraph {
                    para_pr_id_ref: attr_u32(e.attributes(), b"paraPrIDRef")?,
                    style_id_ref: attr_u32(e.attributes(), b"styleIDRef")?,
                    runs: Vec::new(),
                    tables: Vec::new(),
                });
            }
            Event::End(ref e) if local_name(e.name()) == b"subList" => return Ok(()),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <subList>".into())),
            _ => {}
        }
        buf.clear();
    }
}
