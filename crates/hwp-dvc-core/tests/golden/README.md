# Golden-test harness

End-to-end regression guard for `hwp-dvc-core`. Each case loads a real
HWPX fixture, runs `Checker::run` against a spec, serializes the result
as pretty JSON, and diffs that output against a committed
`expected.json` snapshot.

## Layout

```
tests/golden/
├── README.md                         (this file)
├── runner.rs                         (shared harness — see tests/golden.rs for entry)
└── cases/
    ├── charshape_pass/
    │   ├── config.json               (names the fixture + spec)
    │   └── expected.json             (committed snapshot)
    ├── charshape_fail_font/
    ├── macro_present/
    ├── table_nested/
    └── bullet_disallowed/
```

The test binary is defined at `tests/golden.rs` and registered by Cargo
automatically — no manifest change is required. Each case gets its own
`#[test]` function so that a failure names the exact case that regressed.

### Fixture reuse

The HWPX fixtures and spec JSON files are **not** duplicated into each
case directory. Each case's `config.json` references files in the shared
fixtures trees:

- HWPX: `crates/hwp-dvc-core/tests/fixtures/docs/<fixture>.hwpx`
- Spec: `crates/hwp-dvc-core/tests/fixtures/specs/<spec>.json`

## Running

Diff mode (default — what CI runs):

```bash
cargo test -p hwp-dvc-core --test golden
```

Regenerate snapshots against the current Rust build:

```bash
UPDATE_GOLDEN=1 cargo test -p hwp-dvc-core --test golden
```

Regeneration is required when:

- A validator intentionally changes its output (error-code, ordering,
  extra field, etc.).
- A new fixture or spec is added to the shared fixtures tree and a new
  golden case references it.
- An output-formatter change (e.g., a new JSON field) alters every
  snapshot.

Review the diff in `git status` after regenerating and commit the
updated `expected.json` files together with the code change that made
them necessary.

## Current mode — Rust-snapshot regression

The committed `expected.json` files are captured from the **current
Rust implementation**. This is a classic snapshot-testing pattern: it
guards against *regressions* (the Rust output silently changing) but
does **not** independently verify that the Rust output matches the
reference C++ `DVCModel` tool.

That is acceptable for `v0.1` because:

1. The reference C++ tool only builds on Windows (it links against
   Hancom's proprietary OWPML model DLL). Cross-verification therefore
   requires a Windows machine and is not reproducible inside a portable
   CI job.
2. Every per-validator integration test under `tests/check_*.rs`
   already asserts category-level invariants (e.g., the font-fail
   fixture produces at least one `CHARSHAPE_FONT` error). Those tests
   cover *semantic* correctness. The golden harness layers a *byte-for-
   byte* stability guarantee on top of that.

## Future mode — C++-cross-verified

Once a Windows reference build is available, the regen procedure will
change to:

1. Run `DVCModel.exe --spec fixture_spec.json <fixture>.hwpx` on each
   fixture and capture the output.
2. Write the captured output to each case's `expected.json`.
3. Commit the diff against the current (Rust-snapshot) snapshots.

From that point forward, a divergence between the Rust build and the
committed `expected.json` is a real regression against the reference —
not just a drift in the Rust implementation.

This README should be updated when that transition happens: change the
"Current mode" heading to "Historical mode" and promote the procedure
above into the active "Current mode" section.

## Adding a new case

1. Create `tests/golden/cases/<case_name>/config.json` pointing at the
   fixture and spec to exercise:

   ```json
   {
     "fixture": "my_new_fixture.hwpx",
     "spec": "fixture_spec.json",
     "description": "One-line intent of this case."
   }
   ```

2. Add a matching `#[test]` stub to `tests/golden.rs`:

   ```rust
   #[test]
   fn golden_my_new_case() {
       runner::run_case("my_new_case");
   }
   ```

3. Generate the snapshot and commit it:

   ```bash
   UPDATE_GOLDEN=1 cargo test -p hwp-dvc-core --test golden
   ```

Keep the cases small and focused. Five broad cases cover more ground
than fifty near-duplicate ones — the harness exists to catch
unexpected drift, not to re-prove the per-validator tests.
