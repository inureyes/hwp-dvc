//! Integration tests for Korean error message localization (issue #16).
//!
//! These tests load real HWPX fixtures, run the validators, and assert that
//! `DvcErrorInfo.error_string` contains the expected Korean phrases.
//!
//! Global constraints per the issue spec:
//! - `charshape_fail_font.hwpx` → error_string must reference "글꼴".
//! - `macro_present.hwpx`       → error_string must reference "매크로".
//! - `hyperlink_external.hwpx`  → error_string must reference "하이퍼링크".

use std::path::PathBuf;

use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::spec::DvcSpec;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn fixture_doc(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

fn fixture_spec_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/specs");
    p.push(name);
    p
}

fn load_doc(name: &str) -> Document {
    let mut doc = Document::open(fixture_doc(name))
        .unwrap_or_else(|e| panic!("failed to open fixture '{name}': {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture '{name}': {e}"));
    doc
}

fn load_spec() -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec_path("fixture_spec.json"))
        .expect("fixture_spec.json must parse")
}

fn run_checker(doc: &Document, spec: &DvcSpec) -> Vec<hwp_dvc_core::checker::DvcErrorInfo> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope: OutputScope {
            all: true,
            table: true,
            table_detail: true,
            shape: true,
            style: true,
            hyperlink: true,
        },
    };
    checker.run().expect("Checker::run must not fail")
}

// ─────────────────────────────────────────────────────────────────────────────
// Global constraint 1: charshape_fail_font → "글꼴"
// ─────────────────────────────────────────────────────────────────────────────

/// When `charshape_fail_font.hwpx` is validated against a spec that forbids
/// its font, every CHARSHAPE_FONT (1004) error's `error_string` must contain
/// the Korean word for "font" (글꼴).
#[test]
fn charshape_fail_font_error_string_contains_font_word() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec();
    let errors = run_checker(&doc, &spec);

    let font_errors: Vec<_> = errors.iter().filter(|e| e.error_code == 1004).collect();

    assert!(
        !font_errors.is_empty(),
        "charshape_fail_font.hwpx must produce at least one CHARSHAPE_FONT (1004) error; \
         got no errors at all: {errors:?}"
    );

    for e in &font_errors {
        assert!(
            e.error_string.contains("글꼴"),
            "CHARSHAPE_FONT error_string must contain '글꼴' (font); \
             got: '{}'",
            e.error_string
        );
        assert!(
            !e.error_string.is_empty(),
            "error_string must not be empty for code 1004"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Global constraint 2: macro_present → "매크로"
// ─────────────────────────────────────────────────────────────────────────────

/// When `macro_present.hwpx` is validated against a spec with macro
/// `permission = false`, the MACRO_PERMISSION (7001) error's `error_string`
/// must contain "매크로".
#[test]
fn macro_present_error_string_contains_macro_word() {
    let doc = load_doc("macro_present.hwpx");
    let spec = load_spec();
    // fixture_spec.json has macro.permission = false
    assert!(
        spec.macro_.as_ref().map(|m| !m.permission).unwrap_or(false),
        "fixture_spec must have macro.permission == false for this test to be meaningful"
    );

    let errors = run_checker(&doc, &spec);

    let macro_errors: Vec<_> = errors.iter().filter(|e| e.error_code == 7001).collect();

    assert!(
        !macro_errors.is_empty(),
        "macro_present.hwpx with permission=false must emit MACRO_PERMISSION (7001) error; \
         got: {errors:?}"
    );

    for e in &macro_errors {
        assert!(
            e.error_string.contains("매크로"),
            "MACRO_PERMISSION error_string must contain '매크로'; got: '{}'",
            e.error_string
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Global constraint 3: hyperlink_external → "하이퍼링크"
// ─────────────────────────────────────────────────────────────────────────────

/// When `hyperlink_external.hwpx` is validated against a spec with hyperlink
/// `permission = false`, the HYPERLINK_PERMISSION (6901) error's `error_string`
/// must contain "하이퍼링크".
#[test]
fn hyperlink_external_error_string_contains_hyperlink_word() {
    let doc = load_doc("hyperlink_external.hwpx");
    let spec = load_spec();
    // fixture_spec.json has hyperlink.permission = false
    assert!(
        spec.hyperlink
            .as_ref()
            .map(|h| !h.permission)
            .unwrap_or(false),
        "fixture_spec must have hyperlink.permission == false for this test to be meaningful"
    );

    let errors = run_checker(&doc, &spec);

    let hyperlink_errors: Vec<_> = errors.iter().filter(|e| e.error_code == 6901).collect();

    assert!(
        !hyperlink_errors.is_empty(),
        "hyperlink_external.hwpx with permission=false must emit HYPERLINK_PERMISSION (6901) \
         error; got: {errors:?}"
    );

    for e in &hyperlink_errors {
        assert!(
            e.error_string.contains("하이퍼링크"),
            "HYPERLINK_PERMISSION error_string must contain '하이퍼링크'; got: '{}'",
            e.error_string
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Supplementary: error_string API tests
// ─────────────────────────────────────────────────────────────────────────────

/// `error_string` returns a non-empty Korean message for every documented code.
#[test]
fn error_string_returns_non_empty_for_all_documented_codes() {
    use hwp_dvc_core::error::{error_string, ErrorContext};

    let codes: &[u32] = &[
        1003, 1004, 1007, 1008, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 3004, 3033, 3034, 3035,
        3056, 3101, 3102, 3201, 3206, 3207, 3302, 3303, 3304, 3401, 3406, 3407, 3502, 6901, 7001,
    ];
    for &code in codes {
        let msg = error_string(code, ErrorContext::default());
        assert!(
            !msg.is_empty(),
            "error_string({code}) must return a non-empty message"
        );
        // All messages should contain at least some non-ASCII (Korean) content.
        assert!(
            msg.chars().any(|c| c > '\u{0080}'),
            "error_string({code}) must contain Korean characters; got: '{msg}'"
        );
    }
}

/// `error_string` for CHARSHAPE_FONT (1004) with a font name includes the name.
#[test]
fn error_string_font_includes_name() {
    use hwp_dvc_core::error::{error_string, ErrorContext};

    let ctx = ErrorContext::with_font("맑은 고딕");
    let msg = error_string(1004, ctx);
    assert!(
        msg.contains("맑은 고딕"),
        "error_string(1004) with font_name must include the font name; got: '{msg}'"
    );
    assert!(
        msg.contains("글꼴"),
        "error_string(1004) must contain '글꼴'; got: '{msg}'"
    );
}

/// `error_string` for BULLET_SHAPES (3304) with a bullet char includes the char.
#[test]
fn error_string_bullet_includes_char() {
    use hwp_dvc_core::error::{error_string, ErrorContext};

    let ctx = ErrorContext {
        bullet_char: Some("X"),
        ..ErrorContext::default()
    };
    let msg = error_string(3304, ctx);
    assert!(
        msg.contains('X'),
        "error_string(3304) with bullet_char must include the char; got: '{msg}'"
    );
    assert!(
        msg.contains("글머리표"),
        "error_string(3304) must contain '글머리표'; got: '{msg}'"
    );
}

/// `error_string` returns an empty string for an unknown code.
#[test]
fn error_string_unknown_code_returns_empty() {
    use hwp_dvc_core::error::{error_string, ErrorContext};

    let msg = error_string(9999, ErrorContext::default());
    assert!(
        msg.is_empty(),
        "error_string(9999) must return an empty string; got: '{msg}'"
    );
}
