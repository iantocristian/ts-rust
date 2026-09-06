//! Positions: byte offsets inside the compiler, converted at the edge
//! (docs/design/text.md, section 2.3).
//!
//! The three conversion families deliberately disagree, and each is ported with
//! its own rounding, clamping and panic behavior rather than unified:
//!
//! * the API's `PositionMap` applies the preceding cumulative delta and counts a
//!   WTF-8 sentinel as one UTF-16 unit;
//! * the LSP converters stop before consuming a partial surrogate pair and count
//!   a sentinel as three units, because the standard decoder rejects surrogates;
//! * the scanner consumes a whole character before testing the accumulated
//!   count, so it rounds an interior offset forward.
//!
//! For the single-line text `U+1F600`, UTF-16 offset 1 maps to byte offset 1, 0
//! and 4 respectively.

use crate::rune::{decode_js_string_rune, decode_rune, utf16_rune_len, Rune, Runes, RUNE_SELF};

/// `core.TextPos`.
pub type TextPos = i32;

// ------------------------------------------------------------------ line maps

// port: tsc/internal/stringutil/util.go:IsLineBreak
/// The ECMAScript line terminator set: LF, CR, LS and PS.
pub fn is_line_break(ch: Rune) -> bool {
    matches!(ch, 0x0A | 0x0D | 0x2028 | 0x2029)
}

// port: tsc/internal/core/core.go:ComputeECMALineStarts
// port: tsc/internal/core/core.go:ComputeECMALineStartsSeq
/// Line starts under the ECMAScript line terminator set.
pub fn compute_ecma_line_starts(text: &[u8]) -> Vec<TextPos> {
    // Upstream sizes the vector from a line-feed count; ADR 0017 keeps the
    // dependency set small, so the count stays a plain scan.
    #[allow(clippy::naive_bytecount)]
    let mut line_starts = Vec::with_capacity(text.iter().filter(|b| **b == b'\n').count() + 1);
    let text_len = text.len();
    let mut pos = 0usize;
    let mut line_start = 0usize;
    while pos < text_len {
        let b = text[pos];
        if b < RUNE_SELF {
            pos += 1;
            if b == b'\r' {
                if pos < text_len && text[pos] == b'\n' {
                    pos += 1;
                }
                line_starts.push(line_start as TextPos);
                line_start = pos;
            } else if b == b'\n' {
                line_starts.push(line_start as TextPos);
                line_start = pos;
            }
        } else {
            let (ch, size) = decode_rune(&text[pos..]);
            pos += size;
            if is_line_break(ch) {
                line_starts.push(line_start as TextPos);
                line_start = pos;
            }
        }
    }
    line_starts.push(line_start as TextPos);
    line_starts
}

/// `lsconv.LSPLineMap`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspLineMap {
    pub line_starts: Vec<TextPos>,
    /// Whether the whole text is ASCII; the converters take the byte fast path
    /// when it is.
    pub ascii_only: bool,
}

// port: tsc/internal/ls/lsconv/linemap.go:ComputeLSPLineStarts
/// Line starts under the LSP line terminator set: only CR, LF and CRLF.
pub fn compute_lsp_line_starts(text: &[u8]) -> LspLineMap {
    // Upstream sizes the vector from a line-feed count; ADR 0017 keeps the
    // dependency set small, so the count stays a plain scan.
    #[allow(clippy::naive_bytecount)]
    let mut line_starts = Vec::with_capacity(text.iter().filter(|b| **b == b'\n').count() + 1);
    let mut ascii_only = true;
    let text_len = text.len();
    let mut pos = 0usize;
    let mut line_start = 0usize;
    while pos < text_len {
        let b = text[pos];
        if b < RUNE_SELF {
            pos += 1;
            if b == b'\r' {
                if pos < text_len && text[pos] == b'\n' {
                    pos += 1;
                }
                line_starts.push(line_start as TextPos);
                line_start = pos;
            } else if b == b'\n' {
                line_starts.push(line_start as TextPos);
                line_start = pos;
            }
        } else {
            let (_, size) = decode_rune(&text[pos..]);
            pos += size;
            ascii_only = false;
        }
    }
    line_starts.push(line_start as TextPos);
    LspLineMap {
        line_starts,
        ascii_only,
    }
}

/// `slices.BinarySearch`: the insertion index and whether the target was found.
fn binary_search(values: &[TextPos], target: TextPos) -> (usize, bool) {
    let (mut lo, mut hi) = (0usize, values.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if values[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    (lo, lo < values.len() && values[lo] == target)
}

impl LspLineMap {
    // port: tsc/internal/ls/lsconv/linemap.go:LSPLineMap.ComputeIndexOfLineStart
    /// The index of the line containing `target_pos`. Unlike the converter's own
    /// search, this does not decrement past zero.
    pub fn compute_index_of_line_start(&self, target_pos: TextPos) -> i64 {
        let (index, found) = binary_search(&self.line_starts, target_pos);
        let mut line_number = index as i64;
        if !found && line_number > 0 {
            line_number -= 1;
        }
        line_number
    }
}

// ------------------------------------------------------------- position map

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositionMapEntry {
    /// UTF-8 byte offset after this multi-byte character.
    utf8_pos: i64,
    /// Cumulative (utf8 - utf16) difference through this character.
    delta: i64,
}

/// `ast.PositionMap`: the API's byte/UTF-16 conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

// port: tsc/internal/ast/positionmap.go:ComputePositionMap
/// Build the API position map: a sentinel counts as one UTF-16 unit, a malformed
/// byte as one, an astral character as two.
pub fn compute_position_map(text: &[u8]) -> PositionMap {
    let mut entries = Vec::new();
    let mut delta: i64 = 0;
    let mut i = 0usize;
    while i < text.len() {
        let b = text[i];
        if b < RUNE_SELF {
            i += 1;
            continue;
        }
        let (r, size) = decode_js_string_rune(&text[i..]);
        let utf16_size = if r >= 0x1_0000 { 2 } else { 1 };
        delta += size as i64 - utf16_size;
        entries.push(PositionMapEntry {
            utf8_pos: (i + size) as i64,
            delta,
        });
        i += size;
    }
    PositionMap {
        ascii_only: entries.is_empty(),
        entries,
    }
}

impl PositionMap {
    // port: tsc/internal/ast/positionmap.go:PositionMap.IsAsciiOnly
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    // port: tsc/internal/ast/positionmap.go:PositionMap.UTF8ToUTF16
    /// Byte offset to UTF-16 offset. Offsets inside a byte sequence keep
    /// upstream's arithmetic; nothing is rounded or clamped.
    pub fn utf8_to_utf16(&self, utf8_offset: i64) -> i64 {
        if self.ascii_only {
            return utf8_offset;
        }
        let (mut lo, mut hi) = (0usize, self.entries.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.entries[mid].utf8_pos <= utf8_offset {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return utf8_offset;
        }
        utf8_offset - self.entries[lo - 1].delta
    }

    // port: tsc/internal/ast/positionmap.go:PositionMap.UTF16ToUTF8
    /// UTF-16 offset to byte offset, with the same binary search in reverse.
    pub fn utf16_to_utf8(&self, utf16_offset: i64) -> i64 {
        if self.ascii_only {
            return utf16_offset;
        }
        let (mut lo, mut hi) = (0usize, self.entries.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let utf16_pos = self.entries[mid].utf8_pos - self.entries[mid].delta;
            if utf16_pos <= utf16_offset {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return utf16_offset;
        }
        utf16_offset + self.entries[lo - 1].delta
    }
}

// ------------------------------------------------------------ LSP converters

/// `lsproto.PositionEncodingKind`, restricted to the two the converters branch on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
}

/// `lsconv.Converters` for an ordinary (not content-mapped) script.
#[derive(Clone, Copy, Debug)]
pub struct Converters {
    position_encoding: PositionEncoding,
}

// port: tsc/internal/ls/lsconv/converters.go:NewConverters
impl Converters {
    pub fn new(position_encoding: PositionEncoding) -> Self {
        Self { position_encoding }
    }

    // port: tsc/internal/ls/lsconv/converters.go:Converters.lineAndCharacterToPosition
    // port: tsc/internal/ls/lsconv/converters.go:FromLSPPosition
    // port: tsc/internal/ls/lsconv/converters.go:Converters.lspPositionToVirtual
    /// Zero-based LSP line and character to a byte offset. The scan uses the
    /// standard decoder so a malformed byte advances by one, and it stops before
    /// a character that would overshoot the requested count.
    ///
    /// The line and character arrive as protocol `uint32` values and are
    /// narrowed to `TextPos` exactly as upstream narrows them, so the clamped
    /// branch keeps its wrapping arithmetic for extreme inputs.
    ///
    /// # Panics
    /// When the narrowed line is negative, as Go's slice index panics.
    pub fn line_and_character_to_position(
        &self,
        text: &[u8],
        line_map: &LspLineMap,
        line: u32,
        character: u32,
    ) -> TextPos {
        let line = line as TextPos;
        let character = character as TextPos;
        let text_len = text.len() as TextPos;

        // Clamp line to valid range.
        if i64::from(line) >= line_map.line_starts.len() as i64 {
            return text_len;
        }
        let index = usize::try_from(line).expect("line index out of range");
        let start = line_map.line_starts[index];
        let line_end = if index + 1 < line_map.line_starts.len() {
            line_map.line_starts[index + 1]
        } else {
            text_len
        };

        if line_map.ascii_only || self.position_encoding == PositionEncoding::Utf8 {
            return start.max(start.wrapping_add(character).min(line_end));
        }

        let mut utf16_char: TextPos = 0;
        let mut pos = start as usize;
        let end = line_end as usize;
        while pos < end {
            let (r, size) = decode_rune(&text[pos..]);
            let unit_len = utf16_rune_len(r);
            if utf16_char + unit_len > character {
                break;
            }
            utf16_char += unit_len;
            pos += size;
        }
        pos as TextPos
    }

    // port: tsc/internal/ls/lsconv/converters.go:Converters.positionToLineAndCharacter
    // port: tsc/internal/ls/lsconv/converters.go:Converters.ToLSPPosition
    /// A byte offset to a zero-based LSP line and character, clamping the
    /// position into the text first.
    pub fn position_to_line_and_character(
        &self,
        text: &[u8],
        line_map: &LspLineMap,
        position: TextPos,
    ) -> (u32, u32) {
        let position = position.max(0).min(text.len() as TextPos);
        let (index, is_line_start) = binary_search(&line_map.line_starts, position);
        let mut line = index as i64;
        if !is_line_start {
            line -= 1;
        }
        line = line.max(0).min(line_map.line_starts.len() as i64 - 1);
        let start = i64::from(line_map.line_starts[line as usize]);

        let character = if line_map.ascii_only || self.position_encoding == PositionEncoding::Utf8 {
            i64::from(position) - start
        } else {
            // Rescan the byte prefix as UTF-16, with Go's `range` decoding.
            let mut character = 0i64;
            for (_, r, _) in Runes::new(&text[start as usize..position as usize]) {
                character += i64::from(utf16_rune_len(r));
            }
            character
        };
        (line as u32, character as u32)
    }
}

// ------------------------------------------------------- scanner utilities

// port: tsc/internal/core/core.go:UTF16Len
/// UTF-16 code units in a byte string, with an ASCII fast path.
pub fn utf16_len(s: &[u8]) -> i64 {
    for i in 0..s.len() {
        if s[i] >= RUNE_SELF {
            let mut n = i as i64;
            for (_, r, _) in Runes::new(&s[i..]) {
                n += i64::from(utf16_rune_len(r));
            }
            return n;
        }
    }
    s.len() as i64
}

// port: tsc/internal/core/core.go:PositionToLineAndByteOffset
/// Zero-based line and raw byte offset from the line start.
pub fn position_to_line_and_byte_offset(position: i64, line_starts: &[TextPos]) -> (i64, i64) {
    // sort.Search finds the first line start strictly greater than position.
    let mut lo = 0usize;
    let mut hi = line_starts.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if i64::from(line_starts[mid]) > position {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let line = (lo as i64 - 1).max(0);
    (line, position - i64::from(line_starts[line as usize]))
}

// port: tsc/internal/scanner/scanner.go:ComputeLineOfPosition
/// The scanner's own line search, which returns `low - 1` on a miss.
pub fn compute_line_of_position(line_starts: &[TextPos], pos: i64) -> i64 {
    let mut low: i64 = 0;
    let mut high: i64 = line_starts.len() as i64 - 1;
    while low <= high {
        let middle = low + ((high - low) >> 1);
        let value = i64::from(line_starts[middle as usize]);
        match value.cmp(&pos) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle - 1,
            std::cmp::Ordering::Equal => return middle,
        }
    }
    low - 1
}

// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndByteOffset
/// Line plus raw byte offset, panicking on a bad line number.
///
/// # Panics
/// When the line is outside the line map, exactly as upstream panics.
pub fn compute_position_of_line_and_byte_offset(
    line_starts: &[TextPos],
    line: i64,
    byte_offset: i64,
) -> i64 {
    assert!(
        line >= 0 && line < line_starts.len() as i64,
        "Bad line number. Line: {line}, lineStarts.length: {}.",
        line_starts.len()
    );
    i64::from(line_starts[line as usize]) + byte_offset
}

// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndUTF16Character
/// Line plus UTF-16 character offset to a byte position. A whole character is
/// consumed before the accumulated count is tested, so an interior surrogate
/// offset rounds forward, unlike the LSP converter.
///
/// # Panics
/// With `allow_edits` false, on a bad line number or a character offset past the
/// end of the line, exactly as upstream panics.
pub fn compute_position_of_line_and_utf16_character(
    line_starts: &[TextPos],
    line: i64,
    character: i64,
    text: &[u8],
    allow_edits: bool,
) -> i64 {
    let mut line = line;
    if line < 0 || line >= line_starts.len() as i64 {
        assert!(
            allow_edits,
            "Bad line number. Line: {line}, lineStarts.length: {}.",
            line_starts.len()
        );
        line = if line < 0 {
            0
        } else {
            line_starts.len() as i64 - 1
        };
    }

    let line_start = i64::from(line_starts[line as usize]);

    if character > 0 {
        let line_end = if line + 1 < line_starts.len() as i64 {
            i64::from(line_starts[(line + 1) as usize])
        } else {
            text.len() as i64
        };
        let mut utf16_count: i64 = 0;
        let mut pos = line_start;
        while pos < line_end {
            if utf16_count >= character {
                break;
            }
            let (r, size) = decode_rune(&text[pos as usize..]);
            utf16_count += i64::from(utf16_rune_len(r));
            pos += size as i64;
        }
        if !allow_edits {
            assert!(
                !(pos == line_end && utf16_count < character),
                "Bad UTF-16 character offset. Line: {line}, character: {character}."
            );
            debug_assert!(pos <= text.len() as i64);
            return pos;
        }
        if pos > text.len() as i64 {
            return text.len() as i64;
        }
        return pos;
    }

    let res = line_start;
    if allow_edits {
        if res > text.len() as i64 {
            return text.len() as i64;
        }
        return res;
    }
    debug_assert!(res <= text.len() as i64);
    res
}

// port: tsc/internal/scanner/scanner.go:GetECMALineAndUTF16CharacterOfPosition
/// Zero-based line and UTF-16 character offset for a byte position, counting the
/// byte prefix with [`utf16_len`].
///
/// # Panics
/// When the position is outside the line, as Go's slice expression panics.
pub fn ecma_line_and_utf16_character_of_position(
    text: &[u8],
    line_starts: &[TextPos],
    pos: i64,
) -> (i64, i64) {
    let line = compute_line_of_position(line_starts, pos);
    let start = i64::from(line_starts[usize::try_from(line).expect("line index out of range")]);
    assert!(
        start <= pos && pos <= text.len() as i64,
        "slice bounds out of range [{start}:{pos}] with length {}",
        text.len()
    );
    (line, utf16_len(&text[start as usize..pos as usize]))
}

// port: tsc/internal/scanner/scanner.go:GetECMALineAndByteOffsetOfPosition
/// Zero-based line and raw byte offset for a byte position.
///
/// # Panics
/// When the computed line index is outside the line map.
pub fn ecma_line_and_byte_offset_of_position(line_starts: &[TextPos], pos: i64) -> (i64, i64) {
    let line = compute_line_of_position(line_starts, pos);
    let start = i64::from(line_starts[usize::try_from(line).expect("line index out of range")]);
    (line, pos - start)
}
