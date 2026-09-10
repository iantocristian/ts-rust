package harnessutil

// Observation hook installed only in the read-only source overlay. The original
// compiler/harness still constructs and sorts both complete diagnostic sets.
import "github.com/microsoft/TypeScript/tsc/internal/ast"

var S08ObserveDiagnostics func(pre, post []*ast.Diagnostic)
