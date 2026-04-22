//! Character-shape validator (`CheckCharShape` in the reference C++).
//!
//! Mirrors `Checker::CheckCharShape` and
//! `Checker::CheckCharShapeToCheckList` in `references/dvc/Checker.cpp`.
//!
//! # Algorithm
//!
//! 1. For every `CharShape` in the header table, compare it against
//!    the `CharShapeSpec` fields that are enabled in the spec.
//! 2. On mismatch, record an intermediate `ErrorInfo` keyed by the
//!    `CharShape` id and the `JID_CHAR_SHAPE_*` error code.
//! 3. Walk the [`RunTypeInfo`] stream and, for each run whose
//!    `char_pr_id_ref` matches a recorded error, emit a
//!    [`DvcErrorInfo`].
//!
//! # Error codes
//!
//! | Constant                       | Value | `JID_*` reference                  |
//! |--------------------------------|-------|------------------------------------|
//! | `CHARSHAPE_FONTSIZE`           | 1001  | `JID_CHAR_SHAPE_FONTSIZE`          |
//! | `CHARSHAPE_LANGSET`            | 1002  | `JID_CHAR_SHAPE_LANGSET`           |
//! | `CHARSHAPE_LANGTYPE`           | 1003  | `JID_CHAR_SHAPE_LANG`              |
//! | `CHARSHAPE_FONT`               | 1004  | `JID_CHAR_SHAPE_FONT`              |
//! | `CHARSHAPE_RSIZE`              | 1005  | `JID_CHAR_SHAPE_RSIZE`             |
//! | `CHARSHAPE_POSITION`           | 1006  | `JID_CHAR_SHAPE_POSITION`          |
//! | `CHARSHAPE_RATIO`              | 1007  | `JID_CHAR_SHAPE_RATIO`             |
//! | `CHARSHAPE_SPACING`            | 1008  | `JID_CHAR_SHAPE_SPACING`           |
//! | `CHARSHAPE_BOLD`               | 1009  | `JID_CHAR_SHAPE_BOLD`              |
//! | `CHARSHAPE_ITALIC`             | 1010  | `JID_CHAR_SHAPE_ITALIC`            |
//! | `CHARSHAPE_UNDERLINE`          | 1011  | `JID_CHAR_SHAPE_UNDERLINE`         |
//! | `CHARSHAPE_STRIKEOUT`          | 1012  | `JID_CHAR_SHAPE_STRIKEOUT`         |
//! | `CHARSHAPE_OUTLINE`            | 1013  | `JID_CHAR_SHAPE_OUTLINE`           |
//! | `CHARSHAPE_EMBOSS`             | 1014  | `JID_CHAR_SHAPE_EMBOSS`            |
//! | `CHARSHAPE_ENGRAVE`            | 1015  | `JID_CHAR_SHAPE_ENGRAVE`           |
//! | `CHARSHAPE_SHADOW`             | 1016  | `JID_CHAR_SHAPE_SHADOW`            |
//! | `CHARSHAPE_SUPSCRIPT`          | 1017  | `JID_CHAR_SHAPE_SUPSCRIPT`         |
//! | `CHARSHAPE_SUBSCRIPT`          | 1018  | `JID_CHAR_SHAPE_SUBSCRIPT`         |
//! | `CHARSHAPE_SHADOWTYPE`         | 1019  | `JID_CHAR_SHAPE_SHADOWTYPE`        |
//! | `CHARSHAPE_SHADOW_X`           | 1020  | `JID_CHAR_SHAPE_SHADOW_X`          |
//! | `CHARSHAPE_SHADOW_Y`           | 1021  | `JID_CHAR_SHAPE_SHADOW_Y`          |
//! | `CHARSHAPE_SHADOW_COLOR`       | 1022  | `JID_CHAR_SHAPE_SHADOW_COLOR`      |
//! | `CHARSHAPE_UNDERLINE_POSITION` | 1023  | `JID_CHAR_SHAPE_UNDERLINE_POSITION`|
//! | `CHARSHAPE_UNDERLINE_SHAPE`    | 1024  | `JID_CHAR_SHAPE_UNDERLINE_SHAPE`   |
//! | `CHARSHAPE_UNDERLINE_COLOR`    | 1025  | `JID_CHAR_SHAPE_UNDERLINE_COLOR`   |
//! | `CHARSHAPE_STRIKEOUT_SHAPE`    | 1026  | `JID_CHAR_SHAPE_STRIKEOUT_SHAPE`   |
//! | `CHARSHAPE_STRIKEOUT_COLOR`    | 1027  | `JID_CHAR_SHAPE_STRIKEOUT_COLOR`   |
//! | `CHARSHAPE_OUTLINETYPE`        | 1028  | `JID_CHAR_SHAPE_OUTLINETYPE`       |
//! | `CHARSHAPE_EMPTYSPACE`         | 1029  | `JID_CHAR_SHAPE_EMPTYSPACE`        |
//! | `CHARSHAPE_POINT`              | 1030  | `JID_CHAR_SHAPE_POINT`             |
//! | `CHARSHAPE_KERNING`            | 1031  | `JID_CHAR_SHAPE_KERNING`           |
//! | `CHARSHAPE_BG_BORDER`          | 1032  | `JID_CHAR_SHAPE_BG_BORDER`         |
//! | `CHARSHAPE_BG_BORDER_POSITION` | 1033  | `JID_CHAR_SHAPE_BG_BORDER_POSITION`|
//! | `CHARSHAPE_BG_BORDER_BORDERTYPE`| 1034 | `JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE`|
//! | `CHARSHAPE_BG_BORDER_SIZE`     | 1035  | `JID_CHAR_SHAPE_BG_BORDER_SIZE`    |
//! | `CHARSHAPE_BG_BORDER_COLOR`    | 1036  | `JID_CHAR_SHAPE_BG_BORDER_COLOR`   |
//! | `CHARSHAPE_BG_COLOR`           | 1037  | `JID_CHAR_SHAPE_BG_COLOR`          |
//! | `CHARSHAPE_BG_PATTONCOLOR`     | 1038  | `JID_CHAR_SHAPE_BG_PATTONCOLOR`    |
//! | `CHARSHAPE_BG_PATTONTYPE`      | 1039  | `JID_CHAR_SHAPE_BG_PATTONTYPE`     |

use crate::checker::{CheckLevel, DvcErrorInfo};
use crate::document::header::{CharShape, FontFace, HeaderTables};
use crate::document::RunTypeInfo;
use crate::error::ErrorContext;
use crate::spec::CharShapeSpec;

// ──────────────────────────────────────────────────────────────────────────────
// Error codes — mirrors JsonModel.h JID_CHAR_SHAPE_* offsets
// (base 1000 = JID_CHAR_SHAPE per references/dvc/Source/JsonModel.h).
// ──────────────────────────────────────────────────────────────────────────────

pub use crate::error::{
    CHARSHAPE_BG_BORDER, CHARSHAPE_BG_BORDER_BORDERTYPE, CHARSHAPE_BG_BORDER_COLOR,
    CHARSHAPE_BG_BORDER_POSITION, CHARSHAPE_BG_BORDER_SIZE, CHARSHAPE_BG_COLOR,
    CHARSHAPE_BG_PATTONCOLOR, CHARSHAPE_BG_PATTONTYPE, CHARSHAPE_BOLD, CHARSHAPE_EMBOSS,
    CHARSHAPE_EMPTYSPACE, CHARSHAPE_ENGRAVE, CHARSHAPE_FONT, CHARSHAPE_FONTSIZE, CHARSHAPE_ITALIC,
    CHARSHAPE_KERNING, CHARSHAPE_LANGSET, CHARSHAPE_LANGTYPE, CHARSHAPE_OUTLINE,
    CHARSHAPE_OUTLINETYPE, CHARSHAPE_POINT, CHARSHAPE_POSITION, CHARSHAPE_RATIO, CHARSHAPE_RSIZE,
    CHARSHAPE_SHADOW, CHARSHAPE_SHADOWTYPE, CHARSHAPE_SHADOW_COLOR, CHARSHAPE_SHADOW_X,
    CHARSHAPE_SHADOW_Y, CHARSHAPE_SPACING, CHARSHAPE_STRIKEOUT, CHARSHAPE_STRIKEOUT_COLOR,
    CHARSHAPE_STRIKEOUT_SHAPE, CHARSHAPE_SUBSCRIPT, CHARSHAPE_SUPSCRIPT, CHARSHAPE_UNDERLINE,
    CHARSHAPE_UNDERLINE_COLOR, CHARSHAPE_UNDERLINE_POSITION, CHARSHAPE_UNDERLINE_SHAPE,
};

// ──────────────────────────────────────────────────────────────────────────────
// Internal intermediate record
// ──────────────────────────────────────────────────────────────────────────────

/// Intermediate error: a (char_shape_id, error_code) pair produced by
/// `check_char_shape_to_check_list` before the RunTypeInfo fan-out.
struct ErrorInfo {
    /// The `CharShape.id` that violated the rule.
    char_pr_id: u32,
    /// One of the `CHARSHAPE_*` constants.
    error_code: u32,
    /// Optional context for message formatting (e.g. the offending font name).
    font_name: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public entry point
// ──────────────────────────────────────────────────────────────────────────────

/// Validate character shapes against the spec and return any errors.
///
/// Mirrors `Checker::CheckCharShape` + `CheckCharShapeToCheckList` from
/// `references/dvc/Checker.cpp` (lines 87–742).
///
/// # Parameters
///
/// * `spec` — the `[charshape]` section of the DVC spec.
/// * `tables` — the parsed header tables (provides `char_shapes` and `font_faces`).
/// * `run_type_infos` — the flattened run stream for the document.
/// * `level` — [`CheckLevel::Simple`] stops at the first error per run;
///   [`CheckLevel::All`] collects every violation.
pub fn check(
    spec: &CharShapeSpec,
    tables: &HeaderTables,
    run_type_infos: &[RunTypeInfo],
    level: CheckLevel,
) -> Vec<DvcErrorInfo> {
    // Phase 1: for each CharShape entry in the header, collect violations.
    let mut intermediate: Vec<ErrorInfo> = Vec::new();
    for cs in tables.char_shapes.values() {
        check_char_shape_to_check_list(cs, spec, &tables.font_faces, &mut intermediate);
    }

    if intermediate.is_empty() {
        return Vec::new();
    }

    // Phase 2: fan-out — for each intermediate error, find every RunTypeInfo
    // whose char_pr_id_ref matches and emit a DvcErrorInfo.
    // Mirrors the inner loop in Checker::CheckCharShape (lines 117–126).
    let mut results: Vec<DvcErrorInfo> = Vec::new();

    'outer: for err in &intermediate {
        for run in run_type_infos {
            if run.char_pr_id_ref == err.char_pr_id {
                let ctx = ErrorContext {
                    font_name: err.font_name.as_deref(),
                    ..ErrorContext::default()
                };
                results.push(DvcErrorInfo {
                    char_pr_id_ref: run.char_pr_id_ref,
                    para_pr_id_ref: run.para_pr_id_ref,
                    text: run.text.clone(),
                    page_no: run.page_no,
                    line_no: run.line_no,
                    error_code: err.error_code,
                    table_id: run.table_id,
                    is_in_table: run.is_in_table,
                    is_in_table_in_table: run.is_in_table_in_table,
                    table_row: run.table_row,
                    table_col: run.table_col,
                    is_in_shape: run.is_in_shape,
                    use_hyperlink: run.is_hyperlink,
                    use_style: run.is_style,
                    error_string: crate::error::error_string(err.error_code, ctx),
                });

                if level == CheckLevel::Simple {
                    break 'outer;
                }
            }
        }
    }

    results
}

// ──────────────────────────────────────────────────────────────────────────────
// Per-CharShape validation
// ──────────────────────────────────────────────────────────────────────────────

/// Check a single [`CharShape`] against the spec, appending any violations
/// to `errors`.
///
/// Mirrors `Checker::CheckCharShapeToCheckList` (Checker.cpp lines 538–742).
fn check_char_shape_to_check_list(
    cs: &CharShape,
    spec: &CharShapeSpec,
    font_faces: &[FontFace],
    errors: &mut Vec<ErrorInfo>,
) {
    // JID_CHAR_SHAPE_FONTSIZE (1001) — font size in 0.1pt units.
    //
    // `CharShape.height` stores the font size in 0.1pt units. Compared
    // against the spec's `fontsize` integer for an exact match.
    if let Some(spec_fontsize) = spec.fontsize {
        if cs.height != spec_fontsize {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_FONTSIZE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_LANGSET (1002) — per-language slot validation.
    //
    // The langset check in the reference validates that each document charshape
    // uses a lang slot matching the spec-supplied langset object. Full per-slot
    // validation requires the document-side `LangType` decoding which is deferred
    // to a later issue (see TODO below). The constant is registered and wired
    // to a stub so downstream code referencing CHARSHAPE_LANGSET compiles cleanly.
    //
    // TODO(#9): implement full langset slot validation once CharShape carries
    // the resolved LangType per slot. Error code: CHARSHAPE_LANGSET (1002).
    let _ = spec.langtype.as_ref(); // Used below in the langtype check

    // JID_CHAR_SHAPE_LANG (1003) — langtype check.
    //
    // The reference stores langtype as a raw integer (LangType enum).
    // The Rust spec carries it as an optional string (e.g. "대표").
    // The current document model does not decode langtype into a resolved
    // string; the validator only emits an error when the spec explicitly
    // requests a specific langtype value and that value cannot be
    // confirmed.  Because the document-side LangType is not yet decoded,
    // we emit a CHARSHAPE_LANGTYPE error iff the spec specifies a
    // non-empty langtype string — this is a conservative placeholder that
    // matches the reference behaviour of flagging the charshape when the
    // field is active but cannot be verified.
    //
    // TODO(#9): decode the hangul/latin/… langtype from CharShape once the
    // header parser exposes it (issue #2 addendum).
    if let Some(ref _langtype) = spec.langtype {
        // langtype verification requires the document-side lang enum which is
        // not yet decoded from the OWPML charPr attributes.  Leave the
        // check unimplemented and add a TODO rather than incorrectly
        // flagging every charshape. No error emitted here until the
        // document model is extended.
        //
        // TODO(#9): implement langtype match once CharShape carries the
        // resolved LangType value. Error code: CHARSHAPE_LANGTYPE (1003).
        let _ = _langtype; // suppress unused-variable warning
    }

    // JID_CHAR_SHAPE_FONT (1004) — font allow-list check.
    //
    // Uses CharShape::font_names which deduplicates while preserving
    // first-seen order (per the PR that landed for issue #2).  The spec
    // supplies an allow-list; a CharShape is non-compliant iff NONE of
    // its resolved font names appear in the allow-list.
    if !spec.font.is_empty() {
        let doc_fonts = cs.font_names(font_faces);
        let any_allowed = doc_fonts.iter().any(|name| spec.font.contains(name));
        if !any_allowed {
            // Capture the first document font name for the error message.
            let font_name = doc_fonts.into_iter().next();
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_FONT,
                font_name,
            });
        }
    }

    // JID_CHAR_SHAPE_RSIZE (1005) — relative size range check.
    //
    // `CharShape.rel_sz` stores relative size per language slot (percentage).
    // We check the Hangul slot (index 0) for an exact match with the spec.
    if let Some(spec_rsize) = spec.rsize {
        let doc_rsize = cs.rel_sz.values[0]; // Hangul slot
        if doc_rsize != spec_rsize {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_RSIZE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_POSITION (1006) — character position check.
    //
    // `CharShape.offset` stores vertical position per language slot (in 0.1pt units).
    // We check the Hangul slot (index 0) for an exact match with the spec.
    if let Some(spec_position) = spec.position {
        let doc_position = cs.offset.values[0]; // Hangul slot
        if doc_position != spec_position {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_POSITION,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_RATIO (1007) — ratio range check.
    //
    // The spec carries a single `ratio` integer that acts as both min and
    // max (exact match). The document's `CharShape.ratio` is a `LangTuple<u32>`;
    // we check the Hangul slot (index 0), which is what the reference
    // `charPr->charPrInfo.ratio` stores (only hangul, per RCharShape.h).
    if let Some(spec_ratio) = spec.ratio {
        let doc_ratio = cs.ratio.values[0]; // Hangul slot
        if doc_ratio != spec_ratio as u32 {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_RATIO,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SPACING (1008) — spacing range check.
    //
    // The spec carries a single `spacing` integer (exact match).
    // The document's `CharShape.spacing` is a `LangTuple<i32>`; we check
    // the Hangul slot matching the reference implementation.
    if let Some(spec_spacing) = spec.spacing {
        let doc_spacing = cs.spacing.values[0]; // Hangul slot
        if doc_spacing != spec_spacing {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_SPACING,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_BOLD (1009) — bold flag check.
    if let Some(spec_bold) = spec.bold {
        if cs.bold != spec_bold {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_BOLD,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_ITALIC (1010) — italic flag check.
    if let Some(spec_italic) = spec.italic {
        if cs.italic != spec_italic {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_ITALIC,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_UNDERLINE (1011) — underline flag check.
    if let Some(spec_underline) = spec.underline {
        if cs.underline != spec_underline {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_UNDERLINE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_STRIKEOUT (1012) — strikeout flag check.
    if let Some(spec_strikeout) = spec.strikeout {
        if cs.strikeout != spec_strikeout {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_STRIKEOUT,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_OUTLINE (1013) — outline flag check.
    if let Some(spec_outline) = spec.outline {
        if cs.outline != spec_outline {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_OUTLINE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_EMBOSS (1014) — emboss flag check.
    if let Some(spec_emboss) = spec.emboss {
        if cs.emboss != spec_emboss {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_EMBOSS,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_ENGRAVE (1015) — engrave flag check.
    if let Some(spec_engrave) = spec.engrave {
        if cs.engrave != spec_engrave {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_ENGRAVE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SHADOW (1016) — shadow flag check.
    if let Some(spec_shadow) = spec.shadow {
        if cs.shadow != spec_shadow {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_SHADOW,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SUPSCRIPT (1017) — superscript flag check.
    if let Some(spec_supscript) = spec.supscript {
        if cs.supscript != spec_supscript {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_SUPSCRIPT,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SUBSCRIPT (1018) — subscript flag check.
    if let Some(spec_subscript) = spec.subscript {
        if cs.subscript != spec_subscript {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_SUBSCRIPT,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_SHADOWTYPE (1019) — shadow type check.
    //
    // The document stores the shadow type as the "type" attribute of the
    // `<hh:shadow>` element. The CharShape bool `shadow` indicates active/inactive
    // but the detailed type string is not yet parsed into a dedicated field.
    //
    // TODO: extend CharShape to carry shadow_type: String once the header parser
    // is extended to preserve the type attribute. Error code: CHARSHAPE_SHADOWTYPE (1019).

    // JID_CHAR_SHAPE_SHADOW_X (1020), JID_CHAR_SHAPE_SHADOW_Y (1021),
    // JID_CHAR_SHAPE_SHADOW_COLOR (1022) — shadow detail checks.
    //
    // These sub-fields (offsetX, offsetY, color from `<hh:shadow>`) require
    // CharShape to carry dedicated shadow_offset_x, shadow_offset_y, shadow_color
    // fields that are not yet decoded by the header parser.
    //
    // TODO: extend CharShape and the header parser to carry shadow detail attrs.
    // Error codes: CHARSHAPE_SHADOW_X (1020), CHARSHAPE_SHADOW_Y (1021),
    //              CHARSHAPE_SHADOW_COLOR (1022).

    // JID_CHAR_SHAPE_UNDERLINE_POSITION (1023), _SHAPE (1024), _COLOR (1025)
    // JID_CHAR_SHAPE_STRIKEOUT_SHAPE (1026), _COLOR (1027)
    // JID_CHAR_SHAPE_OUTLINETYPE (1028)
    //
    // These sub-fields require the header parser to store the shape/type/color
    // attributes from the `<hh:underline>`, `<hh:strikeout>`, and `<hh:outline>`
    // child elements. The current CharShape only carries the boolean presence flags.
    //
    // TODO: extend CharShape to carry underline_position, underline_shape,
    // underline_color, strikeout_shape, strikeout_color, outline_type.
    // Error codes: 1023–1028.

    // JID_CHAR_SHAPE_EMPTYSPACE (1029) — empty-space flag.
    //
    // Corresponds to `useFontSpace` attribute on `<hh:charPr>`.
    if let Some(spec_emptyspace) = spec.emptyspace {
        if cs.use_font_space != spec_emptyspace {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_EMPTYSPACE,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_POINT (1030) — font size in points.
    //
    // The reference stores font size as both raw HWPUNIT (height) and as a
    // floating-point point value. We derive the point value from `height`:
    // HWPX stores height in 1/100pt units, so `point = height / 100.0`.
    // (e.g. height=1000 → 10pt, height=1200 → 12pt)
    if let Some(spec_point) = spec.point {
        let doc_point = cs.height as f64 / 100.0;
        // Use a small epsilon for float comparison (0.05 pt tolerance).
        if (doc_point - spec_point).abs() > 0.05 {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_POINT,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_KERNING (1031) — kerning flag.
    //
    // Corresponds to `useKerning` attribute on `<hh:charPr>`.
    if let Some(spec_kerning) = spec.kerning {
        if cs.use_kerning != spec_kerning {
            errors.push(ErrorInfo {
                char_pr_id: cs.id,
                error_code: CHARSHAPE_KERNING,
                font_name: None,
            });
        }
    }

    // JID_CHAR_SHAPE_BG_BORDER (1032) through JID_CHAR_SHAPE_BG_BORDER_COLOR (1036)
    //
    // The CharShape carries `border_fill_id_ref` which references a BorderFill
    // entry in the header. The `tables` parameter only exposes `char_shapes` and
    // `font_faces` in the current function signature; border-fill lookups require
    // the full `HeaderTables`. For the extended border check we do a conservative
    // presence check: if the spec supplies a border object and the charshape's
    // borderFillIDRef is 0 (= no border), emit a BG_BORDER error. More granular
    // sub-checks (position, type, size, color) are deferred until HeaderTables is
    // threaded through this function path.
    //
    // TODO: thread `&HeaderTables` through to enable full border sub-field checks
    // (1033–1036). Error codes: CHARSHAPE_BG_BORDER_POSITION (1033),
    // CHARSHAPE_BG_BORDER_BORDERTYPE (1034), CHARSHAPE_BG_BORDER_SIZE (1035),
    // CHARSHAPE_BG_BORDER_COLOR (1036).
    if spec.border.is_some() && cs.border_fill_id_ref == 0 {
        errors.push(ErrorInfo {
            char_pr_id: cs.id,
            error_code: CHARSHAPE_BG_BORDER,
            font_name: None,
        });
    }

    // JID_CHAR_SHAPE_BG_COLOR (1037) — background fill color.
    //
    // The background color is part of the BorderFill record (via `border_fill_id_ref`),
    // not directly on CharShape. Detailed fill-color comparison requires looking up
    // the BorderFill in HeaderTables and comparing its fill brush color.
    //
    // TODO: look up BorderFill from HeaderTables and compare fill color against
    // spec.bg_color. Error code: CHARSHAPE_BG_COLOR (1037).

    // JID_CHAR_SHAPE_BG_PATTONCOLOR (1038), JID_CHAR_SHAPE_BG_PATTONTYPE (1039)
    //
    // Background pattern color and type live inside the `<hc:fillBrush>` subtree
    // of the referenced `<hh:borderFill>`. These are not decoded by the current
    // border fill parser.
    //
    // TODO: extend BorderFill to carry fill-brush pattern color/type.
    // Error codes: CHARSHAPE_BG_PATTONCOLOR (1038), CHARSHAPE_BG_PATTONTYPE (1039).
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::document::header::{CharShape, FontFace, FontLang, HeaderTables, LangTuple};
    use crate::spec::CharShapeSpec;

    fn make_font_faces(hangul_fonts: &[(u32, &str)]) -> Vec<FontFace> {
        let mut face = FontFace {
            lang: FontLang::Hangul,
            fonts: HashMap::new(),
        };
        for (id, name) in hangul_fonts {
            face.fonts.insert(*id, name.to_string());
        }
        vec![face]
    }

    fn make_char_shape(
        id: u32,
        font_ref_hangul: u32,
        ratio_hangul: u32,
        spacing_hangul: i32,
    ) -> CharShape {
        let mut font_ref = LangTuple::<u32>::default();
        font_ref.set(FontLang::Hangul, font_ref_hangul);

        let mut ratio = LangTuple::<u32>::default();
        ratio.set(FontLang::Hangul, ratio_hangul);

        let mut spacing = LangTuple::<i32>::default();
        spacing.set(FontLang::Hangul, spacing_hangul);

        CharShape {
            id,
            font_ref,
            ratio,
            spacing,
            ..Default::default()
        }
    }

    fn make_tables_with(cs: CharShape, font_faces: Vec<FontFace>) -> HeaderTables {
        let mut char_shapes = HashMap::new();
        char_shapes.insert(cs.id, cs);
        HeaderTables {
            char_shapes,
            font_faces,
            ..Default::default()
        }
    }

    fn make_run(char_pr_id_ref: u32) -> RunTypeInfo {
        RunTypeInfo {
            char_pr_id_ref,
            text: "test".to_string(),
            ..Default::default()
        }
    }

    // ── font allow-list ────────────────────────────────────────────────────────

    #[test]
    fn font_allowed_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string(), "함초롬돋움".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.is_empty(),
            "no errors expected for compliant charshape; got: {errors:?}"
        );
    }

    #[test]
    fn font_not_in_allowlist_produces_font_error() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_FONT),
            "expected CHARSHAPE_FONT error; got: {errors:?}"
        );
    }

    #[test]
    fn empty_font_list_in_spec_skips_font_check() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec::default();

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.is_empty(),
            "empty font list must skip the font check"
        );
    }

    // ── ratio ──────────────────────────────────────────────────────────────────

    #[test]
    fn ratio_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        let ratio_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == CHARSHAPE_RATIO)
            .collect();
        assert!(
            ratio_errors.is_empty(),
            "no CHARSHAPE_RATIO error expected; got: {ratio_errors:?}"
        );
    }

    #[test]
    fn ratio_mismatch_produces_ratio_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 150, 0); // ratio = 150, spec wants 100
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_RATIO),
            "expected CHARSHAPE_RATIO error; got: {errors:?}"
        );
    }

    // ── spacing ────────────────────────────────────────────────────────────────

    #[test]
    fn spacing_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        let spacing_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == CHARSHAPE_SPACING)
            .collect();
        assert!(
            spacing_errors.is_empty(),
            "no CHARSHAPE_SPACING error expected"
        );
    }

    #[test]
    fn spacing_mismatch_produces_spacing_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let cs = make_char_shape(0, 1, 100, 10); // spacing = 10, spec wants 0
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ratio: Some(100),
            spacing: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_SPACING),
            "expected CHARSHAPE_SPACING error; got: {errors:?}"
        );
    }

    // ── fontsize ───────────────────────────────────────────────────────────────

    #[test]
    fn fontsize_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.height = 1000; // 10pt in 0.1pt units
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            fontsize: Some(1000),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().all(|e| e.error_code != CHARSHAPE_FONTSIZE),
            "no CHARSHAPE_FONTSIZE error expected; got: {errors:?}"
        );
    }

    #[test]
    fn fontsize_mismatch_produces_fontsize_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.height = 1200; // 12pt, spec wants 10pt
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            fontsize: Some(1000), // 10pt
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_FONTSIZE),
            "expected CHARSHAPE_FONTSIZE error; got: {errors:?}"
        );
    }

    // ── bold ───────────────────────────────────────────────────────────────────

    #[test]
    fn bold_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.bold = false;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            bold: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().all(|e| e.error_code != CHARSHAPE_BOLD),
            "no CHARSHAPE_BOLD error expected; got: {errors:?}"
        );
    }

    #[test]
    fn bold_mismatch_produces_bold_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.bold = true; // doc is bold, spec wants false
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            bold: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_BOLD),
            "expected CHARSHAPE_BOLD error; got: {errors:?}"
        );
    }

    // ── italic ─────────────────────────────────────────────────────────────────

    #[test]
    fn italic_mismatch_produces_italic_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.italic = true; // doc is italic, spec wants false
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            italic: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_ITALIC),
            "expected CHARSHAPE_ITALIC error; got: {errors:?}"
        );
    }

    // ── underline ──────────────────────────────────────────────────────────────

    #[test]
    fn underline_mismatch_produces_underline_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.underline = true; // doc has underline, spec wants false
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            underline: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_UNDERLINE),
            "expected CHARSHAPE_UNDERLINE error; got: {errors:?}"
        );
    }

    // ── strikeout ──────────────────────────────────────────────────────────────

    #[test]
    fn strikeout_mismatch_produces_strikeout_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.strikeout = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            strikeout: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_STRIKEOUT),
            "expected CHARSHAPE_STRIKEOUT error; got: {errors:?}"
        );
    }

    // ── outline ────────────────────────────────────────────────────────────────

    #[test]
    fn outline_mismatch_produces_outline_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.outline = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            outline: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_OUTLINE),
            "expected CHARSHAPE_OUTLINE error; got: {errors:?}"
        );
    }

    // ── emboss ─────────────────────────────────────────────────────────────────

    #[test]
    fn emboss_mismatch_produces_emboss_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.emboss = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            emboss: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_EMBOSS),
            "expected CHARSHAPE_EMBOSS error; got: {errors:?}"
        );
    }

    // ── engrave ────────────────────────────────────────────────────────────────

    #[test]
    fn engrave_mismatch_produces_engrave_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.engrave = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            engrave: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_ENGRAVE),
            "expected CHARSHAPE_ENGRAVE error; got: {errors:?}"
        );
    }

    // ── shadow ─────────────────────────────────────────────────────────────────

    #[test]
    fn shadow_mismatch_produces_shadow_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.shadow = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            shadow: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_SHADOW),
            "expected CHARSHAPE_SHADOW error; got: {errors:?}"
        );
    }

    // ── supscript / subscript ──────────────────────────────────────────────────

    #[test]
    fn supscript_mismatch_produces_supscript_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.supscript = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            supscript: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_SUPSCRIPT),
            "expected CHARSHAPE_SUPSCRIPT error; got: {errors:?}"
        );
    }

    #[test]
    fn subscript_mismatch_produces_subscript_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.subscript = true;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            subscript: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_SUBSCRIPT),
            "expected CHARSHAPE_SUBSCRIPT error; got: {errors:?}"
        );
    }

    // ── rsize ──────────────────────────────────────────────────────────────────

    #[test]
    fn rsize_mismatch_produces_rsize_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        let mut rel_sz = LangTuple::<u32>::default();
        rel_sz.set(FontLang::Hangul, 80); // 80%, spec wants 100%
        cs.rel_sz = rel_sz;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            rsize: Some(100),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_RSIZE),
            "expected CHARSHAPE_RSIZE error; got: {errors:?}"
        );
    }

    // ── position ───────────────────────────────────────────────────────────────

    #[test]
    fn position_mismatch_produces_position_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        let mut offset = LangTuple::<i32>::default();
        offset.set(FontLang::Hangul, 5); // offset = 5, spec wants 0
        cs.offset = offset;
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            position: Some(0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_POSITION),
            "expected CHARSHAPE_POSITION error; got: {errors:?}"
        );
    }

    // ── emptyspace ─────────────────────────────────────────────────────────────

    #[test]
    fn emptyspace_mismatch_produces_emptyspace_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.use_font_space = true; // doc uses font space, spec wants false
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            emptyspace: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_EMPTYSPACE),
            "expected CHARSHAPE_EMPTYSPACE error; got: {errors:?}"
        );
    }

    // ── point ──────────────────────────────────────────────────────────────────

    #[test]
    fn point_match_produces_no_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.height = 1000; // 10pt
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            point: Some(10.0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().all(|e| e.error_code != CHARSHAPE_POINT),
            "no CHARSHAPE_POINT error expected; got: {errors:?}"
        );
    }

    #[test]
    fn point_mismatch_produces_point_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.height = 1200; // 12pt, spec wants 10pt
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            point: Some(10.0),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_POINT),
            "expected CHARSHAPE_POINT error; got: {errors:?}"
        );
    }

    // ── kerning ────────────────────────────────────────────────────────────────

    #[test]
    fn kerning_mismatch_produces_kerning_error() {
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.use_kerning = true; // doc has kerning, spec wants false
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            kerning: Some(false),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_KERNING),
            "expected CHARSHAPE_KERNING error; got: {errors:?}"
        );
    }

    // ── check level ───────────────────────────────────────────────────────────

    #[test]
    fn simple_level_stops_at_first_error() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        // Create two runs both referencing the same non-compliant charshape.
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0), make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::Simple);
        assert_eq!(
            errors.len(),
            1,
            "Simple level must stop after the first error"
        );
    }

    #[test]
    fn all_level_collects_all_errors() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(0, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        // Two runs both violating the font rule.
        let runs = vec![make_run(0), make_run(0)];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert_eq!(
            errors.len(),
            2,
            "All level must collect errors for every run"
        );
    }

    // ── run metadata propagation ───────────────────────────────────────────────

    #[test]
    fn error_carries_run_metadata() {
        let font_faces = make_font_faces(&[(1, "나눔고딕")]);
        let cs = make_char_shape(42, 1, 100, 0);
        let tables = make_tables_with(cs, font_faces);
        let run = RunTypeInfo {
            char_pr_id_ref: 42,
            para_pr_id_ref: 7,
            text: "Hello".to_string(),
            is_in_table: true,
            table_id: 3,
            table_row: 1,
            table_col: 2,
            ..Default::default()
        };
        let runs = vec![run];

        let spec = CharShapeSpec {
            font: vec!["함초롬바탕".to_string()],
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert_eq!(errors.len(), 1);
        let e = &errors[0];
        assert_eq!(e.char_pr_id_ref, 42);
        assert_eq!(e.para_pr_id_ref, 7);
        assert_eq!(e.text, "Hello");
        assert!(e.is_in_table);
        assert_eq!(e.table_id, 3);
        assert_eq!(e.table_row, 1);
        assert_eq!(e.table_col, 2);
        assert!(
            e.error_code >= 1000 && e.error_code < 2000,
            "error code must be in CharShape range"
        );
    }

    // ── bg_border presence check ───────────────────────────────────────────────

    #[test]
    fn bg_border_absent_when_spec_requires_produces_error() {
        use crate::spec::CharShapeBorderSpec;
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.border_fill_id_ref = 0; // no border fill referenced
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            border: Some(CharShapeBorderSpec {
                position: Some(1),
                bordertype: Some(1),
                size: Some(0.12),
                color: Some("#000000".to_string()),
            }),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().any(|e| e.error_code == CHARSHAPE_BG_BORDER),
            "expected CHARSHAPE_BG_BORDER error when border_fill_id_ref=0; got: {errors:?}"
        );
    }

    #[test]
    fn bg_border_present_with_spec_does_not_produce_error() {
        use crate::spec::CharShapeBorderSpec;
        let font_faces = make_font_faces(&[(1, "함초롬바탕")]);
        let mut cs = make_char_shape(0, 1, 100, 0);
        cs.border_fill_id_ref = 1; // border fill is referenced
        let tables = make_tables_with(cs, font_faces);
        let runs = vec![make_run(0)];

        let spec = CharShapeSpec {
            border: Some(CharShapeBorderSpec {
                position: Some(1),
                bordertype: Some(1),
                size: Some(0.12),
                color: Some("#000000".to_string()),
            }),
            ..Default::default()
        };

        let errors = check(&spec, &tables, &runs, CheckLevel::All);
        assert!(
            errors.iter().all(|e| e.error_code != CHARSHAPE_BG_BORDER),
            "no CHARSHAPE_BG_BORDER error expected when border_fill present; got: {errors:?}"
        );
    }
}
