package main

import (
	stdjson "encoding/json"
	"errors"
	"reflect"
	"strings"
	"testing"

	jsonv2 "github.com/go-json-experiment/json"
	"github.com/go-json-experiment/json/jsontext"
	"github.com/microsoft/TypeScript/tsc/internal/api"
	"github.com/microsoft/TypeScript/tsc/internal/json"
)

func TestDescribeErrorPreservesStructuredContextAndExactCause(t *testing.T) {
	err := &jsonv2.SemanticError{
		ByteOffset: 17, JSONPointer: "/outer", JSONKind: '0', JSONValue: []byte("42"),
		GoType: reflect.TypeFor[int](),
		Err: &jsontext.SyntacticError{
			ByteOffset: 19, JSONPointer: "/outer/inner", Err: errors.New("exact syntax reason"),
		},
	}
	got := describeError(err)
	if got.Class != "semantic" || got.GoType != "int" || got.JSONKind != "number" || got.JSONValue != "42" {
		t.Fatalf("semantic error fields lost: %#v", got)
	}
	if *got.Location != (errorLocation{17, "/outer"}) || *got.Cause.Location != (errorLocation{19, "/outer/inner"}) {
		t.Fatalf("nested locations lost: %#v", got)
	}
	if got.Cause.Class != "syntax" || got.Cause.Cause.Reason != "exact syntax reason" {
		t.Fatalf("nested syntax cause lost: %#v", got)
	}
	encoded, err2 := stdjson.Marshal(got)
	if err2 != nil {
		t.Fatal(err2)
	}
	if strings.Contains(string(encoded), "cannot") || strings.Contains(string(encoded), "unable to") {
		t.Fatalf("unstable semantic rendering leaked into snapshot: %s", encoded)
	}
	if !strings.Contains(string(encoded), `"goErrorType":"*json.SemanticError"`) {
		t.Fatalf("concrete Go error type was not preserved: %s", encoded)
	}
}

func TestUnknownErrorWordingIsNotNormalized(t *testing.T) {
	first := describeError(errors.New("custom codec cannot decode this value"))
	second := describeError(errors.New("custom codec unable to decode this value"))
	if reflect.DeepEqual(first, second) {
		t.Fatal("different unknown error causes must remain different")
	}
	if first.Reason != "custom codec cannot decode this value" || second.Reason != "custom codec unable to decode this value" {
		t.Fatal("unknown error text was rewritten")
	}
}

func TestNativeErrorFixturesRetainTheirSpecificFailure(t *testing.T) {
	_, err := json.Marshal(json.Value(`{"unterminated":`))
	if err == nil {
		t.Fatal("malformed raw JSON unexpectedly marshaled")
	}
	raw := describeError(err)
	if raw.Class != "semantic" || raw.Cause == nil || raw.Cause.Class != "syntax" || raw.Cause.Cause == nil || raw.Cause.Cause.Reason != "unexpected EOF" {
		t.Fatalf("wrong raw JSON failure: %#v", raw)
	}
	if *raw.Cause.Location != (errorLocation{16, "/unterminated"}) {
		t.Fatalf("raw JSON failure lost its syntax location: %#v", raw.Cause)
	}
	var document api.DocumentIdentifier
	err = json.Unmarshal([]byte("42"), &document)
	if err == nil {
		t.Fatal("number unexpectedly decoded as DocumentIdentifier")
	}
	doc := describeError(err)
	if doc.Class != "semantic" || doc.Cause == nil || doc.Cause.Reason != "DocumentIdentifier: expected string or object, got number" {
		t.Fatalf("wrong document identifier failure: %#v", doc)
	}
	if doc.GoType != "api.DocumentIdentifier" || doc.GoPackage != "github.com/microsoft/TypeScript/tsc/internal/api" {
		t.Fatalf("document error lost its exact Go type: %#v", doc)
	}
}
