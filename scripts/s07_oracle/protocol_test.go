package main

import (
	"bufio"
	"bytes"
	"errors"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"io"
	"strings"
	"testing"
)

const validRequest = `{"version":1,"id":"x","primary":null,"op":"bind","source_hex":"","filename":"/a.ts","path":"/a.ts","script_kind":3,"jsx":false,"force":false,"operations":["parse","parsed_graph","bind","bound_graph","repeat_bind","repeated_graph"]}`

func TestS07StrictRequests(t *testing.T) {
	for _, raw := range []string{
		strings.Replace(validRequest, `"version":1`, `"version":1,"version":1`, 1),
		strings.Replace(validRequest, `"version":1`, `"version":true`, 1),
		strings.Replace(validRequest, `"version":1`, `"version":1.0`, 1),
		strings.Replace(validRequest, `"source_hex":""`, `"source_hex":"FF"`, 1),
		strings.Replace(validRequest, `"op":"bind"`, `"op":"missing"`, 1),
		strings.Replace(validRequest, `"id":"x"`, `"id":"\ud800"`, 1),
		strings.Replace(validRequest, `"script_kind":3`, `"script_kind":2147483648`, 1),
		validRequest + ` {}`,
	} {
		if _, err := parseRequest([]byte(raw)); err == nil {
			t.Fatalf("accepted %s", raw)
		}
	}
	if _, err := parseRequest([]byte(validRequest)); err != nil {
		t.Fatal(err)
	}
}
func TestS07FramingAndPanicIsolation(t *testing.T) {
	for _, raw := range []string{validRequest, validRequest + "\n" + validRequest + "\n"} {
		if err := run(strings.NewReader(raw), io.Discard); err == nil {
			t.Fatal("accepted incomplete/duplicate request")
		}
	}
	outcome, message := capture(func() error { panic("source assertion") })
	if outcome != "panic" || message != "source assertion" {
		t.Fatal(outcome, message)
	}
	if err := run(strings.NewReader(validRequest+"\n"), failedWriter{}); err == nil || !strings.Contains(err.Error(), "injected") {
		t.Fatal(err)
	}
}

type failedWriter struct{}

func (failedWriter) Write([]byte) (int, error) { return 0, errors.New("injected writer failure") }
func TestS07AliasRanges(t *testing.T) {
	values := make([]*ast.Node, 5)
	independent := make([]*ast.Node, 3)
	queue := []queued{{"nodes", 1, values[1:3]}, {"declarations", 1, values[2:3]}, {"nodes", 2, independent}, {"nodes", 3, values[:0]}}
	records := []map[string]any{{}, {}, {}, {}}
	aliases(queue, records)
	if records[0]["backing_group"] != records[1]["backing_group"] || records[0]["backing_group"] == records[2]["backing_group"] || records[0]["backing_start"] != uintptr(0) || records[1]["backing_start"] != uintptr(1) || records[3]["backing_group"] != 0 {
		t.Fatal(records)
	}
}
func TestS07RejectSharedSynthetic(t *testing.T) {
	g := newGraph(nil)
	payload := ast.NewFlowReduceLabelData(nil, nil)
	first := &ast.FlowNode{Node: payload}
	g.synthetic(first, 1, "reduce")
	g.synthetic(first, 1, "reduce")
	defer func() {
		if value := recover(); value != "S07 observer: synthetic flow payload has multiple owners" {
			t.Fatalf("wrong invariant result: %v", value)
		}
	}()
	g.synthetic(&ast.FlowNode{Node: payload}, 2, "reduce")
}
func TestS07Fragments(t *testing.T) {
	var out bytes.Buffer
	s := session{id: "fragment", out: bufio.NewWriter(&out)}
	observeGraph(&s, "parsed_graph", "texts", map[string]any{"values_hex": []string{strings.Repeat("ab", 256*1024)}})
	if err := s.out.Flush(); err != nil {
		t.Fatal(err)
	}
	if s.seq != 5 || s.err != nil {
		t.Fatal(s.seq, s.err)
	}
	for _, line := range bytes.Split(bytes.TrimSpace(out.Bytes()), []byte("\n")) {
		if len(line) > maxResponse {
			t.Fatal("oversized fragment")
		}
	}
}
