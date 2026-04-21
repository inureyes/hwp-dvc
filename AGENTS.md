# hwp-dvc — project guide for Claude

A Rust port of Hancom's HWPX Document Validation Checker (DVC).

The canonical Windows/C++ implementation sits under `references/dvc/`.
Everything in this repository outside `references/` is the Rust
rewrite.

## Repository layout

```
hwp-dvc/
├── Cargo.toml                 # workspace root
├── crates/
│   ├── hwp-dvc-core/          # library: spec, document, checker, output
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs       # DvcError, DvcResult, ErrorCode
│   │       ├── spec/          # DvcSpec + category specs (CheckList)
│   │       ├── document/      # HwpxArchive, Document, RunTypeInfo
│   │       ├── checker/       # Checker, DvcErrorInfo, CheckLevel, OutputScope
│   │       └── output/        # JSON formatter
│   └── hwp-dvc-cli/           # binary: `hwp-dvc`
│       └── src/main.rs
├── references/                # C++ reference, NOT part of the build (gitignored)
├── README.md
└── CLAUDE.md                  # this file
```

## What we are porting

The reference DVC validates HWPX documents against a JSON-defined
spec. Ten categories are in scope for v0.1:

| # | Category          | C++ model (`Source/C*.h`) | C++ reader (`Source/R*.h`) | Rust spec (`spec/mod.rs`) |
|---|-------------------|---------------------------|-----------------------------|---------------------------|
| 1 | CharShape         | `CCharShape`              | `RCharShape`                | `CharShapeSpec`           |
| 2 | ParaShape         | `CParaShape`              | `RParaShape`                | `ParaShapeSpec`           |
| 3 | Table             | `CTable`                  | `RTable`, `RBorderFills`    | `TableSpec`               |
| 4 | SpecialCharacter  | `CSpecialCharacter`       | (scan text runs)            | `SpecialCharacterSpec`    |
| 5 | OutlineShape      | `COutlineShape`           | `ROutlineShape`             | `OutlineShapeSpec`        |
| 6 | Bullet            | `CBullet`                 | `RBullets`                  | `BulletSpec`              |
| 7 | ParaNumBullet     | `CParaNumBullet`          | `RBullets`                  | `ParaNumBulletSpec`       |
| 8 | Style             | `CStyle`                  | via header.xml              | `StyleSpec`               |
| 9 | Hyperlink         | `CHyperlink`              | via run control chars       | `HyperlinkSpec`           |
| 10| Macro             | `CMacro`                  | via header.xml              | `MacroSpec`               |

Error code ranges are kept identical to the reference so that output
consumers of the C++ tool can ingest Rust output unchanged. See
`crates/hwp-dvc-core/src/error.rs` (`ErrorCode` enum) and the
`JID_*` defines in `references/dvc/Source/JsonModel.h`.

## HWPX format in one paragraph

An HWPX file is a ZIP archive containing OWPML XML. The parts we care
about live under `Contents/`:

- `header.xml` — charshapes, parashapes, borderfills, bullets, numbers,
  outline, styles, fonts, tab definitions.
- `section0.xml`, `section1.xml`, … — paragraphs, runs, tables,
  controls, objects.

The C++ reference delegates parsing to Hancom's OWPML model DLL. In
Rust we parse XML directly with `quick-xml`. Do not try to link
against or wrap the OWPML DLL.

## Module map (C++ → Rust)

| C++ (`references/dvc/…`)                 | Rust (`crates/hwp-dvc-core/…`)            |
|------------------------------------------|-------------------------------------------|
| `export/ExportInterface.h` (`IDVC`)      | top-level re-exports in `lib.rs`          |
| `DVCModule.cpp/.h`                       | CLI `main.rs` + `Checker::run`            |
| `CommandParser.cpp/.h`                   | `clap` in `crates/hwp-dvc-cli/src/main.rs`|
| `Factory.cpp/.h`                         | `output::Format` + `to_json`              |
| `Checker.cpp/.h`                         | `checker::Checker`                        |
| `Source/CheckList.cpp/.h`, `C*.cpp/.h`   | `spec::*`                                 |
| `Source/OWPMLReader.cpp/.h`, `R*.cpp/.h` | `document::*`                             |
| `Source/DVCOutputJson.cpp/.h`            | `output::to_json`                         |
| `Source/DVCErrorInfo.cpp/.h`             | `checker::DvcErrorInfo`                   |
| `Source/JsonModel.h`                     | `error::ErrorCode` + per-category consts  |

When porting a C++ method, reference the source location in a comment
on the Rust side (e.g. `// port of Checker::CheckCharShape`). That
makes future diffs against the reference cheap.

## Conventions

- Rust 2021, MSRV 1.75, `thiserror` for library errors, `anyhow` only in
  the CLI binary. Never add `anyhow` as a core dependency.
- Errors: library returns `DvcResult<T>`; use `DvcError::NotImplemented`
  for intentional stubs so they surface clearly in logs.
- Feature gating: keep optional output formats (XML/text) behind Cargo
  features once added, not runtime flags.
- No `unsafe` in core without a block comment justifying it.
- Tests live next to the code (`#[cfg(test)] mod tests`). Integration
  tests with real HWPX fixtures should live under
  `crates/hwp-dvc-core/tests/` with fixtures under `tests/fixtures/`.
- Do not commit HWPX fixtures that contain proprietary content. Use the
  examples shipped with the reference only for local testing.
- The reference C++ uses wide strings on Windows (`wchar_t`) and narrow
  on Linux. In Rust we always use `String`/`&str` (UTF-8); translate at
  the XML parser boundary.

## Commands

```bash
# build everything
cargo build --workspace

# run tests
cargo test --workspace

# run the CLI (once Document::parse is implemented)
cargo run -p hwp-dvc-cli -- --spec path/to/spec.json path/to/doc.hwpx

# lint
cargo clippy --workspace --all-targets -- -D warnings

# format
cargo fmt --all
```

## Current status

- `spec` — struct-level model in place; parses the sample `test.json`
  shape. Missing fields should be added only when a corresponding
  checker needs them.
- `document` — ZIP reading works; OWPML XML parsing is a TODO
  (`Document::parse` returns `NotImplemented`).
- `checker` — skeleton only; `Checker::run` returns an empty vec.
  Individual `check_*` methods need porting.
- `output` — JSON formatter works end-to-end with the field names the
  reference produces.
- CLI — argument surface matches the reference; wired up to core.

## Porting priorities (suggested)

1. `Document::parse` → header.xml + section*.xml → `RunTypeInfo` list.
2. `spec` + `checker::check_charshape` + `checker::check_parashape`.
3. Table + SpecialCharacter + Style + Hyperlink + Macro (easier
   categories that don't require the Bullet/Outline numbering logic).
4. OutlineShape, Bullet, ParaNumBullet (require numbering state).
5. XML output, remaining error strings, localization.

## Things to avoid

- Do not reintroduce OWPML DLL bindings — the port is platform-neutral
  Rust.
- Do not rename the JSON output field names (`charIDRef`, `paraPrIDRef`,
  etc.); downstream tools match on them.
- Do not put business logic in the CLI crate; everything that is not
  argument parsing or I/O belongs in `hwp-dvc-core`.
- Never `unwrap()` in library code; propagate `DvcError` instead.
