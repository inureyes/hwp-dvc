//! Integration tests for [`OutputScope`] filtering in [`Checker::run`].
//!
//! These tests verify that `OutputScope` correctly gates each validator
//! category, using real HWPX fixtures paired with `fixture_spec.json`.
//!
//! # Global constraints (from issue #17)
//!
//! 1. `charshape_fail_font.hwpx` + fixture_spec:
//!    - `scope=default` → ≥ 1 error (CharShape errors pass through).
//!    - `scope={table only}` → 0 CharShape errors (shape gated out).
//!    - `scope={all}` → ≥ 1 error (explicit all = same as default).
//!
//! 2. `table_nested.hwpx` + fixture_spec:
//!    - `scope=default` → ≥ 1 TABLE_IN_TABLE error.
//!    - `scope={hyperlink only}` → 0 table errors (table gated out).

use std::path::PathBuf;

use hwp_dvc_core::checker::char_shape::CHARSHAPE_FONT;
use hwp_dvc_core::checker::table::TABLE_IN_TABLE;
use hwp_dvc_core::checker::{CheckLevel, Checker, OutputScope};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::spec::DvcSpec;

// ──────────────────────────────────────────────────────────────────────────────
// Test helpers
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
        .unwrap_or_else(|e| panic!("failed to open fixture '{name}': {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture '{name}': {e}"));
    doc
}

fn load_spec(name: &str) -> DvcSpec {
    DvcSpec::from_json_file(fixture_spec_path(name))
        .unwrap_or_else(|e| panic!("failed to load spec '{name}': {e}"))
}

fn run_with_scope(doc: &Document, spec: &DvcSpec, scope: OutputScope) -> Vec<u32> {
    let checker = Checker {
        spec,
        document: doc,
        level: CheckLevel::All,
        scope,
    };
    checker
        .run()
        .expect("Checker::run must not fail")
        .into_iter()
        .map(|e| e.error_code)
        .collect()
}

// ──────────────────────────────────────────────────────────────────────────────
// Scope::default — all flags false → all validators emit
// ──────────────────────────────────────────────────────────────────────────────

/// `charshape_fail_font.hwpx` with default scope (no flags set) must
/// produce at least one CHARSHAPE_FONT (1004) error.
#[test]
fn charshape_fail_font_default_scope_emits_charshape_errors() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let codes = run_with_scope(&doc, &spec, OutputScope::default());

    assert!(
        codes.contains(&CHARSHAPE_FONT),
        "default scope must emit CharShape errors; got codes: {codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// scope.table only — CharShape errors suppressed
// ──────────────────────────────────────────────────────────────────────────────

/// `charshape_fail_font.hwpx` with `scope={table=true}` must NOT emit
/// any CharShape (1000-range) errors — the shape scope gate must filter them.
#[test]
fn charshape_fail_font_table_scope_suppresses_charshape_errors() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        table: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    let charshape_codes: Vec<u32> = codes
        .into_iter()
        .filter(|&c| (1000..2000).contains(&c))
        .collect();

    assert!(
        charshape_codes.is_empty(),
        "table-only scope must suppress CharShape (1000-range) errors; \
         got: {charshape_codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// scope.all (explicit) — same as default
// ──────────────────────────────────────────────────────────────────────────────

/// `charshape_fail_font.hwpx` with `scope={all=true}` must emit at least
/// one CHARSHAPE_FONT error — `all` is equivalent to the default.
#[test]
fn charshape_fail_font_all_scope_emits_charshape_errors() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        all: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        codes.contains(&CHARSHAPE_FONT),
        "all scope must emit CharShape errors; got codes: {codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// table_nested.hwpx — TABLE_IN_TABLE present with default scope
// ──────────────────────────────────────────────────────────────────────────────

/// `table_nested.hwpx` with default scope must produce at least one
/// TABLE_IN_TABLE (3056) error.
#[test]
fn table_nested_default_scope_emits_table_in_table_error() {
    let doc = load_doc("table_nested.hwpx");
    let spec = load_spec("fixture_spec.json");

    let codes = run_with_scope(&doc, &spec, OutputScope::default());

    assert!(
        codes.contains(&TABLE_IN_TABLE),
        "default scope must emit TABLE_IN_TABLE; got codes: {codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// table_nested.hwpx — hyperlink-only scope suppresses table errors
// ──────────────────────────────────────────────────────────────────────────────

/// `table_nested.hwpx` with `scope={hyperlink=true}` must NOT emit any
/// Table (3000-range) errors — the table scope gate must filter them.
#[test]
fn table_nested_hyperlink_scope_suppresses_table_errors() {
    let doc = load_doc("table_nested.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        hyperlink: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    let table_codes: Vec<u32> = codes
        .into_iter()
        .filter(|&c| (3000..3100).contains(&c))
        .collect();

    assert!(
        table_codes.is_empty(),
        "hyperlink-only scope must suppress Table (3000-range) errors; \
         got: {table_codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// scope.shape — gates CharShape + ParaShape
// ──────────────────────────────────────────────────────────────────────────────

/// `charshape_fail_font.hwpx` with `scope={shape=true}` must emit at least
/// one CharShape error (shape scope includes CharShape + ParaShape).
#[test]
fn charshape_fail_font_shape_scope_emits_charshape_errors() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        shape: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        codes.contains(&CHARSHAPE_FONT),
        "shape scope must emit CharShape errors; got codes: {codes:?}"
    );
}

/// `charshape_fail_font.hwpx` with `scope={shape=true}` must NOT emit
/// Table (3000-range) errors — only shape-related validators emit.
#[test]
fn charshape_fail_font_shape_scope_suppresses_table_errors() {
    let doc = load_doc("charshape_fail_font.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        shape: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    let table_codes: Vec<u32> = codes
        .into_iter()
        .filter(|&c| (3000..3100).contains(&c))
        .collect();

    assert!(
        table_codes.is_empty(),
        "shape-only scope must suppress Table errors; got: {table_codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// scope.style — gates Style validator
// ──────────────────────────────────────────────────────────────────────────────

/// `style_custom.hwpx` with `scope={style=true}` must emit style errors.
#[test]
fn style_custom_style_scope_emits_style_errors() {
    use hwp_dvc_core::checker::style::STYLE_PERMISSION;

    let doc = load_doc("style_custom.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        style: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        codes.contains(&STYLE_PERMISSION),
        "style scope must emit STYLE_PERMISSION; got codes: {codes:?}"
    );
}

/// `style_custom.hwpx` with `scope={table=true}` must NOT emit Style errors.
#[test]
fn style_custom_table_scope_suppresses_style_errors() {
    use hwp_dvc_core::checker::style::STYLE_PERMISSION;

    let doc = load_doc("style_custom.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        table: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        !codes.contains(&STYLE_PERMISSION),
        "table-only scope must suppress Style errors; got codes: {codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// scope.hyperlink — gates Hyperlink validator
// ──────────────────────────────────────────────────────────────────────────────

/// `hyperlink_external.hwpx` with `scope={hyperlink=true}` must emit
/// hyperlink errors.
#[test]
fn hyperlink_external_hyperlink_scope_emits_hyperlink_errors() {
    use hwp_dvc_core::checker::hyperlink::HYPERLINK_PERMISSION;

    let doc = load_doc("hyperlink_external.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        hyperlink: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        codes.contains(&HYPERLINK_PERMISSION),
        "hyperlink scope must emit HYPERLINK_PERMISSION; got codes: {codes:?}"
    );
}

/// `hyperlink_external.hwpx` with `scope={style=true}` must NOT emit
/// hyperlink errors.
#[test]
fn hyperlink_external_style_scope_suppresses_hyperlink_errors() {
    use hwp_dvc_core::checker::hyperlink::HYPERLINK_PERMISSION;

    let doc = load_doc("hyperlink_external.hwpx");
    let spec = load_spec("fixture_spec.json");

    let scope = OutputScope {
        style: true,
        ..OutputScope::default()
    };
    let codes = run_with_scope(&doc, &spec, scope);

    assert!(
        !codes.contains(&HYPERLINK_PERMISSION),
        "style-only scope must suppress Hyperlink errors; got codes: {codes:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// OutputScope unit tests — is_default() and allows() logic
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn output_scope_default_is_default() {
    use hwp_dvc_core::checker::ScopeCategory;

    let scope = OutputScope::default();
    assert!(scope.allows(ScopeCategory::Shape));
    assert!(scope.allows(ScopeCategory::Table));
    assert!(scope.allows(ScopeCategory::Style));
    assert!(scope.allows(ScopeCategory::Hyperlink));
    assert!(scope.allows(ScopeCategory::Ungated));
}

#[test]
fn output_scope_all_allows_every_category() {
    use hwp_dvc_core::checker::ScopeCategory;

    let scope = OutputScope {
        all: true,
        ..OutputScope::default()
    };
    assert!(scope.allows(ScopeCategory::Shape));
    assert!(scope.allows(ScopeCategory::Table));
    assert!(scope.allows(ScopeCategory::Style));
    assert!(scope.allows(ScopeCategory::Hyperlink));
    assert!(scope.allows(ScopeCategory::Ungated));
}

#[test]
fn output_scope_table_only_gates_non_table() {
    use hwp_dvc_core::checker::ScopeCategory;

    let scope = OutputScope {
        table: true,
        ..OutputScope::default()
    };
    assert!(!scope.allows(ScopeCategory::Shape));
    assert!(scope.allows(ScopeCategory::Table));
    assert!(!scope.allows(ScopeCategory::Style));
    assert!(!scope.allows(ScopeCategory::Hyperlink));
    // Ungated always passes regardless of scope.
    assert!(scope.allows(ScopeCategory::Ungated));
}

#[test]
fn output_scope_shape_only_gates_non_shape() {
    use hwp_dvc_core::checker::ScopeCategory;

    let scope = OutputScope {
        shape: true,
        ..OutputScope::default()
    };
    assert!(scope.allows(ScopeCategory::Shape));
    assert!(!scope.allows(ScopeCategory::Table));
    assert!(!scope.allows(ScopeCategory::Style));
    assert!(!scope.allows(ScopeCategory::Hyperlink));
    assert!(scope.allows(ScopeCategory::Ungated));
}

#[test]
fn output_scope_tabledetail_enables_table_category() {
    use hwp_dvc_core::checker::ScopeCategory;

    let scope = OutputScope {
        table_detail: true,
        ..OutputScope::default()
    };
    assert!(!scope.allows(ScopeCategory::Shape));
    assert!(scope.allows(ScopeCategory::Table));
    assert!(!scope.allows(ScopeCategory::Style));
    assert!(!scope.allows(ScopeCategory::Hyperlink));
    assert!(scope.allows(ScopeCategory::Ungated));
}
