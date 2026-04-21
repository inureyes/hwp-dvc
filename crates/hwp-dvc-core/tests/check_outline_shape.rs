//! Integration tests for the `CheckOutlineShape` validator.
//!
//! Each test follows the full pipeline:
//!   `Document::open` → `Document::parse` → `Checker::new` → `Checker::run`
//!
//! # Fixtures used
//!
//! | Fixture                   | Spec                    | Expected outcome                             |
//! |---------------------------|-------------------------|----------------------------------------------|
//! | `outline_multilevel.hwpx` | `hancom_test.json`      | zero OutlineShape-range (3200+) errors       |
//! | `outline_multilevel.hwpx` | inline wrong spec       | ≥ 1 OUTLINESHAPE_LEVEL_NUMBERSHAPE error     |
//!
//! The `outline_multilevel.hwpx` fixture has levels 1–10 whose `numFormat`
//! and template text match exactly the entries in `hancom_test.json`.
//! The "wrong spec" sub-test mutates one level's `numbershape` to a value
//! that diverges from the document — that is a **synthetic fail case**
//! authored directly in this test rather than a separate fixture file.
//!
//! # Global constraint
//!
//! Per the issue orchestrator's constraint, `outline_multilevel.hwpx`
//! must produce **zero** errors in the 3200-range (OutlineShape) when
//! checked against `hancom_test.json`.

use std::path::PathBuf;

use hwp_dvc_core::checker::{Checker, DvcErrorInfo};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::outline_shape_codes::{
    OUTLINESHAPE_LEVEL_NUMBERSHAPE, OUTLINESHAPE_LEVEL_NUMBERTYPE,
};
use hwp_dvc_core::error::ErrorCode;
use hwp_dvc_core::spec::DvcSpec;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

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

/// Load and run the full checker pipeline. Returns all errors.
fn run_check(doc_name: &str, spec_name: &str) -> Vec<DvcErrorInfo> {
    let spec = DvcSpec::from_json_file(spec_fixture(spec_name))
        .unwrap_or_else(|e| panic!("failed to parse spec {spec_name}: {e}"));

    let mut doc = Document::open(doc_fixture(doc_name))
        .unwrap_or_else(|e| panic!("failed to open {doc_name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse {doc_name}: {e}"));

    Checker::new(&spec, &doc)
        .run()
        .unwrap_or_else(|e| panic!("Checker::run failed for {doc_name}: {e}"))
}

/// Partition errors into OutlineShape-range (3200–3299) and other.
fn partition_outline_errors(errors: &[DvcErrorInfo]) -> (Vec<&DvcErrorInfo>, Vec<&DvcErrorInfo>) {
    let base = ErrorCode::OutlineShape as u32;
    let next = ErrorCode::Bullet as u32;
    errors
        .iter()
        .partition(|e| e.error_code >= base && e.error_code < next)
}

// ---------------------------------------------------------------------------
// Pass case: outline_multilevel + hancom_test.json → zero 3200-range errors
// ---------------------------------------------------------------------------

/// The multilevel outline fixture must produce zero OutlineShape-range errors
/// when validated against `hancom_test.json` — the spec perfectly matches the
/// fixture's outline layout (levels 1–10, numFormat+template for each).
#[test]
fn outline_multilevel_passes_hancom_test_spec() {
    let errors = run_check("outline_multilevel.hwpx", "hancom_test.json");
    let (outline_errors, _) = partition_outline_errors(&errors);
    assert!(
        outline_errors.is_empty(),
        "outline_multilevel.hwpx must produce zero OutlineShape errors against hancom_test.json; \
         got {outline_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Fail case: inline wrong spec → fires OUTLINESHAPE_LEVEL_NUMBERSHAPE
//
// Synthetic fail case: we use the same fixture but replace level 1's
// `numbershape` with 8 (HANGUL_SYLLABLE). The fixture's level 1 is DIGIT
// (ordinal 0), so this must produce an OUTLINESHAPE_LEVEL_NUMBERSHAPE error.
// ---------------------------------------------------------------------------

/// When the spec demands a different `numbershape` for level 1, the validator
/// must emit at least one `OUTLINESHAPE_LEVEL_NUMBERSHAPE` error.
///
/// **Synthetic fail case**: the fixture has level 1 = DIGIT (ordinal 0), but
/// the inline spec specifies ordinal 8 (HANGUL_SYLLABLE).
#[test]
fn wrong_numbershape_for_level1_fires_error() {
    // Build a spec where level 1 has the wrong numbershape (8 = HANGUL_SYLLABLE
    // instead of 0 = DIGIT).
    let spec_json = r#"{
        "outlineshape": {
            "leveltype": [
                { "level": 1, "numbershape": 8 }
            ]
        }
    }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("inline spec parses");

    let mut doc =
        Document::open(doc_fixture("outline_multilevel.hwpx")).expect("outline_multilevel opens");
    doc.parse().expect("outline_multilevel parses");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run succeeds");

    let (outline_errors, _) = partition_outline_errors(&errors);
    assert!(
        outline_errors
            .iter()
            .any(|e| e.error_code == OUTLINESHAPE_LEVEL_NUMBERSHAPE),
        "wrong numbershape for level 1 must fire OUTLINESHAPE_LEVEL_NUMBERSHAPE; \
         got {outline_errors:?}"
    );
}

/// When the spec demands a different `numbertype` template for level 2, the
/// validator must emit at least one `OUTLINESHAPE_LEVEL_NUMBERTYPE` error.
///
/// **Synthetic fail case**: the fixture level 2 template is `"^2."` but the
/// spec specifies `"^2)"`.
#[test]
fn wrong_numbertype_for_level2_fires_error() {
    let spec_json = r#"{
        "outlineshape": {
            "leveltype": [
                { "level": 2, "numbertype": "^2)", "numbershape": 8 }
            ]
        }
    }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("inline spec parses");

    let mut doc =
        Document::open(doc_fixture("outline_multilevel.hwpx")).expect("outline_multilevel opens");
    doc.parse().expect("outline_multilevel parses");

    let errors = Checker::new(&spec, &doc).run().expect("run succeeds");

    let (outline_errors, _) = partition_outline_errors(&errors);
    assert!(
        outline_errors
            .iter()
            .any(|e| e.error_code == OUTLINESHAPE_LEVEL_NUMBERTYPE),
        "wrong numbertype for level 2 must fire OUTLINESHAPE_LEVEL_NUMBERTYPE; \
         got {outline_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// No-spec guard: missing outlineshape key → zero outline errors
// ---------------------------------------------------------------------------

/// When the spec has no `outlineshape` key at all, the outline checker must
/// be a no-op — zero OutlineShape-range errors regardless of the document.
#[test]
fn no_outlineshape_spec_produces_zero_outline_errors() {
    let errors = run_check("outline_multilevel.hwpx", "fixture_spec.json");
    let (outline_errors, _) = partition_outline_errors(&errors);
    assert!(
        outline_errors.is_empty(),
        "fixture_spec.json has no outlineshape key; must produce zero outline errors; \
         got {outline_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Error code range sanity checks
// ---------------------------------------------------------------------------

/// `OUTLINESHAPE_LEVEL_NUMBERTYPE` must be in the OutlineShape range [3200, 3300).
#[test]
fn outlineshape_level_numbertype_constant_in_range() {
    let base = ErrorCode::OutlineShape as u32;
    let next = ErrorCode::Bullet as u32;
    assert!(
        OUTLINESHAPE_LEVEL_NUMBERTYPE >= base,
        "OUTLINESHAPE_LEVEL_NUMBERTYPE ({OUTLINESHAPE_LEVEL_NUMBERTYPE}) must be >= {base}"
    );
    assert!(
        OUTLINESHAPE_LEVEL_NUMBERTYPE < next,
        "OUTLINESHAPE_LEVEL_NUMBERTYPE ({OUTLINESHAPE_LEVEL_NUMBERTYPE}) must be < {next}"
    );
}

/// `OUTLINESHAPE_LEVEL_NUMBERSHAPE` must be in the OutlineShape range [3200, 3300).
#[test]
fn outlineshape_level_numbershape_constant_in_range() {
    let base = ErrorCode::OutlineShape as u32;
    let next = ErrorCode::Bullet as u32;
    assert!(
        OUTLINESHAPE_LEVEL_NUMBERSHAPE >= base,
        "OUTLINESHAPE_LEVEL_NUMBERSHAPE ({OUTLINESHAPE_LEVEL_NUMBERSHAPE}) must be >= {base}"
    );
    assert!(
        OUTLINESHAPE_LEVEL_NUMBERSHAPE < next,
        "OUTLINESHAPE_LEVEL_NUMBERSHAPE ({OUTLINESHAPE_LEVEL_NUMBERSHAPE}) must be < {next}"
    );
}
