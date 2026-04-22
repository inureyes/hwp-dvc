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
//!
//! ## Top-level numbering checks
//!
//! - `start_number` → `Numbering::start` fires `OUTLINESHAPE_STARTNUMBER` (3202)
//! - `value`        → `ParaHead::start` (per-level) fires `OUTLINESHAPE_VALUE` (3203)
//! - level count    → `leveltype` length vs document levels fires `OUTLINESHAPE_LEVELTYPE` (3204)
//! - level index    → `LevelType::level` vs `ParaHead::level` fires `OUTLINESHAPE_LEVELTYPE_LEVEL` (3205)
//!
//! ## Per-level checks (from the original implementation)
//!
//! - `num_format`      → `LevelType::numbershape` (via ordinal mapping) fires `OUTLINESHAPE_LEVEL_NUMBERSHAPE` (3207)
//! - `num_format_text` → `LevelType::numbertype` (template string, when present) fires `OUTLINESHAPE_LEVEL_NUMBERTYPE` (3206)
//!
//! One [`DvcErrorInfo`] is emitted per (run, mismatched field) pair.
//!
//! # Error codes (complete 3200-range)
//!
//! | Mismatch                   | Code                            |
//! |----------------------------|---------------------------------|
//! | shape type name            | `OUTLINESHAPE_TYPE` (3201)      |
//! | start number               | `OUTLINESHAPE_STARTNUMBER` (3202)|
//! | per-level start value      | `OUTLINESHAPE_VALUE` (3203)     |
//! | level-count wrapper        | `OUTLINESHAPE_LEVELTYPE` (3204) |
//! | level index within wrapper | `OUTLINESHAPE_LEVELTYPE_LEVEL` (3205)|
//! | `numbertype` template      | `OUTLINESHAPE_LEVEL_NUMBERTYPE` (3206)|
//! | `numbershape` enum         | `OUTLINESHAPE_LEVEL_NUMBERSHAPE` (3207)|
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

use crate::checker::numbering as num_walker;
use crate::checker::DvcErrorInfo;
use crate::document::header::types::enums::HeadingType;
use crate::document::header::Numbering;
use crate::document::{Document, RunTypeInfo};
use crate::error::outline_shape_codes::{
    OUTLINESHAPE_LEVELTYPE, OUTLINESHAPE_LEVELTYPE_LEVEL, OUTLINESHAPE_LEVEL_NUMBERSHAPE,
    OUTLINESHAPE_LEVEL_NUMBERTYPE, OUTLINESHAPE_STARTNUMBER, OUTLINESHAPE_VALUE,
};
use crate::spec::OutlineShapeSpec;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Validate outline numbering shapes for every unique outline paragraph in
/// `document` against `spec`.
///
/// Returns an empty `Vec` when:
/// - the spec has no constraints (no `leveltype`, no `start_number`, no `value`), or
/// - no paragraph uses outline numbering, or
/// - all outline paragraphs match the spec.
#[must_use]
pub fn check(document: &Document, spec: &OutlineShapeSpec) -> Vec<DvcErrorInfo> {
    let spec_is_empty =
        spec.leveltype.is_empty() && spec.start_number.is_none() && spec.value.is_none();
    if spec_is_empty {
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

    // Track which numbering IDs have already been checked for top-level
    // constraints (start_number, value, leveltype wrapper) so we do not
    // emit the same error multiple times for different outline paragraphs
    // that share the same numbering template.
    let mut checked_numbering_ids: HashSet<u32> = HashSet::new();

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

        // --- Top-level numbering checks (once per unique numbering template) ---
        if checked_numbering_ids.insert(numbering.id) {
            check_numbering_top_level(run, numbering, spec, &mut errors);
        }

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

        // --- value (per-level start value) ---
        // When `spec.value` is set, validate every `ParaHead::start` against it.
        // Fires OUTLINESHAPE_VALUE (3203).
        if let Some(expected_value) = spec.value {
            if para_head.start != expected_value {
                errors.push(num_walker::make_error(run, OUTLINESHAPE_VALUE));
            }
        }

        // --- numbertype (template string) ---
        // Only checked when the spec supplies a `numbertype` value.
        if let Some(expected_type) = &spec_entry.numbertype {
            if &para_head.num_format_text != expected_type {
                errors.push(num_walker::make_error(run, OUTLINESHAPE_LEVEL_NUMBERTYPE));
            }
        }

        // --- numbershape (enum ordinal → num_format string) ---
        // Unknown ordinals return `None` from `num_shape_to_str`; treat as a
        // definite mismatch (conservative) by mapping `None` to `""`.
        let expected_shape_str = num_walker::num_shape_to_str(spec_entry.numbershape).unwrap_or("");
        if para_head.num_format != expected_shape_str {
            errors.push(num_walker::make_error(run, OUTLINESHAPE_LEVEL_NUMBERSHAPE));
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Top-level numbering checks (start_number, leveltype count)
// ---------------------------------------------------------------------------

/// Check the top-level [`Numbering`] constraints that are validated once
/// per unique numbering template (not per run).
///
/// - `start_number`: `Numbering::start` vs `spec.start_number` → `OUTLINESHAPE_STARTNUMBER` (3202)
/// - `leveltype` count + level-index: delegated to [`num_walker::check_level_sequence`]
///   using codes `OUTLINESHAPE_LEVELTYPE` (3204) and `OUTLINESHAPE_LEVELTYPE_LEVEL` (3205).
fn check_numbering_top_level(
    run: &RunTypeInfo,
    numbering: &Numbering,
    spec: &OutlineShapeSpec,
    errors: &mut Vec<DvcErrorInfo>,
) {
    // --- start_number (3202) ---
    // Compare the top-level `<hh:numbering start="…"/>` attribute against
    // the spec's expected start number.
    if let Some(expected_start) = spec.start_number {
        if numbering.start != expected_start {
            errors.push(num_walker::make_error(run, OUTLINESHAPE_STARTNUMBER));
        }
    }

    // --- leveltype count (3204) + level-index (3205) ---
    // Delegated to the shared level-sequence walker. Both the count check and
    // the index-sequential check are structurally identical to the para-num
    // variant; only the error codes differ.
    num_walker::check_level_sequence(
        run,
        numbering,
        &spec.leveltype,
        OUTLINESHAPE_LEVELTYPE,
        OUTLINESHAPE_LEVELTYPE_LEVEL,
        errors,
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
// All shared helpers (num_shape_to_str, make_error, check_level_sequence)
// live in `crate::checker::numbering`. This module has no private helpers
// of its own beyond those delegations above.

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
        OUTLINESHAPE_LEVELTYPE, OUTLINESHAPE_LEVELTYPE_LEVEL, OUTLINESHAPE_LEVEL_NUMBERSHAPE,
        OUTLINESHAPE_LEVEL_NUMBERTYPE, OUTLINESHAPE_STARTNUMBER, OUTLINESHAPE_VALUE,
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
    /// `num_format_text` at level 1.  `start` defaults to 0.
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

    /// Build a one-level [`Numbering`] with an explicit top-level `start` value.
    fn one_level_numbering_with_start(
        id: u32,
        numbering_start: u32,
        num_format: &str,
        num_format_text: &str,
        para_head_start: u32,
    ) -> Numbering {
        Numbering {
            id,
            start: numbering_start,
            para_heads: vec![ParaHead {
                level: 1,
                start: para_head_start,
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
            ..Default::default()
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

    /// A fully empty spec (no leveltype, no start_number, no value) is a no-op.
    #[test]
    fn empty_spec_produces_no_errors() {
        let num = one_level_numbering(1, "HANGUL_SYLLABLE", "^2.");
        let doc = doc_with_outline_para(0, 1, 1, num);
        let spec = OutlineShapeSpec::default();
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
    // New sub-code tests: STARTNUMBER (3202), VALUE (3203),
    //                     LEVELTYPE (3204), LEVELTYPE_LEVEL (3205)
    // -----------------------------------------------------------------------

    /// `start_number` mismatch fires `OUTLINESHAPE_STARTNUMBER` (3202).
    ///
    /// The spec declares `start_number: 1` but the document's `<hh:numbering
    /// start="0"/>` has start=0.
    #[test]
    fn start_number_mismatch_fires_error() {
        // numbering_start=0, para_head_start=0, format="DIGIT"
        let num = one_level_numbering_with_start(1, 0, "DIGIT", "^1.", 0);
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = OutlineShapeSpec {
            start_number: Some(1), // document has 0, expects 1
            leveltype: vec![LevelType {
                level: 1,
                numbertype: None,
                numbershape: 0, // DIGIT
            }],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        assert!(
            errors
                .iter()
                .any(|e| e.error_code == OUTLINESHAPE_STARTNUMBER),
            "start_number mismatch must fire OUTLINESHAPE_STARTNUMBER (3202); got {errors:?}"
        );
    }

    /// When `start_number` matches, no `OUTLINESHAPE_STARTNUMBER` error.
    #[test]
    fn matching_start_number_produces_no_error() {
        let num = one_level_numbering_with_start(1, 1, "DIGIT", "^1.", 1);
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = OutlineShapeSpec {
            start_number: Some(1), // matches
            leveltype: vec![LevelType {
                level: 1,
                numbertype: None,
                numbershape: 0, // DIGIT
            }],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        let start_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == OUTLINESHAPE_STARTNUMBER)
            .collect();
        assert!(
            start_errors.is_empty(),
            "matching start_number must not produce STARTNUMBER errors; got {errors:?}"
        );
    }

    /// `value` mismatch fires `OUTLINESHAPE_VALUE` (3203).
    ///
    /// The spec declares `value: 1` (expected per-level start) but the
    /// document's `<hh:paraHead start="0"/>` has start=0.
    #[test]
    fn value_mismatch_fires_error() {
        // para_head_start=0, but spec expects value=1
        let num = one_level_numbering_with_start(1, 0, "DIGIT", "^1.", 0);
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = OutlineShapeSpec {
            value: Some(1), // expects per-level start=1; document has 0
            leveltype: vec![LevelType {
                level: 1,
                numbertype: None,
                numbershape: 0,
            }],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        assert!(
            errors.iter().any(|e| e.error_code == OUTLINESHAPE_VALUE),
            "value mismatch must fire OUTLINESHAPE_VALUE (3203); got {errors:?}"
        );
    }

    /// When `value` matches, no `OUTLINESHAPE_VALUE` error.
    #[test]
    fn matching_value_produces_no_error() {
        let num = one_level_numbering_with_start(1, 0, "DIGIT", "^1.", 1);
        let doc = doc_with_outline_para(0, 1, 0, num);
        let spec = OutlineShapeSpec {
            value: Some(1), // matches para_head_start=1
            leveltype: vec![LevelType {
                level: 1,
                numbertype: None,
                numbershape: 0,
            }],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        let val_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == OUTLINESHAPE_VALUE)
            .collect();
        assert!(
            val_errors.is_empty(),
            "matching value must not produce VALUE errors; got {errors:?}"
        );
    }

    /// Level-count mismatch fires `OUTLINESHAPE_LEVELTYPE` (3204).
    ///
    /// The spec has 2 leveltype entries but the document only declares 1
    /// `<hh:paraHead>` level — the level-count wrapper is violated.
    #[test]
    fn level_count_mismatch_fires_leveltype_error() {
        // Document: only 1 level (level 1).
        let num = one_level_numbering(1, "DIGIT", "^1.");
        let doc = doc_with_outline_para(0, 1, 0, num);
        // Spec: 2 levels — document has only 1.
        let spec = OutlineShapeSpec {
            leveltype: vec![
                LevelType {
                    level: 1,
                    numbertype: None,
                    numbershape: 0,
                },
                LevelType {
                    level: 2,
                    numbertype: None,
                    numbershape: 0,
                },
            ],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        assert!(
            errors
                .iter()
                .any(|e| e.error_code == OUTLINESHAPE_LEVELTYPE),
            "level count mismatch must fire OUTLINESHAPE_LEVELTYPE (3204); got {errors:?}"
        );
    }

    /// `level` index within a `leveltype` entry that is not sequential fires
    /// `OUTLINESHAPE_LEVELTYPE_LEVEL` (3205).
    ///
    /// The spec's `leveltype` array must have entries with `level` values that
    /// are sequential 1-indexed integers (entry 0 → level 1, entry 1 → level 2,
    /// …). If an entry's `level` field skips or repeats an index, 3205 fires.
    ///
    /// **Synthetic fail case**: spec declares two entries with levels `[1, 3]`
    /// (gap at 2 — entry index 1 should be level 2 but is level 3).
    #[test]
    fn level_index_mismatch_fires_leveltype_level_error() {
        // Two-level numbering (para_heads at levels 1 and 3).
        let num = Numbering {
            id: 1,
            start: 0,
            para_heads: vec![
                ParaHead {
                    level: 1,
                    num_format: "DIGIT".into(),
                    num_format_text: "^1.".into(),
                    ..Default::default()
                },
                ParaHead {
                    level: 3, // note: level 2 is skipped
                    num_format: "DIGIT".into(),
                    num_format_text: "^2.".into(),
                    ..Default::default()
                },
            ],
        };
        let doc = doc_with_outline_para(0, 1, 0, num); // heading_level=0 → para_level=1
                                                       // Spec leveltype entries at index 0 → level=1 (ok) and index 1 → level=3 (bad; expected 2).
        let spec = OutlineShapeSpec {
            leveltype: vec![
                LevelType {
                    level: 1,
                    numbertype: None,
                    numbershape: 0,
                },
                LevelType {
                    level: 3,
                    numbertype: None,
                    numbershape: 0,
                }, // wrong: expected level=2
            ],
            ..Default::default()
        };
        let errors = check(&doc, &spec);
        assert!(
            errors
                .iter()
                .any(|e| e.error_code == OUTLINESHAPE_LEVELTYPE_LEVEL),
            "non-sequential level index must fire OUTLINESHAPE_LEVELTYPE_LEVEL (3205); got {errors:?}"
        );
    }

    // -----------------------------------------------------------------------
    // num_shape_to_str (via shared numbering helper)
    // -----------------------------------------------------------------------

    #[test]
    fn ordinal_mapping_spot_checks() {
        use crate::checker::numbering::num_shape_to_str;
        assert_eq!(num_shape_to_str(0), Some("DIGIT"));
        assert_eq!(num_shape_to_str(1), Some("CIRCLED_DIGIT"));
        assert_eq!(num_shape_to_str(8), Some("HANGUL_SYLLABLE"));
        assert_eq!(num_shape_to_str(9), Some("CIRCLED_HANGUL_SYLLABLE"));
        assert_eq!(num_shape_to_str(10), Some("HANGUL_JAMO"));
        assert_eq!(num_shape_to_str(16), Some("DECAGON_CIRCLE_HANJA"));
        assert_eq!(num_shape_to_str(99), None); // unknown → None
    }
}
