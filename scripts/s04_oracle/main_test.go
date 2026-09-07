package main

import (
	"encoding/json"
	"strings"
	"testing"
)

const validProbe = `{"id":"fixture/1","group":"fixture","criterion":"utf16_positions","op":"scanner_to_position","text":"610a62","a":-1,"b":0,"flag":false,"panic_message":true}`

func TestRejectMalformedProbeBeforeEvaluation(t *testing.T) {
	for name, input := range map[string]string{
		"unknown operation": strings.Replace(validProbe, "scanner_to_position", "missing_operation", 1),
		"unknown field":     strings.Replace(validProbe, `"flag":false`, `"flag":false,"unexpected":1`, 1),
		"case alias":        strings.Replace(validProbe, `"flag":false`, `"flag":false,"Flag":true`, 1),
		"duplicate field":   strings.Replace(validProbe, `"flag":false`, `"flag":true,"flag":false`, 1),
		"missing field":     strings.Replace(validProbe, `"flag":false,`, "", 1),
		"null field":        strings.Replace(validProbe, `"flag":false`, `"flag":null`, 1),
		"wrong type":        strings.Replace(validProbe, `"flag":false`, `"flag":0`, 1),
		"empty identity":    strings.Replace(validProbe, `"id":"fixture/1"`, `"id":""`, 1),
		"invalid hex":       strings.Replace(validProbe, `"610a62"`, `"61zz"`, 1),
		"odd hex":           strings.Replace(validProbe, `"610a62"`, `"610"`, 1),
		"overflow offset":   strings.Replace(validProbe, `"a":-1`, `"a":9223372036854775808`, 1),
		"invalid quote":     strings.Replace(validProbe, "scanner_to_position", "escape", 1),
	} {
		t.Run(name, func(t *testing.T) {
			var probe Case
			if err := json.Unmarshal([]byte(input), &probe); err == nil {
				t.Fatal("malformed request was accepted as a behavioral probe")
			}
		})
	}
}

func TestContractPanicMessage(t *testing.T) {
	var probe Case
	if err := json.Unmarshal([]byte(validProbe), &probe); err != nil {
		t.Fatal(err)
	}
	result := evaluate(probe)
	if !result.Panic || result.Value != "Bad line number. Line: -1, lineStarts.length: 2." {
		t.Fatalf("unexpected contract panic: %+v", result)
	}
	probe.PanicMessage = false
	result = evaluate(probe)
	if !result.Panic || result.Value != nil {
		t.Fatalf("ordinary panic must retain the occurrence-only schema: %+v", result)
	}
}
