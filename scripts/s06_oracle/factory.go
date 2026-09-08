// Access-only witnesses: these scripts construct inputs and call unchanged Go APIs.
package main

import (
	"encoding/hex"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/api/encoder"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"os"
)

var factoryScenarios = []string{
	"hooks-counts-update-clone", "raw-slice-same", "visitor-nil-flatten-disable",
	"visitor-lift-contracts", "visitor-role-hooks", "deep-clone-locations-and-parents",
	"subtree-cache-and-exclusions", "token-subtree-and-precedence", "source-file-clone-omissions", "source-cache-panic-once", "node-index-nil-after-sort", "source-file-hooks",
}

func factoryStages(scenario string) []string {
	for _, name := range factoryScenarios {
		if name == scenario {
			return []string{"factory"}
		}
	}
	return nil
}
func witness(s *session, label string, ints []int64, bools []bool, strings []string) {
	if ints == nil {
		ints = []int64{}
	}
	if bools == nil {
		bools = []bool{}
	}
	if strings == nil {
		strings = []string{}
	}
	s.observe("factory", "factory", map[string]any{"label": label, "ints": ints, "bools": bools, "strings": strings})
}
func executeFactory(s *session, r request) {
	s.stage("factory", func() error { factoryWitness(s, r.data["scenario"].(string)); return nil })
}
func factoryWitness(s *session, scenario string) {
	f := ast.NewNodeFactory(ast.NodeFactoryHooks{})
	switch scenario {
	case "source-file-hooks":
		sourceFileHooksWitness(s)
	case "hooks-counts-update-clone":
		ids := map[*ast.Node]int64{}
		var next int64
		id := func(n *ast.Node) int64 {
			if n == nil {
				return 0
			}
			return ids[n]
		}
		var factory *ast.NodeFactory
		event := func(name string, node, original *ast.Node) {
			witness(s, name, []int64{id(node), id(original), int64(node.Kind), int64(node.Flags), int64(node.Pos()), int64(node.End()), int64(factory.NodeCount()), int64(factory.TextCount())}, nil, nil)
		}
		nested := false
		factory = ast.NewNodeFactory(ast.NodeFactoryHooks{
			OnCreate: func(node *ast.Node) {
				next++
				ids[node] = next
				event("create", node, nil)
				node.Flags = ast.NodeFlags(^uint32(0))
				if !nested && node.Kind == ast.KindIdentifier {
					nested = true
					factory.NewToken(ast.KindUnknown)
				}
			},
			OnUpdate: func(node, original *ast.Node) {
				event("update", node, original)
				original.Loc = core.NewTextRange(20, 21)
			},
			OnClone: func(node, original *ast.Node) { event("clone", node, original) },
		})
		name := factory.NewIdentifier("x")
		name.Loc = core.NewTextRange(1, 2)
		name.Flags = 3
		dot := factory.NewToken(ast.KindQuestionDotToken)
		access := factory.NewPropertyAccessExpression(name, dot, name, ast.NodeFlagsOptionalChain)
		access.Loc = core.NewTextRange(3, 9)
		access.Flags = 7
		witness(s, "constructed", []int64{id(name), id(access), int64(access.Flags), int64(factory.NodeCount()), int64(factory.TextCount())}, nil, nil)
		same := factory.UpdatePropertyAccessExpression(access.AsPropertyAccessExpression(), name, dot, name, access.Flags)
		clone := access.Clone(factory)
		witness(s, "result", []int64{id(clone), int64(clone.Pos()), int64(clone.End()), int64(access.Pos()), int64(access.End()), int64(factory.NodeCount()), int64(factory.TextCount())}, []bool{same == access}, nil)
	case "raw-slice-same":
		a := f.NewIdentifier("a")
		raw := []*ast.Node{a, nil}
		original := f.NewSyntaxList(raw)
		same := f.UpdateSyntaxList(original.AsSyntaxList(), raw)
		copied := f.UpdateSyntaxList(original.AsSyntaxList(), append([]*ast.Node{}, raw...))
		empty := f.NewSyntaxList(make([]*ast.Node, 0))
		sameEmpty := f.UpdateSyntaxList(empty.AsSyntaxList(), nil)
		witness(s, "same", []int64{int64(f.NodeCount()), int64(f.TextCount())}, []bool{same == original, copied == original, sameEmpty == empty, empty.AsSyntaxList().Children == nil}, nil)
	case "visitor-nil-flatten-disable":
		a := f.NewIdentifier("a")
		c := f.NewIdentifier("c")
		syntax := f.NewSyntaxList([]*ast.Node{c, nil})
		input := []*ast.Node{nil, a, a, c}
		calls := 0
		var v *ast.NodeVisitor
		v = ast.NewNodeVisitor(func(node *ast.Node) *ast.Node {
			calls++
			if calls == 3 {
				v.Visit = nil
			}
			if node == a {
				return syntax
			}
			return node
		}, f, ast.NodeVisitorHooks{})
		nodes, changed := v.VisitSlice(input)
		values := []int64{int64(calls), int64(len(nodes))}
		for _, n := range nodes {
			if n == nil {
				values = append(values, 0)
			} else if n == c {
				values = append(values, 2)
			} else {
				values = append(values, 1)
			}
		}
		witness(s, "slice", values, []bool{changed, nodes == nil}, nil)
		nodes, changed = v.VisitSlice(nil)
		witness(s, "nil", []int64{int64(len(nodes))}, []bool{changed, nodes == nil}, nil)
	case "visitor-lift-contracts":
		a := f.NewIdentifier("a")
		syntax := f.NewSyntaxList([]*ast.Node{a})
		v := ast.NewNodeVisitor(func(n *ast.Node) *ast.Node { return n }, f, ast.NodeVisitorHooks{})
		input := []*ast.Node{syntax}
		result, changed := v.VisitSlice(input)
		lifted := v.VisitNode(syntax)
		witness(s, "unchanged", []int64{int64(len(result))}, []bool{changed, result[0] == syntax, lifted == a}, nil)
		for _, entry := range []struct {
			name  string
			nodes []*ast.Node
		}{{"empty", nil}, {"nil_child", []*ast.Node{nil}}, {"nested", []*ast.Node{syntax}}} {
			wrapper := f.NewSyntaxList(entry.nodes)
			var out *ast.Node
			outcome, message := capture(func() error { out = v.VisitNode(wrapper); return nil })
			witness(s, entry.name, nil, []bool{out == nil}, []string{outcome, message})
		}
	case "visitor-role-hooks":
		token := f.NewToken(ast.KindThisKeyword)
		expr := f.NewIdentifier("x")
		loop := f.NewWhileStatement(expr, token)
		remove := func(*ast.Node, *ast.NodeVisitor) *ast.Node { return nil }
		v := ast.NewNodeVisitor(func(n *ast.Node) *ast.Node { return n }, f, ast.NodeVisitorHooks{VisitNode: remove})
		direct := v.VisitEmbeddedStatement(token)
		updated := v.VisitEachChild(loop).AsWhileStatement()
		block := updated.Statement.AsBlock()
		witness(s, "embedded", []int64{int64(updated.Statement.Kind), int64(len(block.Statements.Nodes))}, []bool{direct == token, updated.Expression == nil}, nil)
		access := f.NewPropertyAccessExpression(expr, token, expr, 0)
		rewritten := v.VisitEachChild(access).AsPropertyAccessExpression()
		witness(s, "token", nil, []bool{rewritten.QuestionDotToken == token, rewritten.Expression == nil, rewritten.Name() == nil}, nil)
	case "deep-clone-locations-and-parents":
		child := f.NewIdentifier("child")
		child.Loc = core.NewTextRange(1, 6)
		list := f.NewNodeList([]*ast.Node{child})
		list.Loc = core.NewTextRange(0, 7)
		root := f.NewArrayLiteralExpression(list, false)
		root.Loc = core.NewTextRange(0, 8)
		destination := ast.NewNodeFactory(ast.NodeFactoryHooks{})
		cloned := destination.DeepCloneNode(root)
		cl := cloned.AsArrayLiteralExpression().Elements
		cc := cl.Nodes[0]
		witness(s, "synthetic", []int64{int64(cloned.Pos()), int64(cloned.End()), int64(cl.Pos()), int64(cl.End()), int64(cc.Pos()), int64(cc.End()), int64(child.Pos()), int64(child.End())}, []bool{cloned == root, cc == child, cl.HasTrailingComma(), cc.Parent == nil}, nil)
		reparsed := destination.DeepCloneReparse(root)
		rc := reparsed.AsArrayLiteralExpression().Elements.Nodes[0]
		witness(s, "reparse", []int64{int64(reparsed.Pos()), int64(reparsed.End()), int64(rc.Pos()), int64(rc.End()), int64(reparsed.Flags)}, []bool{rc.Parent == reparsed, reparsed.Parent == nil}, nil)
		empty := f.NewArrayLiteralExpression(f.NewNodeList(nil), false)
		emptyClone := destination.DeepCloneNode(empty)
		witness(s, "empty", nil, []bool{emptyClone != empty, emptyClone.AsArrayLiteralExpression().Elements != empty.AsArrayLiteralExpression().Elements, emptyClone.AsArrayLiteralExpression().Elements.Nodes == nil}, nil)
	case "subtree-cache-and-exclusions":
		this := f.NewKeywordExpression(ast.KindThisKeyword)
		ident := f.NewIdentifier("x")
		outer := f.NewComputedPropertyName(this)
		before := outer.SubtreeFacts()
		outer.AsComputedPropertyName().Expression = ident
		after := outer.SubtreeFacts()
		cloned := outer.Clone(f)
		witness(s, "cache", []int64{int64(before), int64(after), int64(cloned.SubtreeFacts())}, nil, nil)
		awaited := f.NewAwaitExpression(this)
		function := f.NewFunctionExpression(nil, nil, nil, nil, nil, nil, nil, awaited)
		arrow := f.NewArrowFunction(nil, nil, nil, nil, nil, nil, awaited)
		parentFunction := f.NewComputedPropertyName(function)
		parentArrow := f.NewComputedPropertyName(arrow)
		witness(s, "scope", []int64{int64(function.SubtreeFacts()), int64(parentFunction.SubtreeFacts()), int64(arrow.SubtreeFacts()), int64(parentArrow.SubtreeFacts())}, nil, nil)
		malformed := f.NewCallExpression(nil, nil, nil, nil, 0)
		declaration := f.NewVariableDeclaration(nil, nil, malformed, nil)
		witness(s, "erased", []int64{int64(declaration.SubtreeFacts())}, nil, nil)
		outcome, message := capture(func() error { malformed.SubtreeFacts(); return nil })
		class := message
		if outcome == "panic" {
			fmt.Fprintln(os.Stderr, "factory subtree nil witness:", message)
			if message == "runtime error: invalid memory address or nil pointer dereference" {
				class = "nil-dereference"
			}
		}
		witness(s, "invalid", nil, nil, []string{outcome, class})
	case "token-subtree-and-precedence":
		kinds := []int{-32768, -1}
		for i := 0; i <= int(ast.KindCount); i++ {
			kinds = append(kinds, i)
		}
		kinds = append(kinds, 32767)
		for _, raw := range kinds {
			kind := ast.Kind(raw)
			token := f.NewToken(kind)
			witness(s, "token", []int64{int64(raw), int64(token.SubtreeFacts()), int64(ast.GetBinaryOperatorPrecedence(kind))}, nil, nil)
		}
	case "source-file-clone-omissions":
		eof := f.NewToken(ast.KindEndOfFile)
		node := f.NewSourceFile(ast.SourceFileParseOptions{FileName: "/fixture.ts"}, "x\xff", nil, eof)
		sf := node.AsSourceFile()
		node.Loc = core.NewTextRange(0, 2)
		node.Flags = 7
		sf.ScriptKind = core.ScriptKindTS
		sf.IdentifierCount = 3
		sf.NodeCount = 7
		sf.TextCount = 11
		sf.Hash.Hi = 13
		sf.Hash.Lo = 17
		sf.CheckJsDirective = &ast.CheckJsDirective{Enabled: true}
		sf.ExternalModuleIndicator = node
		sf.SetHasLazyJSDoc(true)
		doc := f.NewToken(ast.KindUnknown)
		eof.Flags |= ast.NodeFlagsHasJSDoc
		sf.SetJSDocCache(map[*ast.Node][]*ast.Node{eof: {doc}})
		unchanged := f.UpdateSourceFile(sf, nil, eof)
		cloned := node.Clone(f).AsSourceFile()
		witness(s, "source", []int64{int64(cloned.Kind), int64(cloned.Flags), int64(cloned.Pos()), int64(cloned.End()), int64(cloned.ScriptKind), int64(cloned.IdentifierCount), int64(cloned.NodeCount), int64(cloned.TextCount), int64(cloned.Hash.Hi), int64(cloned.Hash.Lo)}, []bool{unchanged == node, cloned.CheckJsDirective == nil, cloned.ExternalModuleIndicator == node, cloned.EndOfFileToken == eof}, []string{hex.EncodeToString([]byte(cloned.Text())), cloned.FileName()})
		originalDocs := eof.EagerJSDoc(sf)
		clonedDocs := eof.EagerJSDoc(cloned)
		witness(s, "source-jsdoc-cache", []int64{int64(len(originalDocs)), int64(len(clonedDocs))}, []bool{len(originalDocs) == 1 && originalDocs[0] == doc}, nil)
	case "node-index-nil-after-sort":
		a, b, c := f.NewToken(ast.KindUnknown), f.NewToken(ast.KindUnknown), f.NewToken(ast.KindUnknown)
		table := &encoder.NodeIndexTable{Nodes: []*ast.Node{nil, a, b, a, c, nil}}
		outcome, message := capture(func() error { table.GetIndex(nil); return nil })
		fmt.Fprintln(os.Stderr, "factory node-index nil witness:", message)
		class := message
		if outcome == "panic" && message == "runtime error: invalid memory address or nil pointer dereference" {
			class = "nil-query-after-sort"
		}
		witness(s, "nil-query", nil, nil, []string{outcome, class})
		ai, bi, ci := ast.GetNodeId(a), ast.GetNodeId(b), ast.GetNodeId(c)
		witness(s, "after-sort", []int64{int64(table.GetIndex(a)), int64(table.GetIndex(b)), int64(table.GetIndex(c))}, []bool{ai < bi, ai < ci, bi < ci}, nil)
		witness(s, "absent", []int64{int64(table.GetIndex(f.NewToken(ast.KindUnknown)))}, nil, nil)

	case "source-cache-panic-once":
		sf := f.NewSourceFile(ast.SourceFileParseOptions{FileName: "/cache.ts"}, "", nil, nil).AsSourceFile()
		key := ast.NewSourceFileDataKey[*int]()
		calls := 0
		outcome, message := capture(func() error {
			ast.GetOrComputeSourceFileData(sf, key, func(*ast.SourceFile) *int { calls++; panic("source initializer failed") })
			return nil
		})
		witness(s, "first", []int64{int64(calls)}, nil, []string{outcome, message})
		result := ast.GetOrComputeSourceFileData(sf, key, func(*ast.SourceFile) *int { calls++; value := 7; return &value })
		witness(s, "second", []int64{int64(calls)}, []bool{result == nil}, nil)
	default:
		panic(fmt.Sprintf("unknown factory scenario %q", scenario))
	}
}

func sourceFileHooksWitness(s *session) {
	var factory *ast.NodeFactory
	var original *ast.Node
	var phase int64
	observe := func(label string, node *ast.Node) {
		file := node.AsSourceFile()
		witness(s, label, []int64{phase, int64(file.ScriptKind), int64(file.IdentifierCount), int64(file.NodeCount), int64(file.TextCount), int64(file.Hash.Hi), int64(file.Hash.Lo), int64(len(file.Diagnostics())), int64(node.Flags), int64(node.Pos()), int64(node.End()), int64(factory.NodeCount()), int64(factory.TextCount())}, nil, []string{hex.EncodeToString([]byte(file.FileName())), hex.EncodeToString([]byte(file.Text()))})
	}
	factory = ast.NewNodeFactory(ast.NodeFactoryHooks{
		OnCreate: func(node *ast.Node) {
			if node.Kind != ast.KindSourceFile {
				return
			}
			phase++
			observe("source-create", node)
			file := node.AsSourceFile()
			file.ScriptKind = core.ScriptKindJSX
			file.IdentifierCount = int(phase * 100)
			file.NodeCount = int(phase * 10)
			file.TextCount = int(phase * 20)
			file.Hash.Hi = uint64(phase)
			file.Hash.Lo = uint64(phase + 1)
			file.SetDiagnostics([]*ast.Diagnostic{ast.NewExternalDiagnostic(nil, core.TextRange{}, "hook", 1, 42, "hook")})
			if original != nil {
				original.AsSourceFile().ScriptKind = core.ScriptKind(phase * 2)
				original.AsSourceFile().IdentifierCount = 999
			}
		},
		OnUpdate: func(node, original *ast.Node) { observe("source-update", node) },
		OnClone:  func(node, original *ast.Node) { observe("source-clone", node) },
	})
	original = factory.NewSourceFile(ast.SourceFileParseOptions{FileName: "/hooks.ts"}, "source\xff", nil, nil)
	original.Flags = 7
	original.Loc = core.NewTextRange(4, 9)
	original.AsSourceFile().ScriptKind = core.ScriptKindJS
	cloned := original.Clone(factory)
	eof := factory.NewToken(ast.KindEndOfFile)
	updated := factory.UpdateSourceFile(original.AsSourceFile(), nil, eof)
	observe("source-original-result", original)
	observe("source-cloned-result", cloned)
	observe("source-updated-result", updated)
}
