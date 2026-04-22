//! Shared helpers for the section-XML parser. A thinner version of
//! the header parser's `common.rs` — the section walker only needs
//! string/u32 attribute coercion and element-name extraction.

use quick_xml::events::attributes::{AttrError, Attributes};
use quick_xml::name::QName;

use crate::error::{DvcError, DvcResult};

/// Wrap a `quick_xml` attribute error into [`DvcError::Document`].
pub(super) fn xml_err(e: AttrError) -> DvcError {
    DvcError::Document(format!("bad attribute: {e}"))
}

/// Return the local part of an element name (the portion after the
/// `ns:` prefix, e.g. `p` for `hp:p`).
pub(super) fn local_name<'a>(n: QName<'a>) -> &'a [u8] {
    n.local_name().into_inner()
}

pub(super) fn attr_str(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<Option<String>> {
    for a in attrs {
        let a = a.map_err(xml_err)?;
        if local_name(a.key) == key {
            let v = a
                .unescape_value()
                .map_err(|e| DvcError::Document(format!("attr decode: {e}")))?;
            return Ok(Some(v.into_owned()));
        }
    }
    Ok(None)
}

pub(super) fn attr_u32(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<u32> {
    match attr_str(attrs, key)? {
        Some(s) => s.trim().parse::<u32>().map_err(|e| {
            DvcError::Document(format!(
                "expected u32 for attribute '{}', got '{}': {e}",
                String::from_utf8_lossy(key),
                s
            ))
        }),
        None => Ok(0),
    }
}

pub(super) fn attr_string(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<String> {
    Ok(attr_str(attrs, key)?.unwrap_or_default())
}

/// Parse a signed-integer attribute. Returns `0` when absent.
pub(super) fn attr_i32(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<i32> {
    match attr_str(attrs, key)? {
        Some(s) => s.trim().parse::<i32>().map_err(|e| {
            DvcError::Document(format!(
                "expected i32 for attribute '{}', got '{}': {e}",
                String::from_utf8_lossy(key),
                s
            ))
        }),
        None => Ok(0),
    }
}

/// Parse a `"0"`/`"1"` boolean attribute the way OWPML writers emit it.
///
/// Treats `"1"` and `"true"` (case-insensitive) as `true`, `"0"` /
/// `"false"` / absent as `false`. Any other literal is rejected with a
/// [`DvcError::Document`] so malformed OWPML surfaces loudly.
pub(super) fn attr_bool01(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<bool> {
    match attr_str(attrs, key)? {
        None => Ok(false),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.eq_ignore_ascii_case("1") || trimmed.eq_ignore_ascii_case("true") {
                Ok(true)
            } else if trimmed.eq_ignore_ascii_case("0")
                || trimmed.eq_ignore_ascii_case("false")
                || trimmed.is_empty()
            {
                Ok(false)
            } else {
                Err(DvcError::Document(format!(
                    "expected 0/1 for attribute '{}', got '{}'",
                    String::from_utf8_lossy(key),
                    trimmed
                )))
            }
        }
    }
}
