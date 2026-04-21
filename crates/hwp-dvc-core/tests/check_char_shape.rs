//! Integration tests for `checker::char_shape::check` against real HWPX fixtures.
//!
//! # Fixture → spec pairing
//!
//! | Fixture                    | Spec                          | Expected outcome              |
//! |----------------------------|-------------------------------|-------------------------------|
//! | `charshape_pass.hwpx`      | `fixture_spec.json`           | zero 1000-range errors        |
//! | `charshape_fail_font.hwpx` | `fixture_spec.json`           | ≥ 1 `CHARSHAPE_FONT` (1004)   |
//! | `charshape_fail_ratio.hwpx`| `fixture_spec.json`           | ≥ 1 `CHARSHAPE_RATIO` (1007)  |
//!
//! The spec allow-list is `["함초롬바탕", "함초롬돋움"]` and ratio/spacing
//! are `100` / `0` respectively, matching the defaults used by Hancom Writer.

use std::path::PathBuf;

use hwp_dvc_core::checker::char_shape::{self, CHARSHAPE_FONT, CHARSHAPE_RATIO, CHARSHAPE_SPACING};
use hwp_dvc_core::checker::CheckLevel;
use hwp_dvc_core::document::Document;
use hwp_dvc_core::spec::DvcSpec;

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
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

// ──────────────────────────────────────────────────────────────────────────────
// Pass fixture: charshape_pass.hwpx against fixture_spec.json
// ──────────────────────────────────────────────────────────────────────────────

/// The pass fixture uses only 함초롬바탕 / 함초롬돋움, ratio=100, spacing=0 —
/// all values permitted by `fixture_spec.json`. Zero 1000-range errors expected.
#[test]
fn charshape_pass_produces_no_charshape_errors() {
    let doc = load_doc("charshape_pass.hwpx");
    let spec = load_spec("fixture_spec.json");

    let header = doc.header.as_ref().expect("header must be parsed");
    let charshape_spec = spec
        .charshape
        .as_ref()
        .expect("fixture_spec must have charshape");

    let errors = char_shape::check(charshape_spec, header, &doc.run_type_infos, CheckLevel::All);

    let charshape_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code >= 1000 && e.error_code < 2000)
        .collect();

    assert!(
        charshape_errors.is_empty(),
        "charshape_pass must produce zero 1000-range errors against fixture_spec; \
         got {} error(s): {:?}",
        charshape_errors.len(),
        charshape_errors,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Fail fixture: charshape_fail_font.hwpx
// ──────────────────────────────────────────────────────────────────────────────

/// The fail-font fixture contains a run with a font not in the spec allow-list.
/// At least one CHARSHAPE_FONT (1004) error must be reported.
#[test]
fn charshape_fail_font_reports_font_error() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let header = doc.header.as_ref().expect("header must be parsed");
    let charshape_spec = spec
        .charshape
        .as_ref()
        .expect("fixture_spec must have charshape");

    let errors = char_shape::check(charshape_spec, header, &doc.run_type_infos, CheckLevel::All);

    let font_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == CHARSHAPE_FONT)
        .collect();

    assert!(
        !font_errors.is_empty(),
        "charshape_fail_font must produce at least one CHARSHAPE_FONT (1004) error; \
         got zero. All errors reported: {:?}",
        errors,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Fail fixture: charshape_fail_ratio.hwpx
// ──────────────────────────────────────────────────────────────────────────────

/// The fail-ratio fixture contains a run whose ratio does not equal 100.
/// At least one CHARSHAPE_RATIO (1007) error must be reported.
#[test]
fn charshape_fail_ratio_reports_ratio_error() {
    let doc = load_doc("charshape_fail_ratio.hwpx");
    let spec = load_spec("fixture_spec.json");

    let header = doc.header.as_ref().expect("header must be parsed");
    let charshape_spec = spec
        .charshape
        .as_ref()
        .expect("fixture_spec must have charshape");

    let errors = char_shape::check(charshape_spec, header, &doc.run_type_infos, CheckLevel::All);

    let ratio_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == CHARSHAPE_RATIO)
        .collect();

    assert!(
        !ratio_errors.is_empty(),
        "charshape_fail_ratio must produce at least one CHARSHAPE_RATIO (1007) error; \
         got zero. All errors reported: {:?}",
        errors,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Checker::run integration
// ──────────────────────────────────────────────────────────────────────────────

/// Verify that `Checker::run` routes charshape validation correctly.
/// The pass fixture must produce zero charshape errors when run through
/// the top-level orchestrator.
#[test]
fn checker_run_routes_charshape_pass() {
    use hwp_dvc_core::checker::Checker;

    let doc = load_doc("charshape_pass.hwpx");
    let spec = load_spec("fixture_spec.json");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");

    let charshape_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code >= 1000 && e.error_code < 2000)
        .collect();

    assert!(
        charshape_errors.is_empty(),
        "Checker::run must produce zero charshape errors for the pass fixture; \
         got: {:?}",
        charshape_errors,
    );
}

/// Verify that `Checker::run` surfaces at least one charshape error for the
/// fail-font fixture.
#[test]
fn checker_run_routes_charshape_fail_font() {
    use hwp_dvc_core::checker::Checker;

    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");

    let font_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == CHARSHAPE_FONT)
        .collect();

    assert!(
        !font_errors.is_empty(),
        "Checker::run must surface CHARSHAPE_FONT errors for charshape_fail_font; \
         got: {:?}",
        errors,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Spacing check: pass case
// ──────────────────────────────────────────────────────────────────────────────

/// The pass fixture has spacing=0 matching the spec. No CHARSHAPE_SPACING error.
#[test]
fn charshape_pass_has_no_spacing_error() {
    let doc = load_doc("charshape_pass.hwpx");
    let spec = load_spec("fixture_spec.json");

    let header = doc.header.as_ref().expect("header must be parsed");
    let charshape_spec = spec
        .charshape
        .as_ref()
        .expect("fixture_spec must have charshape");

    let errors = char_shape::check(charshape_spec, header, &doc.run_type_infos, CheckLevel::All);

    let spacing_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == CHARSHAPE_SPACING)
        .collect();

    assert!(
        spacing_errors.is_empty(),
        "charshape_pass must have no CHARSHAPE_SPACING errors; got: {spacing_errors:?}"
    );
}
