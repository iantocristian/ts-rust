//! The e4 producer's Rust half: computes every E4 fixture in tests/e4/fixtures.json
//! with the ts_jsstring crate and compares the results with the pinned Go
//! oracle's output (tools/oracle-e4), then prints one metric per S04 criterion.
//!
//! Usage: `e4_harness <fixtures.json> <oracle.json>`

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::Deserialize;
use serde_json::{json, Value};
use ts_jsstring::escape::{escape_string_worker, LiteralTextFlags, QuoteChar};
use ts_jsstring::helpers::{lower_first_char, to_lower_js, to_upper_js, truncate_by_runes};
use ts_jsstring::jsstring::{classify, JsString, Validity};
use ts_jsstring::line_map::{
    compute_ecma_line_starts, compute_line_of_position, compute_lsp_line_starts,
    position_to_line_and_byte_offset, utf16_len,
};
use ts_jsstring::lsp::{
    line_and_character_to_position, position_to_line_and_character, PositionEncoding,
};
use ts_jsstring::position_map::compute_position_map;
use ts_jsstring::scanner_positions::{
    compute_position_of_line_and_byte_offset, compute_position_of_line_and_utf16_character,
    get_ecma_end_line_position, get_ecma_line_and_byte_offset_of_position,
    get_ecma_line_and_utf16_character_of_position,
};
use ts_jsstring::source_text::decode_bytes;
use ts_jsstring::wtf8::{
    combine_surrogate_pairs, decode_js_string_rune, decode_rune, encode_js_string_rune,
};

#[derive(Deserialize)]
struct Fixtures {
    decode: Vec<Simple>,
    helpers: Vec<Helper>,
    escape: Vec<Escape>,
    slices: Vec<Slices>,
    texts: Vec<Text>,
}

#[derive(Deserialize)]
struct Simple {
    id: String,
    input: String,
}

#[derive(Deserialize)]
struct Helper {
    id: String,
    input: String,
    truncate: Vec<i64>,
}

#[derive(Deserialize)]
struct Escape {
    id: String,
    input: String,
    quote: String,
    flags: u32,
}

#[derive(Deserialize)]
struct Slices {
    id: String,
    input: String,
    ranges: Vec<[usize; 2]>,
}

#[derive(Deserialize)]
struct Text {
    id: String,
    #[serde(rename = "text")]
    bytes: String,
    utf16_offsets: Vec<usize>,
    byte_offsets: Vec<usize>,
    positions: Vec<[i64; 2]>,
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex fixture"))
        .collect()
}

fn hx(b: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        write!(s, "{x:02X}").expect("writing to a String cannot fail");
    }
    s
}

/// A guarded computation: a panic becomes the string "panic", as in the oracle.
fn guard(f: impl FnOnce() -> Value) -> Value {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| Value::String("panic".into()))
}

fn pair(a: impl Into<Value>, b: impl Into<Value>) -> Value {
    json!([a.into(), b.into()])
}

struct Compare {
    failures: BTreeMap<&'static str, Vec<String>>,
    checks: usize,
}

impl Compare {
    fn eq(
        &mut self,
        criterion: &'static str,
        what: &str,
        actual: &Value,
        expected: Option<&Value>,
    ) {
        self.checks += 1;
        if Some(actual) != expected {
            self.failures.entry(criterion).or_default().push(format!(
                "{what}: rust {actual} vs go {}",
                expected.map_or("<missing>".to_string(), Value::to_string)
            ));
        }
    }

    fn rule(&mut self, criterion: &'static str, what: &str, holds: bool) {
        self.checks += 1;
        if !holds {
            self.failures
                .entry(criterion)
                .or_default()
                .push(what.to_string());
        }
    }
}

fn expected<'a>(oracle: &'a Value, section: &str, id: &str) -> Option<&'a Value> {
    oracle.get(section)?.get(id)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 3,
        "usage: e4_harness <fixtures.json> <oracle.json>"
    );
    let fixtures: Fixtures = serde_json::from_slice(&std::fs::read(&args[1]).expect("fixtures"))
        .expect("fixtures schema");
    let oracle: Value =
        serde_json::from_slice(&std::fs::read(&args[2]).expect("oracle")).expect("oracle json");
    std::panic::set_hook(Box::new(|_| {}));
    let mut cmp = Compare {
        failures: BTreeMap::new(),
        checks: 0,
    };

    for f in &fixtures.decode {
        let actual = json!({ "output": hx(&decode_bytes(&unhex(&f.input))), "ok": true });
        cmp.eq(
            "source_decoding",
            &format!("decode/{}", f.id),
            &actual,
            expected(&oracle, "decode", &f.id),
        );
    }

    for f in &fixtures.helpers {
        let s = unhex(&f.input);
        let truncate: BTreeMap<String, String> = f
            .truncate
            .iter()
            .map(|&n| (n.to_string(), hx(truncate_by_runes(&s, n))))
            .collect();
        let mut js_runes = Vec::new();
        let mut reencoded = Vec::new();
        let mut i = 0;
        while i < s.len() {
            let (r, size) = decode_js_string_rune(&s[i..]);
            js_runes.push(json!([r, size]));
            encode_js_string_rune(&mut reencoded, r);
            i += size;
        }
        let mut std_runes = Vec::new();
        let mut i = 0;
        while i < s.len() {
            let (r, size) = decode_rune(&s[i..]);
            std_runes.push(json!([r, size]));
            i += size;
        }
        let actual = json!({
            "lower": hx(&to_lower_js(&s)),
            "upper": hx(&to_upper_js(&s)),
            "lower_first": hx(&lower_first_char(&s)),
            "combined": hx(&combine_surrogate_pairs(&s)),
            "truncate": truncate,
            "js_runes": js_runes,
            "std_runes": std_runes,
            "reencoded": hx(&reencoded),
            "utf16_len": utf16_len(&s),
        });
        cmp.eq(
            "helper_semantics",
            &format!("helpers/{}", f.id),
            &actual,
            expected(&oracle, "helpers", &f.id),
        );
    }

    for f in &fixtures.escape {
        let quote = match f.quote.as_str() {
            "\"" => QuoteChar::Double,
            "'" => QuoteChar::Single,
            "`" => QuoteChar::Backtick,
            other => panic!("unknown quote {other}"),
        };
        let mut out = Vec::new();
        escape_string_worker(
            &unhex(&f.input),
            quote,
            LiteralTextFlags::from_bits(f.flags),
            &mut out,
        );
        let actual = json!({ "output": hx(&out) });
        cmp.eq(
            "helper_semantics",
            &format!("escape/{}", f.id),
            &actual,
            expected(&oracle, "escape", &f.id),
        );
    }

    for f in &fixtures.slices {
        let bytes = unhex(&f.input);
        let parent = JsString::from_bytes(bytes.as_slice());
        let mut actual = serde_json::Map::new();
        for &[a, b] in &f.ranges {
            let slice = parent.slice(a..b);
            actual.insert(format!("{a}-{b}"), Value::String(hx(slice.as_bytes())));
            let valid = std::str::from_utf8(slice.as_bytes()).is_ok();
            cmp.rule(
                "slice_validity",
                &format!("slices/{}/{a}-{b}: tag matches the bytes", f.id),
                slice.tag() == classify(slice.as_bytes()),
            );
            cmp.rule(
                "slice_validity",
                &format!("slices/{}/{a}-{b}: str view only for valid UTF-8", f.id),
                slice.as_str().is_some() == (valid && slice.tag() == Validity::Utf8),
            );
            let boundaries = parent
                .as_str()
                .is_some_and(|s| s.is_char_boundary(a) && s.is_char_boundary(b));
            cmp.rule(
                "slice_validity",
                &format!("slices/{}/{a}-{b}: slice_str needs boundaries", f.id),
                parent.slice_str(a..b).is_some() == boundaries,
            );
            if !valid {
                cmp.rule(
                    "slice_validity",
                    &format!(
                        "slices/{}/{a}-{b}: invalid bytes are Raw or Wtf8, never Utf8",
                        f.id
                    ),
                    slice.tag() != Validity::Utf8,
                );
            }
        }
        cmp.eq(
            "slice_validity",
            &format!("slices/{}", f.id),
            &Value::Object(actual),
            expected(&oracle, "slices", &f.id),
        );
    }

    for f in &fixtures.texts {
        let text = unhex(&f.bytes);
        let text = text.as_slice();
        let ecma = compute_ecma_line_starts(text);
        let lsp = compute_lsp_line_starts(text);
        let pm = compute_position_map(text);
        let exp = expected(&oracle, "texts", &f.id);
        let get = |k: &str| exp.and_then(|e| e.get(k));
        let sub = |k: &str, sk: &str| exp.and_then(|e| e.get(k)).and_then(|m| m.get(sk));
        let t = |k: &'static str| -> &'static str { k };

        cmp.eq(
            "utf8_positions",
            &format!("texts/{}/ecma_line_starts", f.id),
            &json!(ecma),
            get("ecma_line_starts"),
        );
        cmp.eq(
            "utf8_positions",
            &format!("texts/{}/file_ecma_line_starts", f.id),
            &json!(ecma),
            get("file_ecma_line_starts"),
        );
        cmp.eq(
            "utf8_positions",
            &format!("texts/{}/lsp_line_starts", f.id),
            &json!(lsp.line_starts),
            get("lsp_line_starts"),
        );
        cmp.eq(
            "utf8_positions",
            &format!("texts/{}/lsp_ascii_only", f.id),
            &json!(lsp.ascii_only),
            get("lsp_ascii_only"),
        );
        cmp.eq(
            "utf16_positions",
            &format!("texts/{}/pm_ascii_only", f.id),
            &json!(pm.is_ascii_only()),
            get("pm_ascii_only"),
        );

        for &u in &f.utf16_offsets {
            cmp.eq(
                "utf16_positions",
                &format!("texts/{}/utf16_to_utf8/{u}", f.id),
                &json!(pm.utf16_to_utf8(u)),
                sub("utf16_to_utf8", &u.to_string()),
            );
        }
        for &b in &f.byte_offsets {
            let k = b.to_string();
            cmp.eq(
                "utf16_positions",
                &format!("texts/{}/utf8_to_utf16/{b}", f.id),
                &json!(pm.utf8_to_utf16(b)),
                sub("utf8_to_utf16", &k),
            );
            if b <= text.len() {
                cmp.eq(
                    "utf16_positions",
                    &format!("texts/{}/utf16_len_prefix/{b}", f.id),
                    &json!(utf16_len(&text[..b])),
                    sub("utf16_len_prefix", &k),
                );
            }
            let v = guard(|| {
                let (line, off) = position_to_line_and_byte_offset(b, &ecma);
                pair(line, off)
            });
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/pos_to_line_byte/{b}", f.id),
                &v,
                sub("pos_to_line_byte", &k),
            );
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/line_of_position/{b}", f.id),
                &json!(compute_line_of_position(&ecma, b)),
                sub("line_of_position", &k),
            );
            let v = guard(|| {
                let (line, ch) = get_ecma_line_and_utf16_character_of_position(text, &ecma, b);
                pair(line, ch)
            });
            cmp.eq(
                "utf16_positions",
                &format!("texts/{}/ecma_line_utf16_of_pos/{b}", f.id),
                &v,
                sub("ecma_line_utf16_of_pos", &k),
            );
            let v = guard(|| {
                let (line, off) = get_ecma_line_and_byte_offset_of_position(&ecma, b);
                pair(line, off)
            });
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/ecma_line_byte_of_pos/{b}", f.id),
                &v,
                sub("ecma_line_byte_of_pos", &k),
            );
            for (enc, key, criterion) in [
                (
                    PositionEncoding::Utf16,
                    "lsp_pos_utf16",
                    t("utf16_positions"),
                ),
                (PositionEncoding::Utf8, "lsp_pos_utf8", t("utf8_positions")),
            ] {
                let v = guard(|| {
                    let (l, c) = position_to_line_and_character(text, &lsp, enc, b);
                    pair(l, c)
                });
                cmp.eq(
                    criterion,
                    &format!("texts/{}/{key}/{b}", f.id),
                    &v,
                    sub(key, &k),
                );
            }
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/lsp_index_of_line_start/{b}", f.id),
                &json!(lsp.compute_index_of_line_start(b)),
                sub("lsp_index_of_line_start", &k),
            );
        }
        for line in 0..ecma.len() {
            let v = guard(|| json!(get_ecma_end_line_position(text, &ecma, line)));
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/ecma_end_line/{line}", f.id),
                &v,
                sub("ecma_end_line", &line.to_string()),
            );
        }
        for &[l, c] in &f.positions {
            let k = format!("{l},{c}");
            for (enc, key, criterion) in [
                (PositionEncoding::Utf16, "lsp_utf16", t("utf16_positions")),
                (PositionEncoding::Utf8, "lsp_utf8", t("utf8_positions")),
            ] {
                let v = guard(|| {
                    json!(line_and_character_to_position(
                        text,
                        &lsp,
                        enc,
                        l as i32 as u32,
                        c as i32 as u32
                    ))
                });
                cmp.eq(
                    criterion,
                    &format!("texts/{}/{key}/{k}", f.id),
                    &v,
                    sub(key, &k),
                );
            }
            let v = guard(|| {
                json!(compute_position_of_line_and_utf16_character(
                    &ecma, l, c, text, true
                ))
            });
            cmp.eq(
                "utf16_positions",
                &format!("texts/{}/scanner_utf16_edit/{k}", f.id),
                &v,
                sub("scanner_utf16_edit", &k),
            );
            let v = guard(|| {
                json!(compute_position_of_line_and_utf16_character(
                    &ecma, l, c, text, false
                ))
            });
            cmp.eq(
                "utf16_positions",
                &format!("texts/{}/scanner_utf16_strict/{k}", f.id),
                &v,
                sub("scanner_utf16_strict", &k),
            );
            let v = guard(|| json!(compute_position_of_line_and_byte_offset(&ecma, l, c)));
            cmp.eq(
                "utf8_positions",
                &format!("texts/{}/scanner_byte/{k}", f.id),
                &v,
                sub("scanner_byte", &k),
            );
        }
    }

    let criteria = [
        "source_decoding",
        "helper_semantics",
        "slice_validity",
        "utf8_positions",
        "utf16_positions",
    ];
    let mut metrics = serde_json::Map::new();
    for c in criteria {
        let failures = cmp.failures.get(c).map_or(0, Vec::len);
        metrics.insert(c.to_string(), Value::Bool(failures == 0));
        for f in cmp.failures.get(c).into_iter().flatten().take(25) {
            eprintln!("e4 {c}: {f}");
        }
        if failures > 25 {
            eprintln!("e4 {c}: {} more", failures - 25);
        }
    }
    eprintln!("e4: {} comparisons", cmp.checks);
    println!("{}", json!({ "metrics": metrics }));
}
