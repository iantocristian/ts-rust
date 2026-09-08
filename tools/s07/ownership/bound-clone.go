package main

import (
	"encoding/json"
	"os"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/binder"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
)

// This is clone/update observation after actual binding, without rebinding a
// transformed source whose children remain parented to the original.
func main() {
	original := parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: "/bound.js", Path: "/bound.js"}, "exports.x = this; async function f() { return this; }", core.ScriptKindJS)
	parsedFlags := original.Flags
	parsedFunctionFlags := original.Statements.Nodes[1].Flags
	binder.BindSourceFile(original)
	factory := ast.NewNodeFactory(ast.NodeFactoryHooks{})
	cloned := original.Clone(factory).AsSourceFile()
	updated := factory.UpdateSourceFile(original, original.Statements.Clone(factory), original.EndOfFileToken).AsSourceFile()
	function := original.Statements.Nodes[1].Clone(factory)
	json.NewEncoder(os.Stdout).Encode(map[string]any{
		"parsedFlags":         parsedFlags,
		"boundFlags":          original.Flags,
		"cloneFlags":          cloned.Flags,
		"updateFlags":         updated.Flags,
		"parsedFunctionFlags": parsedFunctionFlags,
		"boundFunctionFlags":  original.Statements.Nodes[1].Flags,
		"cloneFunctionFlags":  function.Flags,
		"commonJS":            original.CommonJSModuleIndicator != nil,
		"cloneCommonJS":       cloned.CommonJSModuleIndicator == original.CommonJSModuleIndicator,
		"updateCommonJS":      updated.CommonJSModuleIndicator == original.CommonJSModuleIndicator,
		"cloneBound":          cloned.IsBound(),
		"updateBound":         updated.IsBound(),
		"sameCloneStatements": cloned.Statements == original.Statements,
		"sameUpdateChild":     updated.Statements.Nodes[0] == original.Statements.Nodes[0],
		"sameCloneEOF":        cloned.EndOfFileToken == original.EndOfFileToken,
		"sameUpdateEOF":       updated.EndOfFileToken == original.EndOfFileToken,
		"sameUnchangedUpdate": factory.UpdateSourceFile(original, original.Statements, original.EndOfFileToken) == original.AsNode(),
	})
}
