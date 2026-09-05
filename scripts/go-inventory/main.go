// go-inventory lists every top-level Go function and method of the pinned tsc module.
//
//	go run scripts/go-inventory/main.go <path-to-TypeScript>/tsc > data/go-functions.tsv
//
// The output is the denominator for function-level port coverage: the status tool
// matches "port: <file>:<name>" markers in Rust sources against these rows.
package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: go-inventory <tsc module dir>")
		os.Exit(2)
	}
	root := os.Args[1]
	fset := token.NewFileSet()
	fmt.Println("file\tpackage\treceiver\tname\tstart\tend")
	err := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			name := d.Name()
			rel, _ := filepath.Rel(root, path)
			if name == "testdata" || strings.HasPrefix(name, "_") || strings.HasPrefix(name, ".") || filepath.ToSlash(rel) == "internal/fourslash/tests" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(path, ".go") || strings.HasSuffix(path, "_test.go") {
			return nil
		}
		f, err := parser.ParseFile(fset, path, nil, parser.SkipObjectResolution)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			return nil
		}
		rel, _ := filepath.Rel(root, path)
		rel = filepath.ToSlash(rel)
		pkg := filepath.ToSlash(filepath.Dir(rel))
		for _, decl := range f.Decls {
			fn, ok := decl.(*ast.FuncDecl)
			if !ok {
				continue
			}
			recv := ""
			if fn.Recv != nil && len(fn.Recv.List) > 0 {
				recv = exprString(fn.Recv.List[0].Type)
			}
			fmt.Printf("tsc/%s\t%s\t%s\t%s\t%d\t%d\n", rel, pkg, recv, fn.Name.Name, fset.Position(fn.Pos()).Line, fset.Position(fn.End()).Line)
		}
		return nil
	})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func exprString(e ast.Expr) string {
	switch t := e.(type) {
	case *ast.StarExpr:
		return "*" + exprString(t.X)
	case *ast.Ident:
		return t.Name
	case *ast.IndexExpr:
		return exprString(t.X)
	case *ast.IndexListExpr:
		return exprString(t.X)
	default:
		return "?"
	}
}
