//! [`build_run_type_infos`] — walk a parsed `HeaderTables` + section
//! AST and produce the `Vec<RunTypeInfo>` that Phase 2 validators
//! consume.
//!
//! The walk is a straight recursion through paragraphs and tables:
//! see `PARA_WALK` below. No layout information is produced — that is
//! the deferred issue #19.
//!
//! # Flag semantics
//!
//! Every emitted [`RunTypeInfo`] carries the booleans that the
//! Phase 2 validators gate on:
//!
//! | Flag                    | When set                                                          |
//! |-------------------------|-------------------------------------------------------------------|
//! | `is_in_table`           | run lives inside any table cell, at any nesting depth             |
//! | `is_in_table_in_table`  | the innermost enclosing table has `nesting_depth >= 1`            |
//! | `is_in_shape`           | run lives inside a `<hp:drawText>` — currently always `false`     |
//! | `is_hyperlink`          | the run sits between a `<hp:fieldBegin type="HYPERLINK">` pair    |
//! | `is_style`              | paragraph's `styleIDRef` does not resolve to the default 바탕글   |
//!
//! `is_in_shape` is intentionally wired as always-false: the section
//! AST from issue #3 does not yet surface `<hp:drawText>`
//! containers, and the Phase 2 validators that care about shapes
//! (notably `CheckCharShape` when it wants to skip runs inside
//! drawing objects) will add that plumbing in their own PRs. See
//! [`IS_IN_SHAPE_SIMPLIFICATION`].

use crate::document::header::HeaderTables;
use crate::document::section::{Cell, Paragraph, Section, Table};
use crate::document::RunTypeInfo;

/// Human-readable summary of which `<hp:run>` instances map to a
/// [`RunTypeInfo`]. Kept as a `const` so the rule is discoverable at
/// the module level and doc-searchable.
pub const RUN_TYPE_EMISSION_POLICY: &str =
    "one RunTypeInfo per <hp:run>…</hp:run> (non-empty element), matching the reference's \
     `if (pRunType && pRunType->HasChildList())` gate. Control-only empty runs emitted by the \
     section parser are still included because they carry charPrIDRef — excluding them would \
     drop the charshape signal the Phase 2 validator needs.";

/// Documentation marker: `is_in_shape` stays `false` in this issue.
/// Drawing-object containers are not yet surfaced by the Section AST
/// (issue #3); the Phase 2 validator that needs this discrimination
/// will extend the AST when it lands.
pub const IS_IN_SHAPE_SIMPLIFICATION: &str =
    "is_in_shape deferred: <hp:drawText> ancestors not carried through the section AST yet";

/// Consume a [`HeaderTables`] + `Vec<Section>` and produce the
/// flattened `Vec<RunTypeInfo>` in document order.
///
/// Walks every paragraph in every section — including paragraphs
/// nested inside table cells, recursively — and emits one
/// [`RunTypeInfo`] per run. `page_no` and `line_no` are left at 0
/// (see issue #19).
#[must_use]
pub fn build_run_type_infos(header: &HeaderTables, sections: &[Section]) -> Vec<RunTypeInfo> {
    // Resolve the default-style id lazily: many HWPX documents key
    // 바탕글 to id 0, but Hancom writers occasionally renumber it
    // and the issue specifically calls out to check the header's
    // Style table rather than hard-coding 0.
    let default_style = default_style_id(header);

    // Pre-size loosely: most fixtures emit under 200 runs; oversizing
    // is cheap because the struct is small.
    let mut out: Vec<RunTypeInfo> = Vec::with_capacity(sections.len() * 64);

    for section in sections {
        let ctx = SectionCtx {
            outline_shape_id_ref: section.outline_shape_id_ref,
            default_style_id: default_style,
        };
        for paragraph in &section.paragraphs {
            walk_paragraph(&ctx, paragraph, /* cell = */ None, &mut out);
        }
    }

    out
}

/// Resolve the built-in default-style id: the [`Style`] whose `name`
/// attribute matches `"바탕글"`. Returns 0 if the header has no such
/// entry (which never happens in a real HWPX but keeps the function
/// total).
///
/// Exposed as `pub` so integration tests can assert the resolution
/// mirrors what validators will see.
///
/// [`Style`]: crate::document::header::Style
#[must_use]
pub fn default_style_id(header: &HeaderTables) -> u32 {
    header
        .style_by_name("바탕글")
        .map(|s| s.id)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Internal walker
// ---------------------------------------------------------------------------

/// Section-wide context used by the walker. One per section —
/// intentionally cheap to copy because the walker passes it by
/// reference.
struct SectionCtx {
    /// The `outlineShapeIDRef` resolved from `<hp:secPr>` at section
    /// parse time. Mirrors the C++ reference's per-section resolution;
    /// see `RunTypeInfo::outline_shape_id_ref`.
    outline_shape_id_ref: u32,
    /// The style id corresponding to 바탕글. A paragraph whose
    /// `style_id_ref` equals this value is considered "unstyled".
    default_style_id: u32,
}

/// Table-cell context accumulated as the walker descends into nested
/// tables. Top-level paragraphs pass `None`.
#[derive(Clone, Copy)]
struct CellCtx {
    /// The id of the **innermost** enclosing `<hp:tbl>`. Mirrors the
    /// C++ reference's `pTableType = (CTableType*)pTc->GetParentObj()
    /// ->GetParentObj()` — i.e., the table that directly owns the
    /// enclosing cell, not the outermost.
    table_id: u32,
    /// The innermost cell's row address.
    row: u32,
    /// The innermost cell's column address.
    col: u32,
    /// True when the innermost enclosing table has
    /// `nesting_depth >= 1`. The section walker (#3) already
    /// encodes nesting depth on each [`Table`]; carrying it here
    /// lets the run builder avoid a second ancestry scan.
    is_table_in_table: bool,
}

/// Walk one [`Paragraph`], emitting one [`RunTypeInfo`] per run and
/// recursing into any tables owned by the paragraph.
fn walk_paragraph(
    ctx: &SectionCtx,
    p: &Paragraph,
    cell: Option<CellCtx>,
    out: &mut Vec<RunTypeInfo>,
) {
    let is_style = p.style_id_ref != ctx.default_style_id;
    for run in &p.runs {
        let mut info = RunTypeInfo {
            char_pr_id_ref: run.char_pr_id_ref,
            para_pr_id_ref: p.para_pr_id_ref,
            text: run.text.clone(),
            // Intentionally left 0 — see module-level
            // `PAGE_LINE_OUT_OF_SCOPE` and issue #19.
            page_no: 0,
            line_no: 0,
            outline_shape_id_ref: ctx.outline_shape_id_ref,
            is_hyperlink: run.is_hyperlink,
            is_style,
            // `is_in_shape` stays false for issue #4 (see module doc).
            is_in_shape: false,
            ..RunTypeInfo::default()
        };
        if let Some(c) = cell {
            info.is_in_table = true;
            info.is_in_table_in_table = c.is_table_in_table;
            info.table_id = c.table_id;
            info.table_row = c.row;
            info.table_col = c.col;
        }
        out.push(info);
    }

    for table in &p.tables {
        walk_table(ctx, table, out);
    }
}

/// Walk one [`Table`], recursing into each cell's paragraphs. The
/// table's own `nesting_depth` is used to seed `is_table_in_table`
/// for the runs discovered inside its cells.
fn walk_table(ctx: &SectionCtx, t: &Table, out: &mut Vec<RunTypeInfo>) {
    // The reference reports the cell's enclosing table id — not the
    // outermost table. The walker naturally satisfies that because
    // it visits the cell via its direct parent table, so `t.id` here
    // is the right value to stamp.
    let is_table_in_table = t.nesting_depth >= 1;
    for row in &t.rows {
        for cell in &row.cells {
            walk_cell(ctx, t.id, cell, is_table_in_table, out);
        }
    }
}

fn walk_cell(
    ctx: &SectionCtx,
    table_id: u32,
    cell: &Cell,
    is_table_in_table: bool,
    out: &mut Vec<RunTypeInfo>,
) {
    let cell_ctx = CellCtx {
        table_id,
        row: cell.row,
        col: cell.col,
        is_table_in_table,
    };
    for p in &cell.paragraphs {
        walk_paragraph(ctx, p, Some(cell_ctx), out);
    }
}

// ---------------------------------------------------------------------------
// Unit tests (header-free, synthetic AST)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::header::Style;
    use crate::document::section::{Cell, Paragraph, Row, Run, Section, Table};

    fn header_with_default_style(id: u32) -> HeaderTables {
        let mut h = HeaderTables::default();
        h.styles.insert(
            id,
            Style {
                id,
                style_type: "PARA".into(),
                name: "바탕글".into(),
                ..Default::default()
            },
        );
        h
    }

    fn plain_paragraph(style_id_ref: u32, para_pr_id_ref: u32, text: &str) -> Paragraph {
        Paragraph {
            para_pr_id_ref,
            style_id_ref,
            runs: vec![Run {
                char_pr_id_ref: 0,
                text: text.into(),
                is_hyperlink: false,
            }],
            tables: Vec::new(),
        }
    }

    #[test]
    fn emits_one_info_per_run_top_level() {
        let h = header_with_default_style(0);
        let mut s = Section::default();
        s.paragraphs.push(Paragraph {
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![
                Run {
                    char_pr_id_ref: 1,
                    text: "hello".into(),
                    is_hyperlink: false,
                },
                Run {
                    char_pr_id_ref: 2,
                    text: " world".into(),
                    is_hyperlink: false,
                },
            ],
            tables: Vec::new(),
        });

        let infos = build_run_type_infos(&h, &[s]);
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].text, "hello");
        assert_eq!(infos[0].char_pr_id_ref, 1);
        assert_eq!(infos[1].text, " world");
        assert_eq!(infos[1].char_pr_id_ref, 2);
        // Plain top-level runs: table flags off, page/line zero.
        for i in &infos {
            assert!(!i.is_in_table);
            assert!(!i.is_in_table_in_table);
            assert_eq!(i.page_no, 0);
            assert_eq!(i.line_no, 0);
        }
    }

    #[test]
    fn flags_hyperlink_runs() {
        let h = header_with_default_style(0);
        let mut s = Section::default();
        s.paragraphs.push(Paragraph {
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![
                Run {
                    char_pr_id_ref: 0,
                    text: "plain".into(),
                    is_hyperlink: false,
                },
                Run {
                    char_pr_id_ref: 0,
                    text: "linked".into(),
                    is_hyperlink: true,
                },
            ],
            tables: Vec::new(),
        });
        let infos = build_run_type_infos(&h, &[s]);
        assert!(!infos[0].is_hyperlink);
        assert!(infos[1].is_hyperlink);
    }

    #[test]
    fn flags_styled_paragraph_against_default_style() {
        // 바탕글 has id=7 in this synthetic header — a paragraph with
        // style_id_ref=0 is therefore *not* the default (is_style=true),
        // and style_id_ref=7 is the default (is_style=false).
        let h = header_with_default_style(7);
        let mut s = Section::default();
        s.paragraphs.push(plain_paragraph(0, 0, "not default"));
        s.paragraphs.push(plain_paragraph(7, 0, "default style"));

        let infos = build_run_type_infos(&h, &[s]);
        assert!(infos[0].is_style, "id=0 is not 바탕글(id=7) here");
        assert!(!infos[1].is_style, "id=7 is 바탕글 in this fixture");
    }

    #[test]
    fn flags_table_and_table_in_table() {
        let h = header_with_default_style(0);
        let mut s = Section::default();

        // Build a 1x1 table whose cell contains a 1x1 nested table
        // whose cell contains a paragraph. The outermost paragraph
        // carries no text but owns the outer table.
        let inner_para = Paragraph {
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![Run {
                char_pr_id_ref: 0,
                text: "deep".into(),
                is_hyperlink: false,
            }],
            tables: Vec::new(),
        };
        let inner_table = Table {
            id: 222,
            border_fill_id_ref: 0,
            row_cnt: 1,
            col_cnt: 1,
            rows: vec![Row {
                cells: vec![Cell {
                    row: 0,
                    col: 0,
                    border_fill_id_ref: 0,
                    paragraphs: vec![inner_para],
                }],
            }],
            nesting_depth: 1,
            ..Table::default()
        };
        let middle_para = Paragraph {
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![Run {
                char_pr_id_ref: 0,
                text: "middle".into(),
                is_hyperlink: false,
            }],
            tables: vec![inner_table],
        };
        let outer_table = Table {
            id: 111,
            border_fill_id_ref: 0,
            row_cnt: 1,
            col_cnt: 1,
            rows: vec![Row {
                cells: vec![Cell {
                    row: 0,
                    col: 0,
                    border_fill_id_ref: 0,
                    paragraphs: vec![middle_para],
                }],
            }],
            nesting_depth: 0,
            ..Table::default()
        };
        s.paragraphs.push(Paragraph {
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: Vec::new(),
            tables: vec![outer_table],
        });

        let infos = build_run_type_infos(&h, &[s]);
        // Two runs in total: one in the outer cell's paragraph
        // ("middle"), one in the inner cell's paragraph ("deep").
        assert_eq!(infos.len(), 2);

        let middle = infos.iter().find(|i| i.text == "middle").unwrap();
        assert!(middle.is_in_table);
        assert!(!middle.is_in_table_in_table);
        assert_eq!(middle.table_id, 111);

        let deep = infos.iter().find(|i| i.text == "deep").unwrap();
        assert!(deep.is_in_table);
        assert!(deep.is_in_table_in_table, "inner table has depth 1");
        assert_eq!(deep.table_id, 222, "innermost table id is stamped");
    }

    #[test]
    fn copies_outline_shape_id_ref_from_section() {
        let h = header_with_default_style(0);
        let s = Section {
            outline_shape_id_ref: 42,
            paragraphs: vec![plain_paragraph(0, 0, "hi")],
            ..Default::default()
        };

        let infos = build_run_type_infos(&h, &[s]);
        assert_eq!(infos[0].outline_shape_id_ref, 42);
    }

    #[test]
    fn default_style_id_falls_back_to_zero_when_style_missing() {
        // A header with no 바탕글 entry at all (a malformed document)
        // must still return a sensible u32 rather than panic.
        let h = HeaderTables::default();
        assert_eq!(default_style_id(&h), 0);
    }
}
