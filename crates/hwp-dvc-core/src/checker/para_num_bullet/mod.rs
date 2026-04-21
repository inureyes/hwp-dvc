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
//! 2. Find the [`ParaHead`] matching `ParaShape.heading_level`.
//! 3. For each [`LevelType`] in the spec whose `level` matches `paraHead.level`:
//!    - If `levelType.numbertype` is `Some(nt)` and it differs from
//!      `paraHead.num_format` → emit [`PARANUM_LEVEL_NUMBERTYPE`] (3406).
//!    - If `levelType.numbershape` maps to a format string that differs
//!      from `paraHead.num_format` → emit [`PARANUM_LEVEL_NUMBERSHAPE`] (3407).
//!
//! # Level walking
//!
//! TODO: Issue #12 (`CheckOutlineShape`) walks levels with a similar
//! helper. Once both PRs are merged, the level-walker logic in this module
//! and in `checker::outline_shape` (if it exists) should be unified into a
//! shared `checker::level_walk` helper to avoid duplication.
//! See Epic #1 technical notes for context.
//!
//! # Covered error codes
//!
//! | Constant                    | Code | Description                     |
//! |-----------------------------|------|---------------------------------|
//! | [`PARANUM_LEVEL_NUMBERTYPE`] | 3406 | Level number-type mismatch      |
//! | [`PARANUM_LEVEL_NUMBERSHAPE`]| 3407 | Level number-shape mismatch     |
//!
//! [`RunTypeInfo`]: crate::document::RunTypeInfo
//! [`ParaShape`]: crate::document::header::ParaShape
//! [`Numbering`]: crate::document::header::Numbering
//! [`ParaHead`]: crate::document::header::ParaHead

use std::collections::HashSet;

use crate::checker::DvcErrorInfo;
use crate::document::header::{HeadingType, Numbering, ParaHead};
use crate::document::{Document, RunTypeInfo};
use crate::error::para_num_bullet_codes::{PARANUM_LEVEL_NUMBERSHAPE, PARANUM_LEVEL_NUMBERTYPE};
use crate::spec::ParaNumBulletSpec;

/// Validate every paragraph that uses paragraph numbering against the spec
/// and return one error per offending (para_pr_id_ref, field) pair.
///
/// Returns an empty `Vec` immediately when `spec.leveltype` is empty
/// (nothing to validate).
#[must_use]
pub fn check(document: &Document, spec: &ParaNumBulletSpec) -> Vec<DvcErrorInfo> {
    if spec.leveltype.is_empty() {
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

        let para_head = match find_para_head(numbering, para_shape.heading_level) {
            Some(ph) => ph,
            None => continue,
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

    // --- numbertype check ---
    // The spec's `numbertype` field is an optional string (e.g. "DIGIT",
    // "HANGUL_SYLLABLE"). Compare it against `paraHead.num_format` which
    // holds the OWPML `numFormat` attribute value.
    if let Some(ref expected_nt) = spec_level.numbertype {
        if *expected_nt != para_head.num_format {
            errors.push(make_error(run, PARANUM_LEVEL_NUMBERTYPE));
        }
    }

    // --- numbershape check ---
    // The spec's `numbershape` field is a u32 ordinal that maps to the
    // `NumberShapeType` enum in the reference C++. We convert it to the
    // corresponding OWPML `numFormat` attribute string and compare.
    let expected_shape_str = number_shape_to_format_str(spec_level.numbershape);
    if let Some(expected_str) = expected_shape_str {
        if expected_str != para_head.num_format {
            errors.push(make_error(run, PARANUM_LEVEL_NUMBERSHAPE));
        }
    }
}

/// Map a `NumberShapeType` ordinal (from the DVC spec JSON) to the
/// corresponding OWPML `numFormat` attribute string.
///
/// The mapping mirrors the `NumberShapeType` enum in
/// `references/dvc/Source/DVCInterface.h`:
///
/// ```text
/// DIGIT                  = 0
/// CIRCLED_DIGIT          = 1
/// ROMAN_CAPITAL          = 2
/// ROMAN_SMALL            = 3
/// LATIN_CAPITAL          = 4
/// LATIN_SMALL            = 5
/// CIRCLED_LATIN_CAPITAL  = 6
/// CIRCLED_LATIN_SMALL    = 7
/// HANGUL_SYLLABLE        = 8
/// CIRCLED_HANGUL_SYLLABLE= 9
/// HANGUL_JAMO            = 10
/// CIRCLED_HANGUL_JAMO    = 11
/// HANGUL_PHONETIC        = 12
/// IDEOGRAPH              = 13
/// CIRCLED_IDEOGRAPH      = 14
/// DECAGON_CIRCLE         = 15
/// DECAGON_CIRCLE_HANJA   = 16
/// ```
///
/// Returns `None` for unknown ordinals (future-proofing).
fn number_shape_to_format_str(ordinal: u32) -> Option<&'static str> {
    match ordinal {
        0 => Some("DIGIT"),
        1 => Some("CIRCLED_DIGIT"),
        2 => Some("ROMAN_CAPITAL"),
        3 => Some("ROMAN_SMALL"),
        4 => Some("LATIN_CAPITAL"),
        5 => Some("LATIN_SMALL"),
        6 => Some("CIRCLED_LATIN_CAPITAL"),
        7 => Some("CIRCLED_LATIN_SMALL"),
        8 => Some("HANGUL_SYLLABLE"),
        9 => Some("CIRCLED_HANGUL_SYLLABLE"),
        10 => Some("HANGUL_JAMO"),
        11 => Some("CIRCLED_HANGUL_JAMO"),
        12 => Some("HANGUL_PHONETIC"),
        13 => Some("IDEOGRAPH"),
        14 => Some("CIRCLED_IDEOGRAPH"),
        15 => Some("DECAGON_CIRCLE"),
        16 => Some("DECAGON_CIRCLE_HANJA"),
        _ => None,
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
    // number_shape_to_format_str mapping
    // -------------------------------------------------------------------------

    #[test]
    fn number_shape_mapping_covers_all_known_ordinals() {
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
                number_shape_to_format_str(ordinal),
                Some(expected),
                "ordinal {ordinal} must map to {expected}"
            );
        }
        assert_eq!(
            number_shape_to_format_str(17),
            None,
            "unknown ordinal must return None"
        );
    }
}
