//! port: tsc/internal/ls/lsconv/converters.go (raw coordinate conversion)
//!
//! The LSP converter counts UTF-16 units with the standard decoder, so a
//! sentinel counts as three units. It stops before a character that would
//! exceed the requested count, so an interior astral offset maps to the
//! character's start (docs/design/text.md, 2.3).

use crate::line_map::{LspLineMap, TextPos};
use crate::wtf8::{decode_rune, utf16_rune_len};

/// The negotiated `PositionEncodingKind`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
}

/// port: tsc/internal/ls/lsconv/converters.go:Converters.lineAndCharacterToPosition
///
/// Upstream converts the wire's `uint32` line and character to `core.TextPos`,
/// an `int32`, so values at or above 2^31 wrap negative: a negative line indexes
/// the line-start table out of range and panics, and a negative character
/// clamps to the line start. Both are preserved.
///
/// # Panics
/// When the wrapped line number is negative, as the Go slice index does.
pub fn line_and_character_to_position(
    text: &[u8],
    line_map: &LspLineMap,
    encoding: PositionEncoding,
    line: u32,
    character: u32,
) -> TextPos {
    let line = i64::from(line as i32);
    let char = i64::from(character as i32);
    let text_len = text.len() as i64;
    let line_count = line_map.line_starts.len() as i64;

    if line >= line_count {
        return text_len as usize;
    }
    assert!(
        line >= 0,
        "runtime error: index out of range [{line}] with length {line_count}"
    );
    let line = line as usize;

    let start = line_map.line_starts[line] as i64;
    let line_end = if line + 1 < line_map.line_starts.len() {
        line_map.line_starts[line + 1] as i64
    } else {
        text_len
    };

    if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        return start.max((start + char).min(line_end)) as usize;
    }

    let mut utf16_char = 0i64;
    let mut pos = start as usize;
    let end = line_end as usize;
    while pos < end {
        let (r, size) = decode_rune(&text[pos..]);
        let u16_len = utf16_rune_len(r);
        if utf16_char + u16_len > char {
            break;
        }
        utf16_char += u16_len;
        pos += size;
    }
    pos
}

/// port: tsc/internal/ls/lsconv/converters.go:Converters.positionToLineAndCharacter
pub fn position_to_line_and_character(
    text: &[u8],
    line_map: &LspLineMap,
    encoding: PositionEncoding,
    position: usize,
) -> (u32, u32) {
    let position = position.min(text.len());
    let line = match line_map.line_starts.binary_search(&position) {
        Ok(line) => line as i64,
        Err(insertion) => insertion as i64 - 1,
    };
    let line = line.clamp(0, line_map.line_starts.len() as i64 - 1) as usize;
    let start = line_map.line_starts[line];
    let character = if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        position - start
    } else {
        let mut character = 0;
        let mut i = start;
        while i < position {
            let (r, size) = decode_rune(&text[i..position]);
            character += utf16_rune_len(r).max(0) as usize;
            i += size;
        }
        character
    };
    (line as u32, character as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_map::compute_lsp_line_starts;

    #[test]
    fn astral_interior_offset_maps_to_zero() {
        let text = "😀".as_bytes();
        let map = compute_lsp_line_starts(text);
        assert_eq!(
            line_and_character_to_position(text, &map, PositionEncoding::Utf16, 0, 1),
            0
        );
        assert_eq!(
            line_and_character_to_position(text, &map, PositionEncoding::Utf16, 0, 2),
            4
        );
        assert_eq!(
            position_to_line_and_character(text, &map, PositionEncoding::Utf16, 4),
            (0, 2)
        );
    }

    #[test]
    fn sentinel_is_three_units() {
        let text = b"\xED\xA0\x80x";
        let map = compute_lsp_line_starts(text);
        assert_eq!(
            position_to_line_and_character(text, &map, PositionEncoding::Utf16, 3),
            (0, 3)
        );
        assert_eq!(
            line_and_character_to_position(text, &map, PositionEncoding::Utf16, 0, 3),
            3
        );
        assert_eq!(
            line_and_character_to_position(text, &map, PositionEncoding::Utf8, 0, 3),
            3
        );
    }
}
