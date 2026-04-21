//! HWPX Document Validation Checker (DVC) — core library.
//!
//! This crate is a Rust port of the reference C++ DVC implementation
//! (see `references/dvc/` in the repository). It exposes the pieces
//! needed to validate an HWPX document against a JSON-defined spec:
//!
//! - [`spec`]   — parsing the validation spec (the "CheckList")
//! - [`document`] — reading an HWPX file (OWPML reader)
//! - [`checker`]  — comparing document against spec and producing errors
//! - [`output`]   — formatting results as JSON (and, later, XML/text)
//! - [`error`]    — error types and error codes
//!
//! The surface is intentionally minimal right now; modules are stubs
//! that will be filled in as the port progresses.

pub mod checker;
pub mod document;
pub mod error;
pub mod output;
pub mod spec;

pub use error::{DvcError, DvcResult, ErrorCode};
