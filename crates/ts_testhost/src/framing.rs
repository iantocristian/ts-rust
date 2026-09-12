//! Bounded Content-Length framing and strict JSON decoding for the test host.
//!
//! Valid framing follows pinned `internal/jsonrpc/baseproto.go`. The transport
//! contract deliberately rejects ambiguous or unbounded malformed input earlier.

use std::fmt;
use std::io::{self, BufRead, Write};

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

pub const MAX_BODY: usize = 8 * 1024 * 1024;
pub const MAX_HEADER: usize = 8192;
/// Maximum number of nested JSON arrays and objects in a message.
pub const MAX_JSON_DEPTH: usize = 64;

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn header_line<R: BufRead>(reader: &mut R, total: &mut usize) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        if *total == MAX_HEADER {
            return Err(invalid("testhost header exceeds byte limit"));
        }
        let mut byte = [0];
        match reader.read_exact(&mut byte) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof && *total == 0 => {
                return Ok(None);
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated testhost header",
                ));
            }
            Err(error) => return Err(error),
        }
        *total += 1;
        if byte[0] == b'\n' {
            if line.pop() != Some(b'\r') {
                return Err(invalid("testhost header requires CRLF"));
            }
            return Ok(Some(line));
        }
        if line.last() == Some(&b'\r') {
            return Err(invalid("testhost header contains a bare carriage return"));
        }
        line.push(byte[0]);
    }
}

fn content_length(value: &[u8]) -> io::Result<usize> {
    let value = value.trim_ascii();
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(invalid(
            "Content-Length must be a positive decimal byte count",
        ));
    }
    let mut length = 0_usize;
    for digit in value {
        length = length
            .checked_mul(10)
            .and_then(|length| length.checked_add(usize::from(digit - b'0')))
            .filter(|length| *length <= MAX_BODY)
            .ok_or_else(|| invalid("testhost body exceeds byte limit"))?;
    }
    if length == 0 {
        return Err(invalid("Content-Length must be positive"));
    }
    Ok(length)
}

/// Read exactly one payload, retaining subsequent frames in `reader`.
///
/// `None` means EOF before any header byte. EOF inside a header or body is an
/// error. After any error the connection must be discarded; framing recovery is
/// not part of this protocol.
pub fn read<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut total = 0;
    let mut length = None;
    loop {
        let Some(line) = header_line(reader, &mut total)? else {
            return Ok(None);
        };
        if line.is_empty() {
            break;
        }
        let colon = line
            .iter()
            .position(|byte| *byte == b':')
            .ok_or_else(|| invalid("testhost header has no colon"))?;
        let (name, value) = (&line[..colon], &line[colon + 1..]);
        if name.is_empty()
            || !name
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(byte))
            || !value
                .iter()
                .all(|byte| *byte == b'\t' || (b' '..=b'~').contains(byte))
        {
            return Err(invalid("malformed testhost header"));
        }
        if name == b"Content-Length" {
            if length.is_some() {
                return Err(invalid("duplicate Content-Length header"));
            }
            length = Some(content_length(value)?);
        }
    }
    let length = length.ok_or_else(|| invalid("missing Content-Length header"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            io::Error::new(io::ErrorKind::UnexpectedEof, "truncated testhost body")
        } else {
            error
        }
    })?;
    Ok(Some(body))
}

/// Write a nonempty bounded payload and flush it as one Content-Length frame.
pub fn write<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    if payload.is_empty() || payload.len() > MAX_BODY {
        return Err(invalid("testhost payload must contain 1..=MAX_BODY bytes"));
    }
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
    writer.write_all(payload)?;
    writer.flush()
}

struct JsonSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for JsonSeed {
    type Value = Value;

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for JsonSeed {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON with unique object keys and bounded nesting")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        if self.depth == MAX_JSON_DEPTH {
            return Err(de::Error::custom("JSON nesting exceeds depth limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(Self {
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Value, A::Error> {
        if self.depth == MAX_JSON_DEPTH {
            return Err(de::Error::custom("JSON nesting exceeds depth limit"));
        }
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            let value = object.next_value_seed(Self {
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

/// Decode one complete JSON value, rejecting recursively duplicated keys.
///
/// The body size and nesting limits also apply to direct callers. Invalid UTF-8,
/// non-finite numbers and trailing non-whitespace bytes are errors.
pub fn parse_json(bytes: &[u8]) -> io::Result<Value> {
    if bytes.len() > MAX_BODY {
        return Err(invalid("testhost body exceeds byte limit"));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = JsonSeed { depth: 0 }
        .deserialize(&mut deserializer)
        .map_err(invalid)?;
    deserializer.end().map_err(invalid)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn fragmented_and_coalesced_frames_preserve_utf8_byte_lengths() {
        let payloads = [r#"{"text":"😀"}"#.as_bytes(), b"[null,true,1]"];
        let mut wire = Vec::new();
        for payload in payloads {
            write(&mut wire, payload).unwrap();
        }
        assert!(wire.starts_with(b"Content-Length: 15\r\n\r\n"));
        let mut reader = BufReader::with_capacity(1, Cursor::new(wire));
        for payload in payloads {
            assert_eq!(read(&mut reader).unwrap().unwrap(), payload);
        }
        assert!(read(&mut reader).unwrap().is_none());
    }

    #[test]
    fn eof_is_clean_only_before_the_first_header_byte() {
        assert!(read(&mut Cursor::new(b"")).unwrap().is_none());
        for wire in [
            b"C".as_slice(),
            b"Content-Length: 1\r\n",
            b"Content-Length: 2\r\n\r\nx",
        ] {
            assert_eq!(
                read(&mut Cursor::new(wire)).unwrap_err().kind(),
                io::ErrorKind::UnexpectedEof
            );
        }
    }

    #[test]
    fn malformed_and_ambiguous_headers_are_rejected() {
        for wire in [
            b"Content-Length: 1\n\nx".as_slice(),
            b"Content-Length: 1\rx\r\n\r\nx",
            b"Content-Length: 1\r\nContent-Length: 1\r\n\r\nx",
            b"Content-Length: -1\r\n\r\nx",
            b"Content-Length: +1\r\n\r\nx",
            b"Content-Length: 0\r\n\r\nx",
            b"Content-Length: \r\n\r\nx",
            b"Content-Length: 1.0\r\n\r\nx",
            b"Content-Length: 8388609\r\n\r\nx",
            b"Content-Length: 99999999999999999999999999999\r\n\r\nx",
            b"content-length: 1\r\n\r\nx",
            b"X\r\nContent-Length: 1\r\n\r\nx",
            b"Bad Name: x\r\nContent-Length: 1\r\n\r\nx",
            b"X: \x01\r\nContent-Length: 1\r\n\r\nx",
            b"\r\n",
        ] {
            assert_eq!(
                read(&mut Cursor::new(wire)).unwrap_err().kind(),
                io::ErrorKind::InvalidData,
                "wire: {wire:?}"
            );
        }
    }

    #[test]
    fn header_limit_counts_all_lines_and_the_final_separator() {
        let suffix = b"\r\nContent-Length: \t1 \t\r\n\r\n";
        let mut wire = b"X: ".to_vec();
        wire.resize(MAX_HEADER - suffix.len(), b'a');
        wire.extend_from_slice(suffix);
        wire.push(b'x');
        assert_eq!(read(&mut Cursor::new(&wire)).unwrap().unwrap(), b"x");
        wire.insert(3, b'a');
        assert_eq!(
            read(&mut Cursor::new(wire)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[derive(Default)]
    struct FlushWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl Write for FlushWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    #[test]
    fn writes_flush_and_reject_invalid_sizes_before_emitting_bytes() {
        let mut writer = FlushWriter::default();
        write(&mut writer, b"{}").unwrap();
        assert_eq!(writer.bytes, b"Content-Length: 2\r\n\r\n{}");
        assert_eq!(writer.flushes, 1);
        assert!(write(&mut writer, b"").is_err());
        assert!(write(&mut writer, &vec![0; MAX_BODY + 1]).is_err());
        assert_eq!(writer.bytes, b"Content-Length: 2\r\n\r\n{}");
        assert_eq!(writer.flushes, 1);
    }

    #[test]
    fn strict_json_rejects_duplicate_keys_at_every_depth() {
        for bytes in [
            br#"{"a":1,"a":2}"#.as_slice(),
            br#"[{"a":{"b":1,"b":2}}]"#,
            br#"{"a":1,"\u0061":2}"#,
        ] {
            assert!(parse_json(bytes)
                .unwrap_err()
                .to_string()
                .contains("duplicate JSON object key"));
        }
        assert_eq!(
            parse_json(br#"[{"a":1},{"a":2}]"#).unwrap(),
            serde_json::json!([{ "a": 1 }, { "a": 2 }])
        );
    }

    #[test]
    fn strict_json_rejects_trailing_invalid_and_nonfinite_input() {
        for bytes in [
            b"{} []".as_slice(),
            b"NaN",
            b"Infinity",
            b"1e400",
            b"{\"x\":\"\xff\"}",
            br#""\ud800""#,
            b"",
        ] {
            assert!(parse_json(bytes).is_err(), "payload: {bytes:?}");
        }
        assert_eq!(parse_json(b"\r\nnull \t").unwrap(), Value::Null);
    }

    #[test]
    fn strict_json_depth_is_explicitly_bounded() {
        let nested = |count: usize| format!("{}0{}", "[".repeat(count), "]".repeat(count));
        assert!(parse_json(nested(MAX_JSON_DEPTH).as_bytes()).is_ok());
        assert!(parse_json(nested(MAX_JSON_DEPTH + 1).as_bytes()).is_err());
        let mixed = format!("{}0{}", "{\"x\":[".repeat(33), "]}".repeat(33));
        assert!(parse_json(mixed.as_bytes()).is_err());
    }
}
