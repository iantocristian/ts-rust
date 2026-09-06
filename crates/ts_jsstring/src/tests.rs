//! Unit tests for the invariants the E4 harness cannot state as an oracle
//! comparison, plus the worked examples the design note fixes by hand.

use crate::escape::{escape_jsx_attribute_string, escape_non_ascii_string};
use crate::helpers::{lower_first_char, truncate_by_runes};
use crate::jsstring::{JsString, Validity};
use crate::positions::{
    compute_ecma_line_starts, compute_lsp_line_starts, compute_position_map,
    compute_position_of_line_and_utf16_character, Converters, PositionEncoding,
};
use crate::rune::{combine_surrogate_pairs, decode_js_string_rune, encode_js_string_rune};
use crate::source_text::{ByteOrderMark, SourceText};
use crate::{escape_string, to_lower_js, to_upper_js, QuoteChar};

/// The three-byte WTF-8 sentinel for a lone high surrogate U+D800.
const HIGH_SENTINEL: &[u8] = b"\xED\xA0\x80";
/// The sentinel for a lone low surrogate U+DC00.
const LOW_SENTINEL: &[u8] = b"\xED\xB0\x80";
/// U+1F600, four UTF-8 bytes and two UTF-16 units.
const GRINNING: &[u8] = "\u{1F600}".as_bytes();

#[test]
fn sentinels_round_trip() {
    assert_eq!(encode_js_string_rune(0xD800), HIGH_SENTINEL);
    assert_eq!(decode_js_string_rune(HIGH_SENTINEL), (0xD800, 3));
    // The standard decoder rejects the same bytes one at a time.
    assert_eq!(
        crate::rune::decode_rune(HIGH_SENTINEL),
        (crate::RUNE_ERROR, 1)
    );
}

#[test]
fn combine_surrogate_pairs_joins_only_adjacent_halves() {
    // U+D800 followed by U+DC00 is the surrogate pair for U+10000.
    let joined = [HIGH_SENTINEL, LOW_SENTINEL].concat();
    assert_eq!(combine_surrogate_pairs(&joined), "\u{10000}".as_bytes());
    // U+1F600's own halves join back to U+1F600.
    let grinning_halves = [b"\xED\xA0\xBD".as_slice(), b"\xED\xB8\x80".as_slice()].concat();
    assert_eq!(combine_surrogate_pairs(&grinning_halves), GRINNING);
    // A high sentinel with no partner survives unchanged.
    assert_eq!(combine_surrogate_pairs(HIGH_SENTINEL), HIGH_SENTINEL);
    // So does a low sentinel that comes first.
    let reversed = [LOW_SENTINEL, HIGH_SENTINEL].concat();
    assert_eq!(combine_surrogate_pairs(&reversed), reversed);
}

#[test]
fn validity_classification() {
    assert_eq!(JsString::from_text("abc").validity(), Validity::Utf8);
    assert_eq!(
        JsString::new(HIGH_SENTINEL.to_vec()).validity(),
        Validity::Wtf8
    );
    assert_eq!(JsString::new(b"\xFF".to_vec()).validity(), Validity::Raw);
    // A sentinel cut in half is neither UTF-8 nor WTF-8.
    assert_eq!(
        JsString::new(b"\xED\xA0".to_vec()).validity(),
        Validity::Raw
    );
}

#[test]
fn slicing_reclassifies_and_never_yields_an_invalid_str() {
    let s = JsString::new([GRINNING, HIGH_SENTINEL].concat());
    assert_eq!(s.validity(), Validity::Wtf8);
    assert!(s.as_str().is_none());

    let head = s.slice(0, 4).unwrap();
    assert_eq!(head.validity(), Validity::Utf8);
    assert_eq!(head.as_str(), Some("\u{1F600}"));

    // Splitting the astral character leaves representable but raw bytes.
    let split = s.slice(0, 2).unwrap();
    assert_eq!(split.validity(), Validity::Raw);
    assert!(split.as_str().is_none());
    assert_eq!(split.as_bytes(), &GRINNING[..2]);

    // Splitting the sentinel does too.
    let cut = s.slice(4, 6).unwrap();
    assert_eq!(cut.validity(), Validity::Raw);
    assert!(cut.as_str().is_none());

    assert!(s.slice(0, s.len() + 1).is_none());
    assert!(s.slice(3, 2).is_none());
}

#[test]
fn helper_replacements_and_fragments() {
    // TruncateByRunes counts each malformed byte separately and can split a
    // sentinel: `ED A0 80` truncated to one rune is just `ED`.
    assert_eq!(truncate_by_runes(HIGH_SENTINEL, 1), b"\xED");
    // The case mappers preserve complete sentinels but replace a stray byte.
    assert_eq!(to_upper_js(HIGH_SENTINEL), HIGH_SENTINEL);
    assert_eq!(to_lower_js(HIGH_SENTINEL), HIGH_SENTINEL);
    assert_eq!(to_upper_js(b"a\xFF"), b"A\xEF\xBF\xBD");
    assert_eq!(to_lower_js(b"A\xFF"), b"a\xEF\xBF\xBD");
    // LowerFirstChar decodes with the standard decoder, so a sentinel prefix
    // becomes one replacement character and the remaining bytes are copied.
    assert_eq!(lower_first_char(HIGH_SENTINEL), b"\xEF\xBF\xBD\xA0\x80");
}

#[test]
fn escaping_surrogates_and_stray_bytes() {
    assert_eq!(
        escape_string(HIGH_SENTINEL, QuoteChar::DoubleQuote),
        b"\\uD800"
    );
    assert_eq!(escape_string(b"\xFF", QuoteChar::DoubleQuote), b"\\uFFFD");
    // A double quote is not escaped inside a single-quoted literal.
    assert_eq!(escape_string(b"\"", QuoteChar::SingleQuote), b"\"");
    assert_eq!(escape_string(b"\"", QuoteChar::DoubleQuote), b"\\\"");
    // Template literals keep a lone LF and still escape CRLF as one unit.
    assert_eq!(escape_string(b"a\nb", QuoteChar::Backtick), b"a\nb");
    assert_eq!(escape_string(b"a\r\nb", QuoteChar::Backtick), b"a\\r\\nb");
    // Non-ASCII escaping is the other flag combination.
    assert_eq!(
        escape_non_ascii_string("\u{00E9}".as_bytes(), QuoteChar::DoubleQuote),
        b"\\u00E9"
    );
    assert_eq!(
        escape_string("\u{00E9}".as_bytes(), QuoteChar::DoubleQuote),
        "\u{00E9}".as_bytes()
    );
    assert_eq!(
        escape_jsx_attribute_string(b"a\"b", QuoteChar::DoubleQuote),
        b"a&quot;b"
    );
}

#[test]
fn bom_decoding() {
    let (text, bom) = SourceText::decode_with_bom(b"\xEF\xBB\xBFabc");
    assert_eq!(text.as_bytes(), b"abc");
    assert_eq!(bom, ByteOrderMark::Utf8);

    let (text, bom) = SourceText::decode_with_bom(b"\xFF\xFEa\x00b\x00");
    assert_eq!(text.as_bytes(), b"ab");
    assert_eq!(bom, ByteOrderMark::Utf16Le);

    let (text, bom) = SourceText::decode_with_bom(b"\xFE\xFF\x00a\x00b");
    assert_eq!(text.as_bytes(), b"ab");
    assert_eq!(bom, ByteOrderMark::Utf16Be);

    // An unpaired surrogate under a UTF-16 BOM becomes U+FFFD, as utf16.Decode
    // produces it; a trailing odd byte is dropped.
    let text = SourceText::decode(b"\xFF\xFE\x00\xD8\x41");
    assert_eq!(text.as_bytes(), b"\xEF\xBF\xBD");

    // Everything else is kept byte for byte, valid UTF-8 or not.
    let text = SourceText::decode(b"\xFFx");
    assert_eq!(text.as_bytes(), b"\xFFx");
    assert!(!text.is_valid_utf8());
    assert!(text.as_str().is_none());
}

#[test]
fn the_three_conversions_disagree_on_an_interior_astral_offset() {
    let text = GRINNING;
    let map = compute_position_map(text);
    assert_eq!(map.utf16_to_utf8(1), 1);

    let lsp = compute_lsp_line_starts(text);
    let converters = Converters::new(PositionEncoding::Utf16);
    assert_eq!(
        converters.line_and_character_to_position(text, &lsp, 0, 1),
        0
    );

    let ecma = compute_ecma_line_starts(text);
    assert_eq!(
        compute_position_of_line_and_utf16_character(&ecma, 0, 1, text, false),
        4
    );
}

#[test]
fn line_maps_differ_on_the_unicode_separators() {
    for separator in ["\u{2028}", "\u{2029}"] {
        let text = format!("a{separator}b");
        assert_eq!(compute_ecma_line_starts(text.as_bytes()), vec![0, 4]);
        assert_eq!(
            compute_lsp_line_starts(text.as_bytes()).line_starts,
            vec![0]
        );
    }
}

#[test]
fn sentinels_count_once_for_the_api_and_three_times_for_the_lsp() {
    let text = HIGH_SENTINEL;
    let map = compute_position_map(text);
    assert_eq!(map.utf8_to_utf16(3), 1);

    let lsp = compute_lsp_line_starts(text);
    let converters = Converters::new(PositionEncoding::Utf16);
    assert_eq!(
        converters.position_to_line_and_character(text, &lsp, 3),
        (0, 3)
    );
}

#[test]
#[should_panic(expected = "Bad line number")]
fn scanner_conversion_panics_on_a_bad_line_without_allow_edits() {
    compute_position_of_line_and_utf16_character(&[0], 4, 0, b"abc", false);
}

#[test]
fn scanner_conversion_clamps_with_allow_edits() {
    assert_eq!(
        compute_position_of_line_and_utf16_character(&[0], 4, 0, b"abc", true),
        0
    );
    assert_eq!(
        compute_position_of_line_and_utf16_character(&[0], 0, 99, b"abc", true),
        3
    );
}
