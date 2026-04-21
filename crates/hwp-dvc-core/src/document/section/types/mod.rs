//! Typed AST for OWPML `Contents/section*.xml`.
//!
//! The shape set here mirrors the relevant parts of the reference C++
//! (`OWPMLReader.cpp`, `RTable.{h,cpp}`) while staying simpler: we only
//! keep the fields a downstream validator or the Phase 1c
//! `RunTypeInfo` builder (#4) needs. Heavy layout data
//! (`linesegarray`, `secPr`, page geometry) is intentionally dropped —
//! those belong to the deferred paging work (#19).
//!
//! # Module layout
//!
//! - [`nodes`] — the paragraph/run/table struct records and their impls.
//!
//! Every element carries the IDs the `HeaderTables` are keyed on
//! (`paraPrIDRef`, `charPrIDRef`, `styleIDRef`, `borderFillIDRef`) as
//! plain `u32`s. Resolving those IDs into shape records is out of
//! scope for this issue (#3) and done by the validators.

pub mod nodes;

pub use nodes::{Cell, Paragraph, Row, Run, Section, Table};
