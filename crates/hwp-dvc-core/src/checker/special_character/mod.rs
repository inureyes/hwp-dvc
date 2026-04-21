//! `CheckSpacialCharacter` validator — issue #8.
//!
//! Walks `run_type_infos` and checks every Unicode codepoint in each
//! run's text against `[SpecialCharacterSpec.minimum,
//! SpecialCharacterSpec.maximum]`. One [`DvcErrorInfo`] is emitted per
//! offending run (not per character), using error code
//! [`SPECIALCHAR_MIN`] (3101) when the first out-of-range codepoint is
//! below the minimum, or [`SPECIALCHAR_MAX`] (3102) when it is above
//! the maximum.
//!
//! # Reference
//!
//! Mirrors `Checker::CheckSpacialCharacter` in
//! `references/dvc/Source/Checker.cpp`.

use crate::checker::DvcErrorInfo;
use crate::document::RunTypeInfo;
use crate::error::{SPECIALCHAR_MAX, SPECIALCHAR_MIN};
use crate::spec::SpecialCharacterSpec;

/// Validate every run against the special-character codepoint range.
///
/// Returns a `Vec<DvcErrorInfo>` (possibly empty) with one entry per
/// run that contains at least one out-of-range codepoint. The returned
/// vector is appended to the caller's error list — callers are
/// responsible for concatenation.
///
/// Run text is XML-unescaped before codepoint checks because the
/// section parser stores the raw escape sequences (e.g. `&#x7;` for
/// BEL) without decoding them. Unescaping here ensures the validator
/// sees the logical Unicode codepoints that the XML author intended.
/// If unescaping fails the raw text is used as a fallback so that the
/// check degrades gracefully on malformed input.
///
/// # Arguments
///
/// * `spec` — the `specialcharacter` section from the DVC spec.
/// * `run_type_infos` — the flattened run stream produced by
///   [`crate::document::run_type::build_run_type_infos`].
#[must_use]
pub fn check(spec: &SpecialCharacterSpec, run_type_infos: &[RunTypeInfo]) -> Vec<DvcErrorInfo> {
    let mut errors: Vec<DvcErrorInfo> = Vec::new();

    for run in run_type_infos {
        // Unescape XML character references (e.g. `&#x7;` → U+0007) so
        // that we compare logical Unicode codepoints, not escape sequences.
        let logical: std::borrow::Cow<'_, str> =
            quick_xml::escape::unescape(&run.text).unwrap_or(std::borrow::Cow::Borrowed(&run.text));

        // Find the first offending codepoint in this run, if any.
        let violation = logical.chars().find(|&c| {
            let cp = c as u32;
            cp < spec.minimum || cp > spec.maximum
        });

        let Some(offending_char) = violation else {
            continue;
        };

        let cp = offending_char as u32;
        let error_code = if cp < spec.minimum {
            SPECIALCHAR_MIN
        } else {
            SPECIALCHAR_MAX
        };

        errors.push(DvcErrorInfo {
            char_pr_id_ref: run.char_pr_id_ref,
            para_pr_id_ref: run.para_pr_id_ref,
            text: run.text.clone(),
            page_no: run.page_no,
            line_no: run.line_no,
            error_code,
            table_id: run.table_id,
            is_in_table: run.is_in_table,
            is_in_table_in_table: run.is_in_table_in_table,
            table_row: run.table_row,
            table_col: run.table_col,
            is_in_shape: run.is_in_shape,
            use_hyperlink: run.is_hyperlink,
            use_style: run.is_style,
            error_string: format!(
                "codepoint U+{cp:04X} is out of allowed range [{}, {}]",
                spec.minimum, spec.maximum
            ),
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::RunTypeInfo;
    use crate::spec::SpecialCharacterSpec;

    fn spec(min: u32, max: u32) -> SpecialCharacterSpec {
        SpecialCharacterSpec {
            minimum: min,
            maximum: max,
        }
    }

    fn run_with(text: &str) -> RunTypeInfo {
        RunTypeInfo {
            text: text.to_owned(),
            ..RunTypeInfo::default()
        }
    }

    #[test]
    fn empty_runs_produces_no_errors() {
        let errors = check(&spec(32, 1_048_575), &[]);
        assert!(errors.is_empty());
    }

    #[test]
    fn clean_ascii_and_korean_passes() {
        // Standard printable ASCII (U+0020..=U+007E) and Hangul syllables
        // (U+AC00..=U+D7A3) are both inside [32, 1_048_575].
        let runs = vec![
            run_with("Hello, world!"),
            run_with("안녕하세요"),
            run_with("テスト"),
            run_with("中文"),
        ];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert!(
            errors.is_empty(),
            "expected no errors for clean text; got: {:?}",
            errors
        );
    }

    #[test]
    fn bel_control_char_triggers_min_error() {
        // BEL (U+0007) is below minimum=32 → SPECIALCHAR_MIN (3101).
        let runs = vec![
            run_with("ok"),
            run_with("bad\u{0007}text"),
            run_with("fine"),
        ];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert_eq!(errors.len(), 1, "only the BEL run should produce an error");
        assert_eq!(errors[0].error_code, SPECIALCHAR_MIN);
        assert_eq!(errors[0].text, "bad\u{0007}text");
        assert!(errors[0].error_string.contains("U+0007"));
    }

    #[test]
    fn codepoint_above_max_triggers_max_error() {
        // Use a tight upper bound so a normal codepoint exceeds it.
        // U+1F600 (😀, emoji) > 127.
        let runs = vec![run_with("😀")];
        let errors = check(&spec(32, 127), &runs);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, SPECIALCHAR_MAX);
    }

    #[test]
    fn one_error_per_run_not_per_character() {
        // A run with three BEL chars must still produce exactly one error.
        let runs = vec![run_with("\u{0007}\u{0007}\u{0007}")];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert_eq!(errors.len(), 1, "one error per run, not per character");
    }

    #[test]
    fn multiple_offending_runs_each_emit_one_error() {
        let runs = vec![
            run_with("clean"),
            run_with("\u{0007}bad1"),
            run_with("also-clean"),
            run_with("\u{0008}bad2"),
        ];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.error_code == SPECIALCHAR_MIN));
    }

    #[test]
    fn emoji_inside_wide_range_passes() {
        // Default fixture range: [32, 1_048_575]. U+1F600 = 128_512, inside.
        let runs = vec![run_with("Hello 😀 World")];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert!(errors.is_empty(), "emoji within range must not be flagged");
    }

    #[test]
    fn boundary_values_pass() {
        // Exactly at minimum (U+0020 = space) and maximum (U+0FFFFF = 1_048_575).
        let boundary = "\u{0020}\u{FFFFF}";
        let runs = vec![run_with(boundary)];
        let errors = check(&spec(32, 1_048_575), &runs);
        assert!(errors.is_empty(), "boundary codepoints must not be flagged");
    }

    #[test]
    fn run_metadata_is_copied_into_error() {
        let run = RunTypeInfo {
            text: "\u{0007}".to_owned(),
            char_pr_id_ref: 5,
            para_pr_id_ref: 3,
            is_in_table: true,
            table_id: 99,
            table_row: 1,
            table_col: 2,
            is_hyperlink: true,
            is_style: true,
            ..RunTypeInfo::default()
        };
        let errors = check(&spec(32, 1_048_575), &[run]);
        assert_eq!(errors.len(), 1);
        let e = &errors[0];
        assert_eq!(e.char_pr_id_ref, 5);
        assert_eq!(e.para_pr_id_ref, 3);
        assert!(e.is_in_table);
        assert_eq!(e.table_id, 99);
        assert_eq!(e.table_row, 1);
        assert_eq!(e.table_col, 2);
        assert!(e.use_hyperlink);
        assert!(e.use_style);
    }
}
