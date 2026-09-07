// Access-only behavioral adapter: each operation calls the unchanged Go pin.
package main

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"unicode/utf8"
)

const maxRequest = 16 * 1024 * 1024
const maxResponse = 1024 * 1024

type request struct {
	id, op string
	data   map[string]any
}

func parseRequest(line []byte) (request, error) {
	if !utf8.Valid(line) {
		return request{}, fmt.Errorf("invalid JSON UTF-8")
	}
	if err := scalarEscapes(line); err != nil {
		return request{}, err
	}
	decoder := json.NewDecoder(bytes.NewReader(line))
	decoder.UseNumber()
	value, err := strictValue(decoder)
	if err != nil {
		return request{}, err
	}
	if _, err = decoder.Token(); err != io.EOF {
		return request{}, fmt.Errorf("trailing JSON value")
	}
	m, ok := value.(map[string]any)
	if !ok {
		return request{}, fmt.Errorf("request object required")
	}
	op, err := text(m["op"])
	if err != nil {
		return request{}, err
	}
	names := map[string]string{
		"parse":      "version id primary op source_hex filename path script_kind jsx force operations",
		"kind_names": "version id primary op first count", "decode": "version id primary op wire_hex entrypoint",
		"path":    "version id primary op path_hex",
		"factory": "version id primary op scenario",
		"codec":   "version id primary op scenario",
	}
	spec, ok := names[op]
	if !ok {
		return request{}, fmt.Errorf("unknown operation")
	}
	if err = fields(m, spec); err != nil {
		return request{}, err
	}
	version, err := integer(m["version"])
	if err != nil || version != 1 {
		return request{}, fmt.Errorf("unsupported version")
	}
	id, err := text(m["id"])
	if err != nil || id == "" {
		return request{}, fmt.Errorf("nonempty id required")
	}
	if m["primary"] != nil {
		primary, err := text(m["primary"])
		if err != nil || primary == "" || op != "parse" {
			return request{}, fmt.Errorf("invalid primary row")
		}
	}
	switch op {
	case "codec":
		name, err := text(m["scenario"])
		if err != nil || !codecScenario(name) {
			return request{}, fmt.Errorf("invalid codec scenario")
		}
	case "factory":
		name, err := text(m["scenario"])
		if err != nil || factoryStages(name) == nil {
			return request{}, fmt.Errorf("invalid factory scenario")
		}
	case "parse":
		for _, key := range []string{"filename", "path"} {
			if _, err = text(m[key]); err != nil {
				return request{}, err
			}
		}
		if _, err = byteString(m["source_hex"]); err != nil {
			return request{}, err
		}
		kind, err := integer(m["script_kind"])
		if err != nil || kind < -2147483648 || kind > 2147483647 {
			return request{}, fmt.Errorf("invalid script kind")
		}
		for _, key := range []string{"jsx", "force"} {
			if _, err = boolean(m[key]); err != nil {
				return request{}, err
			}
		}
		ops, ok := m["operations"].([]any)
		want := []string{"parse", "node_index_before", "encode_source_file", "node_index_after"}
		if !ok || len(ops) != len(want) {
			return request{}, fmt.Errorf("invalid parser operation sequence")
		}
		for i, x := range want {
			if ops[i] != x {
				return request{}, fmt.Errorf("invalid parser operation order")
			}
		}
	case "kind_names":
		first, e1 := integer(m["first"])
		count, e2 := integer(m["count"])
		if e1 != nil || e2 != nil || first < -32768 || first > 32767 || count < 1 || count > 32768-first {
			return request{}, fmt.Errorf("invalid kind range")
		}
	case "decode":
		if _, err = byteString(m["wire_hex"]); err != nil {
			return request{}, err
		}
		if m["entrypoint"] != "nodes" && m["entrypoint"] != "source_file" {
			return request{}, fmt.Errorf("invalid decoder entrypoint")
		}
	case "path":
		if _, err = byteString(m["path_hex"]); err != nil {
			return request{}, err
		}
	}
	return request{id, op, m}, nil
}

type session struct {
	out         *bufio.Writer
	id          string
	seq, stages int
	err         error
}

func (s *session) frame(value map[string]any) {
	if s.err != nil {
		return
	}
	value["version"] = 1
	value["id"] = s.id
	data, err := json.Marshal(value)
	if err != nil {
		s.err = err
		return
	}
	if len(data) > maxResponse {
		s.err = fmt.Errorf("adapter observation exceeds frozen record limit")
		return
	}
	if _, err = s.out.Write(append(data, '\n')); err != nil {
		s.err = err
	}
}
func (s *session) observe(stage, kind string, value any) {
	s.frame(map[string]any{"tag": "observation", "seq": s.seq, "stage": stage, "kind": kind, "value": value})
	s.seq++
}

// Only the pinned operation runs under recovery. Protocol parsing, framing and
// output failures remain process failures, never expected behavioral panics.
func capture(action func() error) (outcome, message string) {
	outcome = "ok"
	defer func() {
		if value := recover(); value != nil {
			outcome = "panic"
			message = fmt.Sprint(value)
		}
	}()
	if err := action(); err != nil {
		outcome = "error"
		message = err.Error()
	}
	return
}
func (s *session) stage(name string, action func() error) bool {
	outcome, message := capture(action)
	s.frame(map[string]any{"tag": "stage", "stage": name, "outcome": outcome, "message_hex": hex.EncodeToString([]byte(message))})
	s.stages++
	return outcome == "ok"
}
func run(in io.Reader, out io.Writer) error {
	reader := bufio.NewScanner(in)
	reader.Buffer(make([]byte, 64*1024), maxRequest+1)
	reader.Split(func(data []byte, atEOF bool) (int, []byte, error) {
		if at := bytes.IndexByte(data, '\n'); at >= 0 {
			return at + 1, data[:at], nil
		}
		if atEOF && len(data) != 0 {
			return 0, nil, fmt.Errorf("truncated request record")
		}
		return 0, nil, nil
	})
	writer := bufio.NewWriter(out)
	seen := map[string]bool{}
	for reader.Scan() {
		line := reader.Bytes()
		if len(line) > maxRequest {
			return fmt.Errorf("request record too large")
		}
		req, err := parseRequest(line)
		if err != nil {
			return err
		}
		if seen[req.id] {
			return fmt.Errorf("duplicate request identity %q", req.id)
		}
		seen[req.id] = true
		s := session{out: writer, id: req.id}
		s.frame(map[string]any{"tag": "begin", "op": req.op})
        if s.err != nil {return s.err}
        if err=writer.Flush();err!=nil{return err}
		execute(&s, req)
		if s.err != nil {
			return s.err
		}
		s.frame(map[string]any{"tag": "end", "observations": s.seq, "stages": s.stages})
		if s.err != nil {
			return s.err
		}
		if err = writer.Flush(); err != nil {
			return err
		}
	}
	if err := reader.Err(); err != nil {
		return err
	}
	return nil
}
func main() {
	if err := run(os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, "S06 oracle protocol:", err)
		os.Exit(2)
	}
}
