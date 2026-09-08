package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"strings"
	"testing"
)

func testWire(source, actions string) string {
	return fmt.Sprintf(`{"version":1,"id":"test","source_hex":%q,"decode_source":false,"target":0,"variant":0,"skip_trivia":true,"actions":%s}`, bytesHex(source), actions)
}

func TestRequestRejectsMalformedProtocol(t *testing.T) {
	valid := testWire("", `[{"op":"scan"}]`)
	bad := []string{
		strings.Replace(valid, `"version":1`, `"version":true`, 1),
		strings.Replace(valid, `"version":1`, `"version":1.0`, 1),
		strings.Replace(valid, `"id":"test"`, `"id":"test","id":"other"`, 1),
		strings.Replace(valid, `"id":"test"`, `"id":null`, 1),
		strings.Replace(valid, `"source_hex":""`, `"source_hex":"FF"`, 1),
		strings.Replace(valid, `"source_hex":""`, `"source_hex":"f"`, 1),
		strings.Replace(valid, `"variant":0`, `"variant":2`, 1),
		strings.Replace(valid, `"target":0`, `"target":13`, 1),
		strings.Replace(valid, `"id":"test"`, "\"id\":\"\xff\"", 1),
		valid + `{}`,
	}
	for _, actions := range []string{
		`[]`, `[{"op":"scan","op":"reset"}]`, `[{"op":"scan","flag":true}]`,
		`[{"op":"missing"}]`, `[{"op":"reset_pos","pos":true}]`,
		`[{"op":"reset_pos","pos":9223372036854775808}]`,
		`[{"op":"identifier_point","point":2147483648}]`,
		`[{"op":"identifier_block","first":1114111,"count":2}]`,
		`[{"op":"rescan_slash","report_errors":"maybe"}]`,
		`[{"op":"rescan_slash","report_errors":true}]`,
		`[{"op":"observe","getter":"secret"}]`, `[{"op":"rewind"}]`, `[{"op":"mark"}]`,
	} {
		bad = append(bad, testWire("", actions))
	}
	for index, wire := range bad {
		t.Run(fmt.Sprint(index), func(t *testing.T) {
			if _, err := decodeRequest([]byte(wire)); err == nil {
				t.Fatal("malformed input accepted")
			}
		})
	}
	if _, err := decodeRequest([]byte(testWire("", `[{"op":"mark"},{"op":"mark"},{"op":"commit"},{"op":"rewind"}]`))); err != nil {
		t.Fatalf("valid LIFO checkpoint input: %v", err)
	}
}

func TestDiagnosticWitnesses(t *testing.T) {
	for _, test := range []struct {
		source, reporting string
		codes             []int
	}{
		{"/a/zz", "true", []int{1499, 1499}},
		{"/a/zz", "false", nil},
		{"/a/zz", "omitted", nil},
		{"/abc", "true", []int{1161}},
		{"/abc", "false", []int{1161}},
	} {
		t.Run(test.source+test.reporting, func(t *testing.T) {
			wire := testWire(test.source, fmt.Sprintf(`[{"op":"scan"},{"op":"rescan_slash","report_errors":%q}]`, test.reporting))
			request, err := decodeRequest([]byte(wire))
			if err != nil {
				t.Fatal(err)
			}
			runner := newRunner(request)
			runner.observation("test", 0, 0, request.Actions[0])
			observation := runner.observation("test", 1, 1, request.Actions[1])
			if observation["status"] != "ok" {
				t.Fatalf("unexpected panic: %v", observation)
			}
			diagnostics := observation["diagnostics"].([]map[string]any)
			if len(diagnostics) != len(test.codes) {
				t.Fatalf("diagnostic count %d, want %d", len(diagnostics), len(test.codes))
			}
			for index, code := range test.codes {
				if diagnostics[index]["code"] != int32(code) {
					t.Fatalf("code %v, want %d", diagnostics[index]["code"], code)
				}
			}
		})
	}
}

func TestBigintPanicRetainsActualMalformedInput(t *testing.T) {
	request, err := decodeRequest([]byte(testWire("0x\xffn", `[{"op":"pseudo_bigint"}]`)))
	if err != nil {
		t.Fatal(err)
	}
	observation := newRunner(request).observation("test", 0, 0, request.Actions[0])
	value := observation["value"].(map[string]any)
	if observation["status"] != "panic" || value["class"] != "invalid_bigint" || value["input_hex"] != "3078ff" || value["message"] != `Failed to parse big int: "0x\xff"` {
		t.Fatalf("panic lost input/payload: %v", observation)
	}
}

func TestPersistentRequestsStartFreshAndRequireLF(t *testing.T) {
	first := testWire("/a/zz", `[{"op":"set_on_error","flag":false},{"op":"scan"},{"op":"rescan_slash","report_errors":"true"}]`)
	second := testWire("/a/zz", `[{"op":"scan"},{"op":"rescan_slash","report_errors":"true"}]`)
	var output bytes.Buffer
	if err := serve(strings.NewReader(first+"\n"+second+"\n"), &output); err != nil {
		t.Fatal(err)
	}
	decoder := json.NewDecoder(&output)
	counts := []int{}
	for decoder.More() {
		var record map[string]any
		if err := decoder.Decode(&record); err != nil {
			t.Fatal(err)
		}
		if record["event"] == "observation" {
			counts = append(counts, len(record["diagnostics"].([]any)))
		}
	}
	if fmt.Sprint(counts) != "[0 0 0 0 2]" {
		t.Fatalf("callback state leaked across requests: %v", counts)
	}
	output.Reset()
	if err := serve(strings.NewReader(second), &output); err == nil || output.Len() != 0 {
		t.Fatal("truncated input was executed")
	}
}

func TestDecodedSourceControlsPanicPayload(t *testing.T) {
	for _, source := range []string{"\xff\xfe0\x00b\x002\x00n\x00", "\xfe\xff\x000\x00b\x002\x00n", "\xef\xbb\xbf0b2n"} {
		wire := strings.Replace(testWire(source, `[{"op":"pseudo_bigint"}]`), `"decode_source":false`, `"decode_source":true`, 1)
		request, err := decodeRequest([]byte(wire))
		if err != nil {
			t.Fatal(err)
		}
		observation := newRunner(request).observation("test", 0, 0, request.Actions[0])
		value := observation["value"].(map[string]any)
		if observation["status"] != "panic" || value["class"] != "invalid_bigint" || value["input_hex"] != "306232" || value["message"] != `Failed to parse big int: "0b2"` {
			t.Fatalf("panic identity did not use decoded input: %v", observation)
		}
	}
}
