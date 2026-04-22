//! Integration tests for `checker::table::check`.
//!
//! Global constraints (from issue #11):
//! - `table_simple.hwpx` (pass) → zero 3000-range errors.
//! - `table_nested.hwpx` (fail) → at least one TABLE_IN_TABLE (3056) error.
//!
//! Standard-mode field coverage (issue #41):
//! - `size-width`, `size-height`, `fixed`, `margin-*`, `caption-*`,
//!   position enums, and flow flags all emit their corresponding
//!   `JID_TABLE_*` error codes when the document value violates the spec.
//! - Spec JSON round-trips the reference format (e.g.
//!   `{"min":a,"max":b}` range objects and bare numeric shorthand).

use std::path::PathBuf;

use hwp_dvc_core::checker::table::{
    TABLE_BORDER_CELL_SPACING, TABLE_BORDER_COLOR, TABLE_BORDER_SIZE, TABLE_BORDER_TYPE,
    TABLE_CAPTION_POSITION, TABLE_HDIRECTION, TABLE_HTYPE, TABLE_IN_TABLE, TABLE_MARGIN_BOTTOM,
    TABLE_MARGIN_LEFT, TABLE_MARGIN_RIGHT, TABLE_MARGIN_TOP, TABLE_NUM_VER_TYPE, TABLE_POS,
    TABLE_SIZE_FIXED, TABLE_SIZE_HEIGHT, TABLE_SIZE_WIDTH, TABLE_SOALLOW_OVERLAP,
    TABLE_SOFLOW_WITH_TEXT, TABLE_TREAT_AS_CHAR,
};
use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::spec::{DvcSpec, IntRange};

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

// ---------------------------------------------------------------------------
// Standard-mode field coverage (issue #41)
// ---------------------------------------------------------------------------
//
// The `table_simple.hwpx` fixture carries the following attribute
// values that these tests rely on:
//   <hp:tbl textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES"
//           rowCnt="2" colCnt="2" cellSpacing="0"
//           lock="0" noAdjust="0" numberingType="TABLE" …>
//     <hp:sz width="41954" height="2564" protect="0"/>
//     <hp:pos treatAsChar="1" flowWithText="1" allowOverlap="0"
//             holdAnchorAndSO="0" horzRelTo="COLUMN" horzAlign="LEFT"
//             vertRelTo="PARA" vertAlign="TOP" …/>
//     <hp:outMargin left="283" right="283" top="283" bottom="283"/>
//   </hp:tbl>
// The tests below flip one field at a time to trigger exactly one
// `JID_TABLE_*` error each.

#[test]
fn size_width_out_of_range_generates_size_width_error() {
    // Demand 0..=10 but the fixture's width is 41954.
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "table": { "size-width": { "min": 0, "max": 10 } } }"#)
        .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_SIZE_WIDTH),
        "expected 3001; got: {error_codes:?}"
    );
}

#[test]
fn size_height_out_of_range_generates_size_height_error() {
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(
        r#"{ "table": { "size-height": { "min": 100000, "max": 200000 } } }"#,
    )
    .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_SIZE_HEIGHT),
        "expected 3002; got: {error_codes:?}"
    );
}

#[test]
fn size_fixed_mismatch_generates_size_fixed_error() {
    // Fixture has protect="0", demand true.
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "table": { "fixed": true } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_SIZE_FIXED),
        "expected 3003; got: {error_codes:?}"
    );
}

#[test]
fn treat_as_char_mismatch_generates_treat_as_char_error() {
    // table_nested.hwpx has treatAsChar="0" on the outer table.
    let doc = open_doc("table_nested.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "treatAsChar": true } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_TREAT_AS_CHAR),
        "expected 3004; got: {error_codes:?}"
    );
}

#[test]
fn pos_mismatch_generates_pos_error() {
    // Fixture has textWrap="TOP_AND_BOTTOM" (pos=0); demand pos=3 (SQUARE).
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "table": { "pos": 3 } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_POS),
        "expected 3005; got: {error_codes:?}"
    );
}

#[test]
fn horizontal_type_mismatch_generates_htype_error() {
    // Fixture has horzRelTo="COLUMN" (=2); demand PAPER (=0).
    let doc = open_doc("table_simple.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "horizontal-type": 0 } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_HTYPE),
        "expected 3007; got: {error_codes:?}"
    );
}

#[test]
fn horizontal_direction_mismatch_generates_hdirection_error() {
    // Fixture has horzAlign="LEFT" (=0); demand CENTER (=1).
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "table": { "horizontal-direction": 1 } }"#)
        .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_HDIRECTION),
        "expected 3008; got: {error_codes:?}"
    );
}

#[test]
fn flow_with_text_mismatch_generates_soflow_with_text_error() {
    // Fixture has flowWithText="1"; demand false.
    let doc = open_doc("table_simple.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "soflowwithtext": false } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_SOFLOW_WITH_TEXT),
        "expected 3013; got: {error_codes:?}"
    );
}

#[test]
fn allow_overlap_mismatch_generates_soallow_overlap_error() {
    // Fixture has allowOverlap="0"; demand true.
    let doc = open_doc("table_simple.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "soallowoverlap": true } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_SOALLOW_OVERLAP),
        "expected 3014; got: {error_codes:?}"
    );
}

#[test]
fn numbering_type_mismatch_generates_numvertype_error() {
    // Fixture has numberingType="TABLE" (=2); demand NONE (=0).
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "table": { "numbertype": 0 } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_NUM_VER_TYPE),
        "expected 3020; got: {error_codes:?}"
    );
}

#[test]
fn margin_left_out_of_range_generates_margin_left_error() {
    // Fixture outMargin left=283; demand ≥1000.
    let doc = open_doc("table_simple.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "margin-left": { "min": 1000, "max": 2000 } } }"#)
            .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_MARGIN_LEFT),
        "expected 3022; got: {error_codes:?}"
    );
}

#[test]
fn margin_right_top_bottom_out_of_range_generates_errors() {
    let doc = open_doc("table_simple.hwpx");
    // All four outMargins are 283 in the fixture; require 1..=10 which none meet.
    let spec = DvcSpec::from_json_str(
        r#"{ "table": {
          "margin-right":  { "min": 1, "max": 10 },
          "margin-top":    { "min": 1, "max": 10 },
          "margin-bottom": { "min": 1, "max": 10 }
        } }"#,
    )
    .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(error_codes.contains(&TABLE_MARGIN_RIGHT));
    assert!(error_codes.contains(&TABLE_MARGIN_TOP));
    assert!(error_codes.contains(&TABLE_MARGIN_BOTTOM));
}

#[test]
fn border_cellspacing_out_of_range_generates_cellspacing_error() {
    // Fixture has cellSpacing="0"; demand {min:10,max:100}.
    let doc = open_doc("table_simple.hwpx");
    let spec = DvcSpec::from_json_str(
        r#"{ "table": { "border-cellspacing": { "min": 10, "max": 100 } } }"#,
    )
    .expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        error_codes.contains(&TABLE_BORDER_CELL_SPACING),
        "expected 3036; got: {error_codes:?}"
    );
}

#[test]
fn caption_checks_skipped_when_no_caption_in_document() {
    // The `table_simple` fixture has no <hp:caption>; caption-position
    // spec should not produce an error on a caption-less table.
    let doc = open_doc("table_simple.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "table": { "caption-position": 7 } }"#).expect("spec parses");
    let error_codes = run_table_check(&doc, &spec);
    assert!(
        !error_codes.contains(&TABLE_CAPTION_POSITION),
        "caption check must be a no-op when the document has no caption; got: {error_codes:?}"
    );
}

#[test]
fn int_range_shorthand_parses_as_min_equals_max() {
    // Reference C++ semantics: a bare scalar is equivalent to {min:v, max:v}.
    // The fixture width is 41954; demanding exact value `41954` passes,
    // demanding `99999` fails with 3001.
    let doc = open_doc("table_simple.hwpx");

    let exact_ok =
        DvcSpec::from_json_str(r#"{ "table": { "size-width": 41954 } }"#).expect("spec parses");
    assert!(!run_table_check(&doc, &exact_ok).contains(&TABLE_SIZE_WIDTH));

    let exact_fail =
        DvcSpec::from_json_str(r#"{ "table": { "size-width": 99999 } }"#).expect("spec parses");
    assert!(run_table_check(&doc, &exact_fail).contains(&TABLE_SIZE_WIDTH));
}

#[test]
fn int_range_deserializes_both_object_and_scalar_forms() {
    // Direct spec-level unit test: both `42` and `{"min":42,"max":42}`
    // should produce the same IntRange, and the range should accept 42
    // but reject 43.
    let scalar: IntRange = serde_json::from_str("42").unwrap();
    let object: IntRange = serde_json::from_str(r#"{"min":42,"max":42}"#).unwrap();
    assert_eq!(scalar, object);
    assert!(scalar.contains(42));
    assert!(!scalar.contains(43));
}
