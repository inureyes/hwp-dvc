//! Validation engine — compares a [`Document`] against a [`DvcSpec`]
//! and produces [`DvcErrorInfo`] records.
//!
//! Maps to `Checker` in `references/dvc/Checker.h`. Each `Check*`
//! method in the C++ version becomes an associated function here.

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
        Self { spec, document, level: CheckLevel::default(), scope: OutputScope::default() }
    }

    /// Run every enabled check and return the collected errors.
    ///
    /// TODO: port `CheckCharShape`, `CheckParaShape`, `CheckTable`,
    /// `CheckSpacialCharacter`, `CheckOutlineShape`, `CheckBullet`,
    /// `CheckParaNumBullet`, `CheckHyperlink`, `CheckStyle`,
    /// `CheckMacro` from `references/dvc/Checker.cpp`.
    pub fn run(&self) -> DvcResult<Vec<DvcErrorInfo>> {
        Ok(Vec::new())
    }
}
