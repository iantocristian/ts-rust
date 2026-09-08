package main

import (
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/binder"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
)

func execute(s *session, r request) {
	var file *ast.SourceFile
	if !s.stage("parse", func() error {
		opts := ast.SourceFileParseOptions{FileName: r.data["filename"].(string), Path: tspath.Path(r.data["path"].(string)), ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: r.data["jsx"].(bool), Force: r.data["force"].(bool)}}
		file = parser.ParseSourceFile(opts, string(raw(r.data["source_hex"])), core.ScriptKind(r.data["script_kind"].(int64)))
		return nil
	}) {
		return
	}
	dumpGraph(s, "parsed_graph", file)
	s.stage("parsed_graph", func() error { return nil })
	if !s.stage("bind", func() error { binder.BindSourceFile(file); return nil }) {
		return
	}
	dumpGraph(s, "bound_graph", file)
	s.stage("bound_graph", func() error { return nil })
	if !s.stage("repeat_bind", func() error { binder.BindSourceFile(file); return nil }) {
		return
	}
	dumpGraph(s, "repeated_graph", file)
	s.stage("repeated_graph", func() error { return nil })
}

func raw(value any) []byte {
	result, err := byteString(value)
	if err != nil {
		panic(err)
	}
	return result
}
