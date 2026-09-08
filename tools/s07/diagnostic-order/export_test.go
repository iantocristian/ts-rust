package ast

import (
	"bytes"
	"fmt"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnostics"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"os"
	"testing"
)

func TestS07DiagnosticOrder(t *testing.T) {
	var output bytes.Buffer
	base := func() *Diagnostic {
		return &Diagnostic{loc: core.NewTextRange(1, 2), code: 100, category: diagnostics.CategoryError, messageKey: "key"}
	}
	emit := func(label string, left, right *Diagnostic) {
		order := CompareDiagnostics(left, right)
		if order < 0 {
			order = -1
		} else if order > 0 {
			order = 1
		}
		fmt.Fprintf(&output, "%s\t%d\t%t\t%t\n", label, order, EqualDiagnostics(left, right), EqualDiagnosticsNoRelatedInfo(left, right))
	}
	pair := func(label string, edit func(*Diagnostic, *Diagnostic)) {
		left, right := base(), base()
		edit(left, right)
		emit(label, left, right)
	}
	source := func(name, path string) *SourceFile {
		return &SourceFile{parseOptions: SourceFileParseOptions{FileName: name, Path: tspath.Path(path)}}
	}
	_ = source

	same := base()
	emit("same", same, same)
	pair("file_nil", func(l, r *Diagnostic) { r.file = source("/a.ts", "/a.ts") })
	pair("file_name_bytes", func(l, r *Diagnostic) { l.file = source("/\xff.ts", "/a.ts"); r.file = source("/\xc0.ts", "/z.ts") })
	pair("same_name_other_path", func(l, r *Diagnostic) { l.file = source("/a.ts", "/a.ts"); r.file = source("/a.ts", "/z.ts") })
	pair("position_extremes", func(l, r *Diagnostic) {
		l.loc = core.NewTextRange(-2147483648, 2)
		r.loc = core.NewTextRange(2147483647, 2)
	})
	pair("end_extremes", func(l, r *Diagnostic) {
		l.loc = core.NewTextRange(1, 2147483647)
		r.loc = core.NewTextRange(1, -2147483648)
	})
	pair("code_extremes", func(l, r *Diagnostic) { l.code = -2147483648; r.code = 2147483647 })
	pair("category_extremes", func(l, r *Diagnostic) { l.category = 2147483647; r.category = -2147483648 })
	pair("source_bytes", func(l, r *Diagnostic) { l.source = "\xff"; r.source = "\xc0" })
	pair("localized_identity", func(l, r *Diagnostic) {
		l.messageText = "a"
		r.messageText = "z"
		l.messageKey = "z"
		r.messageKey = "a"
	})
	pair("adhoc_identity", func(l, r *Diagnostic) {
		l.code = -1
		r.code = -1
		l.message = diagnostics.NewAdHocMessage("z")
		r.message = diagnostics.NewAdHocMessage("a")
	})
	pair("arguments", func(l, r *Diagnostic) { l.messageArgs = []string{"a", "\xff"}; r.messageArgs = []string{"a", "\xc0"} })
	pair("chain_code_ignored_by_sort", func(l, r *Diagnostic) {
		a, b := base(), base()
		a.code = 1
		b.code = 2
		l.messageChain = []*Diagnostic{a}
		r.messageChain = []*Diagnostic{b}
	})
	pair("chain_text_ignored_by_both", func(l, r *Diagnostic) {
		a, b := base(), base()
		a.messageText = "a"
		b.messageText = "z"
		l.messageChain = []*Diagnostic{a}
		r.messageChain = []*Diagnostic{b}
	})
	pair("larger_chain_first", func(l, r *Diagnostic) {
		l.messageChain = []*Diagnostic{base(), base()}
		r.messageChain = []*Diagnostic{base()}
	})
	pair("recursive_size_before_arguments", func(l, r *Diagnostic) {
		a, b := base(), base()
		a.messageArgs = []string{"z"}
		b.messageArgs = []string{"a"}
		a.messageChain = []*Diagnostic{base()}
		l.messageChain = []*Diagnostic{a}
		r.messageChain = []*Diagnostic{b}
	})
	pair("recursive_arguments", func(l, r *Diagnostic) {
		a, b := base(), base()
		a.messageArgs = []string{"a"}
		b.messageArgs = []string{"z"}
		l.messageChain = []*Diagnostic{a}
		r.messageChain = []*Diagnostic{b}
	})
	pair("larger_related_first", func(l, r *Diagnostic) { l.relatedInformation = []*Diagnostic{base()} })
	pair("related_file_order", func(l, r *Diagnostic) {
		a, b := base(), base()
		a.file = source("/z.ts", "/a.ts")
		b.file = source("/a.ts", "/z.ts")
		l.relatedInformation = []*Diagnostic{a}
		r.relatedInformation = []*Diagnostic{b}
	})
	pair("flags_ignored", func(l, r *Diagnostic) {
		l.reportsUnnecessary = true
		l.reportsDeprecated = true
		l.skippedOnNoEmit = true
	})
	pair("nil_empty_arguments", func(l, r *Diagnostic) { r.messageArgs = []string{} })
	if err := os.WriteFile(os.Getenv("S07_DIAGNOSTIC_ORDER_OUTPUT"), output.Bytes(), 0600); err != nil {
		t.Fatal(err)
	}
}
