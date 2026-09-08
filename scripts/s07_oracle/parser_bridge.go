// Access-only observer of the parser's already-created missing-list sentinel.
package parser

import "github.com/microsoft/TypeScript/tsc/internal/ast"

func S07IsMissingList(list *ast.NodeList) bool { return isMissingNodeList(list) }
