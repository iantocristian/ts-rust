// Differential oracle: invokes the pinned Go functions; all byte strings use hex on the wire.
// This file is installed under internal/vfs/s04oracle to satisfy Go's internal import rule.
package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"unicode/utf8"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/ls/lsconv"
	"github.com/microsoft/TypeScript/tsc/internal/printer"
	"github.com/microsoft/TypeScript/tsc/internal/scanner"
	"github.com/microsoft/TypeScript/tsc/internal/stringutil"
	filedecoder "github.com/microsoft/TypeScript/tsc/internal/vfs/internal"
)

type Case struct {
	ID           string `json:"id"`
	Group        string `json:"group"`
	Criterion    string `json:"criterion"`
	Op           string `json:"op"`
	Text         string `json:"text"`
	A            int64  `json:"a"`
	B            int64  `json:"b"`
	Flag         bool   `json:"flag"`
	PanicMessage bool   `json:"panic_message,omitempty"`
	decoded      []byte
}

// Validate adapter inputs before recovering behavioral panics. An unknown
// operation or malformed fixture must fail the run rather than match a panic.
func (c *Case) UnmarshalJSON(input []byte) error {
	decoder := json.NewDecoder(bytes.NewReader(input))
	opening, err := decoder.Token()
	if err != nil || opening != json.Delim('{') {
		return fmt.Errorf("probe must be an object")
	}
	fields := make(map[string]json.RawMessage)
	for decoder.More() {
		token, err := decoder.Token()
		if err != nil {
			return err
		}
		key := token.(string)
		switch key {
		case "id", "group", "criterion", "op", "text", "a", "b", "flag", "panic_message":
		default:
			return fmt.Errorf("unknown probe key %q", key)
		}
		if _, exists := fields[key]; exists {
			return fmt.Errorf("duplicate probe key %q", key)
		}
		var value json.RawMessage
		if err := decoder.Decode(&value); err != nil {
			return err
		}
		if bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
			return fmt.Errorf("null probe field %q", key)
		}
		fields[key] = value
	}
	for _, key := range []string{"id", "group", "criterion", "op", "text", "a", "b", "flag"} {
		if _, exists := fields[key]; !exists {
			return fmt.Errorf("missing probe field %q", key)
		}
	}
	type wireCase Case
	decoder = json.NewDecoder(bytes.NewReader(input))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode((*wireCase)(c)); err != nil {
		return err
	}
	if c.ID == "" || c.Group == "" || c.Criterion == "" {
		return fmt.Errorf("probe identity fields must be nonempty")
	}
	switch c.Op {
	case "source", "slice", "source_slice", "lower", "upper", "lower_first",
		"truncate", "combine", "encode", "decode_js", "decode_utf8", "escape",
		"api_to_utf16", "api_to_utf8", "api_ascii", "ecma_lines", "lsp_lines",
		"lsp_line_index", "lsp_to_position", "lsp_from_position", "scanner_to_position",
		"scanner_byte_position", "scanner_line", "scanner_end_line",
		"scanner_from_position", "byte_from_position", "utf16_len":
	default:
		return fmt.Errorf("unknown operation: %s", c.Op)
	}
	if c.Op == "escape" && ((c.A != 34 && c.A != 39 && c.A != 96) || c.B < 0 || c.B > 3) {
		return fmt.Errorf("invalid escape quote or flags")
	}
	c.decoded, err = hex.DecodeString(c.Text)
	return err
}

type Result struct {
	ID    string `json:"id"`
	Panic bool   `json:"panic"`
	Value any    `json:"value"`
}

func bytesResult(text string) any { return hex.EncodeToString([]byte(text)) }

func strView(text string) any {
	if utf8.ValidString(text) {
		return bytesResult(text)
	}
	return nil
}

func sourceFile(text string) *ast.SourceFile {
	return ast.NewNodeFactory(ast.NodeFactoryHooks{}).NewSourceFile(
		ast.SourceFileParseOptions{FileName: "/s04.ts"}, text, nil, nil,
	).AsSourceFile()
}

func validity(text string) string {
	if utf8.ValidString(text) {
		return "Utf8"
	}
	for len(text) > 0 {
		ch, size := stringutil.DecodeJSStringRune(text)
		if ch == utf8.RuneError && size == 1 {
			return "Raw"
		}
		text = text[size:]
	}
	return "Wtf8"
}

func evaluate(c Case) (r Result) {
	r.ID = c.ID
	defer func() {
		if payload := recover(); payload != nil {
			r.Panic = true
			r.Value = nil
			if c.PanicMessage {
				r.Value = fmt.Sprint(payload)
			}
		}
	}()
	text := string(c.decoded)
	switch c.Op {
	case "source":
		s := filedecoder.S04DecodeBytes(text)
		r.Value = []any{bytesResult(s), strView(s)}
	case "slice", "source_slice":
		if c.Op == "source_slice" {
			text = filedecoder.S04DecodeBytes(text)
		}
		if c.A < 0 || c.B < c.A || c.B > int64(len(text)) {
			r.Value = nil
			break
		}
		s := text[c.A:c.B]
		r.Value = []any{bytesResult(s), validity(s), strView(s)}
	case "lower":
		r.Value = bytesResult(stringutil.ToLowerJS(text))
	case "upper":
		r.Value = bytesResult(stringutil.ToUpperJS(text))
	case "lower_first":
		r.Value = bytesResult(stringutil.LowerFirstChar(text))
	case "truncate":
		r.Value = bytesResult(stringutil.TruncateByRunes(text, int(c.A)))
	case "combine":
		r.Value = bytesResult(stringutil.CombineSurrogatePairs(text))
	case "encode":
		r.Value = bytesResult(stringutil.EncodeJSStringRune(rune(c.A)))
	case "decode_js":
		ch, size := stringutil.DecodeJSStringRune(text)
		r.Value = []int{int(ch), size}
	case "decode_utf8":
		ch, size := utf8.DecodeRuneInString(text)
		r.Value = []int{int(ch), size}
	case "escape":
		r.Value = bytesResult(printer.S04Escape(text, int(c.A), int(c.B)))
	case "api_to_utf16":
		r.Value = ast.ComputePositionMap(text).UTF8ToUTF16(int(c.A))
	case "api_to_utf8":
		r.Value = ast.ComputePositionMap(text).UTF16ToUTF8(int(c.A))
	case "api_ascii":
		r.Value = ast.ComputePositionMap(text).IsAsciiOnly()
	case "ecma_lines":
		r.Value = core.ComputeECMALineStarts(text)
	case "lsp_lines":
		lm := lsconv.ComputeLSPLineStarts(text)
		r.Value = []any{lm.LineStarts, lm.AsciiOnly}
	case "lsp_line_index":
		r.Value = lsconv.ComputeLSPLineStarts(text).ComputeIndexOfLineStart(core.TextPos(c.A))
	case "lsp_to_position":
		r.Value = lsconv.S04ToPosition(text, uint32(c.A), uint32(c.B), c.Flag)
	case "lsp_from_position":
		r.Value = lsconv.S04FromPosition(text, int32(c.A), c.Flag)
	case "scanner_to_position":
		r.Value = scanner.ComputePositionOfLineAndUTF16Character(core.ComputeECMALineStarts(text), int(c.A), core.UTF16Offset(c.B), text, c.Flag)
	case "scanner_byte_position":
		r.Value = scanner.ComputePositionOfLineAndByteOffset(core.ComputeECMALineStarts(text), int(c.A), int(c.B))
	case "scanner_line":
		r.Value = scanner.ComputeLineOfPosition(core.ComputeECMALineStarts(text), int(c.A))
	case "scanner_end_line":
		r.Value = scanner.GetECMAEndLinePosition(sourceFile(text), int(c.A))
	case "scanner_from_position":
		line, character := scanner.GetECMALineAndUTF16CharacterOfPosition(sourceFile(text), int(c.A))
		r.Value = []int{line, int(character)}
	case "byte_from_position":
		line, offset := core.PositionToLineAndByteOffset(int(c.A), core.ComputeECMALineStarts(text))
		r.Value = []int{line, offset}
	case "utf16_len":
		r.Value = int(core.UTF16Len(text))
	default:
		panic("unknown operation: " + c.Op)
	}
	return
}

func main() {
	var cases []Case
	decoder := json.NewDecoder(os.Stdin)
	if err := decoder.Decode(&cases); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF || len(cases) == 0 {
		fmt.Fprintln(os.Stderr, "expected one nonempty probe array")
		os.Exit(1)
	}
	results := make([]Result, 0, len(cases))
	for _, c := range cases {
		results = append(results, evaluate(c))
	}
	if err := json.NewEncoder(os.Stdout).Encode(results); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
