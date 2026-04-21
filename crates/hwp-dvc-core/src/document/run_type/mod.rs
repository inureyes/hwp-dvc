//! Phase 1c builder: `Vec<RunTypeInfo>` from header tables + section AST.
//!
//! This module produces the **validator-facing stream** that every
//! Phase 2 validator consumes as input. One [`RunTypeInfo`] is emitted
//! per `<hp:run>` that carries a child list in the OWPML reference
//! C++ (`OWPMLReader::GetRunTypeInfos`). Here that condition is
//! approximated by "the run's tag was `<hp:run>...</hp:run>` rather
//! than `<hp:run/>`" — the section walker (#3) preserves that
//! distinction by only creating a [`Run`] with an empty text for an
//! empty element. See [`RUN_TYPE_EMISSION_POLICY`] for the full rule.
//!
//! # Scope (this issue, #4)
//!
//! The reference populates `pageNo` / `lineNo` by walking
//! `<hp:linesegarray>` and tracking cumulative vertical position.
//! That requires a minimal layout engine and is the deferred issue
//! [#19]. This module leaves both fields at 0 and documents that
//! explicitly with [`PAGE_LINE_OUT_OF_SCOPE`].
//!
//! # Entry point
//!
//! [`build_run_type_infos`] is the one function callers use.
//!
//! [#19]: https://github.com/inureyes/hwp-dvc/issues/19
//! [`Run`]: crate::document::section::Run

pub(crate) mod builder;

pub use builder::{
    build_run_type_infos, default_style_id, IS_IN_SHAPE_SIMPLIFICATION, RUN_TYPE_EMISSION_POLICY,
};

/// Documentation marker: `pageNo` / `lineNo` are intentionally left at
/// 0 in this issue. Populating them requires the layout engine
/// deferred to issue #19 (`feat: page/line numbering for RunTypeInfo`).
/// Validators that do not depend on pagination can proceed without
/// these fields; the only consumer is XML output formatting, which is
/// also in a later issue.
pub const PAGE_LINE_OUT_OF_SCOPE: &str = "pageNo/lineNo reserved for issue #19 (layout engine)";
