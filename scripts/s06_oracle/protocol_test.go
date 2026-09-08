package main

import (
	"bytes"
	"errors"
	"io"
	"strings"
	"testing"
)

const validRequest = `{"version":1,"id":"x","primary":null,"op":"path","path_hex":"2f612f2e2e2f62"}`

func TestS06StrictRequests(t *testing.T) {
	for _, raw := range []string{
		strings.Replace(validRequest, `"version":1`, `"version":1,"version":1`, 1),
		strings.Replace(validRequest, `"version":1`, `"version":true`, 1),
		strings.Replace(validRequest, `"version":1`, `"version":1.0`, 1),
		strings.Replace(validRequest, `"primary":null`, `"primary":"primary"`, 1),
		strings.Replace(validRequest, `"path_hex":"2f612f2e2e2f62"`, `"path_hex":"FF"`, 1),
		strings.Replace(validRequest, `"op":"path"`, `"op":"missing"`, 1),
		validRequest + ` {}`,
		strings.Replace(validRequest, `"id":"x"`, `"id":"\ud800"`, 1),
		strings.Replace(validRequest, `"id":"x"`, `"id":"\udc00"`, 1),
	} {
		if _, err := parseRequest([]byte(raw)); err == nil {
			t.Fatalf("accepted %s", raw)
		}
	}
	if _, err := parseRequest([]byte(validRequest)); err != nil {
		t.Fatal(err)
	}
}
func TestS06TruncatedAndDuplicateRequests(t *testing.T) {
	for _, raw := range []string{validRequest, validRequest + "\n" + validRequest + "\n"} {
		if err := run(strings.NewReader(raw), io.Discard); err == nil {
			t.Fatalf("accepted invalid framing %s", raw)
		}
	}
}

type failedWriter struct{}

func (failedWriter) Write([]byte) (int, error) { return 0, errors.New("injected output failure") }
func TestS06OutputFailureIsNotBehavior(t *testing.T) {
	if err := run(strings.NewReader(validRequest+"\n"), failedWriter{}); err == nil || !strings.Contains(err.Error(), "injected output failure") {
		t.Fatal(err)
	}
}
func TestS06ResponseFrameAndPanicIsolation(t *testing.T) {
	var output bytes.Buffer
	if err := run(strings.NewReader(validRequest+"\n"), &output); err != nil {
		t.Fatal(err)
	}
	for _, want := range []string{`"tag":"begin"`, `"normalized_hex":"2f62"`, `"outcome":"ok"`, `"observations":1`, `"stages":1`} {
		if !strings.Contains(output.String(), want) {
			t.Fatal(want, output.String())
		}
	}
	outcome, message := capture(func() error { panic("specific pinned assertion") })
	if outcome != "panic" || message != "specific pinned assertion" {
		t.Fatal(outcome, message)
	}
	outcome, message = capture(func() error { return errors.New("specific returned error") })
	if outcome != "error" || message != "specific returned error" {
		t.Fatal(outcome, message)
	}
}
