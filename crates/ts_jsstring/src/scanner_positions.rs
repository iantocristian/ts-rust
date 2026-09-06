//! port: tsc/internal/scanner/scanner.go (line and UTF-16 character utilities)
//!
//! The scanner's converters consume a whole character before checking the
//! accumulated count, so an interior astral offset rounds forward. Out-of-range
//! inputs clamp when `allow_edits` is set and panic otherwise, with upstream's
//! messages (docs/design/text.md, 2.3).

use crate::line_map::{compute_line_of_position, utf16_len, TextPos};
use crate::wtf8::{decode_rune, is_line_break, utf16_rune_len};

/// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndByteOffset
///
/// # Panics
/// On a line number outside the map, as upstream does.
pub fn compute_position_of_line_and_byte_offset(
    line_starts: &[TextPos],
    line: i64,
    byte_offset: i64,
) -> i64 {
    let len = line_starts.len() as i64;
    assert!(
        line >= 0 && line < len,
        "Bad line number. Line: {line}, lineStarts.length: {len}."
    );
    line_starts[line as usize] as i64 + byte_offset
}

/// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndUTF16Character
///
/// # Panics
/// Without `allow_edits`, on a bad line number or a character offset past the
/// end of the line, as upstream does.
pub fn compute_position_of_line_and_utf16_character(
    line_starts: &[TextPos],
    line: i64,
    character: i64,
    text: &[u8],
    allow_edits: bool,
) -> usize {
    let len = line_starts.len() as i64;
    let mut line = line;
    if line < 0 || line >= len {
        if allow_edits {
            line = line.clamp(0, len - 1);
        } else {
            panic!("Bad line number. Line: {line}, lineStarts.length: {len}.");
        }
    }
    let line = line as usize;
    let line_start = line_starts[line];

    if character > 0 {
        let line_end = if line + 1 < line_starts.len() {
            line_starts[line + 1]
        } else {
            text.len()
        };
        let mut utf16_count = 0i64;
        let mut pos = line_start;
        while pos < line_end {
            if utf16_count >= character {
                break;
            }
            let (r, size) = decode_rune(&text[pos..]);
            utf16_count += utf16_rune_len(r);
            pos += size;
        }
        if !allow_edits {
            assert!(
                !(pos == line_end && utf16_count < character),
                "Bad UTF-16 character offset. Line: {line}, character: {character}."
            );
            assert!(pos <= text.len());
            return pos;
        }
        return pos.min(text.len());
    }

    let res = line_start;
    if allow_edits {
        return res.min(text.len());
    }
    assert!(res <= text.len());
    res
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineOfPosition
pub fn get_ecma_line_of_position(line_starts: &[TextPos], pos: usize) -> usize {
    compute_line_of_position(line_starts, pos)
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineAndUTF16CharacterOfPosition
///
/// # Panics
/// When `pos` precedes its line start, as the Go slice expression does.
pub fn get_ecma_line_and_utf16_character_of_position(
    text: &[u8],
    line_starts: &[TextPos],
    pos: usize,
) -> (usize, usize) {
    let line = compute_line_of_position(line_starts, pos);
    (line, utf16_len(&text[line_starts[line]..pos]))
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineAndByteOffsetOfPosition
pub fn get_ecma_line_and_byte_offset_of_position(
    line_starts: &[TextPos],
    pos: usize,
) -> (usize, i64) {
    let line = compute_line_of_position(line_starts, pos);
    (line, pos as i64 - line_starts[line] as i64)
}

/// port: tsc/internal/scanner/scanner.go:GetECMAEndLinePosition
pub fn get_ecma_end_line_position(text: &[u8], line_starts: &[TextPos], line: usize) -> i64 {
    let mut pos = line_starts[line];
    loop {
        let (ch, size) = decode_rune(&text[pos.min(text.len())..]);
        if size == 0 || is_line_break(ch) {
            return pos as i64 - 1;
        }
        pos += size;
    }
}

/// port: tsc/internal/scanner/scanner.go:GetECMAPositionOfLineAndUTF16Character
pub fn get_ecma_position_of_line_and_utf16_character(
    text: &[u8],
    line_starts: &[TextPos],
    line: i64,
    character: i64,
) -> usize {
    compute_position_of_line_and_utf16_character(line_starts, line, character, text, false)
}

/// port: tsc/internal/scanner/scanner.go:GetECMAPositionOfLineAndByteOffset
pub fn get_ecma_position_of_line_and_byte_offset(
    line_starts: &[TextPos],
    line: i64,
    byte_offset: i64,
) -> i64 {
    compute_position_of_line_and_byte_offset(line_starts, line, byte_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_map::compute_ecma_line_starts;

    #[test]
    fn astral_interior_offset_rounds_forward() {
        let text = "😀".as_bytes();
        let starts = compute_ecma_line_starts(text);
        assert_eq!(
            compute_position_of_line_and_utf16_character(&starts, 0, 1, text, false),
            4
        );
        assert_eq!(
            get_ecma_line_and_utf16_character_of_position(text, &starts, 4),
            (0, 2)
        );
    }

    #[test]
    fn clamp_versus_panic() {
        let text = b"ab\ncd";
        let starts = compute_ecma_line_starts(text);
        assert_eq!(
            compute_position_of_line_and_utf16_character(&starts, 9, 1, text, true),
            4
        );
        assert_eq!(
            compute_position_of_line_and_utf16_character(&starts, 0, 9, text, true),
            3
        );
        let bad_line = std::panic::catch_unwind(|| {
            compute_position_of_line_and_utf16_character(&starts, 9, 1, text, false)
        });
        assert!(bad_line.is_err());
        let bad_char = std::panic::catch_unwind(|| {
            compute_position_of_line_and_utf16_character(&starts, 0, 9, text, false)
        });
        assert!(bad_char.is_err());
        assert_eq!(get_ecma_end_line_position(text, &starts, 0), 1);
    }
}
