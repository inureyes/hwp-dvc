//! `CheckTable` — validates table borders, treat-as-char, and nested-table
//! policy against [`TableSpec`].
//!
//! Maps to `Checker::CheckTable` / `CheckTableToCheckList` /
//! `CheckFromBorderInfo` in `references/dvc/Checker.cpp`.
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
//! # Cell-level detail mode
//!
//! When `OutputScope.table_detail` is `true` the validator should also
//! inspect each cell's border fill against the spec. That mode is a
//! **TODO** — this pass implements table-summary mode only, which checks
//! the outer `borderFillIDRef` of each `<hp:tbl>` element.
//!
//! # treatAsChar
//!
//! The `treatAsChar` attribute lives in the `<hp:pos>` child of
//! `<hp:tbl>`. The current `Table` AST node (issue #3) does not expose
//! this field. Checking `treatAsChar` is therefore a **TODO** pending an
//! additive field on [`crate::document::section::types::Table`].

use crate::checker::{CheckLevel, DvcErrorInfo, OutputScope};
use crate::document::header::types::{Border, LineType};
use crate::document::section::types::Table;
use crate::document::{Document, HeaderTables};
use crate::error::{DvcResult, ErrorContext};
use crate::spec::BorderSpec;
use crate::spec::TableSpec;

// ---------------------------------------------------------------------------
// Error codes (mirrored in error.rs)
// ---------------------------------------------------------------------------

/// Border line-type mismatch (JID_TABLE_BORDER_TYPE).
pub const TABLE_BORDER_TYPE: u32 = 3033;
/// Border width mismatch (JID_TABLE_BORDER_SIZE).
pub const TABLE_BORDER_SIZE: u32 = 3034;
/// Border color mismatch (JID_TABLE_BORDER_COLOR).
pub const TABLE_BORDER_COLOR: u32 = 3035;
/// treat-as-char mismatch (JID_TABLE_TREATASCHAR).
pub const TABLE_TREAT_AS_CHAR: u32 = 3004;
/// Nested table where policy forbids it (JID_TABLE_TABLEINTABLE).
pub const TABLE_IN_TABLE: u32 = 3056;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all table checks for the given document against `spec`.
///
/// Returns a (possibly empty) vector of [`DvcErrorInfo`] records. The
/// `level` and `scope` parameters are forwarded from [`crate::checker::Checker`].
pub fn check(
    document: &Document,
    spec: &TableSpec,
    level: CheckLevel,
    scope: OutputScope,
) -> DvcResult<Vec<DvcErrorInfo>> {
    let _ = scope; // cell-detail mode is a TODO; see module-level doc
    let header = match &document.header {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let mut errors = Vec::new();

    for section in &document.sections {
        for table in section.all_tables() {
            check_table(table, spec, header, level, &mut errors);
            if level == CheckLevel::Simple && !errors.is_empty() {
                return Ok(errors);
            }
        }
    }

    Ok(errors)
}

// ---------------------------------------------------------------------------
// Per-table validation
// ---------------------------------------------------------------------------

fn check_table(
    table: &Table,
    spec: &TableSpec,
    header: &HeaderTables,
    level: CheckLevel,
    errors: &mut Vec<DvcErrorInfo>,
) {
    // --- 1. Border checks ---------------------------------------------------
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

    // --- 2. treatAsChar check (TODO) ----------------------------------------
    // The `Table` AST node does not yet carry `treat_as_char` (from
    // `<hp:pos treatAsChar=".."/>`). This check is deferred until an additive
    // field is added to `crate::document::section::types::Table`. Error code
    // TABLE_TREAT_AS_CHAR (3004) will be emitted once the field is available.
    let _ = TABLE_TREAT_AS_CHAR; // suppress dead-code lint until TODO is done

    // --- 3. Table-in-table check --------------------------------------------
    if spec.table_in_table == Some(false) && table.nesting_depth >= 1 {
        errors.push(DvcErrorInfo {
            error_code: TABLE_IN_TABLE,
            table_id: table.id,
            is_in_table: table.nesting_depth >= 1,
            is_in_table_in_table: table.nesting_depth >= 2,
            error_string: crate::error::error_string(TABLE_IN_TABLE, ErrorContext::default()),
            ..Default::default()
        });
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
}
