use thiserror::Error;

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
