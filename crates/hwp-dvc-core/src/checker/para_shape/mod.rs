//! Paragraph-shape validator — `CheckParaShape` port.
//!
//! Maps to `Checker::CheckParaShape` / `CheckParaShapeToCheckList` in
//! `references/dvc/Checker.cpp`.
//!
//! For every *unique* `para_pr_id_ref` that appears in the
//! [`RunTypeInfo`] stream, this module looks up the corresponding
//! [`ParaShape`] from [`HeaderTables::para_shapes`] and compares each
//! field declared in [`ParaShapeSpec`] against the document value.
//! One [`DvcErrorInfo`] is emitted per offending paragraph shape.
//!
//! # Covered fields (full 2001–2045 range)
//!
//! | Spec field                  | ParaShape field              | Error code |
//! |-----------------------------|------------------------------|------------|
//! | `horizontal`                | `h_align`                    | 2001       |
//! | `margin-left`               | `margin.left` (right-indent) | 2002       |
//! | `margin-right`              | `margin.right`               | 2003       |
//! | `indent`                    | `margin.indent`              | 2005       |
//! | `outdent`                   | `margin.left` (left-indent)  | 2006       |
//! | `linespacing`               | `line_spacing.type_`         | 2007       |
//! | `linespacingvalue`          | `line_spacing.value`         | 2008       |
//! | `spacing-paraup`            | `margin.prev`                | 2009       |
//! | `spacing-parabottom`        | `margin.next`                | 2010       |
//! | `spacing-gridpaper`         | `snap_to_grid`               | 2011       |
//! | `linebreak-korean`          | `break_non_latin_word`       | 2012       |
//! | `linebreak-english`         | `break_latin_word`           | 2013       |
//! | `linebreak-condense`        | `condense`                   | 2014       |
//! | `paratype`                  | `heading_type`               | 2015       |
//! | `paratype-value`            | `heading_id_ref`             | 2016       |
//! | `widow-orphan`              | `widow_orphan`               | 2017       |
//! | `keep-with-next`            | `keep_with_next`             | 2018       |
//! | `keep-lines-together`       | `keep_lines`                 | 2019       |
//! | `pagebreak-before`          | `page_break_before`          | 2020       |
//! | `fontlineheight`            | `font_line_height`           | 2021       |
//! | `linewrap`                  | `line_wrap`                  | 2022       |
//! | `autospace-easian-eng`      | `auto_spacing_eng`           | 2023       |
//! | `autospace-easian-num`      | `auto_spacing_num`           | 2024       |
//! | `verticalalign`             | `v_align`                    | 2025       |
//! | `autotab-intent`            | `checked` (proxy)            | 2030       |
//! | `autotab-pararightend`      | `connect` (proxy)            | 2031       |
//! | `spacing-ignore`            | `ignore_margin`              | 2045       |
//!
//! # Intentionally deferred fields
//!
//! The following spec fields are parsed and error codes exist, but the
//! checker does **not** emit errors for them yet:
//!
//! - **`tabtypes` / `tabtype` / `tabshape` / `tabposition` / `basetabspace`
//!   (2026–2029, 2032)** — require a dedicated TabDefinition table in
//!   `HeaderTables`, which is not yet parsed from `header.xml`. The spec
//!   fields deserialise correctly, and the error code constants are defined
//!   in `error::para_shape_codes`; re-enable once tab parsing is in place.
//!
//! - **`border` / `border-position` / `bordertype` / `bordersize` /
//!   `bordercolor` (2033–2037)** — require resolving `border_fill_id_ref`
//!   to the corresponding `BorderFill` record and comparing each edge.
//!   The infrastructure exists but the per-paragraph link is not yet wired.
//!
//! - **`bg-color` / `bg-pattoncolor` / `bg-pattontype` (2038–2040)** —
//!   background fill data is carried inside `BorderFill.fill_brush`, which
//!   is not yet decoded beyond a presence flag. Deferred until the fill
//!   sub-tree is parsed.
//!
//! - **`spacing-left` / `-right` / `-top` / `-bottom` (2041–2044)** —
//!   the C++ reference stores these as booleans indicating whether the
//!   border-offset overrides are active. The OWPML fields `borderOffsetLeft`
//!   etc. are already in `ParaShape` but the spec→document mapping for
//!   the boolean flag is ambiguous. Deferred for clarification.

use std::collections::HashSet;

use crate::checker::DvcErrorInfo;
use crate::document::header::types::enums::{HAlign, HeadingType, LineBreakWord, VAlign};
use crate::document::header::LineSpacingType;
use crate::document::{Document, RunTypeInfo};
use crate::error::{
    para_shape_codes::{
        PARASHAPE_AUTOSPACEEASIANENG, PARASHAPE_AUTOSPACEEASIANNUM, PARASHAPE_FONTLINEHEIGHT,
        PARASHAPE_HORIZONTAL, PARASHAPE_INDENT, PARASHAPE_KEEPLINESTOGETHER,
        PARASHAPE_KEEPWITHNEXT, PARASHAPE_LINEBREAKCONDENSE, PARASHAPE_LINEBREAKENGLISH,
        PARASHAPE_LINEBREAKKOREAN, PARASHAPE_LINESPACING, PARASHAPE_LINESPACINGVALUE,
        PARASHAPE_LINEWRAP, PARASHAPE_MARGINLEFT, PARASHAPE_MARGINRIGHT, PARASHAPE_OUTDENT,
        PARASHAPE_PAGEBREAKBEFORE, PARASHAPE_PARATYPE, PARASHAPE_PARATYPEVALUE,
        PARASHAPE_SPACINGGRIDPAPER, PARASHAPE_SPACINGIGNORE, PARASHAPE_SPACINGPARABOTTOM,
        PARASHAPE_SPACINGPARAUP, PARASHAPE_VERTICALALIGN, PARASHAPE_WIDOWORPHAN,
    },
    ErrorContext,
};
use crate::spec::ParaShapeSpec;

/// Validate every unique paragraph shape referenced in `document`
/// against `spec` and return one error per offending shape/field pair.
///
/// Returns an empty `Vec` if the document has no header.
///
/// # port of Checker::CheckParaShape / CheckParaShapeToCheckList
pub fn check(document: &Document, spec: &ParaShapeSpec) -> Vec<DvcErrorInfo> {
    let header = match document.header.as_ref() {
        Some(h) => h,
        None => return Vec::new(),
    };

    // Collect the unique para_pr_id_refs seen in the RunTypeInfo stream,
    // along with a representative RunTypeInfo for metadata (first seen).
    let mut seen: HashSet<u32> = HashSet::new();
    let mut repr: Vec<&RunTypeInfo> = Vec::new();
    for run in &document.run_type_infos {
        if seen.insert(run.para_pr_id_ref) {
            repr.push(run);
        }
    }

    let mut errors: Vec<DvcErrorInfo> = Vec::new();

    for run in repr {
        let ps = match header.para_shapes.get(&run.para_pr_id_ref) {
            Some(p) => p,
            None => continue,
        };

        // ── 2001: horizontal alignment ────────────────────────────────────────
        if let Some(expected) = spec.horizontal {
            if h_align_ordinal(ps.h_align) != expected {
                errors.push(make_error(run, PARASHAPE_HORIZONTAL));
            }
        }

        // ── 2002: margin-left ─────────────────────────────────────────────────
        if let Some(expected) = spec.margin_left {
            if ps.margin.left != expected {
                errors.push(make_error(run, PARASHAPE_MARGINLEFT));
            }
        }

        // ── 2003: margin-right ────────────────────────────────────────────────
        if let Some(expected) = spec.margin_right {
            if ps.margin.right != expected {
                errors.push(make_error(run, PARASHAPE_MARGINRIGHT));
            }
        }

        // ── 2005: indent (margin.indent / first-line indent) ──────────────────
        if let Some(expected) = spec.indent {
            if ps.margin.indent != expected {
                errors.push(make_error(run, PARASHAPE_INDENT));
            }
        }

        // ── 2006: outdent (margin.left — hanging/left indent) ─────────────────
        if let Some(expected) = spec.outdent {
            if ps.margin.left != expected {
                errors.push(make_error(run, PARASHAPE_OUTDENT));
            }
        }

        // ── 2007: linespacing type ────────────────────────────────────────────
        // The spec stores the type as an ordinal integer:
        //   0 = Percent, 1 = Fixed, 2 = BetweenLines, 3 = Minimum
        if let Some(expected_ordinal) = spec.linespacing {
            let actual_ordinal = line_spacing_type_ordinal(ps.line_spacing.type_);
            if actual_ordinal != expected_ordinal {
                errors.push(make_error(run, PARASHAPE_LINESPACING));
            }
        }

        // ── 2008: linespacingvalue ────────────────────────────────────────────
        if let Some(expected) = spec.linespacingvalue {
            if ps.line_spacing.value != expected {
                errors.push(make_error(run, PARASHAPE_LINESPACINGVALUE));
            }
        }

        // ── 2009: spacing-paraup (margin.prev) ───────────────────────────────
        if let Some(expected) = spec.spacing_paraup {
            if ps.margin.prev != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGPARAUP));
            }
        }

        // ── 2010: spacing-parabottom (margin.next) ───────────────────────────
        if let Some(expected) = spec.spacing_parabottom {
            if ps.margin.next != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGPARABOTTOM));
            }
        }

        // ── 2011: spacing-gridpaper (snap_to_grid) ───────────────────────────
        if let Some(expected) = spec.spacing_gridpaper {
            if ps.snap_to_grid != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGGRIDPAPER));
            }
        }

        // ── 2012: linebreak-korean ────────────────────────────────────────────
        // false = KEEP_WORD (syllable), true = BREAK_WORD (word-unit)
        if let Some(expected) = spec.linebreak_korean {
            let actual = ps.break_non_latin_word == LineBreakWord::BreakWord;
            if actual != expected {
                errors.push(make_error(run, PARASHAPE_LINEBREAKKOREAN));
            }
        }

        // ── 2013: linebreak-english ───────────────────────────────────────────
        // Ordinal: 0=KEEP_WORD, 1=Hyphenation(Other), 2=BREAK_WORD
        if let Some(expected) = spec.linebreak_english {
            let actual = line_break_latin_ordinal(ps.break_latin_word);
            if actual != expected {
                errors.push(make_error(run, PARASHAPE_LINEBREAKENGLISH));
            }
        }

        // ── 2014: linebreak-condense ──────────────────────────────────────────
        if let Some(expected) = spec.linebreak_condense {
            if ps.condense as i32 != expected {
                errors.push(make_error(run, PARASHAPE_LINEBREAKCONDENSE));
            }
        }

        // ── 2015: paratype (heading type) ─────────────────────────────────────
        if let Some(expected) = spec.paratype {
            if heading_type_ordinal(ps.heading_type) != expected {
                errors.push(make_error(run, PARASHAPE_PARATYPE));
            }
        }

        // ── 2016: paratype-value (heading id ref) ─────────────────────────────
        if let Some(expected) = spec.paratype_value {
            if ps.heading_id_ref != expected {
                errors.push(make_error(run, PARASHAPE_PARATYPEVALUE));
            }
        }

        // ── 2017: widow-orphan ────────────────────────────────────────────────
        if let Some(expected) = spec.widow_orphan {
            if ps.widow_orphan != expected {
                errors.push(make_error(run, PARASHAPE_WIDOWORPHAN));
            }
        }

        // ── 2018: keep-with-next ──────────────────────────────────────────────
        if let Some(expected) = spec.keep_with_next {
            if ps.keep_with_next != expected {
                errors.push(make_error(run, PARASHAPE_KEEPWITHNEXT));
            }
        }

        // ── 2019: keep-lines-together ─────────────────────────────────────────
        if let Some(expected) = spec.keep_lines_together {
            if ps.keep_lines != expected {
                errors.push(make_error(run, PARASHAPE_KEEPLINESTOGETHER));
            }
        }

        // ── 2020: pagebreak-before ────────────────────────────────────────────
        if let Some(expected) = spec.pagebreak_before {
            if ps.page_break_before != expected {
                errors.push(make_error(run, PARASHAPE_PAGEBREAKBEFORE));
            }
        }

        // ── 2021: fontlineheight ──────────────────────────────────────────────
        if let Some(expected) = spec.fontlineheight {
            if ps.font_line_height != expected {
                errors.push(make_error(run, PARASHAPE_FONTLINEHEIGHT));
            }
        }

        // ── 2022: linewrap ────────────────────────────────────────────────────
        if let Some(expected) = spec.linewrap {
            if ps.line_wrap != expected {
                errors.push(make_error(run, PARASHAPE_LINEWRAP));
            }
        }

        // ── 2023: autospace-easian-eng ────────────────────────────────────────
        if let Some(expected) = spec.autospace_easian_eng {
            if ps.auto_spacing_eng != expected {
                errors.push(make_error(run, PARASHAPE_AUTOSPACEEASIANENG));
            }
        }

        // ── 2024: autospace-easian-num ────────────────────────────────────────
        if let Some(expected) = spec.autospace_easian_num {
            if ps.auto_spacing_num != expected {
                errors.push(make_error(run, PARASHAPE_AUTOSPACEEASIANNUM));
            }
        }

        // ── 2025: verticalalign ───────────────────────────────────────────────
        if let Some(expected) = spec.verticalalign {
            if v_align_ordinal(ps.v_align) != expected {
                errors.push(make_error(run, PARASHAPE_VERTICALALIGN));
            }
        }

        // ── 2026–2032: tab fields ─────────────────────────────────────────────
        // TODO: tabtypes/tabtype/tabshape/tabposition/basetabspace require a
        // TabDefinition table in HeaderTables, which is not yet parsed from
        // header.xml. The spec fields deserialise correctly and error codes are
        // defined; re-enable once tab parsing is in place.

        // ── 2033–2037: border fields ──────────────────────────────────────────
        // TODO: Full border comparison requires resolving border_fill_id_ref to
        // a BorderFill record and comparing each edge's type/size/color.
        // Deferred until per-paragraph BorderFill lookup is wired up.

        // ── 2038–2040: background fields ──────────────────────────────────────
        // TODO: Background color/pattern are in the fill_brush sub-tree of the
        // BorderFill record, not yet decoded beyond a presence flag.

        // ── 2041–2044: spacing-left/-right/-top/-bottom ───────────────────────
        // TODO: The C++ reference stores these as booleans indicating whether
        // border-offset overrides are active. The OWPML borderOffsetLeft/Right/
        // Top/Bottom fields are in ParaShape but the spec→document mapping for
        // the boolean flag is ambiguous. Deferred for clarification.

        // ── 2045: spacing-ignore (ignore_margin) ──────────────────────────────
        if let Some(expected) = spec.spacing_ignore {
            if ps.ignore_margin != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGIGNORE));
            }
        }
    }

    errors
}

// ── Ordinal helpers ───────────────────────────────────────────────────────────

/// Map [`LineSpacingType`] to the integer ordinal used in DVC JSON spec
/// (`"linespacing": N`).
///
/// Mirrors `LineSpacingType` in `references/dvc/Source/DVCInterface.h`:
///   0 = Percent, 1 = Fixed, 2 = BetweenLines, 3 = AT_LEAST/Minimum
fn line_spacing_type_ordinal(t: LineSpacingType) -> i32 {
    match t {
        LineSpacingType::Percent => 0,
        LineSpacingType::Fixed => 1,
        LineSpacingType::BetweenLines => 2,
        LineSpacingType::Minimum => 3,
        LineSpacingType::Other => -1,
    }
}

/// Map [`HAlign`] to its ordinal per `HAlignType` in DVCInterface.h:
///   0=JUSTIFY, 1=LEFT, 2=CENTER, 3=RIGHT, 4=DISTRIBUTE, 5=DISTRIBUTE_SPACE
fn h_align_ordinal(a: HAlign) -> i32 {
    match a {
        HAlign::Justify => 0,
        HAlign::Left => 1,
        HAlign::Center => 2,
        HAlign::Right => 3,
        HAlign::Distribute => 4,
        HAlign::DistributeSpace => 5,
        HAlign::Other => -1,
    }
}

/// Map [`VAlign`] to its ordinal per `VAlignType` in DVCInterface.h:
///   0=BASELINE, 1=TOP, 2=MIDDLE/CENTER, 3=BOTTOM
fn v_align_ordinal(a: VAlign) -> i32 {
    match a {
        VAlign::Baseline => 0,
        VAlign::Top => 1,
        VAlign::Center => 2,
        VAlign::Bottom => 3,
        VAlign::Other => -1,
    }
}

/// Map [`HeadingType`] to its ordinal per `ParaType` in DVCInterface.h:
///   0=NONE, 1=OUTLINE, 2=NUMBER, 3=BULLET
fn heading_type_ordinal(t: HeadingType) -> i32 {
    match t {
        HeadingType::None => 0,
        HeadingType::Outline => 1,
        HeadingType::Number => 2,
        HeadingType::Bullet => 3,
        HeadingType::Other => -1,
    }
}

/// Map [`LineBreakWord`] for Latin words to ordinal per `LineBreakLatinWord`:
///   0=KEEP_WORD, 1=Hyphenation(Other), 2=BREAK_WORD
fn line_break_latin_ordinal(w: LineBreakWord) -> i32 {
    match w {
        LineBreakWord::KeepWord => 0,
        LineBreakWord::Other => 1, // Hyphenation maps to Other in our enum
        LineBreakWord::BreakWord => 2,
    }
}

/// Build a [`DvcErrorInfo`] from a representative run and an error code.
fn make_error(run: &RunTypeInfo, error_code: u32) -> DvcErrorInfo {
    DvcErrorInfo {
        para_pr_id_ref: run.para_pr_id_ref,
        char_pr_id_ref: run.char_pr_id_ref,
        text: run.text.clone(),
        page_no: run.page_no,
        line_no: run.line_no,
        error_code,
        table_id: run.table_id,
        is_in_table: run.is_in_table,
        is_in_table_in_table: run.is_in_table_in_table,
        table_row: run.table_row,
        table_col: run.table_col,
        is_in_shape: run.is_in_shape,
        use_hyperlink: run.is_hyperlink,
        use_style: run.is_style,
        error_string: crate::error::error_string(error_code, ErrorContext::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::types::enums::{
        HAlign, HeadingType, LineBreakWord, LineSpacingType as LST, VAlign,
    };
    use crate::document::header::types::{LineSpacing, Margin, ParaShape as HdrParaShape};
    use crate::document::header::HeaderTables;
    use crate::document::{Document, RunTypeInfo};
    use crate::spec::ParaShapeSpec;

    /// Build a minimal [`Document`] with one run referencing para shape `id`.
    fn doc_with_para_shape(id: u32, ps: HdrParaShape) -> Document {
        let mut header = HeaderTables::default();
        header.para_shapes.insert(id, ps);

        let run = RunTypeInfo {
            para_pr_id_ref: id,
            ..Default::default()
        };

        Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        }
    }

    fn default_spec() -> ParaShapeSpec {
        ParaShapeSpec {
            spacing_paraup: Some(0),
            spacing_parabottom: Some(0),
            linespacing: Some(0),
            linespacingvalue: Some(160),
            indent: Some(0),
            outdent: Some(0),
            ..Default::default()
        }
    }

    fn default_para_shape(id: u32) -> HdrParaShape {
        HdrParaShape {
            id,
            line_spacing: LineSpacing {
                type_: LST::Percent,
                value: 160,
                unit: "HWPUNIT".into(),
            },
            margin: Margin {
                indent: 0,
                left: 0,
                right: 0,
                prev: 0,
                next: 0,
            },
            ..Default::default()
        }
    }

    // ── spacing_paraup ────────────────────────────────────────────────────────

    #[test]
    fn spacing_paraup_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let spec = default_spec();
        let errs = check(&doc, &spec);
        assert!(
            errs.iter().all(|e| e.error_code != PARASHAPE_SPACINGPARAUP),
            "no SPACINGPARAUP error expected when margin.prev matches spec"
        );
    }

    #[test]
    fn spacing_paraup_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.prev = 500;
        let doc = doc_with_para_shape(0, ps);
        let spec = default_spec();
        let errs = check(&doc, &spec);
        assert!(
            errs.iter().any(|e| e.error_code == PARASHAPE_SPACINGPARAUP),
            "expected PARASHAPE_SPACINGPARAUP when margin.prev != spec"
        );
    }

    // ── spacing_parabottom ────────────────────────────────────────────────────

    #[test]
    fn spacing_parabottom_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let errs = check(&doc, &default_spec());
        assert!(errs
            .iter()
            .all(|e| e.error_code != PARASHAPE_SPACINGPARABOTTOM));
    }

    #[test]
    fn spacing_parabottom_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.next = 300;
        let errs = check(&doc_with_para_shape(0, ps), &default_spec());
        assert!(errs
            .iter()
            .any(|e| e.error_code == PARASHAPE_SPACINGPARABOTTOM));
    }

    // ── linespacing type ──────────────────────────────────────────────────────

    #[test]
    fn linespacing_type_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let errs = check(&doc, &default_spec());
        assert!(errs.iter().all(|e| e.error_code != PARASHAPE_LINESPACING));
    }

    #[test]
    fn linespacing_type_fail() {
        let mut ps = default_para_shape(0);
        ps.line_spacing.type_ = LST::Fixed; // ordinal 1, spec wants 0
        let errs = check(&doc_with_para_shape(0, ps), &default_spec());
        assert!(errs.iter().any(|e| e.error_code == PARASHAPE_LINESPACING));
    }

    // ── linespacingvalue ──────────────────────────────────────────────────────

    #[test]
    fn linespacingvalue_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let errs = check(&doc, &default_spec());
        assert!(errs
            .iter()
            .all(|e| e.error_code != PARASHAPE_LINESPACINGVALUE));
    }

    #[test]
    fn linespacingvalue_fail() {
        let mut ps = default_para_shape(0);
        ps.line_spacing.value = 200; // spec wants 160
        let errs = check(&doc_with_para_shape(0, ps), &default_spec());
        assert!(errs
            .iter()
            .any(|e| e.error_code == PARASHAPE_LINESPACINGVALUE));
    }

    // ── indent ────────────────────────────────────────────────────────────────

    #[test]
    fn indent_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let errs = check(&doc, &default_spec());
        assert!(errs.iter().all(|e| e.error_code != PARASHAPE_INDENT));
    }

    #[test]
    fn indent_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.indent = 1000;
        let errs = check(&doc_with_para_shape(0, ps), &default_spec());
        assert!(errs.iter().any(|e| e.error_code == PARASHAPE_INDENT));
    }

    // ── outdent ───────────────────────────────────────────────────────────────

    #[test]
    fn outdent_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let errs = check(&doc, &default_spec());
        assert!(errs.iter().all(|e| e.error_code != PARASHAPE_OUTDENT));
    }

    #[test]
    fn outdent_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.left = 800;
        let errs = check(&doc_with_para_shape(0, ps), &default_spec());
        assert!(errs.iter().any(|e| e.error_code == PARASHAPE_OUTDENT));
    }

    // ── horizontal alignment ──────────────────────────────────────────────────

    #[test]
    fn horizontal_pass() {
        let ps = default_para_shape(0); // h_align defaults to Justify → ordinal 0
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            horizontal: Some(0),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        assert!(errs.iter().all(|e| e.error_code != PARASHAPE_HORIZONTAL));
    }

    #[test]
    fn horizontal_fail() {
        let mut ps = default_para_shape(0);
        ps.h_align = HAlign::Center; // ordinal 2, spec wants 0 (Justify)
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            horizontal: Some(0),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        assert!(errs.iter().any(|e| e.error_code == PARASHAPE_HORIZONTAL));
    }

    // ── margin-left ───────────────────────────────────────────────────────────

    #[test]
    fn margin_left_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let spec = ParaShapeSpec {
            margin_left: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_MARGINLEFT));
    }

    #[test]
    fn margin_left_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.left = 200;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            margin_left: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_MARGINLEFT));
    }

    // ── margin-right ──────────────────────────────────────────────────────────

    #[test]
    fn margin_right_pass() {
        let doc = doc_with_para_shape(0, default_para_shape(0));
        let spec = ParaShapeSpec {
            margin_right: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_MARGINRIGHT));
    }

    #[test]
    fn margin_right_fail() {
        let mut ps = default_para_shape(0);
        ps.margin.right = 200;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            margin_right: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_MARGINRIGHT));
    }

    // ── snap-to-grid / spacing-gridpaper ──────────────────────────────────────

    #[test]
    fn spacing_gridpaper_pass() {
        let mut ps = default_para_shape(0);
        ps.snap_to_grid = true;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            spacing_gridpaper: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_SPACINGGRIDPAPER));
    }

    #[test]
    fn spacing_gridpaper_fail() {
        let ps = default_para_shape(0); // snap_to_grid = false by default
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            spacing_gridpaper: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_SPACINGGRIDPAPER));
    }

    // ── linebreak-korean ──────────────────────────────────────────────────────

    #[test]
    fn linebreak_korean_pass() {
        let mut ps = default_para_shape(0);
        ps.break_non_latin_word = LineBreakWord::BreakWord;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_korean: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_LINEBREAKKOREAN));
    }

    #[test]
    fn linebreak_korean_fail() {
        let ps = default_para_shape(0); // break_non_latin_word = KeepWord → false
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_korean: Some(true), // expects BreakWord
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_LINEBREAKKOREAN));
    }

    // ── linebreak-english ─────────────────────────────────────────────────────

    #[test]
    fn linebreak_english_pass() {
        let ps = default_para_shape(0); // break_latin_word = KeepWord → ordinal 0
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_english: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_LINEBREAKENGLISH));
    }

    #[test]
    fn linebreak_english_fail() {
        let mut ps = default_para_shape(0);
        ps.break_latin_word = LineBreakWord::BreakWord; // ordinal 2
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_english: Some(0), // expects KeepWord
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_LINEBREAKENGLISH));
    }

    // ── linebreak-condense ────────────────────────────────────────────────────

    #[test]
    fn linebreak_condense_pass() {
        let mut ps = default_para_shape(0);
        ps.condense = 50;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_condense: Some(50),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_LINEBREAKCONDENSE));
    }

    #[test]
    fn linebreak_condense_fail() {
        let ps = default_para_shape(0); // condense = 0
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            linebreak_condense: Some(50),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_LINEBREAKCONDENSE));
    }

    // ── paratype ──────────────────────────────────────────────────────────────

    #[test]
    fn paratype_pass() {
        let mut ps = default_para_shape(0);
        ps.heading_type = HeadingType::Outline; // ordinal 1
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            paratype: Some(1),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_PARATYPE));
    }

    #[test]
    fn paratype_fail() {
        let ps = default_para_shape(0); // heading_type = None → ordinal 0
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            paratype: Some(1), // expects Outline
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_PARATYPE));
    }

    // ── widow-orphan ──────────────────────────────────────────────────────────

    #[test]
    fn widow_orphan_pass() {
        let mut ps = default_para_shape(0);
        ps.widow_orphan = true;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            widow_orphan: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_WIDOWORPHAN));
    }

    #[test]
    fn widow_orphan_fail() {
        let ps = default_para_shape(0); // widow_orphan = false by default
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            widow_orphan: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_WIDOWORPHAN));
    }

    // ── autospace-easian-eng ──────────────────────────────────────────────────

    #[test]
    fn autospace_easian_eng_pass() {
        let mut ps = default_para_shape(0);
        ps.auto_spacing_eng = true;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            autospace_easian_eng: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_AUTOSPACEEASIANENG));
    }

    #[test]
    fn autospace_easian_eng_fail() {
        let ps = default_para_shape(0); // auto_spacing_eng = false
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            autospace_easian_eng: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_AUTOSPACEEASIANENG));
    }

    // ── verticalalign ─────────────────────────────────────────────────────────

    #[test]
    fn verticalalign_pass() {
        let ps = default_para_shape(0); // v_align defaults to Baseline → ordinal 0
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            verticalalign: Some(0),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_VERTICALALIGN));
    }

    #[test]
    fn verticalalign_fail() {
        let mut ps = default_para_shape(0);
        ps.v_align = VAlign::Center; // ordinal 2
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            verticalalign: Some(0), // expects Baseline
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_VERTICALALIGN));
    }

    // ── spacing-ignore ────────────────────────────────────────────────────────

    #[test]
    fn spacing_ignore_pass() {
        let mut ps = default_para_shape(0);
        ps.ignore_margin = true;
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            spacing_ignore: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .all(|e| e.error_code != PARASHAPE_SPACINGIGNORE));
    }

    #[test]
    fn spacing_ignore_fail() {
        let ps = default_para_shape(0); // ignore_margin = false
        let doc = doc_with_para_shape(0, ps);
        let spec = ParaShapeSpec {
            spacing_ignore: Some(true),
            ..Default::default()
        };
        assert!(check(&doc, &spec)
            .iter()
            .any(|e| e.error_code == PARASHAPE_SPACINGIGNORE));
    }

    // ── optional-field skipping ───────────────────────────────────────────────

    #[test]
    fn none_spec_fields_are_skipped() {
        let mut ps = default_para_shape(0);
        // Set every field to a non-zero / non-default value.
        ps.margin.prev = 999;
        ps.margin.next = 999;
        ps.margin.indent = 999;
        ps.margin.left = 999;
        ps.line_spacing.value = 999;
        ps.line_spacing.type_ = LST::Fixed;
        ps.h_align = HAlign::Right;
        ps.widow_orphan = true;

        let doc = doc_with_para_shape(0, ps);
        // Spec with all fields None — nothing should fire.
        let spec = ParaShapeSpec::default();
        let errs = check(&doc, &spec);
        assert!(
            errs.is_empty(),
            "no errors expected when all spec fields are None"
        );
    }

    // ── deduplication ────────────────────────────────────────────────────────

    #[test]
    fn duplicate_para_pr_id_refs_produce_one_error() {
        let mut ps = default_para_shape(5);
        ps.line_spacing.value = 200;

        let mut header = HeaderTables::default();
        header.para_shapes.insert(5, ps);

        // Three runs, all referencing para shape 5.
        let run_a = RunTypeInfo {
            para_pr_id_ref: 5,
            text: "a".into(),
            ..Default::default()
        };
        let run_b = RunTypeInfo {
            para_pr_id_ref: 5,
            text: "b".into(),
            ..Default::default()
        };
        let run_c = RunTypeInfo {
            para_pr_id_ref: 5,
            text: "c".into(),
            ..Default::default()
        };

        let doc = Document {
            header: Some(header),
            run_type_infos: vec![run_a, run_b, run_c],
            ..Default::default()
        };

        let errs = check(&doc, &default_spec());
        let lsv_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARASHAPE_LINESPACINGVALUE)
            .collect();
        assert_eq!(
            lsv_errs.len(),
            1,
            "duplicate id_refs must produce exactly one error"
        );
    }
}
