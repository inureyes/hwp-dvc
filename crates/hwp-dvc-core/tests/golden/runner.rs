//! Shared runner for the golden-test harness.
//!
//! Each golden case lives under `tests/golden/cases/<name>/` and contains:
//! - `config.json` — names the fixture HWPX and spec JSON to load.
//! - `expected.json` — the pretty JSON output captured from the current
//!   Rust build (written on the first run with `UPDATE_GOLDEN=1`).
//!
//! The actual HWPX fixtures and spec files are **not** duplicated into
//! the case directories — they are referenced by filename from the
//! shared `tests/fixtures/docs/` and `tests/fixtures/specs/` trees.
//!
//! See `crates/hwp-dvc-core/tests/golden/README.md` for the regeneration
//! procedure and for discussion of the Rust-snapshot vs C++-cross-verified
//! modes.
//!
//! # Public API
//!
//! - [`run_case`] — the single entry point every per-case `#[test]` calls.
//!   Given a case folder name (e.g. `"charshape_pass"`), it loads the
//!   fixture, runs the checker, serializes the result as pretty JSON,
//!   and either diffs against `expected.json` (default) or writes a
//!   fresh `expected.json` (when `UPDATE_GOLDEN=1`).
//!
//! # Environment variables
//!
//! | Variable         | Effect                                                       |
//! |------------------|--------------------------------------------------------------|
//! | `UPDATE_GOLDEN`  | When set (any non-empty value), write `expected.json` rather |
//! |                  | than diff against it. Used to regenerate snapshots.          |
//!
//! # Failure mode
//!
//! A mismatch panics with a human-readable diff-like message listing the
//! first differing characters and the full expected/actual blobs, so CI
//! logs surface enough context to understand the regression.
//!
//! Keeping this file under 200 lines intentionally — the harness should
//! be simple enough to audit at a glance.
#![allow(dead_code)] // Used by the `golden` integration test binary.

use std::path::{Path, PathBuf};

use hwp_dvc_core::checker::Checker;
use hwp_dvc_core::document::Document;
use hwp_dvc_core::output;
use hwp_dvc_core::spec::DvcSpec;
use serde::Deserialize;

/// Per-case metadata stored in `config.json`.
#[derive(Debug, Deserialize)]
struct CaseConfig {
    /// Filename of the HWPX fixture under `tests/fixtures/docs/`.
    fixture: String,
    /// Filename of the spec JSON under `tests/fixtures/specs/`.
    spec: String,
    /// Human-readable description — not used by the harness, kept so
    /// reviewers can read the intent of each case without opening the
    /// fixture.
    #[serde(default)]
    description: String,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn case_dir(name: &str) -> PathBuf {
    manifest_dir().join("tests/golden/cases").join(name)
}

fn fixtures_doc_dir() -> PathBuf {
    manifest_dir().join("tests/fixtures/docs")
}

fn fixtures_spec_dir() -> PathBuf {
    manifest_dir().join("tests/fixtures/specs")
}

fn load_config(case: &str) -> CaseConfig {
    let path = case_dir(case).join("config.json");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("golden case '{case}': reading {}: {e}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("golden case '{case}': parsing {}: {e}", path.display()))
}

/// Load the fixture, run the checker, and return the pretty-JSON output.
fn produce_actual(case: &str, cfg: &CaseConfig) -> String {
    let fixture_path = fixtures_doc_dir().join(&cfg.fixture);
    let spec_path = fixtures_spec_dir().join(&cfg.spec);

    let mut doc = Document::open(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "golden case '{case}': Document::open({}): {e}",
            fixture_path.display()
        )
    });
    doc.parse().unwrap_or_else(|e| {
        panic!(
            "golden case '{case}': Document::parse({}): {e}",
            fixture_path.display()
        )
    });

    let spec = DvcSpec::from_json_file(&spec_path).unwrap_or_else(|e| {
        panic!(
            "golden case '{case}': loading spec {}: {e}",
            spec_path.display()
        )
    });

    let checker = Checker::new(&spec, &doc);
    let errors = checker
        .run()
        .unwrap_or_else(|e| panic!("golden case '{case}': Checker::run: {e}"));

    output::to_json(&errors, true)
        .unwrap_or_else(|e| panic!("golden case '{case}': output::to_json: {e}"))
}

fn should_update() -> bool {
    std::env::var_os("UPDATE_GOLDEN").is_some_and(|v| !v.is_empty())
}

fn write_expected(expected_path: &Path, actual: &str) {
    // Append a trailing newline so committed files end cleanly.
    let mut payload = actual.to_owned();
    if !payload.ends_with('\n') {
        payload.push('\n');
    }
    std::fs::write(expected_path, payload).unwrap_or_else(|e| {
        panic!(
            "golden regen: writing {} failed: {e}",
            expected_path.display()
        )
    });
}

fn assert_matches(case: &str, expected_path: &Path, actual: &str) {
    let expected_raw = std::fs::read_to_string(expected_path).unwrap_or_else(|e| {
        panic!(
            "golden case '{case}': reading expected {} failed: {e}.\n\
             If this is a new case, run `UPDATE_GOLDEN=1 cargo test --test golden` \
             to generate it.",
            expected_path.display()
        )
    });
    // Normalize trailing whitespace so hand edits and regen output compare equal.
    let expected = expected_raw.trim_end_matches(['\n', '\r']);
    let actual_trimmed = actual.trim_end_matches(['\n', '\r']);

    if expected != actual_trimmed {
        let divergence = first_diff(expected, actual_trimmed);
        panic!(
            "golden case '{case}': output does not match {}.\n\
             First divergence at byte {divergence}.\n\
             --- expected ---\n{expected}\n\
             --- actual ---\n{actual_trimmed}\n\
             ----------------\n\
             To accept this output as the new baseline, run:\n\
             UPDATE_GOLDEN=1 cargo test -p hwp-dvc-core --test golden",
            expected_path.display()
        );
    }
}

/// Return the byte offset of the first differing character between
/// `expected` and `actual`, or the shorter length when one is a prefix
/// of the other.
fn first_diff(expected: &str, actual: &str) -> usize {
    expected
        .bytes()
        .zip(actual.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| expected.len().min(actual.len()))
}

/// Entry point invoked by each per-case `#[test]`.
///
/// - In default (check) mode: load the case's `config.json`, run the
///   checker, and panic if the serialized output differs from the
///   committed `expected.json`.
/// - In regen mode (`UPDATE_GOLDEN=1`): write the serialized output to
///   `expected.json`, creating or overwriting the file. The test still
///   passes in this mode so that `cargo test` reports success once every
///   case has been regenerated.
pub fn run_case(case: &str) {
    let cfg = load_config(case);
    let actual = produce_actual(case, &cfg);
    let expected_path = case_dir(case).join("expected.json");

    if should_update() {
        write_expected(&expected_path, &actual);
        return;
    }
    assert_matches(case, &expected_path, &actual);
}
