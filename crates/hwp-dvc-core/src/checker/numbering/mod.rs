//! Shared level-walker helpers for numbering-based validators.
//!
//! Both [`crate::checker::outline_shape`] and
//! [`crate::checker::para_num_bullet`] traverse `Numbering → ParaHead` level
//! entries and validate them against a `leveltype` spec array. The structural
//! traversal logic and the number-shape ordinal mapping are identical in both
//! checkers; only the error codes differ (3204/3205 for outline-shape vs
//! 3404/3405 for para-num-bullet).
//!
//! This module provides:
//! - [`num_shape_to_str`] — maps a `NumberShapeType` ordinal to the OWPML
//!   `numFormat` string. Returns `None` for unknown ordinals.
//! - [`check_level_sequence`] — validates that a spec's `leveltype` array has
//!   sequential 1-indexed `level` values and that its length matches the
//!   document's `Numbering.para_heads` count. Pushes zero, one, or two
//!   [`DvcErrorInfo`] records using caller-supplied error codes.
//! - [`make_error`] — constructs a [`DvcErrorInfo`] from a representative
//!   [`RunTypeInfo`] and an error code, filling all standard metadata fields.
//!
//! # Port note
//!
//! Extracted from the private helpers in `Checker::CheckOutlineShape` and
//! `Checker::CheckParaNumBullet` (`references/dvc/Checker.cpp`). Consolidation
//! is tracked by issue #45 of Epic #38.

use crate::checker::DvcErrorInfo;
use crate::document::{header::Numbering, RunTypeInfo};
use crate::error::ErrorContext;
use crate::spec::LevelType;

// ---------------------------------------------------------------------------
// Number-shape ordinal mapping
// ---------------------------------------------------------------------------

/// Map a `NumberShapeType` ordinal (from a DVC spec JSON file) to the
/// corresponding OWPML `numFormat` attribute string.
///
/// The mapping mirrors `NumberShapeType` in
/// `references/dvc/Source/DVCInterface.h`:
///
/// ```text
/// DIGIT                   = 0
/// CIRCLED_DIGIT           = 1
/// ROMAN_CAPITAL           = 2
/// ROMAN_SMALL             = 3
/// LATIN_CAPITAL           = 4
/// LATIN_SMALL             = 5
/// CIRCLED_LATIN_CAPITAL   = 6
/// CIRCLED_LATIN_SMALL     = 7
/// HANGUL_SYLLABLE         = 8
/// CIRCLED_HANGUL_SYLLABLE = 9
/// HANGUL_JAMO             = 10
/// CIRCLED_HANGUL_JAMO     = 11
/// HANGUL_PHONETIC         = 12
/// IDEOGRAPH               = 13
/// CIRCLED_IDEOGRAPH       = 14
/// DECAGON_CIRCLE          = 15
/// DECAGON_CIRCLE_HANJA    = 16
/// ```
///
/// Returns `None` for unknown ordinals so callers can skip the comparison
/// rather than treating every unknown ordinal as a mismatch.
#[must_use]
pub fn num_shape_to_str(ordinal: u32) -> Option<&'static str> {
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

// ---------------------------------------------------------------------------
// Level-sequence checker
// ---------------------------------------------------------------------------

/// Validate the `leveltype` array of a spec against a document [`Numbering`].
///
/// Two checks are performed (mirroring `getLevelType` / `CheckOutlineShape` /
/// `CheckParaNumBullet` in `references/dvc/Checker.cpp`):
///
/// 1. **Level-count check** (`leveltype_code`): when `spec_levels` is
///    non-empty, its length must equal `numbering.para_heads.len()`.
///    A length mismatch pushes one error with `leveltype_code`.
///
/// 2. **Level-index check** (`leveltype_level_code`): each `LevelType` entry
///    at index `i` must have `level == i + 1` (sequential, 1-indexed).
///    The first out-of-sequence entry pushes one error with
///    `leveltype_level_code` and the scan stops.
///
/// Both codes are supplied by the caller so the same logic can be used for
/// `OUTLINESHAPE_LEVELTYPE` (3204) + `OUTLINESHAPE_LEVELTYPE_LEVEL` (3205)
/// and for `PARANUM_LEVELTYPE` (3404) + `PARANUM_LEVELTYPE_LEVEL` (3405).
///
/// Returns the number of errors pushed (0, 1, or 2).
pub fn check_level_sequence(
    run: &RunTypeInfo,
    numbering: &Numbering,
    spec_levels: &[LevelType],
    leveltype_code: u32,
    leveltype_level_code: u32,
    errors: &mut Vec<DvcErrorInfo>,
) -> usize {
    let before = errors.len();

    // --- level-count check ---
    if !spec_levels.is_empty() && spec_levels.len() != numbering.para_heads.len() {
        errors.push(make_error(run, leveltype_code));
    }

    // --- level-index check ---
    for (idx, lt) in spec_levels.iter().enumerate() {
        let expected_level = (idx as u32) + 1;
        if lt.level != expected_level {
            errors.push(make_error(run, leveltype_level_code));
            // One error per numbering template is sufficient; stop after the
            // first out-of-sequence entry to avoid flooding.
            break;
        }
    }

    errors.len() - before
}

// ---------------------------------------------------------------------------
// Error construction
// ---------------------------------------------------------------------------

/// Build a [`DvcErrorInfo`] from a representative run and an error code,
/// filling every standard metadata field.
///
/// This helper is shared between `outline_shape` and `para_num_bullet` so
/// that both emit identically-shaped [`DvcErrorInfo`] records.
#[must_use]
pub fn make_error(run: &RunTypeInfo, error_code: u32) -> DvcErrorInfo {
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

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::types::shapes::{Numbering, ParaHead};
    use crate::document::RunTypeInfo;
    use crate::spec::LevelType;

    // Sentinel error codes for tests — use values that don't collide with real codes.
    const CODE_LT: u32 = 9901;
    const CODE_LL: u32 = 9902;

    fn run() -> RunTypeInfo {
        RunTypeInfo::default()
    }

    fn numbering_with_n_levels(n: usize) -> Numbering {
        Numbering {
            id: 1,
            start: 0,
            para_heads: (1..=(n as u32))
                .map(|lvl| ParaHead {
                    level: lvl,
                    ..Default::default()
                })
                .collect(),
        }
    }

    fn levels_sequential(n: u32) -> Vec<LevelType> {
        (1..=n)
            .map(|l| LevelType {
                level: l,
                numbertype: None,
                numbershape: 0,
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // num_shape_to_str
    // -----------------------------------------------------------------------

    #[test]
    fn num_shape_known_ordinals() {
        assert_eq!(num_shape_to_str(0), Some("DIGIT"));
        assert_eq!(num_shape_to_str(8), Some("HANGUL_SYLLABLE"));
        assert_eq!(num_shape_to_str(16), Some("DECAGON_CIRCLE_HANJA"));
    }

    #[test]
    fn num_shape_unknown_ordinal_returns_none() {
        assert_eq!(num_shape_to_str(17), None);
        assert_eq!(num_shape_to_str(99), None);
    }

    // -----------------------------------------------------------------------
    // check_level_sequence — no errors
    // -----------------------------------------------------------------------

    #[test]
    fn empty_spec_levels_emits_no_errors() {
        let num = numbering_with_n_levels(3);
        let mut errors: Vec<DvcErrorInfo> = Vec::new();
        let pushed = check_level_sequence(&run(), &num, &[], CODE_LT, CODE_LL, &mut errors);
        assert_eq!(pushed, 0);
        assert!(errors.is_empty());
    }

    #[test]
    fn matching_count_and_sequential_levels_emits_no_errors() {
        let num = numbering_with_n_levels(3);
        let levels = levels_sequential(3);
        let mut errors: Vec<DvcErrorInfo> = Vec::new();
        let pushed = check_level_sequence(&run(), &num, &levels, CODE_LT, CODE_LL, &mut errors);
        assert_eq!(pushed, 0);
        assert!(errors.is_empty());
    }

    // -----------------------------------------------------------------------
    // check_level_sequence — level-count mismatch
    // -----------------------------------------------------------------------

    #[test]
    fn count_mismatch_emits_leveltype_error() {
        let num = numbering_with_n_levels(1); // doc has 1 level
        let levels = levels_sequential(2); // spec has 2 levels
        let mut errors: Vec<DvcErrorInfo> = Vec::new();
        check_level_sequence(&run(), &num, &levels, CODE_LT, CODE_LL, &mut errors);
        assert!(
            errors.iter().any(|e| e.error_code == CODE_LT),
            "count mismatch must push CODE_LT"
        );
    }

    // -----------------------------------------------------------------------
    // check_level_sequence — level-index mismatch
    // -----------------------------------------------------------------------

    #[test]
    fn non_sequential_level_emits_leveltype_level_error() {
        let num = numbering_with_n_levels(2);
        // Spec entry at index 1 has level=3, expected level=2.
        let levels = vec![
            LevelType {
                level: 1,
                numbertype: None,
                numbershape: 0,
            },
            LevelType {
                level: 3,
                numbertype: None,
                numbershape: 0,
            },
        ];
        let mut errors: Vec<DvcErrorInfo> = Vec::new();
        check_level_sequence(&run(), &num, &levels, CODE_LT, CODE_LL, &mut errors);
        assert!(
            errors.iter().any(|e| e.error_code == CODE_LL),
            "non-sequential level must push CODE_LL"
        );
    }

    #[test]
    fn only_one_level_level_error_per_numbering() {
        // Two out-of-sequence entries — but only one error should be emitted.
        let num = numbering_with_n_levels(3);
        let levels = vec![
            LevelType {
                level: 2,
                numbertype: None,
                numbershape: 0,
            }, // wrong (idx=0 → expected 1)
            LevelType {
                level: 3,
                numbertype: None,
                numbershape: 0,
            }, // also wrong
            LevelType {
                level: 4,
                numbertype: None,
                numbershape: 0,
            }, // also wrong
        ];
        let mut errors: Vec<DvcErrorInfo> = Vec::new();
        check_level_sequence(&run(), &num, &levels, CODE_LT, CODE_LL, &mut errors);
        let ll_errors: Vec<_> = errors.iter().filter(|e| e.error_code == CODE_LL).collect();
        assert_eq!(ll_errors.len(), 1, "at most one CODE_LL error per call");
    }
}
