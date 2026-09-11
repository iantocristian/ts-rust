package harnessutil

// Observation hook installed only in the read-only source overlay. The original
// compiler/harness still constructs and sorts both complete diagnostic sets.
import (
	"fmt"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
)

var S08ObserveDiagnostics func(pre, post []*ast.Diagnostic)
var S08ObserveStage func(stage string) func()
var S08ObserveOptionRejection func(message string)

func s08OptionFatalf(t *testing.T, format string, args ...any) {
	t.Helper()
	if S08ObserveOptionRejection != nil {
		S08ObserveOptionRejection(fmt.Sprintf(format, args...))
	}
	t.Fatalf(format, args...)
}

// Restore only after recording succeeds: a panic in an observer must retain its
// harness stage when the outer runner classifies the failure.
func S08EnterObservation() func() {
	if S08ObserveStage != nil {
		return S08ObserveStage("harness_observation")
	}
	return func() {}
}
