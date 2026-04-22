//! `CheckStyle` validator — mirrors `Checker::CheckStyle` in the
//! reference C++ implementation (`references/dvc/Checker.cpp`).
//!
//! # Logic
//!
//! When the spec carries `"style": { "permission": false }`, every
//! [`RunTypeInfo`] whose `is_style == true` (i.e. whose paragraph was
//! formatted with a non-default style, meaning any style other than
//! "바탕글") produces one [`DvcErrorInfo`] with `error_code` in the
//! [`ErrorCode::Style`] (3500) range.
//!
//! When `permission == true`, no errors are emitted — the document is
//! free to use custom styles.
//!
//! The error code emitted per run is [`STYLE_PERMISSION`] (3502),
//! following the reference's scheme where 3500 is the category base
//! and individual sub-codes are offset from it.
//!
//! [`RunTypeInfo`]: crate::document::RunTypeInfo

use crate::checker::DvcErrorInfo;
use crate::document::RunTypeInfo;
use crate::error::{ErrorCode, ErrorContext};
use crate::spec::StyleSpec;

/// Concrete error code emitted when a run uses a non-default style and
/// `StyleSpec.permission == false`. Offset from the `Style` base (3500)
/// to leave room for the category base itself and a future "allowed
/// style list" code at 3501.
pub const STYLE_PERMISSION: u32 = ErrorCode::Style as u32 + 2;

/// Run the style check over a slice of [`RunTypeInfo`]s.
///
/// # Parameters
/// - `spec`  — the `StyleSpec` extracted from the user's DVC JSON file.
/// - `runs`  — the flattened run stream produced by
///   [`crate::document::run_type::build_run_type_infos`].
///
/// # Returns
/// A `Vec<DvcErrorInfo>` — empty when `spec.permission == true` or when
/// no run uses a custom style. Each entry's `error_code` is
/// [`STYLE_PERMISSION`] and `use_style` is set to `true`.
#[must_use]
pub fn check(spec: &StyleSpec, runs: &[RunTypeInfo]) -> Vec<DvcErrorInfo> {
    if spec.permission {
        // Styles are permitted: nothing to report.
        return Vec::new();
    }

    runs.iter()
        .filter(|r| r.is_style)
        .map(|r| DvcErrorInfo {
            char_pr_id_ref: r.char_pr_id_ref,
            para_pr_id_ref: r.para_pr_id_ref,
            text: r.text.clone(),
            page_no: r.page_no,
            line_no: r.line_no,
            error_code: STYLE_PERMISSION,
            table_id: r.table_id,
            is_in_table: r.is_in_table,
            is_in_table_in_table: r.is_in_table_in_table,
            table_row: r.table_row,
            table_col: r.table_col,
            is_in_shape: r.is_in_shape,
            use_hyperlink: r.is_hyperlink,
            use_style: true,
            error_string: crate::error::error_string(STYLE_PERMISSION, ErrorContext::default()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::RunTypeInfo;
    use crate::spec::StyleSpec;

    fn run_with_style(is_style: bool) -> RunTypeInfo {
        RunTypeInfo {
            is_style,
            text: "테스트".into(),
            char_pr_id_ref: 1,
            para_pr_id_ref: 0,
            ..Default::default()
        }
    }

    #[test]
    fn permission_true_emits_no_errors() {
        let spec = StyleSpec { permission: true };
        let runs = vec![run_with_style(true), run_with_style(false)];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "permission=true must not produce any Style errors"
        );
    }

    #[test]
    fn permission_false_emits_error_for_styled_runs() {
        let spec = StyleSpec { permission: false };
        let runs = vec![
            run_with_style(true),
            run_with_style(false),
            run_with_style(true),
        ];
        let errors = check(&spec, &runs);
        assert_eq!(
            errors.len(),
            2,
            "exactly two runs have is_style=true, so two errors expected"
        );
        for e in &errors {
            assert_eq!(
                e.error_code, STYLE_PERMISSION,
                "error_code must be STYLE_PERMISSION ({})",
                STYLE_PERMISSION
            );
            assert!(e.use_style, "use_style flag must be set");
        }
    }

    #[test]
    fn permission_false_no_styled_runs_emits_no_errors() {
        let spec = StyleSpec { permission: false };
        let runs = vec![run_with_style(false), run_with_style(false)];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "no styled runs means no errors even when permission=false"
        );
    }

    #[test]
    fn error_fields_mirror_run_fields() {
        let spec = StyleSpec { permission: false };
        let run = RunTypeInfo {
            is_style: true,
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

    #[test]
    fn style_permission_constant_is_in_style_range() {
        // Guard: STYLE_PERMISSION must be in [3500, 3600).
        assert!(
            STYLE_PERMISSION >= ErrorCode::Style as u32,
            "STYLE_PERMISSION must be >= Style base (3500)"
        );
        assert!(
            STYLE_PERMISSION < ErrorCode::Page as u32,
            "STYLE_PERMISSION must be < next category (Page=4000)"
        );
    }
}
