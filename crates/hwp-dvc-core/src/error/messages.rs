//! Korean error message resolver — issue #16.
//!
//! Provides a table-driven mapping from error codes to human-readable
//! Korean strings that match the reference C++ DVC implementation.
//!
//! # Usage
//!
//! ```rust
//! use hwp_dvc_core::error::messages::{error_string, ErrorContext};
//!
//! let msg = error_string(1004, ErrorContext::with_font("나눔고딕"));
//! assert!(msg.contains("글꼴"));
//! ```
//!
//! # English override
//!
//! Set the environment variable `HWP_DVC_LANG=en` to get English messages
//! instead of Korean. This is intended for debugging and testing only.

/// Optional context parameters for error message formatting.
///
/// Each field corresponds to a variable that may appear in a message
/// template. Pass only the fields relevant to the error being reported;
/// unused fields are silently ignored.
#[derive(Debug, Default, Clone)]
pub struct ErrorContext<'a> {
    /// The offending font name (used by `CHARSHAPE_FONT` / 1004).
    pub font_name: Option<&'a str>,
    /// A numeric "found" value (used by ratio/spacing/line-spacing errors).
    pub found_value: Option<i64>,
    /// A numeric "expected" value (used by ratio/spacing/line-spacing errors).
    pub expected_value: Option<i64>,
    /// The offending bullet character (used by `BULLET_SHAPES` / 3304).
    pub bullet_char: Option<&'a str>,
}

impl<'a> ErrorContext<'a> {
    /// Convenience constructor for a font-name context.
    #[must_use]
    pub fn with_font(font_name: &'a str) -> Self {
        Self {
            font_name: Some(font_name),
            ..Self::default()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Static message table
// ─────────────────────────────────────────────────────────────────────────────

/// Static Korean message for error codes that need no context substitution.
///
/// Codes that require runtime interpolation (e.g. font name) are handled
/// separately in [`error_string`].
const STATIC_MESSAGES_KO: &[(u32, &str)] = &[
    // ── CharShape (1000-range) ────────────────────────────────────────────
    (1003, "언어 종류가 허용되지 않습니다"),
    // 1004 is dynamic — see error_string()
    (1007, "글자 장평 값이 허용된 범위를 벗어났습니다"),
    (1008, "글자 자간 값이 허용된 범위를 벗어났습니다"),
    // ── ParaShape (2000-range) ────────────────────────────────────────────
    (2004, "첫째 줄 들여쓰기 값이 허용된 범위를 벗어났습니다"),
    (2005, "들여쓰기 값이 허용된 범위를 벗어났습니다"),
    (2006, "내어쓰기 값이 허용된 범위를 벗어났습니다"),
    (2007, "줄 간격 유형이 허용되지 않습니다"),
    (2008, "줄 간격 값이 허용된 범위를 벗어났습니다"),
    (2009, "문단 위 간격 값이 허용된 범위를 벗어났습니다"),
    (2010, "문단 아래 간격 값이 허용된 범위를 벗어났습니다"),
    // ── Table (3000-range) ────────────────────────────────────────────────
    (3004, "표의 글자 취급 속성이 허용되지 않습니다"),
    (3033, "표 테두리 선 종류가 허용되지 않습니다"),
    (3034, "표 테두리 선 굵기가 허용되지 않습니다"),
    (3035, "표 테두리 선 색상이 허용되지 않습니다"),
    (3056, "표 안에 표가 허용되지 않습니다"),
    // ── SpecialCharacter (3100-range) ─────────────────────────────────────
    (3101, "허용 범위보다 낮은 특수 문자가 포함되어 있습니다"),
    (3102, "허용 범위보다 높은 특수 문자가 포함되어 있습니다"),
    // ── OutlineShape (3200-range) ─────────────────────────────────────────
    (3201, "개요 모양 유형이 허용되지 않습니다"),
    (3206, "개요 수준의 번호 형식이 허용되지 않습니다"),
    (3207, "개요 수준의 번호 모양이 허용되지 않습니다"),
    // ── Bullet (3300-range) ───────────────────────────────────────────────
    (3302, "글머리표 확인 유형이 허용되지 않습니다"),
    (3303, "글머리표 문자 코드가 허용되지 않습니다"),
    // 3304 is dynamic — see error_string()
    // ── ParaNumBullet (3400-range) ────────────────────────────────────────
    (3401, "문단 번호 유형이 허용되지 않습니다"),
    (3406, "문단 번호 수준의 번호 형식이 허용되지 않습니다"),
    (3407, "문단 번호 수준의 번호 모양이 허용되지 않습니다"),
    // ── Style (3500-range) ────────────────────────────────────────────────
    (3502, "스타일 사용이 허용되지 않습니다"),
    // ── Hyperlink (6900-range) ────────────────────────────────────────────
    (6901, "하이퍼링크 사용이 허용되지 않습니다"),
    // ── Macro (7000-range) ───────────────────────────────────────────────
    (7001, "매크로 스크립트가 포함되어 있어 허용되지 않습니다"),
];

/// Static English message table (used when `HWP_DVC_LANG=en`).
const STATIC_MESSAGES_EN: &[(u32, &str)] = &[
    (1003, "language type is not allowed"),
    // 1004 is dynamic
    (1007, "character ratio is out of the allowed range"),
    (1008, "character spacing is out of the allowed range"),
    (2004, "first-line indent is out of the allowed range"),
    (2005, "paragraph indent is out of the allowed range"),
    (2006, "paragraph outdent is out of the allowed range"),
    (2007, "line-spacing type is not allowed"),
    (2008, "line-spacing value is out of the allowed range"),
    (2009, "paragraph spacing above is out of the allowed range"),
    (2010, "paragraph spacing below is out of the allowed range"),
    (3004, "table treat-as-char attribute is not allowed"),
    (3033, "table border line type is not allowed"),
    (3034, "table border line width is not allowed"),
    (3035, "table border color is not allowed"),
    (3056, "table inside table is not allowed"),
    (
        3101,
        "text contains a special character below the allowed minimum",
    ),
    (
        3102,
        "text contains a special character above the allowed maximum",
    ),
    (3201, "outline shape type is not allowed"),
    (3206, "outline level number format is not allowed"),
    (3207, "outline level number shape is not allowed"),
    (3302, "bullet check type is not allowed"),
    (3303, "bullet character code is not allowed"),
    // 3304 is dynamic
    (3401, "paragraph numbering type is not allowed"),
    (3406, "paragraph numbering level format is not allowed"),
    (3407, "paragraph numbering level shape is not allowed"),
    (3502, "use of custom styles is not allowed"),
    (6901, "use of hyperlinks is not allowed"),
    (7001, "macro script is present but macros are not permitted"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Language selection
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` when the `HWP_DVC_LANG` environment variable is set to `"en"`.
fn is_english_override() -> bool {
    std::env::var("HWP_DVC_LANG")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("en"))
        .unwrap_or(false)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Return a human-readable error message for `code`.
///
/// Korean by default; English when `HWP_DVC_LANG=en`.
///
/// Dynamic messages (those requiring runtime substitution) are handled
/// inline; all others are looked up in the static message table. An empty
/// string is returned for unknown codes.
#[must_use]
pub fn error_string(code: u32, ctx: ErrorContext<'_>) -> String {
    if is_english_override() {
        error_string_lang(code, ctx, STATIC_MESSAGES_EN, true)
    } else {
        error_string_lang(code, ctx, STATIC_MESSAGES_KO, false)
    }
}

/// Internal helper that selects between language tables.
fn error_string_lang(
    code: u32,
    ctx: ErrorContext<'_>,
    table: &[(u32, &str)],
    english: bool,
) -> String {
    match code {
        // ── CHARSHAPE_FONT (1004) — dynamic: include the font name ──────────
        1004 => {
            if let Some(name) = ctx.font_name {
                if english {
                    format!("'{name}' is not an allowed font")
                } else {
                    format!("'{name}'은(는) 허용된 글꼴이 아닙니다")
                }
            } else if english {
                "font is not in the allowed list".to_owned()
            } else {
                "글꼴이 허용 목록에 없습니다".to_owned()
            }
        }

        // ── BULLET_SHAPES (3304) — dynamic: include the bullet char ─────────
        3304 => {
            if let Some(ch) = ctx.bullet_char {
                if english {
                    format!("bullet character '{ch}' is not in the allowed list")
                } else {
                    format!("글머리표 문자 '{ch}'이(가) 허용 목록에 없습니다")
                }
            } else if english {
                "bullet character is not in the allowed list".to_owned()
            } else {
                "글머리표 문자가 허용 목록에 없습니다".to_owned()
            }
        }

        // ── Static lookup ────────────────────────────────────────────────────
        _ => table
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, msg)| (*msg).to_owned())
            .unwrap_or_default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ErrorContext<'static> {
        ErrorContext::default()
    }

    // ── Korean output (default) ───────────────────────────────────────────────

    #[test]
    fn charshape_font_without_name_contains_glyph_word() {
        let msg = error_string(1004, ctx());
        assert!(
            msg.contains("글꼴"),
            "1004 without name must mention 글꼴: {msg}"
        );
    }

    #[test]
    fn charshape_font_with_name_contains_font_name_and_glyph_word() {
        let msg = error_string(1004, ErrorContext::with_font("나눔고딕"));
        assert!(msg.contains("나눔고딕"), "must include font name: {msg}");
        assert!(msg.contains("글꼴"), "must mention 글꼴: {msg}");
    }

    #[test]
    fn charshape_ratio_message_is_korean() {
        let msg = error_string(1007, ctx());
        assert!(!msg.is_empty(), "1007 must have a message");
        // Korean text is non-ASCII — verify the message contains Korean
        assert!(
            msg.chars().any(|c| c > '\u{0080}'),
            "1007 message must contain Korean characters: {msg}"
        );
    }

    #[test]
    fn charshape_spacing_message_is_korean() {
        let msg = error_string(1008, ctx());
        assert!(!msg.is_empty(), "1008 must have a message");
    }

    #[test]
    fn parashape_linespacing_message_is_korean() {
        let msg = error_string(2007, ctx());
        assert!(!msg.is_empty(), "2007 must have a message");
        assert!(msg.chars().any(|c| c > '\u{0080}'));
    }

    #[test]
    fn table_in_table_message_is_korean() {
        let msg = error_string(3056, ctx());
        assert!(!msg.is_empty(), "3056 must have a message");
        assert!(msg.contains("표"), "must mention 표 (table): {msg}");
    }

    #[test]
    fn bullet_shapes_without_char_contains_bullet_word() {
        let msg = error_string(3304, ctx());
        assert!(
            msg.contains("글머리표"),
            "3304 without char must mention 글머리표: {msg}"
        );
    }

    #[test]
    fn bullet_shapes_with_char_contains_char() {
        let ctx = ErrorContext {
            bullet_char: Some("□"),
            ..Default::default()
        };
        let msg = error_string(3304, ctx);
        assert!(
            msg.contains("□"),
            "3304 with char must include the char: {msg}"
        );
        assert!(msg.contains("글머리표"), "must mention 글머리표: {msg}");
    }

    #[test]
    fn hyperlink_message_is_korean() {
        let msg = error_string(6901, ctx());
        assert!(!msg.is_empty(), "6901 must have a message");
        assert!(msg.contains("하이퍼링크"), "must mention 하이퍼링크: {msg}");
    }

    #[test]
    fn macro_message_contains_macro_word() {
        let msg = error_string(7001, ctx());
        assert!(!msg.is_empty(), "7001 must have a message");
        assert!(msg.contains("매크로"), "must mention 매크로: {msg}");
    }

    #[test]
    fn style_message_is_korean() {
        let msg = error_string(3502, ctx());
        assert!(!msg.is_empty(), "3502 must have a message");
        assert!(msg.contains("스타일"), "must mention 스타일: {msg}");
    }

    #[test]
    fn unknown_code_returns_empty_string() {
        let msg = error_string(9999, ctx());
        assert!(
            msg.is_empty(),
            "unknown code must return empty string: {msg}"
        );
    }

    #[test]
    fn all_documented_codes_have_non_empty_messages() {
        let codes: &[u32] = &[
            1003, 1004, 1007, 1008, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 3004, 3033, 3034,
            3035, 3056, 3101, 3102, 3201, 3206, 3207, 3302, 3303, 3304, 3401, 3406, 3407, 3502,
            6901, 7001,
        ];
        for &code in codes {
            let msg = error_string(code, ctx());
            assert!(!msg.is_empty(), "code {code} must have a non-empty message");
        }
    }
}
