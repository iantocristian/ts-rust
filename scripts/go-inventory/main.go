// go-inventory lists every top-level Go function and method of the pinned tsc module.
//
//	go run scripts/go-inventory/main.go --pin <commit> <path-to-TypeScript>/tsc
//
// Use ledger-init.py to publish this together with the ledger and hash manifest.
// All input comes from pinned Git blobs in a verified clean checkout.
package main

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

func main() {
	if err := run(os.Args[1:], os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, "go-inventory:", err)
		os.Exit(1)
	}
}

func git(root string, args ...string) ([]byte, error) {
	cmd := exec.Command("git", append([]string{"-C", root}, args...)...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("git %s: %s", strings.Join(args, " "), strings.TrimSpace(string(output)))
	}
	return output, nil
}

func resolvePin(root, pin string) (string, error) {
	output, err := git(root, "rev-parse", "--verify", "--end-of-options", pin+"^{commit}")
	return strings.TrimSpace(string(output)), err
}

func cleanPin(root, requested string) (string, error) {
	head, err := resolvePin(root, "HEAD")
	if err != nil {
		return "", err
	}
	if requested != "" {
		pin, err := resolvePin(root, requested)
		if err != nil {
			return "", err
		}
		if pin != head {
			return "", fmt.Errorf("requested pin %s does not match upstream HEAD %s", requested, head)
		}
	}
	status, err := git(root, "status", "--porcelain=v1", "--untracked-files=all")
	if err != nil {
		return "", err
	}
	if len(status) != 0 {
		return "", fmt.Errorf("upstream checkout is dirty")
	}
	return head, nil
}

func included(path string) bool {
	if !(strings.HasPrefix(path, "tsc/internal/") || strings.HasPrefix(path, "tsc/cmd/")) ||
		!strings.HasSuffix(path, ".go") || strings.HasSuffix(path, "_test.go") ||
		strings.HasPrefix(path, "tsc/internal/fourslash/tests/") {
		return false
	}
	parts := strings.Split(path, "/")
	for _, part := range parts[:len(parts)-1] {
		if part == "testdata" || strings.HasPrefix(part, "_") || strings.HasPrefix(part, ".") {
			return false
		}
	}
	return true
}

func run(args []string, output io.Writer) error {
	flags := flag.NewFlagSet("go-inventory", flag.ContinueOnError)
	requested := flags.String("pin", "", "commit that must match the clean checkout's HEAD")
	if err := flags.Parse(args); err != nil {
		return err
	}
	if flags.NArg() != 1 {
		return fmt.Errorf("usage: go-inventory [--pin COMMIT] <tsc module dir>")
	}
	module, err := filepath.Abs(flags.Arg(0))
	if err != nil {
		return err
	}
	top, err := git(module, "rev-parse", "--show-toplevel")
	if err != nil {
		return err
	}
	root := strings.TrimSpace(string(top))
	resolvedModule, err := filepath.EvalSymlinks(module)
	if err != nil {
		return err
	}
	resolvedRoot, err := filepath.EvalSymlinks(root)
	if err != nil {
		return err
	}
	if resolvedModule != filepath.Join(resolvedRoot, "tsc") {
		return fmt.Errorf("expected the upstream checkout's tsc directory")
	}
	pin, err := cleanPin(root, *requested)
	if err != nil {
		return err
	}
	tree, err := git(root, "ls-tree", "-rz", "--name-only", pin, "--", "tsc/internal", "tsc/cmd")
	if err != nil {
		return err
	}
	var paths []string
	var requests strings.Builder
	for _, path := range strings.Split(string(tree), "\x00") {
		if !included(path) {
			continue
		}
		if strings.ContainsAny(path, "\t\r\n") {
			return fmt.Errorf("unsupported control character in source path %q", path)
		}
		paths = append(paths, path)
		fmt.Fprintf(&requests, "%s:%s\n", pin, path)
	}
	if len(paths) == 0 {
		return fmt.Errorf("upstream pin contains no compiler Go source files")
	}
	cmd := exec.Command("git", "-C", root, "cat-file", "--batch")
	cmd.Stdin = strings.NewReader(requests.String())
	var stderr bytes.Buffer
	cmd.Stderr = &stderr
	blobs, err := cmd.Output()
	if err != nil {
		return fmt.Errorf("read pinned source: %s", strings.TrimSpace(stderr.String()))
	}
	reader := bufio.NewReader(bytes.NewReader(blobs))
	fset := token.NewFileSet()
	var result bytes.Buffer
	fmt.Fprintf(&result, "# upstream %s\nfile\tpackage\treceiver\tname\tstart\tend\tid\n", pin)
	seen := make(map[string]bool)
	for _, path := range paths {
		header, err := reader.ReadString('\n')
		if err != nil {
			return err
		}
		fields := strings.Fields(header)
		if len(fields) != 3 || fields[1] != "blob" {
			return fmt.Errorf("cannot read pinned source %s: %s", path, header)
		}
		size, err := strconv.Atoi(fields[2])
		if err != nil || size < 0 {
			return fmt.Errorf("invalid Git blob size for %s", path)
		}
		content := make([]byte, size)
		if _, err := io.ReadFull(reader, content); err != nil {
			return err
		}
		if newline, err := reader.ReadByte(); err != nil || newline != '\n' {
			return fmt.Errorf("invalid Git blob terminator for %s", path)
		}
		f, err := parser.ParseFile(fset, path, content, parser.SkipObjectResolution)
		if err != nil {
			return err // Never silently reduce the coverage denominator.
		}
		pkg := strings.TrimPrefix(filepath.ToSlash(filepath.Dir(path)), "tsc/")
		initIndex := 0
		for _, decl := range f.Decls {
			fn, ok := decl.(*ast.FuncDecl)
			if !ok {
				continue
			}
			recv := ""
			if fn.Recv != nil && len(fn.Recv.List) > 0 {
				recv, err = receiverName(fn.Recv.List[0].Type)
				if err != nil {
					return fmt.Errorf("%s:%d: %w", path, fset.Position(fn.Pos()).Line, err)
				}
			}
			name := fn.Name.Name
			if recv != "" {
				name = recv + "." + name
			} else if name == "init" {
				initIndex++
				name = fmt.Sprintf("init#%d", initIndex)
			}
			id := path + ":" + name
			if seen[id] {
				return fmt.Errorf("duplicate function inventory ID %s", id)
			}
			seen[id] = true
			fmt.Fprintf(&result, "%s\t%s\t%s\t%s\t%d\t%d\t%s\n", path, pkg, recv, fn.Name.Name,
				fset.Position(fn.Pos()).Line, fset.Position(fn.End()).Line, id)
		}
	}
	if _, err := cleanPin(root, pin); err != nil {
		return err
	}
	_, err = output.Write(result.Bytes())
	return err
}

func receiverName(e ast.Expr) (string, error) {
	switch t := e.(type) {
	case *ast.StarExpr:
		return receiverName(t.X)
	case *ast.Ident:
		return t.Name, nil
	case *ast.IndexExpr:
		return receiverName(t.X)
	case *ast.IndexListExpr:
		return receiverName(t.X)
	default:
		return "", fmt.Errorf("unsupported receiver syntax %T", e)
	}
}
