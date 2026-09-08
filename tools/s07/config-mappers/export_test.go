package tsoptions

import (
	"encoding/hex"
	stdjson "encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/collections"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
	"math"
	"os"
	"strconv"
	"testing"
)

type s07JsonValue struct {
	Kind, Hex, Bits string
	Boolean         bool
	Values          []s07JsonValue
	Entries         []struct {
		Key   string
		Value s07JsonValue
	}
}

func (v s07JsonValue) value() any {
	switch v.Kind {
	case "null":
		return nil
	case "empty":
		return struct{}{}
	case "boolean":
		return v.Boolean
	case "string":
		b, _ := hex.DecodeString(v.Hex)
		return string(b)
	case "number":
		n, _ := strconv.ParseUint(v.Bits, 16, 64)
		return math.Float64frombits(n)
	case "nil_array":
		return []any(nil)
	case "array":
		a := []any{}
		for _, x := range v.Values {
			a = append(a, x.value())
		}
		return a
	case "object":
		m := collections.NewOrderedMapWithSizeHint[string, any](len(v.Entries))
		for _, entry := range v.Entries {
			key, _ := hex.DecodeString(entry.Key)
			m.Set(string(key), entry.Value.value())
		}
		return m
	default:
		panic("unknown JSON value kind")
	}
}
func TestS07ConfigMappers(t *testing.T) {
	var input struct {
		Mappers []struct {
			ID, Text                       string
			RunExternalCode, CaseSensitive bool
			Files                          map[string]string
		}
		JSON []struct {
			ID    string
			Value s07JsonValue
		}
	}
	raw, err := os.ReadFile(os.Getenv("S07_MAPPERS_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	if err = stdjson.Unmarshal(raw, &input); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	for _, r := range input.Mappers {
		host := resolveContentMapperHost{fs: vfstest.FromMap(r.Files, r.CaseSensitive)}
		source := NewTsconfigSourceFileFromFilePath("/home/project/tsconfig.json", tspath.Path("/home/project/tsconfig.json"), r.Text)
		options := &core.CompilerOptions{RunExternalCode: core.IfElse(r.RunExternalCode, core.TSTrue, core.TSFalse)}
		result := ParseJsonSourceFileConfigFileContent(source, host, "/home/project", options, nil, "/home/project/tsconfig.json", nil, nil)
		var mappers []any
		for _, m := range result.ContentMappers() {
			mappers = append(mappers, map[string]any{"package": m.Package, "extensions": m.Extensions, "options_hex": hex.EncodeToString(m.Options), "directory": m.PackageDirectory, "manifest": m.Manifest})
		}
		ds := []any{}
		for _, d := range result.Errors {
			args := []string{}
			for _, arg := range d.MessageArgs() {
				args = append(args, hex.EncodeToString([]byte(arg)))
			}
			ds = append(ds, map[string]any{"code": d.Code(), "pos": d.Pos(), "end": d.End(), "args_hex": args})
		}
		rows = append(rows, map[string]any{"id": r.ID, "mappers": mappers, "diagnostics": ds})
	}
	jsonRows := []any{}
	for _, r := range input.JSON {
		text, err := core.StringifyJson(r.Value.value(), "", "")
		out := map[string]any{"id": r.ID, "error": err != nil}
		if err == nil {
			out["text_hex"] = hex.EncodeToString([]byte(text))
		}
		jsonRows = append(jsonRows, out)
	}
	raw, err = stdjson.MarshalIndent(map[string]any{"mappers": rows, "json": jsonRows}, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err = os.WriteFile(os.Getenv("S07_MAPPERS_OUTPUT"), append(raw, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
