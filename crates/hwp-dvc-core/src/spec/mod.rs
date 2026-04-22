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
    // ── Alignment (2001) ────────────────────────────────────────────────────
    /// `"horizontal"` — expected HAlign ordinal (0=JUSTIFY, 1=LEFT, 2=RIGHT,
    /// 3=CENTER, 4=DISTRIBUTE, 5=DISTRIBUTE_SPACE).
    #[serde(default)]
    pub horizontal: Option<i32>,

    // ── Margins (2002, 2003) ─────────────────────────────────────────────────
    /// `"margin-left"` — expected left margin in HWPUNIT.
    #[serde(rename = "margin-left", default)]
    pub margin_left: Option<i32>,
    /// `"margin-right"` — expected right margin in HWPUNIT.
    #[serde(rename = "margin-right", default)]
    pub margin_right: Option<i32>,

    // ── Indent / outdent (2004–2006) ─────────────────────────────────────────
    #[serde(default)]
    pub firstline: Option<bool>,
    #[serde(default)]
    pub indent: Option<i32>,
    #[serde(default)]
    pub outdent: Option<i32>,

    // ── Line spacing (2007–2008) ──────────────────────────────────────────────
    #[serde(default)]
    pub linespacing: Option<i32>,
    #[serde(default)]
    pub linespacingvalue: Option<i32>,

    // ── Paragraph spacing (2009–2011) ────────────────────────────────────────
    #[serde(rename = "spacing-paraup", default)]
    pub spacing_paraup: Option<i32>,
    #[serde(rename = "spacing-parabottom", default)]
    pub spacing_parabottom: Option<i32>,
    /// `"spacing-gridpaper"` — whether snap-to-grid must be enabled.
    #[serde(rename = "spacing-gridpaper", default)]
    pub spacing_gridpaper: Option<bool>,

    // ── Line break rules (2012–2014) ──────────────────────────────────────────
    /// `"linebreak-korean"` — Korean line-break mode; false = syllable,
    /// true = word-unit.
    #[serde(rename = "linebreak-korean", default)]
    pub linebreak_korean: Option<bool>,
    /// `"linebreak-english"` — Latin word line-break ordinal
    /// (0=KEEP_WORD, 1=BREAK_WORD).
    #[serde(rename = "linebreak-english", default)]
    pub linebreak_english: Option<i32>,
    /// `"linebreak-condense"` — condense value (25–100).
    #[serde(rename = "linebreak-condense", default)]
    pub linebreak_condense: Option<i32>,

    // ── Para heading type (2015–2016) ─────────────────────────────────────────
    /// `"paratype"` — heading type ordinal (0=NONE, 1=OUTLINE, 2=NUMBER,
    /// 3=BULLET).
    #[serde(default)]
    pub paratype: Option<i32>,
    /// `"paratype-value"` — heading id reference.
    #[serde(rename = "paratype-value", default)]
    pub paratype_value: Option<u32>,

    // ── Keep / break options (2017–2022) ──────────────────────────────────────
    #[serde(rename = "widow-orphan", default)]
    pub widow_orphan: Option<bool>,
    #[serde(rename = "keep-with-next", default)]
    pub keep_with_next: Option<bool>,
    #[serde(rename = "keep-lines-together", default)]
    pub keep_lines_together: Option<bool>,
    #[serde(rename = "pagebreak-before", default)]
    pub pagebreak_before: Option<bool>,
    #[serde(default)]
    pub fontlineheight: Option<bool>,
    #[serde(default)]
    pub linewrap: Option<bool>,

    // ── Autospace (2023–2024) ─────────────────────────────────────────────────
    #[serde(rename = "autospace-easian-eng", default)]
    pub autospace_easian_eng: Option<bool>,
    #[serde(rename = "autospace-easian-num", default)]
    pub autospace_easian_num: Option<bool>,

    // ── Vertical alignment (2025) ─────────────────────────────────────────────
    /// `"verticalalign"` — VAlign ordinal (0=BASELINE, 1=TOP, 2=CENTER,
    /// 3=BOTTOM).
    #[serde(default)]
    pub verticalalign: Option<i32>,

    // ── Tab definitions (2026–2032) ───────────────────────────────────────────
    // TODO: Full per-tab field validation (tabtypes array, tabtype/tabshape/
    // tabposition sub-fields) requires a TabDefinition table in HeaderTables
    // which is not yet parsed from header.xml. These spec fields are recognised
    // by the deserialiser but the checker currently emits no errors for them.
    #[serde(default)]
    pub tabtypes: Option<serde_json::Value>,
    #[serde(rename = "autotab-intent", default)]
    pub autotab_intent: Option<bool>,
    #[serde(rename = "autotab-pararightend", default)]
    pub autotab_pararightend: Option<bool>,
    #[serde(default)]
    pub basetabspace: Option<i32>,

    // ── Paragraph border (2033–2037) ──────────────────────────────────────────
    // TODO: Full border comparison requires resolving border_fill_id_ref to a
    // BorderFill record and comparing type/size/color per edge. The BorderFill
    // table is available in HeaderTables but linking it via border_fill_id_ref
    // and splitting per-position are not yet wired for paragraph shapes.
    #[serde(default)]
    pub border: Option<serde_json::Value>,

    // ── Background (2038–2040) ────────────────────────────────────────────────
    // TODO: Background color and pattern fields are not yet decoded from the
    // paragraph's associated BorderFill record. Deferred until the paragraph
    // BorderFill linkage is established.
    #[serde(rename = "bg-color", default)]
    pub bg_color: Option<u32>,
    #[serde(rename = "bg-pattoncolor", default)]
    pub bg_pattoncolor: Option<u32>,
    #[serde(rename = "bg-pattontype", default)]
    pub bg_pattontype: Option<i32>,

    // ── Border spacing flags (2041–2044) ──────────────────────────────────────
    #[serde(rename = "spacing-left", default)]
    pub spacing_left: Option<bool>,
    #[serde(rename = "spacing-right", default)]
    pub spacing_right: Option<bool>,
    #[serde(rename = "spacing-top", default)]
    pub spacing_top: Option<bool>,
    #[serde(rename = "spacing-bottom", default)]
    pub spacing_bottom: Option<bool>,

    // ── Ignore margin (2045) ──────────────────────────────────────────────────
    #[serde(rename = "spacing-ignore", default)]
    pub spacing_ignore: Option<bool>,
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
