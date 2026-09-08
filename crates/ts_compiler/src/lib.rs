//! Immutable program loading, without checker or emitter construction.
//! Every successful `Program::load` represents an executed loader closure;
//! unsupported source operations fail with a named boundary.
mod cache;
mod include_reason;
mod output_paths;
mod program_diagnostics;
mod verify_options;
pub use verify_options::{verify_compiler_options, FileIncludeDiagnostic, OptionVerification};
mod loader;
mod metadata;
mod resolver_host;
pub use cache::{FileCache, ProgramFile};
pub use loader::{Error, Program, ProgramOptions, Resolution, TypeResolution};
pub use metadata::SourceFileMetaData;
pub use resolver_host::ProgramResolverHost;

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod ownership_tests;
#[cfg(test)]
mod tests;
