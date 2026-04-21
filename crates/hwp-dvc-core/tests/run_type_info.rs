//! Integration tests for `Document::parse()` → `Vec<RunTypeInfo>`
//! against real HWPX fixtures committed under `tests/fixtures/docs/`.
//!
//! Each test opens a fixture end-to-end with [`Document::open`],
//! calls [`Document::parse`], and asserts a concrete domain fact on
//! the resulting `run_type_infos` stream.
//!
//! | Fixture                | Assertion                                                             |
//! |------------------------|-----------------------------------------------------------------------|
//! | `charshape_pass`       | at least one plain run with 함초롬바탕 charshape, no table, no link.  |
//! | `table_nested`         | at least one run inside a table; at least one inside a nested table.  |
//! | `hyperlink_external`   | at least one run flagged `is_hyperlink = true`.                       |
//!
//! The fixture-paired spec (`tests/fixtures/specs/fixture_spec.json`)
//! is not loaded here — the run-type builder is spec-agnostic; spec
//! wiring is tested by the validator integrations in later issues.

use std::path::PathBuf;

use hwp_dvc_core::document::header::FontLang;
use hwp_dvc_core::document::{Document, RunTypeInfo};

/// Absolute path to a fixture under `tests/fixtures/docs/`.
fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("fixtures");
    p.push("docs");
    p.push(name);
    p
}

fn parse(name: &str) -> Document {
    let mut doc = Document::open(fixture(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

/// Return `true` if `s` contains at least one character in the
/// Korean syllable range (U+AC00..=U+D7A3).
fn contains_korean(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

#[test]
fn charshape_pass_produces_plain_run_with_hamcho_font() {
    let doc = parse("charshape_pass.hwpx");
    let header = doc
        .header
        .as_ref()
        .expect("Document::parse must populate the header");

    assert!(
        !doc.run_type_infos.is_empty(),
        "charshape_pass must expose at least one RunTypeInfo"
    );

    // At least one run must have non-empty Korean text, sit outside
    // any table/hyperlink, and resolve its charshape to a set that
    // includes 함초롬바탕.
    let candidate: &RunTypeInfo = doc
        .run_type_infos
        .iter()
        .find(|r| {
            !r.text.is_empty()
                && !r.is_in_table
                && !r.is_hyperlink
                && contains_korean(&r.text)
                && header
                    .char_shapes
                    .get(&r.char_pr_id_ref)
                    .map(|cs| {
                        cs.font_names(&header.font_faces)
                            .iter()
                            .any(|n| n == "함초롬바탕")
                    })
                    .unwrap_or(false)
        })
        .unwrap_or_else(|| {
            panic!(
                "expected at least one plain non-table, non-hyperlink run with 함초롬바탕 \
                 charshape; got: {:?}",
                doc.run_type_infos
                    .iter()
                    .map(|r| (
                        r.text.clone(),
                        r.char_pr_id_ref,
                        r.is_in_table,
                        r.is_hyperlink
                    ))
                    .collect::<Vec<_>>()
            )
        });

    // Spot-check the flags on the same run, matching the fixture's
    // single-paragraph shape.
    assert!(
        !candidate.is_in_table_in_table,
        "charshape_pass has no tables at all"
    );
    assert_eq!(
        candidate.page_no, 0,
        "page_no must be 0 in this issue (layout deferred to #19)"
    );
    assert_eq!(
        candidate.line_no, 0,
        "line_no must be 0 in this issue (layout deferred to #19)"
    );

    // Sanity: the Hangul font of the charshape this run references
    // resolves to 함초롬바탕, mirroring what the header test asserts
    // at the table level.
    assert_eq!(
        header.font_name(candidate.char_pr_id_ref, FontLang::Hangul),
        Some("함초롬바탕"),
        "the chosen run's Hangul font must be 함초롬바탕"
    );
}

#[test]
fn table_nested_flags_runs_inside_tables() {
    let doc = parse("table_nested.hwpx");

    assert!(
        !doc.run_type_infos.is_empty(),
        "table_nested must expose at least one RunTypeInfo"
    );

    // At least one run must be flagged `is_in_table` — a cell of
    // either the outer or the inner table will satisfy this.
    assert!(
        doc.run_type_infos.iter().any(|r| r.is_in_table),
        "expected at least one RunTypeInfo with is_in_table=true; got: {:?}",
        doc.run_type_infos
            .iter()
            .map(|r| (r.text.clone(), r.is_in_table, r.is_in_table_in_table))
            .collect::<Vec<_>>()
    );

    // At least one run must be flagged `is_in_table_in_table` —
    // specifically the fixture's inner 1x1 table inside the outer
    // 2x2 cell (1,1). The section-parsing test already guarantees
    // the nesting depth is recorded; here we assert it propagates
    // through to the RunTypeInfo stream.
    assert!(
        doc.run_type_infos.iter().any(|r| r.is_in_table_in_table),
        "expected at least one RunTypeInfo with is_in_table_in_table=true; got: {:?}",
        doc.run_type_infos
            .iter()
            .filter(|r| r.is_in_table)
            .map(|r| (
                r.text.clone(),
                r.table_id,
                r.table_row,
                r.table_col,
                r.is_in_table_in_table
            ))
            .collect::<Vec<_>>()
    );

    // A run inside an inner table must also be flagged `is_in_table`
    // (not just `is_in_table_in_table`). If we ever decouple those
    // semantics this assertion catches it.
    for r in &doc.run_type_infos {
        if r.is_in_table_in_table {
            assert!(
                r.is_in_table,
                "is_in_table_in_table implies is_in_table; got text={:?}",
                r.text
            );
        }
    }

    // Sanity: runs outside tables exist too (the fixture's opening
    // paragraph before the table). Guards against a regression that
    // accidentally marks every run as table-bound.
    assert!(
        doc.run_type_infos.iter().any(|r| !r.is_in_table),
        "expected at least one top-level (non-table) RunTypeInfo in table_nested"
    );
}

#[test]
fn hyperlink_external_flags_hyperlink_run() {
    let doc = parse("hyperlink_external.hwpx");

    assert!(
        !doc.run_type_infos.is_empty(),
        "hyperlink_external must expose at least one RunTypeInfo"
    );

    let hyperlink_runs: Vec<&RunTypeInfo> = doc
        .run_type_infos
        .iter()
        .filter(|r| r.is_hyperlink)
        .collect();

    assert!(
        !hyperlink_runs.is_empty(),
        "expected at least one RunTypeInfo with is_hyperlink=true; got texts: {:?}",
        doc.run_type_infos
            .iter()
            .map(|r| (r.text.clone(), r.is_hyperlink))
            .collect::<Vec<_>>()
    );

    // A hyperlink run should not simultaneously be flagged as being
    // inside a table for this fixture — it is a plain-paragraph
    // hyperlink. Guards against the flag-propagation code
    // accidentally OR-ing unrelated flags.
    for r in hyperlink_runs {
        assert!(
            !r.is_in_table,
            "hyperlink run unexpectedly flagged in-table: {:?}",
            r.text
        );
    }
}

#[test]
fn parse_is_idempotent_and_replaces_prior_state() {
    let mut doc = Document::open(fixture("charshape_pass.hwpx")).expect("open ok");
    doc.parse().expect("first parse ok");
    let first_count = doc.run_type_infos.len();
    doc.parse().expect("second parse ok");
    assert_eq!(
        doc.run_type_infos.len(),
        first_count,
        "Document::parse must be idempotent — repeated calls yield the same RunTypeInfo count"
    );
    assert!(doc.header.is_some(), "header must remain populated");
}
