// Copied into the disposable tooling worktree under tsc/internal/s03-wire-fixtures.
// This bridge invokes pinned serializers; it supplies observations for future
// Rust API tests, not a claim that the API runtime has already been ported.
package main

import (
	stdjson "encoding/json"
	"errors"
	"fmt"
	"os"

	jsonv2 "github.com/go-json-experiment/json"
	"github.com/go-json-experiment/json/jsontext"
	"github.com/microsoft/TypeScript/tsc/internal/api"
	"github.com/microsoft/TypeScript/tsc/internal/collections"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/json"
	"github.com/microsoft/TypeScript/tsc/internal/packagejson"
)

type fixture struct {
	ID        string     `json:"id"`
	Operation string     `json:"operation"`
	GoType    string     `json:"goType"`
	Input     any        `json:"input"`
	Output    any        `json:"output,omitempty"`
	Error     *wireError `json:"error,omitempty"`
}

type errorLocation struct {
	ByteOffset  int64  `json:"byteOffset"`
	JSONPointer string `json:"jsonPointer"`
}

type wireError struct {
	Class       string         `json:"class"`
	GoErrorType string         `json:"goErrorType"`
	Location    *errorLocation `json:"location,omitempty"`
	JSONKind    string         `json:"jsonKind,omitempty"`
	JSONValue   string         `json:"jsonValue,omitempty"`
	GoType      string         `json:"goType,omitempty"`
	GoPackage   string         `json:"goPackage,omitempty"`
	Reason      string         `json:"reason,omitempty"`
	Cause       *wireError     `json:"cause,omitempty"`
}

// SemanticError.Error deliberately randomizes its wording once per process.
// Preserve its exported facts and unwrap causes instead of editing that text.
// Unknown error types retain their exact reason: a new failure is not silently
// accepted as an existing bounds, syntax, or application error.
func describeError(err error) *wireError {
	if err == nil {
		return nil
	}
	result := &wireError{GoErrorType: fmt.Sprintf("%T", err)}
	switch typed := err.(type) {
	case *jsonv2.SemanticError:
		result.Class = "semantic"
		result.Location = &errorLocation{typed.ByteOffset, string(typed.JSONPointer)}
		if typed.JSONKind != 0 {
			result.JSONKind = typed.JSONKind.String()
		}
		result.JSONValue = string(typed.JSONValue)
		if typed.GoType != nil {
			result.GoType = typed.GoType.String()
			result.GoPackage = typed.GoType.PkgPath()
		}
	case *jsontext.SyntacticError:
		result.Class = "syntax"
		result.Location = &errorLocation{typed.ByteOffset, string(typed.JSONPointer)}
	default:
		result.Class = "cause"
		result.Reason = err.Error()
	}
	result.Cause = describeError(errors.Unwrap(err))
	return result
}

func observedError(id string, err error) *wireError {
	// Keep the full upstream rendering in captured stderr for diagnosis without
	// allowing its intentionally unstable modal verb into deterministic data.
	fmt.Fprintf(os.Stderr, "S03 wire fixture %s: %v\n", id, err)
	return describeError(err)
}

func main() {
	var rows []fixture
	marshal := func(id, goType string, value any) {
		data, err := json.Marshal(value)
		row := fixture{ID: id, Operation: "marshal", GoType: goType, Input: id}
		if err != nil {
			row.Error = observedError(id, err)
		} else {
			row.Output = string(data)
		}
		rows = append(rows, row)
	}
	marshal("ordinary-empty-options", "api.UpdateSnapshotParams", api.UpdateSnapshotParams{})
	marshal("ordinary-empty-slices", "api.UpdateSnapshotParams", api.UpdateSnapshotParams{OpenProjects: []api.DocumentIdentifier{}})
	marshal("ordinary-required-zero-id", "api.ReleaseParams", api.ReleaseParams{})
	marshal("ordinary-max-uint64-id", "api.ReleaseParams", api.ReleaseParams{Snapshot: ^api.SnapshotID(0)})
	marshal("ordinary-null-interface", "api.ReadConfigFileResponse", api.ReadConfigFileResponse{})
	marshal("batch-empty-responses", "api.BatchRequestsResponse", &api.BatchRequestsResponse{})
	marshal("batch-continuation", "api.BatchRequestsResponse", &api.BatchRequestsResponse{ContinuationToken: "next"})
	marshal("tristate-unknown", "core.Tristate", core.TSUnknown)
	marshal("tristate-true", "core.Tristate", core.TSTrue)
	marshal("tristate-false", "core.Tristate", core.TSFalse)
	marshal("enum-jsx-emit", "core.JsxEmit", core.JsxEmit(2))
	marshal("enum-module-detection", "core.ModuleDetectionKind", core.ModuleDetectionKind(2))
	marshal("enum-module-kind", "core.ModuleKind", core.ModuleKind(2))
	marshal("enum-module-resolution", "core.ModuleResolutionKind", core.ModuleResolutionKind(2))
	marshal("enum-newline", "core.NewLineKind", core.NewLineKind(1))
	marshal("enum-script-target", "core.ScriptTarget", core.ScriptTarget(2))
	marshal("literal-method", "api.Method", api.MethodRelease)
	marshal("literal-import-adder", "api.ImportAdderActionKind", api.ImportAdderActionKind("importSymbol"))
	marshal("raw-json-object", "json.Value", json.Value(`{"z":1,"a":[true,null]}`))
	marshal("raw-json-invalid", "json.Value", json.Value(`{"unterminated":`))
	ordered := collections.NewOrderedMapWithSizeHint[string, []string](2)
	ordered.Set("z", []string{"last"})
	ordered.Set("a", nil)
	marshal("ordered-map-insertion-order", "collections.OrderedMap[string, []string]", ordered)

	for i, input := range []string{`"foo.ts"`, `{"uri":"file:///foo.ts"}`, `{"uri":"file:///foo.ts","extra":true}`, `{}`, `42`} {
		var value api.DocumentIdentifier
		err := json.Unmarshal([]byte(input), &value)
		row := fixture{ID: fmt.Sprintf("document-identifier-%d", i), Operation: "unmarshal", GoType: "api.DocumentIdentifier", Input: input}
		if err != nil {
			row.Error = observedError(row.ID, err)
		} else {
			row.Output = struct {
				FileName string `json:"fileName"`
				URI      string `json:"uri"`
			}{value.FileName, string(value.URI)}
		}
		rows = append(rows, row)
	}
	for _, input := range []string{`true`, `false`, `null`, `42`} {
		var value core.Tristate
		err := json.Unmarshal([]byte(input), &value)
		row := fixture{ID: "tristate-decode-" + input, Operation: "unmarshal", GoType: "core.Tristate", Input: input}
		if err != nil {
			row.Error = observedError(row.ID, err)
		} else {
			row.Output = int(value)
		}
		rows = append(rows, row)
	}
	for i, input := range []string{`null`, `true`, `42`, `"text"`, `[null,1]`, `{"z":1,"a":2}`} {
		var value packagejson.JSONValue
		err := json.Unmarshal([]byte(input), &value)
		row := fixture{ID: fmt.Sprintf("package-json-kind-%d", i), Operation: "unmarshal", GoType: "packagejson.JSONValue", Input: input}
		if err != nil {
			row.Error = observedError(row.ID, err)
		} else {
			row.Output = value.Type.String()
		}
		rows = append(rows, row)
	}
	data, err := stdjson.MarshalIndent(struct {
		SchemaVersion int       `json:"schemaVersion"`
		Scope         string    `json:"scope"`
		Fixtures      []fixture `json:"fixtures"`
	}{1, "Representative pinned-Go wire observations; Rust runtime comparison belongs to the API sprint.", rows}, "", "  ")
	if err != nil {
		panic(err)
	}
	if _, err := os.Stdout.Write(append(data, '\n')); err != nil {
		panic(err)
	}
}
