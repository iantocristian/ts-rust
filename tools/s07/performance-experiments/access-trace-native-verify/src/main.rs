#![forbid(unsafe_code)]
//! Streaming reference-equivalent verifier. Stdin is one uncompressed trace;
//! the Python parent owns gzip integrity and compressed-byte limits.
mod json;
mod observations;
mod registry;

use observations::{Observations, Record};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

const MIB: usize = 1024 * 1024;
#[derive(Debug)]
pub struct Error(String);
impl Error {
    fn new(message: &str) -> Self {
        Self(message.to_owned())
    }
}
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self(error.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error::new(message))
    }
}

pub struct Settings {
    max_payload: u64,
    max_block: usize,
    max_record: usize,
    max_files: u64,
    max_active_nodes: usize,
    allow_empty: bool,
    expected: [Option<u64>; 4],
    progress_records: u64,
}
impl Settings {
    fn read(value: &Value) -> Result<Self> {
        let value = registry::object(value)?;
        let number = |name: &str, default: u64| {
            value
                .get(name)
                .map(registry::integer)
                .transpose()
                .map(|n| n.unwrap_or(default))
        };
        let size = |name, default| {
            usize::try_from(number(name, default)?).map_err(|_| Error::new("limit exceeds usize"))
        };
        let max_block = size("max_block", MIB as u64)?;
        let max_record = size("max_record", MIB as u64)?;
        ensure(
            (52..=MIB).contains(&max_block) && (52..=MIB).contains(&max_record),
            "invalid block/record limits",
        )?;
        let progress_records = number("progress_records", 1_000_000)?;
        ensure(progress_records > 0, "invalid progress interval")?;
        let mut expected = [None; 4];
        if let Some(values) = value.get("expected") {
            for (name, value) in registry::object(values)? {
                let index = ["files", "source_bytes", "nodes", "symbols"]
                    .iter()
                    .position(|candidate| candidate == name)
                    .ok_or_else(|| Error::new("unknown expected total"))?;
                expected[index] = Some(registry::integer(value)?);
            }
        }
        let allow_empty = value
            .get("allow_empty")
            .map(|v| {
                v.as_bool()
                    .ok_or_else(|| Error::new("allow_empty must be bool"))
            })
            .transpose()?
            .unwrap_or(false);
        Ok(Self {
            max_payload: number("max_payload", 32 * 1024_u64.pow(3))?,
            max_block,
            max_record,
            max_files: number("max_files", 13094)?,
            max_active_nodes: size("max_active_nodes", 2_000_000)?,
            allow_empty,
            expected,
            progress_records,
        })
    }
}

struct Stream<R> {
    source: R,
    hash: Sha256,
}
impl<R: Read> Stream<R> {
    fn exact(&mut self, target: &mut [u8]) -> Result<()> {
        ensure(target.len() <= MIB, "internal unbounded read")?;
        self.source.read_exact(target)?;
        self.hash.update(target);
        Ok(())
    }
    fn eof(&mut self) -> Result<()> {
        let mut byte = [0; 1];
        ensure(
            self.source.read(&mut byte)? == 0,
            "trailing trace bytes or duplicate footer",
        )
    }
}
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}
fn hex(value: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(value.len() * 2);
    for byte in value {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    text
}

fn verify(
    source: impl Read,
    events: &[Option<registry::Event>],
    settings: &Settings,
) -> Result<Value> {
    let mut source = Stream {
        source,
        hash: Sha256::new(),
    };
    let mut magic = [0; 8];
    source.exact(&mut magic)?;
    ensure(&magic == b"S07TRC01", "trace magic/version mismatch")?;
    let mut observations = Observations::new(events);
    let (mut blocks, mut records, mut payload_bytes) = (0_u64, 0_u64, 0_u64);
    let mut block_hashes = Sha256::new();
    let mut payload = Vec::new();
    let mut next_progress = settings.progress_records;
    loop {
        let mut tag = [0; 4];
        source.exact(&mut tag)?;
        if &tag == b"END1" {
            let mut footer = [0; 56];
            source.exact(&mut footer)?;
            ensure(
                [u64_at(&footer, 0), u64_at(&footer, 8), u64_at(&footer, 16)]
                    == [blocks, records, payload_bytes],
                "footer counters differ",
            )?;
            let digest = block_hashes.finalize();
            ensure(
                footer[24..] == digest[..],
                "footer block-hash digest differs",
            )?;
            ensure(observations.capture_ended, "missing capture_end")?;
            source.eof()?;
            let mut report = observations.report();
            let object = report
                .as_object_mut()
                .ok_or_else(|| Error::new("internal report is not object"))?;
            object.insert("blocks".into(), blocks.into());
            object.insert("records".into(), records.into());
            object.insert("payload_bytes".into(), payload_bytes.into());
            object.insert("stream_sha256".into(), hex(&source.hash.finalize()).into());
            object.insert("block_hashes_sha256".into(), hex(&digest).into());
            return Ok(report);
        }
        ensure(&tag == b"BLK1", "unknown block tag or missing footer")?;
        let mut header = [0; 48];
        source.exact(&mut header)?;
        let sequence = u64_at(&header, 0);
        let count = u64::from(u32_at(&header, 8));
        let length = usize::try_from(u32_at(&header, 12))
            .map_err(|_| Error::new("block length overflow"))?;
        ensure(sequence == blocks, "block sequence differs")?;
        ensure(
            length > 0
                && length <= settings.max_block
                && count > 0
                && count <= (length / 52) as u64,
            "invalid block length or record count",
        )?;
        let next_bytes = payload_bytes
            .checked_add(length as u64)
            .ok_or_else(|| Error::new("payload size overflow"))?;
        ensure(
            next_bytes <= settings.max_payload,
            "raw-payload limit exceeded",
        )?;
        payload.resize(length, 0);
        source.exact(&mut payload)?;
        let digest = Sha256::digest(&payload);
        ensure(header[16..] == digest[..], "block payload hash differs")?;
        let mut offset = 0;
        for _ in 0..count {
            ensure(offset + 4 <= length, "truncated record length")?;
            let size = usize::try_from(u32_at(&payload, offset))
                .map_err(|_| Error::new("record size overflow"))?;
            ensure(
                size >= 48 && size <= settings.max_record - 4 && size <= length - offset - 4,
                "invalid or truncated record length",
            )?;
            let body = &payload[offset + 4..offset + 4 + size];
            let record = Record {
                op: u16_at(body, 0),
                site: u16_at(body, 2),
                file: u32_at(body, 4),
                domain: u32_at(body, 8),
                reserved: u32_at(body, 12),
                values: [
                    u64_at(body, 16),
                    u64_at(body, 24),
                    u64_at(body, 32),
                    u64_at(body, 40),
                ],
                blob: &body[48..],
            };
            let spec = events
                .get(usize::from(record.op))
                .and_then(Option::as_ref)
                .ok_or_else(|| Error::new("unknown event operation"))?;
            observations.event(&record, spec, settings)?;
            offset += 4 + size;
        }
        ensure(offset == length, "uncounted records or trailing payload")?;
        blocks += 1;
        records += count;
        payload_bytes = next_bytes;
        block_hashes.update(digest);
        if records >= next_progress {
            eprintln!(
                "{}",
                json!({"verify_progress": {"records": records, "blocks": blocks,
                "files": observations.totals.files, "payload_bytes": payload_bytes}})
            );
            next_progress =
                (records / settings.progress_records + 1).saturating_mul(settings.progress_records);
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure(
        args.len() == 4,
        "usage: native-verifier CONFIG PROTOCOL_REGISTRY STATE_REGISTRY HOOKS_REGISTRY < trace.raw",
    )?;
    let settings = Settings::read(&json::read(&PathBuf::from(&args[0]))?)?;
    let events = registry::load(&args[1..].iter().map(PathBuf::from).collect::<Vec<_>>())?;
    let report = verify(
        BufReader::with_capacity(65536, io::stdin().lock()),
        &events,
        &settings,
    )?;
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer(&mut output, &report)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("invalid trace: {error}");
        std::process::exit(2);
    }
}
