package compiler

// Access-only bridge to the actual pinned compiler file loader. No checker and
// no substitute list of files; processAllProgramFiles determines the closure.
import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/bundled"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnostics"
	"github.com/microsoft/TypeScript/tsc/internal/module"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
	"os"
	"sort"
	"testing"
)

type s07Request struct {
	ID                   string               `json:"id"`
	Cwd                  string               `json:"cwd"`
	CaseSensitive        bool                 `json:"case_sensitive"`
	Files                map[string]string    `json:"files"`
	Symlinks             map[string]string    `json:"symlinks"`
	Roots                []string             `json:"roots"`
	Options              core.CompilerOptions `json:"options"`
	SkipModuleResolution bool                 `json:"skip_module_resolution"`
}
type s07File struct {
	Name     string
	Path     string
	SHA256   string
	Bytes    int
	Meta     ast.SourceFileMetaData
	Lib      bool
	Imports  []string
	External bool
}
type s07Resolution struct {
	File   string
	Name   string
	Mode   core.ResolutionMode
	Result *s07Module
}
type s07Module struct {
	*module.ResolvedModule
	ResolutionDiagnostics []s07Diagnostic
}

func s07ModuleResult(result *module.ResolvedModule) *s07Module {
	if result == nil {
		return nil
	}
	value := &s07Module{ResolvedModule: result}
	for _, d := range result.ResolutionDiagnostics {
		value.ResolutionDiagnostics = append(value.ResolutionDiagnostics, s07Diag(d))
	}
	return value
}

type s07TypeResult struct {
	*module.ResolvedTypeReferenceDirective
	ResolutionDiagnostics []s07Diagnostic
}

func s07TypeReferenceResult(result *module.ResolvedTypeReferenceDirective) *s07TypeResult {
	if result == nil {
		return nil
	}
	value := &s07TypeResult{ResolvedTypeReferenceDirective: result}
	for _, d := range result.ResolutionDiagnostics {
		value.ResolutionDiagnostics = append(value.ResolutionDiagnostics, s07Diag(d))
	}
	return value
}

type s07TypeResolution struct {
	File   string
	Name   string
	Mode   core.ResolutionMode
	Result *s07TypeResult
}
type s07Diagnostic struct {
	File     string
	Pos      int
	End      int
	Code     int32
	Category int
	Key      string
	Args     []string
	Text     string
	Chain    []s07Diagnostic
	Related  []s07Diagnostic
}

func s07Diag(d *ast.Diagnostic) s07Diagnostic {
	r := s07Diagnostic{Pos: d.Pos(), End: d.End(), Code: d.Code(), Category: int(d.Category()), Key: string(d.MessageKey()), Args: d.MessageArgs(), Text: d.MessageText()}
	if d.File() != nil {
		r.File = d.File().FileName()
	}
	for _, c := range d.MessageChain() {
		r.Chain = append(r.Chain, s07Diag(c))
	}
	for _, c := range d.RelatedInformation() {
		r.Related = append(r.Related, s07Diag(c))
	}
	return r
}

type s07Observation struct {
	ID              string
	Files           []s07File
	Missing         []string
	Resolutions     []s07Resolution
	TypeResolutions []s07TypeResolution
	Diagnostics     []s07Diagnostic
	Trace           []string
	Panic           string `json:",omitempty"`
}

func TestS07Programs(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S07_PROGRAM_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []s07Request
	if err = json.Unmarshal(raw, &requests); err != nil {
		t.Fatal(err)
	}
	seen := map[string]bool{}
	for _, req := range requests {
		if req.ID == "" || seen[req.ID] {
			t.Fatalf("empty or duplicate request ID %q", req.ID)
		}
		seen[req.ID] = true
	}
	rows := []s07Observation{}
	for _, req := range requests {
		row := func() (row s07Observation) {
			row = s07Observation{ID: req.ID, Files: []s07File{}, Resolutions: []s07Resolution{}, TypeResolutions: []s07TypeResolution{}}
			defer func() {
				if value := recover(); value != nil {
					row.Panic = fmt.Sprint(value)
				}
			}()
			files := map[string]any{}
			for name, textHex := range req.Files {
				data, err := hex.DecodeString(textHex)
				if err != nil {
					t.Fatalf("%s: %s: %v", req.ID, name, err)
				}
				files[name] = data
			}
			for name, target := range req.Symlinks {
				if _, exists := files[name]; exists {
					t.Fatalf("%s: duplicate symlink/file %s", req.ID, name)
				}
				files[name] = vfstest.Symlink(target)
			}
			fs := bundled.WrapFS(vfstest.FromMap(files, req.CaseSensitive))
			host := NewCompilerHost(req.Cwd, fs, bundled.LibPath(), nil, func(msg *diagnostics.Message, args ...any) {
				row.Trace = append(row.Trace, fmt.Sprintf("%d:%v", msg.Code(), args))
			}, nil)
			config := tsoptions.NewParsedCommandLine(&req.Options, req.Roots, nil, tspath.ComparePathsOptions{CurrentDirectory: req.Cwd, UseCaseSensitiveFileNames: req.CaseSensitive})
			opts := ProgramOptions{Host: host, Config: config, SkipModuleResolution: req.SkipModuleResolution}
			loaded := processAllProgramFiles(opts, true)
			p := &Program{opts: opts, processedFiles: loaded}
			row.Missing = loaded.missingFiles
			for _, file := range loaded.files {
				hash := sha256.Sum256([]byte(file.Text()))
				f := s07File{Name: file.FileName(), Path: string(file.Path()), SHA256: hex.EncodeToString(hash[:]), Bytes: len(file.Text()), Meta: loaded.sourceFileMetaDatas[file.Path()], Lib: loaded.libFiles[file.Path()] != nil, Imports: []string{}, External: ast.IsExternalModule(file)}
				for _, node := range file.Imports() {
					f.Imports = append(f.Imports, node.Text())
				}
				row.Files = append(row.Files, f)
			}
			for path, resolutions := range loaded.resolvedModules {
				for key, result := range resolutions {
					row.Resolutions = append(row.Resolutions, s07Resolution{File: string(path), Name: key.Name, Mode: key.Mode, Result: s07ModuleResult(result)})
				}
			}
			for path, resolutions := range loaded.typeResolutionsInFile {
				for key, result := range resolutions {
					row.TypeResolutions = append(row.TypeResolutions, s07TypeResolution{File: string(path), Name: key.Name, Mode: key.Mode, Result: s07TypeReferenceResult(result)})
				}
			}
			sort.Slice(row.TypeResolutions, func(i, j int) bool {
				a, b := row.TypeResolutions[i], row.TypeResolutions[j]
				if a.File != b.File {
					return a.File < b.File
				}
				if a.Name != b.Name {
					return a.Name < b.Name
				}
				return a.Mode < b.Mode
			})
			sort.Slice(row.Resolutions, func(i, j int) bool {
				a, b := row.Resolutions[i], row.Resolutions[j]
				if a.File != b.File {
					return a.File < b.File
				}
				if a.Name != b.Name {
					return a.Name < b.Name
				}
				return a.Mode < b.Mode
			})
			for _, d := range loaded.includeProcessor.getDiagnostics(p).GetDiagnostics() {
				row.Diagnostics = append(row.Diagnostics, s07Diag(d))
			}
			return row
		}()
		rows = append(rows, row)
	}
	output, err := json.MarshalIndent(rows, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	output = append(output, '\n')
	if err = os.WriteFile(os.Getenv("S07_PROGRAM_OUTPUT"), output, 0600); err != nil {
		t.Fatal(err)
	}
}
