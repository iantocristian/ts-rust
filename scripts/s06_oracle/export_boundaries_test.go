package testrunner

import (
	"encoding/hex"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

func s06TestInput(t *testing.T, source, logical string) s06Case {
	t.Helper()
	file := filepath.Join(t.TempDir(), "case.ts")
	if err := os.WriteFile(file, []byte(source), 0o600); err != nil {
		t.Fatal(err)
	}
	return s06Extract(t, file, source, logical)
}

func TestS06LoadingBoundaries(t *testing.T) {
	for _, test := range []struct{ name, source, extracted, final, route string }{
		{"default", "let x;", "let x;", "let x;", "virtual_file"},
		{"double-bom", "\xef\xbb\xbf\xef\xbb\xbflet x;", "\xef\xbb\xbflet x;", "let x;", "virtual_file"},
		{"virtual-bom", "// @filename: a.ts\n\xef\xbb\xbflet x;", "\xef\xbb\xbflet x;", "let x;", "virtual_file"},
		{"config-direct", "// @filename: tsconfig.json\n\xef\xbb\xbf{}", "\xef\xbb\xbf{}", "\xef\xbb\xbf{}", "initial_config_direct"},
		{"newlines", "\n\r\nlet x;\r\n", "let x;\n", "let x;\n", "virtual_file"},
	} {
		t.Run(test.name, func(t *testing.T) {
			got := s06TestInput(t, test.source, "fixture")
			if len(got.Configurations) != 1 || len(got.Units) != 1 || len(got.Variants) != 1 || len(got.Variants[0].Inputs) != 1 {
				t.Fatal("default variant or unit missing")
			}
			input := got.Variants[0].Inputs[0]
			if got.Units[0].TextHex != hex.EncodeToString([]byte(test.extracted)) || input.TextHex != hex.EncodeToString([]byte(test.final)) || input.Route != test.route {
				t.Fatalf("wrong loader observation: %#v / %#v", got.Units[0], input)
			}
		})
	}
}

func TestS06ConfigurationAndPackageMetadata(t *testing.T) {
	source := "// @target: es6,es2015,es2020\n// @module: nodenext;\n// @moduleDetection: auto\n// @filename: /package.json\n{\"type\":\"module\"}\n// @filename: /a.js\nlet x;"
	got := s06TestInput(t, source, "fixture")
	if len(got.Configurations) != 2 || got.RawSettings["module"] != "nodenext" || got.GlobalOptions["module"] != "nodenext;" {
		t.Fatal("alias deduplication or semicolon distinction changed")
	}
	for _, variant := range got.Variants {
		if !variant.Inputs[1].Force || variant.Inputs[1].Metadata.PackageJsonType != "module" {
			t.Fatal("package format did not reach actual parse options")
		}
		found := false
		for _, read := range variant.Reads {
			if read.Path == "/package.json" && read.Exists {
				found = true
			}
		}
		if !found {
			t.Fatal("missing package metadata input")
		}
	}
}

func TestS06LegacyProjectionRetainsRejectedSetting(t *testing.T) {
	got := s06TestInput(t, "// @module: none\n// @target: es5,es2015\nlet x;", s06LegacyPaths[0])
	if got.LegacyProjection == nil || got.RawSettings["module"] != "none" || len(got.Configurations) != 2 {
		t.Fatal("legacy setting or variants lost")
	}
	for _, variant := range got.Variants {
		if variant.ParseSettings["module"] != "none" || len(variant.OptionDiagnostics) != 1 || variant.OptionDiagnostics[0].Code != 6046 {
			t.Fatal("missing native rejected-option outcome")
		}
	}
}

func TestS06FailureProbe(t *testing.T) {
	switch os.Getenv("S06_FAILURE_PROBE") {
	case "unreviewed":
		s06TestInput(t, "// @module: none\nlet x;", "tsc/testdata/tests/cases/compiler/eighth.ts")
	case "changed":
		s06TestInput(t, "// @module: commonjs\nlet x;", s06LegacyPaths[0])
	default:
		t.Skip("negative subprocess witness")
	}
	t.Fatal("invalid projection unexpectedly accepted")
}

func TestS06RejectsUnreviewedLegacyProjection(t *testing.T) {
	for mode, message := range map[string]string{"unreviewed": "Unknown value 'none' for option 'module'", "changed": "requires original module exactly none"} {
		cmd := exec.Command(os.Args[0], "-test.run=^TestS06FailureProbe$", "-test.timeout=20s")
		cmd.Env = append(os.Environ(), "S06_FAILURE_PROBE="+mode)
		output, err := cmd.CombinedOutput()
		if err == nil || !strings.Contains(string(output), message) || strings.Contains(string(output), "unexpectedly accepted") {
			t.Fatalf("negative probe %s did not fail for the expected reason: %s (%v)", mode, output, err)
		}
	}
}
