package main

import (
	"encoding/hex"
	"fmt"

	"github.com/microsoft/TypeScript/tsc/internal/api/encoder"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
)

func raw(value any) []byte {
	result, err := byteString(value)
	if err != nil {
		panic(err)
	}
	return result
}
func execute(s *session, r request) {
	switch r.op {
	case "codec":
		executeCodec(s, r)
	case "factory":
		executeFactory(s, r)
	case "kind_names":
		s.stage("kind_names", func() error {
			first := r.data["first"].(int64)
			count := r.data["count"].(int64)
			for value := first; value < first+count; value++ {
				s.observe("kind_names", "kind", map[string]any{"raw": value, "name": ast.Kind(value).String()})
			}
			return nil
		})
	case "path":
		s.stage("path", func() error {
			value := string(raw(r.data["path_hex"]))
			s.observe("path", "path", map[string]any{
				"encoded_root_length": tspath.GetEncodedRootLength(value), "normalized_hex": hex.EncodeToString([]byte(tspath.NormalizePath(value))),
				"declaration_file": tspath.IsDeclarationFileName(value),
			})
			return nil
		})
	case "decode":
		var root *ast.Node
		if !s.stage("decode", func() error {
			data := raw(r.data["wire_hex"])
			if r.data["entrypoint"] == "source_file" {
				sf, err := encoder.DecodeSourceFile(data)
				if err != nil {
					return err
				}
				root = sf.AsNode()
			} else {
				var err error
				root, err = encoder.DecodeNodes(data)
				if err != nil {
					return err
				}
			}
			if root == nil {
				s.observe("decode", "root", map[string]any{"node": nil})
			} else {
				s.observe("decode", "root", map[string]any{"node": map[string]any{"kind": int16(root.Kind), "pos": root.Pos(), "end": root.End(), "flags": uint32(root.Flags)}})
			}
			return nil
		}) {
			return
		}
		s.stage("decoded_tree", func() error { emitDecoded(s, root); return nil })
	case "parse":
		parse(s, r)
	default:
		panic("validated operation missing adapter")
	}
}
func diagnostic(s *session, stage, category string, index int, d *ast.Diagnostic) {
	args := []string{}
	for _, arg := range d.MessageArgs() {
		args = append(args, hex.EncodeToString([]byte(arg)))
	}
	s.observe(stage, "diagnostic", map[string]any{"collection": category, "index": index, "code": d.Code(), "category": int(d.Category()),
		"pos": d.Pos(), "end": d.End(), "key_hex": hex.EncodeToString([]byte(d.MessageKey())), "text_hex": hex.EncodeToString([]byte(d.MessageText())), "args_hex": args})
}
func emitTable(s *session, stage string, table *encoder.NodeIndexTable) {
	indices := make(map[*ast.Node]int, len(table.Nodes))
	for index, node := range table.Nodes {
		if node != nil {
			indices[node] = index
		}
	}
	s.observe(stage, "table", map[string]any{"length": len(table.Nodes)})
	for index, node := range table.Nodes {
		if node == nil {
			s.observe(stage, "node", map[string]any{"index": index, "node": nil})
			continue
		}
		// GetIndex is independently exercised for every non-nil member.
		s.observe(stage, "node", map[string]any{"index": index, "node": map[string]any{
			"kind": int16(node.Kind), "pos": node.Pos(), "end": node.End(), "flags": uint32(node.Flags),
			"parent": indices[node.Parent], "lookup": table.GetIndex(node),
		}})
	}
	absent := ast.NewNodeFactory(ast.NodeFactoryHooks{}).NewToken(ast.KindUnknown)
	s.observe(stage, "absent_lookup", map[string]any{"index": table.GetIndex(absent)})
}
func parse(s *session, r request) {
	var sf *ast.SourceFile
	if !s.stage("parse", func() error {
		opts := ast.SourceFileParseOptions{FileName: r.data["filename"].(string), Path: tspath.Path(r.data["path"].(string)),
			ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: r.data["jsx"].(bool), Force: r.data["force"].(bool)}}
		sf = parser.ParseSourceFile(opts, string(raw(r.data["source_hex"])), core.ScriptKind(r.data["script_kind"].(int64)))
		s.observe("parse", "source_file", map[string]any{"kind": int16(sf.Kind), "pos": sf.Pos(), "end": sf.End(), "flags": uint32(sf.Flags),
			"node_count": sf.NodeCount, "text_count": sf.TextCount, "identifier_count": sf.IdentifierCount, "script_kind": int(sf.ScriptKind),
			"language_variant": int(sf.LanguageVariant), "declaration_file": sf.IsDeclarationFile, "hash": encoder.SourceFileHash(sf)})
		for _, collection := range []struct {
			name   string
			values []*ast.Diagnostic
		}{{"parse", sf.Diagnostics()}, {"js", sf.JSDiagnostics()}, {"jsdoc", sf.JSDocDiagnostics()}} {
			for index, d := range collection.values {
				diagnostic(s, "parse", collection.name, index, d)
			}
		}
		return nil
	}) {
		return
	}
	var before *encoder.NodeIndexTable
	if !s.stage("node_index_before", func() error {
		before = encoder.GetNodeIndexTable(sf)
		independent := encoder.BuildNodeIndexTable(sf)
		same := len(before.Nodes) == len(independent.Nodes)
		if same {
			for i, node := range before.Nodes {
				if node != independent.Nodes[i] {
					same = false
					break
				}
			}
		}
		s.observe("node_index_before", "identity", map[string]any{"independent_order_equal": same, "cache_reused": before == encoder.GetNodeIndexTable(sf)})
		emitTable(s, "node_index_before", before)
		return nil
	}) {
		return
	}
	var encodedTable *encoder.NodeIndexTable
	if !s.stage("encode_source_file", func() error {
		encoded, table, err := encoder.EncodeSourceFile(sf)
		if err != nil {
			return err
		}
		encodedTable = table
		s.observe("encode_source_file", "bytes", map[string]any{"length": len(encoded)})
		for offset := 0; offset < len(encoded); offset += 65536 {
			end := min(offset+65536, len(encoded))
			s.observe("encode_source_file", "chunk", map[string]any{"offset": offset, "hex": hex.EncodeToString(encoded[offset:end])})
		}
		return nil
	}) {
		return
	}
	s.stage("node_index_after", func() error {
		after := encoder.GetNodeIndexTable(sf)
		s.observe("node_index_after", "identity", map[string]any{"before_reused": after == before, "encoded_reused": after == encodedTable})
		emitTable(s, "node_index_after", after)
		return nil
	})
}
func emitDecoded(s *session, root *ast.Node) {
	if root == nil {
		return
	}
	// Observe the actual reconstructed nodes through their public child walk. This
	// includes payload-dependent failures after a decoder successfully returns.
	stack := []*ast.Node{root}
	count := 0
	for len(stack) > 0 {
		node := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		children := []*ast.Node{}
		node.ForEachChild(func(child *ast.Node) bool { children = append(children, child); return false })
		s.observe("decoded_tree", "tree_node", map[string]any{"index": count, "kind": int16(node.Kind), "pos": node.Pos(), "end": node.End(), "flags": uint32(node.Flags), "children": len(children)})
		count++
		if count > 1000000 {
			s.err = fmt.Errorf("adapter decoded walk exceeds %d nodes", count)
			return
		}
		for i := len(children) - 1; i >= 0; i-- {
			stack = append(stack, children[i])
		}
	}
}
