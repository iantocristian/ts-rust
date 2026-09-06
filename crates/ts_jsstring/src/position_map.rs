//! port: tsc/internal/ast/positionmap.go
//!
//! The API encoder's bidirectional UTF-8 byte to UTF-16 code unit map. A WTF-8
//! sentinel counts as one unit and a malformed byte as one unit, because the map
//! is built with the sentinel-aware decoder. Neither direction rounds to a
//! character boundary or clamps (docs/design/text.md, 2.3).

use crate::wtf8::{decode_js_string_rune, RUNE_SELF};

struct Entry {
    /// UTF-8 byte offset after this multi-byte character.
    utf8_pos: usize,
    /// Cumulative (utf8 - utf16) difference after this character.
    delta: usize,
}

/// The `PositionMap` type of tsc/internal/ast/positionmap.go.
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<Entry>,
}

/// port: tsc/internal/ast/positionmap.go:ComputePositionMap
pub fn compute_position_map(text: &[u8]) -> PositionMap {
    let mut entries = Vec::new();
    let mut delta = 0;
    let mut i = 0;
    while i < text.len() {
        let b = text[i];
        if b < RUNE_SELF {
            i += 1;
            continue;
        }
        let (r, size) = decode_js_string_rune(&text[i..]);
        let utf16_size = if r >= 0x1_0000 { 2 } else { 1 };
        delta += size - utf16_size;
        entries.push(Entry {
            utf8_pos: i + size,
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
    /// port: tsc/internal/ast/positionmap.go:PositionMap.IsAsciiOnly
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    /// port: tsc/internal/ast/positionmap.go:PositionMap.UTF8ToUTF16
    pub fn utf8_to_utf16(&self, utf8_offset: usize) -> usize {
        if self.ascii_only {
            return utf8_offset;
        }
        let lo = self.entries.partition_point(|e| e.utf8_pos <= utf8_offset);
        if lo == 0 {
            return utf8_offset;
        }
        utf8_offset - self.entries[lo - 1].delta
    }

    /// port: tsc/internal/ast/positionmap.go:PositionMap.UTF16ToUTF8
    pub fn utf16_to_utf8(&self, utf16_offset: usize) -> usize {
        if self.ascii_only {
            return utf16_offset;
        }
        let lo = self
            .entries
            .partition_point(|e| e.utf8_pos - e.delta <= utf16_offset);
        if lo == 0 {
            return utf16_offset;
        }
        utf16_offset + self.entries[lo - 1].delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astral_interior_offset_maps_to_one() {
        let pm = compute_position_map("😀".as_bytes());
        assert!(!pm.is_ascii_only());
        assert_eq!(pm.utf16_to_utf8(1), 1);
        assert_eq!(pm.utf16_to_utf8(2), 4);
        assert_eq!(pm.utf8_to_utf16(4), 2);
    }

    #[test]
    fn sentinel_is_one_unit() {
        let pm = compute_position_map(b"a\xED\xA0\x80b");
        assert_eq!(pm.utf8_to_utf16(4), 2);
        assert_eq!(pm.utf16_to_utf8(2), 4);
    }
}
