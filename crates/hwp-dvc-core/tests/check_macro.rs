//! Integration tests for the `CheckMacro` validator.
//!
//! Global constraint:
//! - `macro_none.hwpx` + `fixture_spec.json` (permission=false) → 0 Macro errors.
//! - `macro_present.hwpx` + `fixture_spec.json` (permission=false) → ≥ 1 Macro error.

use std::path::PathBuf;

use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::macro_codes;
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

fn open_doc(name: &str) -> Document {
    let mut doc =
        Document::open(doc_fixture(name)).unwrap_or_else(|e| panic!("failed to open {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse {name}: {e}"));
    doc
}

fn fixture_spec() -> DvcSpec {
    DvcSpec::from_json_file(spec_fixture("fixture_spec.json"))
        .expect("fixture_spec.json must parse")
}

// ---------------------------------------------------------------------------
// has_macro detection
// ---------------------------------------------------------------------------

#[test]
fn macro_none_hwpx_has_no_macro() {
    let doc = open_doc("macro_none.hwpx");
    assert!(
        !doc.has_macro(),
        "macro_none.hwpx must report has_macro() == false"
    );
}

#[test]
fn macro_present_hwpx_has_macro() {
    let doc = open_doc("macro_present.hwpx");
    assert!(
        doc.has_macro(),
        "macro_present.hwpx must report has_macro() == true"
    );
}

// ---------------------------------------------------------------------------
// checker::macro_::check unit behaviour
// ---------------------------------------------------------------------------

#[test]
fn macro_check_permission_false_no_macro_emits_no_error() {
    use hwp_dvc_core::spec::MacroSpec;
    let doc = open_doc("macro_none.hwpx");
    let spec = MacroSpec { permission: false };
    let errors = hwp_dvc_core::checker::macro_::check(&spec, &doc);
    assert!(
        errors.is_empty(),
        "no macro in document → no error even when permission=false"
    );
}

#[test]
fn macro_check_permission_true_with_macro_emits_no_error() {
    use hwp_dvc_core::spec::MacroSpec;
    let doc = open_doc("macro_present.hwpx");
    let spec = MacroSpec { permission: true };
    let errors = hwp_dvc_core::checker::macro_::check(&spec, &doc);
    assert!(
        errors.is_empty(),
        "macro present but permission=true → no error"
    );
}

#[test]
fn macro_check_permission_false_with_macro_emits_error() {
    use hwp_dvc_core::spec::MacroSpec;
    let doc = open_doc("macro_present.hwpx");
    let spec = MacroSpec { permission: false };
    let errors = hwp_dvc_core::checker::macro_::check(&spec, &doc);
    assert_eq!(
        errors.len(),
        1,
        "macro present AND permission=false → exactly one error"
    );
    assert_eq!(
        errors[0].error_code,
        macro_codes::MACRO_PERMISSION,
        "error code must be MACRO_PERMISSION (7001)"
    );
}

// ---------------------------------------------------------------------------
// Checker::run integration (global constraint)
// ---------------------------------------------------------------------------

#[test]
fn checker_run_macro_none_emits_zero_macro_errors() {
    let spec = fixture_spec();
    let doc = open_doc("macro_none.hwpx");
    let checker = Checker {
        spec: &spec,
        document: &doc,
        level: CheckLevel::All,
        scope: OutputScope::default(),
    };
    let errors = checker.run().expect("run must succeed");
    let macro_errors: Vec<_> = errors.iter().filter(|e| e.error_code / 1000 == 7).collect();
    assert!(
        macro_errors.is_empty(),
        "macro_none.hwpx must produce zero Macro-range errors; got: {macro_errors:?}"
    );
}

#[test]
fn checker_run_macro_present_emits_macro_error() {
    let spec = fixture_spec();
    // fixture_spec.json has `"macro": { "permission": false }`
    assert!(
        spec.macro_.as_ref().map(|m| !m.permission).unwrap_or(false),
        "fixture_spec must have macro.permission == false for this test to be meaningful"
    );
    let doc = open_doc("macro_present.hwpx");
    let checker = Checker {
        spec: &spec,
        document: &doc,
        level: CheckLevel::All,
        scope: OutputScope::default(),
    };
    let errors = checker.run().expect("run must succeed");
    let macro_errors: Vec<_> = errors.iter().filter(|e| e.error_code / 1000 == 7).collect();
    assert!(
        !macro_errors.is_empty(),
        "macro_present.hwpx with permission=false must emit ≥1 Macro-range error"
    );
    assert_eq!(
        macro_errors[0].error_code,
        macro_codes::MACRO_PERMISSION,
        "Macro-range error code must be MACRO_PERMISSION (7001)"
    );
}
