// Command e4 answers the E4 fixture probes from the pinned Corsa packages.
//
// It reads data/e4-fixtures.json and writes one JSON object per criterion of
// probe -> value pairs. The Rust harness writes an object of the same shape from
// ts_jsstring, and scripts/e4-harness.py compares them. Values are hex byte
// strings or short textual encodings so a malformed byte and a panic are both
// representable.
//
// Only exported entry points of the pinned packages are called, so every value
// here comes from production code rather than from a re-derivation of it.
package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/ls/lsconv"
	"github.com/microsoft/TypeScript/tsc/internal/lsp/lsproto"
	"github.com/microsoft/TypeScript/tsc/internal/printer"
	"github.com/microsoft/TypeScript/tsc/internal/scanner"
	"github.com/microsoft/TypeScript/tsc/internal/spanmap"
	"github.com/microsoft/TypeScript/tsc/internal/stringutil"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
)

type fixture struct {
	ID    string `json:"id"`
	Bytes string `json:"bytes"`
	Why   string `json:"why"`
}

type probes struct {
	OutOfRangeOffsets []int `json:"out_of_range_offsets"`
	Characters        []int `json:"characters"`
	OutOfRangeLines   []int `json:"out_of_range_lines"`
}

type manifest struct {
	Files        []fixture `json:"files"`
	Texts        []fixture `json:"texts"`
	Helpers      []fixture `json:"helpers"`
	Runes        []int     `json:"runes"`
	Truncations  []int     `json:"truncations"`
	Probes       probes    `json:"probes"`
	CaseSweep    []int     `json:"case_sweep"`
	RangeSweep   []int     `json:"range_sweep"`
	UpstreamPin  string    `json:"upstream_pin"`
}

// script is the minimal lsconv.Script for an ordinary, non-content-mapped file.
type script struct {
	name string
	text string
}

func (s script) FileName() string         { return s.name }
func (s script) OriginalFileName() string { return s.name }
func (s script) Text() string             { return s.text }
func (s script) OriginalText() string     { return s.text }
func (s script) SpanMap() *spanmap.SpanMap { return nil }

// sourceFile is the minimal ast.SourceFileLike the scanner's ECMAScript
// wrappers need.
type sourceFile struct {
	text      string
	lineStarts []core.TextPos
}

func (f sourceFile) Text() string                { return f.text }
func (f sourceFile) ECMALineMap() []core.TextPos { return f.lineStarts }

// guard runs body and reports "panic" where the pinned code panics, so the
// comparison covers panic behavior instead of stopping at it.
func guard(body func() string) (result string) {
	defer func() {
		if recover() != nil {
			result = "panic"
		}
	}()
	return body()
}

func decodeFixtures(list []fixture) map[string]string {
	out := make(map[string]string, len(list))
	for _, f := range list {
		raw, err := hex.DecodeString(f.Bytes)
		if err != nil {
			panic(fmt.Sprintf("fixture %s: %v", f.ID, err))
		}
		out[f.ID] = string(raw)
	}
	return out
}

func order(list []fixture) []string {
	ids := make([]string, 0, len(list))
	for _, f := range list {
		ids = append(ids, f.ID)
	}
	return ids
}

func joinPositions(values []core.TextPos) string {
	parts := make([]string, len(values))
	for i, v := range values {
		parts[i] = fmt.Sprint(int(v))
	}
	return strings.Join(parts, ",")
}

// offsetProbes is every byte offset of the text plus the declared out-of-range
// values. The Rust harness derives exactly the same list.
func offsetProbes(text string, m *manifest) []int {
	out := make([]int, 0, len(text)+1+len(m.Probes.OutOfRangeOffsets))
	for i := 0; i <= len(text); i++ {
		out = append(out, i)
	}
	return append(out, m.Probes.OutOfRangeOffsets...)
}

func lineProbes(lineStarts []core.TextPos, m *manifest) []int {
	out := make([]int, 0, len(lineStarts)+len(m.Probes.OutOfRangeLines))
	for i := range lineStarts {
		out = append(out, i)
	}
	return append(out, m.Probes.OutOfRangeLines...)
}

func sourceDecoding(m *manifest, dir string) map[string]string {
	out := map[string]string{}
	fs := osvfs.FS()
	for _, f := range m.Files {
		raw, err := hex.DecodeString(f.Bytes)
		if err != nil {
			panic(err)
		}
		path, err := filepath.Abs(filepath.Join(dir, f.ID+".bin"))
		if err != nil {
			panic(err)
		}
		if err := os.WriteFile(path, raw, 0o600); err != nil {
			panic(err)
		}
		// ReadFile is the production path: it decodes byte order marks through
		// the same decodeBytes the compiler uses.
		contents, ok := fs.ReadFile(filepath.ToSlash(path))
		out["decode/"+f.ID] = hex.EncodeToString([]byte(contents))
		out["ok/"+f.ID] = fmt.Sprint(ok)
	}
	return out
}

// classify mirrors the JsString validity tag: valid UTF-8 is Utf8; otherwise, if
// the sentinel-aware decoder consumes every byte, Wtf8; otherwise Raw.
func classify(s string) string {
	if utf8Valid(s) {
		return "utf8"
	}
	for i := 0; i < len(s); {
		r, size := stringutil.DecodeJSStringRune(s[i:])
		if r == 0xFFFD && size == 1 {
			return "raw"
		}
		i += size
	}
	return "wtf8"
}

func utf8Valid(s string) bool {
	for i := 0; i < len(s); {
		r, size := decodeStandard(s[i:])
		if r == 0xFFFD && size == 1 {
			return false
		}
		i += size
	}
	return true
}

func sliceValidity(m *manifest) map[string]string {
	out := map[string]string{}
	texts := decodeFixtures(m.Texts)
	for _, id := range order(m.Texts) {
		text := texts[id]
		n := len(text)
		for a := 0; a <= n; a++ {
			for b := a; b <= n; b++ {
				key := fmt.Sprintf("/%s/%d/%d", id, a, b)
				part := text[a:b]
				out["bytes"+key] = hex.EncodeToString([]byte(part))
				out["validity"+key] = classify(part)
				out["str"+key] = fmt.Sprint(utf8Valid(part))
			}
		}
		// Out-of-range slices are refused, not clamped.
		for _, pair := range [][2]int{{0, n + 1}, {n + 1, n + 2}, {1, 0}} {
			key := fmt.Sprintf("oob/%s/%d/%d", id, pair[0], pair[1])
			out[key] = guard(func() string {
				if pair[0] > pair[1] || pair[1] > n {
					panic("out of range")
				}
				return hex.EncodeToString([]byte(text[pair[0]:pair[1]]))
			})
		}
	}
	return out
}

func helperSemantics(m *manifest) map[string]string {
	out := map[string]string{}
	helpers := decodeFixtures(m.Helpers)
	quotes := map[string]printer.QuoteChar{
		"single":   printer.QuoteCharSingleQuote,
		"double":   printer.QuoteCharDoubleQuote,
		"backtick": printer.QuoteCharBacktick,
	}
	for _, id := range order(m.Helpers) {
		value := helpers[id]
		out["to_lower/"+id] = hex.EncodeToString([]byte(stringutil.ToLowerJS(value)))
		out["to_upper/"+id] = hex.EncodeToString([]byte(stringutil.ToUpperJS(value)))
		out["lower_first/"+id] = hex.EncodeToString([]byte(stringutil.LowerFirstChar(value)))
		out["combine/"+id] = hex.EncodeToString([]byte(stringutil.CombineSurrogatePairs(value)))
		for _, n := range m.Truncations {
			out[fmt.Sprintf("truncate/%s/%d", id, n)] =
				hex.EncodeToString([]byte(stringutil.TruncateByRunes(value, n)))
		}
		for name, quote := range quotes {
			out[fmt.Sprintf("escape/%s/%s", id, name)] =
				hex.EncodeToString([]byte(printer.EscapeString(value, quote)))
		}
		for i := 0; i <= len(value); i++ {
			r, size := stringutil.DecodeJSStringRune(value[i:])
			out[fmt.Sprintf("decode_rune/%s/%d", id, i)] = fmt.Sprintf("%d,%d", r, size)
			sr, ssize := decodeStandard(value[i:])
			out[fmt.Sprintf("decode_standard/%s/%d", id, i)] = fmt.Sprintf("%d,%d", sr, ssize)
		}
	}
	for _, cp := range m.Runes {
		out[fmt.Sprintf("encode_rune/%d", cp)] =
			hex.EncodeToString([]byte(stringutil.EncodeJSStringRune(rune(cp))))
		out[fmt.Sprintf("surrogate/%d", cp)] = fmt.Sprintf("%v,%v,%v",
			stringutil.IsSurrogate(rune(cp)),
			stringutil.IsHighSurrogate(rune(cp)),
			stringutil.IsLowSurrogate(rune(cp)))
	}
	// The sweep covers every code point Unicode 15.1.0 gives a case mapping.
	for _, cp := range m.CaseSweep {
		value := string(rune(cp))
		out[fmt.Sprintf("sweep_lower/%d", cp)] = hex.EncodeToString([]byte(stringutil.ToLowerJS(value)))
		out[fmt.Sprintf("sweep_upper/%d", cp)] = hex.EncodeToString([]byte(stringutil.ToUpperJS(value)))
		out[fmt.Sprintf("sweep_lower_first/%d", cp)] =
			hex.EncodeToString([]byte(stringutil.LowerFirstChar(value)))
	}
	// The Cased and Case_Ignorable tables are only observable through the
	// Final_Sigma context, so each swept code point is placed after a sigma in
	// two contexts. "cased", "case ignorable" and "neither" give three distinct
	// pairs of results.
	for _, cp := range m.RangeSweep {
		value := string(rune(cp))
		out[fmt.Sprintf("sigma_final/%d", cp)] =
			hex.EncodeToString([]byte(stringutil.ToLowerJS(sigmaPrefix + value)))
		out[fmt.Sprintf("sigma_followed/%d", cp)] =
			hex.EncodeToString([]byte(stringutil.ToLowerJS(sigmaPrefix + value + "A")))
	}
	return out
}

// sigmaPrefix is a cased letter followed by a capital sigma, which is the
// backward half of the Final_Sigma condition.
const sigmaPrefix = "\u039f\u03a3"

func utf8Positions(m *manifest) map[string]string {
	out := map[string]string{}
	texts := decodeFixtures(m.Texts)
	converters := lsconv.NewConverters(lsproto.PositionEncodingKindUTF8, nil)
	for _, id := range order(m.Texts) {
		text := texts[id]
		ecma := core.ComputeECMALineStarts(text)
		lsp := lsconv.ComputeLSPLineStarts(text)
		file := sourceFile{text: text, lineStarts: ecma}
		out["ecma_line_starts/"+id] = joinPositions(ecma)
		out["lsp_line_starts/"+id] = joinPositions(lsp.LineStarts)
		out["lsp_ascii_only/"+id] = fmt.Sprint(lsp.AsciiOnly)

		withLineMap := lsconv.NewConverters(lsproto.PositionEncodingKindUTF8,
			func(string) *lsconv.LSPLineMap { return lsp })
		_ = converters
		doc := script{name: "/" + id + ".ts", text: text}

		for _, p := range offsetProbes(text, m) {
			out[fmt.Sprintf("line_of_position/%s/%d", id, p)] =
				fmt.Sprint(scanner.ComputeLineOfPosition(ecma, p))
			line, offset := core.PositionToLineAndByteOffset(p, ecma)
			out[fmt.Sprintf("position_to_line_byte/%s/%d", id, p)] = fmt.Sprintf("%d,%d", line, offset)
			out[fmt.Sprintf("lsp_index_of_line_start/%s/%d", id, p)] =
				fmt.Sprint(lsp.ComputeIndexOfLineStart(core.TextPos(p)))
			out[fmt.Sprintf("ecma_line_byte/%s/%d", id, p)] = guard(func() string {
				line, offset := scanner.GetECMALineAndByteOffsetOfPosition(file, p)
				return fmt.Sprintf("%d,%d", line, offset)
			})
			out[fmt.Sprintf("lsp8_from_position/%s/%d", id, p)] = guard(func() string {
				position, _ := withLineMap.ToLSPPosition(doc, core.TextPos(p))
				return fmt.Sprintf("%d,%d", position.Line, position.Character)
			})
		}
		for _, line := range lineProbes(ecma, m) {
			for _, offset := range []int{0, 1} {
				out[fmt.Sprintf("position_of_line_byte/%s/%d/%d", id, line, offset)] =
					guard(func() string {
						return fmt.Sprint(scanner.ComputePositionOfLineAndByteOffset(ecma, line, offset))
					})
			}
		}
		for _, line := range lineProbes(lsp.LineStarts, m) {
			for _, character := range m.Probes.Characters {
				out[fmt.Sprintf("lsp8_to_position/%s/%d/%d", id, line, character)] =
					guard(func() string {
						mapped := lsconv.FromLSPPosition(withLineMap, doc,
							lsproto.Position{Line: uint32(line), Character: uint32(character)},
							spanmap.FeatureNone)
						return fmt.Sprint(int(mapped[0].Position))
					})
			}
		}
	}
	return out
}

func utf16Positions(m *manifest) map[string]string {
	out := map[string]string{}
	texts := decodeFixtures(m.Texts)
	for _, id := range order(m.Texts) {
		text := texts[id]
		ecma := core.ComputeECMALineStarts(text)
		lsp := lsconv.ComputeLSPLineStarts(text)
		file := sourceFile{text: text, lineStarts: ecma}
		pm := ast.ComputePositionMap(text)
		converters := lsconv.NewConverters(lsproto.PositionEncodingKindUTF16,
			func(string) *lsconv.LSPLineMap { return lsp })
		doc := script{name: "/" + id + ".ts", text: text}

		out["pm_ascii_only/"+id] = fmt.Sprint(pm.IsAsciiOnly())
		out["utf16_len/"+id] = fmt.Sprint(int(core.UTF16Len(text)))

		for _, p := range offsetProbes(text, m) {
			out[fmt.Sprintf("pm_8_to_16/%s/%d", id, p)] = fmt.Sprint(pm.UTF8ToUTF16(p))
			out[fmt.Sprintf("pm_16_to_8/%s/%d", id, p)] = fmt.Sprint(pm.UTF16ToUTF8(p))
			out[fmt.Sprintf("ecma_line_utf16/%s/%d", id, p)] = guard(func() string {
				line, character := scanner.GetECMALineAndUTF16CharacterOfPosition(file, p)
				return fmt.Sprintf("%d,%d", line, int(character))
			})
			out[fmt.Sprintf("lsp16_from_position/%s/%d", id, p)] = guard(func() string {
				position, _ := converters.ToLSPPosition(doc, core.TextPos(p))
				return fmt.Sprintf("%d,%d", position.Line, position.Character)
			})
		}
		for _, line := range lineProbes(ecma, m) {
			for _, character := range m.Probes.Characters {
				for _, allowEdits := range []bool{false, true} {
					out[fmt.Sprintf("scanner_position/%s/%d/%d/%v", id, line, character, allowEdits)] =
						guard(func() string {
							return fmt.Sprint(scanner.ComputePositionOfLineAndUTF16Character(
								ecma, line, core.UTF16Offset(character), text, allowEdits))
						})
				}
			}
		}
		for _, line := range lineProbes(lsp.LineStarts, m) {
			for _, character := range m.Probes.Characters {
				out[fmt.Sprintf("lsp16_to_position/%s/%d/%d", id, line, character)] =
					guard(func() string {
						mapped := lsconv.FromLSPPosition(converters, doc,
							lsproto.Position{Line: uint32(line), Character: uint32(character)},
							spanmap.FeatureNone)
						return fmt.Sprint(int(mapped[0].Position))
					})
			}
		}
	}
	return out
}

func main() {
	if len(os.Args) != 4 {
		fmt.Fprintln(os.Stderr, "usage: e4 <fixtures.json> <scratch dir> <output.json>")
		os.Exit(2)
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var m manifest
	if err := json.Unmarshal(raw, &m); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	result := map[string]map[string]string{
		"source_decoding": sourceDecoding(&m, os.Args[2]),
		"slice_validity":  sliceValidity(&m),
		"helper_semantics": helperSemantics(&m),
		"utf8_positions":  utf8Positions(&m),
		"utf16_positions": utf16Positions(&m),
	}
	encoded, err := json.Marshal(result)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	if err := os.WriteFile(os.Args[3], encoded, 0o600); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	for name, values := range result {
		fmt.Fprintf(os.Stderr, "%s: %d probes\n", name, len(values))
	}
}
