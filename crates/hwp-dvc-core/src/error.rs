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
