package testrunner

// Source-only classification observations. This adapter calls the frozen S06
// physical/virtual loading boundary and the original Go parser. It does not bind
// or check, and does not read any Rust output.
import (
	"encoding/hex"
	"encoding/json"
	goast "go/ast"
	goparser "go/parser"
	"go/token"
	"os"
	"path/filepath"
	"reflect"
	"slices"
	"strconv"
	"strings"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/bundled"
	"github.com/microsoft/TypeScript/tsc/internal/collections"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
)

type s07Diagnostic struct {
	Code    int32  `json:"code"`
	Start   int    `json:"start"`
	End     int    `json:"end"`
	Message string `json:"message"`
}

func s07Diagnostics(ds []*ast.Diagnostic) []s07Diagnostic {
	r := make([]s07Diagnostic, 0, len(ds))
	for _, d := range ds {
		r = append(r, s07Diagnostic{d.Code(), d.Pos(), d.End(), d.MessageText()})
	}
	return r
}

type s07Syntax struct {
	Kinds          map[string]int       `json:"kinds"`
	FirstPositions map[string]int       `json:"first_positions"`
	TypeArguments  []int                `json:"nonempty_type_arguments"`
	Diagnostics    []s07Diagnostic      `json:"diagnostics"`
	References     []*ast.FileReference `json:"references"`
	TypeReferences []*ast.FileReference `json:"type_references"`
	LibReferences  []*ast.FileReference `json:"lib_references"`
	Pragmas        []ast.Pragma         `json:"pragmas"`
	Declaration    bool                 `json:"declaration"`
}

func s07ObserveSyntax(file *ast.SourceFile) s07Syntax {
	r := s07Syntax{Kinds: map[string]int{}, FirstPositions: map[string]int{}, TypeArguments: []int{}, References: file.ReferencedFiles, TypeReferences: file.TypeReferenceDirectives, LibReferences: file.LibReferenceDirectives, Pragmas: file.Pragmas, Declaration: file.IsDeclarationFile}
	seen := map[*ast.Node]bool{}
	var visit func(*ast.Node) bool
	visit = func(node *ast.Node) bool {
		if node == nil || seen[node] {
			return false
		}
		seen[node] = true
		kind := node.Kind.String()
		if r.Kinds[kind] == 0 {
			r.FirstPositions[kind] = node.Pos()
		}
		r.Kinds[kind]++
		switch node.Kind {
		case ast.KindCallExpression, ast.KindNewExpression, ast.KindTaggedTemplateExpression, ast.KindTypeReference, ast.KindExpressionWithTypeArguments, ast.KindImportType, ast.KindTypeQuery, ast.KindJsxOpeningElement, ast.KindJsxSelfClosingElement:
			if len(node.TypeArguments()) != 0 {
				r.TypeArguments = append(r.TypeArguments, node.Pos())
			}
		}
		node.ForEachChild(visit)
		for _, doc := range node.JSDoc(file) {
			visit(doc)
		}
		return false
	}
	visit(file.AsNode())
	r.Diagnostics = s07Diagnostics(file.Diagnostics())
	return r
}

type s07UnitSyntax struct {
	Unit   int       `json:"unit"`
	Route  string    `json:"route"`
	Syntax s07Syntax `json:"syntax"`
}
type s07LoadRequest struct {
	ID                   string                `json:"id"`
	Cwd                  string                `json:"cwd"`
	CaseSensitive        bool                  `json:"case_sensitive"`
	Files                map[string]string     `json:"files"`
	Symlinks             map[string]string     `json:"symlinks"`
	Roots                []string              `json:"roots"`
	Options              *core.CompilerOptions `json:"options"`
	SkipModuleResolution bool                  `json:"skip_module_resolution"`
}
type s07Variant struct {
	Configuration     int                         `json:"configuration"`
	ConfiguredName    string                      `json:"configured_name"`
	Syntax            []s07UnitSyntax             `json:"syntax"`
	Request           s07LoadRequest              `json:"request"`
	OptionDiagnostics []s07Diagnostic             `json:"option_diagnostics"`
	ConfigDiagnostics []s07Diagnostic             `json:"config_diagnostics"`
	ConfigOptions     *core.CompilerOptions       `json:"config_options"`
	ConfigRaw         any                         `json:"config_raw"`
	ConfigOptionKeys  []string                    `json:"config_option_keys"`
	ConfigRootKeys    []string                    `json:"config_root_keys"`
	ExtendedConfigs   []string                    `json:"extended_configs"`
	HarnessOptions    *harnessutil.HarnessOptions `json:"harness_options"`
	ProjectReferences bool                        `json:"project_references"`
	ContentMappers    bool                        `json:"content_mappers"`
}

func s07Variants(t *testing.T, c *s06Case, physical, loaded string) []s07Variant {
	payload := makeUnitsFromTest(loaded, physical)
	settings := extractCompilerSettings(loaded)
	if c.LegacyProjection != nil {
		delete(settings, "module")
	}
	names := harnessutil.GetFileBasedTestConfigurations(t, settings, compilerVaryBy)
	for _, name := range names {
		if c.LegacyProjection != nil {
			name.Config["module"] = "none"
		}
	}
	rows := make([]s07Variant, 0, len(c.Variants))
	for ci, v := range c.Variants {
		config := c.Configurations[ci]
		r := s07Variant{Configuration: ci, ConfiguredName: tspath.GetBaseFileName(c.Path), Syntax: []s07UnitSyntax{}, ConfigDiagnostics: []s07Diagnostic{}}
		for _, name := range names {
			if reflect.DeepEqual(name.Config, config) && name.Name != "" {
				base := r.ConfiguredName
				ext := tspath.GetAnyExtensionFromPath(base, nil, false)
				r.ConfiguredName = base[:len(base)-len(ext)] + "(" + name.Name + ")" + ext
				break
			}
		}
		options := &core.CompilerOptions{}
		if payload.tsConfig != nil {
			options = payload.tsConfig.CompilerOptions().Clone()
			r.ConfigOptions = options.Clone()
			r.ConfigRaw = payload.tsConfig.Raw
			r.ExtendedConfigs = payload.tsConfig.ExtendedSourceFiles()
			if raw, ok := payload.tsConfig.Raw.(*collections.OrderedMap[string, any]); ok {
				r.ConfigRootKeys = slices.Collect(raw.Keys())
				slices.Sort(r.ConfigRootKeys)
				if compilerOptions, ok := raw.GetOrZero("compilerOptions").(*collections.OrderedMap[string, any]); ok {
					r.ConfigOptionKeys = slices.Collect(compilerOptions.Keys())
					slices.Sort(r.ConfigOptionKeys)
				}
			}
			r.ConfigDiagnostics = s07Diagnostics(payload.tsConfig.Errors)
			r.ProjectReferences = len(payload.tsConfig.ProjectReferences()) != 0
			r.ContentMappers = len(payload.tsConfig.ContentMappers()) != 0
		}
		// newCompilerTest normalizes a relative baseUrl before harness parsing.
		fixtureSettings := map[string]string{}
		for k, value := range config {
			fixtureSettings[k] = value
		}
		if base, ok := fixtureSettings["baseurl"]; payload.tsConfig == nil && ok && !tspath.IsRootedDiskPath(base) {
			fixtureSettings["baseurl"] = tspath.GetNormalizedAbsolutePath(base, v.CurrentDirectory)
		}
		harness, errors := harnessutil.S07SubsetOptions(t, fixtureSettings, options, v.CurrentDirectory)
		// CompileFilesEx:159-186 normalizes these path-valued fixture options
		// after SetOptionsFromTestConfig, including options originating in JSON.
		for _, path := range []*string{&options.OutDir, &options.Project, &options.RootDir, &options.TsBuildInfoFile, &options.BaseUrl, &options.DeclarationDir} {
			if *path != "" {
				*path = tspath.GetNormalizedAbsolutePath(*path, v.CurrentDirectory)
			}
		}
		for i, path := range options.RootDirs {
			options.RootDirs[i] = tspath.GetNormalizedAbsolutePath(path, v.CurrentDirectory)
		}
		for i, path := range options.TypeRoots {
			options.TypeRoots[i] = tspath.GetNormalizedAbsolutePath(path, v.CurrentDirectory)
		}
		r.HarnessOptions = harness
		r.OptionDiagnostics = s07Diagnostics(errors)
		r.Request = s07LoadRequest{Cwd: v.CurrentDirectory, CaseSensitive: harness.UseCaseSensitiveFileNames, Files: map[string]string{}, Symlinks: map[string]string{}, Roots: []string{}, Options: options}
		// Input assembly is the pinned compiler-runner fixture boundary at
		// compiler_runner.go:296-344; the compiler loader itself runs separately.
		units := payload.testUnitData
		lastOnly := false
		includeFixtureLib := false
		if len(units) != 0 && payload.tsConfig == nil {
			last := units[len(units)-1]
			lastOnly = config["noimplicitreferences"] != "" || strings.Contains(last.content, requireStr) || referencesRegex.MatchString(last.content)
		}
		for i, unit := range units {
			name := tspath.GetNormalizedAbsolutePath(unit.name, v.CurrentDirectory)
			r.Request.Files[name] = hex.EncodeToString([]byte(unit.content))
			root := !lastOnly || i == len(units)-1
			if payload.tsConfig != nil {
				root = slices.Contains(payload.tsConfig.ParsedConfig.FileNames, name)
			}
			if root && strings.Contains(unit.content, "/.lib/") {
				includeFixtureLib = true
			}
			if root && !tspath.FileExtensionIs(name, tspath.ExtensionJson) && !tspath.FileExtensionIs(name, tspath.ExtensionTsBuildInfo) {
				r.Request.Roots = append(r.Request.Roots, name)
			}
		}
		if includeFixtureLib {
			for name, bytes := range harnessutil.S07SubsetLibFiles() {
				r.Request.Files[name] = bytes
			}
		}
		for from, to := range payload.symlinks {
			r.Request.Symlinks[tspath.GetNormalizedAbsolutePath(from, v.CurrentDirectory)] = tspath.GetNormalizedAbsolutePath(to, v.CurrentDirectory)
		}
		for _, input := range v.Inputs {
			bytes, err := hex.DecodeString(input.TextHex)
			if err != nil {
				t.Fatal(err)
			}
			file := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: input.Filename, Path: tspath.Path(input.Path), ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: input.JSX, Force: input.Force}}, string(bytes), core.ScriptKind(input.ScriptKind))
			r.Syntax = append(r.Syntax, s07UnitSyntax{input.Unit, input.Route, s07ObserveSyntax(file)})
		}
		rows = append(rows, r)
	}
	return rows
}

func TestS07SubsetExport(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S07_SUBSET_INPUT"))
	if err != nil {
		t.Fatal(err)
	}
	var paths []string
	if err = json.Unmarshal(raw, &paths); err != nil {
		t.Fatal(err)
	}
	out, err := os.Create(os.Getenv("S07_SUBSET_OUTPUT"))
	if err != nil {
		t.Fatal(err)
	}
	defer out.Close()
	encoder := json.NewEncoder(out)
	kinds := []string{}
	for kind := ast.KindUnknown; kind < ast.KindCount; kind++ {
		kinds = append(kinds, kind.String())
	}
	optionKeys := []string{}
	optionType := reflect.TypeFor[core.CompilerOptions]()
	for i := 0; i < optionType.NumField(); i++ {
		tag := optionType.Field(i).Tag.Get("json")
		if tag != "" {
			optionKeys = append(optionKeys, strings.Split(tag, ",")[0])
		}
	}
	// The pragma names are literal equality guards in the pinned parser's
	// extractPragmas function. Inspect its Go AST, rather than inventing a list
	// from the observed corpus (which might not exercise every accepted pragma).
	source, err := goparser.ParseFile(token.NewFileSet(), "../parser/parser.go", nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	pragmaSet := map[string]bool{}
	for _, decl := range source.Decls {
		function, ok := decl.(*goast.FuncDecl)
		if !ok || function.Name.Name != "extractPragmas" {
			continue
		}
		goast.Inspect(function.Body, func(node goast.Node) bool {
			comparison, ok := node.(*goast.BinaryExpr)
			if !ok || (comparison.Op != token.EQL && comparison.Op != token.NEQ) {
				return true
			}
			name, ok := comparison.X.(*goast.Ident)
			if !ok || (name.Name != "pragmaName" && name.Name != "tagName") {
				return true
			}
			literal, ok := comparison.Y.(*goast.BasicLit)
			if !ok || literal.Kind != token.STRING {
				return true
			}
			nameValue, err := strconv.Unquote(literal.Value)
			if err != nil {
				t.Fatal(err)
			}
			pragmaSet[nameValue] = true
			return true
		})
	}
	pragmaNames := make([]string, 0, len(pragmaSet))
	for name := range pragmaSet {
		pragmaNames = append(pragmaNames, name)
	}
	slices.Sort(pragmaNames)
	if err = encoder.Encode(map[string]any{"record_kind": "syntax_kinds", "kinds": kinds, "option_keys": optionKeys, "pragma_names": pragmaNames}); err != nil {
		t.Fatal(err)
	}
	for _, path := range paths {
		if !t.Run(path, func(t *testing.T) {
			physical, err := filepath.Abs(filepath.Join("..", "..", "..", filepath.FromSlash(path)))
			if err != nil {
				t.Fatal(err)
			}
			raw, err := os.ReadFile(physical)
			if err != nil {
				t.Fatal(err)
			}
			loaded, ok := osvfs.FS().ReadFile(physical)
			if !ok {
				t.Fatal("missing physical case")
			}
			c := s06Extract(t, physical, string(raw), path)
			if err = encoder.Encode(map[string]any{"record_kind": "case", "case": c, "variants": s07Variants(t, &c, physical, loaded)}); err != nil {
				t.Fatal(err)
			}
		}) {
			t.FailNow()
		}
	}
	fs := bundled.WrapFS(osvfs.FS())
	for _, name := range bundled.LibNames {
		path := bundled.LibPath() + "/" + name
		text, ok := fs.ReadFile(path)
		if !ok {
			t.Fatal("missing library")
		}
		file := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: path, Path: tspath.Path(path)}, text, core.ScriptKindTS)
		if err = encoder.Encode(map[string]any{"record_kind": "library", "path": "tsc/internal/bundled/libs/" + name, "filename": path, "text_hex": hex.EncodeToString([]byte(text)), "syntax": s07ObserveSyntax(file)}); err != nil {
			t.Fatal(err)
		}
	}
	fixtureLibs := harnessutil.S07SubsetLibFiles()
	fixtureNames := make([]string, 0, len(fixtureLibs))
	files := map[string][]byte{}
	for name, text := range fixtureLibs {
		fixtureNames = append(fixtureNames, name)
		bytes, err := hex.DecodeString(text)
		if err != nil {
			t.Fatal(err)
		}
		files[name] = bytes
	}
	slices.Sort(fixtureNames)
	fixtureFS := vfstest.FromMap(files, true)
	for _, name := range fixtureNames {
		text, ok := fixtureFS.ReadFile(name)
		if !ok {
			t.Fatal("missing fixture library")
		}
		file := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: name, Path: tspath.Path(name)}, text, core.ScriptKindTS)
		if err = encoder.Encode(map[string]any{"record_kind": "fixture_library", "path": "tsc/testdata/tests/lib/" + strings.TrimPrefix(name, "/.lib/"), "filename": name, "raw_hex": fixtureLibs[name], "text_hex": hex.EncodeToString([]byte(text)), "syntax": s07ObserveSyntax(file)}); err != nil {
			t.Fatal(err)
		}
	}
}
