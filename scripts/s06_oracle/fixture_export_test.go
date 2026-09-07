// Source literals and native observations; no Rust output selects this inventory.
package testrunner

import (
	"encoding/hex"
	"encoding/json"
	goast "go/ast"
	goparser "go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/api/encoder"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions/tsoptionstest"
)

func TestS06FixtureExport(t *testing.T) {
	output := os.Getenv("S06_FIXTURE_OUTPUT")
	if output == "" {
		t.Skip("fixture exporter is invoked explicitly")
	}
	result := map[string]any{}
	kinds := make([]string, 65536)
	for i := -32768; i <= 32767; i++ {
		kinds[i+32768] = ast.Kind(i).String()
	}
	result["kind_names"] = kinds
	result["kind_first"] = -32768
	result["kind_count"] = int(ast.KindCount)
	result["wire_constants"] = map[string]int{"version": int(encoder.ProtocolVersion), "header_size": encoder.HeaderSize, "node_size": encoder.NodeSize, "kind_count": int(ast.KindCount), "synthetic_expression": int(ast.KindSyntheticExpression), "syntax_list": int(ast.KindSyntaxList), "jsdoc_type_literal": int(ast.KindJSDocTypeLiteral), "comma_token": int(ast.KindCommaToken)}
	fuzzDir := filepath.Join("..", "parser", "testdata", "fuzz", "FuzzParser")
	entries, err := os.ReadDir(fuzzDir)
	if err != nil {
		t.Fatal(err)
	}
	fuzz := []map[string]any{}
	for _, entry := range entries {
		if entry.IsDir() {
			t.Fatal("unexpected fuzz seed directory")
		}
		raw, err := os.ReadFile(filepath.Join(fuzzDir, entry.Name()))
		if err != nil {
			t.Fatal(err)
		}
		lines := strings.Split(strings.TrimSuffix(string(raw), "\n"), "\n")
		if len(lines) != 5 || lines[0] != "go test fuzz v1" {
			t.Fatal("unexpected Go fuzz seed encoding")
		}
		values := make([]string, 2)
		for i := range 2 {
			if !strings.HasPrefix(lines[i+1], "string(") || !strings.HasSuffix(lines[i+1], ")") {
				t.Fatal("invalid string seed")
			}
			values[i], err = strconv.Unquote(lines[i+1][7 : len(lines[i+1])-1])
			if err != nil {
				t.Fatal(err)
			}
		}
		flags := make([]bool, 2)
		for i := range 2 {
			if lines[i+3] != "bool(true)" && lines[i+3] != "bool(false)" {
				t.Fatal("invalid bool seed")
			}
			flags[i] = lines[i+3] == "bool(true)"
		}
		fuzz = append(fuzz, map[string]any{"name": entry.Name(), "raw_sha256": s06Hash(string(raw)), "extension": values[0], "source_hex": hex.EncodeToString([]byte(values[1])), "jsx": flags[0], "force": flags[1]})
	}
	result["fuzz_seeds"] = fuzz
	file, err := goparser.ParseFile(token.NewFileSet(), filepath.Join("..", "parser", "parser_test.go"), nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	regressions := []map[string]any{}
	for _, decl := range file.Decls {
		fn, ok := decl.(*goast.FuncDecl)
		if !ok || !strings.HasPrefix(fn.Name.Name, "Test") {
			continue
		}
		var source string
		count := 0
		goast.Inspect(fn.Body, func(node goast.Node) bool {
			assignment, ok := node.(*goast.AssignStmt)
			if !ok || len(assignment.Lhs) != 1 || len(assignment.Rhs) != 1 {
				return true
			}
			name, ok := assignment.Lhs[0].(*goast.Ident)
			if !ok || name.Name != "sourceText" {
				return true
			}
			literal, ok := assignment.Rhs[0].(*goast.BasicLit)
			if !ok || literal.Kind != token.STRING {
				t.Fatal("regression source is no longer a literal")
			}
			source, err = strconv.Unquote(literal.Value)
			if err != nil {
				t.Fatal(err)
			}
			count++
			return true
		})
		if count != 1 {
			t.Fatalf("regression %s has %d source literals", fn.Name.Name, count)
		}
		regressions = append(regressions, map[string]any{"name": fn.Name.Name, "source_hex": hex.EncodeToString([]byte(source))})
	}
	result["parser_regressions"] = regressions
	legacy := []map[string]any{}
	for _, target := range []string{"", "es5", "es2015", "es2020"} {
		args := []string{"--module", "none"}
		if target != "" {
			args = append(args, "--target", target)
		}
		parsed := tsoptions.ParseCommandLine(args, tsoptionstest.NewVFSParseConfigHost(map[string]string{}, "/.src", true))
		opts := parsed.CompilerOptions()
		diags := []map[string]any{}
		for _, d := range parsed.Errors {
			diags = append(diags, map[string]any{"code": d.Code(), "message": d.String()})
		}
		options := map[string]any{}
		for _, name := range []string{"/.src/index.ts", "/.src/index.js", "/.src/index.d.ts"} {
			metadata := ast.SourceFileMetaData{ImpliedNodeFormat: ast.GetImpliedNodeFormatForFile(name, "")}
			options[name] = ast.GetExternalModuleIndicatorOptions(name, opts, metadata)
		}
		legacy = append(legacy, map[string]any{"arguments": args, "diagnostics": diags, "module": int(opts.Module), "target": int(opts.Target), "emit_module": int(opts.GetEmitModuleKind()), "module_detection": int(opts.GetEmitModuleDetectionKind()), "parse_options": options})
	}
	result["legacy_module_none"] = legacy
	result["script_kind_ts"] = int(core.ScriptKindTS)
	data, err := json.Marshal(result)
	if err != nil {
		t.Fatal(err)
	}
	if err = os.WriteFile(output, append(data, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}
