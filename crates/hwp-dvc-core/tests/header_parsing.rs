//! Integration tests for `HwpxArchive::read_header()` against real
//! HWPX fixtures committed under `tests/fixtures/docs/`.
//!
//! Each test opens a fixture end-to-end, parses its `Contents/header.xml`,
//! and asserts a concrete domain fact — never just "it didn't panic".
//! Fixture assignment matches the Epic #1 integration-test table:
//!
//! | Fixture              | Assertion                                         |
//! |----------------------|---------------------------------------------------|
//! | `charshape_pass`     | default CharShape resolves to `함초롬바탕`.       |
//! | `table_nested`       | at least one `BorderFill` has 4 `SOLID` sides.    |
//! | `style_custom`       | user-defined style `"테스트스타일"` is present.   |
//!
//! The spec paired with these fixtures is
//! `tests/fixtures/specs/fixture_spec.json`; the spec is not loaded in
//! these tests (spec wiring is in the checker tests), but the spec
//! listing is what ties fixture choice to this issue.

use std::path::PathBuf;

use hwp_dvc_core::document::header::{FontLang, HeaderTables};
use hwp_dvc_core::document::HwpxArchive;

/// Absolute path to a fixture under `tests/fixtures/docs/`.
fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("fixtures");
    p.push("docs");
    p.push(name);
    p
}

fn load_header(name: &str) -> HeaderTables {
    let archive = HwpxArchive::open(fixture(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    archive
        .read_header()
        .unwrap_or_else(|e| panic!("failed to parse {name}/Contents/header.xml: {e}"))
}

#[test]
fn charshape_pass_header_has_hamcho_font() {
    let tables = load_header("charshape_pass.hwpx");

    // Baseline sanity.
    assert!(
        !tables.char_shapes.is_empty(),
        "charshape_pass must expose at least one charShape"
    );
    assert!(
        !tables.font_faces.is_empty(),
        "charshape_pass must expose at least one fontface"
    );

    // The charshape_pass fixture is the HWP default (fixture_spec.json
    // baseline): every CharShape's Hangul font must ultimately resolve
    // to 함초롬바탕. At least one CharShape must list that font name in
    // its resolved face set.
    let has_hamcho = tables.char_shapes.values().any(|cs| {
        cs.font_names(&tables.font_faces)
            .iter()
            .any(|n| n == "함초롬바탕")
    });

    assert!(
        has_hamcho,
        "expected at least one CharShape whose font names contain '함초롬바탕'; \
         got charshape font sets: {:?}",
        tables
            .char_shapes
            .values()
            .map(|cs| cs.font_names(&tables.font_faces))
            .collect::<Vec<_>>()
    );

    // Spot-check the FontLang::Hangul slot of CharShape id=0 (the
    // default `바탕글` style's charPr). The declared font id for Hangul
    // at id=0 is `1` in the fixture, and font id 1 in the HANGUL
    // fontface resolves to 함초롬바탕.
    let hangul_font_of_default = tables.font_name(0, FontLang::Hangul);
    assert_eq!(
        hangul_font_of_default,
        Some("함초롬바탕"),
        "default CharShape's Hangul font must be 함초롬바탕"
    );
}

#[test]
fn table_nested_header_has_border_fills() {
    let tables = load_header("table_nested.hwpx");

    assert!(
        !tables.border_fills.is_empty(),
        "table_nested must declare border-fill entries for its table cells"
    );

    // The `table_nested.hwpx` fixture is authored with
    // `실선 0.12mm 검정` on the table's four outer borders. A BorderFill
    // with all four sides SOLID must therefore be present.
    let has_four_solid = tables.border_fills.values().any(|bf| bf.four_sides_solid());

    assert!(
        has_four_solid,
        "expected at least one BorderFill with all four sides SOLID; got: {:?}",
        tables
            .border_fills
            .values()
            .map(|bf| (
                bf.id,
                bf.left.line_type,
                bf.right.line_type,
                bf.top.line_type,
                bf.bottom.line_type
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn style_custom_header_has_user_style() {
    let tables = load_header("style_custom.hwpx");

    assert!(
        !tables.styles.is_empty(),
        "style_custom must declare at least one style"
    );

    let custom = tables.style_by_name("테스트스타일");
    assert!(
        custom.is_some(),
        "expected a user-defined style named '테스트스타일'; got styles: {:?}",
        tables
            .styles
            .values()
            .map(|s| (s.id, s.name.clone(), s.style_type.clone()))
            .collect::<Vec<_>>()
    );

    let style = custom.unwrap();
    assert_eq!(style.name, "테스트스타일");
    // The fixture README (tests/fixtures/docs/README.md) specifies that
    // `테스트스타일` is defined as a PARA-type style based on 바탕글.
    assert_eq!(
        style.style_type, "PARA",
        "테스트스타일 must be a PARA-type style"
    );

    // 바탕글 (the built-in default) must also still exist.
    assert!(
        tables.style_by_name("바탕글").is_some(),
        "built-in 바탕글 style must coexist with the user-defined style"
    );
}

#[test]
fn charshape_pass_tables_populated_across_categories() {
    // Broader sanity check covering every HeaderTables bucket. Keeps
    // regressions that silently drop a whole category from sneaking in.
    let tables = load_header("charshape_pass.hwpx");

    assert!(
        !tables.char_shapes.is_empty(),
        "char_shapes must be populated"
    );
    assert!(
        !tables.para_shapes.is_empty(),
        "para_shapes must be populated"
    );
    assert!(
        !tables.border_fills.is_empty(),
        "border_fills must be populated"
    );
    assert!(!tables.styles.is_empty(), "styles must be populated");
    assert!(
        tables.style_by_name("바탕글").is_some(),
        "default 바탕글 style must be present"
    );
    // numberings are optional per document but the baseline fixture
    // includes them, so we still assert them:
    assert!(
        !tables.numberings.is_empty(),
        "numberings must be populated in the baseline fixture"
    );

    // Seven font-face slots are always declared by Hancom writers, one
    // per FontLang.
    assert_eq!(
        tables.font_faces.len(),
        7,
        "HWPX writers emit one fontface per FontLang"
    );
}
