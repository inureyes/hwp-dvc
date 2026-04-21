//! OWPML `Contents/section*.xml` parser — body paragraph/run/table AST.
//!
//! Each HWPX archive holds one or more `Contents/sectionN.xml` parts.
//! Each part represents one body section of the document and is a
//! tree of paragraphs, runs, and tables with arbitrary table-in-table
//! nesting. This module walks that tree and produces a typed AST
//! (see [`types`]) that downstream Phase 1c/Phase 2 code consumes.
//!
//! See `references/dvc/Source/OWPMLReader.cpp` and
//! `references/dvc/Source/RTable.cpp` for the C++ reference walker
//! this Rust implementation mirrors. The Rust version is simpler
//! because it doesn't carry layout information (pagination, line
//! segments) — those are deferred to issue #19.
//!
//! # Entry point
//!
//! [`super::HwpxArchive::read_sections`] is the one function callers
//! are expected to use. It locates every `Contents/sectionN.xml`
//! part inside the archive, parses each in ascending `N` order, and
//! returns one [`Section`] per part.

pub mod parser;
pub mod types;

pub use types::{Cell, Paragraph, Row, Run, Section, Table};
