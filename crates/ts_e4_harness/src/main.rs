//! The E4 harness: answer the fixture probes from `ts_jsstring`.
//!
//! This writes one JSON object per criterion of `probe -> value` pairs, in
//! exactly the shape `oracle/e4` writes from the pinned Corsa packages.
//! `scripts/e4-harness.py` compares the two objects; neither side sees the
//! other's answers, and a probe missing from either side is a mismatch.
//!
//! The probe lists are derived from the fixture manifest by the same rules on
//! both sides: every byte offset of a text plus the declared out-of-range
//! values, every line of the relevant line map plus the declared out-of-range
//! lines, and the declared character offsets. Panics are values here, because
//! several of these conversions are specified to panic and E4 compares that too.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::Deserialize;
use ts_jsstring::escape::QuoteChar;
use ts_jsstring::jsstring::Validity;
use ts_jsstring::positions::{
    compute_ecma_line_starts, compute_line_of_position, compute_lsp_line_starts,
    compute_position_map, compute_position_of_line_and_byte_offset,
    compute_position_of_line_and_utf16_character, ecma_line_and_byte_offset_of_position,
    ecma_line_and_utf16_character_of_position, position_to_line_and_byte_offset, utf16_len,
    Converters, LspLineMap, PositionEncoding, TextPos,
};
use ts_jsstring::rune::{
    combine_surrogate_pairs, decode_js_string_rune, decode_rune, encode_js_string_rune,
    is_high_surrogate, is_low_surrogate, is_surrogate,
};
use ts_jsstring::{
    escape_string, lower_first_char, to_lower_js, to_upper_js, truncate_by_runes, SourceText,
};

#[derive(Deserialize)]
struct Fixture {
    id: String,
    bytes: String,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
struct Probes {
    out_of_range_offsets: Vec<i64>,
    characters: Vec<i64>,
    out_of_range_lines: Vec<i64>,
}

#[derive(Deserialize)]
struct Manifest {
    files: Vec<Fixture>,
    texts: Vec<Fixture>,
    helpers: Vec<Fixture>,
    runes: Vec<i64>,
    truncations: Vec<i64>,
    probes: Probes,
    case_sweep: Vec<u32>,
    range_sweep: Vec<u32>,
}

type Group = BTreeMap<String, String>;

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("fixture hex"))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
}

/// Run a probe whose upstream counterpart can panic, reporting `"panic"` where
/// it does, so panic behavior is compared rather than fatal.
fn guard(body: impl FnOnce() -> String) -> String {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| "panic".to_string())
}

/// Every byte offset of the text plus the declared out-of-range values.
fn offset_probes(text: &[u8], manifest: &Manifest) -> Vec<i64> {
    (0..=text.len() as i64)
        .chain(manifest.probes.out_of_range_offsets.iter().copied())
        .collect()
}

fn line_probes(line_starts: &[TextPos], manifest: &Manifest) -> Vec<i64> {
    (0..line_starts.len() as i64)
        .chain(manifest.probes.out_of_range_lines.iter().copied())
        .collect()
}

fn positions(values: &[TextPos]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn validity_name(validity: Validity) -> &'static str {
    match validity {
        Validity::Utf8 => "utf8",
        Validity::Wtf8 => "wtf8",
        Validity::Raw => "raw",
    }
}

fn source_decoding(manifest: &Manifest) -> Group {
    let mut out = Group::new();
    for file in &manifest.files {
        let decoded = SourceText::decode(&unhex(&file.bytes));
        out.insert(format!("decode/{}", file.id), hex(decoded.as_bytes()));
        // Decoding never fails; `ok` is false only when the file cannot be read,
        // which is not a property of the bytes.
        out.insert(format!("ok/{}", file.id), "true".to_string());
    }
    out
}

fn slice_validity(manifest: &Manifest) -> Group {
    let mut out = Group::new();
    for fixture in &manifest.texts {
        let text = unhex(&fixture.bytes);
        let id = &fixture.id;
        let n = text.len();
        let string = ts_jsstring::JsString::new(text.clone());
        for a in 0..=n {
            for b in a..=n {
                let part = string.slice(a, b).expect("in-range slice");
                let key = format!("/{id}/{a}/{b}");
                out.insert(format!("bytes{key}"), hex(part.as_bytes()));
                out.insert(
                    format!("validity{key}"),
                    validity_name(part.validity()).into(),
                );
                // A str view exists exactly when the sliced bytes are valid
                // UTF-8, whatever the parent's tag was. The oracle answers the
                // same two probes from Go's own validity test and its own
                // implementation of the tag rule, so the relationship between
                // them is established by the comparison rather than asserted
                // here against this implementation's own definitions.
                out.insert(format!("str{key}"), part.as_str().is_some().to_string());
            }
        }
        for (a, b) in [(0, n + 1), (n + 1, n + 2), (1, 0)] {
            out.insert(
                format!("oob/{id}/{a}/{b}"),
                string
                    .slice(a, b)
                    .map_or_else(|| "panic".to_string(), |part| hex(part.as_bytes())),
            );
        }
    }
    out
}

fn helper_semantics(manifest: &Manifest) -> Group {
    let mut out = Group::new();
    let quotes = [
        ("single", QuoteChar::SingleQuote),
        ("double", QuoteChar::DoubleQuote),
        ("backtick", QuoteChar::Backtick),
    ];
    for fixture in &manifest.helpers {
        let value = unhex(&fixture.bytes);
        let id = &fixture.id;
        out.insert(format!("to_lower/{id}"), hex(&to_lower_js(&value)));
        out.insert(format!("to_upper/{id}"), hex(&to_upper_js(&value)));
        out.insert(format!("lower_first/{id}"), hex(&lower_first_char(&value)));
        out.insert(
            format!("combine/{id}"),
            hex(&combine_surrogate_pairs(&value)),
        );
        for n in &manifest.truncations {
            out.insert(
                format!("truncate/{id}/{n}"),
                hex(&truncate_by_runes(&value, *n)),
            );
        }
        for (name, quote) in quotes {
            out.insert(
                format!("escape/{id}/{name}"),
                hex(&escape_string(&value, quote)),
            );
        }
        for i in 0..=value.len() {
            let (r, size) = decode_js_string_rune(&value[i..]);
            out.insert(format!("decode_rune/{id}/{i}"), format!("{r},{size}"));
            let (r, size) = decode_rune(&value[i..]);
            out.insert(format!("decode_standard/{id}/{i}"), format!("{r},{size}"));
        }
    }
    for cp in &manifest.runes {
        let ch = *cp as i32;
        out.insert(format!("encode_rune/{cp}"), hex(&encode_js_string_rune(ch)));
        out.insert(
            format!("surrogate/{cp}"),
            format!(
                "{},{},{}",
                is_surrogate(ch),
                is_high_surrogate(ch),
                is_low_surrogate(ch)
            ),
        );
    }
    for cp in &manifest.case_sweep {
        let value = char::from_u32(*cp)
            .expect("the pinned table holds scalar values")
            .to_string();
        let bytes = value.as_bytes();
        out.insert(format!("sweep_lower/{cp}"), hex(&to_lower_js(bytes)));
        out.insert(format!("sweep_upper/{cp}"), hex(&to_upper_js(bytes)));
        out.insert(
            format!("sweep_lower_first/{cp}"),
            hex(&lower_first_char(bytes)),
        );
    }
    // The Cased and Case_Ignorable tables are only observable through the
    // Final_Sigma context, so each swept code point is placed after a sigma in
    // two contexts. "cased", "case ignorable" and "neither" give three distinct
    // pairs of results.
    for cp in &manifest.range_sweep {
        let mut value = SIGMA_PREFIX.to_string();
        value.push(char::from_u32(*cp).expect("the sweep excludes surrogates"));
        out.insert(
            format!("sigma_final/{cp}"),
            hex(&to_lower_js(value.as_bytes())),
        );
        value.push('A');
        out.insert(
            format!("sigma_followed/{cp}"),
            hex(&to_lower_js(value.as_bytes())),
        );
    }
    out
}

/// A cased letter followed by a capital sigma: the backward half of the
/// Final_Sigma condition.
const SIGMA_PREFIX: &str = "\u{039f}\u{03a3}";

fn utf8_positions(manifest: &Manifest) -> Group {
    let mut out = Group::new();
    let converters = Converters::new(PositionEncoding::Utf8);
    for fixture in &manifest.texts {
        let text = unhex(&fixture.bytes);
        let id = &fixture.id;
        let ecma = compute_ecma_line_starts(&text);
        let lsp = compute_lsp_line_starts(&text);
        out.insert(format!("ecma_line_starts/{id}"), positions(&ecma));
        out.insert(format!("lsp_line_starts/{id}"), positions(&lsp.line_starts));
        out.insert(format!("lsp_ascii_only/{id}"), lsp.ascii_only.to_string());

        for p in offset_probes(&text, manifest) {
            out.insert(
                format!("line_of_position/{id}/{p}"),
                compute_line_of_position(&ecma, p).to_string(),
            );
            let (line, offset) = position_to_line_and_byte_offset(p, &ecma);
            out.insert(
                format!("position_to_line_byte/{id}/{p}"),
                format!("{line},{offset}"),
            );
            out.insert(
                format!("lsp_index_of_line_start/{id}/{p}"),
                lsp.compute_index_of_line_start(p as TextPos).to_string(),
            );
            out.insert(
                format!("ecma_line_byte/{id}/{p}"),
                guard(|| {
                    let (line, offset) = ecma_line_and_byte_offset_of_position(&ecma, p);
                    format!("{line},{offset}")
                }),
            );
            out.insert(
                format!("lsp8_from_position/{id}/{p}"),
                guard(|| {
                    let (line, character) =
                        converters.position_to_line_and_character(&text, &lsp, p as TextPos);
                    format!("{line},{character}")
                }),
            );
        }
        for line in line_probes(&ecma, manifest) {
            for offset in [0i64, 1] {
                out.insert(
                    format!("position_of_line_byte/{id}/{line}/{offset}"),
                    guard(|| {
                        compute_position_of_line_and_byte_offset(&ecma, line, offset).to_string()
                    }),
                );
            }
        }
        for line in line_probes(&lsp.line_starts, manifest) {
            for character in &manifest.probes.characters {
                out.insert(
                    format!("lsp8_to_position/{id}/{line}/{character}"),
                    lsp_to_position(converters, &text, &lsp, line, *character),
                );
            }
        }
    }
    out
}

fn lsp_to_position(
    converters: Converters,
    text: &[u8],
    lsp: &LspLineMap,
    line: i64,
    character: i64,
) -> String {
    guard(|| {
        converters
            .line_and_character_to_position(text, lsp, line as u32, character as u32)
            .to_string()
    })
}

fn utf16_positions(manifest: &Manifest) -> Group {
    let mut out = Group::new();
    let converters = Converters::new(PositionEncoding::Utf16);
    for fixture in &manifest.texts {
        let text = unhex(&fixture.bytes);
        let id = &fixture.id;
        let ecma = compute_ecma_line_starts(&text);
        let lsp = compute_lsp_line_starts(&text);
        let map = compute_position_map(&text);

        out.insert(
            format!("pm_ascii_only/{id}"),
            map.is_ascii_only().to_string(),
        );
        out.insert(format!("utf16_len/{id}"), utf16_len(&text).to_string());

        for p in offset_probes(&text, manifest) {
            out.insert(
                format!("pm_8_to_16/{id}/{p}"),
                map.utf8_to_utf16(p).to_string(),
            );
            out.insert(
                format!("pm_16_to_8/{id}/{p}"),
                map.utf16_to_utf8(p).to_string(),
            );
            out.insert(
                format!("ecma_line_utf16/{id}/{p}"),
                guard(|| {
                    let (line, character) =
                        ecma_line_and_utf16_character_of_position(&text, &ecma, p);
                    format!("{line},{character}")
                }),
            );
            out.insert(
                format!("lsp16_from_position/{id}/{p}"),
                guard(|| {
                    let (line, character) =
                        converters.position_to_line_and_character(&text, &lsp, p as TextPos);
                    format!("{line},{character}")
                }),
            );
        }
        for line in line_probes(&ecma, manifest) {
            for character in &manifest.probes.characters {
                for allow_edits in [false, true] {
                    out.insert(
                        format!("scanner_position/{id}/{line}/{character}/{allow_edits}"),
                        guard(|| {
                            compute_position_of_line_and_utf16_character(
                                &ecma,
                                line,
                                *character,
                                &text,
                                allow_edits,
                            )
                            .to_string()
                        }),
                    );
                }
            }
        }
        for line in line_probes(&lsp.line_starts, manifest) {
            for character in &manifest.probes.characters {
                out.insert(
                    format!("lsp16_to_position/{id}/{line}/{character}"),
                    lsp_to_position(converters, &text, &lsp, line, *character),
                );
            }
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(fixtures), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: ts_e4_harness <fixtures.json> <output.json>");
        std::process::exit(2);
    };
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(&fixtures).unwrap_or_else(|e| panic!("{fixtures}: {e}")),
    )
    .expect("fixture manifest");

    let mut result: BTreeMap<&str, Group> = [
        ("source_decoding", source_decoding(&manifest)),
        ("slice_validity", slice_validity(&manifest)),
        ("helper_semantics", helper_semantics(&manifest)),
    ]
    .into_iter()
    .collect();
    // The position probes are the ones specified to panic, and their messages
    // are not the result. The hook is silenced only for them, so an unexpected
    // panic anywhere else still prints.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    result.insert("utf8_positions", utf8_positions(&manifest));
    result.insert("utf16_positions", utf16_positions(&manifest));
    std::panic::set_hook(previous);

    for (name, group) in &result {
        eprintln!("{name}: {} probes", group.len());
    }
    std::fs::write(
        &output,
        serde_json::to_vec(&result).expect("serializable report"),
    )
    .unwrap_or_else(|e| panic!("{output}: {e}"));
}
