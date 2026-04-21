//! Paragraph-shape validator — `CheckParaShape` port.
//!
//! Maps to `Checker::CheckParaShape` / `CheckParaShapeToCheckList` in
//! `references/dvc/Source/Checker.cpp`.
//!
//! For every *unique* `para_pr_id_ref` that appears in the
//! [`RunTypeInfo`] stream, this module looks up the corresponding
//! [`ParaShape`] from [`HeaderTables::para_shapes`] and compares each
//! field declared in [`ParaShapeSpec`] against the document value.
//! One [`DvcErrorInfo`] is emitted per offending paragraph shape.
//!
//! # Covered fields
//!
//! | Spec field            | ParaShape field            | Error code                  |
//! |-----------------------|----------------------------|-----------------------------|
//! | `indent`              | `margin.indent`            | `PARASHAPE_INDENT` (2005)   |
//! | `outdent`             | `margin.left`              | `PARASHAPE_OUTDENT` (2006)  |
//! | `linespacing`         | `line_spacing.type_`       | `PARASHAPE_LINESPACING` (2007) |
//! | `linespacingvalue`    | `line_spacing.value`       | `PARASHAPE_LINESPACINGVALUE` (2008) |
//! | `spacing-paraup`      | `margin.prev`              | `PARASHAPE_SPACINGPARAUP` (2009) |
//! | `spacing-parabottom`  | `margin.next`              | `PARASHAPE_SPACINGPARABOTTOM` (2010) |
//!
//! # TODO — horizontal alignment, margins, border
//!
//! The following fields from the reference `JID_PARA_SHAPE_*` constants
//! are not yet validated. Each maps to an OWPML field:
//!
//! - `JID_PARA_SHAPE_HORIZONTAL` — `ParaShape.h_align`
//! - `JID_PARA_SHAPE_LEFT_MARGIN` — `ParaShape.margin.left`
//! - `JID_PARA_SHAPE_RIGHT_MARGIN` — `ParaShape.margin.right`
//! - `JID_PARA_SHAPE_FIRSTLINE` (2004) — first-line indent (currently
//!   the same as `margin.indent` in this parser; OWPML distinguishes them
//!   via `<hc:indent>` vs `<hc:intent>`).

use std::collections::HashSet;

use crate::checker::DvcErrorInfo;
use crate::document::header::LineSpacingType;
use crate::document::{Document, RunTypeInfo};
use crate::error::{
    para_shape_codes::{
        PARASHAPE_INDENT, PARASHAPE_LINESPACING, PARASHAPE_LINESPACINGVALUE, PARASHAPE_OUTDENT,
        PARASHAPE_SPACINGPARABOTTOM, PARASHAPE_SPACINGPARAUP,
    },
    ErrorContext,
};
use crate::spec::ParaShapeSpec;

/// Validate every unique paragraph shape referenced in `document`
/// against `spec` and return one error per offending shape/field pair.
///
/// Returns an empty `Vec` if `spec` is `None` (the caller already guards
/// before calling this function, but the signature accepts `Option` to
/// make wiring ergonomic).
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
        let para_shape = match header.para_shapes.get(&run.para_pr_id_ref) {
            Some(ps) => ps,
            None => continue,
        };

        // --- spacing-paraup (margin.prev) ---
        if let Some(expected) = spec.spacing_paraup {
            if para_shape.margin.prev != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGPARAUP));
            }
        }

        // --- spacing-parabottom (margin.next) ---
        if let Some(expected) = spec.spacing_parabottom {
            if para_shape.margin.next != expected {
                errors.push(make_error(run, PARASHAPE_SPACINGPARABOTTOM));
            }
        }

        // --- linespacing type ---
        // The spec stores the type as an ordinal integer:
        //   0 = Percent, 1 = Fixed, 2 = BetweenLines, 3 = Minimum
        if let Some(expected_ordinal) = spec.linespacing {
            let actual_ordinal = line_spacing_type_ordinal(para_shape.line_spacing.type_);
            if actual_ordinal != expected_ordinal {
                errors.push(make_error(run, PARASHAPE_LINESPACING));
            }
        }

        // --- linespacingvalue ---
        if let Some(expected) = spec.linespacingvalue {
            if para_shape.line_spacing.value != expected {
                errors.push(make_error(run, PARASHAPE_LINESPACINGVALUE));
            }
        }

        // --- indent (margin.indent / first-line indent) ---
        if let Some(expected) = spec.indent {
            if para_shape.margin.indent != expected {
                errors.push(make_error(run, PARASHAPE_INDENT));
            }
        }

        // --- outdent (margin.left — hanging/left indent) ---
        if let Some(expected) = spec.outdent {
            if para_shape.margin.left != expected {
                errors.push(make_error(run, PARASHAPE_OUTDENT));
            }
        }
    }

    errors
}

/// Convert a [`LineSpacingType`] variant to the integer ordinal used
/// in the DVC JSON spec (`"linespacing": N`).
fn line_spacing_type_ordinal(t: LineSpacingType) -> i32 {
    match t {
        LineSpacingType::Percent => 0,
        LineSpacingType::Fixed => 1,
        LineSpacingType::BetweenLines => 2,
        LineSpacingType::Minimum => 3,
        LineSpacingType::Other => -1,
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
    use crate::document::header::types::{LineSpacing, Margin, ParaShape as HdrParaShape};
    use crate::document::header::{HeaderTables, LineSpacingType as LST};
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

    // --- spacing_paraup ---

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

    // --- spacing_parabottom ---

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

    // --- linespacing type ---

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

    // --- linespacingvalue ---

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

    // --- indent ---

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

    // --- outdent ---

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

    // --- optional-field skipping ---

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

        let doc = doc_with_para_shape(0, ps);
        // Spec with all fields None — nothing should fire.
        let spec = ParaShapeSpec::default();
        let errs = check(&doc, &spec);
        assert!(
            errs.is_empty(),
            "no errors expected when all spec fields are None"
        );
    }

    // --- deduplication ---

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
