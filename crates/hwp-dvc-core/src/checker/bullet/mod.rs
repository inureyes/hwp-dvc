//! Bullet-character validator (`CheckBullet` in the reference C++).
//!
//! Mirrors `Checker::CheckBullet` and `Checker::CheckBulletToCheckList`
//! in `references/dvc/Checker.cpp`.
//!
//! # Algorithm
//!
//! 1. If the spec has no `bulletshapes` string, skip immediately.
//! 2. For every [`Bullet`] in the header table, compare its `char`
//!    field against the `bulletshapes` allow-list string.
//! 3. For each bullet whose character is not found in the allow-list,
//!    emit a [`DvcErrorInfo`] with [`BULLET_SHAPES`].
//!
//! # AST gap
//!
//! TODO: The [`Paragraph`] AST node does not yet carry a
//! `bullet_id_ref` field that would let us emit per-paragraph errors
//! matching the C++ reference. The section parser (issue #3) parses
//! `<hp:p>` `paraPrIDRef` and `styleIDRef` but does not yet surface
//! `<hp:paraNumPr>` / `<hp:bullet>` linkage. Until that gap is closed,
//! this validator emits one error per violating bullet table entry
//! (keyed to `para_pr_id_ref = 0`). Downstream callers that need
//! per-paragraph fan-out should extend the AST and the
//! `RunTypeInfo` builder (issue #4) to carry the bullet reference.
//!
//! # Error codes
//!
//! | Constant          | Value | `JID_*` reference        |
//! |-------------------|-------|--------------------------|
//! | `BULLET_CHECKTYPE`| 3302  | `JID_BULLET_CHECKTYPE`   |
//! | `BULLET_CODE`     | 3303  | `JID_BULLET_CODE`        |
//! | `BULLET_SHAPES`   | 3304  | `JID_BULLET_SHAPES`      |
//!
//! [`Bullet`]: crate::document::header::Bullet
//! [`Paragraph`]: crate::document::section::Paragraph

use crate::checker::DvcErrorInfo;
use crate::document::header::{Bullet, HeaderTables};
use crate::error::ErrorCode;
use crate::spec::BulletSpec;

/// Bullet check-type mismatch (`JID_BULLET_CHECKTYPE = 3302`).
pub const BULLET_CHECKTYPE: u32 = 3302;

/// Bullet character-code mismatch (`JID_BULLET_CODE = 3303`).
pub const BULLET_CODE: u32 = 3303;

/// Bullet shape not in the allow-list (`JID_BULLET_SHAPES = 3304`).
pub const BULLET_SHAPES: u32 = 3304;

/// Validate bullet characters against the spec allow-list.
///
/// Mirrors `Checker::CheckBullet` + `CheckBulletToCheckList` from
/// `references/dvc/Checker.cpp`.
///
/// # Parameters
///
/// * `spec`   — the `[bullet]` section of the DVC spec.
/// * `header` — the parsed header tables (provides `bullets`).
///
/// # Returns
///
/// A vec of [`DvcErrorInfo`] records, one per violating bullet entry.
/// Returns an empty vec if `spec.bulletshapes` is `None` or if all
/// bullet characters are in the allow-list.
pub fn check(spec: &BulletSpec, header: &HeaderTables) -> Vec<DvcErrorInfo> {
    let allowed = match &spec.bulletshapes {
        Some(s) if !s.is_empty() => s,
        _ => return Vec::new(),
    };

    header
        .bullets
        .values()
        .filter_map(|bullet| check_bullet_to_check_list(bullet, allowed))
        .collect()
}

/// Check a single [`Bullet`] entry against the allow-list.
///
/// Returns `Some(DvcErrorInfo)` when the bullet's character is not
/// found in `allowed`, `None` when it passes.
///
/// Mirrors `Checker::CheckBulletToCheckList`.
///
/// # Private-Use Area characters
///
/// HWPX authors sometimes use symbol fonts (e.g. Wingdings) for bullet
/// glyphs. In those fonts the glyph is stored as a Private Use Area
/// (PUA) code point in the range `U+E000..=U+F8FF`. These code points
/// are font-specific and cannot be compared directly to the Unicode
/// characters in the spec allow-list (which represent the *visual*
/// glyphs, not their PUA surrogates). To avoid false positives the
/// validator skips any bullet whose character falls entirely in the PUA
/// range, treating it as inherently allowed — the same conservative
/// heuristic the reference C++ implementation applies.
fn check_bullet_to_check_list(bullet: &Bullet, allowed: &str) -> Option<DvcErrorInfo> {
    // Image bullets do not carry a text character; skip the shape
    // check for them — the reference only validates text-character
    // bullets via this path.
    if bullet.use_image {
        return None;
    }

    // Skip bullets whose char is empty (malformed/unset).
    if bullet.char.is_empty() {
        return None;
    }

    // PUA characters (U+E000..=U+F8FF) are font-specific glyphs
    // (e.g. Wingdings-encoded bullets). They cannot be meaningfully
    // compared to the Unicode symbols in the allow-list string, so
    // we treat any bullet composed entirely of PUA code points as
    // passing the shape check.
    let all_pua = bullet
        .char
        .chars()
        .all(|c| ('\u{E000}'..='\u{F8FF}').contains(&c));
    if all_pua {
        return None;
    }

    // The bullet `char` field is a UTF-8 string holding one or more
    // code points. We check whether ALL chars of the bullet string are
    // present somewhere in the allow-list. In practice HWPX writers
    // emit a single Unicode code point per bullet.
    let shape_ok = bullet.char.chars().all(|c| allowed.contains(c));

    if shape_ok {
        return None;
    }

    // Emit an error for this bullet entry.
    // `para_pr_id_ref` is 0 because we validate at the bullet-table
    // level, not per-paragraph (see module-level TODO).
    // `error_string` carries the offending bullet character for
    // human-readable reporting.
    Some(DvcErrorInfo {
        para_pr_id_ref: 0,
        error_code: BULLET_SHAPES,
        error_string: bullet.char.clone(),
        // Base code in the Bullet range so callers can range-filter.
        // The error_code itself (3304) already encodes the range.
        table_id: ErrorCode::Bullet as u32, // 3300 — range marker
        ..DvcErrorInfo::default()
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::document::header::{Bullet, HeaderTables};
    use crate::spec::BulletSpec;

    fn make_header_with_bullets(bullets: Vec<Bullet>) -> HeaderTables {
        let mut h = HeaderTables::default();
        for b in bullets {
            h.bullets.insert(b.id, b);
        }
        h
    }

    fn spec_with_shapes(shapes: &str) -> BulletSpec {
        BulletSpec {
            bulletshapes: Some(shapes.to_string()),
        }
    }

    fn spec_no_shapes() -> BulletSpec {
        BulletSpec { bulletshapes: None }
    }

    // ── no spec → skip ────────────────────────────────────────────────────────

    #[test]
    fn no_bulletshapes_in_spec_produces_no_errors() {
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: "X".to_string(),
            use_image: false,
        }]);
        let errors = check(&spec_no_shapes(), &header);
        assert!(
            errors.is_empty(),
            "no bulletshapes spec must produce no errors"
        );
    }

    // ── empty allow-list → skip ────────────────────────────────────────────────

    #[test]
    fn empty_bulletshapes_string_produces_no_errors() {
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: "X".to_string(),
            use_image: false,
        }]);
        let spec = BulletSpec {
            bulletshapes: Some(String::new()),
        };
        let errors = check(&spec, &header);
        assert!(errors.is_empty(), "empty allow-list must skip the check");
    }

    // ── allowed bullets → no errors ───────────────────────────────────────────

    #[test]
    fn bullet_in_allowlist_produces_no_error() {
        let header = make_header_with_bullets(vec![
            Bullet {
                id: 1,
                char: "□".to_string(),
                use_image: false,
            },
            Bullet {
                id: 2,
                char: "○".to_string(),
                use_image: false,
            },
        ]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.is_empty(),
            "all bullets in allow-list must produce no errors; got: {errors:?}"
        );
    }

    // ── disallowed bullet → BULLET_SHAPES error ────────────────────────────────

    #[test]
    fn bullet_not_in_allowlist_produces_bullet_shapes_error() {
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: "X".to_string(), // 'X' not in "□○-•*"
            use_image: false,
        }]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.iter().any(|e| e.error_code == BULLET_SHAPES),
            "bullet 'X' not in allow-list must produce BULLET_SHAPES (3304); got: {errors:?}"
        );
    }

    // ── image bullets are skipped ──────────────────────────────────────────────

    #[test]
    fn image_bullet_is_skipped_even_if_char_is_disallowed() {
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: "X".to_string(),
            use_image: true, // image bullet — char field is irrelevant
        }]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.is_empty(),
            "image bullets must be skipped regardless of char; got: {errors:?}"
        );
    }

    // ── mixed bullets: one allowed, one not ───────────────────────────────────

    #[test]
    fn mixed_bullets_emits_error_only_for_disallowed() {
        let header = make_header_with_bullets(vec![
            Bullet {
                id: 1,
                char: "□".to_string(), // allowed
                use_image: false,
            },
            Bullet {
                id: 2,
                char: "Z".to_string(), // disallowed
                use_image: false,
            },
        ]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert_eq!(
            errors.len(),
            1,
            "only one disallowed bullet; expected 1 error, got: {errors:?}"
        );
        assert_eq!(errors[0].error_code, BULLET_SHAPES);
        assert_eq!(errors[0].error_string, "Z");
    }

    // ── PUA characters are skipped ────────────────────────────────────────────

    #[test]
    fn private_use_area_bullet_is_treated_as_allowed() {
        // U+F0A7 is a Wingdings PUA bullet (visual "•" in that font).
        // It must not produce an error regardless of the allow-list.
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: "\u{F0A7}".to_string(),
            use_image: false,
        }]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.is_empty(),
            "PUA bullet U+F0A7 must be treated as allowed; got: {errors:?}"
        );
    }

    // ── empty char is skipped ──────────────────────────────────────────────────

    #[test]
    fn empty_char_bullet_is_skipped() {
        let header = make_header_with_bullets(vec![Bullet {
            id: 1,
            char: String::new(),
            use_image: false,
        }]);
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.is_empty(),
            "empty-char bullet must be skipped; got: {errors:?}"
        );
    }

    // ── error range check ─────────────────────────────────────────────────────

    #[test]
    fn bullet_error_constants_have_correct_values() {
        // Use equality assertions (not range assertions on constants) to
        // satisfy `clippy::assertions_on_constants`.
        assert_eq!(BULLET_CHECKTYPE, 3302);
        assert_eq!(BULLET_CODE, 3303);
        assert_eq!(BULLET_SHAPES, 3304);
    }

    // ── empty bullet table → no errors ────────────────────────────────────────

    #[test]
    fn empty_bullet_table_produces_no_errors() {
        let header = HeaderTables {
            bullets: HashMap::new(),
            ..Default::default()
        };
        let spec = spec_with_shapes("□○-•*");
        let errors = check(&spec, &header);
        assert!(
            errors.is_empty(),
            "empty bullet table must produce no errors"
        );
    }
}
