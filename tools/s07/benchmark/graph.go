// Untimed reports use the same canonical collector as the binder oracle.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"io"
	"sort"
)

func normalizeGraph(value any, path string, names *[]any) any {
	switch value := value.(type) {
	case map[string]any:
		if raw, ok := value["raw_hex"]; ok && len(value) == 2 && value["identity"] != nil {
			*names = append(*names, map[string]any{"path": path, "raw_hex": raw, "identity": value["identity"]})
			return map[string]any{"identity": value["identity"]}
		}
		result := map[string]any{}
		// Record qualification occurrences in the same lexical field order as Rust.
		keys := make([]string, 0, len(value))
		for key := range value {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		for _, key := range keys {
			result[key] = normalizeGraph(value[key], path+"."+key, names)
		}
		return result
	case []any:
		result := make([]any, len(value))
		for index, item := range value {
			result[index] = normalizeGraph(item, fmt.Sprintf("%s[%d]", path, index), names)
		}
		return result
	default:
		return value
	}
}
func writeGraphReport(out io.Writer, file *ast.SourceFile, input Loaded, index, workers int) {
	records := collectGraph(file)
	raw := sha256.New()
	canonical := sha256.New()
	names := []any{}
	source := records[0].value.(map[string]any)
	for index, item := range records {
		record := []any{item.kind, item.value}
		bytes, err := json.Marshal(record)
		if err != nil {
			panic(err)
		}
		raw.Write(bytes)
		raw.Write([]byte("\n"))
		bytes, err = json.Marshal(normalizeGraph(record, fmt.Sprintf("$[%d]", index), &names))
		if err != nil {
			panic(err)
		}
		canonical.Write(bytes)
		canonical.Write([]byte("\n"))
	}
	loadedHash := sha256.Sum256([]byte(input.text))
	report := map[string]any{"version": 1, "kind": "bind_graph", "workers": workers, "index": index,
		"filename_hex": hx(input.options.FileName), "path_hex": hx(string(input.options.Path)), "script_kind": input.kind, "jsx": input.options.ExternalModuleIndicatorOptions.JSX, "force": input.options.ExternalModuleIndicatorOptions.Force,
		"source_bytes": len(input.text), "source_sha256": hex.EncodeToString(loadedHash[:]), "canonical_sha256": hex.EncodeToString(canonical.Sum(nil)), "raw_sha256": hex.EncodeToString(raw.Sum(nil)),
		"records": len(records), "counts": records[len(records)-1].value, "qualified_names": names, "diagnostics": source["diagnostics"],
		"node_count": source["node_count"], "symbol_count": source["symbol_count"], "parse_diagnostics": len(file.Diagnostics()), "bind_diagnostics": len(file.BindDiagnostics())}
	if err := json.NewEncoder(out).Encode(report); err != nil {
		panic(err)
	}
}

// A selected raw graph is still collected from the full unchanged native pool.
func writeGraphRecords(out io.Writer, file *ast.SourceFile, index, workers int) {
	for ordinal, item := range collectGraph(file) {
		record := map[string]any{"version": 1, "kind": "graph_record", "workers": workers, "index": index, "ordinal": ordinal, "record_kind": item.kind, "value": item.value}
		if err := json.NewEncoder(out).Encode(record); err != nil {
			panic(err)
		}
	}
}
