//! Validation engine — compares a [`Document`] against a [`DvcSpec`]
//! and produces [`DvcErrorInfo`] records.
//!
//! Maps to `Checker` in `references/dvc/Checker.h`. Each `Check*`
//! method in the C++ version becomes an associated function here.

pub mod char_shape;
pub mod hyperlink;
pub mod macro_;
pub mod para_shape;
pub mod special_character;
pub mod style;

use crate::document::Document;
use crate::error::DvcResult;
use crate::spec::DvcSpec;

/// A single validation finding.
///
/// Mirrors `DVCErrorInfo` / `IDVCErrInfo`. Field names match the JSON
/// keys used by the reference output (see `references/dvc/README.md`
/// for an example) so that callers can serialize directly.
#[derive(Debug, Default, Clone)]
pub struct DvcErrorInfo {
    pub char_pr_id_ref: u32,
    pub para_pr_id_ref: u32,
    pub text: String,
    pub page_no: u32,
    pub line_no: u32,
    pub error_code: u32,
    pub table_id: u32,
    pub is_in_table: bool,
    pub is_in_table_in_table: bool,
    pub table_row: u32,
    pub table_col: u32,
    pub is_in_shape: bool,
    pub use_hyperlink: bool,
    pub use_style: bool,
    pub error_string: String,
}

/// Validation level — mirrors `--simple` vs `--all`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CheckLevel {
    /// Stop at the first detected error.
    Simple,
    /// Report every error found.
    #[default]
    All,
}

/// Output scope toggles — mirror `-d/-o/-t/-i/-p/-y/-k`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputScope {
    pub all: bool,
    pub table: bool,
    pub table_detail: bool,
    pub shape: bool,
    pub style: bool,
    pub hyperlink: bool,
}

#[derive(Debug)]
pub struct Checker<'a> {
    pub spec: &'a DvcSpec,
    pub document: &'a Document,
    pub level: CheckLevel,
    pub scope: OutputScope,
}

impl<'a> Checker<'a> {
    pub fn new(spec: &'a DvcSpec, document: &'a Document) -> Self {
        Self {
            spec,
            document,
            level: CheckLevel::default(),
            scope: OutputScope::default(),
        }
    }

    /// Run every enabled check and return the collected errors.
    ///
    /// TODO: port `CheckTable`, `CheckOutlineShape`, `CheckBullet`,
    /// `CheckParaNumBullet` from `references/dvc/Checker.cpp`.
    pub fn run(&self) -> DvcResult<Vec<DvcErrorInfo>> {
        let mut errors: Vec<DvcErrorInfo> = Vec::new();

        // CheckHyperlink — report forbidden hyperlink runs.
        if let Some(spec) = &self.spec.hyperlink {
            errors.extend(hyperlink::check(spec, &self.document.run_type_infos));
        }

        if let Some(style_spec) = &self.spec.style {
            errors.extend(style::check(style_spec, &self.document.run_type_infos));
        }

        // CheckMacro — emit an error when macros are present but forbidden.
        if let Some(macro_spec) = &self.spec.macro_ {
            errors.extend(macro_::check(macro_spec, self.document));
        }

        // CheckSpecialCharacter — codepoint range check on run text.
        if let Some(sc_spec) = &self.spec.specialcharacter {
            errors.extend(special_character::check(
                sc_spec,
                &self.document.run_type_infos,
            ));
        }

        // CheckCharShape — mirrors Checker::CheckCharShape (Checker.cpp:87).
        if let Some(header) = &self.document.header {
            if let Some(charshape_spec) = &self.spec.charshape {
                let mut char_errors = char_shape::check(
                    charshape_spec,
                    header,
                    &self.document.run_type_infos,
                    self.level,
                );
                errors.append(&mut char_errors);
            }
        }

        // CheckParaShape — mirrors Checker::CheckParaShape.
        if let Some(parashape_spec) = &self.spec.parashape {
            errors.extend(para_shape::check(self.document, parashape_spec));
        }

        Ok(errors)
    }
}
