//! Integration tests for `checker::para_shape::check` against real HWPX fixtures.
//!
//! Fixtures live under `tests/fixtures/docs/`. The spec used throughout
//! mirrors `tests/fixtures/specs/fixture_spec.json` with `parashape` fields
//! `spacing-paraup:0`, `spacing-parabottom:0`, `linespacing:0`,
//! `linespacingvalue:160`, `indent:0`, `outdent:0`.
//!
//! | Fixture                        | Expected outcome              |
//! |--------------------------------|-------------------------------|
//! | `parashape_pass.hwpx`          | zero 2000-range errors        |
//! | `parashape_fail_indent.hwpx`   | ≥ 1 PARASHAPE_INDENT (2005)   |
//! | `parashape_fail_linespacing.hwpx` | ≥ 1 PARASHAPE_LINESPACINGVALUE (2008) |

use std::path::PathBuf;

use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::para_shape_codes::{
    PARASHAPE_INDENT, PARASHAPE_LINESPACING, PARASHAPE_LINESPACINGVALUE,
};
use hwp_dvc_core::spec::DvcSpec;

fn fixture_doc(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

fn parse(name: &str) -> Document {
    let mut doc = Document::open(fixture_doc(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

/// The strict spec used for all three fixture tests.
fn strict_spec() -> DvcSpec {
    DvcSpec::from_json_str(
        r#"{
            "parashape": {
                "spacing-paraup":    0,
                "spacing-parabottom": 0,
                "linespacing":       0,
                "linespacingvalue":  160,
                "indent":            0,
                "outdent":           0
            }
        }"#,
    )
    .expect("strict_spec must parse")
}

fn run_checker(doc: &Document, spec: &DvcSpec) -> Vec<hwp_dvc_core::checker::DvcErrorInfo> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope: OutputScope::default(),
    };
    checker.run().expect("Checker::run must not fail")
}

/// Helper: is `code` in the 2000-range?
fn is_parashape(code: u32) -> bool {
    (2000..3000).contains(&code)
}

// ─── pass fixture ────────────────────────────────────────────────────────────

#[test]
fn parashape_pass_produces_zero_errors_in_2000_range() {
    let doc = parse("parashape_pass.hwpx");
    let spec = strict_spec();
    let errs = run_checker(&doc, &spec);
    let parashape_errs: Vec<_> = errs.iter().filter(|e| is_parashape(e.error_code)).collect();
    assert!(
        parashape_errs.is_empty(),
        "parashape_pass.hwpx must produce zero 2000-range errors; got: {:?}",
        parashape_errs
            .iter()
            .map(|e| (e.error_code, e.para_pr_id_ref))
            .collect::<Vec<_>>()
    );
}

// ─── indent fail fixture ─────────────────────────────────────────────────────

#[test]
fn parashape_fail_indent_triggers_indent_error() {
    let doc = parse("parashape_fail_indent.hwpx");
    let spec = strict_spec();
    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter().any(|e| e.error_code == PARASHAPE_INDENT),
        "parashape_fail_indent.hwpx must produce at least one PARASHAPE_INDENT (2005) error; \
         got codes: {:?}",
        errs.iter().map(|e| e.error_code).collect::<Vec<_>>()
    );
}

// ─── linespacing fail fixture ─────────────────────────────────────────────────

#[test]
fn parashape_fail_linespacing_triggers_linespacingvalue_or_linespacing_error() {
    let doc = parse("parashape_fail_linespacing.hwpx");
    let spec = strict_spec();
    let errs = run_checker(&doc, &spec);
    let has_ls_err = errs.iter().any(|e| {
        e.error_code == PARASHAPE_LINESPACING || e.error_code == PARASHAPE_LINESPACINGVALUE
    });
    assert!(
        has_ls_err,
        "parashape_fail_linespacing.hwpx must produce at least one PARASHAPE_LINESPACING (2007) \
         or PARASHAPE_LINESPACINGVALUE (2008) error; got codes: {:?}",
        errs.iter().map(|e| e.error_code).collect::<Vec<_>>()
    );
}

// ─── error metadata sanity ───────────────────────────────────────────────────

#[test]
fn error_para_pr_id_ref_is_populated() {
    // Use the fail_indent fixture to get at least one error and verify
    // that `para_pr_id_ref` is non-zero (the failing shape has id ≥ 1).
    let doc = parse("parashape_fail_indent.hwpx");
    let spec = strict_spec();
    let errs = run_checker(&doc, &spec);
    let indent_errs: Vec<_> = errs
        .iter()
        .filter(|e| e.error_code == PARASHAPE_INDENT)
        .collect();
    assert!(!indent_errs.is_empty());
    for e in &indent_errs {
        // The offending para shape id=20 in the fixture; just assert it is
        // a valid (non-sentinel) id.
        assert!(
            e.para_pr_id_ref > 0,
            "expected non-zero para_pr_id_ref on PARASHAPE_INDENT error"
        );
    }
}
