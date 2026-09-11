package checker

import (
	"encoding/hex"
	"fmt"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/printer"
)

// These are comparator-domain constructions, not claims that a source program
// naturally creates the same records. Only the constructors/accessors are bridged;
// every comparison executes the original pinned implementation.
func S08Residuals(c, other *Checker, file *ast.SourceFile) map[string]any {
	out := map[string]any{}
	a := c.newIntrinsicType(TypeFlagsAny, "any")
	b := c.newIntrinsicType(TypeFlagsAny, "any")
	pairs := func(xs []*Type) [][]int {
		rows := [][]int{}
		for _, x := range xs {
			row := []int{}
			for _, y := range xs {
				row = append(row, CompareTypes(x, y))
			}
			rows = append(rows, row)
		}
		return rows
	}
	out["nil-and-intrinsic-creation"] = pairs([]*Type{nil, a, b})
	out["foreign-checker"] = func() (result any) {
		defer func() {
			if p := recover(); p != nil {
				result = map[string]any{"state": "panic", "message": fmt.Sprint(p)}
			}
		}()
		return map[string]any{"state": "returned", "value": CompareTypes(a, other.anyType)}
	}()
	s1, s2 := &ast.Symbol{Name: "duplicate"}, &ast.Symbol{Name: "duplicate"}
	// The initial comparison itself mints the lazy IDs, in argument order.
	out["duplicate-name-no-declarations"] = []int{c.compareSymbolsWorker(s2, s1), c.compareSymbolsWorker(s1, s2), c.compareSymbolsWorker(nil, s1), c.compareSymbolsWorker(s1, nil)}
	r1, r2 := c.newObjectType(ObjectFlagsReverseMapped, nil), c.newObjectType(ObjectFlagsReverseMapped, nil)
	out["reverse-mapped-no-symbol-or-mapper"] = pairs([]*Type{r1, r2})
	target := c.newObjectType(ObjectFlagsInterface, nil)
	d1 := c.createDeferredTypeReference(target, file.Statements.Nodes[0], nil, nil)
	d2 := c.createDeferredTypeReference(target, file.Statements.Nodes[1], nil, nil)
	out["deferred-source-location"] = pairs([]*Type{d1, d2})
	i1, i2 := c.newObjectType(ObjectFlagsInstantiationExpressionType, nil), c.newObjectType(ObjectFlagsInstantiationExpressionType, nil)
	i1.AsInstantiationExpressionType().node = file.Statements.Nodes[0]
	i2.AsInstantiationExpressionType().node = file.Statements.Nodes[1]
	out["instantiation-expression-location"] = pairs([]*Type{i1, i2})
	simple1, simple2 := newSimpleTypeMapper(a, a), newSimpleTypeMapper(a, b)
	array1, array2 := newArrayTypeMapper([]*Type{a, b}, []*Type{a, a}), newArrayTypeMapper([]*Type{a, b}, []*Type{a, b})
	merged1, merged2 := newMergedTypeMapper(simple1, array1), newMergedTypeMapper(simple1, array2)
	mappers := []*TypeMapper{nil, simple1, simple2, array1, array2, merged1, merged2}
	matrix := [][]int{}
	for _, x := range mappers {
		row := []int{}
		for _, y := range mappers {
			row = append(row, compareTypeMappers(x, y))
		}
		matrix = append(matrix, row)
	}
	out["nested-mapper-comparison"] = matrix
	d3 := c.createDeferredTypeReference(target, file.Statements.Nodes[0], merged1, nil)
	d4 := c.createDeferredTypeReference(target, file.Statements.Nodes[0], merged2, nil)
	out["deferred-same-location-mapper"] = pairs([]*Type{d3, d4})
	out["nil-node-order"] = []int{c.compareNodes(nil, file.Statements.Nodes[0]), c.compareNodes(file.Statements.Nodes[0], nil), c.compareNodes(nil, nil)}
	return out
}

type S08TextRequest struct {
	ID        string `json:"id"`
	SourceHex string `json:"source_hex"`
	ValueHex  string `json:"value_hex"`
}

func S08Text(c *Checker, requests []S08TextRequest) []map[string]any {
	result := []map[string]any{}
	for _, request := range requests {
		source, err := hex.DecodeString(request.SourceHex)
		if err != nil {
			panic(err)
		}
		value, err := hex.DecodeString(request.ValueHex)
		if err != nil {
			panic(err)
		}
		file := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: "/literal.ts", Path: "/literal.ts"}, string(source), core.ScriptKindTS)
		literal := file.Statements.Nodes[0].AsVariableStatement().DeclarationList.AsVariableDeclarationList().Declarations.Nodes[0].AsVariableDeclaration().Initializer
		context := printer.NewEmitContext()
		p := printer.NewPrinter(printer.PrinterOptions{NewLine: core.NewLineKindLF, Target: core.ScriptTargetESNext}, printer.PrintHandlers{}, context)
		factory := printer.NewNodeFactory(context)
		synthetic := factory.NewStringLiteral(string(value), ast.TokenFlagsNone)
		typ := c.newLiteralType(TypeFlagsStringLiteral, string(value), nil)
		verbosity := &VerbosityContext{MaxTruncationLength: 10}
		result = append(result, map[string]any{"id": request.ID,
			"original_hex":      hex.EncodeToString([]byte(p.Emit(literal, file))),
			"synthetic_hex":     hex.EncodeToString([]byte(p.Emit(synthetic, nil))),
			"type_default_hex":  hex.EncodeToString([]byte(c.TypeToString(typ))),
			"type_full_hex":     hex.EncodeToString([]byte(c.TypeToStringEx(typ, nil, TypeFormatFlagsNoTruncation, nil))),
			"type_short_hex":    hex.EncodeToString([]byte(c.TypeToStringEx(typ, nil, TypeFormatFlagsNone, verbosity))),
			"truncated":         verbosity.Truncated,
			"parse_diagnostics": S08Diagnostics(file.Diagnostics())})
	}
	return result
}
