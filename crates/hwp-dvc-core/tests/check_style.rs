//! Integration tests for the `CheckStyle` validator.
//!
//! Each test follows the full pipeline:
//!   `Document::open` → `Document::parse` → `Checker::new` → `Checker::run`
//!
//! Fixtures under `tests/fixtures/docs/` are real HWPX archives that have
//! been committed with the repository. The shared spec is loaded from
//! `tests/fixtures/specs/fixture_spec.json` which carries
//! `"style": { "permission": false }`.
//!
//! | Fixture                 | Expected outcome                                          |
//! |-------------------------|-----------------------------------------------------------|
//! | `style_default_only`    | zero Style-range errors (no custom styles used)           |
//! | `style_custom`          | ≥ 1 Style-range error (custom style used, disallowed)     |

use std::path::PathBuf;

use hwp_dvc_core::checker::style::STYLE_PERMISSION;
use hwp_dvc_core::checker::{Checker, DvcErrorInfo};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::ErrorCode;
use hwp_dvc_core::spec::DvcSpec;

/// Absolute path to a HWPX fixture.
fn doc_fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

/// Absolute path to a spec fixture.
fn spec_fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/specs");
    p.push(name);
    p
}

/// Load, parse, and run the checker against a fixture document+spec pair.
/// Returns the full error list from `Checker::run`.
fn check(doc_name: &str, spec_name: &str) -> Vec<DvcErrorInfo> {
    let spec = DvcSpec::from_json_file(spec_fixture(spec_name))
        .unwrap_or_else(|e| panic!("failed to parse spec {spec_name}: {e}"));

    let mut doc = Document::open(doc_fixture(doc_name))
        .unwrap_or_else(|e| panic!("failed to open {doc_name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse {doc_name}: {e}"));

    let checker = Checker::new(&spec, &doc);
    checker
        .run()
        .unwrap_or_else(|e| panic!("Checker::run failed for {doc_name}: {e}"))
}

/// Partition `errors` into Style-range errors (3500 ≤ code < 4000) and others.
fn partition_style_errors(errors: &[DvcErrorInfo]) -> (Vec<&DvcErrorInfo>, Vec<&DvcErrorInfo>) {
    let style_base = ErrorCode::Style as u32;
    let next_base = ErrorCode::Page as u32;
    errors
        .iter()
        .partition(|e| e.error_code >= style_base && e.error_code < next_base)
}

// ---------------------------------------------------------------------------
// Passing fixture: style_default_only.hwpx
// ---------------------------------------------------------------------------

/// A document that only uses the default "바탕글" style must produce
/// zero Style-range errors when `permission == false`.
#[test]
fn style_default_only_produces_no_style_errors() {
    let errors = check("style_default_only.hwpx", "fixture_spec.json");
    let (style_errors, _) = partition_style_errors(&errors);
    assert!(
        style_errors.is_empty(),
        "style_default_only must have zero Style-range errors; got {style_errors:?}"
    );
}

/// The `style_default_only` fixture must also produce zero `use_style=true`
/// entries — the run stream carries no `is_style=true` runs.
#[test]
fn style_default_only_no_use_style_flags() {
    let errors = check("style_default_only.hwpx", "fixture_spec.json");
    let flagged: Vec<&DvcErrorInfo> = errors.iter().filter(|e| e.use_style).collect();
    assert!(
        flagged.is_empty(),
        "style_default_only must produce no DvcErrorInfo with use_style=true; got {flagged:?}"
    );
}

// ---------------------------------------------------------------------------
// Failing fixture: style_custom.hwpx
// ---------------------------------------------------------------------------

/// A document that applies a custom style must produce at least one
/// Style-range error when `permission == false`.
#[test]
fn style_custom_produces_at_least_one_style_error() {
    let errors = check("style_custom.hwpx", "fixture_spec.json");
    let (style_errors, _) = partition_style_errors(&errors);
    assert!(
        !style_errors.is_empty(),
        "style_custom must have ≥ 1 Style-range error when permission=false; got zero errors"
    );
}

/// Every Style-range error emitted for `style_custom` must carry
/// `error_code == STYLE_PERMISSION` and `use_style == true`.
#[test]
fn style_custom_error_fields_are_correct() {
    let errors = check("style_custom.hwpx", "fixture_spec.json");
    let (style_errors, _) = partition_style_errors(&errors);
    assert!(
        !style_errors.is_empty(),
        "style_custom must emit style errors"
    );
    for e in &style_errors {
        assert_eq!(
            e.error_code, STYLE_PERMISSION,
            "error_code must be STYLE_PERMISSION ({STYLE_PERMISSION}); got {}",
            e.error_code
        );
        assert!(
            e.use_style,
            "use_style must be true on a Style-range error; got {e:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Permission-true guard: style_custom should be clean when permission=true
// ---------------------------------------------------------------------------

/// When the spec permits styles (`permission: true`), even `style_custom`
/// must produce zero Style-range errors — the validator must be a no-op.
#[test]
fn style_custom_clean_when_permission_true() {
    // Build an in-memory spec that enables styles.
    let spec_json = r#"{ "style": { "permission": true } }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("inline spec parses");

    let mut doc = Document::open(doc_fixture("style_custom.hwpx")).expect("style_custom opens");
    doc.parse().expect("style_custom parses");

    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run succeeds");
    let (style_errors, _) = partition_style_errors(&errors);
    assert!(
        style_errors.is_empty(),
        "permission=true must suppress all Style errors; got {style_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// STYLE_PERMISSION constant sanity
// ---------------------------------------------------------------------------

/// Guard that `STYLE_PERMISSION` is within the Style error range.
#[test]
fn style_permission_constant_in_range() {
    let base = ErrorCode::Style as u32;
    let next = ErrorCode::Page as u32;
    assert!(
        STYLE_PERMISSION >= base,
        "STYLE_PERMISSION ({STYLE_PERMISSION}) must be >= Style base ({base})"
    );
    assert!(
        STYLE_PERMISSION < next,
        "STYLE_PERMISSION ({STYLE_PERMISSION}) must be < next category ({next})"
    );
}

// ---------------------------------------------------------------------------
// Type allow-list: allowed_types (JID_STYLE_TYPE = 3501)
// ---------------------------------------------------------------------------

use hwp_dvc_core::checker::style::STYLE_TYPE;

/// When the spec allows only 바탕글 and the document uses only 바탕글,
/// no 3501 errors should be emitted.
#[test]
fn style_default_only_passes_when_normal_is_in_allowed_types() {
    let spec_json = r#"{ "style": { "permission": true, "allowed_types": ["바탕글"] } }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let mut doc = Document::open(doc_fixture("style_default_only.hwpx")).expect("doc opens");
    doc.parse().expect("doc parses");
    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run succeeds");
    let type_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == STYLE_TYPE)
        .collect();
    assert!(
        type_errors.is_empty(),
        "style_default_only must produce no 3501 errors when 바탕글 is allowed; \
         got {type_errors:?}"
    );
}

/// When the spec allows only 본문 and the document uses 바탕글 (not 본문),
/// every run should emit a 3501 error because 바탕글 is not in the list.
#[test]
fn style_default_only_fails_when_normal_not_in_allowed_types() {
    let spec_json = r#"{ "style": { "permission": true, "allowed_types": ["본문"] } }"#;
    let spec = DvcSpec::from_json_str(spec_json).expect("spec parses");
    let mut doc = Document::open(doc_fixture("style_default_only.hwpx")).expect("doc opens");
    doc.parse().expect("doc parses");
    let checker = Checker::new(&spec, &doc);
    let errors = checker.run().expect("run succeeds");
    let type_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.error_code == STYLE_TYPE)
        .collect();
    assert!(
        !type_errors.is_empty(),
        "style_default_only must produce ≥ 1 3501 error when only 본문 is allowed"
    );
    for e in &type_errors {
        assert!(e.use_style, "use_style must be true on 3501 errors");
    }
}

/// style_custom uses a custom (non-standard) style. When permission=false
/// and allowed_types is absent, only 3502 errors should fire (not 3501).
#[test]
fn style_custom_produces_only_permission_errors_when_no_allowed_types() {
    let errors = check("style_custom.hwpx", "fixture_spec.json");
    let (style_errors, _) = partition_style_errors(&errors);
    assert!(
        !style_errors.is_empty(),
        "style_custom must emit style errors under permission=false"
    );
    // All style errors must be 3502, not 3501.
    for e in &style_errors {
        assert_eq!(
            e.error_code, STYLE_PERMISSION,
            "without allowed_types, only STYLE_PERMISSION (3502) should fire; got {}",
            e.error_code
        );
    }
}

/// Guard that `STYLE_TYPE` is within the Style error range.
#[test]
fn style_type_constant_in_range() {
    let base = ErrorCode::Style as u32;
    let next = ErrorCode::Page as u32;
    assert!(
        STYLE_TYPE >= base,
        "STYLE_TYPE ({STYLE_TYPE}) must be >= Style base ({base})"
    );
    assert!(
        STYLE_TYPE < next,
        "STYLE_TYPE ({STYLE_TYPE}) must be < next category ({next})"
    );
}
