//! Integration tests for `checker::para_num_bullet::check`.
//!
//! Uses the `paranum_simple.hwpx` fixture which contains paragraphs
//! formatted with paragraph numbering (heading type NUMBER).
//!
//! | Test                                        | Fixture               | Spec                          | Expected              |
//! |---------------------------------------------|-----------------------|-------------------------------|-----------------------|
//! | `paranum_simple_no_paranum_spec_zero_errors` | `paranum_simple.hwpx` | `fixture_spec.json` (no key)  | zero 3400-range errors |
//! | `paranum_simple_matching_spec_zero_errors`  | `paranum_simple.hwpx` | inline spec matching DIGIT    | zero 3406/3407 errors |
//! | `paranum_simple_mismatch_spec_emits_errors` | `paranum_simple.hwpx` | inline wrong numbertype       | ≥1 3406 error         |

use std::path::PathBuf;

use hwp_dvc_core::checker::{CheckLevel, Checker, DvcErrorInfo, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::para_num_bullet_codes::{
    PARANUM_LEVEL_NUMBERSHAPE, PARANUM_LEVEL_NUMBERTYPE,
};
use hwp_dvc_core::error::ErrorCode;
use hwp_dvc_core::spec::DvcSpec;

fn doc_fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

fn spec_fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/specs");
    p.push(name);
    p
}

fn parse(name: &str) -> Document {
    let mut doc = Document::open(doc_fixture(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

fn run_checker(doc: &Document, spec: &DvcSpec) -> Vec<DvcErrorInfo> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope: OutputScope::default(),
    };
    checker.run().expect("Checker::run must not fail")
}

/// True when `code` is in the 3400-range (ParaNumBullet category).
fn is_paranum_bullet(code: u32) -> bool {
    let base = ErrorCode::ParaNumBullet as u32;
    let next = ErrorCode::Style as u32;
    code >= base && code < next
}

// ---------------------------------------------------------------------------
// Global constraint: paranum_simple.hwpx against fixture_spec.json
// ---------------------------------------------------------------------------

/// `fixture_spec.json` has no `paranumbullet` key. Running the full checker
/// on `paranum_simple.hwpx` against it must produce zero 3400-range errors.
/// This is the primary constraint from Epic #1 Phase 3.
#[test]
fn paranum_simple_fixture_spec_produces_zero_3400_errors() {
    let doc = parse("paranum_simple.hwpx");
    let spec = DvcSpec::from_json_file(spec_fixture("fixture_spec.json"))
        .expect("fixture_spec.json must parse");

    let errs = run_checker(&doc, &spec);
    let paranum_errs: Vec<_> = errs
        .iter()
        .filter(|e| is_paranum_bullet(e.error_code))
        .collect();

    assert!(
        paranum_errs.is_empty(),
        "paranum_simple.hwpx against fixture_spec.json must produce \
         zero 3400-range errors; got: {paranum_errs:?}"
    );
}

// ---------------------------------------------------------------------------
// Matching spec: level 1 is DIGIT — paranum_simple uses DIGIT at level 1
// ---------------------------------------------------------------------------

/// When the spec matches the document's numbering exactly, no errors appear.
#[test]
fn paranum_simple_matching_spec_zero_errors() {
    let doc = parse("paranum_simple.hwpx");
    // paranum_simple has level 1 = DIGIT (numbershape=0).
    // Spec that declares exactly that must produce zero paranum errors.
    let spec = DvcSpec::from_json_str(
        r#"{
            "paranumbullet": {
                "leveltype": [
                    { "level": 1, "numbertype": "DIGIT", "numbershape": 0 }
                ]
            }
        }"#,
    )
    .expect("inline spec must parse");

    let errs = run_checker(&doc, &spec);
    let paranum_errs: Vec<_> = errs
        .iter()
        .filter(|e| is_paranum_bullet(e.error_code))
        .collect();

    assert!(
        paranum_errs.is_empty(),
        "matching spec must produce zero paranum errors; got: {paranum_errs:?}"
    );
}

// ---------------------------------------------------------------------------
// Mismatch spec: level 1 expects HANGUL_SYLLABLE but doc has DIGIT
// ---------------------------------------------------------------------------

/// When the spec's numbertype for level 1 differs from the document, a
/// PARANUM_LEVEL_NUMBERTYPE (3406) error must be emitted.
#[test]
fn paranum_simple_wrong_numbertype_emits_3406() {
    let doc = parse("paranum_simple.hwpx");
    let spec = DvcSpec::from_json_str(
        r#"{
            "paranumbullet": {
                "leveltype": [
                    { "level": 1, "numbertype": "HANGUL_SYLLABLE", "numbershape": 0 }
                ]
            }
        }"#,
    )
    .expect("inline spec must parse");

    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter()
            .any(|e| e.error_code == PARANUM_LEVEL_NUMBERTYPE),
        "wrong numbertype must emit PARANUM_LEVEL_NUMBERTYPE (3406) error"
    );
}

// ---------------------------------------------------------------------------
// Mismatch spec: numbershape expects HANGUL_SYLLABLE (8) but doc has DIGIT
// ---------------------------------------------------------------------------

/// When the spec's numbershape for level 1 differs from the document, a
/// PARANUM_LEVEL_NUMBERSHAPE (3407) error must be emitted.
#[test]
fn paranum_simple_wrong_numbershape_emits_3407() {
    let doc = parse("paranum_simple.hwpx");
    let spec = DvcSpec::from_json_str(
        r#"{
            "paranumbullet": {
                "leveltype": [
                    { "level": 1, "numbershape": 8 }
                ]
            }
        }"#,
    )
    .expect("inline spec must parse");

    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter()
            .any(|e| e.error_code == PARANUM_LEVEL_NUMBERSHAPE),
        "wrong numbershape must emit PARANUM_LEVEL_NUMBERSHAPE (3407) error"
    );
}

// ---------------------------------------------------------------------------
// Constant range guards
// ---------------------------------------------------------------------------

#[test]
fn paranum_level_numbertype_in_range() {
    let base = ErrorCode::ParaNumBullet as u32;
    let next = ErrorCode::Style as u32;
    assert!(
        PARANUM_LEVEL_NUMBERTYPE >= base,
        "PARANUM_LEVEL_NUMBERTYPE ({PARANUM_LEVEL_NUMBERTYPE}) must be >= base ({base})"
    );
    assert!(
        PARANUM_LEVEL_NUMBERTYPE < next,
        "PARANUM_LEVEL_NUMBERTYPE ({PARANUM_LEVEL_NUMBERTYPE}) must be < next ({next})"
    );
}

#[test]
fn paranum_level_numbershape_in_range() {
    let base = ErrorCode::ParaNumBullet as u32;
    let next = ErrorCode::Style as u32;
    assert!(
        PARANUM_LEVEL_NUMBERSHAPE >= base,
        "PARANUM_LEVEL_NUMBERSHAPE ({PARANUM_LEVEL_NUMBERSHAPE}) must be >= base ({base})"
    );
    assert!(
        PARANUM_LEVEL_NUMBERSHAPE < next,
        "PARANUM_LEVEL_NUMBERSHAPE ({PARANUM_LEVEL_NUMBERSHAPE}) must be < next ({next})"
    );
}
