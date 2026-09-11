package checker_test

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"runtime"
	"slices"
	"sync"
	"testing"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/checker"
	"github.com/microsoft/TypeScript/tsc/internal/compiler"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnosticwriter"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/baseline"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/harnessutil"
	"github.com/microsoft/TypeScript/tsc/internal/testutil/tsbaseline"
	"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest"
)

type p2Query struct {
	ID          string `json:"id"`
	File        string `json:"file"`
	Declaration string `json:"declaration"`
	Target      string `json:"target"`
	Operation   string `json:"operation"`
}
type p2Program struct {
	ID      string            `json:"id"`
	Files   map[string]string `json:"files"`
	Roots   []string          `json:"roots"`
	Queries []p2Query         `json:"queries"`
}
type p2Merge struct {
	ID               string              `json:"id"`
	Files            map[string]string   `json:"files"`
	Shared           string              `json:"shared"`
	Symbol           string              `json:"symbol"`
	Modes            []string            `json:"modes"`
	SingleRoots      []string            `json:"single_roots"`
	IndependentRoots map[string][]string `json:"independent_roots"`
}
type p2Spec struct {
	Version int    `json:"version"`
	Scope   string `json:"scope"`
	Options struct {
		Target string `json:"target"`
		Module string `json:"module"`
		Strict bool   `json:"strict"`
		NoLib  bool   `json:"noLib"`
	} `json:"options"`
	Programs []p2Program `json:"programs"`
	Merge    p2Merge     `json:"merge"`
}

// The host caches original parsed SourceFiles only. Each real Program and
// Checker retains its own source list, globals, merges and semantic caches.
type p2SharedHost struct {
	compiler.CompilerHost
	mu    sync.Mutex
	files map[ast.SourceFileParseOptions]*ast.SourceFile
}

func (h *p2SharedHost) GetSourceFile(opts ast.SourceFileParseOptions) *ast.SourceFile {
	h.mu.Lock()
	defer h.mu.Unlock()
	if f := h.files[opts]; f != nil {
		return f
	}
	f := h.CompilerHost.GetSourceFile(opts)
	h.files[opts] = f
	return f
}
func p2Host(files map[string]string) *p2SharedHost {
	return &p2SharedHost{CompilerHost: compiler.NewCompilerHost("/", vfstest.FromMap(files, true), "/no-default-lib", nil, nil, nil), files: map[ast.SourceFileParseOptions]*ast.SourceFile{}}
}
func p2NewProgram(host compiler.CompilerHost, roots []string) *compiler.Program {
	options := &core.CompilerOptions{Target: core.ScriptTargetESNext, Module: core.ModuleKindESNext, Strict: core.TSTrue, NoLib: core.TSTrue}
	config := tsoptions.NewParsedCommandLine(options, roots, nil, tspath.ComparePathsOptions{})
	p := compiler.NewProgram(compiler.ProgramOptions{Config: config, Host: host})
	p.BindSourceFiles()
	return p
}
func p2Hex(s string) string { return hex.EncodeToString([]byte(s)) }
func p2Node(n *ast.Node) any {
	if n == nil {
		return nil
	}
	file := ast.GetSourceFileOfNode(n)
	return map[string]any{"file": file.FileName(), "kind": n.Kind, "pos": n.Pos(), "end": n.End()}
}
func p2Diagnostics(values []*ast.Diagnostic) []map[string]any {
	result := make([]map[string]any, 0, len(values))
	for _, d := range values {
		var file any
		if d.File() != nil {
			file = d.File().FileName()
		}
		result = append(result, map[string]any{"file": file, "pos": d.Pos(), "end": d.End(), "code": d.Code(), "category": d.Category(),
			"key_hex": p2Hex(string(d.MessageKey())), "text_hex": p2Hex(d.MessageText()), "source_hex": p2Hex(d.Source()), "args": d.MessageArgs(),
			"chain": p2Diagnostics(d.MessageChain()), "related": p2Diagnostics(d.RelatedInformation()), "unnecessary": d.ReportsUnnecessary(),
			"deprecated": d.ReportsDeprecated(), "skipped_on_no_emit": d.SkippedOnNoEmit()})
	}
	return result
}
func p2SortedNames(table ast.SymbolTable) []string {
	keys := make([]string, 0, len(table))
	for k := range table {
		keys = append(keys, k)
	}
	slices.Sort(keys)
	return keys
}

type p2Symbols struct {
	ids   map[*ast.Symbol]string
	owner string
	next  int
}

func p2SymbolIDs(files []*ast.SourceFile, owner string) *p2Symbols {
	g := &p2Symbols{ids: map[*ast.Symbol]string{}, owner: owner}
	files = slices.Clone(files)
	slices.SortFunc(files, func(a, b *ast.SourceFile) int { return bytes.Compare([]byte(a.FileName()), []byte(b.FileName())) })
	for _, f := range files {
		next := 0
		var seed func(*ast.Symbol)
		seed = func(s *ast.Symbol) {
			if s == nil || g.ids[s] != "" {
				return
			}
			g.ids[s] = fmt.Sprintf("source:%s:%d", f.FileName(), next)
			next++
			for _, table := range []ast.SymbolTable{s.Members, s.Exports} {
				for _, name := range p2SortedNames(table) {
					seed(table[name])
				}
			}
			seed(s.Parent)
			seed(s.ExportSymbol)
		}
		var visit func(*ast.Node) bool
		visit = func(n *ast.Node) bool {
			seed(n.Symbol())
			seed(n.LocalSymbol())
			for _, name := range p2SortedNames(n.Locals()) {
				seed(n.Locals()[name])
			}
			n.ForEachChild(visit)
			return false
		}
		visit(f.AsNode())
	}
	return g
}
func (g *p2Symbols) id(s *ast.Symbol) any {
	if s == nil {
		return nil
	}
	if g.ids[s] == "" {
		g.ids[s] = fmt.Sprintf("checker:%s:%d", g.owner, g.next)
		g.next++
	}
	return g.ids[s]
}
func (g *p2Symbols) snapshot(root *ast.Symbol) any {
	if root == nil {
		return nil
	}
	queue := []*ast.Symbol{root}
	seen := map[*ast.Symbol]bool{}
	records := []map[string]any{}
	rootID := g.id(root)
	for len(queue) > 0 {
		s := queue[0]
		queue = queue[1:]
		if seen[s] {
			continue
		}
		seen[s] = true
		ref := func(value *ast.Symbol) any {
			if value != nil {
				queue = append(queue, value)
			}
			return g.id(value)
		}
		table := func(t ast.SymbolTable) any {
			if t == nil {
				return nil
			}
			entries := []map[string]any{}
			for _, name := range p2SortedNames(t) {
				entries = append(entries, map[string]any{"name_hex": p2Hex(name), "symbol": ref(t[name])})
			}
			return entries
		}
		var declarations any
		if s.Declarations != nil {
			ds := []any{}
			for _, d := range s.Declarations {
				ds = append(ds, p2Node(d))
			}
			declarations = ds
		}
		records = append(records, map[string]any{"id": g.id(s), "name_hex": p2Hex(s.Name), "flags": uint32(s.Flags), "check_flags": uint32(s.CheckFlags), "declarations": declarations,
			"value_declaration": p2Node(s.ValueDeclaration), "members": table(s.Members), "exports": table(s.Exports), "parent": ref(s.Parent), "export_symbol": ref(s.ExportSymbol)})
	}
	return map[string]any{"root": rootID, "records": records}
}
func p2BoundSnapshot(file *ast.SourceFile) map[string]any {
	g := p2SymbolIDs([]*ast.SourceFile{file}, "bound")
	nodes := []map[string]any{}
	var visit func(*ast.Node) bool
	visit = func(n *ast.Node) bool {
		children := []any{}
		n.ForEachChild(func(child *ast.Node) bool { children = append(children, p2Node(child)); return false })
		nodes = append(nodes, map[string]any{"node": p2Node(n), "flags": uint32(n.Flags), "symbol": g.snapshot(n.Symbol()), "local_symbol": g.snapshot(n.LocalSymbol()), "children": children})
		n.ForEachChild(visit)
		return false
	}
	visit(file.AsNode())
	locals := []map[string]any{}
	for _, name := range p2SortedNames(file.AsNode().Locals()) {
		locals = append(locals, map[string]any{"name_hex": p2Hex(name), "symbol": g.snapshot(file.AsNode().Locals()[name])})
	}
	return map[string]any{"file": file.FileName(), "text_hex": p2Hex(file.Text()), "nodes": nodes, "locals": locals}
}
func p2Declaration(file *ast.SourceFile, name string) *ast.Node {
	var found *ast.Node
	var visit func(*ast.Node) bool
	visit = func(n *ast.Node) bool {
		if (n.Kind == ast.KindTypeAliasDeclaration || n.Kind == ast.KindInterfaceDeclaration || n.Kind == ast.KindVariableDeclaration) && n.Name() != nil && n.Name().Text() == name {
			if found != nil {
				panic("ambiguous declaration: " + name)
			}
			found = n
		}
		n.ForEachChild(visit)
		return false
	}
	visit(file.AsNode())
	if found == nil {
		panic("missing declaration: " + name)
	}
	return found
}
func p2BasicType(c *checker.Checker, typ *checker.Type) map[string]any {
	// P2 exercises context-free display on both sides. Enclosing-declaration
	// name qualification and annotation reuse belong to the later display slice.
	return map[string]any{"flags": uint32(typ.Flags()), "object_flags": uint32(typ.ObjectFlags()), "display_hex": p2Hex(c.TypeToString(typ)), "in_alias_display_hex": p2Hex(c.TypeToStringEx(typ, nil, checker.TypeFormatFlagsInTypeAlias, nil))}
}
func p2Type(c *checker.Checker, typ *checker.Type, g *p2Symbols) map[string]any {
	result := p2BasicType(c, typ)
	properties := []map[string]any{}
	for _, symbol := range c.GetPropertiesOfType(typ) {
		properties = append(properties, map[string]any{"symbol": g.snapshot(symbol), "type": p2BasicType(c, c.GetTypeOfSymbol(symbol))})
	}
	result["properties"] = properties
	return result
}
func p2QueryResult(c *checker.Checker, p *compiler.Program, q p2Query) map[string]any {
	file := p.GetSourceFile(q.File)
	if file == nil {
		panic("query file missing")
	}
	decl := p2Declaration(file, q.Declaration)
	var node *ast.Node
	switch q.Target {
	case "name":
		node = decl.Name()
	case "annotation":
		node = decl.Type()
	case "annotation_name":
		node = decl.Type().AsTypeReferenceNode().TypeName
	case "initializer":
		node = decl.Initializer()
	default:
		panic("unknown query target")
	}
	if node == nil {
		panic("query target absent")
	}
	symbol := c.GetSymbolAtLocation(node)
	var typ *checker.Type
	switch q.Operation {
	case "type_at_location":
		typ = c.GetTypeAtLocation(node)
	case "declared_type":
		if symbol == nil {
			panic("declared type lacks symbol")
		}
		typ = c.GetDeclaredTypeOfSymbol(symbol)
	default:
		panic("unknown query operation")
	}
	if typ == nil {
		panic("native query returned no type")
	}
	g := p2SymbolIDs(p.SourceFiles(), "query")
	return map[string]any{"id": q.ID, "state": "executed", "node": p2Node(node), "symbol": g.snapshot(symbol), "type": p2Type(c, typ, g)}
}
func p2AllDiagnostics(t *testing.T, p *compiler.Program, c *checker.Checker) map[string]any {
	ctx := context.Background()
	phases := map[string][]*ast.Diagnostic{"config": p.GetConfigFileParsingDiagnostics(), "program": p.GetProgramDiagnostics(), "syntactic": p.GetSyntacticDiagnostics(ctx, nil), "bind": p.GetBindDiagnostics(ctx, nil), "semantic": {}, "global": {}}
	for _, file := range p.SourceFiles() {
		phases["semantic"] = append(phases["semantic"], c.GetDiagnostics(ctx, file)...)
	}
	phases["global"] = c.GetGlobalDiagnostics()
	observed := map[string]any{"state": "executed"}
	all := []*ast.Diagnostic{}
	for _, phase := range []string{"config", "program", "syntactic", "bind", "semantic", "global"} {
		ds := compiler.SortAndDeduplicateDiagnostics(phases[phase])
		observed[phase] = p2Diagnostics(ds)
		all = append(all, ds...)
	}
	all = compiler.SortAndDeduplicateDiagnostics(all)
	observed["combined"] = p2Diagnostics(all)
	files := []*harnessutil.TestFile{}
	for _, f := range p.SourceFiles() {
		files = append(files, &harnessutil.TestFile{UnitName: f.FileName(), Content: f.Text()})
	}
	value := baseline.NoContent
	if len(all) > 0 {
		value = tsbaseline.GetErrorBaseline(t, files, diagnosticwriter.WrapASTDiagnostics(all), diagnosticwriter.CompareASTDiagnostics, false)
	}
	if value == baseline.NoContent {
		observed["baseline"] = map[string]any{"state": "no_content"}
	} else {
		observed["baseline"] = map[string]any{"state": "content", "text_hex": p2Hex(value)}
	}
	return observed
}
func p2ObserveProgram(t *testing.T, request p2Program) map[string]any {
	p := p2NewProgram(p2Host(request.Files), request.Roots)
	c, _ := checker.NewChecker(p, nil)
	queries := []map[string]any{}
	for _, q := range request.Queries {
		queries = append(queries, p2QueryResult(c, p, q))
	}
	return map[string]any{"id": request.ID, "state": "executed", "queries": queries, "diagnostics": p2AllDiagnostics(t, p, c)}
}

type p2Merged struct {
	row    map[string]any
	symbol *ast.Symbol
	typ    *checker.Type
}

func p2ObserveMerged(c *checker.Checker, p *compiler.Program, request p2Merge, label, path string) p2Merged {
	decl := p2Declaration(p.GetSourceFile(path), request.Symbol)
	symbol := c.GetSymbolAtLocation(decl.Name())
	if symbol == nil {
		panic("merged symbol absent")
	}
	typ := c.GetDeclaredTypeOfSymbol(symbol)
	g := p2SymbolIDs(p.SourceFiles(), label)
	base := p.GetSourceFile(request.Shared).AsNode().Locals()[request.Symbol]
	return p2Merged{row: map[string]any{"checker": label, "via": path, "symbol": g.snapshot(symbol), "type": p2Type(c, typ, g), "merged_is_bound_base": symbol == base}, symbol: symbol, typ: typ}
}
func p2ObserveMerge(t *testing.T, request p2Merge, mode string) map[string]any {
	host := p2Host(request.Files)
	row := map[string]any{"mode": mode, "state": "executed"}
	a := p2NewProgram(host, request.IndependentRoots["A"])
	base := a.GetSourceFile(request.Shared)
	row["base_before"] = p2BoundSnapshot(base)
	observations := []map[string]any{}
	if mode == "single-checker" || mode == "repeated" {
		p := p2NewProgram(host, request.SingleRoots)
		c, _ := checker.NewChecker(p, nil)
		row["shared_bound_file_identity"] = p.GetSourceFile(request.Shared) == base
		paths := []string{request.Shared}
		if mode == "repeated" {
			paths = []string{request.Shared, "/a.ts", "/b.ts", "/b.ts", "/a.ts", request.Shared}
		}
		var previous p2Merged
		for _, path := range paths {
			value := p2ObserveMerged(c, p, request, "single", path)
			value.row["same_symbol_as_previous"] = previous.symbol != nil && previous.symbol == value.symbol
			value.row["same_type_as_previous"] = previous.typ != nil && previous.typ == value.typ
			observations = append(observations, value.row)
			previous = value
		}
		row["diagnostics"] = map[string]any{"single": p2AllDiagnostics(t, p, c)}
	} else {
		b := p2NewProgram(host, request.IndependentRoots["B"])
		row["shared_bound_file_identity"] = b.GetSourceFile(request.Shared) == base
		var ca, cb *checker.Checker
		var va, vb p2Merged
		if mode == "concurrent" {
			start := make(chan struct{})
			ready := sync.WaitGroup{}
			done := sync.WaitGroup{}
			ready.Add(2)
			done.Add(2)
			go func() {
				defer done.Done()
				ready.Done()
				<-start
				ca, _ = checker.NewChecker(a, nil)
				va = p2ObserveMerged(ca, a, request, "A", request.Shared)
			}()
			go func() {
				defer done.Done()
				ready.Done()
				<-start
				cb, _ = checker.NewChecker(b, nil)
				vb = p2ObserveMerged(cb, b, request, "B", request.Shared)
			}()
			ready.Wait()
			close(start)
			done.Wait()
		} else if mode == "separate-checker" {
			ca, _ = checker.NewChecker(a, nil)
			cb, _ = checker.NewChecker(b, nil)
			va = p2ObserveMerged(ca, a, request, "A", request.Shared)
			vb = p2ObserveMerged(cb, b, request, "B", request.Shared)
		} else {
			panic("unknown merge mode")
		}
		observations = append(observations, va.row, vb.row)
		row["different_merged_symbols"] = va.symbol != vb.symbol
		row["different_types"] = va.typ != vb.typ
		for _, item := range []struct {
			c        *checker.Checker
			p        *compiler.Program
			label    string
			previous p2Merged
		}{{cb, b, "B", vb}, {ca, a, "A", va}} {
			value := p2ObserveMerged(item.c, item.p, request, item.label, request.Shared)
			value.row["same_symbol_as_previous"] = value.symbol == item.previous.symbol
			value.row["same_type_as_previous"] = value.typ == item.previous.typ
			observations = append(observations, value.row)
		}
		row["diagnostics"] = map[string]any{"A": p2AllDiagnostics(t, a, ca), "B": p2AllDiagnostics(t, b, cb)}
		// Retain only B's checker/program. The observed A payload has no native
		// pointers; clearing them permits collection before the next B query.
		ca = nil
		a = nil
		va = p2Merged{}
		runtime.GC()
		last := p2ObserveMerged(cb, b, request, "B", request.Shared)
		last.row["after_release_A"] = true
		last.row["same_symbol_as_previous"] = last.symbol == vb.symbol
		last.row["same_type_as_previous"] = last.typ == vb.typ
		observations = append(observations, last.row)
	}
	row["observations"] = observations
	row["base_after"] = p2BoundSnapshot(base)
	before, _ := json.Marshal(row["base_before"])
	after, _ := json.Marshal(row["base_after"])
	row["base_unchanged"] = bytes.Equal(before, after)
	return row
}
func TestS08P2(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_P2_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	var spec p2Spec
	if err := decoder.Decode(&spec); err != nil {
		t.Fatal(err)
	}
	if decoder.Decode(new(any)) != io.EOF {
		t.Fatal("trailing request")
	}
	if spec.Version != 1 || spec.Options.Target != "ESNext" || spec.Options.Module != "ESNext" || !spec.Options.Strict || !spec.Options.NoLib {
		t.Fatal("unsupported explicit P2 options")
	}
	programs := []map[string]any{}
	for _, request := range spec.Programs {
		programs = append(programs, p2ObserveProgram(t, request))
	}
	merges := []map[string]any{}
	for _, mode := range spec.Merge.Modes {
		merges = append(merges, p2ObserveMerge(t, spec.Merge, mode))
	}
	hash := sha256.Sum256(raw)
	out, err := json.Marshal(map[string]any{"version": 1, "request_sha256": hex.EncodeToString(hash[:]), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH, "programs": programs, "merges": merges})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_P2_OUTPUT"), out, 0600); err != nil {
		t.Fatal(err)
	}
}
