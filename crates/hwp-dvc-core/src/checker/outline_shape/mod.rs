//! `CheckOutlineShape` validator — mirrors `Checker::CheckOutlineShape` and
//! `CheckOutlineParaHeadToCheckList` / `getLevelType` from the reference C++
//! implementation (`references/dvc/Checker.cpp`).
//!
//! # Logic
//!
//! For every paragraph shape in the document that participates in an outline
//! (i.e. its [`ParaShape::heading_type`] is [`HeadingType::Outline`]), look
//! up the corresponding [`Numbering`] entry (via
//! `ParaShape::heading_id_ref`) and the specific level (`heading_level`).
//! From the [`ParaHead`] at that level, compare:
//!
//! - `num_format`      → `LevelType::numbershape` (via ordinal mapping)
//! - `num_format_text` → `LevelType::numbertype` (template string, when present)
//!
//! One [`DvcErrorInfo`] is emitted per (run, mismatched field) pair.
//!
//! # Error codes
//!
//! | Mismatch              | Code                          |
//! |-----------------------|-------------------------------|
//! | `numbertype` template | `OUTLINESHAPE_LEVEL_NUMBERTYPE` (3206) |
//! | `numbershape` enum    | `OUTLINESHAPE_LEVEL_NUMBERSHAPE` (3207) |
//!
//! `OUTLINESHAPE_TYPE` (3201) is reserved for a higher-level shape-name
//! check (matching the named outline shape template, e.g. "OUTLINE_NAME1")
//! that the current `OutlineShapeSpec` does not surface — no `type` field
//! is present in the parsed spec.
//!
//! # Deduplication
//!
//! Like `CheckParaShape`, the validator deduplicates by `para_pr_id_ref`:
//! each unique paragraph shape is checked at most once, and the first
//! matching run in the stream is used as the representative for error
//! metadata (page, line, char shape, etc.).
//!
//! # Gap / TODO
//!
//! The `Document` AST does not currently expose `heading_type`,
//! `heading_id_ref`, or `heading_level` on individual [`RunTypeInfo`]
//! records. Those fields are on [`ParaShape`] (the header-side shape), which
//! is reachable via `run.para_pr_id_ref → header.para_shapes`.  This
//! validator follows the same pattern as `CheckParaShape`: it looks up the
//! resolved `ParaShape` from the header tables and reads the heading fields
//! there.  No `RunTypeInfo` changes are required.
//!
//! [`ParaShape`]: crate::document::header::ParaShape
//! [`ParaHead`]: crate::document::header::ParaHead
//! [`Numbering`]: crate::document::header::Numbering
//! [`HeadingType`]: crate::document::header::types::enums::HeadingType

use std::collections::HashSet;

use crate::checker::DvcErrorInfo;
use crate::document::header::types::enums::HeadingType;
use crate::document::{Document, RunTypeInfo};
use crate::error::outline_shape_codes::{
    OUTLINESHAPE_LEVEL_NUMBERSHAPE, OUTLINESHAPE_LEVEL_NUMBERTYPE,
};
use crate::spec::OutlineShapeSpec;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Validate outline numbering shapes for every unique outline paragraph in
/// `document` against `spec`.
///
/// Returns an empty `Vec` when:
/// - the spec has no `leveltype` entries, or
/// - no paragraph uses outline numbering, or
/// - all outline paragraphs match the spec.
#[must_use]
pub fn check(document: &Document, spec: &OutlineShapeSpec) -> Vec<DvcErrorInfo> {
    if spec.leveltype.is_empty() {
        return Vec::new();
    }

    let header = match document.header.as_ref() {
        Some(h) => h,
        None => return Vec::new(),
    };

    // Deduplicate by `para_pr_id_ref`, keeping the first-seen run as the
    // representative for error metadata — same pattern as `CheckParaShape`.
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

        // Only outline paragraphs participate in this check.
        if para_shape.heading_type != HeadingType::Outline {
            continue;
        }

        // For outline headings, the section-level `outlineShapeIDRef` (stored
        // on each RunTypeInfo as `outline_shape_id_ref`) is the numbering-table
        // key. The para shape's `heading_id_ref` is 0 for outline paragraphs
        // that inherit the section-wide outline shape — using it directly would
        // miss the numbering.  The run carries the correct resolved id.
        let numbering = match header.numberings.get(&run.outline_shape_id_ref) {
            Some(n) => n,
            None => continue,
        };

        // OWPML `<hh:heading level="N"/>` is 0-indexed; `<hh:paraHead level="N"/>`
        // is 1-indexed. Add 1 to look up the right ParaHead.
        let para_level = para_shape.heading_level + 1; // 1-indexed level
        let para_head = match numbering.by_level(para_level) {
            Some(ph) => ph,
            None => continue,
        };

        // Find the spec entry for this level.
        let spec_entry = match spec.leveltype.iter().find(|lt| lt.level == para_level) {
            Some(lt) => lt,
            None => continue,
        };

        // --- numbertype (template string) ---
        // Only checked when the spec supplies a `numbertype` value.
        if let Some(expected_type) = &spec_entry.numbertype {
            if &para_head.num_format_text != expected_type {
                errors.push(make_error(run, OUTLINESHAPE_LEVEL_NUMBERTYPE));
            }
        }

        // --- numbershape (enum ordinal → num_format string) ---
        let expected_shape_str = num_shape_ordinal_to_str(spec_entry.numbershape);
        if para_head.num_format != expected_shape_str {
            errors.push(make_error(run, OUTLINESHAPE_LEVEL_NUMBERSHAPE));
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a `numbershape` ordinal (from the DVC JSON spec) to the OWPML
/// `numFormat` string stored in [`ParaHead::num_format`].
///
/// The ordinal enumeration mirrors the C++ `NumShapeType` enum:
///
/// ```text
/// 0  = DIGIT
/// 1  = CIRCLED_DIGIT
/// 2  = ROMAN_CAPITAL
/// 3  = ROMAN_SMALL
/// 4  = LATIN_CAPITAL
/// 5  = LATIN_SMALL
/// 6  = CIRCLED_LATIN_CAPITAL
/// 7  = CIRCLED_LATIN_SMALL
/// 8  = HANGUL_SYLLABLE
/// 9  = CIRCLED_HANGUL_SYLLABLE
/// 10 = HANGUL_JAMO
/// 11 = CIRCLED_HANGUL_JAMO
/// 12 = HANGUL_PHONETIC
/// 13 = IDEOGRAPH
/// 14 = CIRCLED_IDEOGRAPH
/// 15 = DECAGON_CIRCLE
/// 16 = DECAGON_CIRCLE_HANJA
/// ```
///
/// Unknown ordinals map to `""` so the `!=` comparison that follows will
/// always fire — treating an unknown spec value as a definite mismatch is
/// the safe / conservative choice.
fn num_shape_ordinal_to_str(ordinal: u32) -> &'static str {
    match ordinal {
        0 => "DIGIT",
        1 => "CIRCLED_DIGIT",
        2 => "ROMAN_CAPITAL",
        3 => "ROMAN_SMALL",
        4 => "LATIN_CAPITAL",
        5 => "LATIN_SMALL",
        6 => "CIRCLED_LATIN_CAPITAL",
        7 => "CIRCLED_LATIN_SMALL",
        8 => "HANGUL_SYLLABLE",
        9 => "CIRCLED_HANGUL_SYLLABLE",
        10 => "HANGUL_JAMO",
        11 => "CIRCLED_HANGUL_JAMO",
        12 => "HANGUL_PHONETIC",
        13 => "IDEOGRAPH",
        14 => "CIRCLED_IDEOGRAPH",
        15 => "DECAGON_CIRCLE",
        16 => "DECAGON_CIRCLE_HANJA",
        _ => "",
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
        error_string: String::new(),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::types::enums::HeadingType;
    use crate::document::header::types::shapes::{Numbering, ParaHead, ParaShape};
    use crate::document::header::HeaderTables;
    use crate::document::{Document, RunTypeInfo};
    use crate::error::outline_shape_codes::{
        OUTLINESHAPE_LEVEL_NUMBERSHAPE, OUTLINESHAPE_LEVEL_NUMBERTYPE,
    };
    use crate::spec::{LevelType, OutlineShapeSpec};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal [`Document`] with one run referencing a given
    /// paragraph shape. The paragraph shape has the supplied heading type,
    /// heading level. The `outline_shape_id_ref` on the run is set to
    /// `numbering_id` to mirror the section-level `outlineShapeIDRef`
    /// path used by the validator.
    fn doc_with_outline_para(
        para_id: u32,
        numbering_id: u32,
        heading_level: u32, // 0-indexed (heading_level=0 → level 1)
        numbering: Numbering,
    ) -> Document {
        let mut header = HeaderTables::default();

        let ps = ParaShape {
            id: para_id,
            heading_type: HeadingType::Outline,
            heading_id_ref: 0, // always 0 for section-wide outline shapes
            heading_level,
            ..Default::default()
        };
        header.para_shapes.insert(para_id, ps);
        header.numberings.insert(numbering_id, numbering);

        let run = RunTypeInfo {
            para_pr_id_ref: para_id,
            // The section's outlineShapeIDRef maps to the numbering id.
            outline_shape_id_ref: numbering_id,
            ..Default::default()
        };

        Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        }
    }

    /// Build a one-level [`Numbering`] with the given `num_format` and
    /// `num_format_text` at level 1.
    fn one_level_numbering(id: u32, num_format: &str, num_format_text: &str) -> Numbering {
        Numbering {
            id,
            start: 0,
            para_heads: vec![ParaHead {
                level: 1,
                num_format: num_format.into(),
                num_format_text: num_format_text.into(),
                ..Default::default()
            }],
        }
    }

    /// A spec with a single `LevelType` entry at level 1.
    fn spec_for_level1(numbertype: Option<&str>, numbershape: u32) -> OutlineShapeSpec {
        OutlineShapeSpec {
            leveltype: vec![LevelType {
                level: 1,
                numbertype: numbertype.map(str::to_string),
                numbershape,
            }],
        }
    }

    // -----------------------------------------------------------------------
    // Passing cases
    // -----------------------------------------------------------------------

    /// When the document's outline level 1 matches the spec exactly, no errors.
    #[test]
    fn matching_numbershape_and_numbertype_produces_no_errors() {
        // level 1: DIGIT ("^1.")  → spec ordinal 0, numbertype "^1."
        let num = one_level_numbering(1, "DIGIT", "^1.");
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = spec_for_level1(Some("^1."), 0);
        let errors = check(&doc, &spec);
        assert!(
            errors.is_empty(),
            "matching outline spec must produce no errors; got {errors:?}"
        );
    }

    /// When `numbertype` is absent from the spec, only `numbershape` is checked.
    #[test]
    fn absent_numbertype_skips_template_check() {
        let num = one_level_numbering(1, "DIGIT", "^1.");
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = spec_for_level1(None, 0); // no numbertype in spec
        let errors = check(&doc, &spec);
        assert!(
            errors.is_empty(),
            "absent numbertype must not produce NUMBERTYPE errors; got {errors:?}"
        );
    }

    /// Non-outline paragraphs (HeadingType::None) are silently ignored.
    #[test]
    fn non_outline_paragraphs_are_skipped() {
        let mut header = HeaderTables::default();
        let ps = ParaShape {
            id: 5,
            heading_type: HeadingType::None, // not an outline paragraph
            ..Default::default()
        };
        header.para_shapes.insert(5, ps);

        let run = RunTypeInfo {
            para_pr_id_ref: 5,
            ..Default::default()
        };

        let doc = Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        };

        let spec = spec_for_level1(Some("^1."), 0);
        let errors = check(&doc, &spec);
        assert!(
            errors.is_empty(),
            "non-outline paragraphs must not produce errors; got {errors:?}"
        );
    }

    /// An empty `leveltype` in the spec means the check is a no-op.
    #[test]
    fn empty_spec_produces_no_errors() {
        let num = one_level_numbering(1, "HANGUL_SYLLABLE", "^2.");
        let doc = doc_with_outline_para(0, 1, 1, num);
        let spec = OutlineShapeSpec { leveltype: vec![] };
        let errors = check(&doc, &spec);
        assert!(errors.is_empty(), "empty spec must produce no errors");
    }

    // -----------------------------------------------------------------------
    // Failing cases
    // -----------------------------------------------------------------------

    /// `numbershape` mismatch fires `OUTLINESHAPE_LEVEL_NUMBERSHAPE`.
    ///
    /// **Synthetic fail case**: document has `HANGUL_SYLLABLE` (ordinal 8)
    /// at level 1, but the spec requires ordinal 0 (`DIGIT`).
    #[test]
    fn numbershape_mismatch_fires_error() {
        // Document: level 1, HANGUL_SYLLABLE
        let num = one_level_numbering(1, "HANGUL_SYLLABLE", "^1.");
        let doc = doc_with_outline_para(0, 1, 0, num);
        // Spec: level 1, expects DIGIT (ordinal 0)
        let spec = spec_for_level1(None, 0);
        let errors = check(&doc, &spec);
        assert!(
            errors
                .iter()
                .any(|e| e.error_code == OUTLINESHAPE_LEVEL_NUMBERSHAPE),
            "HANGUL_SYLLABLE vs DIGIT must fire OUTLINESHAPE_LEVEL_NUMBERSHAPE; got {errors:?}"
        );
    }

    /// `numbertype` mismatch fires `OUTLINESHAPE_LEVEL_NUMBERTYPE`.
    ///
    /// **Synthetic fail case**: document template is `"^1)"` but spec expects `"^1."`.
    #[test]
    fn numbertype_mismatch_fires_error() {
        let num = one_level_numbering(1, "DIGIT", "^1)"); // wrong suffix
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = spec_for_level1(Some("^1."), 0); // expects "^1."
        let errors = check(&doc, &spec);
        assert!(
            errors
                .iter()
                .any(|e| e.error_code == OUTLINESHAPE_LEVEL_NUMBERTYPE),
            "wrong numbertype must fire OUTLINESHAPE_LEVEL_NUMBERTYPE; got {errors:?}"
        );
    }

    /// Duplicate `para_pr_id_ref`s produce at most one error per shape
    /// (deduplication by para shape id).
    #[test]
    fn duplicate_para_ids_produce_one_error() {
        let mut header = HeaderTables::default();

        let ps = ParaShape {
            id: 3,
            heading_type: HeadingType::Outline,
            heading_id_ref: 0, // section-wide; actual numbering key is on run
            heading_level: 0,
            ..Default::default()
        };
        header.para_shapes.insert(3, ps);
        header.numberings.insert(
            1,
            Numbering {
                id: 1,
                start: 0,
                para_heads: vec![ParaHead {
                    level: 1,
                    num_format: "HANGUL_SYLLABLE".into(), // wrong
                    num_format_text: "^1.".into(),
                    ..Default::default()
                }],
            },
        );

        // Three runs all referencing the same para shape.
        let make_run = || RunTypeInfo {
            para_pr_id_ref: 3,
            outline_shape_id_ref: 1, // section outline shape → numbering id 1
            ..Default::default()
        };

        let doc = Document {
            header: Some(header),
            run_type_infos: vec![make_run(), make_run(), make_run()],
            ..Default::default()
        };

        let spec = spec_for_level1(None, 0); // expects DIGIT
        let errors = check(&doc, &spec);
        let shape_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == OUTLINESHAPE_LEVEL_NUMBERSHAPE)
            .collect();
        assert_eq!(
            shape_errors.len(),
            1,
            "three runs with the same para_pr_id_ref must produce exactly one NUMBERSHAPE error"
        );
    }

    // -----------------------------------------------------------------------
    // num_shape_ordinal_to_str
    // -----------------------------------------------------------------------

    #[test]
    fn ordinal_mapping_spot_checks() {
        assert_eq!(num_shape_ordinal_to_str(0), "DIGIT");
        assert_eq!(num_shape_ordinal_to_str(1), "CIRCLED_DIGIT");
        assert_eq!(num_shape_ordinal_to_str(8), "HANGUL_SYLLABLE");
        assert_eq!(num_shape_ordinal_to_str(9), "CIRCLED_HANGUL_SYLLABLE");
        assert_eq!(num_shape_ordinal_to_str(10), "HANGUL_JAMO");
        assert_eq!(num_shape_ordinal_to_str(16), "DECAGON_CIRCLE_HANJA");
        assert_eq!(num_shape_ordinal_to_str(99), ""); // unknown → empty
    }
}
