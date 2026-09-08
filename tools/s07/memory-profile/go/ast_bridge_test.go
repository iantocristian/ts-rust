package ast

import (
	"reflect"
	"strings"
	"testing"
)

func memoryTestFile() (*SourceFile, *Node, []*Node) {
	factory := &NodeFactory{}
	text := strings.Repeat("retained_name ", 8)
	identifier := factory.NewIdentifier(text[:13])
	backing := make([]*Node, 4)
	backing[0], backing[1] = identifier, identifier
	file := factory.NewSourceFile(SourceFileParseOptions{FileName: "/memory-test.ts"}, text,
		factory.NewNodeList(backing[:2:4]), factory.NewToken(KindEndOfFile)).AsSourceFile()
	identifier.Parent = file.AsNode()
	return file, identifier, backing
}

func TestMemoryCensusConcreteNodesContainHeaders(t *testing.T) {
	file, _, _ := memoryTestFile()
	result := MemoryProfileCensus(file)
	want := map[string]uint64{
		"ast.SourceFile": uint64(reflect.TypeOf(SourceFile{}).Size()),
		"ast.Identifier": uint64(reflect.TypeOf(Identifier{}).Size()),
		"ast.Token":      uint64(reflect.TypeOf(Token{}).Size()),
	}
	var nodes uint64
	for _, object := range result.Objects {
		if object.Type == "ast.Node" {
			t.Fatal("embedded Node counted as another allocation")
		}
		if object.Category != "concrete_node_including_header" {
			continue
		}
		if size, exists := want[object.Type]; !exists || object.Count != 1 || object.Size != size || object.LogicalBytes != size {
			t.Fatalf("unexpected concrete object: %+v", object)
		}
		nodes += object.Count
	}
	if nodes != 3 || result.NodeHeaderContainedBytes != 3*uint64(reflect.TypeOf(Node{}).Size()) {
		t.Fatalf("wrong contained header count: nodes=%d bytes=%d", nodes, result.NodeHeaderContainedBytes)
	}
	if result.Strings.SourceVisibleUnionBytes != uint64(len(file.text)) || result.Strings.SourceBackedFields < 2 {
		t.Fatalf("source text aliases were lost or double counted: %+v", result.Strings)
	}
}

func TestMemoryCensusCyclesAndSliceAliases(t *testing.T) {
	file, identifier, backing := memoryTestFile()
	symbol := &Symbol{Name: file.text[:13], Declarations: backing[1:2:3]}
	table := SymbolTable{file.text[:13]: symbol}
	symbol.Parent, symbol.ExportSymbol = symbol, symbol
	symbol.Members, symbol.Exports = table, table
	file.Symbol, file.Locals, file.GlobalExports = symbol, table, table
	flow := &FlowNode{Flags: FlowFlagsStart, Node: identifier}
	list := &FlowList{Flow: flow}
	list.Next, flow.Antecedents, flow.Antecedent = list, list, flow
	identifier.AsIdentifier().FlowNode = flow
	result := MemoryProfileCensus(file)
	want := map[string]uint64{"ast.Symbol": 1, "ast.FlowNode": 1, "ast.FlowList": 1}
	for _, object := range result.Objects {
		if count, exists := want[object.Type]; exists {
			if object.Count != count {
				t.Fatalf("shared/cyclic object counted more than once: %+v", object)
			}
			delete(want, object.Type)
		}
	}
	if len(want) != 0 {
		t.Fatalf("missing reachable categories: %v", want)
	}
	if len(result.Maps) != 1 || result.Maps[0].Maps != 1 || result.Maps[0].Entries != 1 {
		t.Fatalf("shared symbol table counted more than once: %+v", result.Maps)
	}
	found := false
	for _, slice := range result.Slices {
		if slice.Type != "[]*ast.Node" {
			continue
		}
		found = true
		// Two headers see overlapping capacity ranges [0,4) and [1,3).
		if slice.LengthElements != 3 || slice.CapacityElements != 6 || slice.VisibleCapacityUnion != 4*uint64(reflect.TypeOf((*Node)(nil)).Size()) {
			t.Fatalf("slice backing aliases incorrectly summed: %+v", slice)
		}
	}
	if !found || result.Strings.SourceVisibleUnionBytes != uint64(len(file.text)) || result.Strings.SourceBackedFields < 4 {
		t.Fatalf("missing shared slice/source string backing: %+v", result)
	}
}

func TestMemoryCensusRanges(t *testing.T) {
	ranges := []memoryRange{{20, 24}, {4, 8}, {5, 7}, {8, 12}, {4, 4}}
	if got := memoryUnion(ranges); got != 12 {
		t.Fatalf("overlap/adjacency union = %d, want 12", got)
	}
	t.Run("overflow", func(t *testing.T) {
		defer func() {
			if recover() == nil {
				t.Fatal("overflow must fail")
			}
		}()
		memoryExtent(^uintptr(0), 1, 1)
	})
}
