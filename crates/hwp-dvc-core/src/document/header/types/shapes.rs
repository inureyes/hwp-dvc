//! Struct records for each OWPML header table.

use std::collections::HashMap;

use super::enums::{
    FontLang, HAlign, HeadingType, LangTuple, LineBreakWord, LineSpacingType, LineType, VAlign,
};

/// Resolved font table: for each of the seven language slots, an
/// `id -> face` mapping. The `id` matches the value of
/// `charPr.font_ref[lang]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontFace {
    pub lang: FontLang,
    pub fonts: HashMap<u32, String>,
}

/// A character-shape record (`<hh:charPr>`).
///
/// Fields track what Phase 2 `CheckCharShape` needs: font references
/// per language, ratio/spacing/rel-size/offset per language,
/// height (0.1 pt units), text/shade colors, and the boolean emphasis
/// flags (bold/italic/…).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharShape {
    pub id: u32,
    pub height: u32,
    pub text_color: String,
    pub shade_color: String,
    pub use_font_space: bool,
    pub use_kerning: bool,
    pub sym_mark: String,
    pub border_fill_id_ref: u32,

    pub font_ref: LangTuple<u32>,
    pub ratio: LangTuple<u32>,
    pub spacing: LangTuple<i32>,
    pub rel_sz: LangTuple<u32>,
    pub offset: LangTuple<i32>,

    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
    pub outline: bool,
    pub emboss: bool,
    pub engrave: bool,
    pub shadow: bool,
    pub supscript: bool,
    pub subscript: bool,
}

impl CharShape {
    /// Resolve the font name of this char-shape for a given language,
    /// consulting the supplied font-face table.
    pub fn font_name<'a>(&self, lang: FontLang, faces: &'a [FontFace]) -> Option<&'a str> {
        let id = self.font_ref.get(lang);
        faces
            .iter()
            .find(|f| f.lang == lang)
            .and_then(|f| f.fonts.get(&id).map(String::as_str))
    }

    /// Collect font names across all seven language slots, deduplicated
    /// while preserving first-seen order. Useful for the Phase 2 font
    /// allow-list check as well as for the fixture test that asserts
    /// the default font is `함초롬바탕`.
    pub fn font_names(&self, faces: &[FontFace]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for &lang in &FontLang::ALL {
            if let Some(name) = self.font_name(lang, faces) {
                if !out.iter().any(|n| n == name) {
                    out.push(name.to_string());
                }
            }
        }
        out
    }
}

/// A single border edge of a [`BorderFill`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Border {
    pub line_type: LineType,
    /// The `width` attribute is emitted as e.g. `"0.12 mm"`; we strip
    /// the unit suffix and keep the mm value as `f32` so that validators
    /// can compare against the spec-supplied floating-point threshold.
    pub width_mm: f32,
    pub color: String,
}

/// A `<hh:borderFill>` record. Only the fields the validators need are
/// decoded; `<hc:fillBrush>` subtrees are tracked as "present or not"
/// because `CheckTable` only needs to know that a fill was supplied, not
/// the exact brush.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BorderFill {
    pub id: u32,
    pub three_d: bool,
    pub shadow: bool,
    pub center_line: String,
    pub break_cell_separate_line: bool,

    pub left: Border,
    pub right: Border,
    pub top: Border,
    pub bottom: Border,
    pub diagonal: Border,

    pub has_fill_brush: bool,
}

impl BorderFill {
    /// True iff left, right, top, bottom are all `Solid`.
    pub fn four_sides_solid(&self) -> bool {
        self.left.line_type == LineType::Solid
            && self.right.line_type == LineType::Solid
            && self.top.line_type == LineType::Solid
            && self.bottom.line_type == LineType::Solid
    }
}

/// `<hh:lineSpacing type="..." value=".." unit=".."/>`.
///
/// Not `Copy` because `unit` is a `String`; the header is parsed once
/// per document and `LineSpacing` values are cloned only when a
/// validator wants to surface one in an error message, so the cost is
/// negligible.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineSpacing {
    pub type_: LineSpacingType,
    pub value: i32,
    pub unit: String,
}

/// Paragraph margin in HWPUNIT (1 HWPUNIT = 1/7200 inch), matching the
/// reference's `Margin` struct but omitting the per-case / default
/// `<hp:switch>` wrapper. We decode the first (`hp:case`) branch —
/// which is what HWPX emitters consistently populate — and leave
/// alternate-namespace handling to a later pass when a concrete test
/// case exposes a divergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Margin {
    pub indent: i32,
    pub left: i32,
    pub right: i32,
    pub prev: i32,
    pub next: i32,
}

/// A paragraph-shape record (`<hh:paraPr>`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParaShape {
    pub id: u32,
    pub tab_pr_id_ref: u32,
    pub condense: u32,
    pub font_line_height: bool,
    pub snap_to_grid: bool,
    pub suppress_line_numbers: bool,
    pub checked: bool,

    pub h_align: HAlign,
    pub v_align: VAlign,

    pub heading_type: HeadingType,
    pub heading_id_ref: u32,
    pub heading_level: u32,

    pub break_latin_word: LineBreakWord,
    pub break_non_latin_word: LineBreakWord,

    pub widow_orphan: bool,
    pub keep_with_next: bool,
    pub keep_lines: bool,
    pub page_break_before: bool,
    pub line_wrap: bool,

    pub auto_spacing_eng: bool,
    pub auto_spacing_num: bool,

    pub margin: Margin,
    pub line_spacing: LineSpacing,

    pub border_fill_id_ref: u32,
    pub border_offset_left: i32,
    pub border_offset_right: i32,
    pub border_offset_top: i32,
    pub border_offset_bottom: i32,
    pub connect: bool,
    pub ignore_margin: bool,
}

/// A `<hh:style>` record. Both `PARA` and `CHAR` style types live in the
/// same table (keyed by `id`); `style_type` distinguishes them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Style {
    pub id: u32,
    pub style_type: String,
    pub name: String,
    pub eng_name: String,
    pub para_pr_id_ref: u32,
    pub char_pr_id_ref: u32,
    pub next_style_id_ref: u32,
    pub lang_id: u32,
    pub lock_form: bool,
}

/// A `<hh:bullet>` record. The character payload is the literal bullet
/// glyph (e.g. `"□"`, `"○"`, …). Image bullets are not decoded here but
/// their `use_image=1` flag is preserved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bullet {
    pub id: u32,
    pub char: String,
    pub use_image: bool,
}

/// A numbering template level (`<hh:paraHead>`). Mirrors the reference's
/// `ParaHead` but keeps numeric enums as raw strings, because the
/// Phase 2 validators compare these to spec strings unmodified.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParaHead {
    pub start: u32,
    pub level: u32,
    pub align: String,
    pub use_inst_width: bool,
    pub auto_indent: bool,
    pub width_adjust: bool,
    pub text_offset_type: String,
    pub text_offset: u32,
    pub num_format: String,
    pub char_pr_id_ref: u32,
    pub checkable: bool,
    pub num_format_text: String,
}

/// A `<hh:numbering>` record: one template per level (up to 10 in
/// practice). Levels are not guaranteed contiguous in the XML, so we
/// keep them in the declaration order they appeared.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Numbering {
    pub id: u32,
    pub start: u32,
    pub para_heads: Vec<ParaHead>,
}

impl Numbering {
    pub fn by_level(&self, level: u32) -> Option<&ParaHead> {
        self.para_heads.iter().find(|p| p.level == level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_sides_solid_detects_all_solid_correctly() {
        let mut bf = BorderFill::default();
        assert!(!bf.four_sides_solid(), "default is None, not Solid");
        bf.left.line_type = LineType::Solid;
        bf.right.line_type = LineType::Solid;
        bf.top.line_type = LineType::Solid;
        assert!(!bf.four_sides_solid(), "only three sides Solid");
        bf.bottom.line_type = LineType::Solid;
        assert!(bf.four_sides_solid(), "all four sides Solid");
        bf.bottom.line_type = LineType::Dash;
        assert!(!bf.four_sides_solid(), "bottom became Dash");
    }

    #[test]
    fn char_shape_font_names_dedups_while_preserving_order() {
        let mut face = FontFace {
            lang: FontLang::Hangul,
            fonts: HashMap::new(),
        };
        face.fonts.insert(0, "Hamcho".into());
        face.fonts.insert(1, "Nanum".into());
        let faces = vec![
            face,
            FontFace {
                lang: FontLang::Latin,
                fonts: {
                    let mut m = HashMap::new();
                    m.insert(0, "Hamcho".into());
                    m
                },
            },
        ];
        let cs = CharShape {
            id: 0,
            font_ref: {
                let mut t = LangTuple::<u32>::default();
                t.set(FontLang::Hangul, 1);
                t.set(FontLang::Latin, 0);
                t
            },
            ..Default::default()
        };
        let names = cs.font_names(&faces);
        // Hangul→Nanum, Latin→Hamcho, rest default to id=0 in Hangul
        // face which maps to Hamcho — so Hamcho should appear exactly
        // once.
        assert_eq!(
            names.iter().filter(|n| n.as_str() == "Hamcho").count(),
            1,
            "Hamcho must be deduplicated"
        );
        assert!(names.contains(&"Nanum".to_string()));
    }
}
