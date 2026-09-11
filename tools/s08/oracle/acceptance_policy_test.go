package testrunner

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
)

func TestS08AcceptancePolicy(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []struct {
		ID      string                `json:"id"`
		Path    string                `json:"path"`
		Options *core.CompilerOptions `json:"options"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&requests); err != nil {
		t.Fatal(err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		t.Fatal("trailing request data")
	}
	rows := make([]map[string]any, 0, len(requests))
	seen := map[string]bool{}
	for _, request := range requests {
		if request.ID == "" || request.Options == nil || seen[request.ID] {
			t.Fatal("invalid or duplicate request")
		}
		seen[request.ID] = true
		row := map[string]any{"id": request.ID, "filename_skip": slices.Contains(skippedTests, filepath.Base(request.Path)), "option_guard": "not_executed"}
		t.Run(request.ID, func(t *testing.T) {
			defer func() {
				if t.Failed() {
					row["option_guard"] = "failed"
				} else if t.Skipped() {
					row["option_guard"] = "skipped"
				}
			}()
			// Execute the original guard, including its fail-before-skip order.
			// A fatal guard/panic fails this observation command, not a new exclusion.
			harnessutil.SkipUnsupportedCompilerOptions(t, request.Options)
			row["option_guard"] = "allowed"
		})
		rows = append(rows, row)
	}
	hash := sha256.Sum256(raw)
	result, err := json.Marshal(map[string]any{"request_sha256": hex.EncodeToString(hash[:]), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH, "rows": rows})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_OUTPUT"), result, 0600); err != nil {
		t.Fatal(err)
	}
}
