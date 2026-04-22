//! `CheckTable` — standard-mode table validator.
//!
//! Validates size, position, alignment, margins, caption, borders, and
//! nesting policy for every `<hp:tbl>` in the document against
//! [`TableSpec`]. Maps to `Checker::CheckTable` /
//! `CheckTableToCheckList` / `CheckFromBorderInfo` in
//! `references/dvc/Checker.cpp` and covers the standard-mode
//! `JID_TABLE_*` codes (3001-3036 and 3056).
//!
//! # Scope
//!
//! This module is the **standard-mode** validator. Cell-level
//! fills / gradients / pictures (`JID_TABLE_*` codes 3037-3055) fall
//! under the separate `--tabledetail` mode and are implemented by a
//! different worker (issue #42). When that mode lands, the
//! [`OutputScope::table_detail`] toggle selects between the two.
//!
//! [`OutputScope::table_detail`]: crate::checker::OutputScope::table_detail
//!
//! # Border position encoding
//!
//! [`BorderSpec::position`] uses the following integer codes, matching the
//! reference C++ `JID_BORDER_*` constants (see `JsonModel.h`):
//! - `1` = left
//! - `2` = right
//! - `3` = top
//! - `4` = bottom
//!
//! # Border type encoding
//!
//! [`BorderSpec::bordertype`] is an index into the Korean border-name
//! enumeration from the DVC schema (see `hancom_full.json`):
//! - `0` = NONE
//! - `1` = SOLID (실선)
//! - `2` = DASH  (파선)
//! - …
//!
//! # Color encoding
//!
//! [`BorderSpec::color`] is a packed 24-bit RGB integer (0x00RRGGBB).
//! The document stores the color as an `"#RRGGBB"` hex string.
//! `color == 0` corresponds to `"#000000"` (black).
//!
//! # Deferred fields
//!
//! `rotation` / `gradientH` / `gradientV` (codes 3017-3019) are defined
//! in the spec and accepted by the deserializer but not yet validated:
//! the `Table` AST does not expose `<hp:rotationInfo>` or the gradient
//! offset attributes. The reference C++ also leaves those cases empty.
//! See the `// TODO: rotation/gradient` comment inside
//! [`check_table`] for the reintroduction plan.
//!
//! # Cell-detail mode
//!
//! When [`OutputScope::table_detail`] is `true` **and** [`TableSpec`]
//! carries at least one cell-detail field (see
//! [`TableSpec::has_cell_detail_fields`]), this module runs the
//! per-cell walker in [`mod@detail`]. Detail-mode findings populate
//! `table_row` / `table_col` on the resulting [`DvcErrorInfo`] so
//! downstream output can report which cell violated. Error codes in
//! detail mode are in the range 3037..=3055 (see
//! [`crate::error::table_detail_codes`]).

pub(super) mod detail;

use crate::checker::{CheckLevel, DvcErrorInfo, OutputScope};
use crate::document::header::types::{Border, LineType};
use crate::document::section::types::Table;
use crate::document::{Document, HeaderTables};
use crate::error::{DvcResult, ErrorContext};
use crate::spec::{BorderSpec, IntRange, TableSpec};

// ---------------------------------------------------------------------------
// Error codes (mirrored in error.rs)
// ---------------------------------------------------------------------------
//
// Re-exported so existing call sites (`checker::table::TABLE_BORDER_TYPE`)
// keep working, and so every standard-mode JID_TABLE_* constant can be
// reached from one place.

pub use crate::error::{
    TABLE_BORDER_CELL_SPACING, TABLE_BORDER_COLOR, TABLE_BORDER_SIZE, TABLE_BORDER_TYPE,
    TABLE_CAPTION_LINE_WRAP, TABLE_CAPTION_POSITION, TABLE_CAPTION_SIZE,
    TABLE_CAPTION_SOCAP_FULL_SIZE, TABLE_CAPTION_SPACING, TABLE_GRADIENT_H, TABLE_GRADIENT_V,
    TABLE_HDIRECTION, TABLE_HTYPE, TABLE_HVALUE, TABLE_IN_TABLE, TABLE_MARGIN_BOTTOM,
    TABLE_MARGIN_LEFT, TABLE_MARGIN_RIGHT, TABLE_MARGIN_TOP, TABLE_NUM_VER_TYPE, TABLE_OBJ_PROTECT,
    TABLE_PARALLEL, TABLE_POS, TABLE_ROTATION, TABLE_SIZE_FIXED, TABLE_SIZE_HEIGHT,
    TABLE_SIZE_WIDTH, TABLE_SOALLOW_OVERLAP, TABLE_SOFLOW_WITH_TEXT, TABLE_SOHOLD_ANCHOR_OBJ,
    TABLE_TEXT_POS, TABLE_TREAT_AS_CHAR, TABLE_VDIRECTION, TABLE_VTYPE, TABLE_VVALUE,
};

// Re-export every cell-detail error code under this module so callers
// and integration tests can reach them via
// `checker::table::TABLE_BGFILL_TYPE` without importing the
// `error::table_detail_codes` submodule directly.
pub use crate::error::table_detail_codes::{
    TABLE_BGFILL_FACECOLOR, TABLE_BGFILL_PATTONCOLOR, TABLE_BGFILL_PATTONTYPE, TABLE_BGFILL_TYPE,
    TABLE_BGGRADATION_BLURCENTER, TABLE_BGGRADATION_BLURLEVEL, TABLE_BGGRADATION_ENDCOLOR,
    TABLE_BGGRADATION_GRADATIONANGLE, TABLE_BGGRADATION_HEIGHTCENTER, TABLE_BGGRADATION_STARTCOLOR,
    TABLE_BGGRADATION_TYPE, TABLE_BGGRADATION_WIDTHCENTER, TABLE_EFFECT_TYPE, TABLE_EFFECT_VALUE,
    TABLE_PICTUREFILL_TYPE, TABLE_PICTUREFILL_VALUE, TABLE_PICTURE_FILE, TABLE_PICTURE_INCLUDE,
    TABLE_WATERMARK,
};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all table checks for the given document against `spec`.
///
/// Returns a (possibly empty) vector of [`DvcErrorInfo`] records. The
/// `level` and `scope` parameters are forwarded from [`crate::checker::Checker`].
///
/// When `scope.table_detail` is `true` and `spec` carries cell-detail
/// fields, this function also walks every cell of every table and
/// emits findings in the 3037..=3055 range.
pub fn check(
    document: &Document,
    spec: &TableSpec,
    level: CheckLevel,
    scope: OutputScope,
) -> DvcResult<Vec<DvcErrorInfo>> {
    // `OutputScope::allows(ScopeCategory::Table)` is evaluated by the
    // caller (`Checker::run`). Standard-mode rules (`JID_TABLE_*` codes
    // 3001-3036 and 3056) always run; cell-detail rules (3037-3055) run
    // only when `scope.table_detail` is set and the spec opts in via
    // [`TableSpec::has_cell_detail_fields`].
    let header = match &document.header {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let mut errors = Vec::new();
    let run_detail = scope.table_detail && spec.has_cell_detail_fields();

    for section in &document.sections {
        for table in section.all_tables() {
            check_table(table, spec, header, level, &mut errors);
            if level == CheckLevel::Simple && !errors.is_empty() {
                return Ok(errors);
            }
            if run_detail {
                detail::check_cells(table, spec, header, level, &mut errors);
                if level == CheckLevel::Simple && !errors.is_empty() {
                    return Ok(errors);
                }
            }
        }
    }

    Ok(errors)
}

// ---------------------------------------------------------------------------
// Per-table validation
// ---------------------------------------------------------------------------

/// Per-table orchestrator.
///
/// The C++ reference `Checker::CheckTableToCheckList` iterates over the
/// spec's "checked list" — the ordered set of fields the user declared
/// in the JSON spec — and dispatches via a giant `switch` on
/// `JID_TABLE_*`. In Rust the checked-list semantics fall out naturally
/// from `Option<...>`: only fields the spec actually set are compared,
/// so we do not need to replicate the "checked list" bookkeeping.
///
/// The function is kept linear and procedural; each block below
/// corresponds to one or more `JID_TABLE_*` codes and matches the
/// reference's order of evaluation.
fn check_table(
    table: &Table,
    spec: &TableSpec,
    header: &HeaderTables,
    level: CheckLevel,
    errors: &mut Vec<DvcErrorInfo>,
) {
    // Short-circuit helper: record an error and, when running in
    // `CheckLevel::Simple`, signal the caller to stop. The closure
    // captures a mutable reference to `errors` plus the table identity.
    let emit = |errors: &mut Vec<DvcErrorInfo>, code: u32| {
        errors.push(DvcErrorInfo {
            error_code: code,
            table_id: table.id,
            is_in_table: table.nesting_depth >= 1,
            is_in_table_in_table: table.nesting_depth >= 2,
            error_string: crate::error::error_string(code, ErrorContext::default()),
            ..Default::default()
        });
    };

    macro_rules! bail_if_simple {
        () => {
            if level == CheckLevel::Simple && !errors.is_empty() {
                return;
            }
        };
    }

    // ── Size: width / height / fixed ───────────────────────────────────
    if let Some(range) = spec.size_width {
        if !range.contains(i64::from(table.width)) {
            emit(errors, TABLE_SIZE_WIDTH);
            bail_if_simple!();
        }
    }
    if let Some(range) = spec.size_height {
        if !range.contains(i64::from(table.height)) {
            emit(errors, TABLE_SIZE_HEIGHT);
            bail_if_simple!();
        }
    }
    if let Some(required) = spec.fixed {
        if table.size_protect != required {
            emit(errors, TABLE_SIZE_FIXED);
            bail_if_simple!();
        }
    }

    // ── treatAsChar ────────────────────────────────────────────────────
    // Reference semantics: error only when the spec requires `true` but
    // the document attribute is `false`. A spec value of `false`
    // disables the check entirely (matches CheckTableToCheckList).
    if spec.treat_as_char == Some(true) && !table.treat_as_char {
        emit(errors, TABLE_TREAT_AS_CHAR);
        bail_if_simple!();
    }

    // ── Position / text-wrap ──────────────────────────────────────────
    if let Some(expected) = spec.pos {
        if let Some(actual) = pos_type_from_str(&table.text_wrap) {
            if actual != expected {
                emit(errors, TABLE_POS);
                bail_if_simple!();
            }
        }
    }
    if let Some(expected) = spec.textpos {
        if let Some(actual) = text_pos_from_str(&table.text_flow) {
            if actual != expected {
                emit(errors, TABLE_TEXT_POS);
                bail_if_simple!();
            }
        }
    }

    // ── Horizontal alignment ──────────────────────────────────────────
    if let Some(expected) = spec.horizontal_type {
        if let Some(actual) = horz_rel_to_from_str(&table.horz_rel_to) {
            if actual != expected {
                emit(errors, TABLE_HTYPE);
                bail_if_simple!();
            }
        }
    }
    if let Some(expected) = spec.horizontal_direction {
        if let Some(actual) = horz_align_from_str(&table.horz_align) {
            if actual != expected {
                emit(errors, TABLE_HDIRECTION);
                bail_if_simple!();
            }
        }
    }
    if let Some(range) = spec.horizontal_value {
        if !range.contains(i64::from(table.horz_offset)) {
            emit(errors, TABLE_HVALUE);
            bail_if_simple!();
        }
    }

    // ── Vertical alignment ────────────────────────────────────────────
    if let Some(expected) = spec.vertical_type {
        if let Some(actual) = vert_rel_to_from_str(&table.vert_rel_to) {
            if actual != expected {
                emit(errors, TABLE_VTYPE);
                bail_if_simple!();
            }
        }
    }
    if let Some(expected) = spec.vertical_direction {
        if let Some(actual) = vert_align_from_str(&table.vert_align) {
            if actual != expected {
                emit(errors, TABLE_VDIRECTION);
                bail_if_simple!();
            }
        }
    }
    if let Some(range) = spec.vertical_value {
        if !range.contains(i64::from(table.vert_offset)) {
            emit(errors, TABLE_VVALUE);
            bail_if_simple!();
        }
    }

    // ── Flow flags ────────────────────────────────────────────────────
    if let Some(required) = spec.soflowwithtext {
        if table.flow_with_text != required {
            emit(errors, TABLE_SOFLOW_WITH_TEXT);
            bail_if_simple!();
        }
    }
    if let Some(required) = spec.soallowoverlap {
        if table.allow_overlap != required {
            emit(errors, TABLE_SOALLOW_OVERLAP);
            bail_if_simple!();
        }
    }
    if let Some(required) = spec.soholdanchorobj {
        if table.hold_anchor_and_so != required {
            emit(errors, TABLE_SOHOLD_ANCHOR_OBJ);
            bail_if_simple!();
        }
    }
    if let Some(required) = spec.parallel {
        if table.affect_l_spacing != required {
            emit(errors, TABLE_PARALLEL);
            bail_if_simple!();
        }
    }

    // ── Rotation / gradient offsets ────────────────────────────────────
    // TODO: rotation/gradientH/gradientV are table-transform attributes
    // not surfaced by the current `Table` AST node. The reference C++
    // `CheckTableToCheckList` has empty cases for these three (they were
    // never implemented upstream either). We accept the spec field so
    // a caller can migrate forward but skip the comparison until the
    // AST grows those fields.
    let _ = TABLE_ROTATION;
    let _ = TABLE_GRADIENT_H;
    let _ = TABLE_GRADIENT_V;
    let _ = spec.rotation;
    let _ = spec.gradient_h;
    let _ = spec.gradient_v;

    // ── Numbering type / object protect ───────────────────────────────
    if let Some(expected) = spec.numbertype {
        if let Some(actual) = num_type_from_str(&table.numbering_type) {
            if actual != expected {
                emit(errors, TABLE_NUM_VER_TYPE);
                bail_if_simple!();
            }
        }
    }
    if let Some(required) = spec.objprotect {
        // OWPML uses 0/1 in the `noAdjust` attribute; treat non-zero as true.
        let actual = table.no_adjust != 0;
        if actual != required {
            emit(errors, TABLE_OBJ_PROTECT);
            bail_if_simple!();
        }
    }

    // ── Outer margins ─────────────────────────────────────────────────
    check_range(
        errors,
        &emit,
        spec.margin_left,
        i64::from(table.out_margin_left),
        TABLE_MARGIN_LEFT,
    );
    bail_if_simple!();
    check_range(
        errors,
        &emit,
        spec.margin_right,
        i64::from(table.out_margin_right),
        TABLE_MARGIN_RIGHT,
    );
    bail_if_simple!();
    check_range(
        errors,
        &emit,
        spec.margin_top,
        i64::from(table.out_margin_top),
        TABLE_MARGIN_TOP,
    );
    bail_if_simple!();
    check_range(
        errors,
        &emit,
        spec.margin_bottom,
        i64::from(table.out_margin_bottom),
        TABLE_MARGIN_BOTTOM,
    );
    bail_if_simple!();

    // ── Caption ───────────────────────────────────────────────────────
    // Caption checks only fire when the document actually carries a
    // `<hp:caption>` element. Specs that mandate a caption for every
    // table belong to a future "caption required" code and are not in
    // scope for this issue.
    if table.has_caption {
        if let Some(expected) = spec.caption_position {
            if let Some(actual) = caption_pos_from_str(&table.caption_side) {
                if actual != expected {
                    emit(errors, TABLE_CAPTION_POSITION);
                    bail_if_simple!();
                }
            }
        }
        check_range(
            errors,
            &emit,
            spec.caption_size,
            i64::from(table.caption_size),
            TABLE_CAPTION_SIZE,
        );
        bail_if_simple!();
        check_range(
            errors,
            &emit,
            spec.caption_spacing,
            i64::from(table.caption_spacing),
            TABLE_CAPTION_SPACING,
        );
        bail_if_simple!();
        if let Some(required) = spec.caption_socapfullsize {
            if table.caption_full_size != required {
                emit(errors, TABLE_CAPTION_SOCAP_FULL_SIZE);
                bail_if_simple!();
            }
        }
        if let Some(required) = spec.caption_linewrap {
            if table.caption_line_wrap != required {
                emit(errors, TABLE_CAPTION_LINE_WRAP);
                bail_if_simple!();
            }
        }
    }

    // ── Border (per-position rules) ───────────────────────────────────
    if !spec.border.is_empty() {
        if let Some(bf) = header.border_fills.get(&table.border_fill_id_ref) {
            for border_spec in &spec.border {
                let doc_border = border_by_position(bf, border_spec.position);
                if let Some(border) = doc_border {
                    check_border_from_info(table, border, border_spec, level, errors);
                    if level == CheckLevel::Simple && !errors.is_empty() {
                        return;
                    }
                }
            }
        }
    }

    // ── Border cell-spacing (3036) ────────────────────────────────────
    if let Some(range) = spec.border_cellspacing {
        if !range.contains(i64::from(table.cell_spacing)) {
            emit(errors, TABLE_BORDER_CELL_SPACING);
            bail_if_simple!();
        }
    }

    // ── Table-in-table ────────────────────────────────────────────────
    if spec.table_in_table == Some(false) && table.nesting_depth >= 1 {
        emit(errors, TABLE_IN_TABLE);
    }
}

/// Range-check helper — emits `code` when `value` falls outside `range`.
///
/// Accepts the `emit` closure by reference so the caller keeps control
/// of identity-tagging (table id, nesting depth, …) without duplicating
/// that logic per call-site.
fn check_range<F>(
    errors: &mut Vec<DvcErrorInfo>,
    emit: &F,
    range: Option<IntRange>,
    value: i64,
    code: u32,
) where
    F: Fn(&mut Vec<DvcErrorInfo>, u32),
{
    if let Some(r) = range {
        if !r.contains(value) {
            emit(errors, code);
        }
    }
}

// ---------------------------------------------------------------------------
// Border-info sub-check (mirrors CheckFromBorderInfo in Checker.cpp)
// ---------------------------------------------------------------------------

fn check_border_from_info(
    table: &Table,
    doc_border: &Border,
    spec_border: &BorderSpec,
    level: CheckLevel,
    errors: &mut Vec<DvcErrorInfo>,
) {
    // Border type check
    let expected_line_type = border_type_to_line_type(spec_border.bordertype);
    if doc_border.line_type != expected_line_type {
        errors.push(DvcErrorInfo {
            error_code: TABLE_BORDER_TYPE,
            table_id: table.id,
            is_in_table: table.nesting_depth >= 1,
            error_string: crate::error::error_string(TABLE_BORDER_TYPE, ErrorContext::default()),
            ..Default::default()
        });
        if level == CheckLevel::Simple {
            return;
        }
    }

    // Border size check (tolerance: 0.005 mm)
    let spec_size = spec_border.size as f32;
    if (doc_border.width_mm - spec_size).abs() > 0.005 {
        errors.push(DvcErrorInfo {
            error_code: TABLE_BORDER_SIZE,
            table_id: table.id,
            is_in_table: table.nesting_depth >= 1,
            error_string: crate::error::error_string(TABLE_BORDER_SIZE, ErrorContext::default()),
            ..Default::default()
        });
        if level == CheckLevel::Simple {
            return;
        }
    }

    // Border color check
    let expected_color_str = color_u32_to_hex(spec_border.color);
    if doc_border.color.to_ascii_uppercase() != expected_color_str {
        errors.push(DvcErrorInfo {
            error_code: TABLE_BORDER_COLOR,
            table_id: table.id,
            is_in_table: table.nesting_depth >= 1,
            error_string: crate::error::error_string(TABLE_BORDER_COLOR, ErrorContext::default()),
            ..Default::default()
        });
        // TABLE_BORDER_COLOR is the last check in this function;
        // early-return for Simple level is handled by the caller.
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the [`Border`] for position code 1–4 from a `BorderFill`.
///
/// Position mapping (from `hancom_full.json` / C++ `JID_BORDER_*`):
/// - 1 = left, 2 = right, 3 = top, 4 = bottom
fn border_by_position(bf: &crate::document::header::BorderFill, position: u32) -> Option<&Border> {
    match position {
        1 => Some(&bf.left),
        2 => Some(&bf.right),
        3 => Some(&bf.top),
        4 => Some(&bf.bottom),
        _ => None,
    }
}

/// Map a DVC spec `bordertype` integer to a [`LineType`].
///
/// Matches the Korean border-name enum in `hancom_full.json`:
/// 0=없음(NONE), 1=실선(SOLID), 2=파선(DASH), 3=점선(DOT), …
fn border_type_to_line_type(bordertype: u32) -> LineType {
    match bordertype {
        0 => LineType::None,
        1 => LineType::Solid,
        2 => LineType::Dash,
        3 => LineType::Dot,
        4 => LineType::DashDot,
        5 => LineType::DashDotDot,
        6 => LineType::LongDash,
        7 => LineType::Circle,
        8 => LineType::DoubleSlim,
        9 => LineType::SlimThick,
        10 => LineType::ThickSlim,
        11 => LineType::SlimThickSlim,
        12 => LineType::Wave,
        13 => LineType::DoubleWave,
        _ => LineType::Other,
    }
}

/// Convert a packed 24-bit RGB integer to an uppercase `"#RRGGBB"` string.
///
/// `color == 0` → `"#000000"` (black), matching the HWPX XML attribute
/// format used by [`Border::color`].
fn color_u32_to_hex(color: u32) -> String {
    format!("#{:06X}", color & 0x00FF_FFFF)
}

// ---------------------------------------------------------------------------
// OWPML string-enum → DVC integer-enum mappers
// ---------------------------------------------------------------------------
//
// Each helper returns `None` for an empty/unknown literal so the
// checker can treat "attribute not emitted" as "no check possible"
// rather than as a false negative. The numeric return values match
// the enum definitions in
// `references/dvc/Source/DVCInterface.h` (`PosType`, `TextPosType`, …)
// which are the integer codes JSON spec authors put in their files.

/// Map `<hp:tbl textWrap>` literals to [`crate::spec::TableSpec::pos`]
/// integer enum values.
///
/// OWPML `ASOTWT_*` (see `enumdef.h`):
/// `SQUARE=0`, `TOP_AND_BOTTOM=1`, `BEHIND_TEXT=2`, `IN_FRONT_OF_TEXT=3`.
/// Reference DVC `PosType`:
/// `WRAP_TOP_AND_BOTTOM=0`, `BRING_IN_FRONT_OF_TEXT=1`,
/// `SEND_BEHIND_TEXT=2`, `WRAP_SQUARE=3`.
///
/// The two vocabularies are re-ordered; this helper performs the
/// translation so spec integers stay stable and document-level enums
/// stay faithful to OWPML.
fn pos_type_from_str(s: &str) -> Option<u32> {
    match s {
        "TOP_AND_BOTTOM" => Some(0),
        "IN_FRONT_OF_TEXT" => Some(1),
        "BEHIND_TEXT" => Some(2),
        "SQUARE" => Some(3),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:tbl textFlow>` literals to [`crate::spec::TableSpec::textpos`].
/// `TextPosType`: `BOTH_SIDES=0`, `LEFT_ONLY=1`, `RIGHT_ONLY=2`, `LARGEST_ONLY=3`.
fn text_pos_from_str(s: &str) -> Option<u32> {
    match s {
        "BOTH_SIDES" => Some(0),
        "LEFT_ONLY" => Some(1),
        "RIGHT_ONLY" => Some(2),
        "LARGEST_ONLY" => Some(3),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:pos horzRelTo>` literals.
/// `HorizontalType`: `HPAPER=0, HPAGE=1, HCOLUMN=2, HPARA=3`.
fn horz_rel_to_from_str(s: &str) -> Option<u32> {
    match s {
        "PAPER" => Some(0),
        "PAGE" => Some(1),
        "COLUMN" => Some(2),
        "PARA" => Some(3),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:pos horzAlign>` literals.
/// `HorizontalDirection`: `HLEFT=0, HCENTER=1, HRIGHT=2, HINSIDE=3, HOUTSIDE=4`.
fn horz_align_from_str(s: &str) -> Option<u32> {
    match s {
        "LEFT" => Some(0),
        "CENTER" => Some(1),
        "RIGHT" => Some(2),
        "INSIDE" => Some(3),
        "OUTSIDE" => Some(4),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:pos vertRelTo>` literals.
/// `VerticalType`: `VPAPER=0, VPAGE=1, VCOLUMN=2` — OWPML also permits
/// `PARA`, which is not in the reference `VerticalType` enum. We map
/// the OWPML `PARA` literal to the same slot as `VCOLUMN` (2) so
/// HWPX-native documents do not erroneously fail the check; authors
/// who want a strict mapping can change it in a follow-up.
fn vert_rel_to_from_str(s: &str) -> Option<u32> {
    match s {
        "PAPER" => Some(0),
        "PAGE" => Some(1),
        "PARA" | "COLUMN" => Some(2),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:pos vertAlign>` literals.
/// `VerticalDirection`: `VTOP=0, VCENTER=1, VBOTTOM=2`.
fn vert_align_from_str(s: &str) -> Option<u32> {
    match s {
        "TOP" => Some(0),
        "CENTER" => Some(1),
        "BOTTOM" => Some(2),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:tbl numberingType>` literals.
/// `NumType`: `NUM_NONE=0, NUM_PICTURE=1, NUM_TABLE=2, NUM_FORMULA=3`.
fn num_type_from_str(s: &str) -> Option<u32> {
    match s {
        "NONE" => Some(0),
        "PICTURE" => Some(1),
        "TABLE" => Some(2),
        "EQUATION" => Some(3),
        "" => None,
        _ => None,
    }
}

/// Map `<hp:caption side>` literals to `CaptionPosType` integers.
///
/// OWPML only emits `LEFT`/`RIGHT`/`TOP`/`BOTTOM` on the `side`
/// attribute; the nine-way DVC enum (`LEFTTOP`, `RIGHTTOP`, …) is a
/// composite of caption side + alignment. This helper only handles
/// the four basic sides; composite positions are a follow-up once the
/// document-side `align` companion is exposed.
fn caption_pos_from_str(s: &str) -> Option<u32> {
    // CaptionPosType: LEFTTOP=0, TOP=1, RIGHTTOP=2, LEFT=3, NONE=4,
    //                 RIGHT=5, LEFTBOTTOM=6, BOTTOM=7, RIGHTBOTTOM=8
    match s {
        "LEFT" => Some(3),
        "RIGHT" => Some(5),
        "TOP" => Some(1),
        "BOTTOM" => Some(7),
        "" => None,
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_type_solid_maps_correctly() {
        assert_eq!(border_type_to_line_type(1), LineType::Solid);
        assert_eq!(border_type_to_line_type(0), LineType::None);
        assert_eq!(border_type_to_line_type(2), LineType::Dash);
    }

    #[test]
    fn color_u32_to_hex_black() {
        assert_eq!(color_u32_to_hex(0), "#000000");
    }

    #[test]
    fn color_u32_to_hex_red() {
        assert_eq!(color_u32_to_hex(0x00FF0000), "#FF0000");
    }

    #[test]
    fn color_u32_to_hex_masks_high_byte() {
        // High byte should be ignored
        assert_eq!(color_u32_to_hex(0xFF000000), "#000000");
    }

    // ── Enum-string mappers ────────────────────────────────────────────

    #[test]
    fn pos_type_maps_known_literals() {
        assert_eq!(pos_type_from_str("TOP_AND_BOTTOM"), Some(0));
        assert_eq!(pos_type_from_str("SQUARE"), Some(3));
        assert_eq!(pos_type_from_str("BEHIND_TEXT"), Some(2));
        assert_eq!(pos_type_from_str(""), None);
        assert_eq!(pos_type_from_str("INVALID"), None);
    }

    #[test]
    fn text_pos_maps_known_literals() {
        assert_eq!(text_pos_from_str("BOTH_SIDES"), Some(0));
        assert_eq!(text_pos_from_str("LEFT_ONLY"), Some(1));
        assert_eq!(text_pos_from_str("LARGEST_ONLY"), Some(3));
        assert_eq!(text_pos_from_str(""), None);
    }

    #[test]
    fn horz_rel_to_maps_known_literals() {
        assert_eq!(horz_rel_to_from_str("PAPER"), Some(0));
        assert_eq!(horz_rel_to_from_str("COLUMN"), Some(2));
        assert_eq!(horz_rel_to_from_str("PARA"), Some(3));
    }

    #[test]
    fn horz_align_maps_known_literals() {
        assert_eq!(horz_align_from_str("LEFT"), Some(0));
        assert_eq!(horz_align_from_str("CENTER"), Some(1));
        assert_eq!(horz_align_from_str("OUTSIDE"), Some(4));
    }

    #[test]
    fn num_type_maps_known_literals() {
        assert_eq!(num_type_from_str("TABLE"), Some(2));
        assert_eq!(num_type_from_str("EQUATION"), Some(3));
        assert_eq!(num_type_from_str(""), None);
    }

    #[test]
    fn caption_pos_maps_known_literals() {
        assert_eq!(caption_pos_from_str("TOP"), Some(1));
        assert_eq!(caption_pos_from_str("BOTTOM"), Some(7));
        assert_eq!(caption_pos_from_str(""), None);
    }

    // ── Error-code constant parity ─────────────────────────────────────

    #[test]
    fn standard_mode_error_codes_match_reference_jsonmodel() {
        // Sanity: verify the public error-code constants still equal the
        // JID_TABLE+n offsets documented in JsonModel.h.
        assert_eq!(TABLE_SIZE_WIDTH, 3001);
        assert_eq!(TABLE_SIZE_HEIGHT, 3002);
        assert_eq!(TABLE_SIZE_FIXED, 3003);
        assert_eq!(TABLE_TREAT_AS_CHAR, 3004);
        assert_eq!(TABLE_POS, 3005);
        assert_eq!(TABLE_TEXT_POS, 3006);
        assert_eq!(TABLE_HTYPE, 3007);
        assert_eq!(TABLE_HDIRECTION, 3008);
        assert_eq!(TABLE_HVALUE, 3009);
        assert_eq!(TABLE_VTYPE, 3010);
        assert_eq!(TABLE_VDIRECTION, 3011);
        assert_eq!(TABLE_VVALUE, 3012);
        assert_eq!(TABLE_SOFLOW_WITH_TEXT, 3013);
        assert_eq!(TABLE_SOALLOW_OVERLAP, 3014);
        assert_eq!(TABLE_SOHOLD_ANCHOR_OBJ, 3015);
        assert_eq!(TABLE_PARALLEL, 3016);
        assert_eq!(TABLE_ROTATION, 3017);
        assert_eq!(TABLE_GRADIENT_H, 3018);
        assert_eq!(TABLE_GRADIENT_V, 3019);
        assert_eq!(TABLE_NUM_VER_TYPE, 3020);
        assert_eq!(TABLE_OBJ_PROTECT, 3021);
        assert_eq!(TABLE_MARGIN_LEFT, 3022);
        assert_eq!(TABLE_MARGIN_RIGHT, 3023);
        assert_eq!(TABLE_MARGIN_TOP, 3024);
        assert_eq!(TABLE_MARGIN_BOTTOM, 3025);
        assert_eq!(TABLE_CAPTION_POSITION, 3026);
        assert_eq!(TABLE_CAPTION_SIZE, 3027);
        assert_eq!(TABLE_CAPTION_SPACING, 3028);
        assert_eq!(TABLE_CAPTION_SOCAP_FULL_SIZE, 3029);
        assert_eq!(TABLE_CAPTION_LINE_WRAP, 3030);
        assert_eq!(TABLE_BORDER_TYPE, 3033);
        assert_eq!(TABLE_BORDER_SIZE, 3034);
        assert_eq!(TABLE_BORDER_COLOR, 3035);
        assert_eq!(TABLE_BORDER_CELL_SPACING, 3036);
        assert_eq!(TABLE_IN_TABLE, 3056);
    }

    // ── Caption checks via synthetic Table AST ─────────────────────────
    //
    // No committed HWPX fixture carries an `<hp:caption>` element (the
    // public sample documents ship without captions). We therefore
    // exercise the caption path by assembling a `Table` AST directly —
    // which also catches the "has_caption gate" logic in `check_table`.

    fn caption_table(side: &str, size: u32, spacing: i32) -> Table {
        Table {
            id: 12345,
            has_caption: true,
            caption_side: side.to_string(),
            caption_size: size,
            caption_spacing: spacing,
            caption_full_size: false,
            caption_line_wrap: true,
            // The caption checker doesn't touch these, but they're
            // required to construct a valid Table.
            ..Table::default()
        }
    }

    fn empty_header() -> HeaderTables {
        HeaderTables::default()
    }

    #[test]
    fn caption_position_mismatch_generates_caption_position_error() {
        let table = caption_table("TOP", 100, 50);
        // side=TOP maps to 1; demand BOTTOM (=7).
        let spec = TableSpec {
            caption_position: Some(7),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(&table, &spec, &empty_header(), CheckLevel::All, &mut errors);
        assert!(errors
            .iter()
            .any(|e| e.error_code == TABLE_CAPTION_POSITION));
    }

    #[test]
    fn caption_size_and_spacing_out_of_range_generate_errors() {
        let table = caption_table("TOP", 100, 50);
        let spec = TableSpec {
            caption_size: Some(IntRange {
                min: 500,
                max: 1000,
            }),
            caption_spacing: Some(IntRange { min: 0, max: 10 }),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(&table, &spec, &empty_header(), CheckLevel::All, &mut errors);
        assert!(errors.iter().any(|e| e.error_code == TABLE_CAPTION_SIZE));
        assert!(errors.iter().any(|e| e.error_code == TABLE_CAPTION_SPACING));
    }

    #[test]
    fn caption_full_size_and_linewrap_mismatch_generate_errors() {
        let table = caption_table("TOP", 100, 50);
        // table has full_size=false, line_wrap=true
        let spec = TableSpec {
            caption_socapfullsize: Some(true),
            caption_linewrap: Some(false),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(&table, &spec, &empty_header(), CheckLevel::All, &mut errors);
        assert!(errors
            .iter()
            .any(|e| e.error_code == TABLE_CAPTION_SOCAP_FULL_SIZE));
        assert!(errors
            .iter()
            .any(|e| e.error_code == TABLE_CAPTION_LINE_WRAP));
    }

    #[test]
    fn no_caption_document_skips_all_caption_checks() {
        // Construct a table with has_caption=false. All caption spec
        // fields must be ignored.
        let table = Table {
            id: 1,
            has_caption: false,
            ..Table::default()
        };
        let spec = TableSpec {
            caption_position: Some(99),
            caption_size: Some(IntRange { min: 0, max: 0 }),
            caption_spacing: Some(IntRange { min: 0, max: 0 }),
            caption_socapfullsize: Some(true),
            caption_linewrap: Some(true),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(&table, &spec, &empty_header(), CheckLevel::All, &mut errors);
        assert!(
            errors.is_empty(),
            "caption checks must be a no-op without a caption; got {errors:?}"
        );
    }

    #[test]
    fn simple_level_stops_at_first_error() {
        // A synthetic table that triggers three range failures, in the
        // order the checker evaluates them (size_width before
        // size_height before fixed). With CheckLevel::Simple, only the
        // first error should be recorded.
        let table = Table {
            id: 1,
            width: 99999,
            height: 99999,
            size_protect: false,
            ..Table::default()
        };
        let spec = TableSpec {
            size_width: Some(IntRange { min: 0, max: 100 }),
            size_height: Some(IntRange { min: 0, max: 100 }),
            fixed: Some(true),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(
            &table,
            &spec,
            &empty_header(),
            CheckLevel::Simple,
            &mut errors,
        );
        assert_eq!(errors.len(), 1, "simple-level must bail after one error");
        assert_eq!(errors[0].error_code, TABLE_SIZE_WIDTH);
    }

    #[test]
    fn all_level_collects_every_violation() {
        let table = Table {
            id: 1,
            width: 99999,
            height: 99999,
            size_protect: false,
            ..Table::default()
        };
        let spec = TableSpec {
            size_width: Some(IntRange { min: 0, max: 100 }),
            size_height: Some(IntRange { min: 0, max: 100 }),
            fixed: Some(true),
            ..TableSpec::default()
        };
        let mut errors = Vec::new();
        check_table(&table, &spec, &empty_header(), CheckLevel::All, &mut errors);
        let codes: Vec<u32> = errors.iter().map(|e| e.error_code).collect();
        assert!(codes.contains(&TABLE_SIZE_WIDTH));
        assert!(codes.contains(&TABLE_SIZE_HEIGHT));
        assert!(codes.contains(&TABLE_SIZE_FIXED));
    }
}
