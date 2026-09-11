//! Retained program data for the checker; no loading or resolution is repeated.
//! Project-reference programs are rejected by `Program::load`. The corresponding
//! checker redirect API remains an explicit unsupported operation.

use crate::{metadata, output_paths, Program, ProgramFile};
use std::sync::{Arc, OnceLock};
use ts_ast::{CompletedFile, NodeId, SourceFileMetaData};
use ts_checker::{CheckerHost, Error};
use ts_core::{CompilerOptions, ModuleKind, ResolutionMode};
use ts_module::ResolvedModule;
use ts_tsoptions::ParsedCommandLine;
use ts_tspath as path;

pub struct ProgramCheckerHost {
    program: Arc<Program>,
    common_source_directory: OnceLock<Result<Vec<u8>, ts_arena::Error>>,
}

impl ProgramCheckerHost {
    pub fn new(program: Arc<Program>) -> Self {
        Self {
            program,
            common_source_directory: OnceLock::new(),
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    fn file(&self, file_name: &[u8]) -> Option<&ProgramFile> {
        let key = path::to_path(
            file_name,
            self.program.current_directory(),
            self.program.host().use_case_sensitive_file_names(),
        );
        self.program.file(key.as_bytes())
    }

    fn required_file(&self, file_name: &[u8]) -> Result<&ProgramFile, Error> {
        self.file(file_name)
            .ok_or_else(|| ts_arena::Error::WrongOwner.into())
    }

    fn file_metadata(&self, file: &ProgramFile) -> Result<&SourceFileMetaData, Error> {
        let source = file.bound().view().source_file()?;
        self.program
            .metadata(source.parse_options().path.as_bytes())
            .ok_or_else(|| ts_arena::Error::InvalidGraph.into())
    }

    fn emitted_file_names(&self) -> Result<Vec<ts_jsstring::JsString>, ts_arena::Error> {
        let mut names = Vec::new();
        for file in self.program.files() {
            if output_paths::may_emit_with_force_dts(file, &self.program, false)? {
                let source = file.bound().view().source_file()?;
                names.push(source.parse_options().file_name.clone());
            }
        }
        Ok(names)
    }
}

impl CheckerHost for ProgramCheckerHost {
    // port: tsc/internal/compiler/program.go:Program.Options
    fn options(&self) -> &CompilerOptions {
        self.program.options()
    }

    // port: tsc/internal/compiler/program.go:Program.SourceFiles
    fn source_file_count(&self) -> usize {
        self.program.files().len()
    }

    fn source_file(&self, index: usize) -> &CompletedFile {
        self.program.files()[index].bound()
    }

    // port: tsc/internal/compiler/program.go:Program.FileExists
    fn file_exists(&self, file_name: &[u8]) -> Result<bool, ts_vfs::Error> {
        self.program.host().file_exists(file_name)
    }

    // port: tsc/internal/compiler/program.go:Program.GetSourceFile
    fn get_source_file(&self, file_name: &[u8]) -> Option<&CompletedFile> {
        self.file(file_name).map(ProgramFile::bound)
    }

    // port: tsc/internal/compiler/program.go:Program.GetSourceFileForResolvedModule
    fn get_source_file_for_resolved_module(&self, file_name: &[u8]) -> Option<&CompletedFile> {
        // Package redirects are already entries in Program's by-path index.
        // The upstream missing-file fallback is a project-reference redirect;
        // programs requiring it are rejected during loading.
        self.get_source_file(file_name)
    }

    // port: tsc/internal/compiler/program.go:Program.GetEmitModuleFormatOfFile
    fn get_emit_module_format_of_file(&self, file_name: &[u8]) -> Result<ModuleKind, Error> {
        let file = self.required_file(file_name)?;
        let source = file.bound().view().source_file()?;
        Ok(metadata::emit_format(
            source.parse_options().file_name.as_bytes(),
            self.program.options(),
            self.file_metadata(file)?,
        ))
    }

    // port: tsc/internal/compiler/program.go:Program.GetEmitSyntaxForUsageLocation
    fn get_emit_syntax_for_usage_location(
        &self,
        file_name: &[u8],
        usage_location: NodeId,
    ) -> Result<ResolutionMode, Error> {
        let file = self.required_file(file_name)?;
        let view = file.bound().view().ast();
        let source = view.source_file(file.source())?;
        Ok(metadata::emit_syntax(
            view,
            source.parse_options().file_name.as_bytes(),
            self.file_metadata(file)?,
            usage_location,
            self.program.options(),
        )?)
    }

    // port: tsc/internal/compiler/program.go:Program.GetImpliedNodeFormatForEmit
    fn get_implied_node_format_for_emit(&self, file_name: &[u8]) -> Result<ModuleKind, Error> {
        let file = self.required_file(file_name)?;
        let source = file.bound().view().source_file()?;
        Ok(metadata::implied_for_emit(
            source.parse_options().file_name.as_bytes(),
            self.program.options().emit_module_kind(),
            self.file_metadata(file)?,
        ))
    }

    // port: tsc/internal/compiler/program.go:Program.GetResolvedModule
    fn get_resolved_module(
        &self,
        file_name: &[u8],
        module_reference: &[u8],
        mode: ResolutionMode,
    ) -> Result<Option<&ResolvedModule>, Error> {
        let file = self.required_file(file_name)?;
        let source = file.bound().view().source_file()?;
        let file_path = source.parse_options().path.as_bytes();
        let resolutions = self.program.resolutions();
        // Loader publication sorts and deduplicates this exact tuple.
        Ok(resolutions
            .binary_search_by(|entry| {
                (entry.file.as_bytes(), entry.name.as_bytes(), entry.mode).cmp(&(
                    file_path,
                    module_reference,
                    mode,
                ))
            })
            .ok()
            .map(|index| &resolutions[index].result))
    }

    // port: tsc/internal/compiler/program.go:Program.GetSourceFileMetaData
    fn get_source_file_meta_data(&self, file_name: &[u8]) -> Result<&SourceFileMetaData, Error> {
        self.file_metadata(self.required_file(file_name)?)
    }

    // port: tsc/internal/compiler/program.go:Program.SourceFileMayBeEmitted
    fn source_file_may_be_emitted(
        &self,
        file: &CompletedFile,
        force_dts_emit: bool,
    ) -> Result<bool, Error> {
        let index = self
            .program
            .owners
            .node_file_index(file.source())
            .ok_or(ts_arena::Error::WrongOwner)?;
        let retained = &self.program.files()[index];
        if retained.source() != file.source() {
            return Err(ts_arena::Error::WrongOwner.into());
        }
        Ok(output_paths::may_emit_with_force_dts(
            retained,
            &self.program,
            force_dts_emit,
        )?)
    }

    // port: tsc/internal/compiler/program.go:Program.IsSourceFileDefaultLibrary
    fn is_source_file_default_library(&self, path: &[u8]) -> bool {
        self.program.is_lib(path)
    }

    fn get_redirect_for_resolution(
        &self,
        file_name: &[u8],
    ) -> Result<Option<&ParsedCommandLine>, Error> {
        self.required_file(file_name)?;
        Err(Error::Unsupported(
            "GetRedirectForResolution: project references",
        ))
    }

    // port: tsc/internal/compiler/program.go:Program.CommonSourceDirectory
    fn common_source_directory(&self) -> Result<&[u8], Error> {
        match self.common_source_directory.get_or_init(|| {
            let files = self.emitted_file_names()?;
            // Program::load already ran the corresponding option verifier,
            // including checkSourceFilesBelongToPath's membership diagnostics.
            Ok(output_paths::common_directory(&self.program, &files))
        }) {
            Ok(directory) => Ok(directory),
            Err(error) => Err((*error).into()),
        }
    }

    // port: tsc/internal/compiler/program.go:Program.GetCurrentDirectory
    fn get_current_directory(&self) -> &[u8] {
        self.program.current_directory()
    }

    // port: tsc/internal/compiler/program.go:Program.UseCaseSensitiveFileNames
    fn use_case_sensitive_file_names(&self) -> bool {
        self.program.host().use_case_sensitive_file_names()
    }
}

#[cfg(test)]
#[path = "checker_host_tests.rs"]
mod tests;
