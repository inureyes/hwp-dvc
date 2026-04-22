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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CharShapeSpec {
    #[serde(default)]
    pub langtype: Option<String>,
    #[serde(default)]
    pub font: Vec<String>,
    #[serde(default)]
    pub ratio: Option<i32>,
    #[serde(default)]
    pub spacing: Option<i32>,
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

/// Table validation spec — mirrors the `CTable` category of the
/// reference DVC.
///
/// Every field is optional: only the fields actually mentioned in the
/// spec JSON are validated. Absent fields are skipped by the checker.
/// JSON keys intentionally match the reference DVC spec format (see
/// `references/dvc/Source/JsonModel.h` and
/// `crates/hwp-dvc-core/tests/fixtures/specs/hancom_full.json`) so
/// existing specs load unchanged.
///
/// Range-valued fields (sizes, margins, offsets) accept either a bare
/// number (interpreted as `min == max`) or an explicit
/// `{ "min": x, "max": y }` object. See [`IntRange`].
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TableSpec {
    // ── Size ────────────────────────────────────────────────────────────
    /// `size-width` — allowed range for the `width` attribute of
    /// `<hp:sz>` (`JID_TABLE_SIZEWIDTH`).
    #[serde(rename = "size-width", default)]
    pub size_width: Option<IntRange>,
    /// `size-height` — allowed range for the `height` attribute of
    /// `<hp:sz>` (`JID_TABLE_SIZEHEIGHT`).
    #[serde(rename = "size-height", default)]
    pub size_height: Option<IntRange>,
    /// `fixed` — required value for the `protect` attribute of
    /// `<hp:sz>` (`JID_TABLE_SIZEFIXED`).
    #[serde(default)]
    pub fixed: Option<bool>,

    // ── treatAsChar ─────────────────────────────────────────────────────
    /// `treatAsChar` — required value for `<hp:pos treatAsChar>`
    /// (`JID_TABLE_TREATASCHAR`). Reference semantics: when the spec
    /// demands `true` the document attribute must also be `true`; a
    /// spec value of `false` disables the check.
    #[serde(rename = "treatAsChar", default)]
    pub treat_as_char: Option<bool>,

    // ── Position / text wrap ────────────────────────────────────────────
    /// `pos` — text-wrap type (`JID_TABLE_POS`). Integer enum:
    /// `0=WRAP_SQUARE`, `1=TOP_AND_BOTTOM`, `2=BEHIND_TEXT`,
    /// `3=IN_FRONT_OF_TEXT`.
    #[serde(default)]
    pub pos: Option<u32>,
    /// `textpos` — text-flow type (`JID_TABLE_TEXTPOS`). Integer enum:
    /// `0=BOTH_SIDES`, `1=LEFT_ONLY`, `2=RIGHT_ONLY`, `3=LARGEST_ONLY`.
    #[serde(default)]
    pub textpos: Option<u32>,

    // ── Horizontal alignment ───────────────────────────────────────────
    /// `horizontal-type` (`JID_TABLE_HTYPE`) — horz-rel-to enum
    /// `0=PAPER`, `1=PAGE`, `2=COLUMN`, `3=PARA`.
    #[serde(rename = "horizontal-type", default)]
    pub horizontal_type: Option<u32>,
    /// `horizontal-direction` (`JID_TABLE_HDIRECTION`) — horz-align
    /// enum `0=LEFT`, `1=CENTER`, `2=RIGHT`, `3=INSIDE`, `4=OUTSIDE`.
    #[serde(rename = "horizontal-direction", default)]
    pub horizontal_direction: Option<u32>,
    /// `horizontal-value` (`JID_TABLE_HVALUE`) — horizontal offset
    /// allowed range (typically `-1000..=1000`).
    #[serde(rename = "horizontal-value", default)]
    pub horizontal_value: Option<IntRange>,

    // ── Vertical alignment ─────────────────────────────────────────────
    /// `vertical-type` (`JID_TABLE_VTYPE`) — vert-rel-to enum
    /// `0=PAPER`, `1=PAGE`, `2=PARA`.
    #[serde(rename = "vertical-type", default)]
    pub vertical_type: Option<u32>,
    /// `vertical-direction` (`JID_TABLE_VDIRECTION`) — vert-align enum
    /// `0=TOP`, `1=CENTER`, `2=BOTTOM`.
    #[serde(rename = "vertical-direction", default)]
    pub vertical_direction: Option<u32>,
    /// `vertical-value` (`JID_TABLE_VVALUE`) — vertical offset
    /// allowed range (typically `-1000..=1000`).
    #[serde(rename = "vertical-value", default)]
    pub vertical_value: Option<IntRange>,

    // ── Flow flags ─────────────────────────────────────────────────────
    /// `soflowwithtext` (`JID_TABLE_SOFLOWWITHTEXT`).
    #[serde(default)]
    pub soflowwithtext: Option<bool>,
    /// `soallowoverlap` (`JID_TABLE_SOALLOWOVERLAP`).
    #[serde(default)]
    pub soallowoverlap: Option<bool>,
    /// `soholdanchorobj` (`JID_TABLE_SOHOLDANCHOROBJ`).
    #[serde(default)]
    pub soholdanchorobj: Option<bool>,
    /// `parallel` (`JID_TABLE_PARALLEL`) — maps to OWPML's
    /// `affectLSpacing` flag.
    #[serde(default)]
    pub parallel: Option<bool>,

    // ── Rotation & gradient offsets ────────────────────────────────────
    /// `rotation` (`JID_TABLE_ROTATION`) — allowed signed range in
    /// hundredths of a degree.
    #[serde(default)]
    pub rotation: Option<IntRange>,
    /// `gradientH` (`JID_TABLE_GRADIENT_H`).
    #[serde(rename = "gradientH", default)]
    pub gradient_h: Option<IntRange>,
    /// `gradientV` (`JID_TABLE_GRADIENT_V`).
    #[serde(rename = "gradientV", default)]
    pub gradient_v: Option<IntRange>,

    // ── Number / protect ───────────────────────────────────────────────
    /// `numbertype` (`JID_TABLE_NUMVERTYPE`) — integer enum
    /// `0=NONE`, `1=PICTURE`, `2=TABLE`, `3=FORMULA`.
    #[serde(default)]
    pub numbertype: Option<u32>,
    /// `objprotect` (`JID_TABLE_OBJPROTECT`) — maps to OWPML
    /// `noAdjust` attribute.
    #[serde(default)]
    pub objprotect: Option<bool>,

    // ── Margins ────────────────────────────────────────────────────────
    /// `margin-left` (`JID_TABLE_MARGIN_LEFT`).
    #[serde(rename = "margin-left", default)]
    pub margin_left: Option<IntRange>,
    /// `margin-right` (`JID_TABLE_MARGIN_RIGHT`).
    #[serde(rename = "margin-right", default)]
    pub margin_right: Option<IntRange>,
    /// `margin-top` (`JID_TABLE_MARGIN_TOP`).
    #[serde(rename = "margin-top", default)]
    pub margin_top: Option<IntRange>,
    /// `margin-bottom` (`JID_TABLE_MARGIN_BOTTOM`).
    #[serde(rename = "margin-bottom", default)]
    pub margin_bottom: Option<IntRange>,

    // ── Caption ────────────────────────────────────────────────────────
    /// `caption-position` (`JID_TABLE_CAPTION_POSITION`). Integer enum
    /// `0=LEFTTOP, 1=TOP, 2=RIGHTTOP, 3=LEFT, 4=NONE, 5=RIGHT,
    ///  6=LEFTBOTTOM, 7=BOTTOM, 8=RIGHTBOTTOM`.
    #[serde(rename = "caption-position", default)]
    pub caption_position: Option<u32>,
    /// `caption-size` (`JID_TABLE_CAPTION_SIZE`).
    #[serde(rename = "caption-size", default)]
    pub caption_size: Option<IntRange>,
    /// `caption-spacing` (`JID_TABLE_CAPTION_SPACING`).
    #[serde(rename = "caption-spacing", default)]
    pub caption_spacing: Option<IntRange>,
    /// `caption-socapfullsize` (`JID_TABLE_CAPTION_SOCAPFULLSIZE`).
    #[serde(rename = "caption-socapfullsize", default)]
    pub caption_socapfullsize: Option<bool>,
    /// `caption-linewrap` (`JID_TABLE_CAPTION_LINEWRAP`).
    #[serde(rename = "caption-linewrap", default)]
    pub caption_linewrap: Option<bool>,

    // ── Borders ────────────────────────────────────────────────────────
    /// `border` — per-position line rules
    /// (`JID_TABLE_BORDER_TYPE`/`_SIZE`/`_COLOR`).
    #[serde(default)]
    pub border: Vec<BorderSpec>,
    /// `border-cellspacing` (`JID_TABLE_BORDER_CELLSPACING`).
    #[serde(rename = "border-cellspacing", default)]
    pub border_cellspacing: Option<IntRange>,

    // ── table-in-table ─────────────────────────────────────────────────
    /// `table-in-table` (`JID_TABLE_TABLE_IN_TABLE`).
    #[serde(rename = "table-in-table", default)]
    pub table_in_table: Option<bool>,
}

/// A closed integer range `[min, max]` used by range-valued spec
/// fields (sizes, margins, offsets, rotation, caption sizing).
///
/// Serialized as either a bare integer (shorthand for `min == max`)
/// or an explicit `{ "min": a, "max": b }` object, matching the
/// reference C++ parser's behaviour in `CTable::parsingElement`. The
/// deserializer accepts both forms transparently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct IntRange {
    pub min: i64,
    pub max: i64,
}

impl IntRange {
    /// Return `true` when `value` lies inside `[min, max]` inclusive.
    #[must_use]
    pub fn contains(&self, value: i64) -> bool {
        value >= self.min && value <= self.max
    }
}

impl Default for IntRange {
    fn default() -> Self {
        Self {
            min: i64::MIN,
            max: i64::MAX,
        }
    }
}

impl<'de> Deserialize<'de> for IntRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Accept both:
        //   "margin-left": 283
        //   "margin-left": { "min": 0, "max": 500 }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Scalar(i64),
            Object {
                #[serde(default)]
                min: Option<i64>,
                #[serde(default)]
                max: Option<i64>,
            },
        }

        match Raw::deserialize(deserializer)? {
            Raw::Scalar(v) => Ok(Self { min: v, max: v }),
            Raw::Object { min, max } => Ok(Self {
                min: min.unwrap_or(i64::MIN),
                max: max.unwrap_or(i64::MAX),
            }),
        }
    }
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
