package checker_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"reflect"
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

// The replica checker is trusted only after every named type it creates agrees
// with a real NewChecker over an empty program, for the same options.
func TestS08StorageFamilies(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var request struct {
		Version int                         `json:"version"`
		Options checker.S08FamiliesOptions  `json:"options"`
		Actions []checker.S08FamiliesAction `json:"actions"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&request); err != nil {
		t.Fatal(err)
	}
	if decoder.Decode(new(any)) != io.EOF || request.Version != 1 {
		t.Fatal("invalid trace request")
	}
	tristate := func(value bool) core.Tristate {
		if value {
			return core.TSTrue
		}
		return core.TSFalse
	}
	fs := bundled.WrapFS(vfstest.FromMap(map[string]string{"/empty.ts": ""}, false))
	host := compiler.NewCompilerHost("/", fs, bundled.LibPath(), nil, nil, nil)
	options := &core.CompilerOptions{NoLib: core.TSTrue, StrictNullChecks: tristate(request.Options.StrictNullChecks), ExactOptionalPropertyTypes: tristate(request.Options.ExactOptionalPropertyTypes)}
	config := tsoptions.NewParsedCommandLine(options, []string{"/empty.ts"}, nil, tspath.ComparePathsOptions{})
	program := compiler.NewProgram(compiler.ProgramOptions{Config: config, Host: host})
	program.BindSourceFiles()
	real, _ := checker.NewChecker(program, nil)
	replica := checker.S08FamiliesReplica(request.Options)
	realNamed := checker.S08FamiliesNamed(real)
	replicaNamed := checker.S08FamiliesNamed(replica)
	if !reflect.DeepEqual(realNamed, replicaNamed) {
		t.Fatalf("replica named types differ from NewChecker: real %v replica %v", realNamed, replicaNamed)
	}
	prefixCounts := checker.S08FamiliesCounts(replica)
	roots, typeRoots := checker.S08FamiliesTrace(replica, request.Actions)
	runtime.GC()
	census := checker.S08FamiliesCensus(replica, typeRoots)
	sum := sha256.Sum256(raw)
	output, err := json.Marshal(map[string]any{
		"request_sha256": hex.EncodeToString(sum[:]), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH,
		"named": replicaNamed, "real_counts": checker.S08FamiliesCounts(real), "prefix_counts": prefixCounts,
		"roots": roots, "counts": checker.S08FamiliesCounts(replica), "census": census,
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_OUTPUT"), output, 0600); err != nil {
		t.Fatal(err)
	}
}
