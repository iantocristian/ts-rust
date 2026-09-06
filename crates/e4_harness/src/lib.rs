use std::panic::{catch_unwind, AssertUnwindSafe};

use ts_jsstring::{
    combine_surrogate_pairs, compute_ecma_line_starts, ecma_line_and_utf16_character_of_position,
    encode_js_string_rune, escape_non_ascii_string, escape_string, escape_string_worker,
    lower_first_char, lsp_line_and_character_to_position, lsp_position_to_line_and_character,
    scanner_position_of_line_and_utf16_character, to_lower_js, to_upper_js, truncate_by_runes,
    EscapeFlags, JsString, LineAndCharacter, LspLineMap, PositionEncoding, PositionMap, QuoteChar,
    SourceText, Validity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Metrics {
    pub source_decoding: bool,
    pub helper_semantics: bool,
    pub slice_validity: bool,
    pub utf8_positions: bool,
    pub utf16_positions: bool,
}

pub fn run_scenarios() -> Metrics {
    source_decoding();
    helper_semantics();
    slice_validity();
    utf8_positions();
    utf16_positions();
    Metrics {
        source_decoding: true,
        helper_semantics: true,
        slice_validity: true,
        utf8_positions: true,
        utf16_positions: true,
    }
}

fn source_decoding() {
    assert_eq!(SourceText::from_bytes(b"plain").as_bytes(), b"plain");
    assert_eq!(
        SourceText::from_bytes(&[0xEF, 0xBB, 0xBF, b'a', 0xFF]).as_bytes(),
        &[b'a', 0xFF]
    );
    assert!(!SourceText::from_bytes(&[0xFF]).is_utf8());
    assert_eq!(
        SourceText::from_bytes(&[0xFF, 0xFE, 0x61, 0x00, 0x3D, 0xD8, 0x00, 0xDE]).as_str(),
        Some("a😀")
    );
    assert_eq!(
        SourceText::from_bytes(&[0xFE, 0xFF, 0x00, 0x61, 0xD8, 0x00]).as_str(),
        Some("a�")
    );
    assert_eq!(
        SourceText::from_bytes(&[0xFF, 0xFE, 0x61, 0x00, 0xFF]).as_str(),
        Some("a")
    );
}

fn helper_semantics() {
    let high = encode_js_string_rune(0xD83D);
    let low = encode_js_string_rune(0xDE00);
    let mut pair = high.as_bytes().to_vec();
    pair.extend_from_slice(low.as_bytes());
    assert_eq!(
        combine_surrogate_pairs(&JsString::from_bytes(pair)).as_str(),
        Some("😀")
    );
    assert_eq!(combine_surrogate_pairs(&high).as_bytes(), high.as_bytes());

    assert_eq!(
        to_lower_js(&JsString::from("HELLO")).as_str(),
        Some("hello")
    );
    assert_eq!(
        to_lower_js(&JsString::from("İSPANYOL ΟΣ")).as_str(),
        Some("i̇spanyol ος")
    );
    assert_eq!(to_lower_js(&JsString::from("ΣA")).as_str(), Some("σa"));
    assert_eq!(to_upper_js(&JsString::from("ßﬁω")).as_str(), Some("SSFIΩ"));
    assert_eq!(to_lower_js(&high).as_bytes(), high.as_bytes());
    assert_eq!(to_upper_js(&low).as_bytes(), low.as_bytes());
    assert_eq!(
        to_lower_js(&JsString::from_bytes([0xFF])).as_bytes(),
        "�".as_bytes()
    );
    assert_eq!(
        to_upper_js(&JsString::from_bytes([0xFF])).as_bytes(),
        "�".as_bytes()
    );

    assert_eq!(truncate_by_runes(&high, 1).as_bytes(), &[0xED]);
    assert_eq!(
        truncate_by_runes(&JsString::from_bytes([0xFF, 0xFE]), 1).as_bytes(),
        &[0xFF]
    );
    assert_eq!(
        lower_first_char(&JsString::from("Hello")).as_str(),
        Some("hello")
    );
    assert_eq!(
        lower_first_char(&high).as_bytes(),
        &[0xEF, 0xBF, 0xBD, 0xA0, 0xBD]
    );

    assert_eq!(
        escape_string(&high, QuoteChar::Double).as_bytes(),
        br"\uD83D"
    );
    assert_eq!(
        escape_string(&JsString::from_bytes([0xFF]), QuoteChar::Double).as_bytes(),
        br"\uFFFD"
    );
    assert_eq!(
        escape_string(&JsString::from("a\"b\n"), QuoteChar::Double).as_bytes(),
        br#"a\"b\n"#
    );
    assert_eq!(
        escape_non_ascii_string(&JsString::from("é😀"), QuoteChar::Double).as_bytes(),
        br"\u00E9\uD83D\uDE00"
    );
    assert_eq!(
        escape_string_worker(
            &JsString::from("\"'"),
            QuoteChar::Double,
            EscapeFlags::JSX_ATTRIBUTE | EscapeFlags::NEVER_ASCII_ESCAPE,
        )
        .as_bytes(),
        b"&quot;'"
    );
}

fn slice_validity() {
    let source = SourceText::from_bytes("a😀z".as_bytes());
    assert_eq!(source.slice(1..5).unwrap().as_str(), Some("😀"));
    assert!(source.slice(2..5).unwrap().as_str().is_none());
    assert_eq!(
        source.slice(2..5).unwrap().to_js_string().validity(),
        Validity::Raw
    );
    assert!(source.slice(5..99).is_err());

    let sentinel = encode_js_string_rune(0xD800);
    assert_eq!(sentinel.validity(), Validity::Wtf8);
    assert_eq!(sentinel.slice(0..3).unwrap().validity(), Validity::Wtf8);
    assert_eq!(sentinel.slice(0..1).unwrap().validity(), Validity::Raw);
    assert!(sentinel.as_str().is_none());
}

fn utf8_positions() {
    let text = b"a\r\nb\rc\nd";
    assert_eq!(compute_ecma_line_starts(text), vec![0, 3, 5, 7]);
    assert_eq!(LspLineMap::new(text).line_starts(), &[0, 3, 5, 7]);

    let separators = "a\u{2028}b\u{2029}c".as_bytes();
    assert_eq!(compute_ecma_line_starts(separators), vec![0, 4, 8]);
    assert_eq!(LspLineMap::new(separators).line_starts(), &[0]);

    let map = LspLineMap::new("ab\né".as_bytes());
    assert_eq!(
        lsp_line_and_character_to_position(
            "ab\né".as_bytes(),
            &map,
            LineAndCharacter {
                line: 99,
                character: 0
            },
            PositionEncoding::Utf8,
        ),
        5
    );
    assert_eq!(
        lsp_line_and_character_to_position(
            "ab\né".as_bytes(),
            &map,
            LineAndCharacter {
                line: 0,
                character: 99
            },
            PositionEncoding::Utf8,
        ),
        3
    );
    assert_eq!(
        lsp_position_to_line_and_character("ab\né".as_bytes(), &map, -5, PositionEncoding::Utf8),
        LineAndCharacter {
            line: 0,
            character: 0
        }
    );
    assert_eq!(
        lsp_position_to_line_and_character("ab\né".as_bytes(), &map, 99, PositionEncoding::Utf8),
        LineAndCharacter {
            line: 1,
            character: 2
        }
    );

    assert_eq!(
        scanner_position_of_line_and_utf16_character(&[0, 3], 99, 0, b"ab\nc", true),
        3
    );
    assert!(catch_unwind(AssertUnwindSafe(|| {
        scanner_position_of_line_and_utf16_character(&[0, 3], 99, 0, b"ab\nc", false)
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        scanner_position_of_line_and_utf16_character(&[0], 0, 99, b"abc", false)
    }))
    .is_err());
    assert_eq!(
        scanner_position_of_line_and_utf16_character(&[0], 0, 99, b"abc", true),
        3
    );
}

fn utf16_positions() {
    let astral = "😀".as_bytes();
    let positions = PositionMap::new(astral);
    assert!(!positions.is_ascii_only());
    assert_eq!(positions.utf16_to_utf8(1), 1);
    assert_eq!(positions.utf8_to_utf16(1), 1);
    assert_eq!(positions.utf8_to_utf16(4), 2);

    let lsp = LspLineMap::new(astral);
    assert_eq!(
        lsp_line_and_character_to_position(
            astral,
            &lsp,
            LineAndCharacter {
                line: 0,
                character: 1
            },
            PositionEncoding::Utf16,
        ),
        0
    );
    assert_eq!(
        scanner_position_of_line_and_utf16_character(&[0], 0, 1, astral, false),
        4
    );

    let sentinel = encode_js_string_rune(0xD800);
    let positions = PositionMap::new(sentinel.as_bytes());
    assert_eq!(positions.utf8_to_utf16(3), 1);
    assert_eq!(positions.utf16_to_utf8(1), 3);
    let lsp = LspLineMap::new(sentinel.as_bytes());
    assert_eq!(
        lsp_line_and_character_to_position(
            sentinel.as_bytes(),
            &lsp,
            LineAndCharacter {
                line: 0,
                character: 1
            },
            PositionEncoding::Utf16,
        ),
        1
    );
    assert_eq!(
        lsp_position_to_line_and_character(sentinel.as_bytes(), &lsp, 3, PositionEncoding::Utf16),
        LineAndCharacter {
            line: 0,
            character: 3
        }
    );

    let text = "a😀b".as_bytes();
    let starts = compute_ecma_line_starts(text);
    assert_eq!(
        ecma_line_and_utf16_character_of_position(text, &starts, 5),
        (0, 3)
    );
    let lsp = LspLineMap::new(text);
    assert_eq!(
        lsp_position_to_line_and_character(text, &lsp, 2, PositionEncoding::Utf16),
        LineAndCharacter {
            line: 0,
            character: 2
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scenarios() {
        assert_eq!(
            run_scenarios(),
            Metrics {
                source_decoding: true,
                helper_semantics: true,
                slice_validity: true,
                utf8_positions: true,
                utf16_positions: true,
            }
        );
    }
}
