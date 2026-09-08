// Access-only assignment for source metadata owned by the pinned AST package.
package ast

func S06SetImports(source *SourceFile, nodes []*Node) { source.imports = nodes }
