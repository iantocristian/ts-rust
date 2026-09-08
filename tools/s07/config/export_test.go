package testrunner

// Access-only source config observations. The requested AST and immutable host
// are passed to the original parser/config interpreter; no Rust result is read.
import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/collections"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
	"math"
	"os"
	"reflect"
	"strings"
	"testing"
)

type s07ConfigRequest struct {
	ID              string            `json:"id"`
	Cwd             string            `json:"cwd"`
	CaseSensitive   bool              `json:"case_sensitive"`
	FileName        string            `json:"file_name"`
	TextHex         string            `json:"text_hex"`
	Files           map[string]string `json:"files"`
	Symlinks        map[string]string `json:"symlinks"`
	RunExternalCode bool              `json:"run_external_code"`
}

func s07ConfigValue(value any) any {
	if value == nil {
		return map[string]any{"kind": "null"}
	}
	switch value := value.(type) {
	case string:
		return map[string]any{"kind": "string", "hex": hex.EncodeToString([]byte(value))}
	case bool:
		return map[string]any{"kind": "boolean", "value": value}
	case core.Tristate:
		return map[string]any{"kind": "boolean", "value": value.IsTrue()}
	case *collections.OrderedMap[string, []string]:
		entries := []any{}
		for key, child := range value.Entries() {
			entries = append(entries, map[string]any{"key": hex.EncodeToString([]byte(key)), "value": s07ConfigValue(child)})
		}
		return map[string]any{"kind": "object", "entries": entries}
	case []string:
		values := []any{}
		for _, child := range value {
			values = append(values, s07ConfigValue(child))
		}
		return map[string]any{"kind": "array", "nil": value == nil, "values": values}
	case float64:
		return map[string]any{"kind": "number", "bits": fmt.Sprintf("%016x", math.Float64bits(value))}
	case *collections.OrderedMap[string, any]:
		entries := []any{}
		for key, child := range value.Entries() {
			entries = append(entries, map[string]any{"key": hex.EncodeToString([]byte(key)), "value": s07ConfigValue(child)})
		}
		return map[string]any{"kind": "object", "entries": entries}
	case []any:
		values := []any{}
		for _, child := range value {
			values = append(values, s07ConfigValue(child))
		}
		return map[string]any{"kind": "array", "nil": value == nil, "values": values}
	case struct{}:
		return map[string]any{"kind": "empty_struct"}
	default:
		reflected := reflect.ValueOf(value)
		if reflected.Kind() == reflect.Pointer {
			return s07ConfigValue(reflected.Elem().Interface())
		}
		if reflected.Kind() == reflect.Int || reflected.Kind() == reflect.Int32 {
			return map[string]any{"kind": "integer", "value": reflected.Int()}
		}
		panic(fmt.Sprintf("unobserved config value %T", value))
	}
}
func s07ConfigDiag(value *ast.Diagnostic) any {
	var file any
	if value.File() != nil {
		file = hex.EncodeToString([]byte(value.File().FileName()))
	}
	args := []string{}
	for _, arg := range value.MessageArgs() {
		args = append(args, hex.EncodeToString([]byte(arg)))
	}
	chain := []any{}
	for _, child := range value.MessageChain() {
		chain = append(chain, s07ConfigDiag(child))
	}
	related := []any{}
	for _, child := range value.RelatedInformation() {
		related = append(related, s07ConfigDiag(child))
	}
	return map[string]any{"file": file, "start": value.Pos(), "end": value.End(), "code": value.Code(), "category": value.Category(), "message_key": hex.EncodeToString([]byte(value.MessageKey())), "message_text": hex.EncodeToString([]byte(value.MessageText())), "source": hex.EncodeToString([]byte(value.Source())), "args": args, "chain": chain, "related": related, "unnecessary": value.ReportsUnnecessary(), "deprecated": value.ReportsDeprecated(), "skipped_on_no_emit": value.SkippedOnNoEmit()}
}
func TestS07Config(t *testing.T) {
	input, err := os.ReadFile(os.Getenv("S07_CONFIG_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []s07ConfigRequest
	if err = json.Unmarshal(input, &requests); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	seen := map[string]bool{}
	for _, request := range requests {
		if seen[request.ID] || request.ID == "" {
			t.Fatal("duplicate/empty request ID")
		}
		seen[request.ID] = true
		files := map[string]any{}
		for name, encoded := range request.Files {
			bytes, err := hex.DecodeString(encoded)
			if err != nil {
				t.Fatal(err)
			}
			files[tspath.GetNormalizedAbsolutePath(name, request.Cwd)] = string(bytes)
		}
		for name, target := range request.Symlinks {
			files[tspath.GetNormalizedAbsolutePath(name, request.Cwd)] = vfstest.Symlink(tspath.GetNormalizedAbsolutePath(target, request.Cwd))
		}
		host := s07ConfigHost{vfstest.FromMap(files, request.CaseSensitive), request.Cwd}
		text, err := hex.DecodeString(request.TextHex)
		if err != nil {
			t.Fatal(err)
		}
		request.FileName = tspath.GetNormalizedAbsolutePath(request.FileName, request.Cwd)
		source := tsoptions.NewTsconfigSourceFileFromFilePath(request.FileName, tspath.ToPath(request.FileName, request.Cwd, request.CaseSensitive), string(text))
		options := &core.CompilerOptions{}
		if request.RunExternalCode {
			options.RunExternalCode = core.TSTrue
		}
		parsed := tsoptions.ParseJsonSourceFileConfigFileContent(source, &host, tspath.GetDirectoryPath(request.FileName), options, nil, request.FileName, nil, nil)
		diagnostics := []any{}
		for _, d := range parsed.Errors {
			diagnostics = append(diagnostics, s07ConfigDiag(d))
		}
		roots := []string{}
		for _, name := range parsed.FileNames() {
			roots = append(roots, hex.EncodeToString([]byte(name)))
		}
		rows = append(rows, map[string]any{"id": request.ID, "options": s07ConfigOptions(parsed.CompilerOptions()), "root_file_names": roots, "config_raw": s07ConfigValue(parsed.Raw), "config_diagnostics": diagnostics, "option_diagnostics": []any{}, "compile_on_save": parsed.CompileOnSave})
	}
	output, err := json.Marshal(rows)
	if err != nil {
		t.Fatal(err)
	}
	if err = os.WriteFile(os.Getenv("S07_CONFIG_OUTPUT"), append(output, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}

func s07ConfigOptions(options *core.CompilerOptions) any {
	entries := []any{}
	value := reflect.ValueOf(options).Elem()
	kind := value.Type()
	for i := range value.NumField() {
		field := value.Field(i)
		tag := strings.Split(kind.Field(i).Tag.Get("json"), ",")[0]
		if tag == "" || field.IsZero() {
			continue
		}
		entries = append(entries, map[string]any{"key": hex.EncodeToString([]byte(tag)), "value": s07ConfigValue(field.Interface())})
	}
	return map[string]any{"kind": "object", "entries": entries}
}
