package checker_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"runtime"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/bundled"
	"github.com/microsoft/TypeScript/tsc/internal/checker"
	"github.com/microsoft/TypeScript/tsc/internal/compiler"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
)

func TestS08Contracts(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []struct {
		AllowJS   bool                        `json:"allow_js"`
		Files     map[string]string           `json:"files"`
		ID        string                      `json:"id"`
		SourceHex string                      `json:"source_hex"`
		Actions   []checker.S08RelationAction `json:"actions"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&requests); err != nil {
		t.Fatal(err)
	}
	if decoder.Decode(new(any)) != io.EOF {
		t.Fatal("trailing request")
	}
	rows := []map[string]any{}
	supplemental := map[string]any{}
	for index, request := range requests {
		row := map[string]any{"id": request.ID, "state": "not_executed"}
		t.Run(request.ID, func(t *testing.T) {
			defer checker.S08ResetObserver()
			defer func() {
				if value := recover(); value != nil {
					row["state"] = "panic"
					row["panic"] = fmt.Sprint(value)
					t.Error(value)
				}
			}()
			content, err := hex.DecodeString(request.SourceHex)
			if err != nil {
				t.Fatal(err)
			}
			files := map[string]string{"/fixture.ts": string(content)}
			for path, text := range request.Files {
				data, err := hex.DecodeString(text)
				if err != nil {
					t.Fatal(err)
				}
				files[path] = string(data)
			}
			fs := bundled.WrapFS(vfstest.FromMap(files, false))
			host := compiler.NewCompilerHost("/", fs, bundled.LibPath(), nil, nil, nil)
			options := &core.CompilerOptions{Target: core.ScriptTargetESNext, Module: core.ModuleKindESNext, Strict: core.TSTrue, SkipLibCheck: core.TSTrue}
			if request.AllowJS {
				options.AllowJs = core.TSTrue
			}
			config := tsoptions.NewParsedCommandLine(options, []string{"/fixture.ts"}, nil, tspath.ComparePathsOptions{})
			program := compiler.NewProgram(compiler.ProgramOptions{Config: config, Host: host})
			program.BindSourceFiles()
			file := program.GetSourceFile("/fixture.ts")
			if file == nil {
				t.Fatal("missing source")
			}
			// Independent checker per relation mode; shared bound inputs. Type lookup
			// is setup, then the immediate repeat sees the first call's resulting state.
			// The fixture spec distinguishes cold work from primitive shortcuts.
			groups := [][]checker.S08RelationAction{}
			for i := 0; i < len(request.Actions); {
				j := i + 1
				for j < len(request.Actions) && request.Actions[j].Mode == request.Actions[i].Mode {
					j++
				}
				groups = append(groups, request.Actions[i:j])
				i = j
			}
			checker.S08AssertModeCoverage(request.Actions)
			observed := []map[string]any{}
			for _, actions := range groups {
				c, _ := checker.NewChecker(program, nil)
				observed = append(observed, checker.S08ObserveRelation(c, file, actions))
			}
			// Diagnostics get another fresh checker; they cannot prewarm the relations.
			c, _ := checker.NewChecker(program, nil)
			diagnostics := c.GetDiagnostics(t.Context(), file)
			row["groups"] = observed
			row["diagnostics"] = checker.S08Diagnostics(diagnostics)
			row["global_diagnostics"] = checker.S08Diagnostics(c.GetGlobalDiagnostics())
			row["state"] = "executed"
			if index == 0 {
				c1, _ := checker.NewChecker(program, nil)
				c2, _ := checker.NewChecker(program, nil)
				supplemental["residuals"] = checker.S08Residuals(c1, c2, file)
				textRaw, err := os.ReadFile(os.Getenv("S08_TEXT_REQUESTS"))
				if err != nil {
					t.Fatal(err)
				}
				var textSpec struct {
					Version int                      `json:"version"`
					Cases   []checker.S08TextRequest `json:"cases"`
				}
				textDecoder := json.NewDecoder(bytes.NewReader(textRaw))
				textDecoder.DisallowUnknownFields()
				if err := textDecoder.Decode(&textSpec); err != nil {
					t.Fatal(err)
				}
				if textDecoder.Decode(new(any)) != io.EOF || textSpec.Version != 1 {
					t.Fatal("invalid text protocol")
				}
				textChecker, _ := checker.NewChecker(program, nil)
				supplemental["text"] = checker.S08Text(textChecker, textSpec.Cases)
			}
		})
		rows = append(rows, row)
	}
	hash := sha256.Sum256(raw)
	out, err := json.Marshal(map[string]any{"request_sha256": hex.EncodeToString(hash[:]), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH, "rows": rows, "supplemental": supplemental})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_OUTPUT"), out, 0600); err != nil {
		t.Fatal(err)
	}
}
