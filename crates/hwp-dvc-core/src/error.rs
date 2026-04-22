pub mod messages;

use thiserror::Error;

pub use messages::{error_string, ErrorContext};

pub type DvcResult<T> = Result<T, DvcError>;

#[derive(Debug, Error)]
pub enum DvcError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("spec error: {0}")]
    Spec(String),

    #[error("document error: {0}")]
    Document(String),

    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

// ──────────────────────────────────────────────────────────────────────────────
// Table-range error code constants (mirrors JsonModel.h JID_TABLE_*)
// ──────────────────────────────────────────────────────────────────────────────
//
// The constants below cover the **standard-mode** table checks
// (`--table`). Cell-level fills/gradients/pictures (JID_TABLE+37 … +55)
// belong to the separate `--tabledetail` mode and are not defined here.

/// JID_TABLE_SIZEWIDTH — table width outside the allowed range.
pub const TABLE_SIZE_WIDTH: u32 = 3001;
/// JID_TABLE_SIZEHEIGHT — table height outside the allowed range.
pub const TABLE_SIZE_HEIGHT: u32 = 3002;
/// JID_TABLE_SIZEFIXED — `protect`/fixed-size flag mismatch.
pub const TABLE_SIZE_FIXED: u32 = 3003;
/// JID_TABLE_TREATASCHAR — `treatAsChar` attribute mismatch.
pub const TABLE_TREAT_AS_CHAR: u32 = 3004;
/// JID_TABLE_POS — text-wrap / position type mismatch.
pub const TABLE_POS: u32 = 3005;
/// JID_TABLE_TEXTPOS — text-flow (around-table text position) mismatch.
pub const TABLE_TEXT_POS: u32 = 3006;
/// JID_TABLE_HTYPE — horizontal relative-to type mismatch.
pub const TABLE_HTYPE: u32 = 3007;
/// JID_TABLE_HDIRECTION — horizontal alignment direction mismatch.
pub const TABLE_HDIRECTION: u32 = 3008;
/// JID_TABLE_HVALUE — horizontal offset value outside allowed range.
pub const TABLE_HVALUE: u32 = 3009;
/// JID_TABLE_VTYPE — vertical relative-to type mismatch.
pub const TABLE_VTYPE: u32 = 3010;
/// JID_TABLE_VDIRECTION — vertical alignment direction mismatch.
pub const TABLE_VDIRECTION: u32 = 3011;
/// JID_TABLE_VVALUE — vertical offset value outside allowed range.
pub const TABLE_VVALUE: u32 = 3012;
/// JID_TABLE_SOFLOWWITHTEXT — flow-with-text flag mismatch.
pub const TABLE_SOFLOW_WITH_TEXT: u32 = 3013;
/// JID_TABLE_SOALLOWOVERLAP — allow-overlap flag mismatch.
pub const TABLE_SOALLOW_OVERLAP: u32 = 3014;
/// JID_TABLE_SOHOLDANCHOROBJ — hold-anchor-and-object flag mismatch.
pub const TABLE_SOHOLD_ANCHOR_OBJ: u32 = 3015;
/// JID_TABLE_PARALLEL — parallel (affect-line-spacing) flag mismatch.
pub const TABLE_PARALLEL: u32 = 3016;
/// JID_TABLE_ROTATION — rotation angle out of allowed range.
pub const TABLE_ROTATION: u32 = 3017;
/// JID_TABLE_GRADIENT_H — horizontal gradient offset out of range.
pub const TABLE_GRADIENT_H: u32 = 3018;
/// JID_TABLE_GRADIENT_V — vertical gradient offset out of range.
pub const TABLE_GRADIENT_V: u32 = 3019;
/// JID_TABLE_NUMVERTYPE — numbering type (TABLE/PICTURE/…) mismatch.
pub const TABLE_NUM_VER_TYPE: u32 = 3020;
/// JID_TABLE_OBJPROTECT — object-protection flag mismatch.
pub const TABLE_OBJ_PROTECT: u32 = 3021;
/// JID_TABLE_MARGIN_LEFT — outer left margin out of allowed range.
pub const TABLE_MARGIN_LEFT: u32 = 3022;
/// JID_TABLE_MARGIN_RIGHT — outer right margin out of allowed range.
pub const TABLE_MARGIN_RIGHT: u32 = 3023;
/// JID_TABLE_MARGIN_TOP — outer top margin out of allowed range.
pub const TABLE_MARGIN_TOP: u32 = 3024;
/// JID_TABLE_MARGIN_BOTTOM — outer bottom margin out of allowed range.
pub const TABLE_MARGIN_BOTTOM: u32 = 3025;
/// JID_TABLE_CAPTION_POSITION — caption position mismatch.
pub const TABLE_CAPTION_POSITION: u32 = 3026;
/// JID_TABLE_CAPTION_SIZE — caption size out of allowed range.
pub const TABLE_CAPTION_SIZE: u32 = 3027;
/// JID_TABLE_CAPTION_SPACING — caption spacing out of allowed range.
pub const TABLE_CAPTION_SPACING: u32 = 3028;
/// JID_TABLE_CAPTION_SOCAPFULLSIZE — caption full-size flag mismatch.
pub const TABLE_CAPTION_SOCAP_FULL_SIZE: u32 = 3029;
/// JID_TABLE_CAPTION_LINEWRAP — caption line-wrap flag mismatch.
pub const TABLE_CAPTION_LINE_WRAP: u32 = 3030;
/// JID_TABLE_BORDER_TYPE — outer border line-type mismatch.
pub const TABLE_BORDER_TYPE: u32 = 3033;
/// JID_TABLE_BORDER_SIZE — outer border width mismatch.
pub const TABLE_BORDER_SIZE: u32 = 3034;
/// JID_TABLE_BORDER_COLOR — outer border color mismatch.
pub const TABLE_BORDER_COLOR: u32 = 3035;
/// JID_TABLE_BORDER_CELLSPACING — between-cell spacing out of allowed range.
pub const TABLE_BORDER_CELL_SPACING: u32 = 3036;
/// JID_TABLE_TABLE_IN_TABLE — nested table where policy forbids it.
pub const TABLE_IN_TABLE: u32 = 3056;

/// Per-cell ("detail") table error codes emitted only when
/// `--tabledetail` / [`crate::checker::OutputScope::table_detail`] is set.
///
/// These mirror the `JID_TABLE_{BGFILL,BGGRADATION,PICTURE,PICTUREFILL,EFFECT,WATERMARK}_*`
/// constants in `references/dvc/Source/JsonModel.h` (category 3000, subrange 3037–3055).
/// The reference C++ leaves each of these as empty `case` stubs in
/// `Checker::CheckTableToCheckList`; the Rust port populates them for
/// the detail-mode pass.
pub mod table_detail_codes {
    /// `JID_TABLE_BGFILL_TYPE` (3037) — cell background-fill type mismatch
    /// (`none` / `color` / `gradation`).
    pub const TABLE_BGFILL_TYPE: u32 = 3037;
    /// `JID_TABLE_BGFILL_FACECOLOR` (3038) — solid-fill face color mismatch.
    pub const TABLE_BGFILL_FACECOLOR: u32 = 3038;
    /// `JID_TABLE_BGFILL_PATTONCOLOR` (3039) — pattern-fill pattern color mismatch.
    pub const TABLE_BGFILL_PATTONCOLOR: u32 = 3039;
    /// `JID_TABLE_BGFILL_PATTONTYPE` (3040) — pattern-fill hatch type mismatch.
    pub const TABLE_BGFILL_PATTONTYPE: u32 = 3040;

    /// `JID_TABLE_BGGRADATION_STARTCOLOR` (3041) — gradient start color mismatch.
    pub const TABLE_BGGRADATION_STARTCOLOR: u32 = 3041;
    /// `JID_TABLE_BGGRADATION_ENDCOLOR` (3042) — gradient end color mismatch.
    pub const TABLE_BGGRADATION_ENDCOLOR: u32 = 3042;
    /// `JID_TABLE_BGGRADATION_TYPE` (3043) — gradient type mismatch
    /// (`linear` / `radial` / `square` / `conical`).
    pub const TABLE_BGGRADATION_TYPE: u32 = 3043;
    /// `JID_TABLE_BGGRADATION_WIDTHCENTER` (3044) — gradient width-center mismatch.
    pub const TABLE_BGGRADATION_WIDTHCENTER: u32 = 3044;
    /// `JID_TABLE_BGGRADATION_HEIGHTCENTER` (3045) — gradient height-center mismatch.
    pub const TABLE_BGGRADATION_HEIGHTCENTER: u32 = 3045;
    /// `JID_TABLE_BGGRADATION_GRADATIONANGLE` (3046) — gradient angle mismatch.
    pub const TABLE_BGGRADATION_GRADATIONANGLE: u32 = 3046;
    /// `JID_TABLE_BGGRADATION_BLURLEVEL` (3047) — gradient blur level mismatch.
    pub const TABLE_BGGRADATION_BLURLEVEL: u32 = 3047;
    /// `JID_TABLE_BGGRADATION_BLURCENTER` (3048) — gradient blur center mismatch.
    pub const TABLE_BGGRADATION_BLURCENTER: u32 = 3048;

    /// `JID_TABLE_PICTURE_FILE` (3049) — picture-fill file reference mismatch.
    pub const TABLE_PICTURE_FILE: u32 = 3049;
    /// `JID_TABLE_PICTURE_INCLUDE` (3050) — picture-fill `include` flag mismatch.
    pub const TABLE_PICTURE_INCLUDE: u32 = 3050;
    /// `JID_TABLE_PICTUREFILL_TYPE` (3051) — picture-fill arrangement type mismatch.
    pub const TABLE_PICTUREFILL_TYPE: u32 = 3051;
    /// `JID_TABLE_PICTUREFILL_VALUE` (3052) — picture-fill numeric value mismatch.
    pub const TABLE_PICTUREFILL_VALUE: u32 = 3052;

    /// `JID_TABLE_EFFECT_TYPE` (3053) — picture-effect type mismatch
    /// (`none` / `gray` / `black` / `org`).
    pub const TABLE_EFFECT_TYPE: u32 = 3053;
    /// `JID_TABLE_EFFECT_VALUE` (3054) — picture-effect numeric value mismatch.
    pub const TABLE_EFFECT_VALUE: u32 = 3054;

    /// `JID_TABLE_WATERMARK` (3055) — watermark setting mismatch.
    pub const TABLE_WATERMARK: u32 = 3055;
}

/// Error code ranges mirror the reference C++ implementation
/// (see `references/dvc/Source/JsonModel.h`).
///
/// Category base values are kept so that individual error codes can
/// be added as constants under each category without renumbering.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    CharShape = 1000,
    ParaShape = 2000,
    Table = 3000,
    SpecialCharacter = 3100,
    OutlineShape = 3200,
    Bullet = 3300,
    ParaNumBullet = 3400,
    Style = 3500,
    Page = 4000,
    DocSummary = 5000,
    Footnote = 6000,
    Endnote = 6100,
    Memo = 6200,
    Chart = 6300,
    WordArt = 6400,
    Formula = 6500,
    Ole = 6600,
    FormObject = 6700,
    Bookmark = 6800,
    Hyperlink = 6900,
    Macro = 7000,
}

// ──────────────────────────────────────────────────────────────────────────────
// Bullet-range error code constants (mirrors JsonModel.h JID_BULLET_*)
// ──────────────────────────────────────────────────────────────────────────────

/// Bullet check-type mismatch (`JID_BULLET_CHECKTYPE = 3302`).
pub const BULLET_CHECKTYPE: u32 = 3302;

/// Bullet character-code mismatch (`JID_BULLET_CODE = 3303`).
pub const BULLET_CODE: u32 = 3303;

/// Bullet shape not in the allow-list (`JID_BULLET_SHAPES = 3304`).
pub const BULLET_SHAPES: u32 = 3304;

/// Specific error codes within the [`ErrorCode::Hyperlink`] (6900) range.
pub mod hyperlink_codes {
    /// A run is flagged as a hyperlink but the spec forbids hyperlinks.
    pub const HYPERLINK_PERMISSION: u32 = 6901;
}

/// Specific error codes in the [`ErrorCode::Macro`] (7000) range.
pub mod macro_codes {
    /// Macro script present in the document but `MacroSpec.permission`
    /// is `false` — the document violates the policy.
    pub const MACRO_PERMISSION: u32 = 7001;
}

/// Special-character sub-codes within the `SpecialCharacter` (3100) range.
pub const SPECIALCHAR_MIN: u32 = 3101;
/// Special-character sub-codes within the `SpecialCharacter` (3100) range.
pub const SPECIALCHAR_MAX: u32 = 3102;

// ──────────────────────────────────────────────────────────────────────────────
// CharShape-range error code constants (mirrors JsonModel.h JID_CHAR_SHAPE_*)
// ──────────────────────────────────────────────────────────────────────────────

/// Font size outside the allowed range (`JID_CHAR_SHAPE_FONTSIZE = JID_CHAR_SHAPE + 1`).
pub const CHARSHAPE_FONTSIZE: u32 = 1001;

/// LangSet slot validation failure (`JID_CHAR_SHAPE_LANGSET = JID_CHAR_SHAPE + 2`).
pub const CHARSHAPE_LANGSET: u32 = 1002;

/// Lang-type mismatch (`JID_CHAR_SHAPE_LANG = JID_CHAR_SHAPE + 3`).
pub const CHARSHAPE_LANGTYPE: u32 = 1003;

/// Font name not in the spec allow-list (`JID_CHAR_SHAPE_FONT = JID_CHAR_SHAPE + 4`).
pub const CHARSHAPE_FONT: u32 = 1004;

/// Relative size outside the allowed range (`JID_CHAR_SHAPE_RSIZE = JID_CHAR_SHAPE + 5`).
pub const CHARSHAPE_RSIZE: u32 = 1005;

/// Character position outside the allowed range (`JID_CHAR_SHAPE_POSITION = JID_CHAR_SHAPE + 6`).
pub const CHARSHAPE_POSITION: u32 = 1006;

/// Ratio value outside the allowed range (`JID_CHAR_SHAPE_RATIO = JID_CHAR_SHAPE + 7`).
pub const CHARSHAPE_RATIO: u32 = 1007;

/// Spacing value outside the allowed range (`JID_CHAR_SHAPE_SPACING = JID_CHAR_SHAPE + 8`).
pub const CHARSHAPE_SPACING: u32 = 1008;

/// Bold flag mismatch (`JID_CHAR_SHAPE_BOLD = JID_CHAR_SHAPE + 9`).
pub const CHARSHAPE_BOLD: u32 = 1009;

/// Italic flag mismatch (`JID_CHAR_SHAPE_ITALIC = JID_CHAR_SHAPE + 10`).
pub const CHARSHAPE_ITALIC: u32 = 1010;

/// Underline flag mismatch (`JID_CHAR_SHAPE_UNDERLINE = JID_CHAR_SHAPE + 11`).
pub const CHARSHAPE_UNDERLINE: u32 = 1011;

/// Strikeout flag mismatch (`JID_CHAR_SHAPE_STRIKEOUT = JID_CHAR_SHAPE + 12`).
pub const CHARSHAPE_STRIKEOUT: u32 = 1012;

/// Outline flag mismatch (`JID_CHAR_SHAPE_OUTLINE = JID_CHAR_SHAPE + 13`).
pub const CHARSHAPE_OUTLINE: u32 = 1013;

/// Emboss flag mismatch (`JID_CHAR_SHAPE_EMBOSS = JID_CHAR_SHAPE + 14`).
pub const CHARSHAPE_EMBOSS: u32 = 1014;

/// Engrave flag mismatch (`JID_CHAR_SHAPE_ENGRAVE = JID_CHAR_SHAPE + 15`).
pub const CHARSHAPE_ENGRAVE: u32 = 1015;

/// Shadow flag mismatch (`JID_CHAR_SHAPE_SHADOW = JID_CHAR_SHAPE + 16`).
pub const CHARSHAPE_SHADOW: u32 = 1016;

/// Superscript flag mismatch (`JID_CHAR_SHAPE_SUPSCRIPT = JID_CHAR_SHAPE + 17`).
pub const CHARSHAPE_SUPSCRIPT: u32 = 1017;

/// Subscript flag mismatch (`JID_CHAR_SHAPE_SUBSCRIPT = JID_CHAR_SHAPE + 18`).
pub const CHARSHAPE_SUBSCRIPT: u32 = 1018;

/// Shadow type mismatch (`JID_CHAR_SHAPE_SHADOWTYPE = JID_CHAR_SHAPE + 19`).
pub const CHARSHAPE_SHADOWTYPE: u32 = 1019;

/// Shadow X offset mismatch (`JID_CHAR_SHAPE_SHADOW_X = JID_CHAR_SHAPE + 20`).
pub const CHARSHAPE_SHADOW_X: u32 = 1020;

/// Shadow Y offset mismatch (`JID_CHAR_SHAPE_SHADOW_Y = JID_CHAR_SHAPE + 21`).
pub const CHARSHAPE_SHADOW_Y: u32 = 1021;

/// Shadow color mismatch (`JID_CHAR_SHAPE_SHADOW_COLOR = JID_CHAR_SHAPE + 22`).
pub const CHARSHAPE_SHADOW_COLOR: u32 = 1022;

/// Underline position mismatch (`JID_CHAR_SHAPE_UNDERLINE_POSITION = JID_CHAR_SHAPE + 23`).
pub const CHARSHAPE_UNDERLINE_POSITION: u32 = 1023;

/// Underline shape mismatch (`JID_CHAR_SHAPE_UNDERLINE_SHAPE = JID_CHAR_SHAPE + 24`).
pub const CHARSHAPE_UNDERLINE_SHAPE: u32 = 1024;

/// Underline color mismatch (`JID_CHAR_SHAPE_UNDERLINE_COLOR = JID_CHAR_SHAPE + 25`).
pub const CHARSHAPE_UNDERLINE_COLOR: u32 = 1025;

/// Strikeout shape mismatch (`JID_CHAR_SHAPE_STRIKEOUT_SHAPE = JID_CHAR_SHAPE + 26`).
pub const CHARSHAPE_STRIKEOUT_SHAPE: u32 = 1026;

/// Strikeout color mismatch (`JID_CHAR_SHAPE_STRIKEOUT_COLOR = JID_CHAR_SHAPE + 27`).
pub const CHARSHAPE_STRIKEOUT_COLOR: u32 = 1027;

/// Outline type mismatch (`JID_CHAR_SHAPE_OUTLINETYPE = JID_CHAR_SHAPE + 28`).
pub const CHARSHAPE_OUTLINETYPE: u32 = 1028;

/// Empty-space flag mismatch (`JID_CHAR_SHAPE_EMPTYSPACE = JID_CHAR_SHAPE + 29`).
pub const CHARSHAPE_EMPTYSPACE: u32 = 1029;

/// Point (font-size in points) mismatch (`JID_CHAR_SHAPE_POINT = JID_CHAR_SHAPE + 30`).
pub const CHARSHAPE_POINT: u32 = 1030;

/// Kerning flag mismatch (`JID_CHAR_SHAPE_KERNING = JID_CHAR_SHAPE + 31`).
pub const CHARSHAPE_KERNING: u32 = 1031;

/// Background border mismatch (`JID_CHAR_SHAPE_BG_BORDER = JID_CHAR_SHAPE + 32`).
pub const CHARSHAPE_BG_BORDER: u32 = 1032;

/// Background border position mismatch (`JID_CHAR_SHAPE_BG_BORDER_POSITION = JID_CHAR_SHAPE + 33`).
pub const CHARSHAPE_BG_BORDER_POSITION: u32 = 1033;

/// Background border-type mismatch (`JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE = JID_CHAR_SHAPE + 34`).
pub const CHARSHAPE_BG_BORDER_BORDERTYPE: u32 = 1034;

/// Background border size mismatch (`JID_CHAR_SHAPE_BG_BORDER_SIZE = JID_CHAR_SHAPE + 35`).
pub const CHARSHAPE_BG_BORDER_SIZE: u32 = 1035;

/// Background border color mismatch (`JID_CHAR_SHAPE_BG_BORDER_COLOR = JID_CHAR_SHAPE + 36`).
pub const CHARSHAPE_BG_BORDER_COLOR: u32 = 1036;

/// Background fill color mismatch (`JID_CHAR_SHAPE_BG_COLOR = JID_CHAR_SHAPE + 37`).
pub const CHARSHAPE_BG_COLOR: u32 = 1037;

/// Background pattern color mismatch (`JID_CHAR_SHAPE_BG_PATTONCOLOR = JID_CHAR_SHAPE + 38`).
pub const CHARSHAPE_BG_PATTONCOLOR: u32 = 1038;

/// Background pattern type mismatch (`JID_CHAR_SHAPE_BG_PATTONTYPE = JID_CHAR_SHAPE + 39`).
pub const CHARSHAPE_BG_PATTONTYPE: u32 = 1039;

/// Outline-shape error codes within the [`ErrorCode::OutlineShape`] (3200) range.
///
/// These mirror `JID_OUTLINESHAPE_*` constants from `references/dvc/Source/JsonModel.h`.
pub mod outline_shape_codes {
    /// `JID_OUTLINESHAPE_TYPE` (3201) — outline shape type mismatch.
    pub const OUTLINESHAPE_TYPE: u32 = 3201;
    /// `JID_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE` (3206) — level numbertype template mismatch.
    pub const OUTLINESHAPE_LEVEL_NUMBERTYPE: u32 = 3206;
    /// `JID_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE` (3207) — level numbershape enum mismatch.
    pub const OUTLINESHAPE_LEVEL_NUMBERSHAPE: u32 = 3207;
}

/// Paragraph-number-bullet error codes (3400-range).
///
/// These map to `JID_PARANUMBULLET_*` constants in
/// `references/dvc/Source/JsonModel.h`.
pub mod para_num_bullet_codes {
    /// `JID_PARANUMBULLET_TYPE` — overall type mismatch (3401).
    pub const PARANUM_TYPE: u32 = 3401;
    /// `JID_PARANUMBULLET_LEVELTYPE_NUMBERTYPE` — level number-type mismatch (3406).
    pub const PARANUM_LEVEL_NUMBERTYPE: u32 = 3406;
    /// `JID_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE` — level number-shape mismatch (3407).
    pub const PARANUM_LEVEL_NUMBERSHAPE: u32 = 3407;
}

/// Individual paragraph-shape error codes (2000-range).
///
/// These map to `JID_PARA_SHAPE_*` constants in the reference C++
/// implementation (`references/dvc/Source/JsonModel.h`).
pub mod para_shape_codes {
    /// JID_PARA_SHAPE_HORIZONTAL (2001) — horizontal alignment mismatch.
    pub const PARASHAPE_HORIZONTAL: u32 = 2001;
    /// JID_PARA_SHAPE_MARGINLEFT (2002) — left margin mismatch.
    pub const PARASHAPE_MARGINLEFT: u32 = 2002;
    /// JID_PARA_SHAPE_MARGINRIGHT (2003) — right margin mismatch.
    pub const PARASHAPE_MARGINRIGHT: u32 = 2003;
    /// JID_PARA_SHAPE_FIRSTLINE (2004) — first-line indent.
    pub const PARASHAPE_FIRSTLINE: u32 = 2004;
    /// JID_PARA_SHAPE_INDENT (2005) — paragraph indent.
    pub const PARASHAPE_INDENT: u32 = 2005;
    /// JID_PARA_SHAPE_OUTDENT (2006) — paragraph outdent (hanging indent).
    pub const PARASHAPE_OUTDENT: u32 = 2006;
    /// JID_PARA_SHAPE_LINESPACING (2007) — line-spacing type mismatch.
    pub const PARASHAPE_LINESPACING: u32 = 2007;
    /// JID_PARA_SHAPE_LINESPACINGVALUE (2008) — line-spacing value mismatch.
    pub const PARASHAPE_LINESPACINGVALUE: u32 = 2008;
    /// JID_PARA_SHAPE_SPACINGPARAUP (2009) — above-paragraph spacing.
    pub const PARASHAPE_SPACINGPARAUP: u32 = 2009;
    /// JID_PARA_SHAPE_SPACINGPARABOTTOM (2010) — below-paragraph spacing.
    pub const PARASHAPE_SPACINGPARABOTTOM: u32 = 2010;
    /// JID_PARA_SHAPE_SPACINGGRIDPAPER (2011) — snap-to-grid mismatch.
    pub const PARASHAPE_SPACINGGRIDPAPER: u32 = 2011;
    /// JID_PARA_SHAPE_LINEBREAKKOREAN (2012) — Korean line-break mode mismatch.
    pub const PARASHAPE_LINEBREAKKOREAN: u32 = 2012;
    /// JID_PARA_SHAPE_LINEBREAKENGLISH (2013) — Latin word line-break mismatch.
    pub const PARASHAPE_LINEBREAKENGLISH: u32 = 2013;
    /// JID_PARA_SHAPE_LINEBREAKCONDENSE (2014) — line-break condense value mismatch.
    pub const PARASHAPE_LINEBREAKCONDENSE: u32 = 2014;
    /// JID_PARA_SHAPE_PARATYPE (2015) — paragraph heading type mismatch.
    pub const PARASHAPE_PARATYPE: u32 = 2015;
    /// JID_PARA_SHAPE_PARATYPEVALUE (2016) — paragraph heading id mismatch.
    pub const PARASHAPE_PARATYPEVALUE: u32 = 2016;
    /// JID_PARA_SHAPE_WIDOWORPHAN (2017) — widow/orphan control mismatch.
    pub const PARASHAPE_WIDOWORPHAN: u32 = 2017;
    /// JID_PARA_SHAPE_KEEPWITHNEXT (2018) — keep-with-next mismatch.
    pub const PARASHAPE_KEEPWITHNEXT: u32 = 2018;
    /// JID_PARA_SHAPE_KEEPLINESTOGETHER (2019) — keep-lines-together mismatch.
    pub const PARASHAPE_KEEPLINESTOGETHER: u32 = 2019;
    /// JID_PARA_SHAPE_PAGEBREAKBEFORE (2020) — page-break-before mismatch.
    pub const PARASHAPE_PAGEBREAKBEFORE: u32 = 2020;
    /// JID_PARA_SHAPE_FONTLINEHEIGHT (2021) — font-line-height flag mismatch.
    pub const PARASHAPE_FONTLINEHEIGHT: u32 = 2021;
    /// JID_PARA_SHAPE_LINEWRAP (2022) — line-wrap flag mismatch.
    pub const PARASHAPE_LINEWRAP: u32 = 2022;
    /// JID_PARA_SHAPE_AUTOSPACEEASIANENG (2023) — East Asian/English autospace mismatch.
    pub const PARASHAPE_AUTOSPACEEASIANENG: u32 = 2023;
    /// JID_PARA_SHAPE_AUTOSPACEEASIANNUM (2024) — East Asian/numeral autospace mismatch.
    pub const PARASHAPE_AUTOSPACEEASIANNUM: u32 = 2024;
    /// JID_PARA_SHAPE_VERTICALALIGN (2025) — vertical alignment mismatch.
    pub const PARASHAPE_VERTICALALIGN: u32 = 2025;
    /// JID_PARA_SHAPE_TABTYPES (2026) — tab array presence mismatch.
    /// TODO: Full per-tab field validation (tabtype/tabshape/tabposition) requires
    /// a TabDefinition table in HeaderTables which is not yet parsed.
    pub const PARASHAPE_TABTYPES: u32 = 2026;
    /// JID_PARA_SHAPE_TABTYPE (2027) — tab type mismatch.
    /// TODO: deferred — requires tab definition parsing.
    pub const PARASHAPE_TABTYPE: u32 = 2027;
    /// JID_PARA_SHAPE_TABSHAPE (2028) — tab shape mismatch.
    /// TODO: deferred — requires tab definition parsing.
    pub const PARASHAPE_TABSHAPE: u32 = 2028;
    /// JID_PARA_SHAPE_TABPOSITION (2029) — tab position mismatch.
    /// TODO: deferred — requires tab definition parsing.
    pub const PARASHAPE_TABPOSITION: u32 = 2029;
    /// JID_PARA_SHAPE_AUTOTABINDENT (2030) — auto-tab-indent flag mismatch.
    pub const PARASHAPE_AUTOTABINDENT: u32 = 2030;
    /// JID_PARA_SHAPE_AUTOTABPARARIGHTEND (2031) — auto-tab-para-right-end flag mismatch.
    pub const PARASHAPE_AUTOTABPARARIGHTEND: u32 = 2031;
    /// JID_PARA_SHAPE_BASETABSPACE (2032) — base tab space mismatch.
    pub const PARASHAPE_BASETABSPACE: u32 = 2032;
    /// JID_PARA_SHAPE_BORDER (2033) — paragraph border presence mismatch.
    /// TODO: Full border comparison requires resolving border_fill_id_ref to a
    /// BorderFill record and comparing type/size/color per edge; deferred until
    /// paragraph-level BorderFill lookup is wired up.
    pub const PARASHAPE_BORDER: u32 = 2033;
    /// JID_PARA_SHAPE_BORDERPOSITION (2034) — border position mismatch.
    /// TODO: deferred — see PARASHAPE_BORDER.
    pub const PARASHAPE_BORDERPOSITION: u32 = 2034;
    /// JID_PARA_SHAPE_BORDERTYPE (2035) — border line type mismatch.
    /// TODO: deferred — see PARASHAPE_BORDER.
    pub const PARASHAPE_BORDERTYPE: u32 = 2035;
    /// JID_PARA_SHAPE_BORDERSIZE (2036) — border size mismatch.
    /// TODO: deferred — see PARASHAPE_BORDER.
    pub const PARASHAPE_BORDERSIZE: u32 = 2036;
    /// JID_PARA_SHAPE_BORDERCOLOR (2037) — border color mismatch.
    /// TODO: deferred — see PARASHAPE_BORDER.
    pub const PARASHAPE_BORDERCOLOR: u32 = 2037;
    /// JID_PARA_SHAPE_BGCOLOR (2038) — background color mismatch.
    /// TODO: deferred — background color/pattern fields are not yet decoded
    /// from the paragraph BorderFill record.
    pub const PARASHAPE_BGCOLOR: u32 = 2038;
    /// JID_PARA_SHAPE_BGPATTONCOLOR (2039) — background pattern color mismatch.
    /// TODO: deferred — see PARASHAPE_BGCOLOR.
    pub const PARASHAPE_BGPATTONCOLOR: u32 = 2039;
    /// JID_PARA_SHAPE_BGPATTONTYPE (2040) — background pattern type mismatch.
    /// TODO: deferred — see PARASHAPE_BGCOLOR.
    pub const PARASHAPE_BGPATTONTYPE: u32 = 2040;
    /// JID_PARA_SHAPE_SPACINGLEFT (2041) — paragraph border left-spacing flag mismatch.
    pub const PARASHAPE_SPACINGLEFT: u32 = 2041;
    /// JID_PARA_SHAPE_SPACINGRIGHT (2042) — paragraph border right-spacing flag mismatch.
    pub const PARASHAPE_SPACINGRIGHT: u32 = 2042;
    /// JID_PARA_SHAPE_SPACINGTOP (2043) — paragraph border top-spacing flag mismatch.
    pub const PARASHAPE_SPACINGTOP: u32 = 2043;
    /// JID_PARA_SHAPE_SPACINGBOTTOM (2044) — paragraph border bottom-spacing flag mismatch.
    pub const PARASHAPE_SPACINGBOTTOM: u32 = 2044;
    /// JID_PARA_SHAPE_SPACINGIGNORE (2045) — ignore-margin flag mismatch.
    pub const PARASHAPE_SPACINGIGNORE: u32 = 2045;
}
