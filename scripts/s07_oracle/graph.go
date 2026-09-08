package main

import (
	"encoding/hex"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"reflect"
	"sort"
	"strconv"
	"unsafe"
)

type queued struct {
	kind  string
	id    int
	value any
}
type graph struct {
	source          *ast.SourceFile
	queue           []queued
	nodes           map[*ast.Node]int
	lists           map[*ast.NodeList]int
	nodeSlices      map[string]int
	textSlices      map[string]int
	symbols         map[*ast.Symbol]int
	tables          map[string]int
	declarations    map[string]int
	flows           map[*ast.FlowNode]int
	flowLists       map[*ast.FlowList]int
	syntheticOwners map[*ast.Node]*ast.FlowNode
}

func newGraph(source *ast.SourceFile) *graph {
	return &graph{source: source, nodes: map[*ast.Node]int{}, lists: map[*ast.NodeList]int{}, nodeSlices: map[string]int{}, textSlices: map[string]int{}, symbols: map[*ast.Symbol]int{}, tables: map[string]int{}, declarations: map[string]int{}, flows: map[*ast.FlowNode]int{}, flowLists: map[*ast.FlowList]int{}, syntheticOwners: map[*ast.Node]*ast.FlowNode{}}
}

// Binder's only synthetic producers immediately attach a fresh payload to one
// fresh FlowNode. Reject broader producers instead of erasing shared identity.
func (g *graph) synthetic(flow *ast.FlowNode, id int, kind string) any {
	if owner, found := g.syntheticOwners[flow.Node]; found && owner != flow {
		panic("S07 observer: synthetic flow payload has multiple owners")
	}
	g.syntheticOwners[flow.Node] = flow
	return map[string]any{"owner_flow": id, "payload": kind, "kind": int16(flow.Node.Kind), "flags": uint32(flow.Node.Flags), "pos": flow.Node.Pos(), "end": flow.Node.End(), "parent": g.nodeRef(flow.Node.Parent)}
}
func ref[T comparable](g *graph, kind string, table map[T]int, key T, value any) int {
	if id, ok := table[key]; ok {
		return id
	}
	id := len(table) + 1
	table[key] = id
	g.queue = append(g.queue, queued{kind, id, value})
	return id
}
func (g *graph) nodeRef(value *ast.Node) int {
	if value == nil {
		return 0
	}
	return ref(g, "node", g.nodes, value, value)
}

type listValue struct {
	list       *ast.NodeList
	modifiers  uint32
	isModifier bool
}

func (g *graph) listRef(value *ast.NodeList, modifiers uint32, isModifier bool) int {
	if value == nil {
		return 0
	}
	return ref(g, "list", g.lists, value, listValue{value, modifiers, isModifier})
}
func sliceKey[T any](value []T) string { return fmt.Sprintf("%p/%d", value, len(value)) }
func (g *graph) nodeSliceRef(value []*ast.Node) int {
	if value == nil {
		return 0
	}
	return ref(g, "nodes", g.nodeSlices, sliceKey(value), value)
}
func (g *graph) textSliceRef(value []string) int {
	if value == nil {
		return 0
	}
	return ref(g, "texts", g.textSlices, sliceKey(value), value)
}
func (g *graph) symbolRef(value *ast.Symbol) int {
	if value == nil {
		return 0
	}
	return ref(g, "symbol", g.symbols, value, value)
}
func (g *graph) tableRef(value ast.SymbolTable) int {
	if value == nil {
		return 0
	}
	return ref(g, "table", g.tables, fmt.Sprintf("%p", value), value)
}
func (g *graph) declarationRef(value []*ast.Node) int {
	if value == nil {
		return 0
	}
	return ref(g, "declarations", g.declarations, fmt.Sprintf("%s/%d", sliceKey(value), cap(value)), value)
}
func (g *graph) flowRef(value *ast.FlowNode) int {
	if value == nil {
		return 0
	}
	return ref(g, "flow", g.flows, value, value)
}
func (g *graph) flowListRef(value *ast.FlowList) int {
	if value == nil {
		return 0
	}
	return ref(g, "flow_list", g.flowLists, value, value)
}
func hx(value string) string { return hex.EncodeToString([]byte(value)) }
func (g *graph) nodeRefs(nodes []*ast.Node) []int {
	values := make([]int, len(nodes))
	for i, node := range nodes {
		values[i] = g.nodeRef(node)
	}
	return values
}
func (g *graph) syntax(node *ast.Node) (string, []any) {
	payload, fields := ast.S07SyntaxFields(node)
	values := make([]any, 0, len(fields))
	for _, field := range fields {
		var value any
		switch field.Kind {
		case "node":
			value = g.nodeRef(field.Value.(*ast.Node))
		case "list":
			value = g.listRef(field.Value.(*ast.NodeList), 0, false)
		case "modifiers":
			list := field.Value.(*ast.ModifierList)
			if list == nil {
				value = 0
			} else {
				value = g.listRef(&list.NodeList, uint32(list.ModifierFlags), true)
			}
		case "nodes":
			value = g.nodeSliceRef(field.Value.([]*ast.Node))
		case "strings":
			value = g.textSliceRef(field.Value.([]string))
		case "text":
			value = hx(field.Value.(string))
		case "bool":
			value = field.Value.(bool)
		case "kind", "number":
			value = reflect.ValueOf(field.Value).Int()
		case "opaque":
			if field.Value != nil {
				panic("S07 observer: checker payload reached binder graph")
			}
			value = nil
		default:
			panic("S07 observer: unknown syntax field category")
		}
		values = append(values, []any{field.Name, field.Kind, value})
	}
	return payload, values
}
func (g *graph) name(raw string, symbol *ast.Symbol) any {
	result := map[string]any{"raw_hex": hx(raw), "identity": nil}
	if symbol == nil {
		return result
	}
	for _, declaration := range symbol.Declarations {
		if declaration == nil {
			continue
		}
		name := ast.GetNameOfDeclaration(declaration)
		if name == nil {
			continue
		}
		if ast.IsAmbientModule(declaration) {
			module := declaration.AsModuleDeclaration()
			pattern := core.TryParsePattern(name.Text())
			if module.Attributes != nil && pattern.IsValid() && pattern.StarIndex >= 0 {
				prefix := "\xfe\"" + name.Text() + "\"pattern@"
				numeric := strconv.FormatUint(ast.S07RawNodeID(module.Attributes), 10)
				if numeric != "0" && raw == prefix+numeric {
					result["identity"] = map[string]any{"kind": "node", "ref": g.nodeRef(module.Attributes), "prefix_hex": hx(prefix), "suffix_hex": ""}
					return result
				}
			}
		}
		if ast.IsPrivateIdentifier(name) {
			class := ast.GetContainingClass(declaration)
			if class == nil || class.Symbol() == nil {
				continue
			}
			owner := class.Symbol()
			prefix := "\xfe#"
			suffix := "@" + name.Text()
			numeric := strconv.FormatUint(ast.S07RawSymbolID(owner), 10)
			if numeric != "0" && raw == prefix+numeric+suffix {
				result["identity"] = map[string]any{"kind": "symbol", "ref": g.symbolRef(owner), "prefix_hex": hx(prefix), "suffix_hex": hx(suffix)}
				return result
			}
		}
	}
	return result
}
func (g *graph) diagnostic(d *ast.Diagnostic) any {
	if d == nil {
		return nil
	}
	args := make([]string, len(d.MessageArgs()))
	for i, arg := range d.MessageArgs() {
		args[i] = hx(arg)
	}
	chain := make([]any, len(d.MessageChain()))
	for i, item := range d.MessageChain() {
		chain[i] = g.diagnostic(item)
	}
	related := make([]any, len(d.RelatedInformation()))
	for i, item := range d.RelatedInformation() {
		related[i] = g.diagnostic(item)
	}
	file := 0
	if d.File() != nil {
		file = g.nodeRef(d.File().AsNode())
	}
	if d.RepopulateInfo() != nil {
		panic("S07 observer: unexpected checker diagnostic repopulation data")
	}
	return map[string]any{"file": file, "pos": d.Pos(), "end": d.End(), "code": d.Code(), "category": d.Category(), "source_hex": hx(d.Source()), "key_hex": hx(string(d.MessageKey())), "text_hex": hx(d.MessageText()), "args_hex": args, "chain": chain, "related": related, "unnecessary": d.ReportsUnnecessary(), "deprecated": d.ReportsDeprecated(), "skipped": d.SkippedOnNoEmit()}
}
func (g *graph) sourceRecord() any {
	file := g.source
	root := g.nodeRef(file.AsNode())
	patterns := make([]any, len(file.PatternAmbientModules))
	for i, item := range file.PatternAmbientModules {
		if item == nil {
			patterns[i] = nil
		} else {
			patterns[i] = map[string]any{"text_hex": hx(item.Pattern.Text), "star_index": item.Pattern.StarIndex, "symbol": g.symbolRef(item.Symbol)}
		}
	}
	diagnostics := []any{}
	for _, collection := range []struct {
		name  string
		items []*ast.Diagnostic
	}{{"parse", file.Diagnostics()}, {"js", file.JSDiagnostics()}, {"jsdoc", file.JSDocDiagnostics()}, {"bind", file.BindDiagnostics()}} {
		for i, d := range collection.items {
			diagnostics = append(diagnostics, []any{collection.name, i, g.diagnostic(d)})
		}
	}
	return map[string]any{"root": root, "node_count": file.NodeCount, "text_count": file.TextCount, "identifier_count": file.IdentifierCount, "symbol_count": file.SymbolCount, "script_kind": file.ScriptKind, "language_variant": file.LanguageVariant, "declaration_file": file.IsDeclarationFile, "is_bound": file.IsBound(), "common_js": g.nodeRef(file.CommonJSModuleIndicator), "external_module": g.nodeRef(file.ExternalModuleIndicator), "global_exports": g.tableRef(file.GlobalExports), "patterns": patterns, "diagnostics": diagnostics}
}
func (g *graph) record(item queued) map[string]any {
	value := map[string]any{"id": item.id}
	switch item.kind {
	case "node":
		node := item.value.(*ast.Node)
		parent := g.nodeRef(node.Parent)
		payload, fields := g.syntax(node)
		docs := g.nodeRefs(node.EagerJSDoc(g.source))
		var flow, end, ret, fall *ast.FlowNode
		var next *ast.Node
		if data := node.FlowNodeData(); data != nil {
			flow = data.FlowNode
		}
		if data := node.BodyData(); data != nil {
			end = data.EndFlowNode
		}
		if data := node.LocalsContainerData(); data != nil {
			next = data.NextContainer
		}
		switch node.Kind {
		case ast.KindConstructor:
			ret = node.AsConstructorDeclaration().ReturnFlowNode
		case ast.KindFunctionDeclaration:
			ret = node.AsFunctionDeclaration().ReturnFlowNode
		case ast.KindFunctionExpression:
			ret = node.AsFunctionExpression().ReturnFlowNode
		case ast.KindClassStaticBlockDeclaration:
			ret = node.AsClassStaticBlockDeclaration().ReturnFlowNode
		case ast.KindCaseClause, ast.KindDefaultClause:
			fall = node.AsCaseOrDefaultClause().FallthroughFlowNode
		}
		value["kind"] = int16(node.Kind)
		value["flags"] = uint32(node.Flags)
		value["pos"] = node.Pos()
		value["end"] = node.End()
		value["parent"] = parent
		value["payload"] = payload
		value["fields"] = fields
		value["jsdoc"] = docs
		value["symbol"] = g.symbolRef(node.Symbol())
		value["local_symbol"] = g.symbolRef(node.LocalSymbol())
		value["locals"] = g.tableRef(node.Locals())
		value["next_container"] = g.nodeRef(next)
		value["flow_node"] = g.flowRef(flow)
		value["end_flow_node"] = g.flowRef(end)
		value["return_flow_node"] = g.flowRef(ret)
		value["fallthrough_flow_node"] = g.flowRef(fall)
	case "list":
		data := item.value.(listValue)
		value["pos"] = data.list.Pos()
		value["end"] = data.list.End()
		value["nodes"] = g.nodeSliceRef(data.list.Nodes)
		value["missing"] = parser.S07IsMissingList(data.list)
		value["modifier_flags"] = data.modifiers
		value["is_modifier"] = data.isModifier
	case "nodes":
		nodes := item.value.([]*ast.Node)
		value["values"] = g.nodeRefs(nodes)
	case "texts":
		values := item.value.([]string)
		texts := make([]string, len(values))
		for i, s := range values {
			texts[i] = hx(s)
		}
		value["values_hex"] = texts
	case "symbol":
		symbol := item.value.(*ast.Symbol)
		value["flags"] = uint32(symbol.Flags)
		value["check_flags"] = uint32(symbol.CheckFlags)
		value["name"] = g.name(symbol.Name, symbol)
		value["declarations"] = g.declarationRef(symbol.Declarations)
		value["value_declaration"] = g.nodeRef(symbol.ValueDeclaration)
		value["members"] = g.tableRef(symbol.Members)
		value["exports"] = g.tableRef(symbol.Exports)
		value["parent"] = g.symbolRef(symbol.Parent)
		value["export_symbol"] = g.symbolRef(symbol.ExportSymbol)
	case "table":
		table := item.value.(ast.SymbolTable)
		keys := make([]string, 0, len(table))
		for key := range table {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		entries := make([]any, len(keys))
		for i, key := range keys {
			entries[i] = []any{g.name(key, table[key]), g.symbolRef(table[key])}
		}
		value["entries"] = entries
	case "declarations":
		nodes := item.value.([]*ast.Node)
		value["values"] = g.nodeRefs(nodes)
		value["capacity"] = cap(nodes)
		value["capacity_values"] = g.nodeRefs(nodes[:cap(nodes)])
	case "flow":
		flow := item.value.(*ast.FlowNode)
		value["flags"] = uint32(flow.Flags)
		value["synthetic"] = nil
		switch data := ast.S07FlowPayload(flow.Node).(type) {
		case nil:
			value["data"] = nil
		case *ast.Node:
			value["data"] = []any{"ast", g.nodeRef(data)}
		case *ast.FlowSwitchClauseData:
			value["synthetic"] = g.synthetic(flow, item.id, "switch")
			value["data"] = []any{"switch", g.nodeRef(data.SwitchStatement), data.ClauseStart, data.ClauseEnd}
		case *ast.FlowReduceLabelData:
			value["synthetic"] = g.synthetic(flow, item.id, "reduce")
			value["data"] = []any{"reduce", g.flowRef(data.Target), g.flowListRef(data.Antecedents)}
		default:
			panic("S07 observer: unsupported flow payload")
		}
		value["antecedent"] = g.flowRef(flow.Antecedent)
		value["antecedents"] = g.flowListRef(flow.Antecedents)
	case "flow_list":
		list := item.value.(*ast.FlowList)
		value["flow"] = g.flowRef(list.Flow)
		value["next"] = g.flowListRef(list.Next)
	default:
		panic("S07 observer: unknown queued record")
	}
	return value
}

type aliasSpan struct {
	domain            string
	start, end, width uintptr
	record            map[string]any
	group             int
}

// Visible overlapping ranges determine storage aliasing. The observer never
// compares allocator addresses or inaccessible allocation prefixes/suffixes.
func aliases(queue []queued, records []map[string]any) {
	spans := []aliasSpan{}
	for i, item := range queue {
		var start, length, width uintptr
		domain := "nodes"
		switch item.kind {
		case "nodes", "declarations":
			nodes := item.value.([]*ast.Node)
			length = uintptr(len(nodes))
			if item.kind == "declarations" {
				length = uintptr(cap(nodes))
			}
			start = uintptr(unsafe.Pointer(unsafe.SliceData(nodes)))
			width = unsafe.Sizeof((*ast.Node)(nil))
		case "texts":
			texts := item.value.([]string)
			domain = "texts"
			length = uintptr(len(texts))
			start = uintptr(unsafe.Pointer(unsafe.SliceData(texts)))
			width = unsafe.Sizeof("")
		default:
			continue
		}
		records[i]["backing_group"] = 0
		records[i]["backing_start"] = 0
		if length > 0 {
			spans = append(spans, aliasSpan{domain, start, start + length*width, width, records[i], len(spans)})
		}
	}
	// Source graphs are small enough for a sorted sweep; disjoint allocations
	// cannot overlap. Union-find retains first-observation order for group IDs.
	order := make([]int, len(spans))
	parents := make([]int, len(spans))
	for i := range spans {
		order[i] = i
		parents[i] = i
	}
	var root func(int) int
	root = func(i int) int {
		if parents[i] != i {
			parents[i] = root(parents[i])
		}
		return parents[i]
	}
	sort.Slice(order, func(i, j int) bool {
		a, b := spans[order[i]], spans[order[j]]
		if a.domain != b.domain {
			return a.domain < b.domain
		}
		return a.start < b.start
	})
	for begin := 0; begin < len(order); {
		first := order[begin]
		end := spans[first].end
		next := begin + 1
		for next < len(order) && spans[order[next]].domain == spans[first].domain && spans[order[next]].start < end {
			other := order[next]
			parents[root(other)] = root(first)
			if spans[other].end > end {
				end = spans[other].end
			}
			next++
		}
		begin = next
	}
	minimum := map[int]uintptr{}
	for i, s := range spans {
		r := root(i)
		if v, ok := minimum[r]; !ok || s.start < v {
			minimum[r] = s.start
		}
	}
	groups := map[int]int{}
	for i, s := range spans {
		r := root(i)
		group, ok := groups[r]
		if !ok {
			group = len(groups) + 1
			groups[r] = group
		}
		s.record["backing_group"] = group
		s.record["backing_start"] = (s.start - minimum[r]) / s.width
	}
}

type graphRecord struct {
	kind  string
	value any
}

func collectGraph(file *ast.SourceFile) []graphRecord {
	g := newGraph(file)
	source := g.sourceRecord()
	records := []map[string]any{}
	for index := 0; index < len(g.queue); index++ {
		records = append(records, g.record(g.queue[index]))
	}
	aliases(g.queue, records)
	output := make([]graphRecord, 0, len(records)+2)
	output = append(output, graphRecord{"source", source})
	for index, item := range g.queue {
		output = append(output, graphRecord{item.kind, records[index]})
	}
	output = append(output, graphRecord{"counts", map[string]any{"node": len(g.nodes), "list": len(g.lists), "nodes": len(g.nodeSlices), "texts": len(g.textSlices), "symbol": len(g.symbols), "table": len(g.tables), "declarations": len(g.declarations), "flow": len(g.flows), "flow_list": len(g.flowLists)}})
	return output
}
