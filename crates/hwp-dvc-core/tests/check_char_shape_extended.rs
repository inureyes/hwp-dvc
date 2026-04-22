//! Extended integration tests for `checker::char_shape::check` — issue #39.
//!
//! This suite verifies the 37 new `JID_CHAR_SHAPE_*` fields added by issue #39
//! beyond the original four (langtype, font, ratio, spacing).
//!
//! # Fixture strategy
//!
//! All tests in this file work against the existing `charshape_pass.hwpx`
//! fixture (authored via 한글, committed for the original charshape issue).
//! We do not require new `.hwpx` files to be authored in 한글 for the pure
//! boolean-flag / numeric-range checks; instead, we exercise the validator
//! logic directly with synthetic in-memory `CharShape` and `CharShapeSpec`
//! objects.
//!
//! Tests that require a real fixture document (baseline pass) reuse
//! `charshape_pass.hwpx` to confirm zero 1000-range errors when all new
//! spec fields are absent (i.e., `None`).
//!
//! The decoration-fail sub-case is verified via the unit-test-level path
//! in `checker/char_shape/mod.rs`. A fixture-level bold-fail variant
//! (`charshape_fail_bold.hwpx`) cannot be synthesized via XML patching
//! because the OWPML `<hh:bold/>` element presence/absence requires
//! re-authoring in 한글 — see `tests/fixtures/docs/README.md` for the
//! authoring procedure when this fixture is eventually needed.
//!
//! # Coverage summary
//!
//! | Group               | Codes      | Strategy                          |
//! |---------------------|------------|-----------------------------------|
//! | Size                | 1001,1005,1006,1030 | Synthetic CharShape + Spec  |
//! | Decoration flags    | 1009–1018  | Synthetic CharShape + Spec        |
//! | Shadow detail       | 1019–1022  | TODO — deferred (no CharShape field)|
//! | Underline detail    | 1023–1025  | TODO — deferred (no CharShape field)|
//! | Strikeout detail    | 1026–1027  | TODO — deferred (no CharShape field)|
//! | Outline detail      | 1028       | TODO — deferred (no CharShape field)|
//! | Misc                | 1029,1031  | Synthetic CharShape + Spec        |
//! | Border presence     | 1032       | Synthetic CharShape + Spec        |
//! | Background          | 1037–1039  | TODO — deferred (no CharShape field)|
//! | Baseline pass       | all        | charshape_pass.hwpx               |

use std::collections::HashMap;
use std::path::PathBuf;

use hwp_dvc_core::checker::char_shape::{
    self, CHARSHAPE_BG_BORDER, CHARSHAPE_BOLD, CHARSHAPE_EMBOSS, CHARSHAPE_EMPTYSPACE,
    CHARSHAPE_ENGRAVE, CHARSHAPE_FONTSIZE, CHARSHAPE_ITALIC, CHARSHAPE_KERNING, CHARSHAPE_OUTLINE,
    CHARSHAPE_POINT, CHARSHAPE_POSITION, CHARSHAPE_RSIZE, CHARSHAPE_SHADOW, CHARSHAPE_STRIKEOUT,
    CHARSHAPE_SUBSCRIPT, CHARSHAPE_SUPSCRIPT, CHARSHAPE_UNDERLINE,
};
use hwp_dvc_core::checker::CheckLevel;
use hwp_dvc_core::document::header::{CharShape, FontFace, FontLang, HeaderTables, LangTuple};
use hwp_dvc_core::document::{Document, RunTypeInfo};
use hwp_dvc_core::spec::{CharShapeBorderSpec, CharShapeSpec, DvcSpec};

// ──────────────────────────────────────────────────────────────────────────────
// Shared test helpers
// ──────────────────────────────────────────────────────────────────────────────

fn fixture_doc(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

fn fixture_spec(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/specs");
    p.push(name);
    p
}

fn load_doc(name: &str) -> Document {
    let mut doc = Document::open(fixture_doc(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

fn load_spec(name: &str) -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec(name))
        .unwrap_or_else(|e| panic!("failed to load spec {name}: {e}"))
}

/// Build a minimal `HeaderTables` with a single `CharShape` and one `FontFace`.
fn tables_with(cs: CharShape) -> HeaderTables {
    let mut char_shapes = HashMap::new();
    char_shapes.insert(cs.id, cs);
    HeaderTables {
        char_shapes,
        font_faces: vec![FontFace {
            lang: FontLang::Hangul,
            fonts: {
                let mut m = HashMap::new();
                m.insert(0, "함초롬바탕".to_string());
                m.insert(1, "함초롬돋움".to_string());
                m
            },
        }],
        ..Default::default()
    }
}

fn run_for(char_pr_id_ref: u32) -> RunTypeInfo {
    RunTypeInfo {
        char_pr_id_ref,
        text: "테스트".to_string(),
        ..Default::default()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Baseline pass test using real fixture
// ──────────────────────────────────────────────────────────────────────────────

/// When all new spec fields are `None`, the existing `charshape_pass.hwpx`
/// must still produce zero 1000-range errors.
#[test]
fn charshape_pass_with_extended_spec_none_fields_produces_no_errors() {
    let doc = load_doc("charshape_pass.hwpx");
    let base_spec = load_spec("fixture_spec.json");

    let header = doc.header.as_ref().expect("header must be parsed");
    let base_charshape = base_spec.charshape.as_ref().expect("charshape must be set");

    // Build an extended spec that carries all the new None fields on top of the
    // existing font/ratio/spacing constraints.
    let extended_spec = CharShapeSpec {
        langtype: base_charshape.langtype.clone(),
        font: base_charshape.font.clone(),
        ratio: base_charshape.ratio,
        spacing: base_charshape.spacing,
        // All new fields absent — must not trigger any new errors.
        ..Default::default()
    };

    let errors = char_shape::check(&extended_spec, header, &doc.run_type_infos, CheckLevel::All);

    let charshape_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code >= 1000 && e.error_code < 2000)
        .collect();

    assert!(
        charshape_errors.is_empty(),
        "charshape_pass with all new spec fields = None must produce zero 1000-range errors; \
         got {} error(s): {:?}",
        charshape_errors.len(),
        charshape_errors,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Size group: fontsize (1001), rsize (1005), position (1006), point (1030)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn fontsize_match_no_error() {
    let mut cs = CharShape {
        id: 0,
        height: 1000,
        ..Default::default()
    };
    cs.font_ref.set(FontLang::Hangul, 0);
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        fontsize: Some(1000),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_FONTSIZE),
        "fontsize match must produce no CHARSHAPE_FONTSIZE error; got: {errors:?}"
    );
}

#[test]
fn fontsize_mismatch_produces_error() {
    let mut cs = CharShape {
        id: 0,
        height: 1200,
        ..Default::default()
    };
    cs.font_ref.set(FontLang::Hangul, 0);
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        fontsize: Some(1000),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_FONTSIZE),
        "expected CHARSHAPE_FONTSIZE error; got: {errors:?}"
    );
}

#[test]
fn rsize_mismatch_produces_error() {
    let mut cs = CharShape {
        id: 0,
        ..Default::default()
    };
    cs.font_ref.set(FontLang::Hangul, 0);
    let mut rel_sz = LangTuple::<u32>::default();
    rel_sz.set(FontLang::Hangul, 80);
    cs.rel_sz = rel_sz;
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        rsize: Some(100),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_RSIZE),
        "expected CHARSHAPE_RSIZE error; got: {errors:?}"
    );
}

#[test]
fn position_mismatch_produces_error() {
    let mut cs = CharShape {
        id: 0,
        ..Default::default()
    };
    cs.font_ref.set(FontLang::Hangul, 0);
    let mut offset = LangTuple::<i32>::default();
    offset.set(FontLang::Hangul, 5);
    cs.offset = offset;
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        position: Some(0),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_POSITION),
        "expected CHARSHAPE_POSITION error; got: {errors:?}"
    );
}

#[test]
fn point_match_no_error() {
    let cs = CharShape {
        id: 0,
        height: 1000,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        point: Some(10.0),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_POINT),
        "point match must produce no CHARSHAPE_POINT error; got: {errors:?}"
    );
}

#[test]
fn point_mismatch_produces_error() {
    let cs = CharShape {
        id: 0,
        height: 1200,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        point: Some(10.0),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_POINT),
        "expected CHARSHAPE_POINT error (12pt vs 10pt spec); got: {errors:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Text decoration toggles (1009–1018)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn decoration_flags_match_produce_no_errors() {
    let cs = CharShape {
        id: 0,
        bold: false,
        italic: false,
        underline: false,
        strikeout: false,
        outline: false,
        emboss: false,
        engrave: false,
        shadow: false,
        supscript: false,
        subscript: false,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        bold: Some(false),
        italic: Some(false),
        underline: Some(false),
        strikeout: Some(false),
        outline: Some(false),
        emboss: Some(false),
        engrave: Some(false),
        shadow: Some(false),
        supscript: Some(false),
        subscript: Some(false),
        ..Default::default()
    };

    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    let decoration_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code >= 1009 && e.error_code <= 1018)
        .collect();
    assert!(
        decoration_errors.is_empty(),
        "all-false decoration flags must produce no 1009-1018 errors; got: {decoration_errors:?}"
    );
}

#[test]
fn bold_true_when_spec_false_produces_bold_error() {
    let cs = CharShape {
        id: 0,
        bold: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        bold: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_BOLD),
        "expected CHARSHAPE_BOLD error; got: {errors:?}"
    );
}

#[test]
fn italic_mismatch_produces_italic_error() {
    let cs = CharShape {
        id: 0,
        italic: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        italic: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_ITALIC),
        "expected CHARSHAPE_ITALIC error; got: {errors:?}"
    );
}

#[test]
fn underline_mismatch_produces_underline_error() {
    let cs = CharShape {
        id: 0,
        underline: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        underline: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_UNDERLINE),
        "expected CHARSHAPE_UNDERLINE error; got: {errors:?}"
    );
}

#[test]
fn strikeout_mismatch_produces_strikeout_error() {
    let cs = CharShape {
        id: 0,
        strikeout: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        strikeout: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_STRIKEOUT),
        "expected CHARSHAPE_STRIKEOUT error; got: {errors:?}"
    );
}

#[test]
fn outline_mismatch_produces_outline_error() {
    let cs = CharShape {
        id: 0,
        outline: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        outline: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_OUTLINE),
        "expected CHARSHAPE_OUTLINE error; got: {errors:?}"
    );
}

#[test]
fn emboss_mismatch_produces_emboss_error() {
    let cs = CharShape {
        id: 0,
        emboss: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        emboss: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_EMBOSS),
        "expected CHARSHAPE_EMBOSS error; got: {errors:?}"
    );
}

#[test]
fn engrave_mismatch_produces_engrave_error() {
    let cs = CharShape {
        id: 0,
        engrave: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        engrave: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_ENGRAVE),
        "expected CHARSHAPE_ENGRAVE error; got: {errors:?}"
    );
}

#[test]
fn shadow_mismatch_produces_shadow_error() {
    let cs = CharShape {
        id: 0,
        shadow: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        shadow: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_SHADOW),
        "expected CHARSHAPE_SHADOW error; got: {errors:?}"
    );
}

#[test]
fn supscript_mismatch_produces_supscript_error() {
    let cs = CharShape {
        id: 0,
        supscript: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        supscript: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_SUPSCRIPT),
        "expected CHARSHAPE_SUPSCRIPT error; got: {errors:?}"
    );
}

#[test]
fn subscript_mismatch_produces_subscript_error() {
    let cs = CharShape {
        id: 0,
        subscript: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        subscript: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_SUBSCRIPT),
        "expected CHARSHAPE_SUBSCRIPT error; got: {errors:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Misc: emptyspace (1029), kerning (1031)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn emptyspace_mismatch_produces_error() {
    let cs = CharShape {
        id: 0,
        use_font_space: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        emptyspace: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_EMPTYSPACE),
        "expected CHARSHAPE_EMPTYSPACE error; got: {errors:?}"
    );
}

#[test]
fn emptyspace_match_no_error() {
    let cs = CharShape {
        id: 0,
        use_font_space: false,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        emptyspace: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_EMPTYSPACE),
        "no CHARSHAPE_EMPTYSPACE error expected; got: {errors:?}"
    );
}

#[test]
fn kerning_mismatch_produces_error() {
    let cs = CharShape {
        id: 0,
        use_kerning: true,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        kerning: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_KERNING),
        "expected CHARSHAPE_KERNING error; got: {errors:?}"
    );
}

#[test]
fn kerning_match_no_error() {
    let cs = CharShape {
        id: 0,
        use_kerning: false,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        kerning: Some(false),
        ..Default::default()
    };
    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_KERNING),
        "no CHARSHAPE_KERNING error expected; got: {errors:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Border presence (1032)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn bg_border_absent_with_spec_produces_bg_border_error() {
    let cs = CharShape {
        id: 0,
        border_fill_id_ref: 0,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        border: Some(CharShapeBorderSpec {
            position: Some(1),
            bordertype: Some(1),
            size: Some(0.12),
            color: Some("#000000".to_string()),
        }),
        ..Default::default()
    };

    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().any(|e| e.error_code == CHARSHAPE_BG_BORDER),
        "expected CHARSHAPE_BG_BORDER error when border_fill_id_ref=0; got: {errors:?}"
    );
}

#[test]
fn bg_border_present_with_spec_no_bg_border_error() {
    let cs = CharShape {
        id: 0,
        border_fill_id_ref: 1,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        border: Some(CharShapeBorderSpec::default()),
        ..Default::default()
    };

    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_BG_BORDER),
        "no CHARSHAPE_BG_BORDER error expected when border_fill present; got: {errors:?}"
    );
}

#[test]
fn bg_border_spec_absent_no_bg_border_error() {
    let cs = CharShape {
        id: 0,
        border_fill_id_ref: 0,
        ..Default::default()
    };
    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        border: None,
        ..Default::default()
    };

    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    assert!(
        errors.iter().all(|e| e.error_code != CHARSHAPE_BG_BORDER),
        "no CHARSHAPE_BG_BORDER error expected when spec.border is None; got: {errors:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Multi-field spec: all active, all matching — zero errors
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn all_new_fields_matching_produce_no_errors() {
    let mut cs = CharShape {
        id: 0,
        height: 1000,
        bold: false,
        italic: false,
        underline: false,
        strikeout: false,
        outline: false,
        emboss: false,
        engrave: false,
        shadow: false,
        supscript: false,
        subscript: false,
        use_font_space: false,
        use_kerning: false,
        border_fill_id_ref: 1, // border present
        ..Default::default()
    };
    let mut rel_sz = LangTuple::<u32>::default();
    rel_sz.set(FontLang::Hangul, 100);
    cs.rel_sz = rel_sz;
    let mut offset = LangTuple::<i32>::default();
    offset.set(FontLang::Hangul, 0);
    cs.offset = offset;

    let tables = tables_with(cs);
    let runs = vec![run_for(0)];

    let spec = CharShapeSpec {
        fontsize: Some(1000),
        rsize: Some(100),
        position: Some(0),
        point: Some(10.0),
        bold: Some(false),
        italic: Some(false),
        underline: Some(false),
        strikeout: Some(false),
        outline: Some(false),
        emboss: Some(false),
        engrave: Some(false),
        shadow: Some(false),
        supscript: Some(false),
        subscript: Some(false),
        emptyspace: Some(false),
        kerning: Some(false),
        border: Some(CharShapeBorderSpec::default()),
        ..Default::default()
    };

    let errors = char_shape::check(&spec, &tables, &runs, CheckLevel::All);
    let new_field_errors: Vec<_> = errors
        .iter()
        .filter(|e| {
            matches!(
                e.error_code,
                1001 | 1005 | 1006 | 1009..=1018 | 1029..=1032
            )
        })
        .collect();

    assert!(
        new_field_errors.is_empty(),
        "all matching new fields must produce no errors; got: {new_field_errors:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Spec field JSON round-trip: extended fields survive serialization
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn extended_charshape_spec_fields_roundtrip_json() {
    // Build the JSON programmatically to avoid raw-string delimiter conflicts
    // with the '#' character in color values.
    let json = serde_json::json!({
        "charshape": {
            "font": ["함초롬바탕"],
            "fontsize": 1000,
            "rsize": 100,
            "position": 0,
            "point": 10.0_f64,
            "bold": false,
            "italic": false,
            "underline": false,
            "strikeout": false,
            "outline": false,
            "emboss": false,
            "engrave": false,
            "shadow": false,
            "supscript": false,
            "subscript": false,
            "shadow-x": 10,
            "shadow-y": 10,
            "shadow-color": "#C0C0C0",
            "underline-position": "BOTTOM",
            "underline-shape": "SOLID",
            "underline-color": "#000000",
            "strikeout-shape": "SOLID",
            "strikeout-color": "#000000",
            "outlinetype": "NONE",
            "emptyspace": false,
            "kerning": false,
            "bg-color": "#FFFFFF",
            "bg-pattoncolor": "#000000",
            "bg-pattontype": "NONE"
        }
    })
    .to_string();

    let spec =
        DvcSpec::from_json_str(&json).expect("extended charshape spec JSON must parse cleanly");
    let cs = spec.charshape.unwrap();

    assert_eq!(cs.fontsize, Some(1000));
    assert_eq!(cs.rsize, Some(100));
    assert_eq!(cs.position, Some(0));
    // serde_json round-trips f64 so we check with a tolerance
    assert!(
        (cs.point.unwrap_or(0.0) - 10.0_f64).abs() < 0.01,
        "point must be ~10.0; got {:?}",
        cs.point
    );
    assert_eq!(cs.bold, Some(false));
    assert_eq!(cs.italic, Some(false));
    assert_eq!(cs.shadow_x, Some(10));
    assert_eq!(cs.shadow_y, Some(10));
    assert_eq!(cs.shadow_color.as_deref(), Some("#C0C0C0"));
    assert_eq!(cs.underline_position.as_deref(), Some("BOTTOM"));
    assert_eq!(cs.underline_shape.as_deref(), Some("SOLID"));
    assert_eq!(cs.underline_color.as_deref(), Some("#000000"));
    assert_eq!(cs.strikeout_shape.as_deref(), Some("SOLID"));
    assert_eq!(cs.strikeout_color.as_deref(), Some("#000000"));
    assert_eq!(cs.outlinetype.as_deref(), Some("NONE"));
    assert_eq!(cs.emptyspace, Some(false));
    assert_eq!(cs.kerning, Some(false));
    assert_eq!(cs.bg_color.as_deref(), Some("#FFFFFF"));
    assert_eq!(cs.bg_pattoncolor.as_deref(), Some("#000000"));
    assert_eq!(cs.bg_pattontype.as_deref(), Some("NONE"));
}
