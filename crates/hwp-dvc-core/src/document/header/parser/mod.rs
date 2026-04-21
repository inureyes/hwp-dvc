//! Event-driven parser for `Contents/header.xml`.
//!
//! The parser is deliberately tolerant of the `xmlns`-prefixed names
//! that HWPX writers emit (`hh:charPr`, `hh:paraPr`, `hh:style`, …).
//! We match on the **local name** (the part after the colon) and
//! ignore the namespace declaration attributes. Namespaces aren't
//! structurally meaningful for HWPX's header because the full prefix
//! set is declared once at `<hh:head>`.
//!
//! Every unrecognized element is skipped with
//! [`Reader::read_to_end_into`]-equivalent skipping. This keeps the
//! parser forward-compatible: Hancom has already shipped at least two
//! minor revisions that added attributes to existing elements without
//! bumping the declared `version`.
//!
//! The parser is split across one file per top-level element group
//! so that no single file exceeds the project's 500-line soft cap.

mod border_fills;
mod char_shapes;
mod common;
mod fontfaces;
mod misc;
mod para_shapes;

use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::DvcResult;

use super::HeaderTables;
use common::local_name;

/// Parse an HWPX `header.xml` byte slice into [`HeaderTables`].
pub fn parse_header(bytes: &[u8]) -> DvcResult<HeaderTables> {
    let mut reader = Reader::from_reader(bytes);
    let config = reader.config_mut();
    config.trim_text(true);
    config.expand_empty_elements = false;

    let mut tables = HeaderTables::default();

    dispatch(&mut reader, &mut tables)?;

    Ok(tables)
}

fn dispatch<B: BufRead>(reader: &mut Reader<B>, tables: &mut HeaderTables) -> DvcResult<()> {
    let mut buf = Vec::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Start(ref e) => match local_name(e.name()) {
                b"fontfaces" => fontfaces::parse(reader, &mut tables.font_faces)?,
                b"borderFills" => border_fills::parse(reader, tables)?,
                b"charProperties" => char_shapes::parse(reader, tables)?,
                b"paraProperties" => para_shapes::parse(reader, tables)?,
                b"styles" => misc::parse_styles(reader, tables)?,
                b"bullets" => misc::parse_bullets(reader, tables)?,
                b"numberings" => misc::parse_numberings(reader, tables)?,
                _ => {}
            },
            Event::Empty(_) => {
                // Element with no children at top level — ignore.
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::types::{
        FontLang, HAlign, HeadingType, LineBreakWord, LineSpacingType, LineType, VAlign,
    };

    /// A minimal, synthetic `header.xml` covering the shape categories
    /// our parser targets. This is unit-level coverage complementing
    /// the fixture-based integration tests in
    /// `tests/header_parsing.rs`.
    const MINI_HEADER: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head"
         xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
         xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
<hh:refList>
<hh:fontfaces itemCnt="1">
<hh:fontface lang="HANGUL" fontCnt="1">
<hh:font id="0" face="TestFont" type="TTF" isEmbedded="0"/>
</hh:fontface>
</hh:fontfaces>
<hh:borderFills itemCnt="1">
<hh:borderFill id="1" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0">
<hh:slash type="NONE" Crooked="0" isCounter="0"/>
<hh:backSlash type="NONE" Crooked="0" isCounter="0"/>
<hh:leftBorder type="SOLID" width="0.12 mm" color="#000000"/>
<hh:rightBorder type="SOLID" width="0.12 mm" color="#000000"/>
<hh:topBorder type="SOLID" width="0.12 mm" color="#000000"/>
<hh:bottomBorder type="DASH" width="0.12 mm" color="#000000"/>
<hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/>
</hh:borderFill>
</hh:borderFills>
<hh:charProperties itemCnt="1">
<hh:charPr id="0" height="1000" textColor="#000000" shadeColor="none" useFontSpace="0" useKerning="0" symMark="NONE" borderFillIDRef="1">
<hh:fontRef hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>
<hh:ratio hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/>
<hh:spacing hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>
<hh:relSz hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/>
<hh:offset hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>
</hh:charPr>
</hh:charProperties>
<hh:paraProperties itemCnt="1">
<hh:paraPr id="0" tabPrIDRef="0" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0">
<hh:align horizontal="JUSTIFY" vertical="BASELINE"/>
<hh:heading type="NONE" idRef="0" level="0"/>
<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/>
<hh:autoSpacing eAsianEng="0" eAsianNum="0"/>
<hh:margin><hc:indent value="0" unit="HWPUNIT"/><hc:left value="0" unit="HWPUNIT"/><hc:right value="0" unit="HWPUNIT"/><hc:prev value="0" unit="HWPUNIT"/><hc:next value="0" unit="HWPUNIT"/></hh:margin>
<hh:lineSpacing type="PERCENT" value="160" unit="HWPUNIT"/>
<hh:border borderFillIDRef="1" offsetLeft="0" offsetRight="0" offsetTop="0" offsetBottom="0" connect="0" ignoreMargin="0"/>
</hh:paraPr>
</hh:paraProperties>
<hh:styles itemCnt="1">
<hh:style id="0" type="PARA" name="바탕글" engName="Normal" paraPrIDRef="0" charPrIDRef="0" nextStyleIDRef="0" langID="1042" lockForm="0"/>
</hh:styles>
<hh:bullets itemCnt="1">
<hh:bullet id="1" char="&#x25A1;" useImage="0"/>
</hh:bullets>
<hh:numberings itemCnt="1">
<hh:numbering id="1" start="0">
<hh:paraHead start="1" level="1" align="LEFT" useInstWidth="1" autoIndent="1" widthAdjust="0" textOffsetType="PERCENT" textOffset="50" numFormat="DIGIT" charPrIDRef="4294967295" checkable="0">^1.</hh:paraHead>
</hh:numbering>
</hh:numberings>
</hh:refList>
</hh:head>"##;

    #[test]
    fn parses_minimal_synthetic_header() {
        let t = parse_header(MINI_HEADER.as_bytes()).expect("parse ok");

        assert_eq!(t.font_faces.len(), 1);
        assert_eq!(t.font_faces[0].lang, FontLang::Hangul);
        assert_eq!(
            t.font_faces[0].fonts.get(&0).map(String::as_str),
            Some("TestFont")
        );

        let bf = t.border_fills.get(&1).expect("border_fill id=1");
        assert_eq!(bf.left.line_type, LineType::Solid);
        assert!((bf.left.width_mm - 0.12).abs() < 1e-6);
        assert_eq!(bf.bottom.line_type, LineType::Dash);
        assert!(
            !bf.four_sides_solid(),
            "bottom is DASH, so not all four sides solid"
        );

        let cs = t.char_shapes.get(&0).expect("charPr id=0");
        assert_eq!(cs.height, 1000);
        assert_eq!(cs.text_color, "#000000");
        assert_eq!(cs.border_fill_id_ref, 1);
        assert_eq!(cs.ratio.get(FontLang::Hangul), 100);
        assert_eq!(
            cs.font_name(FontLang::Hangul, &t.font_faces),
            Some("TestFont")
        );

        let ps = t.para_shapes.get(&0).expect("paraPr id=0");
        assert_eq!(ps.h_align, HAlign::Justify);
        assert_eq!(ps.v_align, VAlign::Baseline);
        assert_eq!(ps.heading_type, HeadingType::None);
        assert_eq!(ps.break_latin_word, LineBreakWord::KeepWord);
        assert_eq!(ps.line_spacing.type_, LineSpacingType::Percent);
        assert_eq!(ps.line_spacing.value, 160);
        assert_eq!(ps.border_fill_id_ref, 1);

        let st = t.styles.get(&0).expect("style id=0");
        assert_eq!(st.name, "바탕글");
        assert_eq!(st.style_type, "PARA");

        let b = t.bullets.get(&1).expect("bullet id=1");
        assert_eq!(b.char, "\u{25A1}");
        assert!(!b.use_image);

        let n = t.numberings.get(&1).expect("numbering id=1");
        assert_eq!(n.para_heads.len(), 1);
        assert_eq!(n.para_heads[0].level, 1);
        assert_eq!(n.para_heads[0].num_format, "DIGIT");
        assert_eq!(n.para_heads[0].num_format_text, "^1.");
    }

    #[test]
    fn missing_header_elements_produce_defaults() {
        // A head with no refList at all should still parse.
        let xml = r#"<?xml version="1.0"?><hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head"></hh:head>"#;
        let t = parse_header(xml.as_bytes()).expect("parse empty head");
        assert!(t.char_shapes.is_empty());
        assert!(t.font_faces.is_empty());
    }

    #[test]
    fn parse_width_mm_strips_unit() {
        use super::common::parse_width_mm;
        assert!((parse_width_mm("0.12 mm") - 0.12).abs() < 1e-6);
        assert!((parse_width_mm("1.5mm") - 1.5).abs() < 1e-6);
        assert!((parse_width_mm("3") - 3.0).abs() < 1e-6);
        assert_eq!(parse_width_mm(""), 0.0);
    }
}
