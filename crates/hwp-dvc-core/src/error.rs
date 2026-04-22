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

/// Individual table error codes (within the `Table = 3000` range).
///
/// These mirror the `JID_TABLE_*` constants in `references/dvc/Source/JsonModel.h`.
/// - `TABLE_BORDER_TYPE`  (3033) — outer border line-type mismatch
/// - `TABLE_BORDER_SIZE`  (3034) — outer border width mismatch
/// - `TABLE_BORDER_COLOR` (3035) — outer border color mismatch
/// - `TABLE_TREAT_AS_CHAR` (3004) — `treatAsChar` attribute mismatch
/// - `TABLE_IN_TABLE`     (3056) — nested table where policy forbids it
pub const TABLE_BORDER_TYPE: u32 = 3033;
pub const TABLE_BORDER_SIZE: u32 = 3034;
pub const TABLE_BORDER_COLOR: u32 = 3035;
pub const TABLE_TREAT_AS_CHAR: u32 = 3004;
pub const TABLE_IN_TABLE: u32 = 3056;

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
    /// JID_PARA_SHAPE_FIRSTLINE — first-line indent.
    pub const PARASHAPE_FIRSTLINE: u32 = 2004;
    /// JID_PARA_SHAPE_INDENT — paragraph indent.
    pub const PARASHAPE_INDENT: u32 = 2005;
    /// JID_PARA_SHAPE_OUTDENT — paragraph outdent (hanging indent).
    pub const PARASHAPE_OUTDENT: u32 = 2006;
    /// JID_PARA_SHAPE_LINESPACING — line-spacing type mismatch.
    pub const PARASHAPE_LINESPACING: u32 = 2007;
    /// JID_PARA_SHAPE_LINESPACINGVALUE — line-spacing value mismatch.
    pub const PARASHAPE_LINESPACINGVALUE: u32 = 2008;
    /// JID_PARA_SHAPE_SPACINGPARAUP — above-paragraph spacing.
    pub const PARASHAPE_SPACINGPARAUP: u32 = 2009;
    /// JID_PARA_SHAPE_SPACINGPARABOTTOM — below-paragraph spacing.
    pub const PARASHAPE_SPACINGPARABOTTOM: u32 = 2010;
}
