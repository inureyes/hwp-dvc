//! Event-driven parser for `Contents/section*.xml`.
//!
//! The parser matches on XML local names so that the HWPX `hp:`
//! namespace prefix is transparent — consistent with the header
//! parser. Unknown elements are skipped with `Reader::read_to_end_into`
//! so that new attributes or children added by a future writer
//! revision do not break the walker.
//!
//! The walker is split across one file per node kind so that no file
//! exceeds the project's 500-line soft cap:
//!
//! | file | responsibility |
//! |------|----------------|
//! | `common.rs` | attribute/name helpers |
//! | `section.rs` | top-level `<hs:sec>` dispatcher |
//! | `paragraph.rs` | `<hp:p>` + `<hp:run>` + `<hp:t>` |
//! | `table.rs` | `<hp:tbl>` + `<hp:tr>` + `<hp:tc>` |

mod common;
mod paragraph;
mod section;
mod table;

pub use section::parse_section;

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal plain-paragraph section with one paragraph and one
    /// text-bearing run.
    const MINI_PLAIN: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
<hp:p id="1" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">
<hp:run charPrIDRef="0"><hp:t>Hello, world.</hp:t></hp:run>
</hp:p>
</hs:sec>"##;

    /// A 1x1 table inside a paragraph's run.
    const MINI_TABLE: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
<hp:p id="1" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">
<hp:run charPrIDRef="0">
<hp:tbl id="100" borderFillIDRef="3" rowCnt="1" colCnt="1">
<hp:tr>
<hp:tc borderFillIDRef="3">
<hp:subList>
<hp:p id="10" paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0"><hp:t>inside</hp:t></hp:run>
</hp:p>
</hp:subList>
<hp:cellAddr colAddr="0" rowAddr="0"/>
</hp:tc>
</hp:tr>
</hp:tbl>
</hp:run>
</hp:p>
</hs:sec>"##;

    /// A 1x1 table inside a 1x1 table cell — exercises nesting_depth.
    const MINI_NESTED: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
<hp:p id="1" paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0">
<hp:tbl id="100" borderFillIDRef="3" rowCnt="1" colCnt="1">
<hp:tr>
<hp:tc borderFillIDRef="3">
<hp:subList>
<hp:p id="10" paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0">
<hp:tbl id="200" borderFillIDRef="3" rowCnt="1" colCnt="1">
<hp:tr>
<hp:tc borderFillIDRef="3">
<hp:subList>
<hp:p id="20" paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0"><hp:t>deep</hp:t></hp:run>
</hp:p>
</hp:subList>
<hp:cellAddr colAddr="0" rowAddr="0"/>
</hp:tc>
</hp:tr>
</hp:tbl>
</hp:run>
</hp:p>
</hp:subList>
<hp:cellAddr colAddr="0" rowAddr="0"/>
</hp:tc>
</hp:tr>
</hp:tbl>
</hp:run>
</hp:p>
</hs:sec>"##;

    /// A paragraph with a hyperlink fieldBegin/fieldEnd pair.
    const MINI_HYPERLINK: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
<hp:p id="1" paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0"><hp:t>before </hp:t></hp:run>
<hp:run charPrIDRef="0"><hp:fieldBegin type="HYPERLINK" name="h1" fieldid="1"/><hp:t>link</hp:t></hp:run>
<hp:run charPrIDRef="0"><hp:fieldEnd type="HYPERLINK" fieldid="1"/><hp:t> after</hp:t></hp:run>
</hp:p>
</hs:sec>"##;

    #[test]
    fn parses_plain_paragraph_run() {
        let s = parse_section(0, MINI_PLAIN.as_bytes()).expect("parse ok");
        assert_eq!(s.paragraphs.len(), 1);
        let p = &s.paragraphs[0];
        assert_eq!(p.runs.len(), 1);
        assert_eq!(p.runs[0].text, "Hello, world.");
        assert_eq!(p.tables.len(), 0);
    }

    #[test]
    fn parses_single_table_at_depth_zero() {
        let s = parse_section(0, MINI_TABLE.as_bytes()).expect("parse ok");
        assert_eq!(s.paragraphs.len(), 1);
        let p = &s.paragraphs[0];
        assert_eq!(p.tables.len(), 1);
        let t = &p.tables[0];
        assert_eq!(t.id, 100);
        assert_eq!(t.nesting_depth, 0);
        assert_eq!(t.row_cnt, 1);
        assert_eq!(t.col_cnt, 1);
        assert_eq!(t.rows.len(), 1);
        let cell = &t.rows[0].cells[0];
        assert_eq!(cell.row, 0);
        assert_eq!(cell.col, 0);
        assert_eq!(cell.paragraphs[0].runs[0].text, "inside");
    }

    #[test]
    fn parses_nested_table_depth_one() {
        let s = parse_section(0, MINI_NESTED.as_bytes()).expect("parse ok");
        // Depth-0 table should be present, and its cell paragraph
        // must own a depth-1 nested table.
        let outer = &s.paragraphs[0].tables[0];
        assert_eq!(outer.nesting_depth, 0);
        let cell = &outer.rows[0].cells[0];
        let inner = &cell.paragraphs[0].tables[0];
        assert_eq!(inner.id, 200);
        assert_eq!(inner.nesting_depth, 1);
        assert_eq!(inner.rows[0].cells[0].paragraphs[0].runs[0].text, "deep");
    }

    #[test]
    fn detects_hyperlink_marker_runs() {
        let s = parse_section(0, MINI_HYPERLINK.as_bytes()).expect("parse ok");
        let p = &s.paragraphs[0];
        // The first run is before the hyperlink — not flagged.
        assert!(!p.runs[0].is_hyperlink);
        // The middle run opens FieldBegin type=HYPERLINK — flagged.
        assert!(p.runs[1].is_hyperlink);
        // The trailing run sees FieldEnd — no longer flagged.
        assert!(!p.runs[2].is_hyperlink);
    }

    #[test]
    fn all_tables_yields_nested_depths_in_document_order() {
        let s = parse_section(0, MINI_NESTED.as_bytes()).expect("parse ok");
        let depths: Vec<u32> = s.all_tables().map(|t| t.nesting_depth).collect();
        assert_eq!(depths, vec![0, 1]);
    }

    #[test]
    fn all_paragraphs_descends_into_cells() {
        let s = parse_section(0, MINI_NESTED.as_bytes()).expect("parse ok");
        // The document has 3 paragraphs in total: top-level (id=1),
        // inside outer cell (id=10), inside inner cell (id=20).
        assert_eq!(s.all_paragraphs().count(), 3);
    }
}
