package main

import (
	"encoding/hex"
	"encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
)

func dumpGraph(s *session, stage string, file *ast.SourceFile) {
	for _, record := range collectGraph(file) {
		observeGraph(s, stage, record.kind, record.value)
	}
}
func observeGraph(s *session, stage, kind string, value any) {
	const chunk = 128 * 1024
	raw, err := json.Marshal(value)
	if err != nil {
		panic(err)
	}
	if len(raw) <= chunk {
		s.observe(stage, kind, value)
		return
	}
	parts := (len(raw) + chunk - 1) / chunk
	for part := 0; part < parts; part++ {
		end := min((part+1)*chunk, len(raw))
		s.observe(stage, "fragment", map[string]any{"record_kind": kind, "part": part, "parts": parts, "payload_hex": hex.EncodeToString(raw[part*chunk : end])})
	}
}
