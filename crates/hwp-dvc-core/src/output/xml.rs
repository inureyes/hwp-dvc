//! XML output formatter.
//!
//! Produces a `<dvcErrors>` document containing one `<error>` element per
//! [`DvcErrorInfo`] record. Field names mirror the JSON keys defined in the
//! companion `json.rs` module so that both outputs are structurally equivalent.
//!
//! # Schema (informal)
//!
//! ```xml
//! <dvcErrors>
//!   <error>
//!     <charIDRef>0</charIDRef>
//!     <paraPrIDRef>0</paraPrIDRef>
//!     <text>example run text</text>
//!     <pageNo>1</pageNo>
//!     <lineNo>0</lineNo>
//!     <errorCode>1004</errorCode>
//!     <tableID>0</tableID>
//!     <isInTable>false</isInTable>
//!     <isInTableInTable>false</isInTableInTable>
//!     <tableRow>0</tableRow>
//!     <tableCol>0</tableCol>
//!   </error>
//! </dvcErrors>
//! ```
//!
//! This module is compiled only when the `xml` Cargo feature is enabled.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use crate::checker::DvcErrorInfo;
use crate::error::DvcResult;

/// Serialize `errors` as an XML string.
///
/// When `pretty` is `true` the output is indented with two spaces; when
/// `false` the entire document is written on a single line (no whitespace
/// between tags).
pub fn to_xml(errors: &[DvcErrorInfo], pretty: bool) -> DvcResult<String> {
    let mut buf: Vec<u8> = Vec::new();

    let mut writer = if pretty {
        Writer::new_with_indent(&mut buf, b' ', 2)
    } else {
        Writer::new(&mut buf)
    };

    // <?xml version="1.0" encoding="UTF-8"?>
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // <dvcErrors>
    writer.write_event(Event::Start(BytesStart::new("dvcErrors")))?;

    for error in errors {
        write_error(&mut writer, error)?;
    }

    // </dvcErrors>
    writer.write_event(Event::End(BytesEnd::new("dvcErrors")))?;

    let s = String::from_utf8(buf)
        .map_err(|e| quick_xml::Error::Io(std::sync::Arc::new(std::io::Error::other(e))))?;
    Ok(s)
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

type XmlWriter<'a> = Writer<&'a mut Vec<u8>>;

fn write_error(w: &mut XmlWriter<'_>, e: &DvcErrorInfo) -> Result<(), quick_xml::Error> {
    w.write_event(Event::Start(BytesStart::new("error")))?;

    write_elem(w, "charIDRef", &e.char_pr_id_ref.to_string())?;
    write_elem(w, "paraPrIDRef", &e.para_pr_id_ref.to_string())?;
    write_elem(w, "text", &e.text)?;
    write_elem(w, "pageNo", &e.page_no.to_string())?;
    write_elem(w, "lineNo", &e.line_no.to_string())?;
    write_elem(w, "errorCode", &e.error_code.to_string())?;
    write_elem(w, "tableID", &e.table_id.to_string())?;
    write_elem(w, "isInTable", bool_str(e.is_in_table))?;
    write_elem(w, "isInTableInTable", bool_str(e.is_in_table_in_table))?;
    write_elem(w, "tableRow", &e.table_row.to_string())?;
    write_elem(w, "tableCol", &e.table_col.to_string())?;

    w.write_event(Event::End(BytesEnd::new("error")))?;
    Ok(())
}

fn write_elem(w: &mut XmlWriter<'_>, tag: &str, value: &str) -> Result<(), quick_xml::Error> {
    w.write_event(Event::Start(BytesStart::new(tag)))?;
    w.write_event(Event::Text(BytesText::new(value)))?;
    w.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

#[inline]
const fn bool_str(v: bool) -> &'static str {
    if v {
        "true"
    } else {
        "false"
    }
}
