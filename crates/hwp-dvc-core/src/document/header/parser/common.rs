//! Shared helpers for the header-XML parser: element-name extraction,
//! generic `Reader::skip`, attribute coercion (string/u32/i32/bool),
//! per-language 7-slot tuple decoding, and OWPML-specific width
//! parsing. Kept in one file because every per-element submodule uses
//! at least half of these.

use std::io::BufRead;

use quick_xml::events::attributes::{AttrError, Attributes};
use quick_xml::events::BytesStart;
use quick_xml::name::QName;
use quick_xml::Reader;

use crate::document::header::types::{FontLang, LangTuple};
use crate::error::{DvcError, DvcResult};

/// Wrap a `quick_xml` attribute error into [`DvcError::Document`].
pub(super) fn xml_err(e: AttrError) -> DvcError {
    DvcError::Document(format!("bad attribute: {e}"))
}

/// Return the local part of an element name (after the colon).
pub(super) fn local_name<'a>(n: QName<'a>) -> &'a [u8] {
    n.local_name().into_inner()
}

/// Skip to the matching end of `start`, discarding all interior events.
pub(super) fn skip<B: BufRead>(reader: &mut Reader<B>, start: &BytesStart<'_>) -> DvcResult<()> {
    reader.read_to_end_into(start.name(), &mut Vec::new())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Attribute helpers
// ---------------------------------------------------------------------------

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

pub(super) fn attr_bool(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<bool> {
    match attr_str(attrs, key)? {
        Some(s) => Ok(matches!(s.trim(), "1" | "true" | "TRUE" | "True")),
        None => Ok(false),
    }
}

pub(super) fn attr_string(attrs: Attributes<'_>, key: &[u8]) -> DvcResult<String> {
    Ok(attr_str(attrs, key)?.unwrap_or_default())
}

/// Parse a width attribute value like `"0.12 mm"` into `f32` millimeters.
pub(super) fn parse_width_mm(s: &str) -> f32 {
    let trimmed = s.trim();
    // Drop any trailing unit suffix (letters/whitespace).
    let numeric_end = trimmed
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(trimmed.len());
    trimmed[..numeric_end].parse::<f32>().unwrap_or(0.0)
}

/// Map each [`FontLang`] to its OWPML attribute name inside per-language
/// tuple elements (`<hh:fontRef hangul="..">`, `<hh:ratio ..>`, …).
fn lang_attr_key(lang: FontLang) -> &'static [u8] {
    match lang {
        FontLang::Hangul => b"hangul",
        FontLang::Latin => b"latin",
        FontLang::Hanja => b"hanja",
        FontLang::Japanese => b"japanese",
        FontLang::Other => b"other",
        FontLang::Symbol => b"symbol",
        FontLang::User => b"user",
    }
}

pub(super) fn parse_lang_tuple_u32(attrs: Attributes<'_>) -> DvcResult<LangTuple<u32>> {
    let mut out = LangTuple::<u32>::default();
    for &lang in &FontLang::ALL {
        out.set(lang, attr_u32(attrs.clone(), lang_attr_key(lang))?);
    }
    Ok(out)
}

pub(super) fn parse_lang_tuple_i32(attrs: Attributes<'_>) -> DvcResult<LangTuple<i32>> {
    let mut out = LangTuple::<i32>::default();
    for &lang in &FontLang::ALL {
        out.set(lang, attr_i32(attrs.clone(), lang_attr_key(lang))?);
    }
    Ok(out)
}

/// Read character data up to the element end with the given local name.
/// Used for `<hh:paraHead>` which carries its template as text content
/// (e.g., `^1.`).
pub(super) fn read_text_until_end<B: BufRead>(
    reader: &mut Reader<B>,
    end_local: &[u8],
) -> DvcResult<String> {
    use quick_xml::events::Event;
    let mut buf = Vec::new();
    let mut out = String::new();
    loop {
        let ev = reader.read_event_into(&mut buf)?;
        match ev {
            Event::Text(t) => {
                let decoded = t.decode().map_err(|e| {
                    DvcError::Document(format!(
                        "text decode in <{}>: {e}",
                        String::from_utf8_lossy(end_local)
                    ))
                })?;
                out.push_str(&decoded);
            }
            Event::End(ref e) if local_name(e.name()) == end_local => return Ok(out),
            Event::Start(ref e) => skip(reader, e)?,
            Event::Empty(_) => {}
            Event::Eof => {
                return Err(DvcError::Document(format!(
                    "unexpected EOF inside <{}>",
                    String::from_utf8_lossy(end_local)
                )))
            }
            _ => {}
        }
        buf.clear();
    }
}
