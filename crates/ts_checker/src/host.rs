//! The program surface the checker consumes (`checker.Program` and `checker.Host`
//! in `tsc/internal/checker/checker.go`; `Host` is
//! `modulespecifiers.ModuleSpecifierGenerationHost`).
//!
//! The trait lives here, below `ts_compiler`, and the compiler implements it over
//! its loader (plan §3). Only members whose types already exist below the
//! compiler are in the trait. The rest are P2 obligations, listed so none of them
//! is quietly given a default:
//!
//! | Upstream member | Status |
//! | --- | --- |
//! | `Options`, `SourceFiles`, `FileExists`, `GetSourceFile`, `IsSourceFileDefaultLibrary`, `CommonSourceDirectory`, `GetCurrentDirectory`, `UseCaseSensitiveFileNames` | In the trait; the loader already has the data |
//! | `GetSourceFileForResolvedModule`, `GetResolvedModule`, `GetEmitModuleFormatOfFile`, `GetImpliedNodeFormatForEmit`, `GetEmitSyntaxForUsageLocation`, `SourceFileMayBeEmitted` | Implemented by the compiler's retained program adapter |
//! | `BindSourceFiles` | Satisfied by construction: program files are `CompletedFile`s |
//! | `GetSourceFileMetaData` | In the trait; metadata lives in `ts_ast`, as upstream |
//! | `GetResolvedModules`, `GetPackagesMap` | P2: whole-map views; add when a caller in the closure needs them |
//! | `GetJSXRuntimeImportSpecifier`, `GetImportHelpersImportSpecifier` | P2: synthesize nodes through the checker AST factory; JSX is outside the frozen denominator, `importHelpers` is not |
//! | `GetRedirectForResolution`, `GetProjectReferenceFromOutputDts`, `GetProjectReferenceFromSource`, `GetRedirectTargets`, `GetSourceOfProjectReferenceIfOutputIncluded` | Explicitly unsupported: project references are excluded from the denominator; the implementation must say so, not return `None` |
//! | `GetSymlinkCache`, `GetPackageJsonInfo`, `GetNearestAncestorDirectoryWithPackageJson`, `GetGlobalTypingsCacheLocation`, `ContentMapperExtensions`, `GetDefaultResolutionModeForFile`, `GetResolvedModuleFromModuleSpecifier`, `GetModeForUsageLocation` | P2, with the node builder's import-type specifier generation |

use crate::Error;
use ts_arena::NodeId;
use ts_ast::{CompletedFile, SourceFileMetaData};
use ts_core::{CompilerOptions, ModuleKind, ResolutionMode};
use ts_module::ResolvedModule;
use ts_tsoptions::ParsedCommandLine;

/// File names and directories are bytes, as everywhere in this port.
pub trait CheckerHost: Send + Sync {
    fn options(&self) -> &CompilerOptions;
    fn source_file_count(&self) -> usize;
    /// Program order; the checker's file index map and node ordering follow it.
    fn source_file(&self, index: usize) -> &CompletedFile;
    fn file_exists(&self, file_name: &[u8]) -> Result<bool, ts_vfs::Error>;
    fn get_source_file(&self, file_name: &[u8]) -> Option<&CompletedFile>;
    fn get_source_file_for_resolved_module(&self, file_name: &[u8]) -> Option<&CompletedFile>;
    fn get_emit_module_format_of_file(&self, file_name: &[u8]) -> Result<ModuleKind, Error>;
    fn get_emit_syntax_for_usage_location(
        &self,
        file_name: &[u8],
        usage_location: NodeId,
    ) -> Result<ResolutionMode, Error>;
    fn get_implied_node_format_for_emit(&self, file_name: &[u8]) -> Result<ModuleKind, Error>;
    fn get_resolved_module(
        &self,
        file_name: &[u8],
        module_reference: &[u8],
        mode: ResolutionMode,
    ) -> Result<Option<&ResolvedModule>, Error>;
    fn get_source_file_meta_data(&self, file_name: &[u8]) -> Result<&SourceFileMetaData, Error>;
    fn source_file_may_be_emitted(
        &self,
        file: &CompletedFile,
        force_dts_emit: bool,
    ) -> Result<bool, Error>;
    fn is_source_file_default_library(&self, path: &[u8]) -> bool;
    /// Project-reference resolution is an explicit unsupported boundary until
    /// reference loading and redirects enter the executable operation closure.
    fn get_redirect_for_resolution(
        &self,
        file_name: &[u8],
    ) -> Result<Option<&ParsedCommandLine>, Error>;
    fn common_source_directory(&self) -> Result<&[u8], Error>;
    fn get_current_directory(&self) -> &[u8];
    fn use_case_sensitive_file_names(&self) -> bool;
}
