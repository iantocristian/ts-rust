// The E4 oracle: runs the pinned Go implementation over tests/e4/fixtures.json
// and prints its results as JSON. It is compiled inside the unmodified upstream
// module through a `go build -overlay` (see scripts/experiments.py), which places
// this file at cmd/oracle-e4 and the wrappers in tools/oracle-e4/overlay into
// the packages whose unexported functions the fixtures exercise. Nothing is
// written into the submodule.
package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/ls/lsconv"
	"github.com/microsoft/TypeScript/tsc/internal/lsp/lsproto"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/printer"
	"github.com/microsoft/TypeScript/tsc/internal/scanner"
	"github.com/microsoft/TypeScript/tsc/internal/spanmap"
	"github.com/microsoft/TypeScript/tsc/internal/stringutil"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/oracle"
)

type fixtures struct {
	Decode []struct {
		ID    string `json:"id"`
		Input string `json:"input"`
	} `json:"decode"`
	Helpers []struct {
		ID       string `json:"id"`
		Input    string `json:"input"`
		Truncate []int  `json:"truncate"`
	} `json:"helpers"`
	Escape []struct {
		ID    string `json:"id"`
		Input string `json:"input"`
		Quote string `json:"quote"`
		Flags int    `json:"flags"`
	} `json:"escape"`
	Slices []struct {
		ID     string   `json:"id"`
		Input  string   `json:"input"`
		Ranges [][2]int `json:"ranges"`
	} `json:"slices"`
	Texts []struct {
		ID           string   `json:"id"`
		Text         string   `json:"text"`
		Utf16Offsets []int    `json:"utf16_offsets"`
		ByteOffsets  []int    `json:"byte_offsets"`
		Positions    [][2]int `json:"positions"`
	} `json:"texts"`
}

type oracleScript struct{ text string }

func (s *oracleScript) FileName() string          { return "/fixture.ts" }
func (s *oracleScript) OriginalFileName() string  { return "/fixture.ts" }
func (s *oracleScript) Text() string              { return s.text }
func (s *oracleScript) OriginalText() string      { return s.text }
func (s *oracleScript) SpanMap() *spanmap.SpanMap { return nil }

func h(s string) string { return strings.ToUpper(hex.EncodeToString([]byte(s))) }

func unhex(s string) string {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return string(b)
}

// guard records a panic as the string "panic" instead of a value.
func guard(f func() any) (result any) {
	defer func() {
		if r := recover(); r != nil {
			result = "panic"
		}
	}()
	return f()
}

func key(l, c int) string { return strconv.Itoa(l) + "," + strconv.Itoa(c) }

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: oracle-e4 <fixtures.json>")
		os.Exit(2)
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var fx fixtures
	if err := json.Unmarshal(raw, &fx); err != nil {
		panic(err)
	}
	out := map[string]any{}

	decode := map[string]any{}
	for _, f := range fx.Decode {
		contents, ok := oracle.DecodeBytes(unhex(f.Input))
		decode[f.ID] = map[string]any{"output": h(contents), "ok": ok}
	}
	out["decode"] = decode

	helpers := map[string]any{}
	for _, f := range fx.Helpers {
		s := unhex(f.Input)
		truncate := map[string]string{}
		for _, n := range f.Truncate {
			truncate[strconv.Itoa(n)] = h(stringutil.TruncateByRunes(s, n))
		}
		jsRunes, stdRunes := [][2]int{}, [][2]int{} // non-nil so empty input encodes as [] like the harness
		var reencoded strings.Builder
		for i := 0; i < len(s); {
			r, size := stringutil.DecodeJSStringRune(s[i:])
			jsRunes = append(jsRunes, [2]int{int(r), size})
			reencoded.WriteString(stringutil.EncodeJSStringRune(r))
			i += size
		}
		for i := 0; i < len(s); {
			r, size := core.DecodeRuneForOracle(s[i:])
			stdRunes = append(stdRunes, [2]int{int(r), size})
			i += size
		}
		helpers[f.ID] = map[string]any{
			"lower":       h(stringutil.ToLowerJS(s)),
			"upper":       h(stringutil.ToUpperJS(s)),
			"lower_first": h(stringutil.LowerFirstChar(s)),
			"combined":    h(stringutil.CombineSurrogatePairs(s)),
			"truncate":    truncate,
			"js_runes":    jsRunes,
			"std_runes":   stdRunes,
			"reencoded":   h(reencoded.String()),
			"utf16_len":   int(core.UTF16Len(s)),
		}
	}
	out["helpers"] = helpers

	escape := map[string]any{}
	for _, f := range fx.Escape {
		escape[f.ID] = map[string]any{"output": h(printer.OracleEscapeStringWorker(unhex(f.Input), rune(f.Quote[0]), f.Flags))}
	}
	out["escape"] = escape

	slices := map[string]any{}
	for _, f := range fx.Slices {
		s := unhex(f.Input)
		ranges := map[string]string{}
		for _, r := range f.Ranges {
			ranges[strconv.Itoa(r[0])+"-"+strconv.Itoa(r[1])] = h(s[r[0]:r[1]])
		}
		slices[f.ID] = ranges
	}
	out["slices"] = slices

	texts := map[string]any{}
	for _, f := range fx.Texts {
		text := unhex(f.Text)
		sf := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: "/fixture.ts", Path: tspath.Path("/fixture.ts")}, text, core.ScriptKindTS)
		ecmaStarts := core.ComputeECMALineStarts(text)
		fileStarts := scanner.GetECMALineStarts(sf)
		lspMap := lsconv.ComputeLSPLineStarts(text)
		pm := ast.ComputePositionMap(text)
		script := &oracleScript{text: text}
		c16 := lsconv.NewConverters(lsproto.PositionEncodingKindUTF16, func(string) *lsconv.LSPLineMap { return lspMap })
		c8 := lsconv.NewConverters(lsproto.PositionEncodingKindUTF8, func(string) *lsconv.LSPLineMap { return lspMap })

		r := map[string]any{
			"ecma_line_starts":       ecmaStarts,
			"file_ecma_line_starts":  fileStarts,
			"lsp_line_starts":        lspMap.LineStarts,
			"lsp_ascii_only":         lspMap.AsciiOnly,
			"pm_ascii_only":          pm.IsAsciiOnly(),
			"utf16_to_utf8":          map[string]any{},
			"utf8_to_utf16":          map[string]any{},
			"utf16_len_prefix":       map[string]any{},
			"pos_to_line_byte":       map[string]any{},
			"line_of_position":       map[string]any{},
			"ecma_line_utf16_of_pos": map[string]any{},
			"ecma_line_byte_of_pos":  map[string]any{},
			"lsp_pos_utf16":          map[string]any{},
			"lsp_pos_utf8":           map[string]any{},
			"lsp_index_of_line_start": map[string]any{},
			"ecma_end_line":          map[string]any{},
			"lsp_utf16":              map[string]any{},
			"lsp_utf8":               map[string]any{},
			"scanner_utf16_edit":     map[string]any{},
			"scanner_utf16_strict":   map[string]any{},
			"scanner_byte":           map[string]any{},
		}
		for _, u := range f.Utf16Offsets {
			r["utf16_to_utf8"].(map[string]any)[strconv.Itoa(u)] = pm.UTF16ToUTF8(u)
		}
		for _, b := range f.ByteOffsets {
			k := strconv.Itoa(b)
			r["utf8_to_utf16"].(map[string]any)[k] = pm.UTF8ToUTF16(b)
			if b <= len(text) {
				r["utf16_len_prefix"].(map[string]any)[k] = int(core.UTF16Len(text[:b]))
			}
			r["pos_to_line_byte"].(map[string]any)[k] = guard(func() any {
				line, off := core.PositionToLineAndByteOffset(b, ecmaStarts)
				return [2]int{line, off}
			})
			r["line_of_position"].(map[string]any)[k] = scanner.ComputeLineOfPosition(ecmaStarts, b)
			r["ecma_line_utf16_of_pos"].(map[string]any)[k] = guard(func() any {
				line, ch := scanner.GetECMALineAndUTF16CharacterOfPosition(sf, b)
				return [2]int{line, int(ch)}
			})
			r["ecma_line_byte_of_pos"].(map[string]any)[k] = guard(func() any {
				line, off := scanner.GetECMALineAndByteOffsetOfPosition(sf, b)
				return [2]int{line, off}
			})
			r["lsp_pos_utf16"].(map[string]any)[k] = guard(func() any {
				p := lsconv.OraclePositionToLineAndCharacter(c16, script, core.TextPos(b))
				return [2]int{int(p.Line), int(p.Character)}
			})
			r["lsp_pos_utf8"].(map[string]any)[k] = guard(func() any {
				p := lsconv.OraclePositionToLineAndCharacter(c8, script, core.TextPos(b))
				return [2]int{int(p.Line), int(p.Character)}
			})
			r["lsp_index_of_line_start"].(map[string]any)[k] = lspMap.ComputeIndexOfLineStart(core.TextPos(b))
		}
		for line := range ecmaStarts {
			line := line
			r["ecma_end_line"].(map[string]any)[strconv.Itoa(line)] = guard(func() any { return scanner.GetECMAEndLinePosition(sf, line) })
		}
		for _, p := range f.Positions {
			l, c := p[0], p[1]
			k := key(l, c)
			r["lsp_utf16"].(map[string]any)[k] = guard(func() any { return int(lsconv.OracleLineAndCharacterToPosition(c16, script, uint32(int32(l)), uint32(int32(c)))) })
			r["lsp_utf8"].(map[string]any)[k] = guard(func() any { return int(lsconv.OracleLineAndCharacterToPosition(c8, script, uint32(int32(l)), uint32(int32(c)))) })
			r["scanner_utf16_edit"].(map[string]any)[k] = guard(func() any {
				return scanner.ComputePositionOfLineAndUTF16Character(ecmaStarts, l, core.UTF16Offset(c), text, true)
			})
			r["scanner_utf16_strict"].(map[string]any)[k] = guard(func() any {
				return scanner.ComputePositionOfLineAndUTF16Character(ecmaStarts, l, core.UTF16Offset(c), text, false)
			})
			r["scanner_byte"].(map[string]any)[k] = guard(func() any { return scanner.ComputePositionOfLineAndByteOffset(ecmaStarts, l, c) })
		}
		texts[f.ID] = r
	}
	out["texts"] = texts

	enc := json.NewEncoder(os.Stdout)
	if err := enc.Encode(out); err != nil {
		panic(err)
	}
}
