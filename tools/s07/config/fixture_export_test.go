package testrunner

import (
	"encoding/hex"
	"encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"
)

type s07FixtureSelector struct {
	ID            string `json:"id"`
	Path          string `json:"path"`
	Configuration int    `json:"configuration"`
}

func TestS07ConfigFixtures(t *testing.T) {
	bytes, err := os.ReadFile(os.Getenv("S07_CONFIG_SELECTORS"))
	if err != nil {
		t.Fatal(err)
	}
	var selectors []s07FixtureSelector
	if err = json.Unmarshal(bytes, &selectors); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	requests := []any{}
	seen := map[string]bool{}
	var previous string
	var extracted s06Case
	var loaded string
	var physicalHex string
	for _, selector := range selectors {
		if seen[selector.ID] || selector.ID == "" {
			t.Fatal("duplicate/empty selector")
		}
		seen[selector.ID] = true
		if !(strings.HasPrefix(selector.Path, "tsc/testdata/tests/cases/compiler/") || strings.HasPrefix(selector.Path, "tsc/testdata/tests/cases/conformance/")) || filepath.ToSlash(filepath.Clean(selector.Path)) != selector.Path {
			t.Fatal("invalid physical path")
		}
		local, err := filepath.Abs(filepath.Join("..", "..", "..", filepath.FromSlash(selector.Path)))
		if err != nil {
			t.Fatal(err)
		}
		if previous != selector.Path {
			physical, err := os.ReadFile(local)
			if err != nil {
				t.Fatal(err)
			}
			physicalHex = hex.EncodeToString(physical)
			var ok bool
			loaded, ok = osvfs.FS().ReadFile(local)
			if !ok {
				t.Fatal("physical read failed")
			}
			extracted = s06Extract(t, local, string(physical), selector.Path)
			previous = selector.Path
		}
		if selector.Configuration < 0 || selector.Configuration >= len(extracted.Configurations) {
			t.Fatal("invalid configuration index")
		}
		settings := extracted.Configurations[selector.Configuration]
		cwd := tspath.GetNormalizedAbsolutePath(settings["currentdirectory"], srcFolder)
		payload := makeUnitsFromTest(loaded, local)
		options := &core.CompilerOptions{}
		var raw any
		var compileOnSave *bool
		diagnostics := []*ast.Diagnostic{}
		if payload.tsConfig != nil {
			options = payload.tsConfig.CompilerOptions().Clone()
			raw = payload.tsConfig.Raw
			compileOnSave = payload.tsConfig.CompileOnSave
			diagnostics = payload.tsConfig.Errors
		}
		fixtureSettings := map[string]string{}
		for key, value := range settings {
			fixtureSettings[key] = value
		}
		if base, ok := fixtureSettings["baseurl"]; payload.tsConfig == nil && ok && !tspath.IsRootedDiskPath(base) {
			fixtureSettings["baseurl"] = tspath.GetNormalizedAbsolutePath(base, cwd)
		}
		_, optionDiagnostics := harnessutil.S07SubsetOptions(t, fixtureSettings, options, cwd)
		for _, path := range []*string{&options.OutDir, &options.Project, &options.RootDir, &options.TsBuildInfoFile, &options.BaseUrl, &options.DeclarationDir} {
			if *path != "" {
				*path = tspath.GetNormalizedAbsolutePath(*path, cwd)
			}
		}
		for i, path := range options.RootDirs {
			options.RootDirs[i] = tspath.GetNormalizedAbsolutePath(path, cwd)
		}
		for i, path := range options.TypeRoots {
			options.TypeRoots[i] = tspath.GetNormalizedAbsolutePath(path, cwd)
		}
		roots := []string{}
		units := payload.testUnitData
		lastOnly := false
		if len(units) > 0 && payload.tsConfig == nil {
			last := units[len(units)-1]
			lastOnly = settings["noimplicitreferences"] != "" || strings.Contains(last.content, requireStr) || referencesRegex.MatchString(last.content)
		}
		for i, unit := range units {
			name := tspath.GetNormalizedAbsolutePath(unit.name, cwd)
			root := !lastOnly || i == len(units)-1
			if payload.tsConfig != nil {
				root = slices.Contains(payload.tsConfig.FileNames(), name)
			}
			if root && !tspath.FileExtensionIs(name, tspath.ExtensionJson) && !tspath.FileExtensionIs(name, tspath.ExtensionTsBuildInfo) {
				roots = append(roots, hex.EncodeToString([]byte(name)))
			}
		}
		configDiagnostics := []any{}
		for _, diag := range diagnostics {
			configDiagnostics = append(configDiagnostics, s07ConfigDiag(diag))
		}
		optionDiags := []any{}
		for _, diag := range optionDiagnostics {
			optionDiags = append(optionDiags, s07ConfigDiag(diag))
		}
		rows = append(rows, map[string]any{"id": selector.ID, "options": s07ConfigOptions(options), "root_file_names": roots, "config_raw": s07ConfigValue(raw), "config_diagnostics": configDiagnostics, "option_diagnostics": optionDiags, "compile_on_save": compileOnSave})
		configCwd := extracted.CurrentDirectory
		if configCwd == "" {
			configCwd = srcFolder
		}
		requests = append(requests, map[string]any{"id": selector.ID, "path": selector.Path, "configuration": selector.Configuration, "raw_sha256": extracted.RawSHA256, "physical_hex": physicalHex, "loaded_sha256": extracted.LoadedSHA256, "cwd": cwd, "config_cwd": configCwd, "case_sensitive": true, "settings": settings, "units": extracted.Units, "symlinks": extracted.Symlinks, "run_external_code": extracted.GlobalOptions["runexternalcode"] == "true"})
	}
	for path, value := range map[string]any{os.Getenv("S07_CONFIG_OUTPUT"): rows, os.Getenv("S07_CONFIG_SOURCE_REQUESTS"): requests} {
		data, err := json.Marshal(value)
		if err != nil {
			t.Fatal(err)
		}
		if err = os.WriteFile(path, append(data, '\n'), 0600); err != nil {
			t.Fatal(err)
		}
	}
}
