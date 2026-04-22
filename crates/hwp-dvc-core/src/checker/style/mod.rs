//! `CheckStyle` validator — mirrors `Checker::CheckStyle` in the
//! reference C++ implementation (`references/dvc/Checker.cpp`).
//!
//! # Logic
//!
//! Two independent checks are performed, each governed by a different
//! field in [`StyleSpec`]:
//!
//! ## 1. Permission gate (error 3502, `STYLE_PERMISSION`)
//!
//! When `spec.permission == false`, every [`RunTypeInfo`] whose
//! `is_style == true` (paragraph uses a style other than 바탕글)
//! produces one [`DvcErrorInfo`] with error code 3502.
//!
//! When `permission == true`, no 3502 errors are emitted.
//!
//! ## 2. Type allow-list (error 3501, `STYLE_TYPE`)
//!
//! When `spec.allowed_types` is non-empty, every run whose paragraph
//! `style_name` is **not** in the allowed list produces one
//! [`DvcErrorInfo`] with error code 3501.
//!
//! This check fires regardless of the `permission` flag — a document
//! may allow custom styles in general but still restrict which
//! logical style types are valid.
//!
//! When `allowed_types` is absent or empty, no 3501 errors are emitted.
//!
//! The error codes mirror `JID_STYLE_TYPE` (3501) and
//! `JID_STYLE_PERMISSION` (3502) from
//! `references/dvc/Source/JsonModel.h`.
//!
//! [`RunTypeInfo`]: crate::document::RunTypeInfo

use crate::checker::DvcErrorInfo;
use crate::document::RunTypeInfo;
use crate::error::{style_codes, ErrorCode, ErrorContext};
use crate::spec::StyleSpec;

// Re-export the canonical constants so that callers that import from
// this module do not also need to import `crate::error::style_codes`.
pub use style_codes::STYLE_PERMISSION;
pub use style_codes::STYLE_TYPE;

/// Guard: `STYLE_PERMISSION` must be in the Style (3500) range.
const _: () = assert!(
    STYLE_PERMISSION >= ErrorCode::Style as u32,
    "STYLE_PERMISSION must be >= Style base (3500)",
);
const _: () = assert!(
    STYLE_TYPE >= ErrorCode::Style as u32,
    "STYLE_TYPE must be >= Style base (3500)",
);

/// Run the style checks over a slice of [`RunTypeInfo`]s.
///
/// Two disjoint error classes are produced:
/// - **3501** (`STYLE_TYPE`) — run's style name is not in
///   `spec.allowed_types` (only when the list is non-empty).
/// - **3502** (`STYLE_PERMISSION`) — run uses a non-default style but
///   `spec.permission == false`.
///
/// # Parameters
/// - `spec`  — the `StyleSpec` extracted from the user's DVC JSON file.
/// - `runs`  — the flattened run stream produced by
///   [`crate::document::run_type::build_run_type_infos`].
///
/// # Returns
/// A `Vec<DvcErrorInfo>` — empty when both gates are satisfied.
#[must_use]
pub fn check(spec: &StyleSpec, runs: &[RunTypeInfo]) -> Vec<DvcErrorInfo> {
    let mut errors = Vec::new();

    // ── 1. Type allow-list check (JID_STYLE_TYPE = 3501) ──────────────────
    // Only active when the spec declares a non-empty allowed_types list.
    if let Some(allowed) = &spec.allowed_types {
        if !allowed.is_empty() {
            for r in runs {
                // Build the set of allowed Korean names once per call.
                // The list is typically short (≤ 23 entries), so a linear
                // scan is cheaper than a HashSet for this cardinality.
                let name_allowed = allowed
                    .iter()
                    .any(|t| t.as_korean_name() == r.style_name.as_str());
                if !name_allowed {
                    errors.push(make_error(r, STYLE_TYPE));
                }
            }
        }
    }

    // ── 2. Permission gate (JID_STYLE_PERMISSION = 3502) ──────────────────
    // Only active when permission == false; skips 바탕글 runs (is_style=false).
    if !spec.permission {
        for r in runs.iter().filter(|r| r.is_style) {
            errors.push(make_error(r, STYLE_PERMISSION));
        }
    }

    errors
}

/// Build a [`DvcErrorInfo`] from a run and a style error code.
fn make_error(r: &RunTypeInfo, error_code: u32) -> DvcErrorInfo {
    DvcErrorInfo {
        char_pr_id_ref: r.char_pr_id_ref,
        para_pr_id_ref: r.para_pr_id_ref,
        text: r.text.clone(),
        page_no: r.page_no,
        line_no: r.line_no,
        error_code,
        table_id: r.table_id,
        is_in_table: r.is_in_table,
        is_in_table_in_table: r.is_in_table_in_table,
        table_row: r.table_row,
        table_col: r.table_col,
        is_in_shape: r.is_in_shape,
        use_hyperlink: r.is_hyperlink,
        use_style: true,
        error_string: crate::error::error_string(error_code, ErrorContext::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::RunTypeInfo;
    use crate::spec::{StyleSpec, StyleType};

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn run_with_style(is_style: bool, style_name: &str) -> RunTypeInfo {
        RunTypeInfo {
            is_style,
            style_name: style_name.into(),
            text: "테스트".into(),
            char_pr_id_ref: 1,
            para_pr_id_ref: 0,
            ..Default::default()
        }
    }

    fn default_run() -> RunTypeInfo {
        run_with_style(false, "바탕글")
    }

    fn body_run() -> RunTypeInfo {
        run_with_style(true, "본문")
    }

    fn custom_run() -> RunTypeInfo {
        run_with_style(true, "커스텀스타일")
    }

    fn spec_permission_only(permission: bool) -> StyleSpec {
        StyleSpec {
            permission,
            allowed_types: None,
        }
    }

    // ── Permission-gate tests (3502) ──────────────────────────────────────────

    #[test]
    fn permission_true_emits_no_permission_errors() {
        let spec = spec_permission_only(true);
        let runs = vec![body_run(), default_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "permission=true must not produce any Style errors"
        );
    }

    #[test]
    fn permission_false_emits_error_for_styled_runs() {
        let spec = spec_permission_only(false);
        let runs = vec![body_run(), default_run(), custom_run()];
        let errors = check(&spec, &runs);
        // Two runs have is_style=true (body and custom).
        let perm_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == STYLE_PERMISSION)
            .collect();
        assert_eq!(perm_errors.len(), 2, "exactly two 3502 errors expected");
        for e in &perm_errors {
            assert!(e.use_style, "use_style flag must be set on 3502 errors");
        }
    }

    #[test]
    fn permission_false_no_styled_runs_emits_no_errors() {
        let spec = spec_permission_only(false);
        let runs = vec![default_run(), default_run()];
        let errors = check(&spec, &runs);
        assert!(errors.is_empty(), "no styled runs → no errors");
    }

    // ── Type allow-list tests (3501) ──────────────────────────────────────────

    #[test]
    fn allowed_types_empty_emits_no_type_errors() {
        let spec = StyleSpec {
            permission: true,
            allowed_types: Some(vec![]),
        };
        let runs = vec![body_run(), custom_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "empty allowed_types list must not emit any 3501 errors"
        );
    }

    #[test]
    fn allowed_types_none_emits_no_type_errors() {
        let spec = StyleSpec {
            permission: true,
            allowed_types: None,
        };
        let runs = vec![body_run(), custom_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "absent allowed_types must not emit any 3501 errors"
        );
    }

    #[test]
    fn allowed_types_matching_run_emits_no_type_error() {
        let spec = StyleSpec {
            permission: true,
            allowed_types: Some(vec![StyleType::Normal, StyleType::Body]),
        };
        // 바탕글 and 본문 are both in the allow-list.
        let runs = vec![default_run(), body_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "all runs match allowed_types → no 3501 errors"
        );
    }

    #[test]
    fn allowed_types_non_matching_run_emits_type_error() {
        let spec = StyleSpec {
            permission: true,
            allowed_types: Some(vec![StyleType::Normal]),
        };
        // 바탕글 is allowed; 본문 is not.
        let runs = vec![default_run(), body_run()];
        let type_errors: Vec<_> = check(&spec, &runs)
            .into_iter()
            .filter(|e| e.error_code == STYLE_TYPE)
            .collect();
        assert_eq!(type_errors.len(), 1, "one 3501 error expected for 본문");
        assert!(type_errors[0].use_style);
    }

    #[test]
    fn custom_style_not_in_allowed_types_emits_type_error() {
        let spec = StyleSpec {
            permission: true,
            allowed_types: Some(vec![StyleType::Normal, StyleType::Body]),
        };
        let runs = vec![custom_run()];
        let errors = check(&spec, &runs);
        let type_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == STYLE_TYPE)
            .collect();
        assert_eq!(type_errors.len(), 1, "custom style must trigger 3501");
    }

    // ── Both checks active simultaneously ────────────────────────────────────

    #[test]
    fn both_checks_fire_independently() {
        // permission=false + allowed_types=[바탕글] means:
        //   • 본문 run triggers 3501 (not in type list) AND 3502 (not 바탕글).
        //   • 바탕글 run triggers neither (is_style=false, name in list).
        let spec = StyleSpec {
            permission: false,
            allowed_types: Some(vec![StyleType::Normal]),
        };
        let runs = vec![default_run(), body_run()];
        let errors = check(&spec, &runs);
        let type_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == STYLE_TYPE)
            .collect();
        let perm_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_code == STYLE_PERMISSION)
            .collect();
        assert_eq!(type_errors.len(), 1, "one 3501 for 본문");
        assert_eq!(perm_errors.len(), 1, "one 3502 for 본문");
    }

    #[test]
    fn permission_true_with_allowed_types_only_3501_fires() {
        // permission=true + allowed_types=[바탕글] means no 3502 but 3501
        // fires for 본문.
        let spec = StyleSpec {
            permission: true,
            allowed_types: Some(vec![StyleType::Normal]),
        };
        let runs = vec![default_run(), body_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.iter().all(|e| e.error_code == STYLE_TYPE),
            "only 3501 errors expected when permission=true"
        );
        assert_eq!(errors.len(), 1, "one 3501 for 본문");
    }

    // ── Error field mirror test ───────────────────────────────────────────────

    #[test]
    fn error_fields_mirror_run_fields() {
        let spec = spec_permission_only(false);
        let run = RunTypeInfo {
            is_style: true,
            style_name: "본문".into(),
            text: "스타일텍스트".into(),
            char_pr_id_ref: 7,
            para_pr_id_ref: 3,
            is_in_table: true,
            table_id: 42,
            table_row: 1,
            table_col: 2,
            is_hyperlink: false,
            ..Default::default()
        };
        let errors = check(&spec, &[run]);
        assert_eq!(errors.len(), 1);
        let e = &errors[0];
        assert_eq!(e.text, "스타일텍스트");
        assert_eq!(e.char_pr_id_ref, 7);
        assert_eq!(e.para_pr_id_ref, 3);
        assert!(e.is_in_table);
        assert_eq!(e.table_id, 42);
        assert_eq!(e.table_row, 1);
        assert_eq!(e.table_col, 2);
        assert!(!e.use_hyperlink);
        assert!(e.use_style);
        assert_eq!(e.error_code, STYLE_PERMISSION);
    }

    // ── Constant range guard ──────────────────────────────────────────────────

    #[test]
    fn style_constants_are_in_style_range() {
        assert!(
            STYLE_PERMISSION >= ErrorCode::Style as u32,
            "STYLE_PERMISSION must be >= Style base (3500)"
        );
        assert!(
            STYLE_PERMISSION < ErrorCode::Page as u32,
            "STYLE_PERMISSION must be < next category (Page=4000)"
        );
        assert!(
            STYLE_TYPE >= ErrorCode::Style as u32,
            "STYLE_TYPE must be >= Style base (3500)"
        );
        assert!(
            STYLE_TYPE < ErrorCode::Page as u32,
            "STYLE_TYPE must be < next category (Page=4000)"
        );
    }
}
