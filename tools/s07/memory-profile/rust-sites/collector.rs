//! Requested allocation traffic in named source regions; staged diagnostics only.
use alloc_tracker::{Operation, Session, ThreadSpan};
use std::cell::Cell;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Site {/* SITE_VARIANTS */}

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Phase {
    Unscoped,
    Preload,
    Parse,
    Publish,
    Bind,
    Retire,
}

const PHASES: [&str; 6] = ["unscoped", "preload", "parse", "publish", "bind", "retire"];
struct Metadata {
    name: &'static str,
    source_file: &'static str,
    source_line: u32,
}
const SITES: &[Metadata] = &[
    /* SITE_METADATA */
];

struct Registry {
    session: Session,
    operations: Vec<Operation>,
    phases: Vec<Operation>,
    unions: Vec<Operation>,
    warmup: Operation,
}
static REGISTRY: OnceLock<Registry> = OnceLock::new();
thread_local! {
    static PHASE: Cell<usize> = const { Cell::new(0) };
    static DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Call before any measured region. Registration itself allocates.
pub fn initialize() {
    REGISTRY.get_or_init(|| {
        let session = Session::new();
        let mut operations = Vec::new();
        let mut phases = Vec::new();
        let mut unions = Vec::new();
        for phase in PHASES {
            phases.push(session.operation(format!("phase/{phase}")));
            unions.push(session.operation(format!("union/{phase}")));
            for metadata in SITES {
                operations.push(session.operation(format!("site/{phase}/{}", metadata.name)));
            }
        }
        let warmup = session.operation("internal/thread_warmup");
        // On this macOS std pin, each Mutex lazily allocates its pthread
        // backing on first lock. Span::drop locks the operation AFTER reading
        // its end counters; leaving that cold would charge its observer bytes
        // to a parent/union scope. Warm the public operation getters before
        // publishing the registry, without creating artificial observations.
        for operation in operations
            .iter()
            .chain(&phases)
            .chain(&unions)
            .chain([&warmup])
        {
            let _ = operation.total_bytes_allocated();
        }
        Registry {
            session,
            operations,
            phases,
            unions,
            warmup,
        }
    });
    initialize_thread();
}

/// Warm each worker's tracking counters before its baseline, outside phase scopes.
pub fn initialize_thread() {
    if let Some(registry) = REGISTRY.get() {
        let _warmup = registry.warmup.measure_thread();
        PHASE.with(|value| {
            let _ = value.get();
        });
        DEPTH.with(|value| {
            let _ = value.get();
        });
    }
}

pub struct PhaseGuard {
    span: Option<ThreadSpan>,
    previous: usize,
}
impl Drop for PhaseGuard {
    fn drop(&mut self) {
        drop(self.span.take());
        PHASE.set(self.previous);
    }
}

/// Phase scopes must enclose complete site guards on the same thread.
pub fn phase(phase: Phase) -> Option<PhaseGuard> {
    let registry = REGISTRY.get()?;
    assert_eq!(DEPTH.get(), 0, "phase changed inside an allocation site");
    let previous = PHASE.replace(phase as usize);
    Some(PhaseGuard {
        span: Some(registry.phases[phase as usize].measure_thread()),
        previous,
    })
}

pub struct SiteGuard {
    span: Option<ThreadSpan>,
    union: Option<ThreadSpan>,
}
impl Drop for SiteGuard {
    fn drop(&mut self) {
        drop(self.span.take());
        drop(self.union.take());
        DEPTH.set(DEPTH.get() - 1);
    }
}

/// Source rows are inclusive. The outermost-site union counts nested regions once.
pub fn site(site: Site) -> Option<SiteGuard> {
    let registry = REGISTRY.get()?;
    let phase = PHASE.get();
    let depth = DEPTH.get();
    DEPTH.set(depth + 1);
    let union = (depth == 0).then(|| registry.unions[phase].measure_thread());
    let span = registry.operations[phase * SITES.len() + site as usize].measure_thread();
    Some(SiteGuard {
        span: Some(span),
        union,
    })
}

#[derive(Debug)]
pub struct Row {
    pub name: &'static str,
    pub phase: &'static str,
    pub source_file: &'static str,
    pub source_line: u32,
    pub scope_kind: &'static str,
    pub requested_bytes: u64,
    pub allocation_calls: u64,
    pub observations: u64,
}

/// All workers and guards must be quiescent. This function allocates its report.
pub fn report() -> Vec<Row> {
    let Some(registry) = REGISTRY.get() else {
        return Vec::new();
    };
    let report = registry.session.to_report();
    let mut result = Vec::new();
    for (key, value) in report.operations() {
        if key.starts_with("internal/") {
            continue;
        }
        let mut parts = key.splitn(3, '/');
        let family = parts.next().expect("registered family");
        let phase_name = parts.next().expect("registered phase");
        let phase = PHASES
            .iter()
            .copied()
            .find(|name| *name == phase_name)
            .expect("registered phase");
        let (name, source_file, source_line, scope_kind) = match family {
            "site" => {
                let name = parts.next().expect("registered source site");
                let metadata = SITES
                    .iter()
                    .find(|site| site.name == name)
                    .expect("registered metadata");
                (
                    metadata.name,
                    metadata.source_file,
                    metadata.source_line,
                    "inclusive_site",
                )
            }
            "phase" => ("phase", "adapter phase guard", 0, "phase"),
            "union" => (
                "selected source regions",
                "dynamic outermost source-site union",
                0,
                "selected_union",
            ),
            _ => unreachable!("registered operation family"),
        };
        result.push(Row {
            name,
            phase,
            source_file,
            source_line,
            scope_kind,
            requested_bytes: value.total_bytes_allocated(),
            allocation_calls: value.total_allocations_count(),
            observations: value.total_iterations(),
        });
    }
    result.sort_by_key(|row| (row.phase, row.scope_kind, row.name));
    result
}
