use crate::Error;
use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};
use ts_arena::Counters;
use ts_ast::{BoundFile, SourceFileParseOptions};
use ts_core::ScriptKind;
use ts_jsstring::{JsString, SourceText};
/// An escaped file retains its complete parsed and bound owner. Graph edges in
/// a Program are IDs; they never retain another Program or create Arc cycles.
#[derive(Debug)]
pub struct ProgramFile {
    pub(crate) bound: BoundFile,
}
impl ProgramFile {
    pub fn bound(&self) -> &BoundFile {
        &self.bound
    }
    pub fn source(&self) -> ts_ast::NodeId {
        self.bound.source()
    }
}
/// Explicit reuse cache. Weak entries do not keep files alive after the final
/// snapshot/response drops. Comparison includes source bytes and parse context.
/// One cache is exclusively borrowed during a load; it has no process global map.
#[derive(Default)]
pub struct FileCache {
    files: BTreeMap<JsString, Vec<Weak<ProgramFile>>>,
}
impl FileCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn prune(&mut self) {
        self.files.retain(|_, entries| {
            entries.retain(|entry| entry.strong_count() != 0);
            !entries.is_empty()
        });
    }
    pub(crate) fn acquire(
        &mut self,
        source: SourceText,
        kind: ScriptKind,
        options: SourceFileParseOptions,
        counters: &Counters,
    ) -> Result<Arc<ProgramFile>, Error> {
        let entries = self.files.entry(options.path.clone()).or_default();
        entries.retain(|entry| entry.strong_count() != 0);
        for entry in entries.iter() {
            if let Some(file) = entry.upgrade() {
                let state = file.bound.view().source_file()?;
                if state.script_kind == kind
                    && state.parse_options() == &options
                    && state.text().as_bytes() == source.as_bytes()
                {
                    return Ok(file);
                }
            }
        }
        let parsed = ts_parser::parse_source_file_with_counters(source, kind, options, counters);
        let root = parsed.root();
        let ast = parsed.publish_unbound();
        let bound = ts_binder::bind_source_file(&ast, root)?;
        let file = Arc::new(ProgramFile { bound });
        entries.push(Arc::downgrade(&file));
        Ok(file)
    }
}
