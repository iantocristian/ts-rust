//go:build ignore

// Access-only adapter for the pinned diagnostics generator. Run beside generate.go
// with `go test generate.go s03_export_test.go -run TestS03ExportDiagnostics`.
package main

import (
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"slices"
	"strconv"
	"strings"
	"testing"
)

type exportedMessage struct {
	Name                         string `json:"name"`
	Key                          string `json:"key"`
	Text                         string `json:"text"`
	Code                         int    `json:"code"`
	Category                     string `json:"category"`
	ReportsUnnecessary           bool   `json:"reportsUnnecessary"`
	ReportsDeprecated            bool   `json:"reportsDeprecated"`
	ElidedInCompatibilityPyramid bool   `json:"elidedInCompatibilityPyramid"`
}

func TestS03ExportDiagnostics(t *testing.T) {
	output := os.Getenv("S03_DIAGNOSTICS_OUTPUT")
	if output == "" {
		t.Fatal("S03_DIAGNOSTICS_OUTPUT must name the export file")
	}
	// The pinned generator gives extras precedence by numeric code.
	messages := readRawMessages("diagnosticMessages.json")
	for code, message := range readRawMessages("extraDiagnosticMessages.json") {
		messages[code] = message
	}
	codes := make([]int, 0, len(messages))
	for code := range messages {
		codes = append(codes, code)
	}
	slices.Sort(codes)
	rows := make([]exportedMessage, 0, len(codes))
	for _, code := range codes {
		message := messages[code]
		name, key := convertPropertyName(message.key, code)
		rows = append(rows, exportedMessage{
			Name: name, Key: key, Text: message.key, Code: code, Category: message.Category,
			ReportsUnnecessary:           message.ReportsUnnecessary,
			ReportsDeprecated:            message.ReportsDeprecated,
			ElidedInCompatibilityPyramid: message.ElidedInCompatibilityPyramid,
		})
	}
	verifyPinnedDeclarations(t, rows)
	data, err := json.MarshalIndent(struct {
		Version  int               `json:"version"`
		Messages []exportedMessage `json:"messages"`
	}{Version: 1, Messages: rows}, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(output, append(data, '\n'), 0o644); err != nil {
		t.Fatal(err)
	}
}

// Cross-check every exported field against the untouched pinned Go output. The
// generator computes names/keys; Go's parser independently reads the checked-in
// declarations, catching missing extras, flags, or a changed adapter inventory.
func verifyPinnedDeclarations(t *testing.T, rows []exportedMessage) {
	t.Helper()
	file, err := parser.ParseFile(token.NewFileSet(), "diagnostics_generated.go", nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	actual := make(map[string]exportedMessage, len(rows))
	for _, row := range rows {
		actual[row.Name] = row
	}
	seen := 0
	for _, declaration := range file.Decls {
		group, ok := declaration.(*ast.GenDecl)
		if !ok || group.Tok != token.VAR {
			continue
		}
		for _, spec := range group.Specs {
			value := spec.(*ast.ValueSpec)
			if len(value.Names) != 1 || len(value.Values) != 1 {
				t.Fatal("unexpected pinned diagnostic declaration shape")
			}
			literal := value.Values[0].(*ast.UnaryExpr).X.(*ast.CompositeLit)
			row := exportedMessage{Name: value.Names[0].Name}
			for _, element := range literal.Elts {
				field := element.(*ast.KeyValueExpr)
				name := field.Key.(*ast.Ident).Name
				switch name {
				case "code":
					row.Code, err = strconv.Atoi(field.Value.(*ast.BasicLit).Value)
				case "category":
					row.Category = strings.TrimPrefix(field.Value.(*ast.Ident).Name, "Category")
				case "key", "text":
					var text string
					text, err = strconv.Unquote(field.Value.(*ast.BasicLit).Value)
					if name == "key" {
						row.Key = text
					} else {
						row.Text = text
					}
				case "reportsUnnecessary":
					row.ReportsUnnecessary = field.Value.(*ast.Ident).Name == "true"
				case "reportsDeprecated":
					row.ReportsDeprecated = field.Value.(*ast.Ident).Name == "true"
				case "elidedInCompatibilityPyramid":
					row.ElidedInCompatibilityPyramid = field.Value.(*ast.Ident).Name == "true"
				default:
					t.Fatalf("unrecognized pinned diagnostic field %q", name)
				}
				if err != nil {
					t.Fatal(err)
				}
			}
			if row != actual[row.Name] {
				t.Fatalf("export differs from pinned diagnostic declaration %s", row.Name)
			}
			seen++
		}
	}
	if seen != len(rows) {
		t.Fatalf("export has %d rows but pinned declarations have %d", len(rows), seen)
	}
}
