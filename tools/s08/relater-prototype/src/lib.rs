//! The reference-based relater prototype required by the S08 plan (§6.3) and
//! ADR 0008: the same relation algorithm as `relater.go`'s core, over a type
//! graph of stable heap cells linked by references instead of arena ids, with
//! lazily resolved members that allocate during a relation.
//!
//! This crate is the P1 feasibility proof. It fixes the safe construction and
//! mutation API the full alternative would use and shows, on the frozen
//! recursive fixtures, that the algorithm's recursion, assumption stack and
//! relation cache work without `unsafe`, lifetime transmutes, leaked storage or
//! unchecked self-references. P7 measures it against the production relater;
//! nothing here is a production path.
//!
//! # Construction and mutation scopes
//!
//! - [`Graph`] owns every [`TypeCell`] strongly (`Rc`). Every edge between types
//!   (`members`, `target`) is a `Weak`, so the only strong references form a
//!   tree from the graph; dropping the graph drops every cell (no cycles).
//! - Lazy state lives in `OnceCell`/`RefCell` fields of the cell. A resolver runs
//!   at most once, allocates into the graph while an `&TypeCell` borrow of the
//!   same graph is live (`Rc` cells never move), and publishes the result before
//!   anyone can read it.
//! - Callers hold `Rc<TypeCell>` handles. A handle keeps its cell alive but not
//!   the cells it points to: after the graph drops, following an edge fails
//!   explicitly (`Error::Released`). The production design keeps the owner alive
//!   from every escaped result for exactly this reason (ADR 0007).
//! - A panic inside a resolver leaves the `OnceCell` empty and the graph
//!   consistent; the algorithm never holds a `RefMut` across a call that can run
//!   user code, so unwinding cannot poison shared state.
#![forbid(unsafe_code)]

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

/// `Ternary`: `x & y` picks the lesser in the order False < Unknown < Maybe < True.
pub type Ternary = i8;
pub const FALSE: Ternary = 0;
pub const UNKNOWN: Ternary = 1;
pub const MAYBE: Ternary = 3;
pub const TRUE: Ternary = -1;

/// The `TypeFlags` bits the prototype's fixtures use.
pub mod flags {
    pub const STRING: u32 = 1 << 2;
    pub const NUMBER: u32 = 1 << 3;
    pub const OBJECT: u32 = 1 << 19;
    pub const PRIMITIVE: u32 = STRING | NUMBER;
    /// `TypeFlagsSingleton`: identical flags mean identical types.
    pub const SINGLETON: u32 = STRING | NUMBER;
}

/// `RelationComparisonResult`.
pub mod relation_result {
    pub const SUCCEEDED: u32 = 1 << 0;
    pub const FAILED: u32 = 1 << 1;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Identity,
    Assignable,
    Subtype,
    StrictSubtype,
    Comparable,
}

pub const MODES: [Mode; 5] = [
    Mode::Identity,
    Mode::Assignable,
    Mode::Subtype,
    Mode::StrictSubtype,
    Mode::Comparable,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// An edge points at a cell whose graph has been released.
    Released,
    /// A member resolver referenced a type that was never declared.
    UndeclaredMember(&'static str),
}

/// A resolved property of an object type.
#[derive(Clone, Debug)]
pub struct Member {
    pub name: &'static str,
    pub optional: bool,
    r#type: Weak<TypeCell>,
}

impl Member {
    pub fn r#type(&self) -> Result<Rc<TypeCell>, Error> {
        self.r#type.upgrade().ok_or(Error::Released)
    }
}

/// How an object type's members are produced when first needed.
type Resolver = Box<dyn Fn(&Graph, &TypeCell) -> Result<Vec<Member>, Error>>;

/// One type: identity, flags and lazily resolved structure.
pub struct TypeCell {
    id: u32,
    flags: u32,
    name: &'static str,
    /// Set once by the first `members` call; empty for primitives.
    members: OnceCell<Vec<Member>>,
    /// Taken by the first `members` call.
    resolver: RefCell<Option<Resolver>>,
    /// Counts resolutions, to prove laziness and single execution.
    resolutions: Cell<u32>,
}

impl std::fmt::Debug for TypeCell {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("TypeCell")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("flags", &self.flags)
            .field("resolved", &self.members.get().is_some())
            .finish_non_exhaustive()
    }
}

impl TypeCell {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn resolutions(&self) -> u32 {
        self.resolutions.get()
    }

    /// The object's members, resolved on first use. Resolution may allocate
    /// into `graph`; the cell itself never moves, so the borrow stays valid.
    pub fn members(&self, graph: &Graph) -> Result<&[Member], Error> {
        if let Some(members) = self.members.get() {
            return Ok(members);
        }
        let resolver = self.resolver.borrow_mut().take();
        let members = match resolver {
            Some(resolver) => resolver(graph, self)?,
            None => Vec::new(),
        };
        self.resolutions.set(self.resolutions.get() + 1);
        // A resolver that re-entered `members` would have initialized the cell
        // already; upstream treats that as a cycle, so the later result loses.
        Ok(self.members.get_or_init(|| members))
    }

    pub fn member(&self, graph: &Graph, name: &str) -> Result<Option<&Member>, Error> {
        Ok(self
            .members(graph)?
            .iter()
            .find(|member| member.name == name))
    }
}

/// The owner of every type cell. Allocation is append-only through a
/// `RefCell<Vec<Rc<_>>>`; readers hold `Rc` clones, never a borrow of the vector.
#[derive(Default)]
pub struct Graph {
    types: RefCell<Vec<Rc<TypeCell>>>,
    next_id: Cell<u32>,
    /// Records created by resolvers during relations (property cells upstream).
    lazy_records: RefCell<Vec<Rc<Member>>>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.types.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Records allocated by member resolvers so far.
    pub fn lazy_records(&self) -> usize {
        self.lazy_records.borrow().len()
    }

    fn allocate(&self, flags: u32, name: &'static str, resolver: Option<Resolver>) -> Rc<TypeCell> {
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        let cell = Rc::new(TypeCell {
            id,
            flags,
            name,
            members: OnceCell::new(),
            resolver: RefCell::new(resolver),
            resolutions: Cell::new(0),
        });
        self.types.borrow_mut().push(cell.clone());
        cell
    }

    pub fn primitive(&self, flags: u32, name: &'static str) -> Rc<TypeCell> {
        self.allocate(flags, name, None)
    }

    /// An object type whose members are declared as names and weak links and
    /// materialized on first use. `declared` may refer to cells created later,
    /// including the object itself, which is how the recursive fixtures are
    /// built without a strong cycle.
    pub fn object(
        &self,
        name: &'static str,
        declared: Vec<(&'static str, bool, Weak<TypeCell>)>,
    ) -> Rc<TypeCell> {
        let resolver: Resolver = Box::new(move |graph, _cell| {
            let mut members = Vec::with_capacity(declared.len());
            for (name, optional, r#type) in &declared {
                if r#type.upgrade().is_none() {
                    return Err(Error::UndeclaredMember(name));
                }
                let member = Member {
                    name,
                    optional: *optional,
                    r#type: r#type.clone(),
                };
                // Upstream allocates a property symbol per member here.
                graph
                    .lazy_records
                    .borrow_mut()
                    .push(Rc::new(member.clone()));
                members.push(member);
            }
            Ok(members)
        });
        self.allocate(flags::OBJECT, name, Some(resolver))
    }

    /// An object type whose resolver panics, for the failure-behavior test.
    pub fn poisoned_object(&self, name: &'static str) -> Rc<TypeCell> {
        let resolver: Resolver = Box::new(|_, _| panic!("injected resolver failure"));
        self.allocate(flags::OBJECT, name, Some(resolver))
    }

    /// A weak link to a cell that will be created by a later call, resolved
    /// through the graph by id once both exist.
    pub fn link(&self, cell: &Rc<TypeCell>) -> Weak<TypeCell> {
        Rc::downgrade(cell)
    }
}

/// One relation's cache (`Relation`): keyed by the ordered source/target ids.
#[derive(Default)]
pub struct Relation {
    results: RefCell<HashMap<(u32, u32), u32>>,
}

impl Relation {
    pub fn entries(&self) -> usize {
        self.results.borrow().len()
    }

    /// The cache's result flags, sorted, for comparison with the Go observer.
    pub fn result_flags(&self) -> Vec<u32> {
        let mut flags: Vec<u32> = self.results.borrow().values().copied().collect();
        flags.sort_unstable();
        flags
    }

    fn get(&self, key: (u32, u32)) -> u32 {
        self.results.borrow().get(&key).copied().unwrap_or(0)
    }

    fn set(&self, key: (u32, u32), value: u32) {
        self.results.borrow_mut().insert(key, value);
    }
}

/// The checker state the relater needs: the graph and one cache per relation.
#[derive(Default)]
pub struct Checker {
    pub graph: Graph,
    relations: [Relation; 5],
    diagnostics: RefCell<Vec<String>>,
}

impl Checker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn relation(&self, mode: Mode) -> &Relation {
        &self.relations[MODES.iter().position(|m| *m == mode).expect("mode")]
    }

    pub fn diagnostics(&self) -> Vec<String> {
        self.diagnostics.borrow().clone()
    }

    /// `checkTypeRelatedToEx`: relates and, when asked, reports one diagnostic
    /// for a failure. Returns the top-level ternary the Go observer records and
    /// whether the types are related.
    pub fn check_type_related_to(
        &self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
        mode: Mode,
        report_errors: bool,
    ) -> Result<(Ternary, bool), Error> {
        let relation = self.relation(mode);
        let mut relater = Relater {
            checker: self,
            mode,
            relation,
            maybe_keys: Vec::new(),
            maybe_set: HashSet::new(),
            source_stack: Vec::new(),
            target_stack: Vec::new(),
            expanding_source: false,
            expanding_target: false,
            relation_count: (16_000_000 - relation.entries() as i64) / 8,
            overflow: false,
            error_chain: Vec::new(),
        };
        let result = relater.is_related_to(source, target, report_errors)?;
        if !relater.error_chain.is_empty() {
            self.diagnostics
                .borrow_mut()
                .push(relater.error_chain.join(" | "));
        }
        Ok((result, result != FALSE))
    }
}

/// `getRelationKey` for two non-generic types: identity keys are order-free.
fn relation_key(source: &TypeCell, target: &TypeCell, identity: bool) -> (u32, u32) {
    let (mut s, mut t) = (source.id, target.id);
    if identity && s > t {
        std::mem::swap(&mut s, &mut t);
    }
    (s, t)
}

struct Relater<'c> {
    checker: &'c Checker,
    mode: Mode,
    relation: &'c Relation,
    maybe_keys: Vec<(u32, u32)>,
    maybe_set: HashSet<(u32, u32)>,
    source_stack: Vec<Rc<TypeCell>>,
    target_stack: Vec<Rc<TypeCell>>,
    expanding_source: bool,
    expanding_target: bool,
    relation_count: i64,
    overflow: bool,
    error_chain: Vec<String>,
}

impl Relater<'_> {
    fn report(&mut self, report_errors: bool, message: String) {
        if report_errors {
            self.error_chain.push(message);
        }
    }

    /// `isSimpleTypeRelatedTo` for the fixtures' primitives: a primitive is
    /// related to itself under every relation.
    fn is_simple_type_related_to(source: &TypeCell, target: &TypeCell) -> bool {
        source.flags & flags::PRIMITIVE != 0 && source.flags == target.flags
    }

    /// `isRelatedToEx`.
    fn is_related_to(
        &mut self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
        report_errors: bool,
    ) -> Result<Ternary, Error> {
        if Rc::ptr_eq(source, target) {
            return Ok(TRUE);
        }
        if source.flags & flags::OBJECT != 0 && target.flags & flags::PRIMITIVE != 0 {
            self.report(
                report_errors,
                format!(
                    "Type '{}' is not assignable to type '{}'.",
                    source.name, target.name
                ),
            );
            return Ok(FALSE);
        }
        if self.mode == Mode::Identity {
            if source.flags != target.flags {
                return Ok(FALSE);
            }
            if source.flags & flags::SINGLETON != 0 {
                return Ok(TRUE);
            }
            return self.recursive_type_related_to(source, target, false);
        }
        if self.mode == Mode::Comparable && Self::is_simple_type_related_to(target, source)
            || Self::is_simple_type_related_to(source, target)
        {
            return Ok(TRUE);
        }
        if source.flags & flags::OBJECT != 0 && target.flags & flags::OBJECT != 0 {
            let result = self.recursive_type_related_to(source, target, report_errors)?;
            if result != FALSE {
                return Ok(result);
            }
        }
        self.report(
            report_errors,
            format!(
                "Type '{}' is not assignable to type '{}'.",
                source.name, target.name
            ),
        );
        Ok(FALSE)
    }

    /// `recursiveTypeRelatedTo`: cache, assumptions, depth limits, then structure.
    fn recursive_type_related_to(
        &mut self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
        report_errors: bool,
    ) -> Result<Ternary, Error> {
        if self.overflow {
            return Ok(FALSE);
        }
        let key = relation_key(source, target, self.mode == Mode::Identity);
        let entry = self.relation.get(key);
        if entry != 0 {
            let failed_without_overflow = entry & relation_result::FAILED != 0;
            if report_errors && failed_without_overflow {
                // Elaborating errors: compare again to produce the message.
            } else {
                return Ok(if entry & relation_result::SUCCEEDED != 0 {
                    TRUE
                } else {
                    FALSE
                });
            }
        }
        if self.relation_count <= 0 {
            self.overflow = true;
            return Ok(FALSE);
        }
        // If source and target are already being compared, consider them related with assumptions.
        if self.maybe_set.contains(&key) {
            return Ok(MAYBE);
        }
        if self.source_stack.len() == 100 || self.target_stack.len() == 100 {
            return Ok(MAYBE);
        }
        let maybe_start = self.maybe_keys.len();
        self.maybe_keys.push(key);
        self.maybe_set.insert(key);
        let (save_source, save_target) = (self.expanding_source, self.expanding_target);
        self.source_stack.push(source.clone());
        if !self.expanding_source && is_deeply_nested_type(source, &self.source_stack, 3) {
            self.expanding_source = true;
        }
        self.target_stack.push(target.clone());
        if !self.expanding_target && is_deeply_nested_type(target, &self.target_stack, 3) {
            self.expanding_target = true;
        }
        let result = if self.expanding_source && self.expanding_target {
            MAYBE
        } else {
            self.structured_type_related_to(source, target, report_errors)?
        };
        self.source_stack.pop();
        self.target_stack.pop();
        self.expanding_source = save_source;
        self.expanding_target = save_target;
        if result == FALSE {
            // A false result goes straight into the cache: false under assumptions is false without them.
            self.relation.set(key, relation_result::FAILED);
            self.relation_count -= 1;
            self.reset_maybe_stack(maybe_start, false);
        } else if result == TRUE || (self.source_stack.is_empty() && self.target_stack.is_empty()) {
            // Definite or depth-zero Maybe results record every assumption as
            // succeeded; Unknown results are never recorded. Other Maybe results
            // stay on the stack so depth zero can record them.
            self.reset_maybe_stack(maybe_start, result == TRUE || result == MAYBE);
        }
        Ok(result)
    }

    /// `resetMaybeStack`.
    fn reset_maybe_stack(&mut self, maybe_start: usize, mark_all_as_succeeded: bool) {
        for key in self.maybe_keys.drain(maybe_start..) {
            self.maybe_set.remove(&key);
            if mark_all_as_succeeded {
                self.relation.set(key, relation_result::SUCCEEDED);
                self.relation_count -= 1;
            }
        }
    }

    /// `structuredTypeRelatedTo` for object types: properties only.
    fn structured_type_related_to(
        &mut self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
        report_errors: bool,
    ) -> Result<Ternary, Error> {
        if self.mode == Mode::Identity {
            return self.properties_identical_to(source, target);
        }
        self.properties_related_to(source, target, report_errors)
    }

    /// `propertiesRelatedTo`: every target property must exist in the source
    /// (optional ones excepted) with a related type.
    fn properties_related_to(
        &mut self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
        report_errors: bool,
    ) -> Result<Ternary, Error> {
        // The checker outlives the relater; copying the reference out keeps the
        // graph borrow independent of `&mut self`.
        let graph: &Graph = &self.checker.graph;
        let target_members: Vec<Member> = target.members(graph)?.to_vec();
        let require_optional = matches!(self.mode, Mode::Subtype | Mode::StrictSubtype);
        for member in &target_members {
            if source.member(graph, member.name)?.is_none()
                && (require_optional || !member.optional)
            {
                self.report(
                    report_errors,
                    format!(
                        "Property '{}' is missing in type '{}' but required in type '{}'.",
                        member.name, source.name, target.name
                    ),
                );
                return Ok(FALSE);
            }
        }
        let mut result = TRUE;
        for member in &target_members {
            let Some(source_member) = source.member(graph, member.name)? else {
                continue;
            };
            let source_type = source_member.r#type()?;
            let target_type = member.r#type()?;
            let related = self.is_related_to(&source_type, &target_type, report_errors)?;
            if related == FALSE {
                self.report(
                    report_errors,
                    format!("Types of property '{}' are incompatible.", member.name),
                );
                return Ok(FALSE);
            }
            result &= related;
        }
        Ok(result)
    }

    /// `propertiesIdenticalTo`.
    fn properties_identical_to(
        &mut self,
        source: &Rc<TypeCell>,
        target: &Rc<TypeCell>,
    ) -> Result<Ternary, Error> {
        // The checker outlives the relater; copying the reference out keeps the
        // graph borrow independent of `&mut self`.
        let graph: &Graph = &self.checker.graph;
        let source_members: Vec<Member> = source.members(graph)?.to_vec();
        if source_members.len() != target.members(graph)?.len() {
            return Ok(FALSE);
        }
        let mut result = TRUE;
        for member in &source_members {
            let Some(target_member) = target.member(graph, member.name)?.cloned() else {
                return Ok(FALSE);
            };
            if member.optional != target_member.optional {
                return Ok(FALSE);
            }
            let related = self.is_related_to(&member.r#type()?, &target_member.r#type()?, false)?;
            if related == FALSE {
                return Ok(FALSE);
            }
            result &= related;
        }
        Ok(result)
    }
}

/// `isDeeplyNestedType` with the cell's own identity as its recursion identity
/// (the fixtures have no instantiations): the stack holds `max_depth` or more
/// cells with that identity and non-decreasing ids.
fn is_deeply_nested_type(t: &TypeCell, stack: &[Rc<TypeCell>], max_depth: usize) -> bool {
    if stack.len() < max_depth {
        return false;
    }
    let mut count = 0;
    let mut last_id = 0;
    for cell in stack {
        if cell.id == t.id {
            if cell.id >= last_id {
                count += 1;
                if count >= max_depth {
                    return true;
                }
            }
            last_id = cell.id;
        }
    }
    false
}

#[cfg(test)]
mod tests;
