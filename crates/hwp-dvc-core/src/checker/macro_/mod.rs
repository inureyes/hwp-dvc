//! `CheckMacro` — validates that no macro script is embedded when the
//! spec prohibits macros.
//!
//! Maps to `Checker::CheckMacro` in `references/dvc/Checker.cpp`.
//! The C++ version calls `OWPMLReader::haveMacroInDocument()` (which
//! scans `Contents/content.hpf` for `.js` manifest items) and emits
//! an error when `CMacro::permission` is `false` and a macro is found.

use crate::checker::DvcErrorInfo;
use crate::document::Document;
use crate::error::macro_codes;
use crate::spec::MacroSpec;

/// Run the macro check.
///
/// Emits a [`DvcErrorInfo`] with code [`macro_codes::MACRO_PERMISSION`]
/// (7001) when **both** conditions hold:
///
/// 1. `spec.permission == false` — the validation policy forbids macros.
/// 2. `document.has_macro() == true` — the document contains a `.js`
///    manifest entry in `Contents/content.hpf`.
///
/// Returns an empty `Vec` when either condition is not met (i.e., macros
/// are permitted, or the document has no macro).
pub fn check(spec: &MacroSpec, document: &Document) -> Vec<DvcErrorInfo> {
    if spec.permission || !document.has_macro() {
        return Vec::new();
    }

    let error = DvcErrorInfo {
        error_code: macro_codes::MACRO_PERMISSION,
        error_string: "macro script found but macro permission is false".to_owned(),
        ..DvcErrorInfo::default()
    };
    vec![error]
}
