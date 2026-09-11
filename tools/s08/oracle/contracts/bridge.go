package checker

import (
	"encoding/hex"
	"fmt"
	"math/rand/v2"
	"slices"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
)

// Access-only instrumentation, installed in the pinned checkTypeRelatedToEx.
// The contract driver is sequential; this observer is never a timing executable.
var s08RelationObserver func(*Checker, Ternary)

type S08RelationAction struct {
	Mode         string `json:"mode"`
	Source       string `json:"source"`
	Target       string `json:"target"`
	ReportErrors bool   `json:"report_errors"`
}

func S08Diagnostics(ds []*ast.Diagnostic) []map[string]any {
	out := make([]map[string]any, 0, len(ds))
	for _, d := range ds {
		var file any
		if d.File() != nil {
			file = d.File().FileName()
		}
		out = append(out, map[string]any{"file": file, "pos": d.Pos(), "end": d.End(), "code": d.Code(), "category": d.Category(), "key_hex": hex.EncodeToString([]byte(d.MessageKey())), "text_hex": hex.EncodeToString([]byte(d.MessageText())), "args": d.MessageArgs(), "chain": S08Diagnostics(d.MessageChain()), "related": S08Diagnostics(d.RelatedInformation())})
	}
	return out
}

func S08State(c *Checker) map[string]any {
	relations := map[string]*Relation{"identity": c.identityRelation, "assignable": c.assignableRelation, "subtype": c.subtypeRelation, "strict_subtype": c.strictSubtypeRelation, "comparable": c.comparableRelation}
	caches := map[string]any{}
	for name, r := range relations {
		flags := []uint32{}
		for _, f := range r.results {
			flags = append(flags, uint32(f))
		}
		slices.Sort(flags)
		caches[name] = map[string]any{"entries": len(r.results), "result_flags": flags}
	}
	return map[string]any{"types_created": c.TypeCount, "signatures_created": c.SignatureCount, "instantiations": c.TotalInstantiationCount, "caches": caches}
}

func S08ObserveRelation(c *Checker, file *ast.SourceFile, actions []S08RelationAction) map[string]any {
	declarations := map[string]*ast.Node{}
	for _, n := range file.Statements.Nodes {
		if name := n.Name(); name != nil {
			declarations[name.Text()] = n
		}
	}
	types := map[string]*Type{}
	nodes := map[string]*ast.Node{}
	beforeLookup := S08State(c)
	for _, action := range actions {
		for _, name := range []string{action.Source, action.Target} {
			if types[name] != nil {
				continue
			}
			n := declarations[name]
			if n == nil {
				panic("missing fixture declaration: " + name)
			}
			symbol := c.GetSymbolAtLocation(n.Name())
			if symbol == nil {
				panic("fixture symbol missing: " + name)
			}
			types[name] = c.GetDeclaredTypeOfSymbol(symbol)
			nodes[name] = n
		}
	}
	observations := []map[string]any{}
	for _, action := range actions {
		var relation *Relation
		switch action.Mode {
		case "identity":
			relation = c.identityRelation
		case "assignable":
			relation = c.assignableRelation
		case "subtype":
			relation = c.subtypeRelation
		case "strict_subtype":
			relation = c.strictSubtypeRelation
		case "comparable":
			relation = c.comparableRelation
		default:
			panic("unknown relation")
		}
		before := S08State(c)
		ternaries := []int{}
		s08RelationObserver = func(owner *Checker, result Ternary) {
			if owner == c {
				ternaries = append(ternaries, int(result))
			}
		}
		var errorNode *ast.Node
		if action.ReportErrors {
			errorNode = nodes[action.Source]
		}
		diagnostics := []*ast.Diagnostic{}
		result := c.checkTypeRelatedToEx(types[action.Source], types[action.Target], relation, errorNode, nil, &diagnostics)
		s08RelationObserver = nil
		observations = append(observations, map[string]any{"action": action, "result": result, "ternary_calls": ternaries, "diagnostics": S08Diagnostics(diagnostics), "before": before, "after": S08State(c)})
	}
	// Display is deliberately after the relation sequence, so it cannot warm member
	// resolution before the cold relation operation. No numeric type IDs are compared.
	names := make([]string, 0, len(types))
	for name := range types {
		names = append(names, name)
	}
	slices.Sort(names)
	texts := map[string]string{}
	flags := map[string]uint32{}
	aliasTexts := map[string]string{}
	ordering := []map[string]any{}
	for _, name := range names {
		texts[name] = hex.EncodeToString([]byte(c.TypeToString(types[name])))
		flags[name] = uint32(types[name].flags)
		aliasTexts[name] = hex.EncodeToString([]byte(c.TypeToStringEx(types[name], nodes[name], TypeFormatFlagsInTypeAlias, nil)))
	}
	for _, name := range names {
		typ := types[name]
		if typ.flags&TypeFlagsUnion == 0 {
			continue
		}
		original := typ.Types()
		permutations := [][]int{}
		reverse := make([]int, len(original))
		for i := range reverse {
			reverse[i] = len(original) - 1 - i
		}
		permutations = append(permutations, reverse)
		for seed := uint64(0); seed < 10; seed++ {
			indexes := make([]int, len(original))
			for i := range indexes {
				indexes[i] = i
			}
			rng := rand.New(rand.NewPCG(seed, 17))
			rng.Shuffle(len(indexes), func(i, j int) { indexes[i], indexes[j] = indexes[j], indexes[i] })
			permutations = append(permutations, indexes)
		}
		pairwise := [][]int{}
		for _, a := range original {
			row := []int{}
			for _, b := range original {
				row = append(row, CompareTypes(a, b))
			}
			pairwise = append(pairwise, row)
		}
		for _, permutation := range permutations {
			indexes := slices.Clone(permutation)
			slices.SortFunc(indexes, func(i, j int) int { return CompareTypes(original[i], original[j]) })
			ordering = append(ordering, map[string]any{"type": name, "input": permutation, "sorted": indexes})
		}
		ordering = append(ordering, map[string]any{"type": name, "pairwise": pairwise})
	}
	return map[string]any{"before_lookup": beforeLookup, "actions": observations, "display_hex": texts, "in_alias_display_hex": aliasTexts, "type_flags": flags, "union_ordering": ordering, "final": S08State(c)}
}

func S08ResetObserver() { s08RelationObserver = nil }
func S08AssertModeCoverage(actions []S08RelationAction) {
	m := map[string]bool{}
	for _, a := range actions {
		m[a.Mode] = true
	}
	if len(m) != 5 {
		panic(fmt.Sprintf("fixture modes: %d", len(m)))
	}
}
