// Typed, conservative source closure. This is an obligation audit, not execution coverage.
package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"go/token"
	"go/types"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"slices"
	"sort"
	"strings"

	"golang.org/x/tools/go/callgraph/cha"
	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
)

type Request struct {
	GOOS       string   `json:"goos"`
	GOARCH     string   `json:"goarch"`
	Prefix     string   `json:"prefix"`
	Patterns   []string `json:"patterns"`
	Roots      []string `json:"roots"`
	Interfaces []string `json:"interfaces"`
}
type Function struct {
	ID        string `json:"id"`
	Selector  string `json:"selector"`
	File      string `json:"file"`
	Line      int    `json:"line"`
	End       int    `json:"end_line"`
	Synthetic string `json:"synthetic,omitempty"`
}
type Site struct {
	Caller    string   `json:"caller"`
	File      string   `json:"file"`
	Line      int      `json:"line"`
	Column    int      `json:"column"`
	Kind      string   `json:"kind"`
	Signature string   `json:"signature"`
	Value     string   `json:"value"`
	Targets   []string `json:"targets"`
}
type Member struct {
	Name      string `json:"name"`
	Signature string `json:"signature"`
	File      string `json:"file"`
	Line      int    `json:"line"`
}

func must(e error) {
	if e != nil {
		fmt.Fprintln(os.Stderr, e)
		os.Exit(1)
	}
}

// Go type identity ignores alias spelling and parameter names. SSA's shared
// instantiation cache may retain either spelling depending on map iteration.
// Preserve named types while canonicalizing only those non-semantic choices.
func typeName(t types.Type) string {
	t = types.Unalias(t)
	switch t := t.(type) {
	case *types.Basic:
		return types.Typ[t.Kind()].Name()
	case *types.Named:
		name := t.Obj().Name()
		if t.Obj().Pkg() != nil {
			name = t.Obj().Pkg().Path() + "." + name
		}
		if args := t.TypeArgs(); args != nil && args.Len() != 0 {
			parts := []string{}
			for i := 0; i < args.Len(); i++ {
				parts = append(parts, typeName(args.At(i)))
			}
			name += "[" + strings.Join(parts, ", ") + "]"
		}
		return name
	case *types.Pointer:
		return "*" + typeName(t.Elem())
	case *types.Slice:
		return "[]" + typeName(t.Elem())
	case *types.Array:
		return fmt.Sprintf("[%d]%s", t.Len(), typeName(t.Elem()))
	case *types.Map:
		return "map[" + typeName(t.Key()) + "]" + typeName(t.Elem())
	case *types.Chan:
		prefix := "chan "
		if t.Dir() == types.SendOnly {
			prefix = "chan<- "
		}
		if t.Dir() == types.RecvOnly {
			prefix = "<-chan "
		}
		return prefix + typeName(t.Elem())
	case *types.Signature:
		tuple := func(v *types.Tuple, variadic bool) string {
			parts := []string{}
			for i := 0; i < v.Len(); i++ {
				part := typeName(v.At(i).Type())
				if variadic && i == v.Len()-1 {
					part = "..." + strings.TrimPrefix(part, "[]")
				}
				parts = append(parts, part)
			}
			return strings.Join(parts, ", ")
		}
		result := "func(" + tuple(t.Params(), t.Variadic()) + ")"
		if t.Results().Len() == 1 {
			result += " " + tuple(t.Results(), false)
		} else if t.Results().Len() > 1 {
			result += " (" + tuple(t.Results(), false) + ")"
		}
		return result
	default:
		return types.TypeString(t, func(p *types.Package) string { return p.Path() })
	}
}

func main() {
	dir := flag.String("dir", "", "module directory")
	input := flag.String("request", "", "request JSON")
	flag.Parse()
	raw, e := os.ReadFile(*input)
	must(e)
	var req Request
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	must(decoder.Decode(&req))
	if decoder.Decode(new(any)) != io.EOF {
		must(fmt.Errorf("trailing request"))
	}
	if req.Prefix == "" || req.GOOS == "" || req.GOARCH == "" || len(req.Roots) == 0 {
		must(fmt.Errorf("incomplete request"))
	}
	env := []string{}
	for _, pair := range os.Environ() {
		if !strings.HasPrefix(pair, "GOOS=") && !strings.HasPrefix(pair, "GOARCH=") {
			env = append(env, pair)
		}
	}
	env = append(env, "GOOS="+req.GOOS, "GOARCH="+req.GOARCH)
	cfg := &packages.Config{Dir: *dir, Mode: packages.LoadAllSyntax, Tests: false, Env: env}
	roots, e := packages.Load(cfg, req.Patterns...)
	must(e)
	if packages.PrintErrors(roots) != 0 {
		must(fmt.Errorf("typed package loading failed"))
	}
	// Build source bodies for project packages only. External libraries remain named
	// boundaries; function-valued arguments crossing them are expanded conservatively.
	project := []*packages.Package{}
	pkgs := map[string]*packages.Package{}
	packages.Visit(roots, func(p *packages.Package) bool {
		if strings.HasPrefix(p.PkgPath, req.Prefix) {
			project = append(project, p)
			pkgs[p.PkgPath] = p
		}
		return true
	}, nil)
	sort.Slice(project, func(i, j int) bool { return project[i].PkgPath < project[j].PkgPath })
	aliases := map[string]string{}
	packages.Visit(roots, func(p *packages.Package) bool {
		for _, name := range p.Types.Scope().Names() {
			if obj, ok := p.Types.Scope().Lookup(name).(*types.TypeName); ok && obj.IsAlias() {
				aliases[p.PkgPath+"."+name] = typeName(obj.Type())
			}
		}
		return true
	}, nil)
	names := []string{}
	for name := range aliases {
		names = append(names, regexp.QuoteMeta(name))
	}
	sort.Slice(names, func(i, j int) bool {
		return len(names[i]) > len(names[j]) || len(names[i]) == len(names[j]) && names[i] < names[j]
	})
	aliasPattern := regexp.MustCompile("(?:" + strings.Join(names, "|") + ")(?:[^A-Za-z0-9_]|$)")
	spelling := func(value string) string {
		if len(aliases) == 0 {
			return value
		}
		return aliasPattern.ReplaceAllStringFunc(value, func(match string) string {
			for _, name := range []string{match, strings.TrimSuffix(match, match[len(match)-1:])} {
				if replacement, ok := aliases[name]; ok {
					return replacement + strings.TrimPrefix(match, name)
				}
			}
			return match
		})
	}
	prog, _ := ssautil.Packages(project, ssa.InstantiateGenerics)
	prog.Build()
	graph := cha.CallGraph(prog)
	funcs := ssautil.AllFunctions(prog)
	rel := func(pos token.Pos) (string, int, int) {
		p := prog.Fset.Position(pos)
		if p.Filename == "" {
			return "", 0, 0
		}
		r, e := filepath.Rel(*dir, p.Filename)
		must(e)
		if r == ".." || strings.HasPrefix(r, ".."+string(filepath.Separator)) {
			r = "<external>/" + filepath.Base(p.Filename)
		}
		return filepath.ToSlash(r), p.Line, p.Column
	}
	isProject := func(f *ssa.Function) bool {
		if p := f.Package(); p != nil {
			return strings.HasPrefix(p.Pkg.Path(), req.Prefix)
		}
		origin := f.Origin()
		if origin == nil {
			origin = f
		}
		if obj := origin.Object(); obj != nil && obj.Pkg() != nil {
			return strings.HasPrefix(obj.Pkg().Path(), req.Prefix)
		}
		return strings.Contains(f.String(), req.Prefix)
	}
	selector := func(f *ssa.Function) string {
		origin := f.Origin()
		if origin == nil {
			origin = f
		}
		obj := origin.Object()
		if obj == nil {
			return ""
		}
		s := obj.Name()
		sig := obj.Type().(*types.Signature)
		if recv := sig.Recv(); recv != nil {
			t := recv.Type()
			if p, ok := t.(*types.Pointer); ok {
				t = p.Elem()
			}
			if n, ok := t.(*types.Named); ok {
				s = n.Obj().Name() + "." + s
			}
		}
		return strings.TrimPrefix(obj.Pkg().Path(), req.Prefix) + "::" + s
	}
	keys := map[*ssa.Function]string{}
	key := func(f *ssa.Function) string {
		if key, ok := keys[f]; ok {
			return key
		}
		file, line, col := rel(f.Pos())
		key := fmt.Sprintf("%s@%s:%d:%d#%s", spelling(f.String()), file, line, col, f.Synthetic)
		keys[f] = key
		return key
	}
	bySelector := map[string][]*ssa.Function{}
	byID := map[string]Function{}
	for f := range funcs {
		id := key(f)
		file, line, _ := rel(f.Pos())
		end := line
		if f.Syntax() != nil {
			_, end, _ = rel(f.Syntax().End())
		}
		row := Function{id, selector(f), file, line, end, f.Synthetic}
		if prior, ok := byID[id]; ok && prior != row {
			must(fmt.Errorf("ambiguous SSA identity: %s", id))
		}
		byID[id] = row
		if isProject(f) && row.Selector != "" {
			bySelector[row.Selector] = append(bySelector[row.Selector], f)
		}
	}
	selected := map[*ssa.Function]bool{}
	queue := []*ssa.Function{}
	add := func(f *ssa.Function) {
		if !selected[f] {
			selected[f] = true
			queue = append(queue, f)
		}
	}
	rootIDs := map[string][]string{}
	for _, name := range req.Roots {
		candidates := bySelector[name]
		if len(candidates) == 0 {
			must(fmt.Errorf("missing source root: %s", name))
		}
		for _, f := range candidates {
			add(f)
			rootIDs[name] = append(rootIDs[name], key(f))
		}
		sort.Strings(rootIDs[name])
	}
	// Package initializers and function values they install remain part of the audit.
	for _, p := range project {
		f := prog.Package(p.Types).Func("init")
		if f != nil {
			add(f)
		}
	}
	sites := []Site{}
	boundaries := map[string]bool{}
	for len(queue) > 0 {
		f := queue[0]
		queue = queue[1:]
		if !isProject(f) {
			boundaries[key(f)] = true
			continue
		}
		calls := map[ssa.CallInstruction]map[*ssa.Function]bool{}
		if n := graph.Nodes[f]; n != nil {
			for _, edge := range n.Out {
				if calls[edge.Site] == nil {
					calls[edge.Site] = map[*ssa.Function]bool{}
				}
				calls[edge.Site][edge.Callee.Func] = true
			}
		}
		for _, block := range f.Blocks {
			for _, instruction := range block.Instrs {
				call, ok := instruction.(ssa.CallInstruction)
				if !ok {
					continue
				}
				common := call.Common()
				if _, ok := common.Value.(*ssa.Builtin); ok {
					continue
				}
				targets := calls[call]
				if targets == nil {
					targets = map[*ssa.Function]bool{}
				}
				if target := common.StaticCallee(); target != nil {
					targets[target] = true
				}
				kind := "static"
				if common.IsInvoke() {
					kind = "interface"
				} else if common.StaticCallee() == nil {
					kind = "function_value"
				}
				file, line, col := rel(call.Pos())
				ids := []string{}
				for target := range targets {
					ids = append(ids, key(target))
					add(target)
				}
				sort.Strings(ids)
				ids = slices.Compact(ids)
				sites = append(sites, Site{key(f), file, line, col, kind, typeName(common.Signature()), spelling(common.Value.String()), ids})
				// Sound boundary for callbacks handed to omitted external bodies. This also
				// records literal/method callbacks passed through project APIs: no claim of
				// value-flow precision is made. CHA assumes every matching function can flow.
				for _, arg := range common.Args {
					sig, ok := arg.Type().Underlying().(*types.Signature)
					if !ok {
						continue
					}
					callbackIDs := []string{}
					for candidate := range funcs {
						if isProject(candidate) && types.Identical(sig, candidate.Signature) {
							callbackIDs = append(callbackIDs, key(candidate))
							add(candidate)
						}
					}
					sort.Strings(callbackIDs)
					callbackIDs = slices.Compact(callbackIDs)
					sites = append(sites, Site{key(f), file, line, col, "callback_argument", typeName(sig), spelling(arg.String()), callbackIDs})
				}
			}
		}
	}
	functions := []Function{}
	external := []string{}
	unresolved := []Site{}
	emitted := map[string]bool{}
	for f := range selected {
		id := key(f)
		if isProject(f) && !emitted[id] {
			functions = append(functions, byID[id])
			emitted[id] = true
		}
	}
	for id := range boundaries {
		external = append(external, id)
	}
	sort.Slice(functions, func(i, j int) bool { return functions[i].ID < functions[j].ID })
	sort.Strings(external)
	sort.Slice(sites, func(i, j int) bool {
		a, _ := json.Marshal(sites[i])
		b, _ := json.Marshal(sites[j])
		return string(a) < string(b)
	})
	for _, s := range sites {
		if len(s.Targets) == 0 {
			unresolved = append(unresolved, s)
		}
	}
	interfaces := map[string][]Member{}
	for _, name := range req.Interfaces {
		parts := strings.Split(name, "::")
		p := pkgs[req.Prefix+parts[0]]
		if p == nil {
			must(fmt.Errorf("missing interface package %s", name))
		}
		obj := p.Types.Scope().Lookup(parts[1])
		if obj == nil {
			must(fmt.Errorf("missing interface %s", name))
		}
		iface, ok := obj.Type().Underlying().(*types.Interface)
		if !ok {
			must(fmt.Errorf("not interface %s", name))
		}
		iface.Complete()
		for i := 0; i < iface.NumMethods(); i++ {
			m := iface.Method(i)
			file, line, _ := rel(m.Pos())
			interfaces[name] = append(interfaces[name], Member{m.Name(), typeName(m.Type()), file, line})
		}
	}
	sources := []string{}
	for _, p := range project {
		for _, path := range p.CompiledGoFiles {
			r, e := filepath.Rel(*dir, path)
			must(e)
			sources = append(sources, filepath.ToSlash(r))
		}
	}
	sort.Strings(sources)
	must(json.NewEncoder(os.Stdout).Encode(map[string]any{"version": 1, "audit_target": map[string]string{"goos": req.GOOS, "goarch": req.GOARCH}, "roots": rootIDs, "functions": functions, "calls": sites, "external_boundaries": external, "unresolved": unresolved, "interfaces": interfaces, "sources": sources, "analysis": "SSA with instantiated generics and conservative CHA; callback signatures expanded; all project dependency initializers; no branch feasibility or execution claim; external library bodies are boundaries"}))
}
