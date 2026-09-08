package scanner

import (
	"bytes"
	"encoding/json"
	"strings"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"os"
	"testing"
)

func TestS07ScannerHelpers(t *testing.T) {
	var output bytes.Buffer
	emit := func(label string, values ...int64) {
		fmt.Fprint(&output, label)
		for _, v := range values {
			fmt.Fprintf(&output, "\t%d", v)
		}
		fmt.Fprintln(&output)
	}
	text := func(label, value string) {
		values := make([]int64, len(value))
		for i, v := range []byte(value) {
			values[i] = int64(v)
		}
		emit(label, values...)
	}
	span := func(label string, value core.TextRange) { emit(label, int64(value.Pos()), int64(value.End())) }
	panicPayloads := map[string]string{}
	panics := func(label string, callback func()) (result int64) {
		defer func() {
			if recovered := recover(); recovered != nil {
				message := fmt.Sprint(recovered)
				panicPayloads[label] = fmt.Sprintf("%T: %v", recovered, recovered)
				switch {
				case message == "runtime error: invalid memory address or nil pointer dereference": result = 1
				case message == "interface conversion: ast.nodeData is *ast.ParenthesizedExpression, not *ast.ModuleBlock": result = 2
				case strings.HasPrefix(message, "runtime error: slice bounds out of range ") || strings.HasPrefix(message, "runtime error: index out of range "): result = 3
				case message == "Debug failure. Unexpected reparser-transformed node kind\nNode KindNumericLiteral was unexpected.": result = 4
				default: t.Fatalf("unclassified helper panic %s: %T: %v", label, recovered, recovered)
				}
			}
		}()
		callback()
		return
	}
	f := ast.NewNodeFactory(ast.NodeFactoryHooks{})
	source := func(text string) *ast.SourceFile {
		return f.NewSourceFile(ast.SourceFileParseOptions{FileName: "/file.ts"}, text, nil, nil).AsSourceFile()
	}
	node := func(kind ast.Kind, pos, end int) *ast.Node {
		n := f.NewToken(kind)
		n.Loc = core.NewTextRange(pos, end)
		return n
	}
	ordinary := " /*c*/ name\xff"
	file := source(ordinary)
	name := f.NewIdentifier("cooked")
	name.Loc = core.NewTextRange(0, len(ordinary))
	name.Parent = file.AsNode()
	text("text/trivia", GetSourceTextOfNodeFromSourceFile(file, name, true))
	text("text/trim", GetSourceTextOfNodeFromSourceFile(file, name, false))
	text("name/real", DeclarationNameToString(name))
	text("name/nil", DeclarationNameToString(nil))
	text("name/synthetic", DeclarationNameToString(f.NewIdentifier("x")))
	text("text/nil", GetTextOfNodeFromSourceText(ordinary, nil, false))
	text("text/missing", GetTextOfNodeFromSourceText(ordinary, node(ast.KindIdentifier, 2, 2), false))
	jsdoc := " * A\r\n * B\xe2\x80\xa8 * C"
	doc := f.NewJSDocTypeExpression(nil)
	doc.Loc = core.NewTextRange(0, len(jsdoc))
	text("text/jsdoc", GetTextOfNodeFromSourceText(jsdoc, doc, true))
	typ := node(ast.KindNumberKeyword, 0, len(jsdoc))
	typ.Flags = ast.NodeFlagsReparsed
	text("text/reparsed-type", GetTextOfNodeFromSourceText(jsdoc, typ, true))
	for i, flags := range []ast.TokenFlags{0, ast.TokenFlagsSingleQuote} {
		literal := f.NewStringLiteral("cooked", flags)
		literal.Loc = core.NewTextRange(0, 2)
		literal.Flags = ast.NodeFlagsReparserTransformedLiteral
		text(fmt.Sprintf("text/transformed/%d", i), GetTextOfNodeFromSourceText("x\xff", literal, false))
	}
	transformed := f.NewIdentifier("cooked\xff")
	transformed.Loc = core.NewTextRange(0, 2)
	transformed.Flags = ast.NodeFlagsReparserTransformedLiteral
	text("text/transformed-id", GetTextOfNodeFromSourceText("xx", transformed, false))
	bad := node(ast.KindNumericLiteral, 0, 1)
	bad.Flags = ast.NodeFlagsReparserTransformedLiteral
	emit("text/transformed-panic", panics("text/transformed-panic", func() { GetTextOfNodeFromSourceText("1", bad, false) }))
	emit("text/synthetic-panic", panics("text/synthetic-panic", func() { GetTextOfNodeFromSourceText("x", node(ast.KindIdentifier, -1, -1), false) }))
	for i, pos := range []int{0, 2, len(ordinary), len(ordinary) + 3} {
		span(fmt.Sprintf("token/%d", i), GetRangeOfTokenAtPosition(file, pos))
	}
	emit("token/negative", panics("token/negative", func() { GetRangeOfTokenAtPosition(file, -1) }))
	span("error/source", GetErrorRangeForNode(file, file.AsNode()))
	empty := source(" /*c*/ \r\n")
	span("error/empty", GetErrorRangeForNode(empty, empty.AsNode()))
	jsxt := node(ast.KindJsxText, 0, len(ordinary))
	span("error/jsx", GetErrorRangeForNode(file, jsxt))
	missing := node(ast.KindIdentifier, 2, 2)
	span("error/missing", GetErrorRangeForNode(file, missing))
	declaration := f.NewVariableDeclaration(name, nil, nil, nil)
	declaration.Loc = core.NewTextRange(0, len(ordinary))
	span("error/declaration", GetErrorRangeForNode(file, declaration))
	class := f.NewClassExpression(nil, nil, nil, nil, nil)
	class.Loc = core.NewTextRange(0, len(ordinary))
	span("error/nameless-class", GetErrorRangeForNode(file, class))
	for i, kind := range []ast.Kind{ast.KindReturnStatement, ast.KindYieldExpression} {
		s := source(" /*c*/ return answer")
		span(fmt.Sprintf("error/keyword/%d", i), GetErrorRangeForNode(s, node(kind, 0, len(s.Text()))))
	}
	arrowText := " /*c*/ () => {\r\n answer\n}"
	af := source(arrowText)
	body := f.NewBlock(nil, false)
	body.Loc = core.NewTextRange(12, len(arrowText))
	arrow := f.NewArrowFunction(nil, nil, nil, nil, nil, nil, body)
	arrow.Loc = core.NewTextRange(0, len(arrowText))
	span("error/arrow", GetErrorRangeForNode(af, arrow))
	ctorText := " public /*x*/ constructor() {}"
	cf := source(ctorText)
	ctor := f.NewConstructorDeclaration(nil, nil, nil, nil, nil, nil)
	ctor.Loc = core.NewTextRange(0, len(ctorText))
	span("error/constructor", GetErrorRangeForNode(cf, ctor))
	ctor.Flags = ast.NodeFlagsReparsed
	span("error/constructor-reparsed", GetErrorRangeForNode(cf, ctor))
	satisfied := source("first second x satisfies T")
	expr := node(ast.KindIdentifier, 13, 14)
	target := node(ast.KindNumberKeyword, 99, 100)
	target.Flags = ast.NodeFlagsReparsed
	sat := f.NewSatisfiesExpression(expr, target)
	firstName := f.NewIdentifier("first")
	firstName.Loc = core.NewTextRange(0, 5)
	secondName := f.NewIdentifier("second")
	secondName.Loc = core.NewTextRange(6, 12)
	firstType := node(ast.KindNumberKeyword, 90, 91)
	first := f.NewJSDocSatisfiesTag(firstName, f.NewJSDocTypeExpression(firstType), nil)
	second := f.NewJSDocSatisfiesTag(secondName, f.NewJSDocTypeExpression(target), nil)
	root := f.NewJSDoc(nil, f.NewNodeList([]*ast.Node{first, second}))
	parent := node(ast.KindUnknown, 0, 0)
	parent.Flags = ast.NodeFlagsHasJSDoc
	sat.Parent = parent
	satisfied.SetJSDocCache(map[*ast.Node][]*ast.Node{parent: {root}})
	span("error/satisfies-match", GetErrorRangeForNode(satisfied, sat))
	target.Loc = core.NewTextRange(80, 81) // Target is shared by the matching tag, so change that tag's type separately.
	second.AsJSDocSatisfiesTag().TypeExpression = f.NewJSDocTypeExpression(firstType)
	span("error/satisfies-first", GetErrorRangeForNode(satisfied, sat))
	satisfied.SetJSDocCache(nil)
	span("error/satisfies-no-eager", GetErrorRangeForNode(satisfied, sat))
	target.Flags = 0
	span("error/satisfies-native", GetErrorRangeForNode(satisfied, sat))
	panicJSON, err := json.Marshal(panicPayloads)
	if err != nil { t.Fatal(err) }
	if err := os.WriteFile(os.Getenv("S07_SCANNER_HELPERS_OUTPUT")+".panics.json", panicJSON, 0600); err != nil { t.Fatal(err) }
	if err := os.WriteFile(os.Getenv("S07_SCANNER_HELPERS_OUTPUT"), output.Bytes(), 0600); err != nil {
		t.Fatal(err)
	}
}
