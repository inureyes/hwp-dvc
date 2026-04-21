# hwp-dvc

A Rust reimplementation of Hancom's **HWPX Document Validation Checker
(DVC)** — a tool that checks whether an HWPX document conforms to a
JSON-defined validation spec (allowed fonts, paragraph shapes, table
borders, hyperlink/macro policy, and so on).

The original Windows/C++ DVC from Hancom
([`hancom-io/dvc`](https://github.com/hancom-io/dvc)) is included under
`references/dvc/` for comparison. This project is a cross-platform,
OWPML-DLL-free rewrite in Rust.

> Status: **early work in progress.** The workspace compiles and the
> CLI surface is wired up, but the OWPML reader and individual
> validators are still being ported.

## Features

Planned validation categories (ten, matching the reference):

- Character shape (fonts, size, bold/italic, …)
- Paragraph shape (alignment, indents, line spacing, borders, …)
- Table (borders, margins, treat-as-char, nested tables, …)
- Special characters (allowed code-point range)
- Outline shape / numbering
- Bullet shapes
- Paragraph numbering
- Style permission
- Hyperlink permission
- Macro permission

Output: JSON today; XML and plain text will follow.

## Install / build

Requires Rust 1.75 or newer.

```bash
cargo build --workspace --release
```

The binary is produced at `target/release/hwp-dvc`.

## Usage

```bash
hwp-dvc --spec path/to/spec.json path/to/document.hwpx
```

Common options (mirroring the reference tool):

| flag             | alias | description                                              |
|------------------|-------|----------------------------------------------------------|
| `--spec <PATH>`  | `-f`  | DVC spec JSON (the "checklist"). Required.               |
| `--format json`  | `-F`  | Output format. `json` is the only value today.           |
| `--file <PATH>`  |       | Write output to a file instead of stdout.                |
| `--pretty`       |       | Pretty-print JSON output.                                |
| `--simple`       | `-s`  | Stop at the first error (default: report all).           |
| `--alloption`    | `-o`  | Include every category in the output.                    |
| `--table`        | `-t`  | Include table findings.                                  |
| `--tabledetail`  | `-i`  | Include per-cell table findings.                         |
| `--shape`        | `-p`  | Include shape findings.                                  |
| `--style`        | `-y`  | Include style findings.                                  |
| `--hyperlink`    | `-k`  | Include hyperlink findings.                              |
| `--help`         | `-h`  | Help.                                                    |

A minimal spec looks like:

```json
{
  "charshape": {
    "langtype": "대표",
    "font": ["바탕", "맑은 고딕"],
    "ratio": 100,
    "spacing": 0
  },
  "style":     { "permission": false },
  "hyperlink": { "permission": false },
  "macro":     { "permission": false }
}
```

(See `references/dvc/sample/test.json` for a longer example and
`references/dvc/sample/jsonFullSpec.json` for every supported key.)

## Output format

The JSON output matches the reference tool so that existing consumers
continue to work:

```json
[
  {
    "charIDRef": 6,
    "paraPrIDRef": 0,
    "text": "",
    "pageNo": 2,
    "lineNo": 4,
    "errorCode": 1005,
    "tableID": 0,
    "isInTable": false,
    "isInTableInTable": false,
    "tableRow": 0,
    "tableCol": 0
  }
]
```

Error codes are grouped by category. For example `1000+` are character
shape, `2000+` are paragraph shape, `3000+` are tables, `7000+` are
macros. See `crates/hwp-dvc-core/src/error.rs` for the full list.

## Repository layout

```
hwp-dvc/
├── crates/
│   ├── hwp-dvc-core/   # library: spec parsing, HWPX reader, checker, output
│   └── hwp-dvc-cli/    # `hwp-dvc` binary
├── references/         # C++ reference (not built; gitignored)
├── Cargo.toml          # workspace root
├── CLAUDE.md           # contributor/agent guide
└── README.md
```

## License

Licensed under the Apache License, Version 2.0. See individual files
for copyright notices. The reference implementation under
`references/dvc/` is also Apache-2.0 licensed and © Hancom Inc.

## Acknowledgements

- [`hancom-io/dvc`](https://github.com/hancom-io/dvc) — the original
  DVC that this port mirrors.
- [`hancom-io/hwpx-owpml-model`](https://github.com/hancom-io/hwpx-owpml-model) —
  the OWPML model used by the reference; we reimplement the parts we
  need directly in Rust.
