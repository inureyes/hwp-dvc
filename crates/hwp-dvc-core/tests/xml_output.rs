//! Integration tests for the XML output formatter.
//!
//! These tests are compiled and run only when the `xml` Cargo feature is
//! enabled:
//!
//! ```text
//! cargo test --workspace --features xml
//! ```
//!
//! # Coverage
//!
//! * `charshape_fail_font.hwpx` against `fixture_spec.json` — produces at
//!   least one `CHARSHAPE_FONT` (1004) error. The serialised XML must contain
//!   `<errorCode>1004</errorCode>` and a non-empty `<text>` element.
//!
//! * Pretty-print round-trip: the pretty output is a super-string of the
//!   compact output (all content is present, just with added whitespace).
//!
//! * Empty input: `to_xml(&[], …)` must produce a valid, well-formed XML
//!   document with an empty `<dvcErrors/>` root (or `<dvcErrors></dvcErrors>`).

#![cfg(feature = "xml")]

use std::path::PathBuf;

use hwp_dvc_core::checker::Checker;
use hwp_dvc_core::document::Document;
use hwp_dvc_core::output::to_xml;
use hwp_dvc_core::spec::DvcSpec;

// ──────────────────────────────────────────────────────────────────────────────
// Fixture helpers (mirrors the pattern used in check_char_shape.rs)
// ──────────────────────────────────────────────────────────────────────────────

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
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

fn load_spec(name: &str) -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec_path(name))
        .unwrap_or_else(|e| panic!("failed to load spec {name}: {e}"))
}

// ──────────────────────────────────────────────────────────────────────────────
// Real-fixture integration test (global constraint)
// ──────────────────────────────────────────────────────────────────────────────

/// Load `charshape_fail_font.hwpx`, run the checker against `fixture_spec.json`,
/// serialise the result as XML, and assert that the output contains the expected
/// element tags and the CHARSHAPE_FONT error code.
#[test]
fn xml_output_contains_charshape_font_error() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");

    // Confirm the checker produced at least one 1004 error so the XML
    // assertion below is meaningful.
    let font_errors: Vec<_> = errors.iter().filter(|e| e.error_code == 1004).collect();
    assert!(
        !font_errors.is_empty(),
        "charshape_fail_font.hwpx must produce CHARSHAPE_FONT (1004) errors; got none. \
         All errors: {errors:?}",
    );

    let xml = to_xml(&errors, false).expect("to_xml must not fail");

    // Must have a valid XML declaration.
    assert!(
        xml.starts_with("<?xml"),
        "XML output must start with XML declaration; got: {xml:.80}"
    );

    // Root element.
    assert!(
        xml.contains("<dvcErrors>"),
        "XML output must contain <dvcErrors> root; got: {xml:.200}"
    );

    // Each error must be wrapped in <error>.
    assert!(
        xml.contains("<error>"),
        "XML output must contain <error> elements; got: {xml:.200}"
    );

    // The CHARSHAPE_FONT error code must appear.
    assert!(
        xml.contains("<errorCode>1004</errorCode>"),
        "XML output must contain <errorCode>1004</errorCode>; got: {xml:.500}"
    );

    // A non-empty <text> element must be present.
    assert!(
        xml.contains("<text>"),
        "XML output must contain <text> elements; got: {xml:.200}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Pretty-print test
// ──────────────────────────────────────────────────────────────────────────────

/// The pretty-printed output must contain the same semantic content as the
/// compact output. Verified by checking that all element tags present in the
/// compact form also appear in the pretty form.
#[test]
fn xml_output_pretty_contains_same_elements() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("Checker::run must not fail");

    let compact = to_xml(&errors, false).expect("compact to_xml failed");
    let pretty = to_xml(&errors, true).expect("pretty to_xml failed");

    // Both must have the same root element.
    assert!(compact.contains("<dvcErrors>"), "compact must have <dvcErrors>");
    assert!(pretty.contains("<dvcErrors>"), "pretty must have <dvcErrors>");

    // Pretty output must be longer (indentation adds bytes).
    assert!(
        pretty.len() > compact.len(),
        "pretty output ({} bytes) should be longer than compact ({} bytes)",
        pretty.len(),
        compact.len()
    );

    // errorCode and text elements must appear in both.
    for tag in &["<errorCode>", "<text>", "<charIDRef>", "<paraPrIDRef>"] {
        assert!(compact.contains(tag), "compact missing {tag}");
        assert!(pretty.contains(tag), "pretty missing {tag}");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Empty input test
// ──────────────────────────────────────────────────────────────────────────────

/// `to_xml` on an empty slice must succeed and produce a well-formed XML
/// document with an empty `<dvcErrors>` root.
#[test]
fn xml_output_empty_input() {
    let xml = to_xml(&[], false).expect("to_xml(&[]) must not fail");

    assert!(
        xml.starts_with("<?xml"),
        "empty output must start with XML declaration; got: {xml}"
    );
    assert!(
        xml.contains("<dvcErrors>") && xml.contains("</dvcErrors>"),
        "empty output must have dvcErrors open/close tags; got: {xml}"
    );
    // No <error> elements for empty input.
    assert!(
        !xml.contains("<error>"),
        "empty output must not contain <error> elements; got: {xml}"
    );
}
