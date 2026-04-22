//! OWPML `Contents/header.xml` parser: shape tables.
//!
//! This module parses the six ID-indexed tables that every Phase 2
//! validator needs: character shapes, paragraph shapes, border fills,
//! styles, bullets, and numberings. Together with the font-face list
//! they are the bedrock of validation because the section XML only
//! carries *references* — integer IDs — into them.
//!
//! See `references/dvc/Source/RCharShape.{h,cpp}`,
//! `RParaShape.{h,cpp}`, `RTable.{h,cpp}` (border-fills portion),
//! `RBullets.{h,cpp}`, and `ROutlineShape.{h,cpp}` for the canonical
//! C++ shape.
//!
//! # Entry point
//!
//! [`super::HwpxArchive::read_header`] is the one function callers are
//! expected to use. It locates `Contents/header.xml` inside the
//! archive and returns a populated [`HeaderTables`].

pub mod parser;
pub mod types;

use std::collections::HashMap;

pub use types::{
    Border, BorderFill, Bullet, CellFillBrush, CharShape, FontFace, FontLang, HAlign, HeadingType,
    LangTuple, LineBreakWord, LineSpacing, LineSpacingType, LineType, Margin, Numbering, ParaHead,
    ParaShape, Style, VAlign,
};

/// The parsed header-side shape tables for an HWPX document.
///
/// All maps are keyed by the OWPML-assigned `id` attribute, which is
/// the same integer a section XML paragraph uses to reference the
/// shape. IDs are `u32` because the reference models them as `UINT`.
///
/// `font_faces` stays a `Vec` (not a map) because the key space is the
/// language enum, not an integer id, and there are only seven entries.
#[derive(Debug, Default, Clone)]
pub struct HeaderTables {
    pub char_shapes: HashMap<u32, CharShape>,
    pub para_shapes: HashMap<u32, ParaShape>,
    pub border_fills: HashMap<u32, BorderFill>,
    pub styles: HashMap<u32, Style>,
    pub bullets: HashMap<u32, Bullet>,
    pub numberings: HashMap<u32, Numbering>,
    pub font_faces: Vec<FontFace>,
}

impl HeaderTables {
    /// Find the [`FontFace`] for a given language.
    pub fn font_face(&self, lang: FontLang) -> Option<&FontFace> {
        self.font_faces.iter().find(|f| f.lang == lang)
    }

    /// Resolve a `CharShape.font_ref[lang]` to a face name in one call.
    pub fn font_name(&self, char_shape_id: u32, lang: FontLang) -> Option<&str> {
        let cs = self.char_shapes.get(&char_shape_id)?;
        cs.font_name(lang, &self.font_faces)
    }

    /// Find a [`Style`] by its `name` attribute (e.g. `"바탕글"`,
    /// `"테스트스타일"`). Names are not unique in general, so returns
    /// the first match. Prefer the id-keyed `styles.get` when you have
    /// the id.
    pub fn style_by_name(&self, name: &str) -> Option<&Style> {
        self.styles.values().find(|s| s.name == name)
    }
}
