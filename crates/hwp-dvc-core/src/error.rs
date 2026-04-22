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

/// Lang-type mismatch (`JID_CHAR_SHAPE_LANG = JID_CHAR_SHAPE + 3`).
pub const CHARSHAPE_LANGTYPE: u32 = 1003;

/// Font name not in the spec allow-list (`JID_CHAR_SHAPE_FONT = JID_CHAR_SHAPE + 4`).
pub const CHARSHAPE_FONT: u32 = 1004;

/// Ratio value outside the allowed range (`JID_CHAR_SHAPE_RATIO = JID_CHAR_SHAPE + 7`).
pub const CHARSHAPE_RATIO: u32 = 1007;

/// Spacing value outside the allowed range (`JID_CHAR_SHAPE_SPACING = JID_CHAR_SHAPE + 8`).
pub const CHARSHAPE_SPACING: u32 = 1008;

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
