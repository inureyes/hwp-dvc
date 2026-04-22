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

    // --- Per-cell detail-mode fields (JID_TABLE 3037..=3055) --------------
    //
    // The JSON keys mirror `references/dvc/sample/jsonFullSpec.json` (the
    // hyphenated "bgfill-type" style). These are only consulted when the
    // checker runs in detail mode (`--tabledetail` / `OutputScope::table_detail`).
    /// `bgfill-type` (JID_TABLE_BGFILL_TYPE / 3037) — allowed fill kinds.
    ///
    /// Accepted values: `"none"`, `"color"`, `"gradation"`.
    #[serde(rename = "bgfill-type", default)]
    pub bgfill_type: Option<String>,
    /// `bgfill-facecolor` (3038) — packed 24-bit RGB face color.
    #[serde(rename = "bgfill-facecolor", default)]
    pub bgfill_facecolor: Option<u32>,
    /// `bgfill-pattoncolor` (3039) — packed 24-bit RGB pattern color.
    #[serde(rename = "bgfill-pattoncolor", default)]
    pub bgfill_pattoncolor: Option<u32>,
    /// `bgfill-pattontype` (3040) — integer index into the HATCH_STYLE enum.
    #[serde(rename = "bgfill-pattontype", default)]
    pub bgfill_pattontype: Option<u32>,

    /// `bggradation-startcolor` (3041).
    #[serde(rename = "bggradation-startcolor", default)]
    pub bggradation_startcolor: Option<u32>,
    /// `bggradation-endcolor` (3042).
    #[serde(rename = "bggradation-endcolor", default)]
    pub bggradation_endcolor: Option<u32>,
    /// `bggradation-type` (3043). Accepted: `"linear"`, `"radial"`,
    /// `"square"`, `"conical"` (mirrors the reference C++ BGGradationType).
    #[serde(rename = "bggradation-type", default)]
    pub bggradation_type: Option<String>,
    /// `bggradation-widthcenter` (3044).
    #[serde(rename = "bggradation-widthcenter", default)]
    pub bggradation_widthcenter: Option<u32>,
    /// `bggradation-heightcenter` (3045).
    #[serde(rename = "bggradation-heightcenter", default)]
    pub bggradation_heightcenter: Option<u32>,
    /// `bggradation-gradationangle` (3046).
    #[serde(rename = "bggradation-gradationangle", default)]
    pub bggradation_gradationangle: Option<u32>,
    /// `bggradation-blurlevel` (3047).
    #[serde(rename = "bggradation-blurlevel", default)]
    pub bggradation_blurlevel: Option<u32>,
    /// `bggradation-blurcenter` (3048).
    #[serde(rename = "bggradation-blurcenter", default)]
    pub bggradation_blurcenter: Option<u32>,

    /// `picture-file` (3049). When `Some`, cells without a matching
    /// picture reference emit the error.
    #[serde(rename = "picture-file", default)]
    pub picture_file: Option<String>,
    /// `picture-include` (3050) — whether the picture file should be embedded.
    #[serde(rename = "picture-include", default)]
    pub picture_include: Option<bool>,
    /// `picturefill-type` (3051). Accepted: `"tile"`, `"total"`, `"center"`,
    /// `"zoom"` etc. (the PicFillType enum).
    #[serde(rename = "picturefill-type", default)]
    pub picturefill_type: Option<String>,
    /// `picturefill-value` (3052).
    #[serde(rename = "picturefill-value", default)]
    pub picturefill_value: Option<i32>,

    /// `effect-type` (3053). Accepted: `"none"`, `"gray"`, `"black"`, `"org"`.
    #[serde(rename = "effect-type", default)]
    pub effect_type: Option<String>,
    /// `effect-value` (3054).
    #[serde(rename = "effect-value", default)]
    pub effect_value: Option<i32>,
    /// `watermark` (3055).
    #[serde(default)]
    pub watermark: Option<u32>,
}

impl TableSpec {
    /// Returns `true` when *no* cell-detail field is populated. The
    /// detail-mode validator uses this to short-circuit cheaply when the
    /// spec has no cell requirements.
    pub(crate) fn has_cell_detail_fields(&self) -> bool {
        self.bgfill_type.is_some()
            || self.bgfill_facecolor.is_some()
            || self.bgfill_pattoncolor.is_some()
            || self.bgfill_pattontype.is_some()
            || self.bggradation_startcolor.is_some()
            || self.bggradation_endcolor.is_some()
            || self.bggradation_type.is_some()
            || self.bggradation_widthcenter.is_some()
            || self.bggradation_heightcenter.is_some()
            || self.bggradation_gradationangle.is_some()
            || self.bggradation_blurlevel.is_some()
            || self.bggradation_blurcenter.is_some()
            || self.picture_file.is_some()
            || self.picture_include.is_some()
            || self.picturefill_type.is_some()
            || self.picturefill_value.is_some()
            || self.effect_type.is_some()
            || self.effect_value.is_some()
            || self.watermark.is_some()
    }
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
    /// Expected start number for the top-level `<hh:numbering start="…"/>` element.
    ///
    /// When `Some`, validated against [`Numbering::start`]; a mismatch fires
    /// `OUTLINESHAPE_STARTNUMBER` (3202).
    #[serde(rename = "startNumber", default)]
    pub start_number: Option<u32>,

    /// Expected per-level start value (`<hh:paraHead start="…"/>`).
    ///
    /// When `Some`, every level entry in `leveltype` is validated against
    /// [`ParaHead::start`]; a mismatch fires `OUTLINESHAPE_VALUE` (3203).
    #[serde(default)]
    pub value: Option<u32>,

    /// Required level entries.  When non-empty:
    ///
    /// - The document numbering must declare exactly the same set of levels
    ///   (any mismatch fires `OUTLINESHAPE_LEVELTYPE` / 3204).
    /// - Within each level, the `level` index is validated first
    ///   (`OUTLINESHAPE_LEVELTYPE_LEVEL` / 3205), then `numbertype`
    ///   (`OUTLINESHAPE_LEVEL_NUMBERTYPE` / 3206) and `numbershape`
    ///   (`OUTLINESHAPE_LEVEL_NUMBERSHAPE` / 3207).
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

    #[test]
    fn table_cell_detail_fields_parse() {
        // Every cell-detail field, taken straight from the hyphenated
        // keys in `jsonFullSpec.json`.
        let s = r#"{
            "table": {
                "bgfill-type": "none",
                "bgfill-facecolor": 16777215,
                "bgfill-pattoncolor": 0,
                "bgfill-pattontype": 0,
                "bggradation-startcolor": 0,
                "bggradation-endcolor": 255,
                "bggradation-type": "linear",
                "bggradation-widthcenter": 50,
                "bggradation-heightcenter": 50,
                "bggradation-gradationangle": 90,
                "bggradation-blurlevel": 0,
                "bggradation-blurcenter": 50,
                "picture-file": "embed.png",
                "picture-include": true,
                "picturefill-type": "zoom",
                "picturefill-value": 0,
                "effect-type": "none",
                "effect-value": 0,
                "watermark": 0
            }
        }"#;
        let spec = DvcSpec::from_json_str(s).unwrap();
        let t = spec.table.unwrap();
        assert!(t.has_cell_detail_fields());
        assert_eq!(t.bgfill_type.as_deref(), Some("none"));
        assert_eq!(t.bgfill_facecolor, Some(16777215));
        assert_eq!(t.bggradation_type.as_deref(), Some("linear"));
        assert_eq!(t.picture_file.as_deref(), Some("embed.png"));
        assert_eq!(t.picture_include, Some(true));
        assert_eq!(t.effect_type.as_deref(), Some("none"));
        assert_eq!(t.watermark, Some(0));
    }

    #[test]
    fn table_spec_without_cell_detail_fields_has_no_cell_detail() {
        let s = r#"{ "table": { "treatAsChar": true } }"#;
        let spec = DvcSpec::from_json_str(s).unwrap();
        let t = spec.table.unwrap();
        assert!(!t.has_cell_detail_fields());
    }
}
