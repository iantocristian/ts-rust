//! port: tsc/internal/core/core.go (line starts and UTF-16 length)
//! port: tsc/internal/ls/lsconv/linemap.go
//!
//! Two line maps with different line-break sets: ECMAScript recognizes CR, LF,
//! CRLF, U+2028 and U+2029; LSP recognizes only CR, LF and CRLF and records
//! whether the text is ASCII-only. Both keep byte offsets.

use crate::wtf8::{decode_rune, is_line_break, utf16_rune_len, RUNE_SELF};

/// A byte offset into the source text, Go's `core.TextPos`.
pub type TextPos = usize;

/// port: tsc/internal/core/core.go:ComputeECMALineStarts
/// port: tsc/internal/core/core.go:ComputeECMALineStartsSeq
#[allow(clippy::naive_bytecount)] // a capacity hint over the text, as upstream's strings.Count
pub fn compute_ecma_line_starts(text: &[u8]) -> Vec<TextPos> {
    let mut result = Vec::with_capacity(text.iter().filter(|&&b| b == b'\n').count() + 1);
    let text_len = text.len();
    let mut pos = 0;
    let mut line_start = 0;
    while pos < text_len {
        let b = text[pos];
        if b < RUNE_SELF {
            pos += 1;
            match b {
                b'\r' => {
                    if pos < text_len && text[pos] == b'\n' {
                        pos += 1;
                    }
                    result.push(line_start);
                    line_start = pos;
                }
                b'\n' => {
                    result.push(line_start);
                    line_start = pos;
                }
                _ => {}
            }
        } else {
            let (ch, size) = decode_rune(&text[pos..]);
            pos += size;
            if is_line_break(ch) {
                result.push(line_start);
                line_start = pos;
            }
        }
    }
    result.push(line_start);
    result
}

/// The `LSPLineMap` type of tsc/internal/ls/lsconv/linemap.go.
pub struct LspLineMap {
    pub line_starts: Vec<TextPos>,
    pub ascii_only: bool,
}

/// port: tsc/internal/ls/lsconv/linemap.go:ComputeLSPLineStarts
#[allow(clippy::naive_bytecount)] // a capacity hint over the text, as upstream's strings.Count
pub fn compute_lsp_line_starts(text: &[u8]) -> LspLineMap {
    let mut line_starts = Vec::with_capacity(text.iter().filter(|&&b| b == b'\n').count() + 1);
    let mut ascii_only = true;
    let text_len = text.len();
    let mut pos = 0;
    let mut line_start = 0;
    while pos < text_len {
        let b = text[pos];
        if b < RUNE_SELF {
            pos += 1;
            match b {
                b'\r' => {
                    if pos < text_len && text[pos] == b'\n' {
                        pos += 1;
                    }
                    line_starts.push(line_start);
                    line_start = pos;
                }
                b'\n' => {
                    line_starts.push(line_start);
                    line_start = pos;
                }
                _ => {}
            }
        } else {
            let (_, size) = decode_rune(&text[pos..]);
            pos += size;
            ascii_only = false;
        }
    }
    line_starts.push(line_start);
    LspLineMap {
        line_starts,
        ascii_only,
    }
}

impl LspLineMap {
    /// port: tsc/internal/ls/lsconv/linemap.go:LSPLineMap.ComputeIndexOfLineStart
    pub fn compute_index_of_line_start(&self, target_pos: TextPos) -> usize {
        match self.line_starts.binary_search(&target_pos) {
            Ok(line) => line,
            Err(insertion) => insertion.saturating_sub(1),
        }
    }
}

/// port: tsc/internal/scanner/scanner.go:ComputeLineOfPosition
pub fn compute_line_of_position(line_starts: &[TextPos], pos: usize) -> usize {
    let mut low = 0i64;
    let mut high = line_starts.len() as i64 - 1;
    while low <= high {
        let middle = low + ((high - low) >> 1);
        let value = line_starts[middle as usize];
        match value.cmp(&pos) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle - 1,
            std::cmp::Ordering::Equal => return middle as usize,
        }
    }
    // Go returns low - 1, which is -1 only for a position before the first line start.
    (low - 1).max(0) as usize
}

/// port: tsc/internal/core/core.go:PositionToLineAndByteOffset
pub fn position_to_line_and_byte_offset(
    position: usize,
    line_starts: &[TextPos],
) -> (usize, usize) {
    let line = line_starts
        .partition_point(|&start| start <= position)
        .saturating_sub(1);
    (line, position - line_starts[line])
}

/// port: tsc/internal/core/core.go:UTF16Len
///
/// Go's `range` decodes with the standard decoder, so a malformed byte and each
/// byte of a surrogate sentinel count as one unit.
pub fn utf16_len(s: &[u8]) -> usize {
    let Some(first_non_ascii) = s.iter().position(|&b| b >= RUNE_SELF) else {
        return s.len();
    };
    let mut n = first_non_ascii;
    let mut i = first_non_ascii;
    while i < s.len() {
        let (r, size) = decode_rune(&s[i..]);
        n += utf16_rune_len(r).max(0) as usize;
        i += size;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_break_sets_differ() {
        let text = "a\u{2028}b".as_bytes();
        assert_eq!(compute_ecma_line_starts(text), vec![0, 4]);
        let lsp = compute_lsp_line_starts(text);
        assert_eq!(lsp.line_starts, vec![0]);
        assert!(!lsp.ascii_only);
        assert_eq!(compute_ecma_line_starts(b"a\r\nb\rc\nd"), vec![0, 3, 5, 7]);
    }

    #[test]
    fn utf16_len_counts_bytes_of_sentinels() {
        assert_eq!(utf16_len("😀".as_bytes()), 2);
        assert_eq!(utf16_len(b"\xED\xA0\x80"), 3);
        assert_eq!(utf16_len(b"\xFF"), 1);
    }

    #[test]
    fn line_lookup() {
        let starts = [0usize, 5, 10, 23, 80];
        assert_eq!(compute_line_of_position(&starts, 20), 2);
        assert_eq!(compute_line_of_position(&starts, 10), 2);
        assert_eq!(position_to_line_and_byte_offset(20, &starts), (2, 10));
        let lsp = LspLineMap {
            line_starts: starts.to_vec(),
            ascii_only: true,
        };
        assert_eq!(lsp.compute_index_of_line_start(20), 2);
        assert_eq!(lsp.compute_index_of_line_start(23), 3);
    }
}
