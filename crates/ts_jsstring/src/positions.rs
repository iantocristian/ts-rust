//! Byte and UTF-16 positions retain the distinct API, LSP and scanner contracts.
//!
//! `i32` line starts and LSP offsets match Go's `core.TextPos`; scanner and API
//! offsets use `isize`, matching Go's `int`. Text stays a byte slice because
//! converted offsets need not be UTF-8 boundaries.

use crate::strings::{decode_rune, decode_utf8};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
struct PositionMapEntry {
    utf8_pos: isize,
    delta: isize,
}

#[derive(Clone, Debug)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

impl PositionMap {
    /// port: tsc/internal/ast/positionmap.go:ComputePositionMap
    pub fn new(text: &[u8]) -> Self {
        let mut entries = Vec::new();
        let mut delta = 0;
        let mut pos = 0;
        while pos < text.len() {
            if text[pos].is_ascii() {
                pos += 1;
                continue;
            }
            let (rune, size) = decode_rune(&text[pos..]);
            let utf16_size = if rune >= 0x10000 { 2 } else { 1 };
            delta += size as isize - utf16_size;
            pos += size;
            entries.push(PositionMapEntry {
                utf8_pos: pos as isize,
                delta,
            });
        }
        Self {
            ascii_only: entries.is_empty(),
            entries,
        }
    }

    /// port: tsc/internal/ast/positionmap.go:PositionMap.IsAsciiOnly
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    /// The cumulative delta applies only after a complete sequence. No clamping
    /// or boundary rounding is performed, including on negative offsets.
    ///
    /// port: tsc/internal/ast/positionmap.go:PositionMap.UTF8ToUTF16
    pub fn utf8_to_utf16(&self, offset: isize) -> isize {
        let end = self
            .entries
            .partition_point(|entry| entry.utf8_pos <= offset);
        if end == 0 {
            offset
        } else {
            offset.wrapping_sub(self.entries[end - 1].delta)
        }
    }

    /// port: tsc/internal/ast/positionmap.go:PositionMap.UTF16ToUTF8
    pub fn utf16_to_utf8(&self, offset: isize) -> isize {
        let end = self
            .entries
            .partition_point(|entry| entry.utf8_pos - entry.delta <= offset);
        if end == 0 {
            offset
        } else {
            offset.wrapping_add(self.entries[end - 1].delta)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspLineMap {
    pub line_starts: Vec<i32>,
    pub ascii_only: bool,
}

impl LspLineMap {
    /// port: tsc/internal/ls/lsconv/linemap.go:ComputeLSPLineStarts
    pub fn new(text: &[u8]) -> Self {
        Self::with_source_length(text, text.len())
    }

    // Keep the narrowing conversion at the scan boundary. The separate length
    // also lets tests exercise wrapped TextPos limits without allocating GiBs.
    fn with_source_length(text: &[u8], source_length: usize) -> Self {
        let mut line_starts = Vec::with_capacity(line_capacity(text));
        let mut ascii_only = true;
        let text_len = source_length as i32;
        let mut pos = 0_i32;
        let mut line_start = 0;
        while pos < text_len {
            let byte = text[pos as usize];
            if byte.is_ascii() {
                pos = pos.wrapping_add(1);
                if byte == b'\r' {
                    if pos < text_len && text[pos as usize] == b'\n' {
                        pos = pos.wrapping_add(1);
                    }
                    line_starts.push(line_start);
                    line_start = pos;
                } else if byte == b'\n' {
                    line_starts.push(line_start);
                    line_start = pos;
                }
            } else {
                pos = pos.wrapping_add(decode_utf8(&text[pos as usize..]).1 as i32);
                ascii_only = false;
            }
        }
        line_starts.push(line_start);
        Self {
            line_starts,
            ascii_only,
        }
    }

    /// port: tsc/internal/ls/lsconv/linemap.go:LSPLineMap.ComputeIndexOfLineStart
    pub fn compute_index_of_line_start(&self, target: i32) -> usize {
        let index = self.line_starts.partition_point(|&start| start < target);
        if self.line_starts.get(index) != Some(&target) && index > 0 {
            index - 1
        } else {
            index
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// Conversion for a raw, non-content-mapped script. The caller supplies the
/// corresponding source line map, just as the upstream converter's cache does.
/// The wire's unsigned fields are cast to signed `TextPos` before conversion.
///
/// port: tsc/internal/ls/lsconv/converters.go:Converters.lineAndCharacterToPosition
pub fn lsp_line_and_character_to_position(
    text: &[u8],
    line_map: &LspLineMap,
    position: LspPosition,
    encoding: PositionEncoding,
) -> i32 {
    let line = position.line as i32;
    let character = position.character as i32;
    if line as isize >= line_map.line_starts.len() as isize {
        return text.len() as i32;
    }
    let start = line_map.line_starts[line as usize];
    let line_end = line_map
        .line_starts
        .get(line as usize + 1)
        .copied()
        .unwrap_or(text.len() as i32);
    if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        return start.max(start.wrapping_add(character).min(line_end));
    }
    let mut count = 0_i32;
    let mut pos = start as isize;
    let end = line_end as isize;
    while pos < end {
        let (rune, size) = decode_utf8(&text[pos as usize..]);
        let units = utf16_rune_len(rune);
        if count.wrapping_add(units) > character {
            break;
        }
        count = count.wrapping_add(units);
        pos = pos.wrapping_add(size as isize);
    }
    pos as i32
}

/// The absolute offset is clamped first. Counting a byte prefix deliberately
/// treats a cut UTF-8 sequence as one replacement rune for each remaining byte.
///
/// port: tsc/internal/ls/lsconv/converters.go:Converters.positionToLineAndCharacter
pub fn lsp_position_to_line_and_character(
    text: &[u8],
    line_map: &LspLineMap,
    position: i32,
    encoding: PositionEncoding,
) -> LspPosition {
    let position = position.min(text.len() as i32).max(0);
    let insertion = line_map
        .line_starts
        .partition_point(|&start| start < position);
    let mut line = insertion as isize;
    if line_map.line_starts.get(insertion) != Some(&position) {
        line -= 1;
    }
    line = line.min(line_map.line_starts.len() as isize - 1).max(0);
    let start = line_map.line_starts[line as usize];
    let character = if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        position.wrapping_sub(start)
    } else {
        utf16_len(&text[start as usize..position as usize]) as i32
    };
    LspPosition {
        line: line as u32,
        character: character as u32,
    }
}

/// port: tsc/internal/core/core.go:ComputeECMALineStarts
pub fn compute_ecma_line_starts(text: &[u8]) -> Vec<i32> {
    let mut starts = Vec::with_capacity(line_capacity(text));
    starts.extend(compute_ecma_line_starts_seq(text));
    starts
}

/// Includes the final line, even for empty text or a trailing terminator.
///
/// port: tsc/internal/core/core.go:ComputeECMALineStartsSeq
pub fn compute_ecma_line_starts_seq(text: &[u8]) -> impl Iterator<Item = i32> + '_ {
    ecma_line_starts_with_source_length(text, text.len())
}

fn ecma_line_starts_with_source_length(
    text: &[u8],
    source_length: usize,
) -> impl Iterator<Item = i32> + '_ {
    let text_len = source_length as i32;
    let mut pos = 0_i32;
    let mut line_start = 0;
    let mut finished = false;
    std::iter::from_fn(move || {
        if finished {
            return None;
        }
        while pos < text_len {
            let byte = text[pos as usize];
            let line_break = if byte.is_ascii() {
                pos = pos.wrapping_add(1);
                if byte == b'\r' && pos < text_len && text[pos as usize] == b'\n' {
                    pos = pos.wrapping_add(1);
                }
                byte == b'\r' || byte == b'\n'
            } else {
                let (rune, size) = decode_utf8(&text[pos as usize..]);
                pos = pos.wrapping_add(size as i32);
                is_line_break(rune)
            };
            if line_break {
                let previous_start = line_start;
                line_start = pos;
                return Some(previous_start);
            }
        }
        finished = true;
        Some(line_start)
    })
}

/// This core helper clamps the line index to zero; the scanner's otherwise
/// similar `compute_line_of_position` returns -1 for a negative position.
///
/// port: tsc/internal/core/core.go:PositionToLineAndByteOffset
pub fn position_to_line_and_byte_offset(position: isize, line_starts: &[i32]) -> (isize, isize) {
    let line = line_starts
        .partition_point(|&start| start as isize <= position)
        .saturating_sub(1);
    (
        line as isize,
        position.wrapping_sub(line_starts[line] as isize),
    )
}

/// Counts standard UTF-8 decoding, including one unit per malformed byte.
/// A complete WTF-8 surrogate sentinel therefore counts as three units here.
///
/// port: tsc/internal/core/core.go:UTF16Len
pub fn utf16_len(text: &[u8]) -> isize {
    let Some(mut pos) = text.iter().position(|byte| !byte.is_ascii()) else {
        return text.len() as isize;
    };
    let mut count = pos as isize;
    while pos < text.len() {
        let (rune, size) = decode_utf8(&text[pos..]);
        pos += size;
        count += utf16_rune_len(rune) as isize;
    }
    count
}

/// port: tsc/internal/scanner/scanner.go:ComputeLineOfPosition
pub fn compute_line_of_position(line_starts: &[i32], position: isize) -> isize {
    let mut low = 0;
    let mut high = line_starts.len() as isize - 1;
    while low <= high {
        let middle = low + ((high - low) >> 1);
        let value = line_starts[middle as usize] as isize;
        match value.cmp(&position) {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle - 1,
            Ordering::Equal => return middle,
        }
    }
    low - 1
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineOfPosition
pub fn get_ecma_line_of_position(text: &[u8], position: isize) -> isize {
    compute_line_of_position(&compute_ecma_line_starts(text), position)
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineAndUTF16CharacterOfPosition
pub fn get_ecma_line_and_utf16_character_of_position(
    text: &[u8],
    position: isize,
) -> (isize, isize) {
    let line_starts = compute_ecma_line_starts(text);
    let line = compute_line_of_position(&line_starts, position);
    let start = line_starts[line as usize] as usize;
    (line, utf16_len(&text[start..position as usize]))
}

/// port: tsc/internal/scanner/scanner.go:GetECMALineAndByteOffsetOfPosition
pub fn get_ecma_line_and_byte_offset_of_position(text: &[u8], position: isize) -> (isize, isize) {
    let line_starts = compute_ecma_line_starts(text);
    let line = compute_line_of_position(&line_starts, position);
    (
        line,
        position.wrapping_sub(line_starts[line as usize] as isize),
    )
}

/// Returns the byte preceding the first terminator or EOF. An empty first line
/// returns -1; a multi-byte final character returns its final byte's offset.
///
/// port: tsc/internal/scanner/scanner.go:GetECMAEndLinePosition
pub fn get_ecma_end_line_position(text: &[u8], line: isize) -> isize {
    let line_starts = compute_ecma_line_starts(text);
    let mut pos = line_starts[line as usize] as usize;
    loop {
        let (rune, size) = decode_utf8(&text[pos..]);
        if size == 0 || is_line_break(rune) {
            return pos as isize - 1;
        }
        pos += size;
    }
}

/// port: tsc/internal/scanner/scanner.go:GetECMAPositionOfLineAndUTF16Character
pub fn get_ecma_position_of_line_and_utf16_character(
    text: &[u8],
    line: isize,
    character: isize,
) -> isize {
    compute_position_of_line_and_utf16_character(
        &compute_ecma_line_starts(text),
        line,
        character,
        text,
        false,
    )
}

/// port: tsc/internal/scanner/scanner.go:GetECMAPositionOfLineAndByteOffset
pub fn get_ecma_position_of_line_and_byte_offset(
    text: &[u8],
    line: isize,
    byte_offset: isize,
) -> isize {
    compute_position_of_line_and_byte_offset(&compute_ecma_line_starts(text), line, byte_offset)
}

/// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndByteOffset
pub fn compute_position_of_line_and_byte_offset(
    line_starts: &[i32],
    line: isize,
    byte_offset: isize,
) -> isize {
    assert_valid_line(line_starts, line);
    (line_starts[line as usize] as isize).wrapping_add(byte_offset)
}

/// Unlike LSP this consumes a whole rune before checking the UTF-16 count, so
/// an offset inside an astral pair rounds forward. Line terminators are part of
/// the scanned line. A nonpositive character returns the line start.
///
/// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndUTF16Character
pub fn compute_position_of_line_and_utf16_character(
    line_starts: &[i32],
    mut line: isize,
    character: isize,
    text: &[u8],
    allow_edits: bool,
) -> isize {
    if line < 0 || line >= line_starts.len() as isize {
        if allow_edits {
            if line < 0 {
                line = 0;
            } else {
                line = line_starts.len() as isize - 1;
            }
        } else {
            assert_valid_line(line_starts, line);
        }
    }
    let line_start = line_starts[line as usize] as isize;
    if character > 0 {
        let line_end = line_starts
            .get(line as usize + 1)
            .map_or(text.len() as isize, |&start| start as isize);
        let mut count = 0;
        let mut pos = line_start;
        while pos < line_end {
            if count >= character {
                break;
            }
            let (rune, size) = decode_utf8(&text[pos as usize..]);
            count += utf16_rune_len(rune) as isize;
            pos += size as isize;
        }
        if !allow_edits {
            assert!(
                pos != line_end || count >= character,
                "Bad UTF-16 character offset. Line: {line}, character: {character}."
            );
            assert!(pos <= text.len() as isize);
            return pos;
        }
        return pos.min(text.len() as isize);
    }
    if allow_edits {
        line_start.min(text.len() as isize)
    } else {
        assert!(line_start <= text.len() as isize);
        line_start
    }
}

fn assert_valid_line(line_starts: &[i32], line: isize) {
    assert!(
        line >= 0 && line < line_starts.len() as isize,
        "Bad line number. Line: {line}, lineStarts.length: {}.",
        line_starts.len()
    );
}

fn utf16_rune_len(rune: i32) -> i32 {
    // Every rune supplied here comes from standard UTF-8 decoding, so it is
    // either a Unicode scalar value or U+FFFD, never a surrogate or negative.
    if rune >= 0x10000 {
        2
    } else {
        1
    }
}

fn is_line_break(rune: i32) -> bool {
    matches!(rune, 0x0A | 0x0D | 0x2028 | 0x2029)
}

#[expect(
    clippy::naive_bytecount,
    reason = "This is only a capacity hint; the leaf crate needs no bytecount dependency."
)]
fn line_capacity(text: &[u8]) -> usize {
    text.iter().filter(|&&byte| byte == b'\n').count() + 1
}

#[cfg(test)]
#[path = "positions_tests.rs"]
mod tests;
