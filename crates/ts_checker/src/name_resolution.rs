//! The binder's lexical walk with checker lookup and side effects. The resolver
//! is scoped to one operation; cached checker links are updated after the walk.

use crate::{CheckerState, Error};
use ts_arena::{NodeId, SymbolId};
use ts_ast::{
    symbol_flags as sf, AstView, DeclarationRead, JsString, NodeBinding, SymbolFlags, SymbolRef,
    SymbolTableId, SymbolTableRead,
};
use ts_binder::name_resolver::{
    Hook, NameResolver, NameResolverHooks, ResolvedName, ResolverHost, ResolverOptions,
};
use ts_core::Tristate;
use ts_diagnostics::Message;

struct ReadHost<'a>(&'a CheckerState);
fn arena(error: Error) -> ts_arena::Error {
    match error {
        Error::Arena(error) => error,
        _ => ts_arena::Error::InvalidGraph,
    }
}
impl ResolverHost for ReadHost<'_> {
    fn ast(&self, node: NodeId) -> Result<AstView<'_>, ts_arena::Error> {
        self.0.ast(node).map_err(arena)
    }
    fn binding(&self, node: NodeId) -> Result<Option<NodeBinding>, ts_arena::Error> {
        self.0
            .program()
            .map_err(arena)?
            .bound(node)?
            .node_binding(node)
    }
    fn symbol(&self, id: SymbolId) -> Result<SymbolRef<'_>, ts_arena::Error> {
        self.0.symbol(id).map_err(arena)
    }
    fn table(&self, id: SymbolTableId) -> Result<SymbolTableRead<'_>, ts_arena::Error> {
        self.0.table(id).map_err(arena)
    }
    fn declarations(&self, id: SymbolId) -> Result<DeclarationRead<'_>, ts_arena::Error> {
        self.0.symbol_declarations(id).map_err(arena)
    }
    fn new_transient_symbol(
        &mut self,
        _: SymbolFlags,
        _: JsString,
    ) -> Result<SymbolId, ts_arena::Error> {
        // The binder creates only a missing arguments symbol. NewChecker
        // always installs its own before this resolver can run.
        unreachable!("initialized checker resolver already has argumentsSymbol")
    }
}

enum Effect {
    Error(Option<NodeId>, &'static Message, Vec<JsString>),
    Referenced(SymbolId, SymbolFlags),
    Scope(NodeId, Tristate),
    Resolved(ResolvedName),
}
struct Hooks<'a> {
    state: &'a CheckerState,
    effects: Vec<Effect>,
    failure: Option<Error>,
}
impl Hooks<'_> {
    fn capture<T>(&mut self, result: Result<T, Error>) -> Result<T, ts_arena::Error> {
        result.map_err(|error| {
            self.failure = Some(error);
            arena(error)
        })
    }
}
impl NameResolverHooks for Hooks<'_> {
    fn get_symbol_of_declaration(
        &mut self,
        node: NodeId,
    ) -> Result<Hook<Option<SymbolId>>, ts_arena::Error> {
        let result = self.state.get_symbol_of_declaration(node).map(Hook::Value);
        self.capture(result)
    }
    fn lookup(
        &mut self,
        table: Option<SymbolTableId>,
        name: &[u8],
        meaning: SymbolFlags,
    ) -> Result<Hook<Option<SymbolId>>, ts_arena::Error> {
        let result = self
            .state
            .lookup_symbol(table, name, meaning)
            .map(Hook::Value);
        self.capture(result)
    }
    fn error(
        &mut self,
        location: Option<NodeId>,
        message: &'static Message,
        args: &[JsString],
    ) -> Result<(), ts_arena::Error> {
        self.effects
            .push(Effect::Error(location, message, args.to_vec()));
        Ok(())
    }
    fn symbol_referenced(
        &mut self,
        symbol: SymbolId,
        meaning: SymbolFlags,
    ) -> Result<(), ts_arena::Error> {
        self.effects.push(Effect::Referenced(symbol, meaning));
        Ok(())
    }
    fn get_requires_scope_change_cache(
        &mut self,
        node: NodeId,
    ) -> Result<Tristate, ts_arena::Error> {
        for effect in self.effects.iter().rev() {
            if let Effect::Scope(id, value) = effect {
                if *id == node {
                    return Ok(*value);
                }
            }
        }
        Ok(self
            .state
            .query
            .scope_changes
            .try_get(node)
            .copied()
            .unwrap_or(Tristate::UNKNOWN))
    }
    fn set_requires_scope_change_cache(
        &mut self,
        node: NodeId,
        value: Tristate,
    ) -> Result<(), ts_arena::Error> {
        self.effects.push(Effect::Scope(node, value));
        Ok(())
    }
    fn on_property_with_invalid_initializer(
        &mut self,
        _: Option<NodeId>,
        _: &[u8],
        _: NodeId,
        _: Option<SymbolId>,
    ) -> Result<bool, ts_arena::Error> {
        self.capture(Err(Error::Unsupported(
            "checkAndReportErrorForInvalidInitializer",
        )))
    }
    fn on_failed_to_resolve_symbol(
        &mut self,
        location: Option<NodeId>,
        name: &[u8],
        meaning: SymbolFlags,
        message: &'static Message,
    ) -> Result<(), ts_arena::Error> {
        // Missing source names need the diagnostic/suggestion closure; globals
        // with no location cannot request those source-context suggestions.
        if location.is_some() {
            return self.capture(Err(Error::Unsupported(
                "onFailedToResolveSymbol: source name suggestions",
            )));
        }
        let mut args = vec![JsString::from_bytes(name)];
        if let Some(lib) = suggested_library(name) {
            args.push(JsString::from_bytes(lib));
        } else {
            // The source next tries spelling suggestions, even for globals.
            // A potential suggestion must enter that unported diagnostic path,
            // rather than being mislabeled as an ordinary missing global.
            if let Some(globals) = self.state.builtins.globals {
                let table = self.capture(self.state.table(globals))?;
                let mut candidates = Vec::new();
                for (_, candidate) in table {
                    let Some(candidate) = candidate else { continue };
                    let symbol = self.capture(self.state.symbol(candidate))?;
                    if symbol.flags() & sf::ALIAS != 0 && symbol.flags() & meaning == 0 {
                        return self.capture(Err(Error::Unsupported(
                            "getSpellingSuggestionForName: alias",
                        )));
                    }
                    if symbol.flags() & meaning != 0
                        && !matches!(symbol.name_bytes().first(), Some(b'"' | 0xfe))
                    {
                        candidates.push(symbol.name_bytes());
                    }
                }
                if ts_scanner::get_spelling_suggestion_for_strings(name, candidates).is_some() {
                    return self.capture(Err(Error::Unsupported("getSpellingSuggestionForName")));
                }
            }
        }
        self.effects.push(Effect::Error(None, message, args));
        Ok(())
    }
    fn on_successfully_resolved_symbol(
        &mut self,
        resolution: ResolvedName,
    ) -> Result<(), ts_arena::Error> {
        self.effects.push(Effect::Resolved(resolution));
        Ok(())
    }
}

impl CheckerState {
    // port: tsc/internal/checker/checker.go:Checker.getSymbol
    pub(crate) fn lookup_symbol(
        &self,
        table: Option<SymbolTableId>,
        name: &[u8],
        meaning: SymbolFlags,
    ) -> Result<Option<SymbolId>, Error> {
        if meaning & sf::ALL == 0 {
            return Ok(None);
        }
        let Some(table) = table else { return Ok(None) };
        let Some(symbol) = self.table(table)?.get(name).flatten() else {
            return Ok(None);
        };
        let symbol = self.get_merged_symbol(symbol);
        let flags = self.symbol(symbol)?.flags();
        if flags & meaning != 0 {
            return Ok(Some(symbol));
        }
        if flags & sf::ALIAS != 0 {
            return Err(Error::Unsupported("getSymbolFlags: alias resolution"));
        }
        Ok(None)
    }

    // port: tsc/internal/checker/checker.go:Checker.createNameResolver
    pub(crate) fn resolve_name(
        &mut self,
        location: Option<NodeId>,
        name: &[u8],
        meaning: SymbolFlags,
        message: Option<&'static Message>,
        is_use: bool,
    ) -> Result<Option<SymbolId>, Error> {
        let options = self.program()?.host.options();
        let mut resolver = NameResolver {
            options: ResolverOptions {
                emit_script_target: options.emit_script_target(),
                isolated_modules: options.isolated_modules(),
                verbatim_module_syntax: options.verbatim_module_syntax.is_true(),
                emit_standard_class_fields: options.emit_standard_class_fields(),
            },
            globals: self.builtins.globals,
            arguments_symbol: Some(self.builtins.arguments_symbol),
            require_symbol: Some(self.builtins.require_symbol),
        };
        let mut hooks = Hooks {
            state: self,
            effects: Vec::new(),
            failure: None,
        };
        let result = resolver.resolve(
            &mut ReadHost(self),
            &mut hooks,
            location,
            name,
            meaning,
            message,
            is_use,
            false,
        );
        let failure = hooks.failure;
        let effects = hooks.effects;
        for effect in effects {
            match effect {
                Effect::Error(location, message, args) => {
                    self.error_at(location, message, args)?;
                }
                Effect::Referenced(symbol, meaning) => {
                    *self.query.references.get_or_default(symbol) |= meaning;
                }
                Effect::Scope(node, value) => {
                    *self.query.scope_changes.get_or_default(node) = value;
                }
                Effect::Resolved(resolution) => self.on_source_symbol_resolved(resolution)?,
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(result?)
    }

    // port: tsc/internal/checker/checker.go:Checker.onSuccessfullyResolvedSymbol
    fn on_source_symbol_resolved(&mut self, resolution: ResolvedName) -> Result<(), Error> {
        let Some(location) = resolution.original_location else {
            return Ok(());
        };
        let meaning = resolution.meaning;
        let symbol = self.symbol(resolution.symbol)?;
        // TYPE and VALUE overlap (classes/enums). Testing for any overlap
        // would incorrectly run value-only checks on an ordinary type name.
        if meaning & sf::BLOCK_SCOPED_VARIABLE != 0
            || meaning & (sf::CLASS | sf::ENUM) != 0 && meaning & sf::VALUE == sf::VALUE
        {
            if symbol.export_symbol().is_some() {
                return Err(Error::Unsupported("getExportSymbolOfValueSymbolIfExported"));
            }
            if symbol.flags() & (sf::BLOCK_SCOPED_VARIABLE | sf::CLASS | sf::ENUM) != 0 {
                self.check_resolved_block_scoped_variable(resolution.symbol, location)?;
            }
        }
        if meaning & sf::VALUE == sf::VALUE {
            if let Some(last) = resolution.last_location {
                let view = self.ast(last)?;
                if view.node(last)?.kind() == ts_ast::SyntaxKind::SourceFile
                    && ts_ast::utilities::is_external_or_common_js_module(&view.source_file(last)?)
                {
                    return Err(Error::Unsupported(
                        "onSuccessfullyResolvedSymbol: UMD/isolated module checks",
                    ));
                }
            }
            if resolution.associated_declaration.is_some() && !resolution.within_deferred_context {
                return Err(Error::Unsupported(
                    "onSuccessfullyResolvedSymbol: parameter initializer",
                ));
            }
        }
        let flags = self.symbol(resolution.symbol)?.flags();
        if meaning & sf::VALUE != 0 && flags & sf::ALIAS != 0 && flags & sf::VALUE == 0 {
            return Err(Error::Unsupported(
                "onSuccessfullyResolvedSymbol: type-only alias use",
            ));
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkResolvedBlockScopedVariable
    // port: tsc/internal/checker/checker.go:Checker.isBlockScopedNameDeclaredBeforeUse
    fn check_resolved_block_scoped_variable(
        &self,
        symbol: SymbolId,
        usage: NodeId,
    ) -> Result<(), Error> {
        let flags = self.symbol(symbol)?.flags();
        if flags & sf::CLASS != 0
            && flags & (sf::FUNCTION | sf::FUNCTION_SCOPED_VARIABLE | sf::ASSIGNMENT) != 0
        {
            return Ok(());
        }
        let mut declaration = None;
        for node in self.symbol_declarations(symbol)?.iter().flatten() {
            let read = self.ast(node)?.node(node)?;
            if matches!(
                read.kind().known(),
                Some(
                    ts_ast::SyntaxKind::ClassDeclaration
                        | ts_ast::SyntaxKind::ClassExpression
                        | ts_ast::SyntaxKind::EnumDeclaration
                )
            ) {
                return Err(Error::Unsupported(
                    "checkResolvedBlockScopedVariable: class/enum",
                ));
            }
            if read.kind() == ts_ast::SyntaxKind::VariableDeclaration {
                let parent = read.parent().ok_or(Error::MissingLink("variable parent"))?;
                let parent = self.ast(parent)?.node(parent)?;
                if parent.flags() & ts_ast::node_flags::BLOCK_SCOPED != 0 {
                    declaration = Some(node);
                    break;
                }
            }
        }
        let declaration = declaration.ok_or(Error::Unsupported(
            "checkResolvedBlockScopedVariable: binding/catch declaration",
        ))?;
        let view = self.ast(declaration)?;
        let declared = view.node(declaration)?;
        if declared.flags() & ts_ast::node_flags::AMBIENT != 0 {
            return Ok(());
        }
        let source = ts_ast::utilities::get_source_file_of_node(view, Some(declaration))?;
        let used = self.ast(usage)?;
        if source != ts_ast::utilities::get_source_file_of_node(used, Some(usage))? {
            return Ok(());
        }
        if declared.pos() > used.node(usage)?.pos() {
            return Err(Error::Unsupported(
                "isBlockScopedNameDeclaredBeforeUse: deferred use/TDZ diagnostic",
            ));
        }
        // For a simple variable used later outside its own initializer, Go's
        // immediately-used-in-initializer test is false. Nested functions,
        // binding patterns and uses inside the initializer need its full walk.
        let mut ancestor = Some(usage);
        while let Some(node) = ancestor {
            if node == declaration {
                return Err(Error::Unsupported(
                    "isImmediatelyUsedInInitializerOfBlockScopedVariable",
                ));
            }
            ancestor = self.ast(node)?.node(node)?.parent();
        }
        Ok(())
    }
}

// First library of every feature-map key at the pin. Property-specific entries
// belong to the later missing-property diagnostic path.
// port: tsc/internal/checker/checker.go:Checker.getSuggestedLibForNonExistentName
// Source table: getFeatureMap variable initializer in tsc/internal/checker/utilities.go.
fn suggested_library(name: &[u8]) -> Option<&'static [u8]> {
    match name {
        b"Array"
        | b"Iterator"
        | b"AsyncIterator"
        | b"RegExp"
        | b"Reflect"
        | b"ArrayConstructor"
        | b"ObjectConstructor"
        | b"NumberConstructor"
        | b"Math"
        | b"Map"
        | b"Set"
        | b"PromiseConstructor"
        | b"Symbol"
        | b"WeakMap"
        | b"WeakSet"
        | b"String"
        | b"StringConstructor"
        | b"Promise" => Some(b"es2015"),
        b"ArrayBuffer" | b"MapConstructor" => Some(b"es2024"),
        b"Atomics" | b"SharedArrayBuffer" | b"DateTimeFormat" => Some(b"es2017"),
        b"AsyncIterable"
        | b"AsyncIterableIterator"
        | b"AsyncGenerator"
        | b"AsyncGeneratorFunction"
        | b"RegExpMatchArray"
        | b"RegExpExecArray"
        | b"Intl"
        | b"NumberFormat" => Some(b"es2018"),
        b"RegExpConstructor" | b"Float16Array" => Some(b"es2025"),
        b"SymbolConstructor"
        | b"DataView"
        | b"BigInt"
        | b"RelativeTimeFormat"
        | b"BigInt64Array"
        | b"BigUint64Array" => Some(b"es2020"),
        b"Int8Array" | b"Uint8Array" | b"Uint8ClampedArray" | b"Int16Array" | b"Uint16Array"
        | b"Int32Array" | b"Uint32Array" | b"Float32Array" | b"Float64Array" | b"Error" => {
            Some(b"es2022")
        }
        b"ErrorConstructor"
        | b"Uint8ArrayConstructor"
        | b"DisposableStack"
        | b"AsyncDisposableStack"
        | b"Date" => Some(b"esnext"),
        _ => None,
    }
}
