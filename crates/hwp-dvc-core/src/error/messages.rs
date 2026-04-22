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
    (1001, "글자 크기가 허용된 범위를 벗어났습니다"),
    (1002, "언어 세트가 허용되지 않습니다"),
    (1003, "언어 종류가 허용되지 않습니다"),
    // 1004 is dynamic — see error_string()
    (1005, "글자 상대 크기가 허용된 범위를 벗어났습니다"),
    (1006, "글자 위치가 허용된 범위를 벗어났습니다"),
    (1007, "글자 장평 값이 허용된 범위를 벗어났습니다"),
    (1008, "글자 자간 값이 허용된 범위를 벗어났습니다"),
    (1009, "굵게 속성이 허용되지 않습니다"),
    (1010, "기울임꼴 속성이 허용되지 않습니다"),
    (1011, "밑줄 속성이 허용되지 않습니다"),
    (1012, "취소선 속성이 허용되지 않습니다"),
    (1013, "외곽선 속성이 허용되지 않습니다"),
    (1014, "양각 속성이 허용되지 않습니다"),
    (1015, "음각 속성이 허용되지 않습니다"),
    (1016, "그림자 속성이 허용되지 않습니다"),
    (1017, "위첨자 속성이 허용되지 않습니다"),
    (1018, "아래첨자 속성이 허용되지 않습니다"),
    (1019, "그림자 종류가 허용되지 않습니다"),
    (1020, "그림자 X 방향 오프셋이 허용된 범위를 벗어났습니다"),
    (1021, "그림자 Y 방향 오프셋이 허용된 범위를 벗어났습니다"),
    (1022, "그림자 색상이 허용되지 않습니다"),
    (1023, "밑줄 위치가 허용되지 않습니다"),
    (1024, "밑줄 모양이 허용되지 않습니다"),
    (1025, "밑줄 색상이 허용되지 않습니다"),
    (1026, "취소선 모양이 허용되지 않습니다"),
    (1027, "취소선 색상이 허용되지 않습니다"),
    (1028, "외곽선 종류가 허용되지 않습니다"),
    (1029, "빈 칸 사용 속성이 허용되지 않습니다"),
    (1030, "글자 크기(포인트)가 허용된 범위를 벗어났습니다"),
    (1031, "커닝 속성이 허용되지 않습니다"),
    (1032, "글자 배경 테두리가 허용되지 않습니다"),
    (1033, "글자 배경 테두리 위치가 허용되지 않습니다"),
    (1034, "글자 배경 테두리 종류가 허용되지 않습니다"),
    (1035, "글자 배경 테두리 굵기가 허용되지 않습니다"),
    (1036, "글자 배경 테두리 색상이 허용되지 않습니다"),
    (1037, "글자 배경 색상이 허용되지 않습니다"),
    (1038, "글자 배경 패턴 색상이 허용되지 않습니다"),
    (1039, "글자 배경 패턴 종류가 허용되지 않습니다"),
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
    // ── Table cell-detail (3037-3055) — --tabledetail only ────────────────
    (3037, "셀 배경 채우기 종류가 허용되지 않습니다"),
    (3038, "셀 배경 면 색상이 허용되지 않습니다"),
    (3039, "셀 배경 무늬 색상이 허용되지 않습니다"),
    (3040, "셀 배경 무늬 종류가 허용되지 않습니다"),
    (3041, "셀 그라데이션 시작 색상이 허용되지 않습니다"),
    (3042, "셀 그라데이션 끝 색상이 허용되지 않습니다"),
    (3043, "셀 그라데이션 종류가 허용되지 않습니다"),
    (3044, "셀 그라데이션 가로 중심 값이 허용되지 않습니다"),
    (3045, "셀 그라데이션 세로 중심 값이 허용되지 않습니다"),
    (3046, "셀 그라데이션 각도가 허용되지 않습니다"),
    (3047, "셀 그라데이션 번짐 정도가 허용되지 않습니다"),
    (3048, "셀 그라데이션 번짐 중심 값이 허용되지 않습니다"),
    (3049, "셀 그림 파일 참조가 허용되지 않습니다"),
    (3050, "셀 그림 포함 설정이 허용되지 않습니다"),
    (3051, "셀 그림 채우기 방식이 허용되지 않습니다"),
    (3052, "셀 그림 채우기 값이 허용되지 않습니다"),
    (3053, "셀 그림 효과 종류가 허용되지 않습니다"),
    (3054, "셀 그림 효과 값이 허용되지 않습니다"),
    (3055, "셀 워터마크 설정이 허용되지 않습니다"),
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
    (1001, "font size is out of the allowed range"),
    (1002, "language set is not allowed"),
    (1003, "language type is not allowed"),
    // 1004 is dynamic
    (1005, "relative character size is out of the allowed range"),
    (1006, "character position is out of the allowed range"),
    (1007, "character ratio is out of the allowed range"),
    (1008, "character spacing is out of the allowed range"),
    (1009, "bold attribute is not allowed"),
    (1010, "italic attribute is not allowed"),
    (1011, "underline attribute is not allowed"),
    (1012, "strikeout attribute is not allowed"),
    (1013, "outline attribute is not allowed"),
    (1014, "emboss attribute is not allowed"),
    (1015, "engrave attribute is not allowed"),
    (1016, "shadow attribute is not allowed"),
    (1017, "superscript attribute is not allowed"),
    (1018, "subscript attribute is not allowed"),
    (1019, "shadow type is not allowed"),
    (1020, "shadow X offset is out of the allowed range"),
    (1021, "shadow Y offset is out of the allowed range"),
    (1022, "shadow color is not allowed"),
    (1023, "underline position is not allowed"),
    (1024, "underline shape is not allowed"),
    (1025, "underline color is not allowed"),
    (1026, "strikeout shape is not allowed"),
    (1027, "strikeout color is not allowed"),
    (1028, "outline type is not allowed"),
    (1029, "empty-space attribute is not allowed"),
    (1030, "font size in points is out of the allowed range"),
    (1031, "kerning attribute is not allowed"),
    (1032, "character background border is not allowed"),
    (1033, "character background border position is not allowed"),
    (1034, "character background border type is not allowed"),
    (1035, "character background border size is not allowed"),
    (1036, "character background border color is not allowed"),
    (1037, "character background color is not allowed"),
    (1038, "character background pattern color is not allowed"),
    (1039, "character background pattern type is not allowed"),
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
    // Table cell-detail (3037-3055) — emitted only with --tabledetail
    (3037, "cell background fill type is not allowed"),
    (3038, "cell background face color is not allowed"),
    (3039, "cell background pattern color is not allowed"),
    (3040, "cell background pattern type is not allowed"),
    (3041, "cell gradient start color is not allowed"),
    (3042, "cell gradient end color is not allowed"),
    (3043, "cell gradient type is not allowed"),
    (3044, "cell gradient width-center value is not allowed"),
    (3045, "cell gradient height-center value is not allowed"),
    (3046, "cell gradient angle is not allowed"),
    (3047, "cell gradient blur level is not allowed"),
    (3048, "cell gradient blur center is not allowed"),
    (3049, "cell picture file reference is not allowed"),
    (3050, "cell picture include flag is not allowed"),
    (3051, "cell picture fill type is not allowed"),
    (3052, "cell picture fill value is not allowed"),
    (3053, "cell picture effect type is not allowed"),
    (3054, "cell picture effect value is not allowed"),
    (3055, "cell watermark is not allowed"),
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
            // CharShape (1000-range)
            1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014,
            1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1027, 1028,
            1029, 1030, 1031, 1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039,
            // ParaShape (2000-range)
            2004, 2005, 2006, 2007, 2008, 2009, 2010, // Table standard-mode (3000-range)
            3004, 3033, 3034, 3035,
            // Table cell-detail (3037-3055) — --tabledetail only
            3037, 3038, 3039, 3040, 3041, 3042, 3043, 3044, 3045, 3046, 3047, 3048, 3049, 3050,
            3051, 3052, 3053, 3054, 3055, 3056, // SpecialCharacter (3100-range)
            3101, 3102, // OutlineShape (3200-range)
            3201, 3206, 3207, // Bullet (3300-range)
            3302, 3303, 3304, // ParaNumBullet (3400-range)
            3401, 3406, 3407, // Style (3500-range)
            3502, // Hyperlink (6900-range)
            6901, // Macro (7000-range)
            7001,
        ];
        for &code in codes {
            let msg = error_string(code, ctx());
            assert!(!msg.is_empty(), "code {code} must have a non-empty message");
        }
    }

    #[test]
    fn table_detail_codes_mention_cell_in_korean() {
        // Each cell-level detail code should begin with "셀" (cell) in Korean.
        let cell_codes: &[u32] = &[
            3037, 3038, 3039, 3040, 3041, 3042, 3043, 3044, 3045, 3046, 3047, 3048, 3049, 3050,
            3051, 3052, 3053, 3054, 3055,
        ];
        for &code in cell_codes {
            let msg = error_string(code, ctx());
            assert!(
                msg.contains("셀"),
                "table-detail code {code} must mention 셀 (cell): {msg}"
            );
        }
    }
}
