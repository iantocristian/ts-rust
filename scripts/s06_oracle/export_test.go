// Access-only fixture export. Installed only in a clean disposable pinned tree.
package testrunner

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/bundled"
	"github.com/microsoft/TypeScript/tsc/internal/compiler"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnostics"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions/tsoptionstest"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
)

type s06Unit struct {
	Name        string            `json:"name"`
	TextHex     string            `json:"text_hex"`
	ScriptKind  int32             `json:"script_kind"`
	FileOptions map[string]string `json:"file_options"`
}

type s06Case struct {
	Kind              string              `json:"kind"`
	Path              string              `json:"path"`
	RawSHA256         string              `json:"raw_sha256"`
	LoadedSHA256      string              `json:"loaded_sha256"`
	LoadedBytes       int                 `json:"loaded_bytes"`
	Units             []s06Unit           `json:"units"`
	Symlinks          map[string]string   `json:"symlinks"`
	CurrentDirectory  string              `json:"current_directory"`
	GlobalOptions     map[string]string   `json:"global_options"`
	RawSettings       map[string]string   `json:"raw_settings"`
	Configurations    []map[string]string `json:"configurations"`
	Variants          []s06Variant        `json:"variants"`
	ConfigDiagnostics []s06Diagnostic     `json:"config_diagnostics"`
	LegacyProjection  *s06Legacy          `json:"legacy_projection"`
}

type s06Legacy struct {
	Policy           string            `json:"policy"`
	OriginalSettings map[string]string `json:"original_settings"`
	RejectedHelper   string            `json:"rejected_helper"`
	RejectedMessage  string            `json:"rejected_message"`
}

var s06LegacyPaths = []string{
	"tsc/testdata/tests/cases/compiler/moduleNoneDynamicImport.ts",
	"tsc/testdata/tests/cases/compiler/moduleNoneErrors.ts",
	"tsc/testdata/tests/cases/compiler/noErrorUsingImportExportModuleAugmentationInDeclarationFile1.ts",
	"tsc/testdata/tests/cases/compiler/noErrorUsingImportExportModuleAugmentationInDeclarationFile2.ts",
	"tsc/testdata/tests/cases/compiler/noErrorUsingImportExportModuleAugmentationInDeclarationFile3.ts",
	"tsc/testdata/tests/cases/compiler/requireOfJsonFileWithModuleEmitNone.ts",
	"tsc/testdata/tests/cases/compiler/requireOfJsonFileWithModuleNodeResolutionEmitNone.ts",
}

type s06Diagnostic struct {
	Code  int32 `json:"code"`
	Start int   `json:"start"`
	End   int   `json:"end"`
}

type s06Read struct {
	Path       string `json:"path"`
	Exists     bool   `json:"exists"`
	TextSHA256 string `json:"text_sha256"`
}

type s06TrackingFS struct {
	vfs.FS
	reads map[string]s06Read
}

func (f *s06TrackingFS) ReadFile(path string) (string, bool) {
	text, ok := f.FS.ReadFile(path)
	f.reads[path] = s06Read{path, ok, s06Hash(text)}
	return text, ok
}

type s06ResolutionHost struct {
	fs  vfs.FS
	cwd string
}

func (h s06ResolutionHost) FS() vfs.FS                             { return h.fs }
func (h s06ResolutionHost) GetCurrentDirectory() string            { return h.cwd }
func (h s06ResolutionHost) Trace(_ *diagnostics.Message, _ ...any) {}

type s06ParserInput struct {
	Unit       int                    `json:"unit"`
	Filename   string                 `json:"filename"`
	Path       string                 `json:"path"`
	ScriptKind int32                  `json:"script_kind"`
	Route      string                 `json:"route"`
	TextHex    string                 `json:"text_hex"`
	JSX        bool                   `json:"jsx"`
	Force      bool                   `json:"force"`
	Metadata   ast.SourceFileMetaData `json:"metadata"`
}

type s06Variant struct {
	Configuration     int               `json:"configuration"`
	CurrentDirectory  string            `json:"current_directory"`
	CaseSensitive     bool              `json:"case_sensitive"`
	ParseSettings     map[string]string `json:"parse_settings"`
	OtherSettings     map[string]string `json:"other_settings"`
	Inputs            []s06ParserInput  `json:"inputs"`
	Reads             []s06Read         `json:"reads"`
	OptionDiagnostics []s06Diagnostic   `json:"option_diagnostics"`
}

// This projection is a fixture boundary, not a reimplementation of option
// parsing: every selected value is processed by SetOptionsFromTestConfig.
// target affects default emit module kind; module resolution controls whether
// package type metadata contributes; case sensitivity determines tspath.Path.
var s06ParseSettings = []string{"target", "module", "moduledetection", "moduleresolution", "jsx", "usecasesensitivefilenames"}

// Match newCompilerTest's partition before CompileFilesEx inserts roots then
// auxiliary files. Source-order insertion changes duplicate virtual filenames.
// Shared by S06 parser-input extraction and S07 program-request assembly.
func s06ProgramUnits(payload *testCaseContent, config map[string]string, cwd string) (inputs, other []*testUnit) {
	units := payload.testUnitData
	if payload.tsConfig != nil {
		for _, unit := range units {
			if slices.Contains(payload.tsConfig.ParsedConfig.FileNames, tspath.GetNormalizedAbsolutePath(unit.name, cwd)) {
				inputs = append(inputs, unit)
			} else {
				other = append(other, unit)
			}
		}
		return
	}
	if len(units) != 0 {
		last := units[len(units)-1]
		if config["noimplicitreferences"] != "" || strings.Contains(last.content, requireStr) || referencesRegex.MatchString(last.content) {
			return units[len(units)-1:], units[:len(units)-1]
		}
	}
	return units, nil
}

func s06Resolve(t *testing.T, result *s06Case, loaded, physical string) {
	t.Helper()
	payload := makeUnitsFromTest(loaded, physical)
	result.ConfigDiagnostics = []s06Diagnostic{}
	if payload.tsConfig != nil {
		for _, d := range payload.tsConfig.Errors {
			result.ConfigDiagnostics = append(result.ConfigDiagnostics, s06Diagnostic{d.Code(), d.Loc().Pos(), d.Loc().End()})
		}
	}
	configIndex := -1
	if payload.tsConfigFileUnitData != nil {
		for i, unit := range result.Units {
			if unit.Name == payload.tsConfigFileUnitData.name {
				configIndex = i
				break
			}
		}
	}
	for ci, config := range result.Configurations {
		cwd := tspath.GetNormalizedAbsolutePath(config["currentdirectory"], srcFolder)
		options := &core.CompilerOptions{}
		if payload.tsConfig != nil && payload.tsConfig.ParsedConfig.CompilerOptions != nil {
			options = payload.tsConfig.ParsedConfig.CompilerOptions.Clone()
		}
		harness := harnessutil.HarnessOptions{UseCaseSensitiveFileNames: true, CurrentDirectory: cwd}
		selected, other := map[string]string{}, map[string]string{}
		for key, value := range config {
			if slices.Contains(s06ParseSettings, key) {
				selected[key] = value
			} else {
				other[key] = value
			}
		}
		optionDiagnostics := []s06Diagnostic{}
		if result.LegacyProjection == nil {
			harnessutil.SetOptionsFromTestConfig(t, selected, options, &harness, cwd, false)
		} else {
			if payload.tsConfig != nil {
				t.Fatal("legacy parser projection does not cover a tsconfig")
			}
			args := []string{}
			keys := make([]string, 0, len(selected))
			for key := range selected {
				keys = append(keys, key)
			}
			slices.Sort(keys)
			for _, key := range keys {
				if key != "usecasesensitivefilenames" {
					args = append(args, "--"+key, selected[key])
				}
			}
			parsed := tsoptions.ParseCommandLine(args, tsoptionstest.NewVFSParseConfigHost(map[string]string{}, cwd, true))
			if len(parsed.Errors) != 1 || parsed.Errors[0].Code() != 6046 || parsed.CompilerOptions().Module != core.ModuleKindNone {
				t.Fatal("reviewed legacy option rejection changed")
			}
			options = parsed.CompilerOptions()
			for _, d := range parsed.Errors {
				optionDiagnostics = append(optionDiagnostics, s06Diagnostic{d.Code(), d.Loc().Pos(), d.Loc().End()})
			}
			if value, ok := selected["usecasesensitivefilenames"]; ok {
				harnessutil.SetOptionsFromTestConfig(t, map[string]string{"usecasesensitivefilenames": value}, options, &harness, cwd, false)
			}
		}
		files := map[string]any{}
		inputs, otherUnits := s06ProgramUnits(&payload, config, cwd)
		for _, unit := range slices.Concat(inputs, otherUnits) {
			files[tspath.GetNormalizedAbsolutePath(unit.name, cwd)] = []byte(unit.content)
		}
		for from, to := range result.Symlinks {
			files[tspath.GetNormalizedAbsolutePath(from, cwd)] = vfstest.Symlink(tspath.GetNormalizedAbsolutePath(to, cwd))
		}
		fs := &s06TrackingFS{FS: vfstest.FromMap(files, harness.UseCaseSensitiveFileNames), reads: map[string]s06Read{}}
		host := s06ResolutionHost{fs, cwd}
		variant := s06Variant{Configuration: ci, CurrentDirectory: cwd, CaseSensitive: harness.UseCaseSensitiveFileNames, ParseSettings: selected, OtherSettings: other, Inputs: []s06ParserInput{}, Reads: []s06Read{}}
		variant.OptionDiagnostics = optionDiagnostics
		for i, unit := range result.Units {
			if unit.ScriptKind == int32(core.ScriptKindUnknown) {
				continue
			}
			filename := tspath.GetNormalizedAbsolutePath(unit.Name, cwd)
			text := ""
			route := "virtual_file"
			metadata := ast.SourceFileMetaData{}
			parseOptions := ast.ExternalModuleIndicatorOptions{}
			if i == configIndex {
				bytes, err := hex.DecodeString(unit.TextHex)
				if err != nil {
					t.Fatal(err)
				}
				text = string(bytes)
				route = "initial_config_direct"
				configCWD := result.CurrentDirectory
				if configCWD == "" {
					configCWD = srcFolder
				}
				filename = tspath.GetNormalizedAbsolutePath(unit.Name, configCWD)
			} else {
				var ok bool
				text, ok = fs.ReadFile(filename)
				if !ok {
					t.Fatalf("virtual unit cannot be read: %s", filename)
				}
				metadata = compiler.S06MetadataForFile(host, options, filename)
				parseOptions = ast.GetExternalModuleIndicatorOptions(filename, options, metadata)
			}
			path := tspath.ToPath(filename, cwd, harness.UseCaseSensitiveFileNames)
			if i == configIndex {
				path = tspath.ToPath(filename, cwd, true)
			}
			variant.Inputs = append(variant.Inputs, s06ParserInput{i, filename, string(path), unit.ScriptKind, route, hex.EncodeToString([]byte(text)), parseOptions.JSX, parseOptions.Force, metadata})
		}
		for _, read := range fs.reads {
			variant.Reads = append(variant.Reads, read)
		}
		slices.SortFunc(variant.Reads, func(a, b s06Read) int { return strings.Compare(a.Path, b.Path) })
		result.Variants = append(result.Variants, variant)
	}
}

func s06Hash(text string) string { return fmt.Sprintf("%x", sha256.Sum256([]byte(text))) }

func s06Extract(t *testing.T, path, raw, logicalPath string) s06Case {
	t.Helper()
	loaded, ok := osvfs.FS().ReadFile(path)
	if !ok {
		t.Fatalf("could not load physical case %s", path)
	}
	units, links, cwd, globals, err := ParseTestFilesAndSymlinks(loaded, path,
		func(name, content string, options map[string]string) (s06Unit, error) {
			return s06Unit{name, hex.EncodeToString([]byte(content)), int32(core.GetScriptKindFromFileName(name)), options}, nil
		})
	if err != nil {
		t.Fatal(err)
	}
	settings := extractCompilerSettings(loaded)
	expansionSettings := settings
	var legacy *s06Legacy
	if slices.Contains(s06LegacyPaths, logicalPath) {
		if settings["module"] != "none" {
			t.Fatal("reviewed legacy parser projection requires original module exactly none")
		}
		legacy = &s06Legacy{Policy: "legacy-module-none-parser-input-v1", OriginalSettings: settings, RejectedHelper: "harnessutil.GetFileBasedTestConfigurations", RejectedMessage: "Unknown value 'none' for option 'module'"}
		expansionSettings = map[string]string{}
		for key, value := range settings {
			if key != "module" {
				expansionSettings[key] = value
			}
		}
	}
	configs := harnessutil.GetFileBasedTestConfigurations(t, expansionSettings, compilerVaryBy)
	result := s06Case{Kind: "case", RawSHA256: s06Hash(raw), LoadedSHA256: s06Hash(loaded), LoadedBytes: len(loaded), Units: units, Symlinks: links, CurrentDirectory: cwd, GlobalOptions: globals, RawSettings: settings, Configurations: []map[string]string{}, Variants: []s06Variant{}}
	result.Path = logicalPath
	result.LegacyProjection = legacy
	for _, config := range configs {
		result.Configurations = append(result.Configurations, config.Config)
	}
	if len(result.Configurations) == 0 {
		result.Configurations = append(result.Configurations, map[string]string{})
	}
	if legacy != nil {
		for _, config := range result.Configurations {
			config["module"] = "none"
		}
	}
	slices.SortFunc(result.Configurations, func(a, b map[string]string) int {
		left, _ := json.Marshal(a)
		right, _ := json.Marshal(b)
		return strings.Compare(string(left), string(right))
	})
	if os.Getenv("S06_EXTRACT_ONLY") != "1" {
		s06Resolve(t, &result, loaded, path)
	}
	return result
}

func TestS06Export(t *testing.T) {
	input := os.Getenv("S06_INPUT")
	if input == "" {
		t.Skip("S06_INPUT is only set by the S06 exporter")
	}
	data, err := os.ReadFile(input)
	if err != nil {
		t.Fatal(err)
	}
	var paths []string
	if err = json.Unmarshal(data, &paths); err != nil {
		t.Fatal(err)
	}
	output, err := os.Create(os.Getenv("S06_OUTPUT"))
	if err != nil {
		t.Fatal(err)
	}
	defer output.Close()
	encoder := json.NewEncoder(output)
	for _, path := range paths {
		if !(strings.HasPrefix(path, "tsc/testdata/tests/cases/compiler/") || strings.HasPrefix(path, "tsc/testdata/tests/cases/conformance/")) || filepath.ToSlash(filepath.Clean(path)) != path {
			t.Fatalf("invalid physical path %q", path)
		}
		if !t.Run(path, func(t *testing.T) {
			defer func() {
				if failure := recover(); failure != nil {
					t.Fatalf("preprocessing panic: %v", failure)
				}
			}()
			local, err := filepath.Abs(filepath.Join("..", "..", "..", filepath.FromSlash(path)))
			if err != nil {
				t.Fatal(err)
			}
			raw, err := os.ReadFile(local)
			if err != nil {
				t.Fatal(err)
			}
			result := s06Extract(t, local, string(raw), path)
			result.Path = path
			if err = encoder.Encode(result); err != nil {
				t.Fatal(err)
			}
		}) {
			if os.Getenv("S06_AUDIT_FAILURES") != "1" {
				t.FailNow()
			}
		}
	}
	fs := bundled.WrapFS(osvfs.FS())
	for _, name := range bundled.LibNames {
		path := bundled.LibPath() + "/" + name
		text, ok := fs.ReadFile(path)
		if !ok {
			t.Fatalf("missing bundled lib %s", name)
		}
		if err = encoder.Encode(map[string]any{"kind": "library", "path": "tsc/internal/bundled/libs/" + name, "filename": path, "text_hex": hex.EncodeToString([]byte(text))}); err != nil {
			t.Fatal(err)
		}
	}
}
