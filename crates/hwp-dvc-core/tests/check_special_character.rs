//! Integration tests for the `CheckSpecialCharacter` validator (issue #8).
//!
//! Two HWPX fixtures exercise the full pipeline:
//!
//! | Fixture                      | Expectation                                          |
//! |------------------------------|------------------------------------------------------|
//! | `specialchar_pass.hwpx`      | zero 3100-range errors (spec min=32, max=1\_048\_575) |
//! | `specialchar_fail_ctrl.hwpx` | ≥1 error in the 3100 range (raw BEL U+0007 injected)  |

use std::path::PathBuf;

use hwp_dvc_core::checker::Checker;
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::{SPECIALCHAR_MAX, SPECIALCHAR_MIN};
use hwp_dvc_core::spec::DvcSpec;

fn fixture_doc(name: &str) -> Document {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    let mut doc = Document::open(&p).unwrap_or_else(|e| panic!("open {name}: {e}"));
    doc.parse().unwrap_or_else(|e| panic!("parse {name}: {e}"));
    doc
}

fn specialchar_spec() -> DvcSpec {
    DvcSpec::from_json_str(r#"{"specialcharacter":{"minimum":32,"maximum":1048575}}"#)
        .expect("spec parse")
}

fn specialchar_error_codes() -> std::ops::RangeInclusive<u32> {
    3100..=3199
}

#[test]
fn specialchar_pass_produces_no_3100_errors() {
    let doc = fixture_doc("specialchar_pass.hwpx");
    let spec = specialchar_spec();
    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run ok");

    let range_errors: Vec<_> = errors
        .iter()
        .filter(|e| specialchar_error_codes().contains(&e.error_code))
        .collect();

    assert!(
        range_errors.is_empty(),
        "specialchar_pass.hwpx must produce zero 3100-range errors; got: {:?}",
        range_errors
            .iter()
            .map(|e| (e.error_code, e.text.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn specialchar_fail_ctrl_produces_at_least_one_3100_error() {
    let doc = fixture_doc("specialchar_fail_ctrl.hwpx");
    let spec = specialchar_spec();
    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run ok");

    let range_errors: Vec<_> = errors
        .iter()
        .filter(|e| specialchar_error_codes().contains(&e.error_code))
        .collect();

    assert!(
        !range_errors.is_empty(),
        "specialchar_fail_ctrl.hwpx must produce at least one 3100-range error; got none.\n\
         all errors: {:?}",
        errors.iter().map(|e| e.error_code).collect::<Vec<_>>()
    );

    // All emitted errors must use a recognised sub-code.
    for e in &range_errors {
        assert!(
            e.error_code == SPECIALCHAR_MIN || e.error_code == SPECIALCHAR_MAX,
            "unexpected sub-code {} (expected {} or {})",
            e.error_code,
            SPECIALCHAR_MIN,
            SPECIALCHAR_MAX,
        );
    }
}

#[test]
fn no_spec_produces_no_errors() {
    // When the spec has no `specialcharacter` section the validator is
    // entirely skipped and the error list stays empty.
    let doc = fixture_doc("specialchar_fail_ctrl.hwpx");
    let spec = DvcSpec::from_json_str("{}").expect("empty spec");
    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run ok");

    let range_errors: Vec<_> = errors
        .iter()
        .filter(|e| specialchar_error_codes().contains(&e.error_code))
        .collect();

    assert!(
        range_errors.is_empty(),
        "without a specialcharacter spec section no 3100-range errors must be emitted"
    );
}
