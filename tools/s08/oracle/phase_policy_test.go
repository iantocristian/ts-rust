package core

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"runtime"
	"testing"
)

// This observes option policy only. It does not run a checker or declaration emit.
func TestS08PhasePolicy(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []struct {
		ID          string `json:"id"`
		Declaration *bool  `json:"declaration"`
		Composite   *bool  `json:"composite"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&requests); err != nil {
		t.Fatal(err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		t.Fatalf("trailing input: %v", err)
	}
	state := func(value *bool) Tristate {
		if value == nil {
			return TSUnknown
		}
		if *value {
			return TSTrue
		}
		return TSFalse
	}
	rows := make([]map[string]any, 0, len(requests))
	for _, request := range requests {
		options := CompilerOptions{Declaration: state(request.Declaration), Composite: state(request.Composite)}
		rows = append(rows, map[string]any{"id": request.ID, "declaration_requested": options.GetEmitDeclarations()})
	}
	sum := sha256.Sum256(raw)
	output, err := json.Marshal(map[string]any{
		"rows": rows, "request_sha256": hex.EncodeToString(sum[:]),
		"go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH,
		"scope": "GetEmitDeclarations option policy; no checker or emit execution",
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_OUTPUT"), output, 0600); err != nil {
		t.Fatal(err)
	}
}
