//! Validation engine — compares a [`Document`] against a [`DvcSpec`]
//! and produces [`DvcErrorInfo`] records.
//!
//! Maps to `Checker` in `references/dvc/Checker.h`. Each `Check*`
//! method in the C++ version becomes an associated function here.

pub mod bullet;
pub mod char_shape;
pub mod hyperlink;
pub mod macro_;
pub mod outline_shape;
pub mod para_num_bullet;
pub mod para_shape;
pub mod special_character;
pub mod style;
pub mod table;

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
///
/// When all flags are `false` (the default), every category is emitted —
/// this matches the reference C++ `-d / --default` behaviour.
/// When one or more flags are `true`, only the selected categories emit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputScope {
    /// `-o / --alloption`: emit every category (same as default).
    pub all: bool,
    /// `-t / --table`: emit table-level findings.
    pub table: bool,
    /// `-i / --tabledetail`: emit per-cell table findings.
    pub table_detail: bool,
    /// `-p / --shape`: emit shape findings (CharShape + ParaShape).
    pub shape: bool,
    /// `-y / --style`: emit style findings.
    pub style: bool,
    /// `-k / --hyperlink`: emit hyperlink findings.
    pub hyperlink: bool,
}

impl OutputScope {
    /// Returns `true` when no specific scope flag has been set (i.e. the
    /// caller passed `-d` or nothing), meaning every category should be
    /// emitted.
    #[inline]
    fn is_default(&self) -> bool {
        !self.all
            && !self.table
            && !self.table_detail
            && !self.shape
            && !self.style
            && !self.hyperlink
    }

    /// Returns `true` when this category should be included in the output.
    ///
    /// Category membership:
    /// - `"shape"` covers CharShape (1000-range) and ParaShape (2000-range).
    /// - `"table"` covers Table (3000-range except SpecialCharacter/Bullet/etc.).
    /// - `"style"` covers Style (3500-range).
    /// - `"hyperlink"` covers Hyperlink (6900-range).
    /// - All other validators (SpecialCharacter, OutlineShape, Bullet,
    ///   ParaNumBullet, Macro) always emit regardless of scope, as they
    ///   have no dedicated scope flag in the reference CLI.
    #[inline]
    pub fn allows(&self, category: ScopeCategory) -> bool {
        if self.is_default() || self.all {
            return true;
        }
        match category {
            ScopeCategory::Shape => self.shape,
            ScopeCategory::Table => self.table || self.table_detail,
            ScopeCategory::Style => self.style,
            ScopeCategory::Hyperlink => self.hyperlink,
            // Ungated categories (SpecialCharacter, Bullet, Macro, …) always pass.
            ScopeCategory::Ungated => true,
        }
    }
}

/// Identifies which scope category a validator belongs to.
///
/// Used by [`OutputScope::allows`] to decide whether to include a
/// validator's output in the final error list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeCategory {
    /// CharShape + ParaShape — controlled by `-p / --shape`.
    Shape,
    /// Table — controlled by `-t / --table` or `-i / --tabledetail`.
    Table,
    /// Style — controlled by `-y / --style`.
    Style,
    /// Hyperlink — controlled by `-k / --hyperlink`.
    Hyperlink,
    /// Validators with no dedicated scope flag (SpecialCharacter, OutlineShape,
    /// Bullet, ParaNumBullet, Macro). Always emitted.
    Ungated,
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
    /// Each validator is gated through [`OutputScope::allows`]:
    /// - Default (no flags set) or `--alloption` → all validators emit.
    /// - `--table` / `--tabledetail` → only Table errors.
    /// - `--shape` → only CharShape + ParaShape errors.
    /// - `--style` → only Style errors.
    /// - `--hyperlink` → only Hyperlink errors.
    /// - Ungated validators (SpecialCharacter, OutlineShape, Bullet,
    ///   ParaNumBullet, Macro) always emit.
    pub fn run(&self) -> DvcResult<Vec<DvcErrorInfo>> {
        let mut errors: Vec<DvcErrorInfo> = Vec::new();

        // CheckHyperlink — report forbidden hyperlink runs.
        if self.scope.allows(ScopeCategory::Hyperlink) {
            if let Some(spec) = &self.spec.hyperlink {
                errors.extend(hyperlink::check(spec, &self.document.run_type_infos));
            }
        }

        // CheckStyle — report runs using non-default styles when forbidden.
        if self.scope.allows(ScopeCategory::Style) {
            if let Some(style_spec) = &self.spec.style {
                errors.extend(style::check(style_spec, &self.document.run_type_infos));
            }
        }

        // CheckMacro — emit an error when macros are present but forbidden.
        // Ungated: no dedicated scope flag in the reference CLI.
        if self.scope.allows(ScopeCategory::Ungated) {
            if let Some(macro_spec) = &self.spec.macro_ {
                errors.extend(macro_::check(macro_spec, self.document));
            }
        }

        // CheckSpecialCharacter — codepoint range check on run text.
        // Ungated: no dedicated scope flag in the reference CLI.
        if self.scope.allows(ScopeCategory::Ungated) {
            if let Some(sc_spec) = &self.spec.specialcharacter {
                errors.extend(special_character::check(
                    sc_spec,
                    &self.document.run_type_infos,
                ));
            }
        }

        // CheckCharShape — mirrors Checker::CheckCharShape (Checker.cpp:87).
        // Gated by --shape.
        if self.scope.allows(ScopeCategory::Shape) {
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
        }

        // CheckParaShape — mirrors Checker::CheckParaShape.
        // Gated by --shape (same scope category as CharShape).
        if self.scope.allows(ScopeCategory::Shape) {
            if let Some(parashape_spec) = &self.spec.parashape {
                errors.extend(para_shape::check(self.document, parashape_spec));
            }
        }

        // CheckTable — mirrors Checker::CheckTable. Gated by --table / --tabledetail.
        if self.scope.allows(ScopeCategory::Table) {
            if let Some(table_spec) = &self.spec.table {
                errors.extend(table::check(
                    self.document,
                    table_spec,
                    self.level,
                    self.scope,
                )?);
            }
        }

        // CheckOutlineShape — validate outline numbering shapes per level.
        // Ungated: no dedicated scope flag in the reference CLI.
        if self.scope.allows(ScopeCategory::Ungated) {
            if let Some(outline_spec) = &self.spec.outlineshape {
                errors.extend(outline_shape::check(self.document, outline_spec));
            }
        }

        // CheckBullet — validate bullet characters against the spec allow-list.
        // Ungated: no dedicated scope flag in the reference CLI.
        if self.scope.allows(ScopeCategory::Ungated) {
            if let Some(bullet_spec) = &self.spec.bullet {
                if let Some(header) = &self.document.header {
                    errors.extend(bullet::check(bullet_spec, header));
                }
            }
        }

        // CheckParaNumBullet — mirrors Checker::CheckParaNumBullet.
        // Ungated: no dedicated scope flag in the reference CLI.
        if self.scope.allows(ScopeCategory::Ungated) {
            if let Some(paranum_spec) = &self.spec.paranumbullet {
                errors.extend(para_num_bullet::check(self.document, paranum_spec));
            }
        }

        Ok(errors)
    }
}
