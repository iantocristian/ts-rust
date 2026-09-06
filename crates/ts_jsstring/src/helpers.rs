//! Leaf helpers from `stringutil/util.go` whose decoding and replacement
//! behavior E4 pins down.
//!
//! These deliberately do *not* share one decoding policy: `TruncateByRunes` and
//! `LowerFirstChar` use Go's standard decoder, which counts each malformed byte
//! separately and can split a WTF-8 sentinel, while the case mappers use the
//! sentinel-aware decoder. The fragments that produces are behavior, not damage
//! to repair (docs/design/text.md, section 2.2).

use crate::rune::{decode_rune, write_rune, Rune, Runes};
use crate::unicode_case_generated::SPECIAL_CASING_MAPPINGS;

// port: tsc/internal/stringutil/util.go:TruncateByRunes
/// Truncate to at most `max_length` runes as Go's `range` counts them, so a
/// malformed byte counts as one rune and a sentinel can be split.
pub fn truncate_by_runes(s: &[u8], max_length: i64) -> Vec<u8> {
    if (s.len() as i64) < max_length {
        return s.to_vec();
    }
    if max_length <= 0 {
        return Vec::new();
    }
    let mut rune_count: i64 = 0;
    for (offset, _, _) in Runes::new(s) {
        rune_count += 1;
        if rune_count > max_length {
            return s[..offset].to_vec();
        }
    }
    s.to_vec()
}

// port: tsc/internal/stringutil/util.go:LowerFirstChar
/// Lowercase the first decoded rune with Go's `unicode.ToLower` and copy the
/// rest of the bytes unchanged.
pub fn lower_first_char(s: &[u8]) -> Vec<u8> {
    let (ch, size) = decode_rune(s);
    if size > 0 {
        let mut out = Vec::with_capacity(s.len());
        write_rune(&mut out, unicode_to_lower(ch));
        out.extend_from_slice(&s[size..]);
        return out;
    }
    s.to_vec()
}

/// The Unicode 15.1.0 simple lowercase mapping, derived from the pinned table.
///
/// Upstream's `LowerFirstChar` calls Go's `unicode.ToLower`, whose tables follow
/// the Go toolchain's Unicode version rather than the pin. The port stays on the
/// pinned version, like the rest of this crate (docs/design/text.md, section
/// 2.5), so the two agree on every code point Unicode 15.1.0 defines and differ
/// on later additions. The E4 helper comparison states that domain explicitly
/// rather than letting a toolchain upgrade silently change a port.
///
/// U+0130 is the one entry whose full lowercase mapping is more than one rune
/// (`scripts/gen-unicode-case.py` fails if the pin ever adds another), and its
/// Unicode 15.1.0 simple lowercase mapping is U+0069.
fn unicode_to_lower(ch: Rune) -> Rune {
    if ch == 0x0130 {
        return 0x0069;
    }
    let Ok(index) = SPECIAL_CASING_MAPPINGS.binary_search_by_key(&ch, |(code, _)| *code) else {
        return ch;
    };
    let lower = SPECIAL_CASING_MAPPINGS[index].1.lower;
    let (mapped, size) = decode_rune(lower);
    if size == lower.len() {
        mapped
    } else {
        ch
    }
}
