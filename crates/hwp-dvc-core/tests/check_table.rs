//! Integration tests for `checker::table::check`.
//!
//! Global constraints (from issue #11):
//! - `table_simple.hwpx` (pass) → zero 3000-range errors.
//! - `table_nested.hwpx` (fail) → at least one TABLE_IN_TABLE (3056) error.

use std::path::PathBuf;

use hwp_dvc_core::checker::table::{
    TABLE_BORDER_COLOR, TABLE_BORDER_SIZE, TABLE_BORDER_TYPE, TABLE_IN_TABLE,
};
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

fn run_table_check(doc: &Document, spec: &DvcSpec) -> Vec<u32> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope: OutputScope {
            all: false,
            table: true,
            table_detail: false,
            shape: false,
            style: false,
            hyperlink: false,
        },
    };
    checker
        .run()
        .expect("checker::run should not fail")
        .into_iter()
        .map(|e| e.error_code)
        .collect()
}

// ---------------------------------------------------------------------------
// table_simple.hwpx — must produce zero 3000-range errors
// ---------------------------------------------------------------------------

#[test]
fn table_simple_passes_with_fixture_spec() {
    let doc = open_doc("table_simple.hwpx");
    let spec = open_spec("fixture_spec.json");
    let error_codes = run_table_check(&doc, &spec);

    let table_errors: Vec<u32> = error_codes
        .into_iter()
        .filter(|&c| (3000..4000).contains(&c))
        .collect();

    assert!(
        table_errors.is_empty(),
        "table_simple.hwpx must produce zero 3000-range errors; got: {table_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// table_nested.hwpx — must produce at least one TABLE_IN_TABLE error
// ---------------------------------------------------------------------------

#[test]
fn table_nested_produces_table_in_table_error() {
    let doc = open_doc("table_nested.hwpx");
    let spec = open_spec("fixture_spec.json");
    let error_codes = run_table_check(&doc, &spec);

    assert!(
        error_codes.contains(&TABLE_IN_TABLE),
        "table_nested.hwpx must produce TABLE_IN_TABLE (3056); got: {error_codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Error-code constant sanity checks
// ---------------------------------------------------------------------------

#[test]
fn table_error_codes_have_correct_values() {
    assert_eq!(TABLE_BORDER_TYPE, 3033);
    assert_eq!(TABLE_BORDER_SIZE, 3034);
    assert_eq!(TABLE_BORDER_COLOR, 3035);
    assert_eq!(TABLE_IN_TABLE, 3056);
}

// ---------------------------------------------------------------------------
// Border mismatch detection
// ---------------------------------------------------------------------------

#[test]
fn wrong_bordertype_spec_generates_border_type_error() {
    let doc = open_doc("table_simple.hwpx");
    // Use a spec that demands a DASH border (type=2) — the fixture has SOLID.
    let spec_json = r#"{
        "table": {
            "border": [
                { "position": 1, "bordertype": 2, "size": 0.12, "color": 0 }
            ]
        }
    }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);

    assert!(
        error_codes.contains(&TABLE_BORDER_TYPE),
        "wrong bordertype in spec must produce TABLE_BORDER_TYPE (3033); got: {error_codes:?}"
    );
}

#[test]
fn wrong_border_size_generates_border_size_error() {
    let doc = open_doc("table_simple.hwpx");
    // Demand 0.5 mm but the fixture has 0.12 mm.
    let spec_json = r#"{
        "table": {
            "border": [
                { "position": 1, "bordertype": 1, "size": 0.5, "color": 0 }
            ]
        }
    }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);

    assert!(
        error_codes.contains(&TABLE_BORDER_SIZE),
        "wrong border size must produce TABLE_BORDER_SIZE (3034); got: {error_codes:?}"
    );
}

#[test]
fn wrong_border_color_generates_border_color_error() {
    let doc = open_doc("table_simple.hwpx");
    // Demand red (0xFF0000) but the fixture has black (0).
    let spec_json = r#"{
        "table": {
            "border": [
                { "position": 1, "bordertype": 1, "size": 0.12, "color": 16711680 }
            ]
        }
    }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);

    assert!(
        error_codes.contains(&TABLE_BORDER_COLOR),
        "wrong border color must produce TABLE_BORDER_COLOR (3035); got: {error_codes:?}"
    );
}
