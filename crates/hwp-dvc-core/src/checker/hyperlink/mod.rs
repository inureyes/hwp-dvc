//! Hyperlink validator.
//!
//! Maps to `CheckHyperlink` in `references/dvc/Checker.cpp`.
//!
//! When the spec contains `"hyperlink": { "permission": false }`, every
//! run flagged `is_hyperlink == true` in the [`RunTypeInfo`] stream is
//! reported as a [`DvcErrorInfo`] with `error_code`
//! [`HYPERLINK_PERMISSION`] (6901).

use crate::checker::DvcErrorInfo;
use crate::document::RunTypeInfo;
use crate::error::{ErrorCode, ErrorContext};
use crate::spec::HyperlinkSpec;

/// Error code for a forbidden hyperlink run.
///
/// The [`ErrorCode::Hyperlink`] base is 6900; this constant occupies
/// the first slot (6901), matching the reference C++ `CHyperlink`.
pub const HYPERLINK_PERMISSION: u32 = ErrorCode::Hyperlink as u32 + 1;

/// Walk `run_type_infos` and emit one [`DvcErrorInfo`] per hyperlink
/// run when `spec.permission == false`.
///
/// Returns an empty `Vec` when either `spec.permission == true` (hyperlinks
/// are allowed) or no run is flagged `is_hyperlink`.
pub fn check(spec: &HyperlinkSpec, run_type_infos: &[RunTypeInfo]) -> Vec<DvcErrorInfo> {
    if spec.permission {
        return Vec::new();
    }

    run_type_infos
        .iter()
        .filter(|run| run.is_hyperlink)
        .map(|run| DvcErrorInfo {
            char_pr_id_ref: run.char_pr_id_ref,
            para_pr_id_ref: run.para_pr_id_ref,
            text: run.text.clone(),
            page_no: run.page_no,
            line_no: run.line_no,
            error_code: HYPERLINK_PERMISSION,
            table_id: run.table_id,
            is_in_table: run.is_in_table,
            is_in_table_in_table: run.is_in_table_in_table,
            table_row: run.table_row,
            table_col: run.table_col,
            is_in_shape: run.is_in_shape,
            use_hyperlink: true,
            use_style: false,
            error_string: crate::error::error_string(HYPERLINK_PERMISSION, ErrorContext::default()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::RunTypeInfo;
    use crate::spec::HyperlinkSpec;

    fn hyperlink_run() -> RunTypeInfo {
        RunTypeInfo {
            is_hyperlink: true,
            text: "click me".to_owned(),
            char_pr_id_ref: 1,
            ..Default::default()
        }
    }

    fn plain_run() -> RunTypeInfo {
        RunTypeInfo {
            is_hyperlink: false,
            text: "plain text".to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn permission_true_emits_no_errors() {
        let spec = HyperlinkSpec { permission: true };
        let runs = vec![hyperlink_run(), plain_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "permission=true must never produce errors; got {errors:?}"
        );
    }

    #[test]
    fn permission_false_emits_error_for_hyperlink_run() {
        let spec = HyperlinkSpec { permission: false };
        let runs = vec![hyperlink_run(), plain_run()];
        let errors = check(&spec, &runs);
        assert_eq!(
            errors.len(),
            1,
            "expected exactly one error for one hyperlink run; got {errors:?}"
        );
        assert_eq!(errors[0].error_code, HYPERLINK_PERMISSION);
        assert!(errors[0].use_hyperlink);
        assert_eq!(errors[0].text, "click me");
    }

    #[test]
    fn permission_false_no_hyperlinks_emits_no_errors() {
        let spec = HyperlinkSpec { permission: false };
        let runs = vec![plain_run(), plain_run()];
        let errors = check(&spec, &runs);
        assert!(
            errors.is_empty(),
            "no hyperlink runs means no errors; got {errors:?}"
        );
    }

    #[test]
    fn permission_false_multiple_hyperlinks_emits_one_error_each() {
        let spec = HyperlinkSpec { permission: false };
        let runs = vec![hyperlink_run(), hyperlink_run(), plain_run()];
        let errors = check(&spec, &runs);
        assert_eq!(
            errors.len(),
            2,
            "two hyperlink runs produce two errors; got {errors:?}"
        );
        for e in &errors {
            assert_eq!(e.error_code, HYPERLINK_PERMISSION);
        }
    }

    #[test]
    fn error_code_is_in_hyperlink_range() {
        assert!(
            HYPERLINK_PERMISSION >= ErrorCode::Hyperlink as u32,
            "HYPERLINK_PERMISSION must be >= 6900"
        );
        assert!(
            HYPERLINK_PERMISSION < ErrorCode::Macro as u32,
            "HYPERLINK_PERMISSION must be < 7000 (next category)"
        );
    }
}
