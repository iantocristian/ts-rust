package packagejson

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

type s07Expected struct {
	Present bool   `json:"present"`
	Valid   bool   `json:"valid"`
	Null    bool   `json:"null"`
	Actual  string `json:"actual"`
	Value   any    `json:"value"`
}

func s07Expect[T any](e Expected[T]) s07Expected {
	return s07Expected{e.IsPresent(), e.Valid, e.Null, e.ActualJSONType(), e.Value}
}

type s07Value struct {
	Kind       int8   `json:"kind"`
	Value      any    `json:"value"`
	Falsy      bool   `json:"falsy"`
	ObjectKind string `json:"object_kind,omitempty"`
}
type s07Entry struct {
	Key   string   `json:"key"`
	Value s07Value `json:"value"`
}

func s07JSON(v JSONValue) s07Value {
	result := s07Value{Kind: int8(v.Type), Value: v.Value, Falsy: v.IsFalsy()}
	switch v.Type {
	case JSONValueTypeObject:
		items := []s07Entry{}
		for key, value := range v.AsObject().Entries() {
			items = append(items, s07Entry{key, s07JSON(value)})
		}
		result.Value = items
	case JSONValueTypeArray:
		items := []s07Value{}
		for _, value := range v.AsArray() {
			items = append(items, s07JSON(value))
		}
		result.Value = items
	}
	return result
}
func s07Exports(v ExportsOrImports) s07Value {
	result := s07Value{Kind: int8(v.Type), Value: v.Value, Falsy: v.IsFalsy()}
	switch v.Type {
	case JSONValueTypeObject:
		items := []s07Entry{}
		for key, value := range v.AsObject().Entries() {
			items = append(items, s07Entry{key, s07Exports(value)})
		}
		result.Value = items
		if v.IsSubpaths() {
			result.ObjectKind = "subpaths"
		} else if v.IsImports() {
			result.ObjectKind = "imports"
		} else if v.IsConditions() {
			result.ObjectKind = "conditions"
		} else {
			result.ObjectKind = "invalid"
		}
	case JSONValueTypeArray:
		items := []s07Value{}
		for _, value := range v.AsArray() {
			items = append(items, s07Exports(value))
		}
		result.Value = items
	}
	return result
}
func TestS07PackageJSON(t *testing.T) {
	data, err := os.ReadFile(os.Getenv("S07_PACKAGEJSON_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []struct {
		ID  string `json:"id"`
		Hex string `json:"hex"`
	}
	if err := json.Unmarshal(data, &requests); err != nil {
		t.Fatal(err)
	}
	output := []any{}
	for _, request := range requests {
		raw, err := hex.DecodeString(request.Hex)
		if err != nil {
			t.Fatal(err)
		}
		p, err := Parse(raw)
		fields := map[string]any{"name": s07Expect(p.Name), "version": s07Expect(p.Version), "type": s07Expect(p.Type), "main": s07Expect(p.Main), "types": s07Expect(p.Types), "typings": s07Expect(p.Typings), "tsconfig": s07Expect(p.TSConfig), "dependencies": s07Expect(p.Dependencies), "devDependencies": s07Expect(p.DevDependencies), "peerDependencies": s07Expect(p.PeerDependencies), "optionalDependencies": s07Expect(p.OptionalDependencies)}
		mapper := s07Expect(p.ContentMapper)
		mapper.Value = map[string]any{"exec": s07Expect(p.ContentMapper.Value.Exec), "compilerOptions": s07Expect(p.ContentMapper.Value.CompilerOptions), "dynamicConfig": s07Expect(p.ContentMapper.Value.DynamicConfig)}
		fields["contentMapper"] = mapper
		errorText := ""
		if err != nil {
			errorText = fmt.Sprint(err)
		}
		output = append(output, map[string]any{"id": request.ID, "parseable": err == nil, "error": errorText, "fields": fields, "typesVersions": s07JSON(p.TypesVersions), "exports": s07Exports(p.Exports), "imports": s07Exports(p.Imports)})
	}
	data, err = json.Marshal(output)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S07_PACKAGEJSON_OUTPUT"), append(data, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
