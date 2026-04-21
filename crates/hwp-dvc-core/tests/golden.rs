//! Golden-test suite — end-to-end regression guard against committed
//! `expected.json` snapshots for a curated set of HWPX fixtures.
//!
//! Run with:
//!
//! ```text
//! cargo test -p hwp-dvc-core --test golden
//! ```
//!
//! To regenerate the snapshots against the current Rust build (e.g.,
//! after an intentional output change):
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p hwp-dvc-core --test golden
//! ```
//!
//! See `tests/golden/README.md` for fixture selection rationale, the
//! Rust-snapshot vs C++-cross-verified distinction, and the full
//! regeneration procedure.

#[path = "golden/runner.rs"]
mod runner;

// ──────────────────────────────────────────────────────────────────────────────
// Per-case tests — each `#[test]` corresponds to one case folder under
// `tests/golden/cases/`. They are listed explicitly (rather than walking
// the directory at runtime) so that failures in `cargo test` output
// name the case that regressed.
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn golden_charshape_pass() {
    runner::run_case("charshape_pass");
}

#[test]
fn golden_charshape_fail_font() {
    runner::run_case("charshape_fail_font");
}

#[test]
fn golden_macro_present() {
    runner::run_case("macro_present");
}

#[test]
fn golden_table_nested() {
    runner::run_case("table_nested");
}

#[test]
fn golden_bullet_disallowed() {
    runner::run_case("bullet_disallowed");
}
