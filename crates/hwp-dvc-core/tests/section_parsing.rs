//! Integration tests for `HwpxArchive::read_sections()` against real
//! HWPX fixtures committed under `tests/fixtures/docs/`.
//!
//! Each test opens a fixture end-to-end, parses its
//! `Contents/section*.xml` parts, and asserts a concrete structural
//! fact about the paragraph AST — never just "it didn't panic".
//!
//! | Fixture              | Assertion                                          |
//! |----------------------|----------------------------------------------------|
//! | `parashape_pass`     | plain-paragraph baseline: non-empty Korean run.    |
//! | `table_nested`       | outer 2x2 table is depth 0 with 4 cells.           |
//! | `table_nested`       | inner table exists somewhere at nesting depth 1.   |

use std::path::PathBuf;

use hwp_dvc_core::document::section::{Section, Table};
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

fn load_sections(name: &str) -> Vec<Section> {
    let archive = HwpxArchive::open(fixture(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    archive
        .read_sections()
        .unwrap_or_else(|e| panic!("failed to parse sections in {name}: {e}"))
}

/// Return `true` if `s` contains at least one character in the
/// Korean syllable range (U+AC00..=U+D7A3). Used instead of hard-coded
/// phrase checks because the fixture's sample body may evolve while
/// remaining Korean-language content.
fn contains_korean(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

#[test]
fn parashape_pass_section_has_paragraphs() {
    let sections = load_sections("parashape_pass.hwpx");

    assert!(
        !sections.is_empty(),
        "parashape_pass must expose at least one section part"
    );

    for sec in &sections {
        assert!(
            !sec.paragraphs.is_empty(),
            "section {} in parashape_pass must contain at least one paragraph",
            sec.index
        );
    }

    // The fixture body is a Korean-language paragraph; at least one
    // run somewhere in the document must carry non-empty Korean text.
    let has_korean_text = sections.iter().any(|s| {
        s.all_paragraphs().any(|p| {
            p.runs
                .iter()
                .any(|r| !r.text.is_empty() && contains_korean(&r.text))
        })
    });

    assert!(
        has_korean_text,
        "expected at least one run with non-empty Korean text in parashape_pass; \
         got run texts: {:?}",
        sections
            .iter()
            .flat_map(|s| s
                .all_paragraphs()
                .flat_map(|p| p.runs.iter().map(|r| r.text.clone())))
            .collect::<Vec<_>>()
    );
}

#[test]
fn table_nested_section_detects_outer_table() {
    let sections = load_sections("table_nested.hwpx");

    let top_level_tables: Vec<&Table> = sections
        .iter()
        .flat_map(|s| s.all_tables().filter(|t| t.nesting_depth == 0))
        .collect();

    assert!(
        !top_level_tables.is_empty(),
        "table_nested must contain at least one top-level (depth=0) table"
    );

    // The fixture is authored as a 2x2 outer table. The declared
    // `rowCnt`/`colCnt` on the outer table must agree with that
    // shape, and the actual row/cell tree must yield at least 4
    // cells across its rows.
    let outer = top_level_tables[0];
    assert!(
        outer.row_cnt >= 2 && outer.col_cnt >= 2,
        "outer table must declare at least a 2x2 grid; got rowCnt={} colCnt={}",
        outer.row_cnt,
        outer.col_cnt
    );

    let total_cells: usize = outer.rows.iter().map(|r| r.cells.len()).sum();
    assert!(
        total_cells >= 4,
        "outer table must expose at least 4 cells across its rows; got {total_cells}"
    );
}

#[test]
fn table_nested_section_detects_inner_table() {
    let sections = load_sections("table_nested.hwpx");

    let inner_tables: Vec<&Table> = sections
        .iter()
        .flat_map(|s| s.all_tables().filter(|t| t.nesting_depth >= 1))
        .collect();

    assert!(
        !inner_tables.is_empty(),
        "table_nested must expose at least one nested (depth>=1) table inside a cell; \
         all tables discovered: {:?}",
        sections
            .iter()
            .flat_map(|s| s.all_tables().map(|t| (t.id, t.nesting_depth)))
            .collect::<Vec<_>>()
    );

    // The nesting pattern for the fixture is a single 1x1 inner table
    // inside cell (1,1) of the outer 2x2. nesting_depth exactly 1 is
    // what Phase 1c (#4) reads; assert it to guard against accidental
    // off-by-one depth changes in the walker.
    assert!(
        inner_tables.iter().any(|t| t.nesting_depth == 1),
        "expected a nested table at nesting_depth == 1; got depths: {:?}",
        inner_tables
            .iter()
            .map(|t| t.nesting_depth)
            .collect::<Vec<_>>()
    );
}
