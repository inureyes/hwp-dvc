//! Integration tests for `checker::bullet::check`.
//!
//! Global constraints (from issue #13):
//! - `bullet_allowed.hwpx` (pass) → zero 3300-range errors against
//!   `fixture_spec.json` (allow-list `"□○-•*"`).
//! - `bullet_disallowed.hwpx` (fail) → at least one BULLET_SHAPES (3304) error.

use std::path::PathBuf;

use hwp_dvc_core::checker::bullet::BULLET_SHAPES;
use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::spec::DvcSpec;

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

fn open_doc(name: &str) -> Document {
    let mut doc = Document::open(fixture_doc(name)).unwrap_or_else(|e| panic!("open {name}: {e}"));
    doc.parse().unwrap_or_else(|e| panic!("parse {name}: {e}"));
    doc
}

fn open_spec(name: &str) -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec(name)).unwrap_or_else(|e| panic!("spec {name}: {e}"))
}

fn run_bullet_check(doc: &Document, spec: &DvcSpec) -> Vec<u32> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope: OutputScope::default(),
    };
    checker
        .run()
        .expect("checker::run should not fail")
        .into_iter()
        .map(|e| e.error_code)
        .collect()
}

// ---------------------------------------------------------------------------
// bullet_allowed.hwpx — must produce zero 3300-range errors
// ---------------------------------------------------------------------------

#[test]
fn bullet_allowed_passes_with_fixture_spec() {
    let doc = open_doc("bullet_allowed.hwpx");
    let spec = open_spec("fixture_spec.json");
    let error_codes = run_bullet_check(&doc, &spec);

    let bullet_errors: Vec<u32> = error_codes
        .into_iter()
        .filter(|&c| (3300..3400).contains(&c))
        .collect();

    assert!(
        bullet_errors.is_empty(),
        "bullet_allowed.hwpx must produce zero 3300-range errors; got: {bullet_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// bullet_disallowed.hwpx — must produce at least one BULLET_SHAPES error
// ---------------------------------------------------------------------------

#[test]
fn bullet_disallowed_produces_bullet_shapes_error() {
    let doc = open_doc("bullet_disallowed.hwpx");
    let spec = open_spec("fixture_spec.json");
    let error_codes = run_bullet_check(&doc, &spec);

    assert!(
        error_codes.contains(&BULLET_SHAPES),
        "bullet_disallowed.hwpx must produce BULLET_SHAPES (3304); got: {error_codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Error-code constant sanity checks
// ---------------------------------------------------------------------------

#[test]
fn bullet_error_codes_have_correct_values() {
    use hwp_dvc_core::checker::bullet::{BULLET_CHECKTYPE, BULLET_CODE, BULLET_SHAPES};
    assert_eq!(BULLET_CHECKTYPE, 3302);
    assert_eq!(BULLET_CODE, 3303);
    assert_eq!(BULLET_SHAPES, 3304);
}

// ---------------------------------------------------------------------------
// Inline spec: allowed bullets produce no error
// ---------------------------------------------------------------------------

#[test]
fn inline_spec_with_allowed_bullets_produces_no_error() {
    let doc = open_doc("bullet_allowed.hwpx");
    let spec_json = r#"{ "bullet": { "bulletshapes": "□○-•*▲▶" } }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let error_codes = run_bullet_check(&doc, &spec);

    let bullet_errors: Vec<u32> = error_codes
        .into_iter()
        .filter(|&c| (3300..3400).contains(&c))
        .collect();

    assert!(
        bullet_errors.is_empty(),
        "broad allow-list must produce no bullet errors; got: {bullet_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Inline spec: strict allow-list and PUA bullets
// ---------------------------------------------------------------------------

#[test]
fn inline_spec_strict_allowlist_does_not_flag_pua_bullets() {
    // bullet_allowed.hwpx uses a Wingdings PUA bullet (U+F0A7).
    // Even a strict allow-list that doesn't include that character must
    // produce no error, because PUA code points are font-specific and
    // are exempt from shape validation.
    let doc = open_doc("bullet_allowed.hwpx");
    let spec_json = r#"{ "bullet": { "bulletshapes": "X" } }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");

    let error_codes = run_bullet_check(&doc, &spec);
    let bullet_errors: Vec<u32> = error_codes
        .into_iter()
        .filter(|&c| (3300..3400).contains(&c))
        .collect();

    // bullet_allowed.hwpx has only a PUA bullet (U+F0A7) which is skipped.
    // The strict spec must therefore produce no 3300-range errors for this doc.
    assert!(
        bullet_errors.is_empty(),
        "PUA bullet in strict allow-list must produce no errors; got: {bullet_errors:?}"
    );
}
