//! Validation spec ("CheckList") — parsed from a DVC JSON document.
//!
//! Mirrors `CheckList` and the `C*` classes in the reference C++
//! implementation. Each category is represented by an explicit struct
//! so that the spec remains self-documenting.

use serde::{Deserialize, Serialize};

/// Top-level DVC spec.
///
/// Every field is optional: a spec only needs to define the categories
/// that should be validated. Missing categories are skipped.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct DvcSpec {
    #[serde(default)]
    pub charshape: Option<CharShapeSpec>,
    #[serde(default)]
    pub parashape: Option<ParaShapeSpec>,
    #[serde(default)]
    pub table: Option<TableSpec>,
    #[serde(default)]
    pub specialcharacter: Option<SpecialCharacterSpec>,
    #[serde(default)]
    pub outlineshape: Option<OutlineShapeSpec>,
    #[serde(default)]
    pub bullet: Option<BulletSpec>,
    #[serde(default)]
    pub paranumbullet: Option<ParaNumBulletSpec>,
    #[serde(default)]
    pub style: Option<StyleSpec>,
    #[serde(default)]
    pub hyperlink: Option<HyperlinkSpec>,
    #[serde(rename = "macro", default)]
    pub macro_: Option<MacroSpec>,
}

/// Spec for the background-border sub-check within CharShape.
///
/// Maps to `JID_CHAR_SHAPE_BG_BORDER_*` (1032–1036) in `JsonModel.h`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CharShapeBorderSpec {
    pub position: Option<u32>,
    pub bordertype: Option<u32>,
    pub size: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CharShapeSpec {
    // ── existing fields (1000-range) ─────────────────────────────────────
    #[serde(default)]
    pub langtype: Option<String>,
    #[serde(default)]
    pub font: Vec<String>,
    #[serde(default)]
    pub ratio: Option<i32>,
    #[serde(default)]
    pub spacing: Option<i32>,

    // ── size group (1001, 1005, 1006, 1030) ──────────────────────────────
    /// Font size in 0.1pt units (`JID_CHAR_SHAPE_FONTSIZE = 1001`).
    #[serde(default)]
    pub fontsize: Option<u32>,
    /// Relative size percentage (`JID_CHAR_SHAPE_RSIZE = 1005`).
    #[serde(default)]
    pub rsize: Option<u32>,
    /// Character position in 0.1pt units (`JID_CHAR_SHAPE_POSITION = 1006`).
    #[serde(default)]
    pub position: Option<i32>,
    /// Font size in points (`JID_CHAR_SHAPE_POINT = 1030`).
    #[serde(default)]
    pub point: Option<f64>,

    // ── text decoration toggles (1009–1018) ───────────────────────────────
    /// Bold flag (`JID_CHAR_SHAPE_BOLD = 1009`).
    #[serde(default)]
    pub bold: Option<bool>,
    /// Italic flag (`JID_CHAR_SHAPE_ITALIC = 1010`).
    #[serde(default)]
    pub italic: Option<bool>,
    /// Underline flag (`JID_CHAR_SHAPE_UNDERLINE = 1011`).
    #[serde(default)]
    pub underline: Option<bool>,
    /// Strikeout flag (`JID_CHAR_SHAPE_STRIKEOUT = 1012`).
    #[serde(default)]
    pub strikeout: Option<bool>,
    /// Outline flag (`JID_CHAR_SHAPE_OUTLINE = 1013`).
    #[serde(default)]
    pub outline: Option<bool>,
    /// Emboss flag (`JID_CHAR_SHAPE_EMBOSS = 1014`).
    #[serde(default)]
    pub emboss: Option<bool>,
    /// Engrave flag (`JID_CHAR_SHAPE_ENGRAVE = 1015`).
    #[serde(default)]
    pub engrave: Option<bool>,
    /// Shadow flag (`JID_CHAR_SHAPE_SHADOW = 1016`).
    #[serde(default)]
    pub shadow: Option<bool>,
    /// Superscript flag (`JID_CHAR_SHAPE_SUPSCRIPT = 1017`).
    #[serde(default)]
    pub supscript: Option<bool>,
    /// Subscript flag (`JID_CHAR_SHAPE_SUBSCRIPT = 1018`).
    #[serde(default)]
    pub subscript: Option<bool>,

    // ── shadow detail (1019–1022) ─────────────────────────────────────────
    /// Shadow type string (`JID_CHAR_SHAPE_SHADOWTYPE = 1019`).
    #[serde(default)]
    pub shadowtype: Option<String>,
    /// Shadow X offset (`JID_CHAR_SHAPE_SHADOW_X = 1020`).
    #[serde(rename = "shadow-x", default)]
    pub shadow_x: Option<i32>,
    /// Shadow Y offset (`JID_CHAR_SHAPE_SHADOW_Y = 1021`).
    #[serde(rename = "shadow-y", default)]
    pub shadow_y: Option<i32>,
    /// Shadow color hex string (`JID_CHAR_SHAPE_SHADOW_COLOR = 1022`).
    #[serde(rename = "shadow-color", default)]
    pub shadow_color: Option<String>,

    // ── underline detail (1023–1025) ──────────────────────────────────────
    /// Underline position string (`JID_CHAR_SHAPE_UNDERLINE_POSITION = 1023`).
    #[serde(rename = "underline-position", default)]
    pub underline_position: Option<String>,
    /// Underline shape string (`JID_CHAR_SHAPE_UNDERLINE_SHAPE = 1024`).
    #[serde(rename = "underline-shape", default)]
    pub underline_shape: Option<String>,
    /// Underline color hex string (`JID_CHAR_SHAPE_UNDERLINE_COLOR = 1025`).
    #[serde(rename = "underline-color", default)]
    pub underline_color: Option<String>,

    // ── strikeout detail (1026–1027) ──────────────────────────────────────
    /// Strikeout shape string (`JID_CHAR_SHAPE_STRIKEOUT_SHAPE = 1026`).
    #[serde(rename = "strikeout-shape", default)]
    pub strikeout_shape: Option<String>,
    /// Strikeout color hex string (`JID_CHAR_SHAPE_STRIKEOUT_COLOR = 1027`).
    #[serde(rename = "strikeout-color", default)]
    pub strikeout_color: Option<String>,

    // ── outline detail (1028) ─────────────────────────────────────────────
    /// Outline type string (`JID_CHAR_SHAPE_OUTLINETYPE = 1028`).
    #[serde(default)]
    pub outlinetype: Option<String>,

    // ── misc (1029, 1031) ─────────────────────────────────────────────────
    /// Empty-space flag (`JID_CHAR_SHAPE_EMPTYSPACE = 1029`).
    #[serde(default)]
    pub emptyspace: Option<bool>,
    /// Kerning flag (`JID_CHAR_SHAPE_KERNING = 1031`).
    #[serde(default)]
    pub kerning: Option<bool>,

    // ── border (1032–1036) ────────────────────────────────────────────────
    /// Background border object (`JID_CHAR_SHAPE_BG_BORDER = 1032`).
    #[serde(default)]
    pub border: Option<CharShapeBorderSpec>,

    // ── background (1037–1039) ────────────────────────────────────────────
    /// Background fill color hex string (`JID_CHAR_SHAPE_BG_COLOR = 1037`).
    #[serde(rename = "bg-color", default)]
    pub bg_color: Option<String>,
    /// Background pattern color hex string (`JID_CHAR_SHAPE_BG_PATTONCOLOR = 1038`).
    #[serde(rename = "bg-pattoncolor", default)]
    pub bg_pattoncolor: Option<String>,
    /// Background pattern type string (`JID_CHAR_SHAPE_BG_PATTONTYPE = 1039`).
    #[serde(rename = "bg-pattontype", default)]
    pub bg_pattontype: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParaShapeSpec {
    #[serde(rename = "spacing-paraup", default)]
    pub spacing_paraup: Option<i32>,
    #[serde(rename = "spacing-parabottom", default)]
    pub spacing_parabottom: Option<i32>,
    #[serde(default)]
    pub linespacing: Option<i32>,
    #[serde(default)]
    pub linespacingvalue: Option<i32>,
    #[serde(default)]
    pub indent: Option<i32>,
    #[serde(default)]
    pub outdent: Option<i32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TableSpec {
    #[serde(default)]
    pub border: Vec<BorderSpec>,
    #[serde(rename = "treatAsChar", default)]
    pub treat_as_char: Option<bool>,
    #[serde(rename = "table-in-table", default)]
    pub table_in_table: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BorderSpec {
    pub position: u32,
    pub bordertype: u32,
    pub size: f64,
    pub color: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpecialCharacterSpec {
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OutlineShapeSpec {
    #[serde(default)]
    pub leveltype: Vec<LevelType>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BulletSpec {
    #[serde(default)]
    pub bulletshapes: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParaNumBulletSpec {
    #[serde(default)]
    pub leveltype: Vec<LevelType>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LevelType {
    pub level: u32,
    #[serde(default)]
    pub numbertype: Option<String>,
    pub numbershape: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StyleSpec {
    pub permission: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HyperlinkSpec {
    pub permission: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MacroSpec {
    pub permission: bool,
}

impl DvcSpec {
    pub fn from_json_str(s: &str) -> crate::DvcResult<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }

    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> crate::DvcResult<Self> {
        let bytes = std::fs::read(path)?;
        let spec: Self = serde_json::from_slice(&bytes)?;
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_spec_parses() {
        let spec = DvcSpec::from_json_str("{}").unwrap();
        assert!(spec.charshape.is_none());
    }

    #[test]
    fn charshape_spec_parses() {
        let s = r#"{ "charshape": { "langtype": "대표", "font": ["바탕"], "ratio": 100 } }"#;
        let spec = DvcSpec::from_json_str(s).unwrap();
        let cs = spec.charshape.unwrap();
        assert_eq!(cs.langtype.as_deref(), Some("대표"));
        assert_eq!(cs.font, vec!["바탕".to_string()]);
        assert_eq!(cs.ratio, Some(100));
    }
}
