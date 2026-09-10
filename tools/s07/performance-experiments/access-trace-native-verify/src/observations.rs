use crate::registry::{node_id, Event};
use crate::{ensure, Error, Result, Settings};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const MASK: u64 = u32::MAX as u64;
const UNMAPPED: usize = 65536;

pub struct Record<'a> {
    pub op: u16,
    pub site: u16,
    pub file: u32,
    pub domain: u32,
    pub reserved: u32,
    pub values: [u64; 4],
    pub blob: &'a [u8],
}

#[derive(Default, Serialize)]
pub struct Categories {
    lookup: u64,
    read: u64,
    write: u64,
    other: u64,
}
impl Categories {
    fn increment(&mut self, category: usize) {
        match category {
            0 => self.lookup += 1,
            1 => self.read += 1,
            2 => self.write += 1,
            _ => self.other += 1,
        }
    }
}

#[derive(Default, Serialize)]
struct FileReport {
    file: u32,
    input_index: u64,
    source_bytes: u64,
    binder_operations: Categories,
    state_operations: u64,
    unmapped_node_operations: u64,
    root: u64,
    parsed_nodes: u64,
    nodes: u64,
    symbols: u64,
    parse_diagnostics: u64,
    bind_diagnostics: u64,
    bound_in_place: bool,
    state_node_headers: usize,
}

#[derive(Default, Serialize)]
pub struct Totals {
    pub files: u64,
    pub source_bytes: u64,
    pub nodes: u64,
    pub symbols: u64,
}
impl Totals {
    fn values(&self) -> [u64; 4] {
        [self.files, self.source_bytes, self.nodes, self.symbols]
    }
}

struct Header {
    kind: u16,
    flags: u32,
    shape: u16,
}
struct State {
    node_arena: u64,
    aux_arena: u64,
    nodes: Option<u64>,
    aux: Option<u64>,
    aux_headers: u64,
    blob_bytes: u64,
    ended: bool,
}
#[derive(PartialEq, Eq)]
struct BlobIdentity {
    op: u16,
    site: u16,
    a: u64,
    b: u64,
    length: u64,
}

pub struct Observations {
    pub totals: Totals,
    by_op: Vec<u64>,
    by_site: Vec<u64>,
    by_domain: [u64; 3],
    by_file: Vec<u64>,
    by_op_site: Vec<Vec<u64>>,
    by_op_shape: Vec<Vec<u64>>,
    binder: Categories,
    headers: Vec<Header>,
    state: Option<State>,
    blob: Option<(BlobIdentity, u64)>,
    active: Option<FileReport>,
    files: Vec<FileReport>,
    phase: u16,
    pub capture_ended: bool,
}

fn counts(values: &[u64]) -> BTreeMap<String, u64> {
    values
        .iter()
        .enumerate()
        .filter(|(_, count)| **count != 0)
        .map(|(key, count)| (key.to_string(), *count))
        .collect()
}

impl Observations {
    pub fn new(events: &[Option<Event>]) -> Self {
        Self {
            totals: Totals::default(),
            by_op: vec![0; 150],
            by_site: vec![0; 65536],
            by_domain: [0; 3],
            by_file: vec![0],
            by_op_site: events
                .iter()
                .map(|event| {
                    event.as_ref().map_or_else(Vec::new, |event| {
                        vec![0; event.sites.as_ref().map_or(65536, Vec::len)]
                    })
                })
                .collect(),
            by_op_shape: (0..150)
                .map(|op| {
                    if matches!(op, 100 | 101 | 102 | 103 | 107) {
                        vec![0; UNMAPPED + 1]
                    } else {
                        Vec::new()
                    }
                })
                .collect(),
            binder: Categories::default(),
            headers: Vec::new(),
            state: None,
            blob: None,
            active: None,
            files: Vec::new(),
            phase: 0,
            capture_ended: false,
        }
    }

    pub fn event(&mut self, record: &Record<'_>, spec: &Event, settings: &Settings) -> Result<()> {
        ensure(!self.capture_ended, "event after capture_end")?;
        ensure(
            record.domain == spec.domain,
            "event domain differs from registry",
        )?;
        ensure(
            spec.sites
                .as_ref()
                .is_none_or(|sites| sites.get(usize::from(record.site)) == Some(&true)),
            "unknown event site",
        )?;
        ensure(record.reserved == 0, "record reserved field nonzero")?;
        ensure(spec.blob || record.blob.is_empty(), "unexpected blob")?;
        for (index, field) in &spec.checks {
            field.check(record.values[*index])?;
        }
        for (index, descriptor) in &spec.constraints {
            ensure(
                record.values[*index] < (record.values[*descriptor] & MASK),
                "successful slice index outside descriptor",
            )?;
        }
        if record.domain == 0 {
            self.driver(record, settings)?;
        } else {
            let active = self
                .active
                .as_mut()
                .ok_or_else(|| Error::new("event outside active file"))?;
            ensure(
                active.file == record.file,
                "event file differs from active file",
            )?;
            ensure(
                self.phase == if record.domain == 1 { 4 } else { 6 },
                "event outside declared phase",
            )?;
            if record.domain == 2 {
                self.binder.increment(spec.category);
                active.binder_operations.increment(spec.category);
                self.node_operation(record)?;
            } else {
                active.state_operations += 1;
                self.state_event(record, spec.blob)?;
                if record.op == 14 {
                    let [node, kind, parent, flags] = record.values;
                    node_id(node)?;
                    if parent != 0 {
                        node_id(parent)?;
                    }
                    let flags =
                        u32::try_from(flags).map_err(|_| Error::new("state flags exceed u32"))?;
                    // The protocol preserves a signed i16 after sign extension to i64.
                    ensure(
                        kind == (kind as i16 as i64) as u64,
                        "state kind not sign-extended i16",
                    )?;
                    ensure(
                        self.headers.len() < settings.max_active_nodes,
                        "active-node limit exceeded",
                    )?;
                    self.headers.push(Header {
                        kind: kind as u16,
                        flags,
                        shape: record.site,
                    });
                }
            }
        }
        self.by_op[usize::from(record.op)] += 1;
        self.by_site[usize::from(record.site)] += 1;
        self.by_op_site[usize::from(record.op)][usize::from(record.site)] += 1;
        self.by_domain
            [usize::try_from(record.domain).map_err(|_| Error::new("domain index overflow"))?] += 1;
        self.by_file
            [usize::try_from(record.file).map_err(|_| Error::new("file index overflow"))?] += 1;
        Ok(())
    }

    fn state_event(&mut self, record: &Record<'_>, is_blob: bool) -> Result<()> {
        let [a, b, c, d] = record.values;
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| Error::new("state lacks file"))?;
        if record.op == 10 {
            ensure(self.state.is_none(), "duplicate state owner")?;
            ensure(
                a > 0 && a <= MASK && b > 0 && b <= MASK && a != b,
                "invalid core arena identities",
            )?;
            ensure(active.root >> 32 == a, "root differs from core owner")?;
            ensure(d == active.source_bytes, "source bytes differ from driver")?;
            self.state = Some(State {
                node_arena: a,
                aux_arena: b,
                nodes: None,
                aux: None,
                aux_headers: 0,
                blob_bytes: 0,
                ended: false,
            });
        }
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| Error::new("state lacks owner marker"))?;
        ensure(!state.ended, "record after state end")?;
        if is_blob {
            ensure(record.blob.len() <= 65536, "blob chunk exceeds 64KiB")?;
            let identity = BlobIdentity {
                op: record.op,
                site: record.site,
                a,
                b,
                length: d,
            };
            match &self.blob {
                None => ensure(c == 0, "blob begins at nonzero offset")?,
                Some((previous, offset)) => ensure(
                    identity == *previous && c == *offset,
                    "blob chunk identity/offset differs",
                )?,
            }
            let length =
                u64::try_from(record.blob.len()).map_err(|_| Error::new("blob length overflow"))?;
            let end = c
                .checked_add(length)
                .ok_or_else(|| Error::new("blob extent overflow"))?;
            ensure(end <= d && (length != 0 || d == 0), "invalid blob extent")?;
            state.blob_bytes = state
                .blob_bytes
                .checked_add(length)
                .ok_or_else(|| Error::new("blob total overflow"))?;
            self.blob = if end < d { Some((identity, end)) } else { None };
        } else {
            ensure(self.blob.is_none(), "interrupted or truncated blob")?;
        }
        match record.op {
            11 => {
                let (count, arena) = match record.site {
                    1 => (&mut state.nodes, state.node_arena),
                    2 => (&mut state.aux, state.aux_arena),
                    _ => return Err(Error::new("unknown core arena layout")),
                };
                ensure(
                    count.is_none() && a == arena && b <= c,
                    "duplicate or inconsistent core layout",
                )?;
                *count = Some(b);
            }
            14 => {
                let next_slot = u64::try_from(self.headers.len())
                    .map_err(|_| Error::new("header count overflow"))?
                    + 1;
                ensure(
                    a >> 32 == state.node_arena
                        && a & MASK == next_slot
                        && state.nodes.is_some_and(|count| next_slot <= count),
                    "node physical slot order differs",
                )?;
            }
            24 => {
                ensure(
                    a >> 32 == state.aux_arena
                        && a & MASK == state.aux_headers + 1
                        && state.aux.is_some_and(|count| state.aux_headers < count),
                    "auxiliary physical slot order differs",
                )?;
                state.aux_headers += 1;
            }
            79 => {
                let headers = u64::try_from(self.headers.len())
                    .map_err(|_| Error::new("header count overflow"))?;
                ensure(
                    state.nodes == Some(a) && a == headers && (active.root & MASK) <= headers,
                    "state core/header/root count differs",
                )?;
                ensure(
                    state.aux == Some(b) && b == state.aux_headers,
                    "state auxiliary count differs",
                )?;
                ensure(d == state.blob_bytes, "state byte count differs")?;
                state.ended = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn node_operation(&mut self, record: &Record<'_>) -> Result<()> {
        if !matches!(record.op, 100 | 101 | 102 | 103 | 107) {
            return Ok(());
        }
        let node = record.values[0];
        let header = if self
            .state
            .as_ref()
            .is_some_and(|state| node >> 32 == state.node_arena)
        {
            usize::try_from((node & MASK).wrapping_sub(1))
                .ok()
                .and_then(|index| self.headers.get_mut(index))
        } else {
            None
        };
        let counts = &mut self.by_op_shape[usize::from(record.op)];
        let Some(header) = header else {
            counts[UNMAPPED] += 1;
            self.active
                .as_mut()
                .ok_or_else(|| Error::new("node operation lacks file"))?
                .unmapped_node_operations += 1;
            return Ok(());
        };
        counts[usize::from(header.shape)] += 1;
        match record.op {
            101 => ensure(
                record.values[1] == u64::from(header.kind),
                "named kind differs from initial header",
            )?,
            102 => ensure(
                record.values[1] == u64::from(header.flags),
                "named flags differ from tracked flags",
            )?,
            103 => {
                header.flags = u32::try_from(record.values[1])
                    .map_err(|_| Error::new("new flags exceed u32"))?
            }
            _ => {}
        }
        Ok(())
    }

    fn driver(&mut self, record: &Record<'_>, settings: &Settings) -> Result<()> {
        let [a, b, c, d] = record.values;
        if record.op == 9 {
            ensure(
                self.active.is_none() && record.file == 0,
                "capture end while active/nonzero file",
            )?;
            ensure(
                settings.allow_empty || self.totals.files > 0,
                "empty workload",
            )?;
            ensure(
                record.values == self.totals.values(),
                "capture totals differ",
            )?;
            for (index, expected) in settings.expected.iter().enumerate() {
                ensure(
                    expected.is_none_or(|expected| self.totals.values()[index] == expected),
                    "expected totals differ",
                )?;
            }
            self.capture_ended = true;
            return Ok(());
        }
        if record.op == 1 {
            ensure(
                self.active.is_none()
                    && u64::from(record.file) == self.totals.files + 1
                    && a.checked_add(1) == Some(u64::from(record.file)),
                "file order/identity changed",
            )?;
            ensure(
                u64::from(record.file) <= settings.max_files,
                "file-count limit exceeded",
            )?;
            ensure(c == 0 && d == 0, "file begin unused fields nonzero")?;
            self.active = Some(FileReport {
                file: record.file,
                input_index: a,
                source_bytes: b,
                ..FileReport::default()
            });
            self.by_file.push(0);
            self.phase = 1;
            return Ok(());
        }
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| Error::new("driver outside active file"))?;
        ensure(
            active.file == record.file && record.op == self.phase + 1,
            "driver phase order/identity changed",
        )?;
        match record.op {
            2 | 4 | 5 | 6 => {
                ensure(record.values == [0; 4], "phase marker fields nonzero")?;
                if record.op == 5 {
                    ensure(
                        self.state.as_ref().is_some_and(|state| state.ended),
                        "incomplete state export",
                    )?;
                }
            }
            3 => {
                node_id(a)?;
                ensure(c == 0 && d == 0, "parse end unused fields nonzero")?;
                active.root = a;
                active.parsed_nodes = b;
            }
            7 => {
                active.nodes = a;
                active.symbols = b;
                active.parse_diagnostics = c;
                active.bind_diagnostics = d;
            }
            8 => {
                ensure(a <= 1 && [b, c, d] == [0; 3], "invalid file end fields")?;
                active.bound_in_place = a == 1;
                active.state_node_headers = self.headers.len();
                self.totals.files += 1;
                self.totals.source_bytes = self
                    .totals
                    .source_bytes
                    .checked_add(active.source_bytes)
                    .ok_or_else(|| Error::new("source total overflow"))?;
                self.totals.nodes = self
                    .totals
                    .nodes
                    .checked_add(active.nodes)
                    .ok_or_else(|| Error::new("node total overflow"))?;
                self.totals.symbols = self
                    .totals
                    .symbols
                    .checked_add(active.symbols)
                    .ok_or_else(|| Error::new("symbol total overflow"))?;
                self.files.push(
                    self.active
                        .take()
                        .ok_or_else(|| Error::new("missing finished file"))?,
                );
                self.headers.clear();
                self.state = None;
            }
            _ => return Err(Error::new("unknown driver event")),
        }
        self.phase = record.op;
        Ok(())
    }

    pub fn report(self) -> Value {
        let op_sites: BTreeMap<_, _> = self
            .by_op_site
            .iter()
            .enumerate()
            .filter(|(_, values)| values.iter().any(|n| *n != 0))
            .map(|(op, values)| (op.to_string(), counts(values)))
            .collect();
        let op_shapes: BTreeMap<_, _> = self
            .by_op_shape
            .iter()
            .enumerate()
            .filter(|(_, values)| values.iter().any(|n| *n != 0))
            .map(|(op, values)| {
                let mut counts = counts(values);
                if let Some(value) = counts.remove(&UNMAPPED.to_string()) {
                    counts.insert("unmapped".to_owned(), value);
                }
                (op.to_string(), counts)
            })
            .collect();
        json!({"version": 1, "format": "S07TRC01", "complete": true,
            "totals": self.totals, "by_op": counts(&self.by_op), "by_site": counts(&self.by_site),
            "by_op_site": op_sites, "by_domain": counts(&self.by_domain), "by_file": counts(&self.by_file),
            "by_op_shape": op_shapes, "binder_operations": self.binder, "files": self.files,
            "full_semantic_replay": false, "unobserved_operation_count": null,
            "scope": "Framing, integrity, phase/physical-header/blob-count checks and named kind/flag checks for mapped initial headers; no hardware, list-value replay or complete payload coverage"})
    }
}
