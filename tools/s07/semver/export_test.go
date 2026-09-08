package semver

// Access-only observation of the unchanged pinned semver implementation.
import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"slices"
	"strconv"
	"testing"
)

type s07Input struct {
	Hex     string   `json:"hex"`
	Origins []string `json:"origins"`
}
type s07Version struct {
	Major      uint32   `json:"major"`
	Minor      uint32   `json:"minor"`
	Patch      uint32   `json:"patch"`
	Prerelease []string `json:"prerelease"`
	Build      []string `json:"build"`
	Display    string   `json:"display"`
	Error      string   `json:"error"`
	ErrorType  string   `json:"error_type"`
	Compare    string   `json:"compare"`
}
type s07Range struct {
	OK      bool   `json:"ok"`
	Display string `json:"display"`
	Matches string `json:"matches"`
}

func TestS07Semver(t *testing.T) {
	inputs := map[string][]string{}
	for _, name := range []string{"version_test.go", "version_range_test.go"} {
		fset := token.NewFileSet()
		file, err := parser.ParseFile(fset, name, nil, 0)
		if err != nil {
			t.Fatal(err)
		}
		ast.Inspect(file, func(node ast.Node) bool {
			literal, ok := node.(*ast.BasicLit)
			if !ok || literal.Kind != token.STRING {
				return true
			}
			value, err := strconv.Unquote(literal.Value)
			if err != nil {
				t.Fatal(err)
			}
			inputs[value] = append(inputs[value], fmt.Sprintf("%s:%d", name, fset.Position(literal.Pos()).Line))
			return true
		})
	}
	raw, err := os.ReadFile(os.Getenv("S07_SEMVER_SUPPLEMENTAL"))
	if err != nil {
		t.Fatal(err)
	}
	var supplemental []s07Input
	if err := json.Unmarshal(raw, &supplemental); err != nil {
		t.Fatal(err)
	}
	for _, row := range supplemental {
		decoded, err := hex.DecodeString(row.Hex)
		if err != nil {
			t.Fatal(err)
		}
		inputs[string(decoded)] = append(inputs[string(decoded)], row.Origins...)
	}
	keys := make([]string, 0, len(inputs))
	for value := range inputs {
		keys = append(keys, value)
	}
	slices.Sort(keys)
	requests := make([]s07Input, 0, len(keys))
	versions := make([]Version, len(keys))
	versionRows := make([]s07Version, len(keys))
	rangeRows := make([]s07Range, len(keys))
	for i, value := range keys {
		requests = append(requests, s07Input{hex.EncodeToString([]byte(value)), inputs[value]})
		version, err := TryParseVersion(value)
		versions[i] = version
		row := s07Version{Major: version.major, Minor: version.minor, Patch: version.patch, Prerelease: version.prerelease, Build: version.build, Display: version.String()}
		if err != nil {
			row.Error = err.Error()
			row.ErrorType = fmt.Sprintf("%T", err)
		}
		versionRows[i] = row
	}
	// Index 0 is a nil version; subsequent entries include partial Versions from
	// failed parses, exactly as returned by TryParseVersion.
	targets := make([]*Version, len(versions)+1)
	for i := range versions {
		targets[i+1] = &versions[i]
	}
	compare := func(version *Version) string {
		result := make([]byte, len(targets))
		for i, target := range targets {
			result[i] = map[int]byte{-1: '<', 0: '=', 1: '>'}[version.Compare(target)]
		}
		return string(result)
	}
	for i, value := range keys {
		versionRows[i].Compare = compare(&versions[i])
		versionRange, ok := TryParseVersionRange(value)
		matches := make([]byte, len(targets))
		for j, target := range targets {
			matches[j] = '0'
			if versionRange.Test(target) {
				matches[j] = '1'
			}
		}
		rangeRows[i] = s07Range{ok, versionRange.String(), string(matches)}
	}
	result := struct {
		Inputs     []s07Input   `json:"inputs"`
		Versions   []s07Version `json:"versions"`
		Ranges     []s07Range   `json:"ranges"`
		NilCompare string       `json:"nil_compare"`
	}{requests, versionRows, rangeRows, compare(nil)}
	data, err := json.Marshal(result)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S07_SEMVER_OUTPUT"), append(data, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
