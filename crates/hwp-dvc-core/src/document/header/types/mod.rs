//! Typed header-table structs for OWPML `Contents/header.xml`.
//!
//! The shape set here mirrors the reference C++ (`RCharShape`,
//! `RParaShape`, `RTable` border-fill portion, `RBullets`,
//! `ROutlineShape`) closely enough to drive the Phase 2 validators,
//! while flattening awkward language-split structures into a single
//! seven-entry array indexed by [`FontLang`].
//!
//! Integers that the reference declares as `UINT` are kept as `u32`
//! here. Signed fields stay `i32`. Booleans are `bool`. Enum-like
//! attributes (`LINETYPE2`, `ParaType`, `HAlignType`, …) are decoded
//! into dedicated Rust enums with a fallible `parse` constructor so
//! that unknown values in future HWPX revisions can be flagged without
//! a crash.
//!
//! Every record carries its integer `id` (`charPrID`, `paraPrID`,
//! `borderFillId`, `styleId`, `bulletId`, `numberingId`) so that
//! downstream validators can re-emit it in error messages.
//!
//! # Module layout
//!
//! - [`enums`] — `FontLang` and the attribute-decoding enums plus the
//!   tiny `LangTuple<T>` helper.
//! - [`shapes`] — the struct records and their impls.

pub mod enums;
pub mod shapes;

pub use enums::{
    FontLang, HAlign, HeadingType, LangTuple, LineBreakWord, LineSpacingType, LineType, VAlign,
};
pub use shapes::{
    Border, BorderFill, Bullet, CharShape, FontFace, LineSpacing, Margin, Numbering, ParaHead,
    ParaShape, Style,
};
