package harnessutil

// Access-only bridge installed into a disposable pinned tree. Accepted settings
// execute the original harness parser; removed settings retain the native CLI
// diagnostics instead of being silently omitted or failing the export process.
import (
	"encoding/hex"
	"slices"
	"strings"
	"testing"
	"testing/fstest"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions/tsoptionstest"
)

func S07SubsetLibFiles() map[string]string {
	result := map[string]string{}
	for path, entry := range testLibFolderMap() {
		result[path] = hex.EncodeToString(entry.(*fstest.MapFile).Data)
	}
	return result
}

func S07SubsetOptions(t *testing.T, settings TestConfiguration, options *core.CompilerOptions, cwd string) (*HarnessOptions, []*ast.Diagnostic) {
	// These are the fixture defaults in CompileFiles, not compiler defaults.
	if options.NewLine == core.NewLineKindNone {
		options.NewLine = core.NewLineKindCRLF
	}
	if options.SkipDefaultLibCheck == core.TSUnknown {
		options.SkipDefaultLibCheck = core.TSTrue
	}
	options.NoErrorTruncation = core.TSTrue
	harness := &HarnessOptions{UseCaseSensitiveFileNames: true, CurrentDirectory: cwd}
	keys := make([]string, 0, len(settings))
	for key := range settings {
		keys = append(keys, key)
	}
	slices.Sort(keys)
	var errors []*ast.Diagnostic
	for _, key := range keys {
		value := settings[key]
		decl := getCommandLineOption(key)
		valid := decl != nil || getHarnessOption(key) != nil || key == "typescriptversion"
		if decl != nil && decl.Kind == tsoptions.CommandLineOptionTypeEnum {
			_, valid = decl.EnumMap().Get(strings.ToLower(value))
		}
		if !valid {
			parsed := tsoptions.ParseCommandLine([]string{"--" + key, value}, tsoptionstest.NewVFSParseConfigHost(map[string]string{}, cwd, true))
			if len(parsed.Errors) == 0 {
				t.Fatalf("unsupported harness option %s=%s lacks a native option diagnostic", key, value)
			}
			errors = append(errors, parsed.Errors...)
			continue
		}
		SetOptionsFromTestConfig(t, TestConfiguration{key: value}, options, harness, cwd, false)
	}
	return harness, errors
}
