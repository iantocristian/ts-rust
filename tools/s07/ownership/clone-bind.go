package main

import (
	"encoding/json"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/binder"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"os"
)

func main() {
	original := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: "/clone.ts", Path: "/clone.ts"}, "let x=1; x;", core.ScriptKindTS)
	cloned := original.Clone(ast.NewNodeFactory(ast.NodeFactoryHooks{})).AsSourceFile()
	shared := cloned.Statements.Nodes[1].AsExpressionStatement().Expression
	before := shared.FlowNodeData().FlowNode == nil
	binder.BindSourceFile(cloned)
	json.NewEncoder(os.Stdout).Encode(map[string]any{"originalBound": original.IsBound(), "cloneBound": cloned.IsBound(), "sameStatements": original.Statements == cloned.Statements, "sameEOF": original.EndOfFileToken == cloned.EndOfFileToken, "sharedStatementParentIsOriginal": cloned.Statements.Nodes[1].Parent == original.AsNode(), "sharedFlowWasNil": before, "sharedFlowIsNowPresent": shared.FlowNodeData().FlowNode != nil, "originalLocalsNil": original.LocalsContainerData().Locals == nil, "cloneLocalsHasX": cloned.LocalsContainerData().Locals["x"] != nil})
}
