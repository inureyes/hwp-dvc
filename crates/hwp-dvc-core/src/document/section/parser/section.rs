//! Top-level `<hs:sec>` dispatcher.
//!
//! The section root contains zero or more `<hp:p>` children. Everything
//! else at this level (stray whitespace, processing instructions, the
//! XML declaration) is ignored. Each `<hp:p>` is handed off to the
//! paragraph walker.

use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::document::section::types::Section;
use crate::error::DvcResult;

use super::common::local_name;
use super::paragraph::parse_paragraph;

/// Parse one `Contents/sectionN.xml` byte slice into a [`Section`].
///
/// `index` is the numeric `N` extracted from the part filename
/// (`sectionN.xml`) and preserved on the returned [`Section`] so that
/// downstream code can re-emit it in error messages.
pub fn parse_section(index: u32, bytes: &[u8]) -> DvcResult<Section> {
    let mut reader = Reader::from_reader(bytes);
    let config = reader.config_mut();
    // Leave text alone — paragraph text must preserve surrounding
    // whitespace because the OWPML reference keeps it verbatim.
    config.trim_text(false);
    config.expand_empty_elements = false;

    let mut section = Section {
        index,
        outline_shape_id_ref: 0,
        paragraphs: Vec::new(),
    };

    // Collected by `parse_paragraph` -> `parse_run` when it encounters
    // `<hp:secPr outlineShapeIDRef="..">`. The first occurrence in the
    // section wins (always the first run of the first paragraph in a
    // well-formed HWPX).
    let mut sec_outline_shape_id_ref: Option<u32> = None;

    dispatch(&mut reader, &mut section, &mut sec_outline_shape_id_ref)?;
    section.outline_shape_id_ref = sec_outline_shape_id_ref.unwrap_or(0);
    Ok(section)
}

fn dispatch<B: BufRead>(
    reader: &mut Reader<B>,
    section: &mut Section,
    sec_outline_shape_id_ref: &mut Option<u32>,
) -> DvcResult<()> {
    let mut buf = Vec::new();
    // nesting_depth = 0 at the section root — any `<hp:tbl>` produced
    // directly inside a top-level paragraph becomes a depth-0 table.
    let base_depth = 0u32;
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) if local_name(e.name()) == b"p" => {
                let para = parse_paragraph(reader, e, base_depth, sec_outline_shape_id_ref)?;
                section.paragraphs.push(para);
            }
            Event::Empty(ref e) if local_name(e.name()) == b"p" => {
                // Empty paragraph — rare but legal.
                let para = crate::document::section::types::Paragraph {
                    para_pr_id_ref: super::common::attr_u32(e.attributes(), b"paraPrIDRef")?,
                    style_id_ref: super::common::attr_u32(e.attributes(), b"styleIDRef")?,
                    runs: Vec::new(),
                    tables: Vec::new(),
                };
                section.paragraphs.push(para);
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
        buf.clear();
    }
}
