package ast

import (
	"bytes"
	"encoding/json"
	"strings"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"os"
	"testing"
)

// These observations call the original helpers, never the Rust implementation.
func TestS07ASTHelpers(t *testing.T) {
	var output bytes.Buffer
	emit := func(label string, values ...int64) {
		fmt.Fprint(&output, label)
		for _, v := range values {
			fmt.Fprintf(&output, "\t%d", v)
		}
		fmt.Fprintln(&output)
	}
	b := func(value bool) int64 {
		if value {
			return 1
		}
		return 0
	}
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
	f := NewNodeFactory(NodeFactoryHooks{})
	id := func(text string) *Node { return f.NewIdentifier(text) }
	prop := func(base *Node, name string) *Node { return f.NewPropertyAccessExpression(base, nil, id(name), 0) }
	assign := func(left, right *Node, js bool, operator Kind) *Node {
		if js {
			left.Flags |= NodeFlagsJavaScriptFile
		}
		return f.NewBinaryExpression(nil, left, nil, f.NewToken(operator), right)
	}
	for i, entry := range []struct {
		left, right *Node
		js          bool
		operator    Kind
	}{
		{prop(id("module"), "exports"), id("value"), true, KindEqualsToken},
		{prop(id("module"), "exports"), id("exports"), true, KindEqualsToken},
		{prop(id("exports"), "x"), id("value"), true, KindEqualsToken},
		{prop(prop(id("module"), "exports"), "x"), id("value"), true, KindEqualsToken},
		{prop(f.NewToken(KindThisKeyword), "x"), id("value"), true, KindEqualsToken},
		{prop(id("obj"), "x"), id("value"), false, KindEqualsToken},
		{prop(f.NewToken(KindThisKeyword), "x"), id("value"), false, KindEqualsToken},
		{f.NewElementAccessExpression(id("obj"), nil, id("dynamic"), 0), id("value"), false, KindEqualsToken},
		{prop(id("exports"), "x"), id("value"), true, KindPlusEqualsToken},
		{prop(id("module\xff"), "exports"), id("value"), true, KindEqualsToken},
	} {
		node := assign(entry.left, entry.right, entry.js, entry.operator)
		emit(fmt.Sprintf("assignment/%d", i), int64(GetAssignmentDeclarationKind(node)), b(IsAssignmentExpression(node, false)), b(IsAssignmentExpression(node, true)))
	}
	for i, target := range []*Node{id("exports"), prop(id("module"), "exports"), prop(id("obj"), "member"), f.NewToken(KindThisKeyword), f.NewNumericLiteral("1", 0)} {
		call := f.NewCallExpression(prop(id("Object"), "defineProperty"), nil, nil, f.NewNodeList([]*Node{target, f.NewStringLiteral("x", 0), id("descriptor")}), 0)
		call.Flags |= NodeFlagsJavaScriptFile
		emit(fmt.Sprintf("define/%d", i), b(IsBindableObjectDefinePropertyCall(call)), int64(GetAssignmentDeclarationKind(call)), b(GetNonAssignedNameOfDeclaration(call) != nil))
		call.Flags = 0
		emit(fmt.Sprintf("define-ts/%d", i), int64(GetAssignmentDeclarationKind(call)))
	}
	call := f.NewCallExpression(nil, nil, nil, nil, 0)
	emit("define/no-args", b(IsBindableObjectDefinePropertyCall(call)))
	this := f.NewToken(KindThisKeyword)
	element := f.NewElementAccessExpression(this, nil, f.NewStringLiteral("x", 0), 0)
	wrapped := f.NewParenthesizedExpression(element)
	emit("names", b(IsEntityNameExpressionEx(element, false)), b(IsEntityNameExpressionEx(element, true)), b(IsEntityNameExpressionEx(wrapped, true)), b(IsDottedName(wrapped)), b(IsBindableStaticNameExpression(element, false)), b(IsBindableStaticNameExpression(element, true)), b(IsPushOrUnshiftIdentifier(id("push\xff"))))
	module := func(body *Node) *Node { return f.NewModuleDeclaration(nil, KindNamespaceKeyword, id("M"), nil, body) }
	block := func(nodes ...*Node) *Node { return f.NewModuleBlock(f.NewNodeList(nodes)) }
	iface := func(name string) *Node { return f.NewInterfaceDeclaration(nil, id(name), nil, nil, nil) }
	enumeration := func(name string) *Node {
		return f.NewEnumDeclaration(f.NewModifierList([]*Node{f.NewToken(KindConstKeyword)}), id(name), nil)
	}
	value := func(name string) *Node {
		return f.NewVariableStatement(nil, f.NewVariableDeclarationList(f.NewNodeList([]*Node{f.NewVariableDeclaration(id(name), nil, nil, nil)}), 0))
	}
	alias := func(name string) *Node {
		return f.NewExportDeclaration(nil, false, f.NewNamedExports(f.NewNodeList([]*Node{f.NewExportSpecifier(false, nil, id(name))})), nil, nil)
	}
	for i, body := range []*Node{nil, block(), block(iface("X")), block(enumeration("X")), block(value("X"), nil), block(alias("X"), iface("X")), block(alias("X"), enumeration("X")), block(alias("X"), value("X")), block(alias("X")), block(alias("X"), f.NewImportEqualsDeclaration(nil, false, id("X"), id("Y")))} {
		m := module(body)
		emit(fmt.Sprintf("module/%d", i), int64(GetModuleInstanceState(m)), b(IsInstantiatedModule(m, false)), b(IsInstantiatedModule(m, true)))
	}
	cycle := module(nil)
	cycle.AsModuleDeclaration().Body = cycle
	emit("module/cycle", int64(GetModuleInstanceState(cycle)))
	emit("module/nil-child", panics("module/nil-child", func() { GetModuleInstanceState(module(block(nil))) }))
	// Generated ForEachChild requires the payload selected by the kind header.
	malformed := f.NewParenthesizedExpression(iface("X"))
	malformed.Kind = KindModuleBlock
	emit("module/payload-dispatch", panics("module/payload-dispatch", func() { GetModuleInstanceState(module(malformed)) }))
	decl := f.NewVariableDeclaration(id("x"), nil, nil, nil)
	typed := f.NewVariableDeclaration(id("x"), nil, f.NewToken(KindAnyKeyword), nil)
	object := f.NewObjectLiteralExpression(nil, false)
	object.Flags |= NodeFlagsJavaScriptFile
	class := f.NewClassExpression(nil, nil, nil, nil, nil)
	class.Flags |= NodeFlagsJavaScriptFile
	function := f.NewFunctionExpression(nil, nil, nil, nil, nil, nil, nil, nil)
	emit("expando", b(IsExpandoInitializer(decl, nil)), b(IsExpandoInitializer(decl, object)), b(IsExpandoInitializer(typed, object)), b(IsExpandoInitializer(typed, class)), b(IsExpandoInitializer(typed, function)))
	object.Flags = 0
	emit("expando/ts-object", b(IsExpandoInitializer(decl, object)))
	list := f.NewVariableDeclarationList(f.NewNodeList([]*Node{decl}), 0)
	decl.Parent = list
	statement := f.NewVariableStatement(nil, list)
	list.Parent = statement
	container := f.NewBlock(f.NewNodeList([]*Node{statement}), false)
	statement.Parent = container
	emit("container", b(GetDeclarationContainer(decl) == container))
	emit("container/unparented", panics("container/unparented", func() { GetDeclarationContainer(f.NewVariableDeclaration(id("x"), nil, nil, nil)) }))
	fn := f.NewFunctionExpression(nil, nil, nil, nil, nil, nil, nil, nil)
	wrapper := f.NewParenthesizedExpression(fn)
	fn.Parent = wrapper
	invocation := f.NewCallExpression(wrapper, nil, nil, nil, 0)
	wrapper.Parent = invocation
	emit("iife", b(GetImmediatelyInvokedFunctionExpression(fn) == invocation))
	emit("missing", b(NodeIsMissing(nil)), b(NodeIsPresent(f.NewIdentifier("x"))))
	typeParameter := f.NewToken(KindTypeParameter)
	tokenDeclaration := f.NewToken(KindVariableDeclaration)
	binaryDeclaration := f.NewBinaryExpression(nil, nil, nil, nil, nil)
	binaryDeclaration.Kind = KindUnknown
	emit("declaration/payload", b(IsDeclaration(typeParameter)), b(IsDeclaration(tokenDeclaration)), b(IsDeclaration(binaryDeclaration)))
	typeParameter.Parent = tokenDeclaration
	emit("declaration/type-parameter-parent", b(IsDeclaration(typeParameter)))
	noBody := f.NewConstructorDeclaration(nil, nil, nil, nil, nil, nil)
	missingBody := f.NewBlock(nil, false)
	missingBody.Loc = core.NewTextRange(0, 0)
	missingConstructor := f.NewConstructorDeclaration(nil, nil, nil, nil, nil, missingBody)
	presentBody := f.NewBlock(nil, false)
	presentConstructor := f.NewConstructorDeclaration(nil, nil, nil, nil, nil, presentBody)
	classWithConstructor := f.NewClassDeclaration(nil, nil, nil, nil, f.NewNodeList([]*Node{noBody, missingConstructor, presentConstructor}))
	emit("constructor", b(FindConstructorDeclaration(classWithConstructor) == presentConstructor))
	constX := enumeration("X")
	valueY := value("Y")
	outer := block(constX, valueY)
	for i, names := range [][]string{{"X", "Y"}, {"Y", "X"}} {
		var specifiers []*Node
		for _, name := range names {
			specifiers = append(specifiers, f.NewExportSpecifier(false, nil, id(name)))
		}
		export := f.NewExportDeclaration(nil, false, f.NewNamedExports(f.NewNodeList(specifiers)), nil, nil)
		m := module(export)
		m.Parent = outer
		emit(fmt.Sprintf("module/alias-order/%d", i), int64(GetModuleInstanceState(m)))
	}

	panicJSON, err := json.Marshal(panicPayloads)
	if err != nil { t.Fatal(err) }
	if err := os.WriteFile(os.Getenv("S07_AST_HELPERS_OUTPUT")+".panics.json", panicJSON, 0600); err != nil { t.Fatal(err) }
	if err := os.WriteFile(os.Getenv("S07_AST_HELPERS_OUTPUT"), output.Bytes(), 0600); err != nil {
		t.Fatal(err)
	}
}
