package module_test

import (
	"encoding/hex"
	"encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/module"
	"github.com/microsoft/TypeScript/tsc/internal/packagejson"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
	"os"
	"testing"
)

func TestS07ModuleTraces(t *testing.T) {
	var requests []struct {
		CaseSensitive bool `json:"case_sensitive"`
		ID            string
		Cwd           string
		Options       core.CompilerOptions
		Files         map[string]string
		Operations    []struct {
			Kind string
			Name string
			File string
			Mode core.ResolutionMode
		}
	}
	raw, err := os.ReadFile(os.Getenv("S07_MODULE_TRACE_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(raw, &requests); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	for _, request := range requests {
		resolver := module.NewResolver(&resolutionHostStub{fs: vfstest.FromMap(request.Files, request.CaseSensitive), cwd: request.Cwd}, &request.Options, "", "", nil)
		var previous *packagejson.InfoCacheEntry
		for index, operation := range request.Operations {
			var traces []module.DiagAndArgs
			var observedPackage any
			switch operation.Kind {
			case "automatic":
				module.GetAutomaticTypeDirectiveNames(&request.Options, &resolutionHostStub{fs: vfstest.FromMap(request.Files, request.CaseSensitive), cwd: request.Cwd})
			case "metadata":
				entry := resolver.GetPackageScopeForPath(operation.Name)
				if entry != nil {
					observedPackage = map[string]any{"directory_hex": hex.EncodeToString([]byte(entry.PackageDirectory)), "same_entry": entry == previous, "shared_contents": previous != nil && entry.Contents == previous.Contents}
					previous = entry
				}
			case "module":
				_, traces = resolver.ResolveModuleName(operation.Name, operation.File, operation.Mode, nil)
			case "type":
				_, traces = resolver.ResolveTypeReferenceDirective(operation.Name, operation.File, operation.Mode, nil)
			default:
				t.Fatal("unknown operation")
			}
			output := []any{}
			for _, trace := range traces {
				args := []any{}
				for _, arg := range trace.Args {
					switch arg := arg.(type) {
					case string:
						args = append(args, map[string]any{"text_hex": hex.EncodeToString([]byte(arg))})
					case bool:
						args = append(args, map[string]any{"bool": arg})
					default:
						t.Fatalf("unknown trace argument: %T", arg)
					}
				}
				output = append(output, map[string]any{"code": trace.Message.Code(), "args": args})
			}
			rows = append(rows, map[string]any{"id": request.ID, "operation": index, "traces": output, "package": observedPackage})
		}
	}
	output, err := json.MarshalIndent(rows, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S07_MODULE_TRACE_OUTPUT"), append(output, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
