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
