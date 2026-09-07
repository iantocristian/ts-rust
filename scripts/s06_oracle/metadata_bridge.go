// Access-only: call the pinned metadata loader without constructing a Program.
package compiler

import (
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/module"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
)

func S06MetadataForFile(host module.ResolutionHost, options *core.CompilerOptions, filename string) ast.SourceFileMetaData {
	loader := fileLoader{
		opts:     ProgramOptions{Config: &tsoptions.ParsedCommandLine{ParsedConfig: &tsoptions.ParsedOptions{CompilerOptions: options}}},
		resolver: module.NewResolver(host, options, "", "", nil),
	}
	return loader.loadSourceFileMetaData(filename)
}
