package testrunner

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"strings"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnosticwriter"
	"github.com/microsoft/TypeScript/tsc/internal/repo"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/baseline"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/tsbaseline"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
)

type s08Request struct {
	ID                string            `json:"id"`
	Path              string            `json:"path"`
	RawSHA256         string            `json:"raw_sha256"`
	LoadedSHA256      string            `json:"loaded_sha256"`
	Settings          map[string]string `json:"settings"`
	ConfigurationName string            `json:"configuration_name"`
	ConfiguredName    string            `json:"configured_name"`
	AcceptanceTier    string            `json:"acceptance_tier"`
}

func s08Hash(raw []byte) string { value := sha256.Sum256(raw); return hex.EncodeToString(value[:]) }

func s08Diagnostics(values []*ast.Diagnostic) []map[string]any {
	result := make([]map[string]any, 0, len(values))
	for _, d := range values {
		var file any
		if d.File() != nil {
			file = d.File().FileName()
		}
		result = append(result, map[string]any{
			"file": file, "pos": d.Pos(), "end": d.End(), "code": d.Code(), "category": d.Category(),
			"key_hex": hex.EncodeToString([]byte(d.MessageKey())), "text_hex": hex.EncodeToString([]byte(d.MessageText())),
			"source_hex": hex.EncodeToString([]byte(d.Source())), "args": d.MessageArgs(),
			"chain": s08Diagnostics(d.MessageChain()), "related": s08Diagnostics(d.RelatedInformation()),
			"unnecessary": d.ReportsUnnecessary(), "deprecated": d.ReportsDeprecated(), "skipped_on_no_emit": d.SkippedOnNoEmit(),
		})
	}
	return result
}

func TestS08Baselines(t *testing.T) {
	// Empty content is an observed file, distinct from the source's NoContent
	// sentinel and a baseline disabled by harness policy. Preserve arbitrary bytes.
	for _, fixture := range []struct{ value, expected string }{
		{"", `{"state":"content","text_hex":""}`},
		{baseline.NoContent, `{"state":"no_content"}`},
		{"\xff", `{"state":"content","text_hex":"ff"}`},
	} {
		encoded, err := json.Marshal(tsbaseline.S08BaselineValue(fixture.value))
		if err != nil || string(encoded) != fixture.expected {
			t.Fatalf("baseline outcome contract: %q, %v", encoded, err)
		}
	}
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	var requests []s08Request
	if err := decoder.Decode(&requests); err != nil {
		t.Fatal(err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		t.Fatalf("trailing input: %v", err)
	}
	seen := map[string]bool{}
	for _, request := range requests {
		if request.ID == "" || seen[request.ID] || (request.AcceptanceTier != "acceptance" && request.AcceptanceTier != "informational") {
			t.Fatal("empty/duplicate request")
		}
		seen[request.ID] = true
	}
	output, err := os.Create(os.Getenv("S08_OUTPUT"))
	if err != nil {
		t.Fatal(err)
	}
	defer output.Close()
	encoder := json.NewEncoder(output)
	for _, request := range requests {
		row := map[string]any{"id": request.ID, "acceptance_tier": request.AcceptanceTier, "state": "not_executed", "queries": []tsbaseline.S08Query{}}
		complete := false
		t.Run(request.ID, func(t *testing.T) {
			defer func() {
				if value := recover(); value != nil {
					row["panic"] = fmt.Sprint(value)
					t.Error("upstream panic", value)
				}
				if t.Failed() {
					row["state"] = "upstream_failed"
				} else if !complete {
					if t.Skipped() {
						row["state"] = "upstream_skipped"
					} else {
						row["state"] = "upstream_failed"
					}
				}
				harnessutil.S08ObserveDiagnostics = nil
				tsbaseline.S08Queries = nil
			}()
			path := filepath.Join(repo.RootPath(), strings.TrimPrefix(request.Path, "tsc/"))
			original, err := os.ReadFile(path)
			if err != nil {
				t.Fatal(err)
			}
			if s08Hash(original) != request.RawSHA256 {
				t.Fatal("physical source digest differs")
			}
			loaded, ok := osvfs.FS().ReadFile(path)
			if !ok || s08Hash([]byte(loaded)) != request.LoadedSHA256 {
				t.Fatal("loaded source digest differs")
			}
			row["raw_sha256"] = s08Hash(original)
			row["loaded_sha256"] = s08Hash([]byte(loaded))
			row["native_runner_skipped"] = slices.Contains(skippedTests, tspath.GetBaseFileName(path))
			harnessutil.S08ObserveDiagnostics = func(pre, post []*ast.Diagnostic) {
				row["pre_diagnostics"] = s08Diagnostics(pre)
				row["post_diagnostics"] = s08Diagnostics(post)
			}
			payload := makeUnitsFromTest(loaded, path)
			configuration := &harnessutil.NamedTestConfiguration{Config: request.Settings, Name: request.ConfigurationName}
			c := newCompilerTest(t, request.ID, path, &payload, configuration)
			if c.configuredName != request.ConfiguredName {
				t.Fatal("configured name drift", c.configuredName)
			}
			row["options"] = c.options
			row["harness_options"] = c.harnessOptions
			if request.AcceptanceTier != "informational" || os.Getenv("S08_INCLUDE_INFORMATIONAL") != "1" {
				harnessutil.SkipUnsupportedCompilerOptions(t, c.options)
			}
			files := make([]map[string]any, 0)
			for _, f := range c.result.Program.GetSourceFiles() {
				files = append(files, map[string]any{"name": f.FileName(), "path": string(f.Path()), "sha256": s08Hash([]byte(f.Text())), "bytes": len(f.Text())})
			}
			row["files"] = files
			row["diagnostics"] = s08Diagnostics(c.result.Diagnostics)
			errorValue := baseline.NoContent
			if len(c.result.Diagnostics) > 0 {
				errorValue = tsbaseline.GetErrorBaseline(t, core.Concatenate(c.tsConfigFiles, core.Concatenate(c.toBeCompiled, c.otherFiles)), diagnosticwriter.WrapASTDiagnostics(c.result.Diagnostics), diagnosticwriter.CompareASTDiagnostics, c.options.Pretty.IsTrue())
			}
			row["errors"] = tsbaseline.S08BaselineValue(errorValue)
			for _, d := range c.result.Diagnostics {
				if d.Code() == -1 {
					t.Fatal("source diagnostic assertion (-1)")
				}
			}
			row["types"] = tsbaseline.S08Baseline{State: "disabled"}
			row["symbols"] = tsbaseline.S08Baseline{State: "disabled"}
			if !c.harnessOptions.NoTypesAndSymbols {
				allFiles := core.Filter(core.Concatenate(c.toBeCompiled, c.otherFiles), func(f *harnessutil.TestFile) bool { return c.result.Program.GetSourceFile(f.UnitName) != nil })
				header := tspath.GetPathFromPathComponents(tspath.GetPathComponentsRelativeTo(repo.TestDataPath(), path, tspath.ComparePathsOptions{}))
				tsbaseline.S08Queries = []tsbaseline.S08Query{}
				row["types"], row["symbols"] = tsbaseline.S08TypeSymbolBaselines(c.result.Program, allFiles, header, len(c.result.Diagnostics) > 0)
				row["queries"] = tsbaseline.S08Queries
			}
			row["state"] = "executed"
			complete = true
		})
		if err := encoder.Encode(row); err != nil {
			t.Fatal(err)
		}
	}
	summary, err := json.Marshal(map[string]any{"request_sha256": s08Hash(raw), "rows": len(requests), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_SUMMARY"), summary, 0600); err != nil {
		t.Fatal(err)
	}
}
