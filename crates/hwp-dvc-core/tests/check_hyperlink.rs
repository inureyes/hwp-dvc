//! Integration tests for the hyperlink validator.
//!
//! Exercises the full pipeline: `Document::parse` → `Checker::run` with
//! a real fixture and a spec loaded from `fixture_spec.json`.
//!
//! | Fixture                | Spec                              | Expected                               |
//! |------------------------|-----------------------------------|----------------------------------------|
//! | `hyperlink_none.hwpx`  | `"hyperlink": {"permission": false}` | zero Hyperlink-range errors         |
//! | `hyperlink_external.hwpx` | `"hyperlink": {"permission": false}` | ≥ 1 error with code 6901        |

use std::path::PathBuf;

use hwp_dvc_core::checker::{Checker, DvcErrorInfo};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::ErrorCode;
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

fn parse_doc(name: &str) -> Document {
    let mut doc = Document::open(fixture_doc(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

fn load_spec() -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec("fixture_spec.json"))
        .expect("fixture_spec.json should parse cleanly")
}

fn hyperlink_errors(errors: &[DvcErrorInfo]) -> Vec<&DvcErrorInfo> {
    let base = ErrorCode::Hyperlink as u32;
    let next = ErrorCode::Macro as u32;
    errors
        .iter()
        .filter(|e| e.error_code >= base && e.error_code < next)
        .collect()
}

/// A document with no hyperlinks must produce zero Hyperlink-range errors
/// even when the spec forbids hyperlinks.
#[test]
fn hyperlink_none_produces_zero_hyperlink_errors() {
    let doc = parse_doc("hyperlink_none.hwpx");
    let spec = load_spec();

    // Confirm the spec actually forbids hyperlinks.
    let hl_spec = spec
        .hyperlink
        .as_ref()
        .expect("fixture_spec must have hyperlink section");
    assert!(
        !hl_spec.permission,
        "fixture_spec must set hyperlink.permission = false for this test to be meaningful"
    );

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");
    let hl_errors = hyperlink_errors(&errors);

    assert!(
        hl_errors.is_empty(),
        "hyperlink_none.hwpx has no hyperlinks — expected zero errors in range 6900..6999; \
         got {} errors: {hl_errors:?}",
        hl_errors.len()
    );
}

/// A document with an external hyperlink must produce at least one error
/// in the 6900 range when the spec forbids hyperlinks.
#[test]
fn hyperlink_external_produces_hyperlink_errors() {
    let doc = parse_doc("hyperlink_external.hwpx");
    let spec = load_spec();

    // Sanity: the fixture must actually contain hyperlink runs.
    let hl_run_count = doc.run_type_infos.iter().filter(|r| r.is_hyperlink).count();
    assert!(
        hl_run_count > 0,
        "hyperlink_external.hwpx must contain at least one hyperlink run for this test \
         to be meaningful; got 0"
    );

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");
    let hl_errors = hyperlink_errors(&errors);

    assert!(
        !hl_errors.is_empty(),
        "hyperlink_external.hwpx has {hl_run_count} hyperlink run(s) and spec forbids hyperlinks \
         — expected ≥ 1 error in range 6900..6999; got none"
    );

    // Every hyperlink error must carry use_hyperlink = true.
    for e in &hl_errors {
        assert!(
            e.use_hyperlink,
            "all hyperlink errors must have use_hyperlink=true; got {e:?}"
        );
    }

    // The error count must match the number of flagged hyperlink runs.
    assert_eq!(
        hl_errors.len(),
        hl_run_count,
        "expected one error per hyperlink run ({hl_run_count} runs); got {}",
        hl_errors.len()
    );
}

/// Verify that `Checker::run` emits no hyperlink errors when the spec
/// permits hyperlinks, even for a document that contains them.
#[test]
fn hyperlink_permitted_emits_no_errors() {
    use hwp_dvc_core::spec::HyperlinkSpec;

    let doc = parse_doc("hyperlink_external.hwpx");

    // Build a minimal spec that explicitly *allows* hyperlinks.
    let spec = DvcSpec {
        hyperlink: Some(HyperlinkSpec { permission: true }),
        ..Default::default()
    };

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");
    let hl_errors = hyperlink_errors(&errors);

    assert!(
        hl_errors.is_empty(),
        "permission=true must suppress all hyperlink errors; got {hl_errors:?}"
    );
}
