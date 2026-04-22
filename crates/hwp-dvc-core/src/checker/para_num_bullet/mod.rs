//! Paragraph-number-bullet validator — `CheckParaNumBullet` port.
//!
//! Maps to `Checker::CheckParaNumBullet` / `CheckNumberParaHeadToCheckList`
//! in `references/dvc/Checker.cpp`.
//!
//! # Logic
//!
//! For every unique `para_pr_id_ref` in the [`RunTypeInfo`] stream whose
//! corresponding [`ParaShape`] has `heading_type == HeadingType::Number`:
//!
//! 1. Look up the [`Numbering`] identified by `ParaShape.heading_id_ref`.
//! 2. Validate top-level `Numbering` fields (start_number, value) against
//!    the spec's `start_number` and `value` fields.
//! 3. Find the [`ParaHead`] matching `ParaShape.heading_level`.
//! 4. For each [`LevelType`] in the spec whose `level` matches `paraHead.level`:
//!    - If `levelType.numbertype` is `Some(nt)` and it differs from
//!      `paraHead.num_format` → emit [`PARANUM_LEVEL_NUMBERTYPE`] (3406).
//!    - If `levelType.numbershape` maps to a format string that differs
//!      from `paraHead.num_format` → emit [`PARANUM_LEVEL_NUMBERSHAPE`] (3407).
//!    - If no `paraHead` at the spec's declared level exists in the `Numbering`
//!      → emit [`PARANUM_LEVELTYPE`] (3404).
//!
//! # Level walking
//!
//! The level-walker logic (number-shape ordinal mapping, level-count and
//! level-index validation) is shared with `checker::outline_shape` via the
//! [`crate::checker::numbering`] helper module introduced by issue #45.
//!
//! # Covered error codes
//!
//! | Constant                      | Code | Description                         |
//! |-------------------------------|------|-------------------------------------|
//! | [`PARANUM_STARTNUMBER`]        | 3402 | start-number flag mismatch          |
//! | [`PARANUM_VALUE`]              | 3403 | starting value mismatch             |
//! | [`PARANUM_LEVELTYPE`]          | 3404 | leveltype wrapper — level not found |
//! | [`PARANUM_LEVELTYPE_LEVEL`]    | 3405 | level index mismatch within entry   |
//! | [`PARANUM_LEVEL_NUMBERTYPE`]   | 3406 | Level number-type mismatch          |
//! | [`PARANUM_LEVEL_NUMBERSHAPE`]  | 3407 | Level number-shape mismatch         |
//!
//! [`RunTypeInfo`]: crate::document::RunTypeInfo
//! [`ParaShape`]: crate::document::header::ParaShape
//! [`Numbering`]: crate::document::header::Numbering
//! [`ParaHead`]: crate::document::header::ParaHead

use std::collections::HashSet;

use crate::checker::numbering as num_walker;
use crate::checker::DvcErrorInfo;
use crate::document::header::{HeadingType, Numbering, ParaHead};
use crate::document::{Document, RunTypeInfo};
use crate::error::para_num_bullet_codes::{
    PARANUM_LEVELTYPE, PARANUM_LEVELTYPE_LEVEL, PARANUM_LEVEL_NUMBERSHAPE,
    PARANUM_LEVEL_NUMBERTYPE, PARANUM_STARTNUMBER, PARANUM_VALUE,
};
use crate::spec::ParaNumBulletSpec;

/// Validate every paragraph that uses paragraph numbering against the spec
/// and return one error per offending (para_pr_id_ref, field) pair.
///
/// Returns an empty `Vec` immediately when the spec has no constraints
/// (no `start_number`, no `value`, and no `leveltype` entries).
///
/// # Port note
///
/// This is a port of `Checker::CheckParaNumBullet` /
/// `CheckNumberParaHeadToCheckList` from
/// `references/dvc/Checker.cpp`. The Rust implementation additionally
/// validates `start_number` (3402) and `value` (3403) fields that the
/// reference C++ stores in the spec model but does not emit as explicit
/// error codes from `CheckNumberParaHeadToCheckList` — those codes are
/// registered for completeness so that all 3400-range codes mirror the
/// reference `JsonModel.h` defines.
#[must_use]
pub fn check(document: &Document, spec: &ParaNumBulletSpec) -> Vec<DvcErrorInfo> {
    // Nothing to do when the spec declares no constraints.
    let has_level_constraints = !spec.leveltype.is_empty();
    let has_scalar_constraints = spec.start_number.is_some() || spec.value.is_some();
    if !has_level_constraints && !has_scalar_constraints {
        return Vec::new();
    }

    let header = match document.header.as_ref() {
        Some(h) => h,
        None => return Vec::new(),
    };

    // Collect unique para_pr_id_refs together with a representative run.
    // We only care about paragraphs that use paragraph numbering.
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

        // Only validate paragraphs that use paragraph numbering.
        if para_shape.heading_type != HeadingType::Number {
            continue;
        }

        let numbering = match header.numberings.get(&para_shape.heading_id_ref) {
            Some(n) => n,
            None => continue,
        };

        // --- start_number check (3402) ---
        // The spec's `start_number` flag indicates whether the numbering
        // should restart (`true`) or continue (`false`). In the OWPML model
        // a non-zero `Numbering.start` means restart; zero means continue.
        if let Some(expected_restart) = spec.start_number {
            let doc_restarts = numbering.start != 0;
            if doc_restarts != expected_restart {
                errors.push(num_walker::make_error(run, PARANUM_STARTNUMBER));
            }
        }

        // --- value check (3403) ---
        // The spec's `value` field is the required starting counter value for
        // the first level. The document stores this as `ParaHead.start` on the
        // level-1 paraHead.
        if let Some(expected_value) = spec.value {
            let first_head_start = numbering.para_heads.first().map(|ph| ph.start).unwrap_or(0);
            if first_head_start != expected_value {
                errors.push(num_walker::make_error(run, PARANUM_VALUE));
            }
        }

        if !has_level_constraints {
            continue;
        }

        let para_head = match find_para_head(numbering, para_shape.heading_level) {
            Some(ph) => ph,
            None => {
                // No paraHead found for this heading level even though the spec
                // declares leveltype constraints — emit the wrapper error (3404).
                errors.push(num_walker::make_error(run, PARANUM_LEVELTYPE));
                continue;
            }
        };

        check_para_head(run, para_head, spec, &mut errors);
    }

    errors
}

/// Find the [`ParaHead`] for a given 0-indexed heading level.
///
/// OWPML `<hh:heading level="N">` uses 0-indexed levels, while
/// `<hh:paraHead level="M">` uses 1-indexed levels. The conversion is
/// `paraHead.level == heading_level + 1`. This matches the reference C++
/// `getParaHeadByLevel(rParaPr->headingLevel)` which relies on the OWPML
/// model applying the same +1 offset during parsing.
fn find_para_head(numbering: &Numbering, heading_level: u32) -> Option<&ParaHead> {
    let target = heading_level + 1;
    numbering.para_heads.iter().find(|ph| ph.level == target)
}

/// Compare a single [`ParaHead`] against the spec's level-type entries and
/// push errors into `errors` for any mismatch found.
///
/// Checks performed (in order):
/// 1. Level index mismatch — emit `PARANUM_LEVELTYPE_LEVEL` (3405) when the
///    spec entry's `level` value differs from `paraHead.level`.
/// 2. numbertype — emit `PARANUM_LEVEL_NUMBERTYPE` (3406) when the spec's
///    optional `numbertype` string differs from `paraHead.num_format`.
/// 3. numbershape — emit `PARANUM_LEVEL_NUMBERSHAPE` (3407) when the spec's
///    `numbershape` ordinal maps to a different `numFormat` string.
fn check_para_head(
    run: &RunTypeInfo,
    para_head: &ParaHead,
    spec: &ParaNumBulletSpec,
    errors: &mut Vec<DvcErrorInfo>,
) {
    let spec_level = match spec.leveltype.iter().find(|lt| lt.level == para_head.level) {
        Some(lt) => lt,
        None => return, // No spec entry for this level — skip.
    };

    // --- leveltype level check (3405) ---
    // Validate that the spec's declared `level` matches the actual paraHead
    // level. In practice `find` above ensures they match when this branch is
    // reached, but we emit the error defensively so that callers driving the
    // check with a pre-resolved spec entry do not silently skip it.
    if spec_level.level != para_head.level {
        errors.push(num_walker::make_error(run, PARANUM_LEVELTYPE_LEVEL));
    }

    // --- numbertype check (3406) ---
    // The spec's `numbertype` field is an optional string (e.g. "DIGIT",
    // "HANGUL_SYLLABLE"). Compare it against `paraHead.num_format` which
    // holds the OWPML `numFormat` attribute value.
    if let Some(ref expected_nt) = spec_level.numbertype {
        if *expected_nt != para_head.num_format {
            errors.push(num_walker::make_error(run, PARANUM_LEVEL_NUMBERTYPE));
        }
    }

    // --- numbershape check (3407) ---
    // The spec's `numbershape` field is a u32 ordinal that maps to the
    // `NumberShapeType` enum in the reference C++. We convert it to the
    // corresponding OWPML `numFormat` attribute string and compare.
    // Unknown ordinals return `None` and skip the comparison (future-proofing).
    if let Some(expected_str) = num_walker::num_shape_to_str(spec_level.numbershape) {
        if expected_str != para_head.num_format {
            errors.push(num_walker::make_error(run, PARANUM_LEVEL_NUMBERSHAPE));
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
// All shared helpers (num_shape_to_str via num_walker::num_shape_to_str,
// make_error via num_walker::make_error) now live in
// `crate::checker::numbering`. This module has no private helpers of its own.

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::types::{
        HeadingType, LineSpacing, Margin, Numbering, ParaHead, ParaShape as HdrParaShape,
    };
    use crate::document::header::HeaderTables;
    use crate::document::{Document, RunTypeInfo};
    use crate::spec::{LevelType, ParaNumBulletSpec};

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn doc_with_para_num(
        para_pr_id: u32,
        heading_id_ref: u32,
        heading_level: u32,
        numbering_id: u32,
        num_format: &str,
    ) -> Document {
        let mut header = HeaderTables::default();

        // Build a ParaShape that uses paragraph numbering.
        let ps = HdrParaShape {
            id: para_pr_id,
            heading_type: HeadingType::Number,
            heading_id_ref,
            heading_level,
            line_spacing: LineSpacing {
                type_: crate::document::header::LineSpacingType::Percent,
                value: 160,
                unit: "HWPUNIT".into(),
            },
            margin: Margin::default(),
            ..Default::default()
        };
        header.para_shapes.insert(para_pr_id, ps);

        // Build a Numbering with one ParaHead at level `heading_level + 1`.
        let ph = ParaHead {
            level: heading_level + 1,
            num_format: num_format.to_string(),
            ..Default::default()
        };
        let numbering = Numbering {
            id: numbering_id,
            para_heads: vec![ph],
            ..Default::default()
        };
        header.numberings.insert(numbering_id, numbering);

        let run = RunTypeInfo {
            para_pr_id_ref: para_pr_id,
            text: "테스트".into(),
            ..Default::default()
        };

        Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        }
    }

    fn spec_with_level(
        level: u32,
        numbertype: Option<&str>,
        numbershape: u32,
    ) -> ParaNumBulletSpec {
        ParaNumBulletSpec {
            leveltype: vec![LevelType {
                level,
                numbertype: numbertype.map(String::from),
                numbershape,
            }],
            ..Default::default()
        }
    }

    /// Build a [`Document`] where the `Numbering` has the given `start` value
    /// and the first `paraHead` has the given `para_head_start`.
    fn doc_with_numbering_start(
        para_pr_id: u32,
        heading_id_ref: u32,
        numbering_id: u32,
        numbering_start: u32,
        para_head_start: u32,
    ) -> Document {
        let mut header = HeaderTables::default();

        let ps = HdrParaShape {
            id: para_pr_id,
            heading_type: HeadingType::Number,
            heading_id_ref,
            heading_level: 0,
            line_spacing: LineSpacing {
                type_: crate::document::header::LineSpacingType::Percent,
                value: 160,
                unit: "HWPUNIT".into(),
            },
            margin: Margin::default(),
            ..Default::default()
        };
        header.para_shapes.insert(para_pr_id, ps);

        let ph = ParaHead {
            level: 1,
            start: para_head_start,
            num_format: "DIGIT".to_string(),
            ..Default::default()
        };
        let numbering = Numbering {
            id: numbering_id,
            start: numbering_start,
            para_heads: vec![ph],
        };
        header.numberings.insert(numbering_id, numbering);

        let run = RunTypeInfo {
            para_pr_id_ref: para_pr_id,
            text: "테스트".into(),
            ..Default::default()
        };

        Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        }
    }

    // -------------------------------------------------------------------------
    // Empty spec → no errors
    // -------------------------------------------------------------------------

    #[test]
    fn empty_spec_produces_no_errors() {
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = ParaNumBulletSpec::default();
        let errs = check(&doc, &spec);
        assert!(errs.is_empty(), "empty spec must produce no errors");
    }

    // -------------------------------------------------------------------------
    // No numbered paragraphs → no errors
    // -------------------------------------------------------------------------

    #[test]
    fn non_numbered_paragraph_produces_no_errors() {
        let mut header = HeaderTables::default();
        let ps = HdrParaShape {
            id: 0,
            heading_type: HeadingType::None, // not a numbered paragraph
            ..Default::default()
        };
        header.para_shapes.insert(0, ps);
        let run = RunTypeInfo {
            para_pr_id_ref: 0,
            ..Default::default()
        };
        let doc = Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        };
        let spec = spec_with_level(1, Some("DIGIT"), 0);
        let errs = check(&doc, &spec);
        assert!(
            errs.is_empty(),
            "non-numbered paragraph must not produce errors"
        );
    }

    // -------------------------------------------------------------------------
    // numbertype checks
    // -------------------------------------------------------------------------

    #[test]
    fn numbertype_match_produces_no_error() {
        // doc has DIGIT, spec expects DIGIT → no error.
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = spec_with_level(1, Some("DIGIT"), 0 /* DIGIT */);
        let errs = check(&doc, &spec);
        let nt_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_LEVEL_NUMBERTYPE)
            .collect();
        assert!(nt_errs.is_empty(), "matching numbertype must not error");
    }

    #[test]
    fn numbertype_mismatch_produces_error() {
        // doc has DIGIT, spec expects HANGUL_SYLLABLE → error.
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = spec_with_level(1, Some("HANGUL_SYLLABLE"), 0);
        let errs = check(&doc, &spec);
        assert!(
            errs.iter()
                .any(|e| e.error_code == PARANUM_LEVEL_NUMBERTYPE),
            "mismatching numbertype must produce PARANUM_LEVEL_NUMBERTYPE error"
        );
    }

    #[test]
    fn none_numbertype_skips_check() {
        // spec has no numbertype restriction → no 3406 error regardless of doc value.
        let doc = doc_with_para_num(0, 1, 0, 1, "ROMAN_SMALL");
        let spec = spec_with_level(1, None, 0 /* DIGIT — but numbertype is None */);
        let errs = check(&doc, &spec);
        let nt_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_LEVEL_NUMBERTYPE)
            .collect();
        assert!(
            nt_errs.is_empty(),
            "None numbertype must not produce 3406 errors"
        );
    }

    // -------------------------------------------------------------------------
    // numbershape checks
    // -------------------------------------------------------------------------

    #[test]
    fn numbershape_match_produces_no_error() {
        // doc has DIGIT, spec numbershape=0 (DIGIT) → no error.
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = spec_with_level(1, None, 0);
        let errs = check(&doc, &spec);
        let ns_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_LEVEL_NUMBERSHAPE)
            .collect();
        assert!(ns_errs.is_empty(), "matching numbershape must not error");
    }

    #[test]
    fn numbershape_mismatch_produces_error() {
        // doc has HANGUL_SYLLABLE, spec numbershape=0 (DIGIT) → error.
        let doc = doc_with_para_num(0, 1, 0, 1, "HANGUL_SYLLABLE");
        let spec = spec_with_level(1, None, 0);
        let errs = check(&doc, &spec);
        assert!(
            errs.iter()
                .any(|e| e.error_code == PARANUM_LEVEL_NUMBERSHAPE),
            "mismatching numbershape must produce PARANUM_LEVEL_NUMBERSHAPE error"
        );
    }

    #[test]
    fn unknown_numbershape_ordinal_skips_check() {
        // spec numbershape=99 has no known mapping → no 3407 error.
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = spec_with_level(1, None, 99);
        let errs = check(&doc, &spec);
        let ns_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_LEVEL_NUMBERSHAPE)
            .collect();
        assert!(
            ns_errs.is_empty(),
            "unknown numbershape ordinal must not produce 3407 errors"
        );
    }

    // -------------------------------------------------------------------------
    // Level matching
    // -------------------------------------------------------------------------

    #[test]
    fn spec_level_mismatch_skips_check() {
        // doc paragraph is at level 1, spec only has entry for level 2 → no error.
        let doc = doc_with_para_num(0, 1, 0, 1, "DIGIT");
        let spec = ParaNumBulletSpec {
            leveltype: vec![LevelType {
                level: 2,
                numbertype: Some("HANGUL_SYLLABLE".into()),
                numbershape: 8,
            }],
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        assert!(
            errs.is_empty(),
            "no spec entry for this level must produce no errors"
        );
    }

    // -------------------------------------------------------------------------
    // Deduplication
    // -------------------------------------------------------------------------

    #[test]
    fn duplicate_para_pr_id_refs_produce_one_error() {
        // Three runs all using the same para_pr_id_ref — must only fire once.
        let doc = {
            let mut doc = doc_with_para_num(5, 1, 0, 1, "HANGUL_SYLLABLE");
            // Append two more runs with the same para_pr_id_ref.
            doc.run_type_infos.push(RunTypeInfo {
                para_pr_id_ref: 5,
                text: "b".into(),
                ..Default::default()
            });
            doc.run_type_infos.push(RunTypeInfo {
                para_pr_id_ref: 5,
                text: "c".into(),
                ..Default::default()
            });
            doc
        };
        let spec = spec_with_level(1, None, 0 /* DIGIT vs HANGUL_SYLLABLE */);
        let errs = check(&doc, &spec);
        let ns_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_LEVEL_NUMBERSHAPE)
            .collect();
        assert_eq!(
            ns_errs.len(),
            1,
            "duplicate para_pr_id_refs must produce exactly one error"
        );
    }

    // -------------------------------------------------------------------------
    // start_number checks (3402)
    // -------------------------------------------------------------------------

    #[test]
    fn start_number_match_restart_produces_no_error() {
        // spec says restart=true, doc numbering.start=1 (non-zero → restart) → no error.
        let doc = doc_with_numbering_start(0, 1, 1, 1, 1);
        let spec = ParaNumBulletSpec {
            start_number: Some(true),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        let sn_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_STARTNUMBER)
            .collect();
        assert!(
            sn_errs.is_empty(),
            "matching restart flag must not produce PARANUM_STARTNUMBER error"
        );
    }

    #[test]
    fn start_number_mismatch_restart_produces_error() {
        // spec says restart=true, doc numbering.start=0 (continue) → error 3402.
        let doc = doc_with_numbering_start(0, 1, 1, 0, 1);
        let spec = ParaNumBulletSpec {
            start_number: Some(true),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        assert!(
            errs.iter().any(|e| e.error_code == PARANUM_STARTNUMBER),
            "restart-vs-continue mismatch must produce PARANUM_STARTNUMBER error"
        );
    }

    #[test]
    fn start_number_match_continue_produces_no_error() {
        // spec says restart=false, doc numbering.start=0 (continue) → no error.
        let doc = doc_with_numbering_start(0, 1, 1, 0, 1);
        let spec = ParaNumBulletSpec {
            start_number: Some(false),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        let sn_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_STARTNUMBER)
            .collect();
        assert!(
            sn_errs.is_empty(),
            "matching continue flag must not produce PARANUM_STARTNUMBER error"
        );
    }

    #[test]
    fn start_number_none_skips_check() {
        // spec has no start_number → no 3402 error regardless of doc value.
        let doc = doc_with_numbering_start(0, 1, 1, 5, 1);
        let spec = ParaNumBulletSpec {
            start_number: None,
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        let sn_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_STARTNUMBER)
            .collect();
        assert!(
            sn_errs.is_empty(),
            "None start_number must not produce 3402 errors"
        );
    }

    // -------------------------------------------------------------------------
    // value checks (3403)
    // -------------------------------------------------------------------------

    #[test]
    fn value_match_produces_no_error() {
        // spec says value=1, doc first paraHead.start=1 → no error.
        let doc = doc_with_numbering_start(0, 1, 1, 1, 1);
        let spec = ParaNumBulletSpec {
            value: Some(1),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        let v_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_VALUE)
            .collect();
        assert!(
            v_errs.is_empty(),
            "matching value must not produce PARANUM_VALUE error"
        );
    }

    #[test]
    fn value_mismatch_produces_error() {
        // spec says value=5, doc first paraHead.start=1 → error 3403.
        let doc = doc_with_numbering_start(0, 1, 1, 0, 1);
        let spec = ParaNumBulletSpec {
            value: Some(5),
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        assert!(
            errs.iter().any(|e| e.error_code == PARANUM_VALUE),
            "value mismatch must produce PARANUM_VALUE error"
        );
    }

    #[test]
    fn value_none_skips_check() {
        // spec has no value → no 3403 error regardless of doc value.
        let doc = doc_with_numbering_start(0, 1, 1, 0, 99);
        let spec = ParaNumBulletSpec {
            value: None,
            ..Default::default()
        };
        let errs = check(&doc, &spec);
        let v_errs: Vec<_> = errs
            .iter()
            .filter(|e| e.error_code == PARANUM_VALUE)
            .collect();
        assert!(v_errs.is_empty(), "None value must not produce 3403 errors");
    }

    // -------------------------------------------------------------------------
    // leveltype wrapper checks (3404)
    // -------------------------------------------------------------------------

    #[test]
    fn leveltype_missing_parahead_produces_leveltype_error() {
        // doc has level-0 heading (maps to level 1 paraHead), but spec requests level 2.
        // Since no paraHead at level 2 exists, the validator emits PARANUM_LEVELTYPE (3404).
        let mut header = HeaderTables::default();

        let ps = HdrParaShape {
            id: 0,
            heading_type: HeadingType::Number,
            heading_id_ref: 1,
            heading_level: 1, // heading level 1 → paraHead level 2
            line_spacing: LineSpacing {
                type_: crate::document::header::LineSpacingType::Percent,
                value: 160,
                unit: "HWPUNIT".into(),
            },
            margin: Margin::default(),
            ..Default::default()
        };
        header.para_shapes.insert(0, ps);

        // Numbering only has a paraHead at level 1, not level 2.
        let ph = ParaHead {
            level: 1,
            num_format: "DIGIT".to_string(),
            ..Default::default()
        };
        let numbering = Numbering {
            id: 1,
            start: 0,
            para_heads: vec![ph],
        };
        header.numberings.insert(1, numbering);

        let run = RunTypeInfo {
            para_pr_id_ref: 0,
            ..Default::default()
        };
        let doc = Document {
            header: Some(header),
            run_type_infos: vec![run],
            ..Default::default()
        };

        // Spec has leveltype constraints → when no paraHead is found, emit 3404.
        let spec = spec_with_level(2, None, 8);
        let errs = check(&doc, &spec);
        assert!(
            errs.iter().any(|e| e.error_code == PARANUM_LEVELTYPE),
            "missing paraHead for heading_level must produce PARANUM_LEVELTYPE error; got {errs:?}"
        );
    }

    // -------------------------------------------------------------------------
    // num_shape_to_str mapping (via shared numbering helper)
    // -------------------------------------------------------------------------

    #[test]
    fn number_shape_mapping_covers_all_known_ordinals() {
        use crate::checker::numbering::num_shape_to_str;
        let known = [
            (0u32, "DIGIT"),
            (1, "CIRCLED_DIGIT"),
            (2, "ROMAN_CAPITAL"),
            (3, "ROMAN_SMALL"),
            (4, "LATIN_CAPITAL"),
            (5, "LATIN_SMALL"),
            (6, "CIRCLED_LATIN_CAPITAL"),
            (7, "CIRCLED_LATIN_SMALL"),
            (8, "HANGUL_SYLLABLE"),
            (9, "CIRCLED_HANGUL_SYLLABLE"),
            (10, "HANGUL_JAMO"),
            (11, "CIRCLED_HANGUL_JAMO"),
            (12, "HANGUL_PHONETIC"),
            (13, "IDEOGRAPH"),
            (14, "CIRCLED_IDEOGRAPH"),
            (15, "DECAGON_CIRCLE"),
            (16, "DECAGON_CIRCLE_HANJA"),
        ];
        for (ordinal, expected) in known {
            assert_eq!(
                num_shape_to_str(ordinal),
                Some(expected),
                "ordinal {ordinal} must map to {expected}"
            );
        }
        assert_eq!(
            num_shape_to_str(17),
            None,
            "unknown ordinal must return None"
        );
    }
}
