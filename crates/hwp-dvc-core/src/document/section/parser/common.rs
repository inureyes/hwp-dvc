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
