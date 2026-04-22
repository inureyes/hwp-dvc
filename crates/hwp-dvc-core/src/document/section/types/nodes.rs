//! AST node definitions for an OWPML body section.
//!
//! The five kinds of nodes mirror the OWPML element hierarchy closely
//! enough that a walker can visit them without further indirection:
//!
//! ```text
//! Section
//! └── Paragraph (hp:p)
//!     ├── Run (hp:run)
//!     │   └── text / hyperlink marker
//!     └── Table (hp:tbl)                 // tables attached to this paragraph
//!         └── Row (hp:tr)
//!             └── Cell (hp:tc)
//!                 └── Paragraph (hp:subList/hp:p)   // recurses
//!                     └── Table (hp:tbl, nested)    // nesting_depth += 1
//! ```
//!
//! Nesting depth is recorded on every `Table` so that issue #4 can
//! set `RunTypeInfo::is_in_table_in_table` correctly without walking
//! the tree a second time: any table with `nesting_depth >= 1` is a
//! table-in-table.
//!
//! `Cell` paragraphs may recursively contain more tables; that case
//! drives the nesting-depth counter. See `parser::section` for the
//! walker that builds this AST.

/// A parsed OWPML body section — the output of parsing one
/// `Contents/section*.xml` part.
///
/// A well-formed section has at least one paragraph (HWPX writers
/// always emit at least the section-definition paragraph carrying
/// `<hp:secPr>`). The walker preserves document order: the `i`-th
/// element of `paragraphs` corresponds to the `i`-th `<hp:p>` child of
/// the `<hs:sec>` root.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Section {
    /// The ordinal of the source file this section was parsed from,
    /// extracted from the `Contents/sectionN.xml` filename. Preserved
    /// so that downstream code can re-emit the original file path in
    /// error messages.
    pub index: u32,
    /// `outlineShapeIDRef` attribute of the `<hp:secPr>` that sits
    /// inside the first `<hp:run>` of the first `<hp:p>`. This is a
    /// section-wide value that Phase 1c (#4) copies onto every
    /// `RunTypeInfo` in the section — mirroring the C++ reference's
    /// `OWPMLReader::GetRunTypeInfos` which resolves it via
    /// `FindObjectFromParents(pPType, ID_PARA_SectionDefinitionType)`.
    /// Defaults to 0 if the section has no `<hp:secPr>` (which only
    /// happens for malformed HWPX; well-formed writers always emit
    /// it on the section's first run).
    pub outline_shape_id_ref: u32,
    /// Paragraphs in document order.
    pub paragraphs: Vec<Paragraph>,
}

/// A single `<hp:p>` paragraph.
///
/// All OWPML paragraph elements carry `paraPrIDRef` and `styleIDRef`
/// attributes even when they reference id 0. We copy both here as
/// plain integers — resolution into [`ParaShape`] / [`Style`] records
/// is the validator's responsibility.
///
/// Tables attached to the paragraph are kept in a dedicated vector
/// rather than flattened into the run stream so that the walker can
/// reason about "paragraph has N tables" without re-scanning the
/// runs. In OWPML, tables live *inside* `<hp:run>` elements (a table
/// is a kind of inline object), but for the validator surface the
/// "which paragraph does this table belong to" relationship is what
/// matters.
///
/// [`ParaShape`]: crate::document::header::ParaShape
/// [`Style`]: crate::document::header::Style
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Paragraph {
    /// `paraPrIDRef` — index into [`HeaderTables::para_shapes`].
    ///
    /// [`HeaderTables::para_shapes`]: crate::document::header::HeaderTables
    pub para_pr_id_ref: u32,
    /// `styleIDRef` — index into [`HeaderTables::styles`].
    /// Zero means "no explicit style override"; validators interpret
    /// that as "use the default 바탕글 style".
    ///
    /// [`HeaderTables::styles`]: crate::document::header::HeaderTables
    pub style_id_ref: u32,
    /// Text-bearing runs in document order.
    pub runs: Vec<Run>,
    /// Tables owned by this paragraph, in document order.
    pub tables: Vec<Table>,
}

/// A single `<hp:run>` — a character-property-homogeneous slice of
/// a paragraph.
///
/// Only the fields the Phase 2 validators need are preserved:
///
/// - `char_pr_id_ref` drives `CheckCharShape` / `CheckSpecialCharacter`
/// - `text` is the concatenated body of `<hp:t>` children
/// - `is_hyperlink` is set when the run sits between a
///   `<hp:fieldBegin type="HYPERLINK">` / `<hp:fieldEnd>` pair.
///
/// Runs with no text (pure control runs carrying `<hp:secPr>` or
/// `<hp:ctrl>`) still show up — callers can filter them out when
/// reporting on run text.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Run {
    /// `charPrIDRef` — index into [`HeaderTables::char_shapes`].
    ///
    /// [`HeaderTables::char_shapes`]: crate::document::header::HeaderTables
    pub char_pr_id_ref: u32,
    /// Concatenated text content of every `<hp:t>` child of the run.
    /// Empty for control-only runs.
    pub text: String,
    /// `true` if this run is enclosed by a
    /// `<hp:fieldBegin type="HYPERLINK">` / `<hp:fieldEnd>` pair
    /// anywhere earlier in the paragraph.
    pub is_hyperlink: bool,
}

/// A single `<hp:tbl>` table.
///
/// `nesting_depth` is the count of enclosing `<hp:tbl>` ancestors;
/// top-level tables have depth 0 and a table directly inside another
/// table's cell has depth 1. Anything `>= 1` is a "table in table"
/// for issue #4's purposes.
///
/// # Attributes for the standard-mode validator (issue #41)
///
/// Attributes collected below map 1:1 onto the standard-mode
/// `JID_TABLE_*` error codes defined in
/// `references/dvc/Source/JsonModel.h`. Each carrier field is named
/// after the OWPML attribute it originates from, and the companion
/// raw-string form (`*_raw`) is kept for enum-valued attributes so
/// the validator does not have to repeat the
/// `"TOP_AND_BOTTOM"`→`ASOTWT_TOP_AND_BOTTOM` translation table the
/// reference keeps inside `OWPML::enumdef.h`. A missing or empty
/// string means the writer did not emit the attribute (HWPX allows
/// omission of defaults) and downstream checks treat it as unset.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    /// The `id` attribute of the `<hp:tbl>` element. Non-unique across
    /// nested tables but unique within a single section for a given
    /// nesting path, which is all the validators require.
    pub id: u32,
    /// `borderFillIDRef` — index into [`HeaderTables::border_fills`]
    /// for the table's outer border style.
    ///
    /// [`HeaderTables::border_fills`]: crate::document::header::HeaderTables
    pub border_fill_id_ref: u32,
    /// `rowCnt` attribute. Redundant with `rows.len()` but kept
    /// because the OWPML writer sometimes emits partial row trees
    /// (missing `<hp:tr>` tags) and the declared count is what a
    /// downstream validator should report.
    pub row_cnt: u32,
    /// `colCnt` attribute.
    pub col_cnt: u32,
    /// `cellSpacing` attribute — between-cell spacing in HWPUNITs
    /// (`JID_TABLE_BORDER_CELLSPACING`).
    pub cell_spacing: u32,
    /// Rows in document order.
    pub rows: Vec<Row>,
    /// 0 for a top-level table, 1 inside a cell of a top-level table,
    /// 2 inside a cell of a nested table, and so on.
    pub nesting_depth: u32,

    // ── <hp:tbl> attributes (text-wrap / numbering) ────────────────────
    /// `textWrap` attribute, raw string (`SQUARE`, `TOP_AND_BOTTOM`,
    /// `BEHIND_TEXT`, `IN_FRONT_OF_TEXT`). Empty when absent.
    pub text_wrap: String,
    /// `textFlow` attribute, raw string (`BOTH_SIDES`, `LEFT_ONLY`,
    /// `RIGHT_ONLY`, `LARGEST_ONLY`). Empty when absent.
    pub text_flow: String,
    /// `numberingType` attribute, raw string (`TABLE`, `PICTURE`,
    /// `EQUATION`). Empty when absent.
    pub numbering_type: String,
    /// `lock` attribute — 0 = unlocked, 1 = locked.
    pub lock: u32,
    /// `noAdjust` attribute — the `objectProtect` flag in DVC terms.
    pub no_adjust: u32,

    // ── <hp:sz> attributes ────────────────────────────────────────────
    /// `width` attribute of `<hp:sz>` in HWPUNITs.
    pub width: u32,
    /// `height` attribute of `<hp:sz>` in HWPUNITs.
    pub height: u32,
    /// `protect` attribute of `<hp:sz>` — `true` when size is fixed.
    pub size_protect: bool,

    // ── <hp:pos> attributes ───────────────────────────────────────────
    /// `treatAsChar` attribute of `<hp:pos>` — `true` when the table
    /// is treated as a single character in the text flow.
    pub treat_as_char: bool,
    /// `flowWithText` attribute of `<hp:pos>`.
    pub flow_with_text: bool,
    /// `allowOverlap` attribute of `<hp:pos>`.
    pub allow_overlap: bool,
    /// `holdAnchorAndSO` attribute of `<hp:pos>`.
    pub hold_anchor_and_so: bool,
    /// `affectLSpacing` attribute of `<hp:pos>` — the `parallel`
    /// flag in DVC terms.
    pub affect_l_spacing: bool,
    /// `horzRelTo` attribute, raw string (`PAPER`/`PAGE`/`COLUMN`/`PARA`).
    pub horz_rel_to: String,
    /// `vertRelTo` attribute, raw string (`PAPER`/`PAGE`/`PARA`).
    pub vert_rel_to: String,
    /// `horzAlign` attribute, raw string (`LEFT`/`CENTER`/`RIGHT`/`INSIDE`/`OUTSIDE`).
    pub horz_align: String,
    /// `vertAlign` attribute, raw string (`TOP`/`CENTER`/`BOTTOM`/`INSIDE`/`OUTSIDE`).
    pub vert_align: String,
    /// `horzOffset` attribute — signed offset in HWPUNITs.
    pub horz_offset: i32,
    /// `vertOffset` attribute — signed offset in HWPUNITs.
    pub vert_offset: i32,

    // ── <hp:outMargin> attributes ─────────────────────────────────────
    /// Outer left margin in HWPUNITs.
    pub out_margin_left: u32,
    /// Outer right margin in HWPUNITs.
    pub out_margin_right: u32,
    /// Outer top margin in HWPUNITs.
    pub out_margin_top: u32,
    /// Outer bottom margin in HWPUNITs.
    pub out_margin_bottom: u32,

    // ── Caption attributes (from an adjacent `<hp:caption>` element) ─
    /// `true` if the table carries a `<hp:caption>` sibling.
    ///
    /// Caption attributes below are only meaningful when this flag
    /// is `true`. When absent, the validator skips caption checks.
    pub has_caption: bool,
    /// Caption side, raw string (`LEFT`/`RIGHT`/`TOP`/`BOTTOM`).
    pub caption_side: String,
    /// Caption `sz` (width) in HWPUNITs.
    pub caption_size: u32,
    /// Caption `gap` (spacing from the table) in HWPUNITs.
    pub caption_spacing: i32,
    /// Caption `fullSz` attribute — `true` when caption spans the
    /// full containing width.
    pub caption_full_size: bool,
    /// Caption line-wrap attribute — typically emitted on the
    /// caption's inner paragraph definition.
    pub caption_line_wrap: bool,
}

/// A single `<hp:tr>` row inside a [`Table`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Row {
    pub cells: Vec<Cell>,
}

/// A single `<hp:tc>` cell inside a [`Row`].
///
/// `row`/`col` mirror the `<hp:cellAddr>` 0-indexed coordinates. The
/// paragraph vector is recursive — cell paragraphs are full
/// [`Paragraph`] values and can themselves own [`Table`]s, which is
/// exactly how the walker discovers nested tables.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cell {
    /// 0-indexed row address from `<hp:cellAddr rowAddr="..">`.
    pub row: u32,
    /// 0-indexed column address from `<hp:cellAddr colAddr="..">`.
    pub col: u32,
    /// `borderFillIDRef` for this cell's border style.
    pub border_fill_id_ref: u32,
    /// Paragraphs inside the cell's `<hp:subList>`, in document order.
    pub paragraphs: Vec<Paragraph>,
}

impl Section {
    /// Iterate over every table that appears anywhere in the section,
    /// yielding borrowed `Table` references in document order.
    ///
    /// The iterator visits tables at every nesting level, so a caller
    /// interested only in top-level tables can filter with
    /// `.filter(|t| t.nesting_depth == 0)`.
    pub fn all_tables(&self) -> impl Iterator<Item = &Table> + '_ {
        TableIter::new(self)
    }

    /// Iterate over every paragraph anywhere in the section,
    /// descending into table cells. Order is document order.
    pub fn all_paragraphs(&self) -> impl Iterator<Item = &Paragraph> + '_ {
        ParagraphIter::new(self)
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Depth-first iterator over every [`Table`] in a [`Section`].
///
/// Visits tables attached to a paragraph before descending into the
/// paragraphs nested inside each of that table's cells. This matches
/// the order in which [`Cell::paragraphs`] lays them out.
struct TableIter<'a> {
    // Stack of paragraphs still to descend into.
    para_stack: Vec<&'a Paragraph>,
    // Tables remaining to yield from the most recently popped paragraph.
    current_tables: std::slice::Iter<'a, Table>,
}

impl<'a> TableIter<'a> {
    fn new(section: &'a Section) -> Self {
        Self {
            para_stack: section.paragraphs.iter().rev().collect(),
            current_tables: [].iter(),
        }
    }
}

impl<'a> Iterator for TableIter<'a> {
    type Item = &'a Table;

    fn next(&mut self) -> Option<&'a Table> {
        loop {
            if let Some(t) = self.current_tables.next() {
                // Queue cell paragraphs so nested tables are yielded
                // after this one. Push in reverse so the stack pops in
                // document order.
                for row in t.rows.iter().rev() {
                    for cell in row.cells.iter().rev() {
                        for p in cell.paragraphs.iter().rev() {
                            self.para_stack.push(p);
                        }
                    }
                }
                return Some(t);
            }
            let p = self.para_stack.pop()?;
            self.current_tables = p.tables.iter();
        }
    }
}

/// Depth-first iterator over every [`Paragraph`] in a [`Section`],
/// descending into table cells.
struct ParagraphIter<'a> {
    stack: Vec<&'a Paragraph>,
}

impl<'a> ParagraphIter<'a> {
    fn new(section: &'a Section) -> Self {
        let mut stack: Vec<&'a Paragraph> = section.paragraphs.iter().rev().collect();
        // If no top-level paragraphs exist, the stack starts empty;
        // `next()` will return `None` immediately.
        stack.shrink_to_fit();
        Self { stack }
    }
}

impl<'a> Iterator for ParagraphIter<'a> {
    type Item = &'a Paragraph;

    fn next(&mut self) -> Option<&'a Paragraph> {
        let p = self.stack.pop()?;
        // Push cell paragraphs in reverse so they pop in document order.
        for t in p.tables.iter().rev() {
            for row in t.rows.iter().rev() {
                for cell in row.cells.iter().rev() {
                    for cp in cell.paragraphs.iter().rev() {
                        self.stack.push(cp);
                    }
                }
            }
        }
        Some(p)
    }
}
