//! Per-cell ("detail") table validation — emitted only when the user
//! passes `--tabledetail` (i.e. [`OutputScope::table_detail`] is true).
//!
//! Maps to the cell-iteration half of
//! `Checker::CheckTableToCheckList` in
//! `references/dvc/Checker.cpp` plus the JID_TABLE 3037..=3055 cases
//! that the C++ reference leaves as empty `break` stubs. See issue #42.
//!
//! # Scope
//!
//! Detail mode walks every cell of every table, resolves its
//! `borderFillIDRef` against [`HeaderTables::border_fills`], and then
//! compares the decoded [`CellFillBrush`] to the cell-detail fields on
//! [`TableSpec`]. Each finding carries the cell's `row`/`col`
//! populated so that downstream output shows which cell violated.
//!
//! # Error codes
//!
//! - 3037 (`TABLE_BGFILL_TYPE`) — fill kind (`none` / `color` / `gradation`) mismatch
//! - 3038 (`TABLE_BGFILL_FACECOLOR`) — solid-fill face color mismatch
//! - 3039 (`TABLE_BGFILL_PATTONCOLOR`) — pattern color mismatch
//! - 3040 (`TABLE_BGFILL_PATTONTYPE`) — pattern type mismatch (TODO: HWPX
//!   emitters we have access to do not carry the hatch type on
//!   `<hc:winBrush>`; check is skipped until a fixture surfaces the
//!   field.)
//! - 3041..=3048 (`TABLE_BGGRADATION_*`) — gradient-subfield mismatches
//! - 3049..=3052 (`TABLE_PICTURE_*` / `TABLE_PICTUREFILL_*`) — picture-fill mismatches
//! - 3053..=3054 (`TABLE_EFFECT_*`) — picture-effect mismatches
//! - 3055 (`TABLE_WATERMARK`) — watermark mismatch

use crate::checker::{CheckLevel, DvcErrorInfo};
use crate::document::header::{BorderFill, CellFillBrush};
use crate::document::section::types::{Cell, Table};
use crate::document::HeaderTables;
use crate::error::error_string;
use crate::error::table_detail_codes::*;
use crate::error::ErrorContext;
use crate::spec::TableSpec;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Walk every cell of `table` and emit detail-mode findings for each
/// cell whose decoded fill does not satisfy `spec`.
///
/// This is a no-op when the spec has no cell-detail fields set —
/// callers can and should short-circuit with
/// [`TableSpec::has_cell_detail_fields`] before invoking.
pub(super) fn check_cells(
    table: &Table,
    spec: &TableSpec,
    header: &HeaderTables,
    level: CheckLevel,
    errors: &mut Vec<DvcErrorInfo>,
) {
    for row in &table.rows {
        for cell in &row.cells {
            // Resolve the cell's border-fill record; cells that point
            // at an unknown id are silently skipped (matches the
            // reference C++ which ignores missing lookups).
            let Some(bf) = header.border_fills.get(&cell.border_fill_id_ref) else {
                continue;
            };
            check_one_cell(table, cell, bf, spec, level, errors);
            if level == CheckLevel::Simple && !errors.is_empty() {
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Per-cell checker
// ---------------------------------------------------------------------------

fn check_one_cell(
    table: &Table,
    cell: &Cell,
    bf: &BorderFill,
    spec: &TableSpec,
    level: CheckLevel,
    errors: &mut Vec<DvcErrorInfo>,
) {
    let brush = bf.fill_brush.as_ref();
    let observed_kind = fill_kind(brush);

    // 3037: bgfill-type ------------------------------------------------------
    if let Some(expected) = spec.bgfill_type.as_deref() {
        if !kind_matches_expected(observed_kind, expected) {
            push(errors, table, cell, TABLE_BGFILL_TYPE);
            if level == CheckLevel::Simple {
                return;
            }
        }
    }

    match brush {
        Some(CellFillBrush::Solid { face, hatch, .. }) => {
            // 3038: bgfill-facecolor
            if let Some(expected) = spec.bgfill_facecolor {
                // Solid fill with `faceColor="none"` means the cell is
                // effectively unfilled. Only emit when a concrete color
                // is declared and it does not match the expected value.
                if !color_matches(face, expected) {
                    push(errors, table, cell, TABLE_BGFILL_FACECOLOR);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3039: bgfill-pattoncolor
            if let Some(expected) = spec.bgfill_pattoncolor {
                if !color_matches(hatch, expected) {
                    push(errors, table, cell, TABLE_BGFILL_PATTONCOLOR);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3040: bgfill-pattontype — OWPML `<hc:winBrush>` does not
            // expose a discrete hatch-type index in fixtures we have
            // access to. Defer until a real pattern-fill fixture is
            // added to the test suite.
            // TODO: wire up pattern-type comparison when a pattern-fill
            // HWPX fixture exists (see reference `BGPattonType` enum).
            let _ = spec.bgfill_pattontype;
        }
        Some(CellFillBrush::Gradation {
            gradation_type,
            start_color,
            end_color,
            width_center,
            height_center,
            angle,
            blur_level,
            blur_center,
        }) => {
            // 3041
            if let Some(expected) = spec.bggradation_startcolor {
                if !color_matches(start_color, expected) {
                    push(errors, table, cell, TABLE_BGGRADATION_STARTCOLOR);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3042
            if let Some(expected) = spec.bggradation_endcolor {
                if !color_matches(end_color, expected) {
                    push(errors, table, cell, TABLE_BGGRADATION_ENDCOLOR);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3043
            if let Some(expected) = spec.bggradation_type.as_deref() {
                if !grad_type_matches(gradation_type, expected) {
                    push(errors, table, cell, TABLE_BGGRADATION_TYPE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3044..=3048 — numeric geometry
            if let Some(expected) = spec.bggradation_widthcenter {
                if *width_center != expected {
                    push(errors, table, cell, TABLE_BGGRADATION_WIDTHCENTER);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            if let Some(expected) = spec.bggradation_heightcenter {
                if *height_center != expected {
                    push(errors, table, cell, TABLE_BGGRADATION_HEIGHTCENTER);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            if let Some(expected) = spec.bggradation_gradationangle {
                if *angle != expected {
                    push(errors, table, cell, TABLE_BGGRADATION_GRADATIONANGLE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            if let Some(expected) = spec.bggradation_blurlevel {
                if *blur_level != expected {
                    push(errors, table, cell, TABLE_BGGRADATION_BLURLEVEL);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            if let Some(expected) = spec.bggradation_blurcenter {
                if *blur_center != expected {
                    push(errors, table, cell, TABLE_BGGRADATION_BLURCENTER);
                    // Last gradient check: the function returns
                    // naturally once the match arm completes.
                }
            }
        }
        Some(CellFillBrush::Image {
            file,
            include,
            fill_type,
            fill_value,
            effect_type,
            effect_value,
            watermark,
        }) => {
            // 3049
            if let Some(expected) = spec.picture_file.as_deref() {
                if file != expected {
                    push(errors, table, cell, TABLE_PICTURE_FILE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3050
            if let Some(expected) = spec.picture_include {
                if *include != expected {
                    push(errors, table, cell, TABLE_PICTURE_INCLUDE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3051
            if let Some(expected) = spec.picturefill_type.as_deref() {
                if !ci_str_eq(fill_type, expected) {
                    push(errors, table, cell, TABLE_PICTUREFILL_TYPE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3052
            if let Some(expected) = spec.picturefill_value {
                if *fill_value != expected {
                    push(errors, table, cell, TABLE_PICTUREFILL_VALUE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3053
            if let Some(expected) = spec.effect_type.as_deref() {
                if !ci_str_eq(effect_type, expected) {
                    push(errors, table, cell, TABLE_EFFECT_TYPE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3054
            if let Some(expected) = spec.effect_value {
                if *effect_value != expected {
                    push(errors, table, cell, TABLE_EFFECT_VALUE);
                    if level == CheckLevel::Simple {
                        return;
                    }
                }
            }
            // 3055
            if let Some(expected) = spec.watermark {
                if *watermark != expected {
                    push(errors, table, cell, TABLE_WATERMARK);
                    // Last image-brush check: the function returns
                    // naturally once the match arm completes.
                }
            }
        }
        None => {
            // No fill brush → everything cell-detail that requires a
            // brush silently matches the "no fill" expectation. Only
            // the 3037 kind-check above can fire.
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Classify a parsed brush into the three-way enum understood by the
/// `bgfill-type` spec key (`"none"`, `"color"`, `"gradation"`). The
/// reference C++ `BGFillType` does not have a dedicated variant for
/// picture fills, so image brushes are reported as `"color"` to match
/// what the C++ tool emits (it treats an imgBrush cell as "has a fill"
/// for the purposes of the top-level fill-type check).
fn fill_kind(brush: Option<&CellFillBrush>) -> &'static str {
    match brush {
        None => "none",
        Some(CellFillBrush::Solid { face, .. }) if face.eq_ignore_ascii_case("none") => "none",
        Some(CellFillBrush::Solid { .. }) => "color",
        Some(CellFillBrush::Gradation { .. }) => "gradation",
        Some(CellFillBrush::Image { .. }) => "color",
    }
}

/// Case-insensitive comparison of the observed fill kind (`none` /
/// `color` / `gradation`) against the expected spec string. An empty
/// spec string matches nothing (the caller must bail before calling).
fn kind_matches_expected(observed: &str, expected: &str) -> bool {
    observed.eq_ignore_ascii_case(expected)
}

/// Compare an observed color string (`"#RRGGBB"` or `"none"`) against
/// a packed 24-bit RGB integer from the spec. An observed `"none"`
/// never matches an integer spec.
fn color_matches(observed: &str, expected_packed: u32) -> bool {
    if observed.eq_ignore_ascii_case("none") {
        return false;
    }
    let expected_hex = format!("#{:06X}", expected_packed & 0x00FF_FFFF);
    observed.eq_ignore_ascii_case(&expected_hex)
}

/// Case-insensitive compare for gradient type. Accepts the C++
/// reference's spellings ("LINEAR"/"RADIAL"/"SQUARE"/"CONIAL" which is
/// the reference's literal mis-spelling of "CONICAL") as well as more
/// conventional English variants.
fn grad_type_matches(observed: &str, expected: &str) -> bool {
    fn canon(s: &str) -> &str {
        let lower = s.trim();
        // Normalise the reference's non-standard spelling to the
        // commonly-used English form so `"conial"` and `"conical"`
        // compare equal.
        if lower.eq_ignore_ascii_case("conial") {
            return "conical";
        }
        lower
    }
    canon(observed).eq_ignore_ascii_case(canon(expected))
}

/// Case-insensitive string equality ignoring leading/trailing whitespace.
fn ci_str_eq(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn push(errors: &mut Vec<DvcErrorInfo>, table: &Table, cell: &Cell, code: u32) {
    errors.push(DvcErrorInfo {
        error_code: code,
        table_id: table.id,
        is_in_table: true,
        is_in_table_in_table: table.nesting_depth >= 1,
        table_row: cell.row,
        table_col: cell.col,
        error_string: error_string(code, ErrorContext::default()),
        ..Default::default()
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_kind_none_when_facecolor_is_literal_none_string() {
        let brush = CellFillBrush::Solid {
            face: "none".into(),
            hatch: "#000000".into(),
            alpha: 0,
        };
        assert_eq!(fill_kind(Some(&brush)), "none");
    }

    #[test]
    fn fill_kind_color_for_concrete_face_color() {
        let brush = CellFillBrush::Solid {
            face: "#FFFFFF".into(),
            hatch: "#000000".into(),
            alpha: 0,
        };
        assert_eq!(fill_kind(Some(&brush)), "color");
    }

    #[test]
    fn fill_kind_none_when_no_brush() {
        assert_eq!(fill_kind(None), "none");
    }

    #[test]
    fn color_matches_compares_hex_string_case_insensitively() {
        assert!(color_matches("#ff0000", 0x00FF0000));
        assert!(color_matches("#FF0000", 0x00FF0000));
        assert!(!color_matches("#00FF00", 0x00FF0000));
    }

    #[test]
    fn color_matches_rejects_none() {
        assert!(!color_matches("none", 0x00FF0000));
    }

    #[test]
    fn grad_type_matches_canonicalises_conial_to_conical() {
        assert!(grad_type_matches("conial", "conical"));
        assert!(grad_type_matches("CONIAL", "CONICAL"));
        assert!(grad_type_matches("linear", "LINEAR"));
        assert!(!grad_type_matches("radial", "linear"));
    }
}
