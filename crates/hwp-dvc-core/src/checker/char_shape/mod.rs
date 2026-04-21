//! Character-shape validator (`CheckCharShape` in the reference C++).
//!
//! Mirrors `Checker::CheckCharShape` and
//! `Checker::CheckCharShapeToCheckList` in `references/dvc/Checker.cpp`.
//!
//! # Algorithm
//!
//! 1. For every `CharShape` in the header table, compare it against
//!    the `CharShapeSpec` fields that are enabled in the spec.
//! 2. On mismatch, record an intermediate `ErrorInfo` keyed by the
//!    `CharShape` id and the `JID_CHAR_SHAPE_*` error code.
//! 3. Walk the [`RunTypeInfo`] stream and, for each run whose
//!    `char_pr_id_ref` matches a recorded error, emit a
//!    [`DvcErrorInfo`].
//!
//! # Error codes
//!
//! The four codes exposed by this issue are:
//!
//! | Constant               | Value | `JID_*` reference            |
//! |------------------------|-------|------------------------------|
//! | `CHARSHAPE_LANGTYPE`   | 1003  | `JID_CHAR_SHAPE_LANG`        |
//! | `CHARSHAPE_FONT`       | 1004  | `JID_CHAR_SHAPE_FONT`        |
//! | `CHARSHAPE_RATIO`      | 1007  | `JID_CHAR_SHAPE_RATIO`       |
//! | `CHARSHAPE_SPACING`    | 1008  | `JID_CHAR_SHAPE_SPACING`     |
//!
//! Additional spec fields (bold, italic, shadow, …) are TODO-annotated
//! below with the matching `JID_CHAR_SHAPE_*` constant.

use crate::checker::{CheckLevel, DvcErrorInfo};
use crate::document::header::{CharShape, FontFace, HeaderTables};
use crate::document::RunTypeInfo;
use crate::error::ErrorContext;
use crate::spec::CharShapeSpec;

// ──────────────────────────────────────────────────────────────────────────────
// Error codes — mirrors JsonModel.h JID_CHAR_SHAPE_* offsets
// (base 1000 = JID_CHAR_SHAPE per references/dvc/Source/JsonModel.h).
// ──────────────────────────────────────────────────────────────────────────────

/// Error code for a lang-type mismatch (`JID_CHAR_SHAPE_LANG = 1003`).
pub const CHARSHAPE_LANGTYPE: u32 = 1003;

/// Error code for a font-name not in the allow-list (`JID_CHAR_SHAPE_FONT = 1004`).
pub const CHARSHAPE_FONT: u32 = 1004;

/// Error code for a ratio value outside the allowed range
/// (`JID_CHAR_SHAPE_RATIO = 1007`).
pub const CHARSHAPE_RATIO: u32 = 1007;

/// Error code for a spacing value outside the allowed range
/// (`JID_CHAR_SHAPE_SPACING = 1008`).
pub const CHARSHAPE_SPACING: u32 = 1008;

// TODO: JID_CHAR_SHAPE_FONTSIZE = 1001 — font size range check
// TODO: JID_CHAR_SHAPE_LANGSET  = 1002 — langset object validation
// TODO: JID_CHAR_SHAPE_RSIZE    = 1005 — relative size range check
// TODO: JID_CHAR_SHAPE_POSITION = 1006 — character position check
// TODO: JID_CHAR_SHAPE_BOLD     = 1009 — bold flag check
// TODO: JID_CHAR_SHAPE_ITALIC   = 1010 — italic flag check
// TODO: JID_CHAR_SHAPE_UNDERLINE = 1011 — underline flag check
// TODO: JID_CHAR_SHAPE_STRIKEOUT = 1012 — strikeout flag check
// TODO: JID_CHAR_SHAPE_OUTLINE  = 1013 — outline flag check
// TODO: JID_CHAR_SHAPE_EMBOSS   = 1014 — emboss flag check
// TODO: JID_CHAR_SHAPE_ENGRAVE  = 1015 — engrave flag check
// TODO: JID_CHAR_SHAPE_SHADOW   = 1016 — shadow flag check
// TODO: JID_CHAR_SHAPE_SUPSCRIPT = 1017 — superscript flag check
// TODO: JID_CHAR_SHAPE_SUBSCRIPT = 1018 — subscript flag check
// TODO: JID_CHAR_SHAPE_SHADOWTYPE = 1019 — shadow type check
// TODO: JID_CHAR_SHAPE_SHADOW_X  = 1020 — shadow X direction check
// TODO: JID_CHAR_SHAPE_SHADOW_Y  = 1021 — shadow Y direction check
// TODO: JID_CHAR_SHAPE_SHADOW_COLOR = 1022 — shadow color check
// TODO: JID_CHAR_SHAPE_KERNING   = 1031 — kerning flag check
// TODO: JID_CHAR_SHAPE_BG_BORDER = 1032 — background border check
// TODO: JID_CHAR_SHAPE_BG_COLOR  = 1037 — background color check

// ──────────────────────────────────────────────────────────────────────────────
// Internal intermediate record
// ──────────────────────────────────────────────────────────────────────────────

/// Intermediate error: a (char_shape_id, error_code) pair produced by
/// `check_char_shape_to_check_list` before the RunTypeInfo fan-out.
struct ErrorInfo {
    /// The `CharShape.id` that violated the rule.
    char_pr_id: u32,
    /// One of the `CHARSHAPE_*` constants.
    error_code: u32,
    /// Optional context for message formatting (e.g. the offending font name).
    font_name: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public entry point
// ──────────────────────────────────────────────────────────────────────────────

/// Validate character shapes against the spec and return any errors.
///
/// Mirrors `Checker::CheckCharShape` + `CheckCharShapeToCheckList` from
/// `references/dvc/Checker.cpp` (lines 87–742).
///
/// # Parameters
///
/// * `spec` — the `[charshape]` section of the DVC spec.
/// * `tables` — the parsed header tables (provides `char_shapes` and `font_faces`).
/// * `run_type_infos` — the flattened run stream for the document.
/// * `level` — [`CheckLevel::Simple`] stops at the first error per run;
///   [`CheckLevel::All`] collects every violation.
pub fn check(
    spec: &CharShapeSpec,
    tables: &HeaderTables,
    run_type_infos: &[RunTypeInfo],
    level: CheckLevel,
) -> Vec<DvcErrorInfo> {
    // Phase 1: for each CharShape entry in the header, collect violations.
    let mut intermediate: Vec<ErrorInfo> = Vec::new();
    for cs in tables.char_shapes.values() {
        check_char_shape_to_check_list(cs, spec, &tables.font_faces, &mut intermediate);
    }

    if intermediate.is_empty() {
        return Vec::new();
    }

    // Phase 2: fan-out — for each intermediate error, find every RunTypeInfo
    // whose char_pr_id_ref matches and emit a DvcErrorInfo.
    // Mirrors the inner loop in Checker::CheckCharShape (lines 117–126).
    let mut results: Vec<DvcErrorInfo> = Vec::new();

    'outer: for err in &intermediate {
        for run in run_type_infos {
            if run.char_pr_id_ref == err.char_pr_id {
                let ctx = ErrorContext {
                    font_name: err.font_name.as_deref(),
                    ..ErrorContext::default()
                };
                results.push(DvcErrorInfo {
                    char_pr_id_ref: run.char_pr_id_ref,
                    para_pr_id_ref: run.para_pr_id_ref,
                    text: run.text.clone(),
                    page_no: run.page_no,
                    line_no: run.line_no,
                    error_code: err.error_code,
                    table_id: run.table_id,
                    is_in_table: run.is_in_table,
                    is_in_table_in_table: run.is_in_table_in_table,
                    table_row: run.table_row,
                    table_col: run.table_col,
                    is_in_shape: run.is_in_shape,
                    use_hyperlink: run.is_hyperlink,
                    use_style: run.is_style,
                    error_string: crate::error::error_string(err.error_code, ctx),
                });

                if level == CheckLevel::Simple {
                    break 'outer;
                }
            }
        }
    }

    results
}

// ──────────────────────────────────────────────────────────────────────────────
// Per-CharShape validation
// ──────────────────────────────────────────────────────────────────────────────

/// Check a single [`CharShape`] against the spec, appending any violations
/// to `errors`.
///
/// Mirrors `Checker::CheckCharShapeToCheckList` (Checker.cpp lines 538–742).
fn check_char_shape_to_check_list(
    cs: &CharShape,
    spec: &CharShapeSpec,
    font_faces: &[FontFace],
    errors: &mut Vec<ErrorInfo>,
) {
    // JID_CHAR_SHAPE_LANG (1003) — langtype check.
    //
    // The reference stores langtype as a raw integer (LangType enum).
    // The Rust spec carries it as an optional string (e.g. "대표").
    // The current document model does not decode langtype into a resolved
    // string; the validator only emits an error when the spec explicitly
    // requests a specific langtype value and that value cannot be
    // confirmed.  Because the document-side LangType is not yet decoded,
    // we emit a CHARSHAPE_LANGTYPE error iff the spec specifies a
    // non-empty langtype string — this is a conservative placeholder that
    // matches the reference behaviour of flagging the charshape when the
    // field is active but cannot be verified.
    //
    // TODO: decode the hangul/latin/… langtype from CharShape once the
    // header parser exposes it (issue #2 addendum).
    if let Some(ref _langtype) = spec.langtype {
        // langtype verification requires the document-side lang enum which is
        // not yet decoded from the OWPML charPr attributes.  Leave the
        // check unimplemented and add a TODO rather than incorrectly
        // flagging every charshape. No error emitted here until the
        // document model is extended.
        //
        // TODO(#9): implement langtype match once CharShape carries the
        // resolved LangType value. Error code: CHARSHAPE_LANGTYPE (1003).
        let _ = _langtype; // suppress unused-variable warning
    }

    // JID_CHAR_SHAPE_FONT (1004) — font allow-list check.
    //
    // Uses CharShape::font_names which deduplicates while preserving
    // first-seen order (per the PR that landed for issue #2).  The spec
    // supplies an allow-list; a CharShape is non-compliant iff NONE of
    // its resolved font names appear in the allow-list.
    if !spec.font.is_empty() {
        let doc_fonts = cs.font_names(font_faces);
        let any_allowed = doc_fonts.iter().any(|name| spec.font.contains(name));
        if !any_allowed {
            // Capture the first document font name for the error message.
            let font_name = doc_fonts.into_iter().next();
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_FONT,
                font_name,
            });
        }
    }

    // JID_CHAR_SHAPE_RATIO (1007) — ratio range check.
    //
    // The spec carries a single `ratio` integer that acts as both min and
    // max (exact match). The document's `CharShape.ratio` is a `LangTuple<u32>`;
    // we check the Hangul slot (index 0), which is what the reference
    // `charPr->charPrInfo.ratio` stores (only hangul, per RCharShape.h).
    if let Some(spec_ratio) = spec.ratio {
        let doc_ratio = cs.ratio.values[0]; // Hangul slot
        if doc_ratio != spec_ratio as u32 {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_RATIO,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SPACING (1008) — spacing range check.
    //
    // The spec carries a single `spacing` integer (exact match).
    // The document's `CharShape.spacing` is a `LangTuple<i32>`; we check
    // the Hangul slot matching the reference implementation.
    if let Some(spec_spacing) = spec.spacing {
        let doc_spacing = cs.spacing.values[0]; // Hangul slot
        if doc_spacing != spec_spacing {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_SPACING,
                font_name: None,
            });
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::document::header::{CharShape, FontFace, FontLang, HeaderTables, LangTuple};
    use crate::spec::CharShapeSpec;

    fn make_font_faces(hangul_fonts: &[(u32, &str)]) -> Vec<FontFace> {
        let mut face = FontFace {
            lang: FontLang::Hangul,
            fonts: HashMap::new(),
        };
        for (id, name) in hangul_fonts {
            face.fonts.insert(*id, name.to_string());
        }
        vec![face]
    }

    fn make_char_shape(
        id: u32,
        font_ref_hangul: u32,
        ratio_hangul: u32,
        spacing_hangul: i32,
    ) -> CharShape {
        let mut font_ref = LangTuple::<u32>::default();
        font_ref.set(FontLang::Hangul, font_ref_hangul);

        let mut ratio = LangTuple::<u32>::default();
        ratio.set(FontLang::Hangul, ratio_hangul);

        let mut spacing = LangTuple::<i32>::default();
        spacing.set(FontLang::Hangul, spacing_hangul);

        CharShape {
            id,
            font_ref,
            ratio,
            spacing,
            ..Default::default()
        }
    }

    fn make_tables_with(cs: CharShape, font_faces: Vec<FontFace>) -> HeaderTables {
        let mut char_shapes = HashMap::new();
        char_shapes.insert(cs.id, cs);
        HeaderTables {
            char_shapes,
            font_faces,
            ..Default::default()
        }
    }

    fn make_run(char_pr_id_ref: u32) -> RunTypeInfo {
        RunTypeInfo {
            char_pr_id_ref,
            text: "test".to_string(),
            ..Default::default()
        }
    }

    // ── font allow-list ────────────────────────────────────────────────────────

    #[test]
    fn font_allowed_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string(), "함초롬돋움".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.is_empty(),
            "no errors expected for compliant charshape; got: {errors:?}"
        );
    }

    #[test]
    fn font_not_in_allowlist_produces_font_error() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_FONT),
            "expected CHARSHAPE_FONT error; got: {errors:?}"
        );
    }

    #[test]
    fn empty_font_list_in_spec_skips_font_check() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec![],
            ratio: None,
            spacing: None,
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.is_empty(),
            "empty font list must skip the font check"
        );
    }

    // ── ratio ──────────────────────────────────────────────────────────────────

    #[test]
    fn ratio_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        let ratio_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == CHARSHAPE_RATIO)
            .collect();
        assert!(
            ratio_errors.is_empty(),
            "no CHARSHAPE_RATIO error expected; got: {ratio_errors:?}"
        );
    }

    #[test]
    fn ratio_mismatch_produces_ratio_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 150, 0); // ratio = 150, spec wants 100
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_RATIO),
            "expected CHARSHAPE_RATIO error; got: {errors:?}"
        );
    }

    // ── spacing ────────────────────────────────────────────────────────────────

    #[test]
    fn spacing_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        let spacing_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == CHARSHAPE_SPACING)
            .collect();
        assert!(
            spacing_errors.is_empty(),
            "no CHARSHAPE_SPACING error expected"
        );
    }

    #[test]
    fn spacing_mismatch_produces_spacing_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 10); // spacing = 10, spec wants 0
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_SPACING),
            "expected CHARSHAPE_SPACING error; got: {errors:?}"
        );
    }

    // ── check level ───────────────────────────────────────────────────────────

    #[test]
    fn simple_level_stops_at_first_error() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        // Create two runs both referencing the same non-compliant charshape.
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0), make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: None,
            spacing: None,
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::Simple);
        assert_eq!(
            errors.len(),
            1,
            "Simple level must stop after the first error"
        );
    }

    #[test]
    fn all_level_collects_all_errors() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        // Two runs both violating the font rule.
        let runs = vec![make_run(0), make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: None,
            spacing: None,
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert_eq!(
            errors.len(),
            2,
            "All level must collect errors for every run"
        );
    }

    // ── run metadata propagation ───────────────────────────────────────────────

    #[test]
    fn error_carries_run_metadata() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(42, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let run = RunTypeInfo {
            char_pr_id_ref: 42,
            para_pr_id_ref: 7,
            text: "Hello".to_string(),
            is_in_table: true,
            table_id: 3,
            table_row: 1,
            table_col: 2,
            ..Default::default()
        };
        let runs = vec![run];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: None,
            spacing: None,
            langtype: None,
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert_eq!(errors.len(), 1);
        let e = &errors[0];
        assert_eq!(e.char_pr_id_ref, 42);
        assert_eq!(e.para_pr_id_ref, 7);
        assert_eq!(e.text, "Hello");
        assert!(e.is_in_table);
        assert_eq!(e.table_id, 3);
        assert_eq!(e.table_row, 1);
        assert_eq!(e.table_col, 2);
        assert!(
            e.error_code >= 1000 && e.error_code < 2000,
            "error code must be in CharShape range"
        );
    }
}
