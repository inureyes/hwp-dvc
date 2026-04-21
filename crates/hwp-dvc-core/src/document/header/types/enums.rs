//! Attribute-decoding enums and the small `LangTuple<T>` helper.
//!
//! Each enum carries a `parse(&str) -> Self` (or `Option<Self>` for
//! `FontLang`) that maps OWPML's all-caps tokens to variants. Unknown
//! tokens map to `::Other` rather than panicking — HWPX has shipped
//! multiple minor revisions with new tokens.

use serde::{Deserialize, Serialize};

/// The seven OWPML font language slots, in declaration order.
///
/// Each `<hh:charPr>` carries a per-language tuple (`hangul`, `latin`,
/// `hanja`, `japanese`, `other`, `symbol`, `user`); the reference
/// collapses this to `Hangul` only. We preserve all seven so that the
/// later validators (`CheckCharShape`) can report which slot violated a
/// rule.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontLang {
    #[default]
    Hangul,
    Latin,
    Hanja,
    Japanese,
    Other,
    Symbol,
    User,
}

impl FontLang {
    pub const ALL: [FontLang; 7] = [
        FontLang::Hangul,
        FontLang::Latin,
        FontLang::Hanja,
        FontLang::Japanese,
        FontLang::Other,
        FontLang::Symbol,
        FontLang::User,
    ];

    /// Parse the OWPML `lang=".."` attribute of `<hh:fontface>`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "HANGUL" => Some(Self::Hangul),
            "LATIN" => Some(Self::Latin),
            "HANJA" => Some(Self::Hanja),
            "JAPANESE" => Some(Self::Japanese),
            "OTHER" => Some(Self::Other),
            "SYMBOL" => Some(Self::Symbol),
            "USER" => Some(Self::User),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Hangul => 0,
            Self::Latin => 1,
            Self::Hanja => 2,
            Self::Japanese => 3,
            Self::Other => 4,
            Self::Symbol => 5,
            Self::User => 6,
        }
    }
}

/// Per-language `u32` / `i32` tuple. Default is all zeros.
///
/// Parsed from attributes like
/// `<hh:fontRef hangul="1" latin="0" .../>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LangTuple<T: Copy + Default> {
    pub values: [T; 7],
}

impl<T: Copy + Default> LangTuple<T> {
    pub fn get(&self, lang: FontLang) -> T {
        self.values[lang.index()]
    }

    pub fn set(&mut self, lang: FontLang, v: T) {
        self.values[lang.index()] = v;
    }
}

/// Line-type for cell borders (`<hh:leftBorder type="..." .../>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum LineType {
    #[default]
    None,
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
    LongDash,
    Circle,
    DoubleSlim,
    SlimThick,
    ThickSlim,
    SlimThickSlim,
    Wave,
    DoubleWave,
    ThickThreeD,
    ThickThreeDInset,
    ThinThreeD,
    ThinThreeDInset,
    /// Unrecognized value carried forward for debugging.
    Other,
}

impl LineType {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "NONE" => Self::None,
            "SOLID" => Self::Solid,
            "DASH" => Self::Dash,
            "DOT" => Self::Dot,
            "DASH_DOT" => Self::DashDot,
            "DASH_DOT_DOT" => Self::DashDotDot,
            "LONG_DASH" => Self::LongDash,
            "CIRCLE" => Self::Circle,
            "DOUBLE_SLIM" => Self::DoubleSlim,
            "SLIM_THICK" => Self::SlimThick,
            "THICK_SLIM" => Self::ThickSlim,
            "SLIM_THICK_SLIM" => Self::SlimThickSlim,
            "WAVE" => Self::Wave,
            "DOUBLE_WAVE" => Self::DoubleWave,
            "THICK_3D" => Self::ThickThreeD,
            "THICK_3D_INSET" => Self::ThickThreeDInset,
            "THIN_3D" => Self::ThinThreeD,
            "THIN_3D_INSET" => Self::ThinThreeDInset,
            _ => Self::Other,
        }
    }
}

/// Horizontal alignment (`<hh:align horizontal="..."/>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum HAlign {
    #[default]
    Justify,
    Left,
    Right,
    Center,
    Distribute,
    DistributeSpace,
    Other,
}

impl HAlign {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "JUSTIFY" => Self::Justify,
            "LEFT" => Self::Left,
            "RIGHT" => Self::Right,
            "CENTER" => Self::Center,
            "DISTRIBUTE" => Self::Distribute,
            "DISTRIBUTE_SPACE" => Self::DistributeSpace,
            _ => Self::Other,
        }
    }
}

/// Vertical alignment (`<hh:align vertical="..."/>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum VAlign {
    #[default]
    Baseline,
    Top,
    Center,
    Bottom,
    Other,
}

impl VAlign {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "BASELINE" => Self::Baseline,
            "TOP" => Self::Top,
            "CENTER" => Self::Center,
            "BOTTOM" => Self::Bottom,
            _ => Self::Other,
        }
    }
}

/// Heading type (`<hh:heading type="..."/>`): outline / number / bullet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum HeadingType {
    #[default]
    None,
    Outline,
    Number,
    Bullet,
    Other,
}

impl HeadingType {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "NONE" => Self::None,
            "OUTLINE" => Self::Outline,
            "NUMBER" => Self::Number,
            "BULLET" => Self::Bullet,
            _ => Self::Other,
        }
    }
}

/// Line-break behavior for latin and non-latin words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum LineBreakWord {
    #[default]
    KeepWord,
    BreakWord,
    Other,
}

impl LineBreakWord {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "KEEP_WORD" => Self::KeepWord,
            "BREAK_WORD" => Self::BreakWord,
            _ => Self::Other,
        }
    }
}

/// Spacing type for `<hh:lineSpacing type="..."/>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum LineSpacingType {
    #[default]
    Percent,
    Fixed,
    BetweenLines,
    Minimum,
    Other,
}

impl LineSpacingType {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "PERCENT" => Self::Percent,
            "FIXED" => Self::Fixed,
            "BETWEEN_LINES" => Self::BetweenLines,
            "AT_LEAST" | "MINIMUM" => Self::Minimum,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_lang_parses_all_seven() {
        assert_eq!(FontLang::parse("HANGUL"), Some(FontLang::Hangul));
        assert_eq!(FontLang::parse("LATIN"), Some(FontLang::Latin));
        assert_eq!(FontLang::parse("HANJA"), Some(FontLang::Hanja));
        assert_eq!(FontLang::parse("JAPANESE"), Some(FontLang::Japanese));
        assert_eq!(FontLang::parse("OTHER"), Some(FontLang::Other));
        assert_eq!(FontLang::parse("SYMBOL"), Some(FontLang::Symbol));
        assert_eq!(FontLang::parse("USER"), Some(FontLang::User));
        assert_eq!(FontLang::parse("unknown"), None);
        assert_eq!(FontLang::parse(""), None);
    }

    #[test]
    fn font_lang_indices_are_contiguous() {
        for (i, &lang) in FontLang::ALL.iter().enumerate() {
            assert_eq!(lang.index(), i);
        }
    }

    #[test]
    fn line_type_parses_common_values() {
        assert_eq!(LineType::parse("NONE"), LineType::None);
        assert_eq!(LineType::parse("SOLID"), LineType::Solid);
        assert_eq!(LineType::parse("DASH"), LineType::Dash);
        assert_eq!(LineType::parse("SLIM_THICK_SLIM"), LineType::SlimThickSlim);
        assert_eq!(LineType::parse("THICK_3D"), LineType::ThickThreeD);
        assert_eq!(LineType::parse("SOMETHING_ELSE"), LineType::Other);
    }
}
