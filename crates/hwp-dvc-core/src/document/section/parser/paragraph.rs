//! `<hp:p>` and `<hp:run>` walker.
//!
//! OWPML paragraphs contain one or more `<hp:run>` elements. A run is
//! a slice of the paragraph sharing `charPrIDRef`; it can hold:
//!
//! - `<hp:t>` text — the only thing that produces visible characters.
//! - `<hp:tbl>` tables — inline objects anchored to the paragraph.
//! - `<hp:fieldBegin type="HYPERLINK">` / `<hp:fieldEnd>` pairs that
//!   bracket the runs belonging to one hyperlink. Toggle-style: any
//!   runs between the two markers are "inside" a hyperlink.
//! - Miscellaneous controls (`<hp:secPr>`, `<hp:ctrl>`, `<hp:colPr>`,
//!   `<hp:linesegarray>`, `<hp:fwSpace>`, ...). These do not
//!   contribute text; we skip past their subtrees.

use std::io::BufRead;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::document::section::types::{Paragraph, Run, Table};
use crate::error::{DvcError, DvcResult};

use super::common::{attr_string, attr_u32, local_name};
use super::table::parse_table;

/// Parse an `<hp:p>` element whose start tag has already been
/// consumed from the reader. Consumes through the matching `</hp:p>`.
///
/// `depth` is the current nesting depth. A top-level section
/// paragraph starts at depth 0; a cell paragraph inherits the enclosing
/// table's depth + 1 so that nested tables seen inside it get the
/// correct `nesting_depth` without a second pass.
pub(super) fn parse_paragraph<B: BufRead>(
    reader: &mut Reader<B>,
    start: &BytesStart<'_>,
    depth: u32,
) -> DvcResult<Paragraph> {
    let mut para = Paragraph {
        para_pr_id_ref: attr_u32(start.attributes(), b"paraPrIDRef")?,
        style_id_ref: attr_u32(start.attributes(), b"styleIDRef")?,
        runs: Vec::new(),
        tables: Vec::new(),
    };

    // Hyperlink-field bracket state. `FieldBegin type="HYPERLINK"`
    // flips this on; `FieldEnd` flips it off. At `</run>` close time,
    // we record the current value on the run: a run that saw the
    // `FieldBegin` in its body closes with the flag set and is
    // therefore flagged; a run that saw only the `FieldEnd` closes
    // with the flag cleared.
    let mut in_hyperlink = false;
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"run" => {
                parse_run(reader, e, &mut para, depth, &mut in_hyperlink)?;
            }
            Event::Empty(ref e) if local_name(e.name()) == b"run" => {
                // Control-only empty run — no children means no text
                // and no field-marker state change.
                para.runs.push(Run {
                    char_pr_id_ref: attr_u32(e.attributes(), b"charPrIDRef")?,
                    text: String::new(),
                    is_hyperlink: in_hyperlink,
                });
            }
            Event::End(ref e) if local_name(e.name()) == b"p" => return Ok(para),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <p>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Parse one `<hp:run>` body, pushing the run (and any inline tables
/// it owns) into `para`. Consumes through the matching `</hp:run>`.
///
/// `in_hyperlink` is a rolling flag owned by the parent paragraph:
/// `FieldBegin`/`FieldEnd` markers encountered inside the run toggle
/// it, and the value at `</run>` time is what the run records.
fn parse_run<B: BufRead>(
    reader: &mut Reader<B>,
    start: &BytesStart<'_>,
    para: &mut Paragraph,
    depth: u32,
    in_hyperlink: &mut bool,
) -> DvcResult<()> {
    let char_pr_id_ref = attr_u32(start.attributes(), b"charPrIDRef")?;
    let mut text = String::new();
    let mut tables: Vec<Table> = Vec::new();
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) => match local_name(e.name()) {
                b"t" => {
                    let collected = read_t_text(reader)?;
                    text.push_str(&collected);
                }
                b"tbl" => {
                    let t = parse_table(reader, e, depth)?;
                    tables.push(t);
                }
                b"fieldBegin" if is_hyperlink_field(e)? => {
                    *in_hyperlink = true;
                    skip(reader, e)?;
                }
                b"fieldEnd" if is_hyperlink_field(e)? => {
                    *in_hyperlink = false;
                    skip(reader, e)?;
                }
                _ => skip(reader, e)?,
            },
            Event::Empty(ref e) => match local_name(e.name()) {
                b"fieldBegin" if is_hyperlink_field(e)? => {
                    *in_hyperlink = true;
                }
                b"fieldEnd" if is_hyperlink_field(e)? => {
                    *in_hyperlink = false;
                }
                b"t" => {
                    // Empty text element — nothing to collect.
                }
                // Other inline controls (`<hp:tab/>`, `<hp:fwSpace/>`,
                // `<hp:lineBreak/>`, `<hp:colPr/>`, ...) contribute
                // nothing to the AST surface we expose.
                _ => {}
            },
            Event::End(ref e) if local_name(e.name()) == b"run" => {
                para.runs.push(Run {
                    char_pr_id_ref,
                    text,
                    is_hyperlink: *in_hyperlink,
                });
                para.tables.extend(tables);
                return Ok(());
            }
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <run>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Read the text content of an `<hp:t>` element up to its closing tag.
///
/// The element may contain `Event::Text` interleaved with inline
/// controls (`<hp:tab/>`, `<hp:fwSpace/>`, `<hp:lineBreak/>`, ...).
/// We preserve only text because only text contributes to validation
/// (control codes are checked via `<hp:secPr>` and friends, which live
/// outside `<hp:t>`).
fn read_t_text<B: BufRead>(reader: &mut Reader<B>) -> DvcResult<String> {
    let mut out = String::new();
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Text(t) => {
                let decoded = t
                    .decode()
                    .map_err(|e| DvcError::Document(format!("text decode in <t>: {e}")))?;
                out.push_str(&decoded);
            }
            Event::CData(c) => {
                let s = String::from_utf8(c.into_inner().to_vec())
                    .map_err(|e| DvcError::Document(format!("cdata utf8 in <t>: {e}")))?;
                out.push_str(&s);
            }
            Event::Start(ref e) => skip(reader, e)?,
            Event::Empty(_) => {}
            Event::End(ref e) if local_name(e.name()) == b"t" => return Ok(out),
            Event::Eof => return Err(DvcError::Document("unexpected EOF inside <t>".into())),
            _ => {}
        }
        buf.clear();
    }
}

/// Skip over a start element's body, consuming through its matching
/// end tag.
pub(super) fn skip<B: BufRead>(reader: &mut Reader<B>, start: &BytesStart<'_>) -> DvcResult<()> {
    reader.read_to_end_into(start.name(), &mut Vec::new())?;
    Ok(())
}

/// Return `true` if the `type=` attribute of a `<hp:fieldBegin>` or
/// `<hp:fieldEnd>` element is `HYPERLINK`.
fn is_hyperlink_field(e: &BytesStart<'_>) -> DvcResult<bool> {
    let t = attr_string(e.attributes(), b"type")?;
    Ok(t.eq_ignore_ascii_case("HYPERLINK"))
}
