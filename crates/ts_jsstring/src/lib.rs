//! Byte-preserving JavaScript strings and the position conversions used at
//! TypeScript's API boundaries.

mod case_data;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{BitOr, BitOrAssign, Range};
use std::sync::Arc;

const REPLACEMENT: u32 = 0xFFFD;
const SURROGATE_START: u32 = 0xD800;
const SURROGATE_LOW_START: u32 = 0xDC00;
const SURROGATE_END: u32 = 0xDFFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Validity {
    Utf8,
    Wtf8,
    Raw,
}

#[derive(Clone)]
pub struct SourceText {
    bytes: Arc<[u8]>,
    valid_utf8: bool,
}

impl SourceText {
    /// Applies Corsa's BOM rules. An odd trailing byte in a UTF-16 file is
    /// ignored, matching `encoding/binary.Read` into `len(bytes) / 2` words.
    // port: tsc/internal/vfs/internal/internal.go:decodeBytes
    // port: tsc/internal/vfs/internal/internal.go:decodeUtf16
    pub fn from_bytes(input: &[u8]) -> Self {
        let bytes = if input.starts_with(&[0xFF, 0xFE]) {
            decode_utf16(&input[2..], Endian::Little)
        } else if input.starts_with(&[0xFE, 0xFF]) {
            decode_utf16(&input[2..], Endian::Big)
        } else if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
            input[3..].to_vec()
        } else {
            input.to_vec()
        };
        let valid_utf8 = std::str::from_utf8(&bytes).is_ok();
        Self {
            bytes: bytes.into(),
            valid_utf8,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.valid_utf8 {
            // The validity flag is computed from these immutable bytes.
            std::str::from_utf8(&self.bytes).ok()
        } else {
            None
        }
    }

    pub fn is_utf8(&self) -> bool {
        self.valid_utf8
    }

    pub fn slice(&self, range: Range<usize>) -> Result<SourceSlice, SliceError> {
        if range.start > range.end || range.end > self.bytes.len() {
            return Err(SliceError {
                start: range.start,
                end: range.end,
                len: self.bytes.len(),
            });
        }
        Ok(SourceSlice {
            source: Arc::clone(&self.bytes),
            range,
        })
    }
}

#[derive(Clone)]
pub struct SourceSlice {
    source: Arc<[u8]>,
    range: Range<usize>,
}

impl SourceSlice {
    pub fn as_bytes(&self) -> &[u8] {
        &self.source[self.range.clone()]
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }

    pub fn to_js_string(&self) -> JsString {
        JsString::from_bytes(self.as_bytes())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Endian {
    Little,
    Big,
}

fn decode_utf16(input: &[u8], endian: Endian) -> Vec<u8> {
    let mut words = Vec::with_capacity(input.len() / 2);
    for pair in input.chunks_exact(2) {
        words.push(match endian {
            Endian::Little => u16::from_le_bytes([pair[0], pair[1]]),
            Endian::Big => u16::from_be_bytes([pair[0], pair[1]]),
        });
    }

    let mut output = Vec::with_capacity(words.len() * 2);
    let mut index = 0;
    while index < words.len() {
        let word = u32::from(words[index]);
        if (SURROGATE_START..SURROGATE_LOW_START).contains(&word) {
            if let Some(&next) = words.get(index + 1) {
                let next = u32::from(next);
                if (SURROGATE_LOW_START..=SURROGATE_END).contains(&next) {
                    push_scalar(
                        &mut output,
                        0x1_0000 + ((word - SURROGATE_START) << 10) + (next - SURROGATE_LOW_START),
                    );
                    index += 2;
                    continue;
                }
            }
            push_scalar(&mut output, REPLACEMENT);
        } else if (SURROGATE_LOW_START..=SURROGATE_END).contains(&word) {
            push_scalar(&mut output, REPLACEMENT);
        } else {
            push_scalar(&mut output, word);
        }
        index += 1;
    }
    output
}

#[derive(Clone)]
pub struct JsString {
    bytes: Arc<[u8]>,
    validity: Validity,
}

impl JsString {
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let bytes: Arc<[u8]> = Arc::from(bytes.as_ref());
        let validity = classify(&bytes);
        Self { bytes, validity }
    }

    pub fn from_arc(bytes: Arc<[u8]>) -> Self {
        let validity = classify(&bytes);
        Self { bytes, validity }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.validity == Validity::Utf8 {
            std::str::from_utf8(&self.bytes).ok()
        } else {
            None
        }
    }

    pub fn validity(&self) -> Validity {
        self.validity
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn slice(&self, range: Range<usize>) -> Result<Self, SliceError> {
        if range.start > range.end || range.end > self.bytes.len() {
            return Err(SliceError {
                start: range.start,
                end: range.end,
                len: self.bytes.len(),
            });
        }
        Ok(Self::from_bytes(&self.bytes[range]))
    }

    pub fn code_points(&self) -> CodePoints<'_> {
        CodePoints {
            remaining: &self.bytes,
        }
    }
}

impl From<&str> for JsString {
    fn from(value: &str) -> Self {
        Self {
            bytes: Arc::from(value.as_bytes()),
            validity: Validity::Utf8,
        }
    }
}

impl AsRef<[u8]> for JsString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl fmt::Debug for JsString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsString")
            .field("bytes", &self.bytes)
            .field("validity", &self.validity)
            .finish()
    }
}

impl PartialEq for JsString {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for JsString {}

impl PartialOrd for JsString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl Hash for JsString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedRune {
    pub code_point: u32,
    pub size: usize,
    pub surrogate: bool,
    pub malformed: bool,
}

pub struct CodePoints<'a> {
    remaining: &'a [u8],
}

impl Iterator for CodePoints<'_> {
    type Item = DecodedRune;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let decoded = decode_js_string_rune(self.remaining);
        self.remaining = &self.remaining[decoded.size..];
        Some(decoded)
    }
}

fn classify(bytes: &[u8]) -> Validity {
    if std::str::from_utf8(bytes).is_ok() {
        return Validity::Utf8;
    }
    let mut index = 0;
    let mut saw_surrogate = false;
    while index < bytes.len() {
        let decoded = decode_js_string_rune(&bytes[index..]);
        if decoded.malformed {
            return Validity::Raw;
        }
        saw_surrogate |= decoded.surrogate;
        index += decoded.size;
    }
    if saw_surrogate {
        Validity::Wtf8
    } else {
        Validity::Raw
    }
}

// port: tsc/internal/stringutil/util.go:EncodeJSStringRune
pub fn encode_js_string_rune(code_point: u32) -> JsString {
    let mut bytes = Vec::with_capacity(4);
    if is_surrogate(code_point) {
        bytes.extend_from_slice(&[
            0xED,
            0x80 | u8::try_from((code_point >> 6) & 0x3F).expect("six bits fit in u8"),
            0x80 | u8::try_from(code_point & 0x3F).expect("six bits fit in u8"),
        ]);
    } else if char::from_u32(code_point).is_some() {
        push_scalar(&mut bytes, code_point);
    } else {
        push_scalar(&mut bytes, REPLACEMENT);
    }
    JsString::from_bytes(bytes)
}

// port: tsc/internal/stringutil/util.go:DecodeJSStringRune
pub fn decode_js_string_rune(input: &[u8]) -> DecodedRune {
    if input.len() >= 3
        && input[0] == 0xED
        && (0xA0..=0xBF).contains(&input[1])
        && (0x80..=0xBF).contains(&input[2])
    {
        return DecodedRune {
            code_point: 0xD000 | (u32::from(input[1] & 0x3F) << 6) | u32::from(input[2] & 0x3F),
            size: 3,
            surrogate: true,
            malformed: false,
        };
    }
    decode_standard_utf8(input)
}

fn decode_standard_utf8(input: &[u8]) -> DecodedRune {
    if input.is_empty() {
        return DecodedRune {
            code_point: REPLACEMENT,
            size: 0,
            surrogate: false,
            malformed: false,
        };
    }

    let b0 = input[0];
    if b0 < 0x80 {
        return decoded(u32::from(b0), 1);
    }
    let continuation = |index: usize| input.get(index).is_some_and(|b| (0x80..=0xBF).contains(b));
    if (0xC2..=0xDF).contains(&b0) && continuation(1) {
        return decoded((u32::from(b0 & 0x1F) << 6) | u32::from(input[1] & 0x3F), 2);
    }
    if input.len() >= 3 && continuation(1) && continuation(2) {
        let valid_second = match b0 {
            0xE0 => (0xA0..=0xBF).contains(&input[1]),
            0xE1..=0xEC | 0xEE..=0xEF => true,
            0xED => (0x80..=0x9F).contains(&input[1]),
            _ => false,
        };
        if valid_second {
            return decoded(
                (u32::from(b0 & 0x0F) << 12)
                    | (u32::from(input[1] & 0x3F) << 6)
                    | u32::from(input[2] & 0x3F),
                3,
            );
        }
    }
    if input.len() >= 4 && continuation(1) && continuation(2) && continuation(3) {
        let valid_second = match b0 {
            0xF0 => (0x90..=0xBF).contains(&input[1]),
            0xF1..=0xF3 => true,
            0xF4 => (0x80..=0x8F).contains(&input[1]),
            _ => false,
        };
        if valid_second {
            return decoded(
                (u32::from(b0 & 0x07) << 18)
                    | (u32::from(input[1] & 0x3F) << 12)
                    | (u32::from(input[2] & 0x3F) << 6)
                    | u32::from(input[3] & 0x3F),
                4,
            );
        }
    }
    DecodedRune {
        code_point: REPLACEMENT,
        size: 1,
        surrogate: false,
        malformed: true,
    }
}

fn decoded(code_point: u32, size: usize) -> DecodedRune {
    DecodedRune {
        code_point,
        size,
        surrogate: false,
        malformed: false,
    }
}

fn push_scalar(output: &mut Vec<u8>, code_point: u32) {
    let scalar = char::from_u32(code_point).unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut encoded = [0; 4];
    output.extend_from_slice(scalar.encode_utf8(&mut encoded).as_bytes());
}

fn is_surrogate(code_point: u32) -> bool {
    (SURROGATE_START..=SURROGATE_END).contains(&code_point)
}

fn is_high_surrogate(code_point: u32) -> bool {
    (SURROGATE_START..SURROGATE_LOW_START).contains(&code_point)
}

fn is_low_surrogate(code_point: u32) -> bool {
    (SURROGATE_LOW_START..=SURROGATE_END).contains(&code_point)
}

// port: tsc/internal/stringutil/util.go:CombineSurrogatePairs
pub fn combine_surrogate_pairs(input: &JsString) -> JsString {
    if !input.as_bytes().contains(&0xED) {
        return input.clone();
    }
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let current = decode_js_string_rune(&bytes[index..]);
        if is_high_surrogate(current.code_point) {
            let next = decode_js_string_rune(&bytes[index + current.size..]);
            if is_low_surrogate(next.code_point) {
                push_scalar(
                    &mut output,
                    0x1_0000
                        + ((current.code_point - SURROGATE_START) << 10)
                        + (next.code_point - SURROGATE_LOW_START),
                );
                index += current.size + next.size;
                continue;
            }
        }
        output.extend_from_slice(&bytes[index..index + current.size]);
        index += current.size;
    }
    JsString::from_bytes(output)
}

// port: tsc/internal/stringutil/js_case.go:ToLowerJS
// port: tsc/internal/stringutil/js_case.go:toLowerASCII
// port: tsc/internal/stringutil/js_case.go:isFinalSigmaContext
pub fn to_lower_js(input: &JsString) -> JsString {
    if input.as_bytes().iter().all(u8::is_ascii) {
        let mut bytes = input.as_bytes().to_vec();
        bytes.make_ascii_lowercase();
        return JsString::from_bytes(bytes);
    }

    let mut output = Vec::with_capacity(input.len());
    let mut cased_before = false;
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let decoded = decode_js_string_rune(&bytes[index..]);
        index += decoded.size;
        if decoded.surrogate {
            output.extend_from_slice(&bytes[index - decoded.size..index]);
        } else if let Some(mapping) = case_data::mapping(decoded.code_point) {
            if mapping.final_sigma
                && cased_before
                && !has_cased_after(bytes, index)
                && mapping.conditional_lower.is_some()
            {
                output.extend_from_slice(mapping.conditional_lower.unwrap_or_default().as_bytes());
            } else {
                output.extend_from_slice(mapping.lower.as_bytes());
            }
        } else {
            push_scalar(&mut output, decoded.code_point);
        }
        if !case_data::is_case_ignorable(decoded.code_point) {
            cased_before = case_data::is_cased(decoded.code_point);
        }
    }
    JsString::from_bytes(output)
}

// port: tsc/internal/stringutil/js_case.go:ToUpperJS
// port: tsc/internal/stringutil/js_case.go:toUpperASCII
pub fn to_upper_js(input: &JsString) -> JsString {
    if input.as_bytes().iter().all(u8::is_ascii) {
        let mut bytes = input.as_bytes().to_vec();
        bytes.make_ascii_uppercase();
        return JsString::from_bytes(bytes);
    }

    let mut output = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let decoded = decode_js_string_rune(&bytes[index..]);
        if decoded.surrogate {
            output.extend_from_slice(&bytes[index..index + decoded.size]);
        } else if let Some(mapping) = case_data::mapping(decoded.code_point) {
            output.extend_from_slice(mapping.upper.as_bytes());
        } else {
            push_scalar(&mut output, decoded.code_point);
        }
        index += decoded.size;
    }
    JsString::from_bytes(output)
}

// port: tsc/internal/stringutil/js_case.go:hasSigmaCasedAfter
// port: tsc/internal/stringutil/js_case.go:isSigmaCased
// port: tsc/internal/stringutil/js_case.go:isUnicodeCaseIgnorable
fn has_cased_after(bytes: &[u8], mut index: usize) -> bool {
    while index < bytes.len() {
        let decoded = decode_js_string_rune(&bytes[index..]);
        index += decoded.size;
        if case_data::is_case_ignorable(decoded.code_point) {
            continue;
        }
        return case_data::is_cased(decoded.code_point);
    }
    false
}

// port: tsc/internal/stringutil/util.go:LowerFirstChar
pub fn lower_first_char(input: &JsString) -> JsString {
    let decoded = decode_standard_utf8(input.as_bytes());
    if decoded.size == 0 {
        return input.clone();
    }
    let first = char::from_u32(decoded.code_point)
        .unwrap_or(char::REPLACEMENT_CHARACTER)
        .to_lowercase()
        .next()
        .unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut output = Vec::with_capacity(input.len() + 3);
    let mut encoded = [0; 4];
    output.extend_from_slice(first.encode_utf8(&mut encoded).as_bytes());
    output.extend_from_slice(&input.as_bytes()[decoded.size..]);
    JsString::from_bytes(output)
}

// port: tsc/internal/stringutil/util.go:TruncateByRunes
pub fn truncate_by_runes(input: &JsString, max_length: isize) -> JsString {
    if isize::try_from(input.len()).is_ok_and(|len| len < max_length) {
        return input.clone();
    }
    if max_length <= 0 {
        return JsString::from("");
    }
    let mut index = 0;
    let mut count = 0;
    while index < input.len() {
        count += 1;
        if count > max_length {
            return JsString::from_bytes(&input.as_bytes()[..index]);
        }
        index += decode_standard_utf8(&input.as_bytes()[index..]).size;
    }
    input.clone()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuoteChar {
    Single,
    Double,
    Backtick,
}

impl QuoteChar {
    fn byte(self) -> u8 {
        match self {
            Self::Single => b'\'',
            Self::Double => b'"',
            Self::Backtick => b'`',
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EscapeFlags(u8);

impl EscapeFlags {
    pub const NONE: Self = Self(0);
    pub const NEVER_ASCII_ESCAPE: Self = Self(1 << 0);
    pub const JSX_ATTRIBUTE: Self = Self(1 << 1);

    fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for EscapeFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for EscapeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// port: tsc/internal/printer/utilities.go:escapeStringWorker
// port: tsc/internal/printer/utilities.go:encodeJsxCharacterEntity
pub fn escape_string_worker(input: &JsString, quote: QuoteChar, flags: EscapeFlags) -> JsString {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() + 2);
    let mut index = 0;
    while index < bytes.len() {
        let decoded = decode_js_string_rune(&bytes[index..]);
        let code_point = decoded.code_point;
        let mut size = decoded.size;
        let mut escape = decoded.surrogate || decoded.malformed;
        escape |= match code_point {
            0x5C => !flags.contains(EscapeFlags::JSX_ATTRIBUTE),
            0x24 => quote == QuoteChar::Backtick && bytes.get(index + 1) == Some(&b'{'),
            0x2028 | 0x2029 | 0x85 | 0x0D => true,
            0x0A => quote != QuoteChar::Backtick,
            other if other == u32::from(quote.byte()) => true,
            other => {
                other <= 0x1F || (!flags.contains(EscapeFlags::NEVER_ASCII_ESCAPE) && other > 0x7F)
            }
        };

        if !escape {
            output.extend_from_slice(&bytes[index..index + size]);
            index += size;
            continue;
        }

        if flags.contains(EscapeFlags::JSX_ATTRIBUTE) {
            match code_point {
                0 => output.extend_from_slice(b"&#0;"),
                0x22 => output.extend_from_slice(b"&quot;"),
                0x27 => output.extend_from_slice(b"&apos;"),
                other => output.extend_from_slice(format!("&#x{other:X};").as_bytes()),
            }
        } else if code_point == 0x0D
            && quote == QuoteChar::Backtick
            && bytes.get(index + 1) == Some(&b'\n')
        {
            size += 1;
            output.extend_from_slice(br"\r\n");
        } else if code_point > 0xFFFF {
            let adjusted = code_point - 0x1_0000;
            push_utf16_escape(&mut output, 0xD800 + (adjusted >> 10));
            push_utf16_escape(&mut output, 0xDC00 + (adjusted & 0x3FF));
        } else if decoded.surrogate || decoded.malformed {
            push_utf16_escape(&mut output, code_point);
        } else if code_point == 0 {
            if bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
                output.extend_from_slice(br"\x00");
            } else {
                output.extend_from_slice(br"\0");
            }
        } else if let Some(escaped) = canonical_escape(code_point, quote, bytes.get(index + 1)) {
            output.extend_from_slice(escaped);
        } else {
            push_utf16_escape(&mut output, code_point);
        }
        index += size;
    }
    JsString::from_bytes(output)
}

fn canonical_escape(code_point: u32, quote: QuoteChar, next: Option<&u8>) -> Option<&'static [u8]> {
    match code_point {
        0x09 => Some(br"\t"),
        0x0B => Some(br"\v"),
        0x0C => Some(br"\f"),
        0x08 => Some(br"\b"),
        0x0D => Some(br"\r"),
        0x0A => Some(br"\n"),
        0x5C => Some(br"\\"),
        0x22 => Some(br#"\""#),
        0x27 => Some(br"\'"),
        0x60 => Some(br"\`"),
        0x24 if quote == QuoteChar::Backtick && next == Some(&b'{') => Some(br"\$"),
        0x2028 => Some(br"\u2028"),
        0x2029 => Some(br"\u2029"),
        0x85 => Some(br"\u0085"),
        _ => None,
    }
}

// port: tsc/internal/printer/utilities.go:encodeUtf16EscapeSequence
fn push_utf16_escape(output: &mut Vec<u8>, code_unit: u32) {
    output.extend_from_slice(format!(r"\u{code_unit:04X}").as_bytes());
}

// port: tsc/internal/printer/utilities.go:EscapeString
pub fn escape_string(input: &JsString, quote: QuoteChar) -> JsString {
    escape_string_worker(input, quote, EscapeFlags::NEVER_ASCII_ESCAPE)
}

// port: tsc/internal/printer/utilities.go:escapeNonAsciiString
pub fn escape_non_ascii_string(input: &JsString, quote: QuoteChar) -> JsString {
    escape_string_worker(input, quote, EscapeFlags::NONE)
}

// port: tsc/internal/printer/utilities.go:escapeJsxAttributeString
pub fn escape_jsx_attribute_string(input: &JsString, quote: QuoteChar) -> JsString {
    escape_string_worker(
        input,
        quote,
        EscapeFlags::JSX_ATTRIBUTE | EscapeFlags::NEVER_ASCII_ESCAPE,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceError {
    pub start: usize,
    pub end: usize,
    pub len: usize,
}

impl fmt::Display for SliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "byte slice {}..{} is outside a buffer of length {}",
            self.start, self.end, self.len
        )
    }
}

impl std::error::Error for SliceError {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PositionMapEntry {
    utf8_pos: i64,
    delta: i64,
}

impl PositionMap {
    // port: tsc/internal/ast/positionmap.go:ComputePositionMap
    pub fn new(text: &[u8]) -> Self {
        let mut entries = Vec::new();
        let mut delta = 0_i64;
        let mut index = 0;
        while index < text.len() {
            if text[index] < 0x80 {
                index += 1;
                continue;
            }
            let decoded = decode_js_string_rune(&text[index..]);
            let utf16_size = if decoded.code_point >= 0x1_0000 { 2 } else { 1 };
            delta += i64::try_from(decoded.size).expect("offset fits i64") - utf16_size;
            index += decoded.size;
            entries.push(PositionMapEntry {
                utf8_pos: i64::try_from(index).expect("offset fits i64"),
                delta,
            });
        }
        Self {
            ascii_only: entries.is_empty(),
            entries,
        }
    }

    pub fn is_ascii_only(&self) -> bool {
        // port: tsc/internal/ast/positionmap.go:PositionMap.IsAsciiOnly
        self.ascii_only
    }

    // port: tsc/internal/ast/positionmap.go:PositionMap.UTF8ToUTF16
    pub fn utf8_to_utf16(&self, utf8_offset: i64) -> i64 {
        if self.ascii_only {
            return utf8_offset;
        }
        let index = self
            .entries
            .partition_point(|entry| entry.utf8_pos <= utf8_offset);
        index
            .checked_sub(1)
            .map_or(utf8_offset, |i| utf8_offset - self.entries[i].delta)
    }

    // port: tsc/internal/ast/positionmap.go:PositionMap.UTF16ToUTF8
    pub fn utf16_to_utf8(&self, utf16_offset: i64) -> i64 {
        if self.ascii_only {
            return utf16_offset;
        }
        let index = self
            .entries
            .partition_point(|entry| entry.utf8_pos - entry.delta <= utf16_offset);
        index
            .checked_sub(1)
            .map_or(utf16_offset, |i| utf16_offset + self.entries[i].delta)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspLineMap {
    line_starts: Vec<usize>,
    ascii_only: bool,
}

impl LspLineMap {
    // port: tsc/internal/ls/lsconv/linemap.go:ComputeLSPLineStarts
    pub fn new(text: &[u8]) -> Self {
        let mut line_starts = Vec::new();
        let mut ascii_only = true;
        let mut position = 0;
        let mut line_start = 0;
        while position < text.len() {
            let byte = text[position];
            if byte < 0x80 {
                position += 1;
                if byte == b'\r' {
                    if text.get(position) == Some(&b'\n') {
                        position += 1;
                    }
                    line_starts.push(line_start);
                    line_start = position;
                } else if byte == b'\n' {
                    line_starts.push(line_start);
                    line_start = position;
                }
            } else {
                position += decode_standard_utf8(&text[position..]).size;
                ascii_only = false;
            }
        }
        line_starts.push(line_start);
        Self {
            line_starts,
            ascii_only,
        }
    }

    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    // port: tsc/internal/ls/lsconv/linemap.go:LSPLineMap.ComputeIndexOfLineStart
    pub fn index_of_line_start(&self, target: usize) -> usize {
        match self.line_starts.binary_search(&target) {
            Ok(index) => index,
            Err(0) => 0,
            Err(index) => index - 1,
        }
    }
}

// port: tsc/internal/core/core.go:ComputeECMALineStarts
// port: tsc/internal/core/core.go:ComputeECMALineStartsSeq
pub fn compute_ecma_line_starts(text: &[u8]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut position = 0;
    let mut line_start = 0;
    while position < text.len() {
        let byte = text[position];
        if byte < 0x80 {
            position += 1;
            if byte == b'\r' {
                if text.get(position) == Some(&b'\n') {
                    position += 1;
                }
                starts.push(line_start);
                line_start = position;
            } else if byte == b'\n' {
                starts.push(line_start);
                line_start = position;
            }
        } else {
            let decoded = decode_standard_utf8(&text[position..]);
            position += decoded.size;
            if matches!(decoded.code_point, 0x2028 | 0x2029) {
                starts.push(line_start);
                line_start = position;
            }
        }
    }
    starts.push(line_start);
    starts
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineAndCharacter {
    pub line: u32,
    pub character: u32,
}

// port: tsc/internal/ls/lsconv/converters.go:Converters.lineAndCharacterToPosition
pub fn lsp_line_and_character_to_position(
    text: &[u8],
    line_map: &LspLineMap,
    position: LineAndCharacter,
    encoding: PositionEncoding,
) -> usize {
    let line = usize::try_from(position.line).expect("u32 fits usize");
    let character = usize::try_from(position.character).expect("u32 fits usize");
    if line >= line_map.line_starts.len() {
        return text.len();
    }
    let start = line_map.line_starts[line];
    let line_end = line_map
        .line_starts
        .get(line + 1)
        .copied()
        .unwrap_or(text.len());
    if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        return start.saturating_add(character).min(line_end).max(start);
    }

    let mut utf16_character = 0;
    let mut byte_position = start;
    while byte_position < line_end {
        let decoded = decode_standard_utf8(&text[byte_position..]);
        let units = usize::from(decoded.code_point >= 0x1_0000) + 1;
        if utf16_character + units > character {
            break;
        }
        utf16_character += units;
        byte_position += decoded.size;
    }
    byte_position
}

// port: tsc/internal/ls/lsconv/converters.go:Converters.positionToLineAndCharacter
pub fn lsp_position_to_line_and_character(
    text: &[u8],
    line_map: &LspLineMap,
    position: i64,
    encoding: PositionEncoding,
) -> LineAndCharacter {
    let position = position.clamp(0, i64::try_from(text.len()).expect("length fits i64"));
    let position = usize::try_from(position).expect("position was clamped nonnegative");
    let line = line_map.index_of_line_start(position);
    let start = line_map.line_starts[line];
    let character = if line_map.ascii_only || encoding == PositionEncoding::Utf8 {
        position - start
    } else {
        utf16_len_standard(&text[start..position])
    };
    LineAndCharacter {
        line: u32::try_from(line).expect("line fits u32"),
        character: u32::try_from(character).expect("character fits u32"),
    }
}

// port: tsc/internal/core/core.go:UTF16Len
pub fn utf16_len_standard(text: &[u8]) -> usize {
    if text.iter().all(u8::is_ascii) {
        return text.len();
    }
    let mut units = 0;
    let mut index = 0;
    while index < text.len() {
        let decoded = decode_standard_utf8(&text[index..]);
        units += usize::from(decoded.code_point >= 0x1_0000) + 1;
        index += decoded.size;
    }
    units
}

// port: tsc/internal/scanner/scanner.go:ComputePositionOfLineAndUTF16Character
pub fn scanner_position_of_line_and_utf16_character(
    line_starts: &[usize],
    mut line: isize,
    character: isize,
    text: &[u8],
    allow_edits: bool,
) -> usize {
    let line_count = isize::try_from(line_starts.len()).expect("line count fits isize");
    if line < 0 || line >= line_count {
        if allow_edits {
            line = line.clamp(0, line_count - 1);
        } else {
            panic!(
                "Bad line number. Line: {line}, lineStarts.length: {}.",
                line_starts.len()
            );
        }
    }
    let line = usize::try_from(line).expect("line was validated nonnegative");
    let line_start = line_starts[line];
    if character <= 0 {
        return if allow_edits {
            line_start.min(text.len())
        } else {
            assert!(line_start <= text.len(), "line start exceeds text");
            line_start
        };
    }

    let line_end = line_starts.get(line + 1).copied().unwrap_or(text.len());
    let mut utf16_count = 0_isize;
    let mut position = line_start;
    while position < line_end {
        if utf16_count >= character {
            break;
        }
        let decoded = decode_standard_utf8(&text[position..]);
        utf16_count += if decoded.code_point >= 0x1_0000 { 2 } else { 1 };
        position += decoded.size;
    }
    assert!(
        allow_edits || position != line_end || utf16_count >= character,
        "Bad UTF-16 character offset. Line: {line}, character: {character}"
    );
    if allow_edits {
        position.min(text.len())
    } else {
        assert!(position <= text.len(), "position exceeds text");
        position
    }
}

pub fn ecma_line_and_utf16_character_of_position(
    text: &[u8],
    line_starts: &[usize],
    position: usize,
) -> (usize, usize) {
    let line = match line_starts.binary_search(&position) {
        Ok(index) => index,
        Err(0) => 0,
        Err(index) => index - 1,
    };
    let start = line_starts[line];
    (line, utf16_len_standard(&text[start..position]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_and_reclassifies_slices() {
        let text = JsString::from("😀");
        assert_eq!(text.validity(), Validity::Utf8);
        assert_eq!(text.slice(0..1).unwrap().validity(), Validity::Raw);
        assert_eq!(text.slice(0..4).unwrap().as_str(), Some("😀"));

        let sentinel = encode_js_string_rune(0xD800);
        assert_eq!(sentinel.validity(), Validity::Wtf8);
        assert_eq!(sentinel.slice(0..1).unwrap().validity(), Validity::Raw);
    }

    #[test]
    fn combines_only_adjacent_surrogate_pairs() {
        let mut pair = encode_js_string_rune(0xD83D).as_bytes().to_vec();
        pair.extend_from_slice(encode_js_string_rune(0xDE00).as_bytes());
        let combined = combine_surrogate_pairs(&JsString::from_bytes(pair));
        assert_eq!(combined.as_str(), Some("😀"));
        assert_eq!(
            combine_surrogate_pairs(&encode_js_string_rune(0xD800)).validity(),
            Validity::Wtf8
        );
    }

    #[test]
    fn helper_decoders_are_intentionally_distinct() {
        let sentinel = encode_js_string_rune(0xD800);
        assert_eq!(truncate_by_runes(&sentinel, 1).as_bytes(), &[0xED]);
        assert_eq!(
            lower_first_char(&sentinel).as_bytes(),
            &[0xEF, 0xBF, 0xBD, 0xA0, 0x80]
        );
        assert_eq!(to_lower_js(&sentinel).as_bytes(), sentinel.as_bytes());
        assert_eq!(to_upper_js(&sentinel).as_bytes(), sentinel.as_bytes());
    }

    #[test]
    fn javascript_casing_matches_contextual_and_multi_scalar_cases() {
        assert_eq!(
            to_lower_js(&JsString::from("İSPANYOL")).as_str(),
            Some("i̇spanyol")
        );
        assert_eq!(to_lower_js(&JsString::from("ΟΣ")).as_str(), Some("ος"));
        assert_eq!(to_lower_js(&JsString::from("ΣA")).as_str(), Some("σa"));
        assert_eq!(to_upper_js(&JsString::from("ßﬁ")).as_str(), Some("SSFI"));
    }

    #[test]
    fn source_bom_decoding_matches_go() {
        assert_eq!(
            SourceText::from_bytes(b"\xEF\xBB\xBFabc").as_bytes(),
            b"abc"
        );
        assert_eq!(
            SourceText::from_bytes(&[0xFF, 0xFE, 0x3D, 0xD8, 0x00, 0xDE]).as_str(),
            Some("😀")
        );
        assert_eq!(
            SourceText::from_bytes(&[0xFE, 0xFF, 0xD8, 0x00]).as_str(),
            Some("�")
        );
    }

    #[test]
    fn position_paths_keep_their_distinct_rounding() {
        let text = "😀".as_bytes();
        let position_map = PositionMap::new(text);
        let lsp = LspLineMap::new(text);
        assert_eq!(position_map.utf16_to_utf8(1), 1);
        assert_eq!(
            lsp_line_and_character_to_position(
                text,
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
            scanner_position_of_line_and_utf16_character(&[0], 0, 1, text, false),
            4
        );
    }

    #[test]
    fn line_maps_use_different_line_break_sets() {
        let text = "a\u{2028}b\u{2029}c".as_bytes();
        assert_eq!(compute_ecma_line_starts(text), vec![0, 4, 8]);
        assert_eq!(LspLineMap::new(text).line_starts(), &[0]);
    }

    #[test]
    fn escaping_preserves_sentinels_as_utf16_and_replaces_raw_bytes() {
        assert_eq!(
            escape_string(&encode_js_string_rune(0xD800), QuoteChar::Double).as_bytes(),
            br"\uD800"
        );
        assert_eq!(
            escape_string(&JsString::from_bytes([0xFF]), QuoteChar::Double).as_bytes(),
            br"\uFFFD"
        );
        assert_eq!(
            escape_string(&JsString::from("a\"b"), QuoteChar::Double).as_bytes(),
            br#"a\"b"#
        );
    }
}
