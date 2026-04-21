//! Smoke test: confirm the canonical Hancom DVC spec JSON files
//! (copied verbatim under `tests/fixtures/specs/`) parse cleanly into
//! our `DvcSpec` model. Catches regressions in `spec/mod.rs` that
//! would diverge from the reference schema.

use std::path::PathBuf;

use hwp_dvc_core::spec::DvcSpec;

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/specs");
    p.push(name);
    p
}

#[test]
fn hancom_test_spec_parses() {
    let spec = DvcSpec::from_json_file(fixture("hancom_test.json"))
        .expect("hancom_test.json should parse");
    assert!(spec.charshape.is_some());
    assert!(spec.parashape.is_some());
    assert!(spec.table.is_some());
}

#[test]
fn fixture_spec_parses_and_matches_defaults() {
    let spec = DvcSpec::from_json_file(fixture("fixture_spec.json"))
        .expect("fixture_spec.json should parse");
    let cs = spec.charshape.expect("fixture_spec has charshape");
    assert!(cs.font.iter().any(|f| f == "함초롬바탕"));
}

/// `hancom_full.json` is NOT a runnable spec. It is a key catalog
/// kept under `tests/fixtures/specs/` for developer reference when
/// extending `DvcSpec` in Phase 2+. The reference file intentionally
/// omits commas between fields and carries a `[Json schema]` header
/// marker, so it will not pass `serde_json::from_str` and is not
/// tested here. See `NOTICE` in the same directory.
#[test]
fn hancom_full_catalog_is_readable() {
    let raw = std::fs::read_to_string(fixture("hancom_full.json"))
        .expect("catalog readable");
    assert!(raw.contains("\"charshape\""));
    assert!(raw.contains("\"parashape\""));
    assert!(raw.contains("\"table\""));
}
