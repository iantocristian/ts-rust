package module_test

import (
	"encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/module"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
	"os"
	"testing"
)

func TestS07ConfigResolver(t *testing.T) {
	var requests []struct {
		ID, Name, File, Cwd string
		CaseSensitive       bool `json:"case_sensitive"`
		Files               map[string]string
	}
	raw, err := os.ReadFile(os.Getenv("S07_CONFIG_RESOLVER_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	if err = json.Unmarshal(raw, &requests); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	for _, r := range requests {
		host := &resolutionHostStub{fs: vfstest.FromMap(r.Files, r.CaseSensitive), cwd: r.Cwd}
		result := module.ResolveConfig(r.Name, r.File, host)
		rows = append(rows, map[string]any{"id": r.ID, "result": result})
	}
	raw, err = json.MarshalIndent(rows, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err = os.WriteFile(os.Getenv("S07_CONFIG_RESOLVER_OUTPUT"), append(raw, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
