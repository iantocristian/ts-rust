//! JSON adapter for the S04 differential producer; not a compiler-facing API.
use serde_json::{json, Value};
use std::io::{self, Read};
use ts_jsstring::helpers::{self, LiteralEscapeFlags, QuoteChar};
use ts_jsstring::positions::{self, LspLineMap, LspPosition, PositionEncoding, PositionMap};
use ts_jsstring::{strings, JsString, SourceText};

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn evaluate(case: &Value, bytes: &[u8]) -> Value {
    let a = case["a"].as_i64().unwrap_or_default();
    let b = case["b"].as_i64().unwrap_or_default();
    let flag = case["flag"].as_bool().unwrap_or_default();
    let encoding = if flag {
        PositionEncoding::Utf8
    } else {
        PositionEncoding::Utf16
    };
    match case["op"].as_str().unwrap() {
        "source" => {
            let source = SourceText::from_bytes(bytes);
            json!([
                hex(source.as_bytes()),
                source.as_str().map(|text| hex(text.as_bytes()))
            ])
        }
        "slice" => {
            if a < 0 || b < 0 {
                return Value::Null;
            }
            let source = JsString::from_bytes(bytes);
            source
                .slice(a as usize..b as usize)
                .map_or(Value::Null, |slice| {
                    json!([
                        hex(slice.as_bytes()),
                        format!("{:?}", slice.validity()),
                        slice.as_str().map(|text| hex(text.as_bytes()))
                    ])
                })
        }
        "source_slice" => {
            if a < 0 || b < 0 {
                return Value::Null;
            }
            let source = SourceText::from_bytes(bytes);
            source
                .slice(a as usize..b as usize)
                .map_or(Value::Null, |slice| {
                    json!([
                        hex(slice.as_bytes()),
                        format!("{:?}", slice.validity()),
                        slice.as_str().map(|text| hex(text.as_bytes()))
                    ])
                })
        }
        "lower" => json!(hex(&helpers::to_lower_js(bytes))),
        "upper" => json!(hex(&helpers::to_upper_js(bytes))),
        "lower_first" => json!(hex(&helpers::lower_first_char(bytes))),
        "truncate" => json!(hex(helpers::truncate_by_runes(bytes, a as isize))),
        "combine" => json!(hex(&strings::combine_surrogate_pairs(bytes))),
        "encode" => json!(hex(&strings::encode_rune(a as i32))),
        "decode_js" => json!(strings::decode_rune(bytes)),
        "decode_utf8" => json!(strings::decode_utf8(bytes)),
        "escape" => {
            let quote = match a {
                39 => QuoteChar::Single,
                34 => QuoteChar::Double,
                96 => QuoteChar::Backtick,
                _ => panic!("invalid fixture quote"),
            };
            let flags = match b {
                0 => LiteralEscapeFlags::NONE,
                1 => LiteralEscapeFlags::NEVER_ASCII_ESCAPE,
                2 => LiteralEscapeFlags::JSX_ATTRIBUTE_ESCAPE,
                3 => {
                    LiteralEscapeFlags::NEVER_ASCII_ESCAPE
                        | LiteralEscapeFlags::JSX_ATTRIBUTE_ESCAPE
                }
                _ => panic!("invalid fixture flags"),
            };
            json!(hex(&helpers::escape_string_with_flags(bytes, quote, flags)))
        }
        "api_to_utf16" => json!(PositionMap::new(bytes).utf8_to_utf16(a as isize)),
        "api_to_utf8" => json!(PositionMap::new(bytes).utf16_to_utf8(a as isize)),
        "api_ascii" => json!(PositionMap::new(bytes).is_ascii_only()),
        "ecma_lines" => json!(positions::compute_ecma_line_starts(bytes)),
        "lsp_lines" => {
            let map = LspLineMap::new(bytes);
            json!([map.line_starts, map.ascii_only])
        }
        "lsp_line_index" => json!(LspLineMap::new(bytes).compute_index_of_line_start(a as i32)),
        "lsp_to_position" => json!(positions::lsp_line_and_character_to_position(
            bytes,
            &LspLineMap::new(bytes),
            LspPosition {
                line: a as u32,
                character: b as u32
            },
            encoding,
        )),
        "lsp_from_position" => {
            let p = positions::lsp_position_to_line_and_character(
                bytes,
                &LspLineMap::new(bytes),
                a as i32,
                encoding,
            );
            json!([p.line, p.character])
        }
        "scanner_to_position" => json!(positions::compute_position_of_line_and_utf16_character(
            &positions::compute_ecma_line_starts(bytes),
            a as isize,
            b as isize,
            bytes,
            flag,
        )),
        "scanner_byte_position" => json!(positions::compute_position_of_line_and_byte_offset(
            &positions::compute_ecma_line_starts(bytes),
            a as isize,
            b as isize,
        )),
        "scanner_line" => json!(positions::compute_line_of_position(
            &positions::compute_ecma_line_starts(bytes),
            a as isize
        )),
        "scanner_end_line" => json!(positions::get_ecma_end_line_position(bytes, a as isize)),
        "scanner_from_position" => json!(positions::get_ecma_line_and_utf16_character_of_position(
            bytes, a as isize
        )),
        "byte_from_position" => json!(positions::position_to_line_and_byte_offset(
            a as isize,
            &positions::compute_ecma_line_starts(bytes)
        )),
        "utf16_len" => json!(positions::utf16_len(bytes)),
        _ => unreachable!("operation was validated before entering the panic boundary"),
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Value> = serde_json::from_str(&input).unwrap();
    let supported = [
        "source",
        "slice",
        "source_slice",
        "lower",
        "upper",
        "lower_first",
        "truncate",
        "combine",
        "encode",
        "decode_js",
        "decode_utf8",
        "escape",
        "api_to_utf16",
        "api_to_utf8",
        "api_ascii",
        "ecma_lines",
        "lsp_lines",
        "lsp_line_index",
        "lsp_to_position",
        "lsp_from_position",
        "scanner_to_position",
        "scanner_byte_position",
        "scanner_line",
        "scanner_end_line",
        "scanner_from_position",
        "byte_from_position",
        "utf16_len",
    ];
    for case in &cases {
        assert!(
            supported.contains(&case["op"].as_str().unwrap()),
            "unknown operation"
        );
    }
    // Panics are observable upstream outcomes for out-of-range positions. Compare
    // their occurrence without flooding the producer log with expected backtraces.
    std::panic::set_hook(Box::new(|_| {}));
    let results: Vec<Value> = cases
        .iter()
        .map(|case| {
            let bytes = unhex(case["text"].as_str().unwrap());
            match std::panic::catch_unwind(|| evaluate(case, &bytes)) {
                Ok(value) => json!({"id":case["id"], "panic":false, "value":value}),
                Err(_) => json!({"id":case["id"], "panic":true, "value":null}),
            }
        })
        .collect();
    serde_json::to_writer(io::stdout().lock(), &results).unwrap();
}
