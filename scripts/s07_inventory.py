#!/usr/bin/env python3
"""Reconstruct S07 source inventories and the pinned VS Code byte manifest.

Read-only verification is the default; --write updates reviewed artifacts.
This is inventory tooling, not a parity or experiment metric producer.
"""

import argparse
import collections
import csv
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
COMMIT = "a44adf7f53e00964ab890f9f8758a334f1fc15bc"
ARCHIVE_URL = f"https://codeload.github.com/microsoft/vscode/tar.gz/{COMMIT}"
ARCHIVE_SHA256 = "8b1b58e949bcc5b99967a2c2e19ed1f59ab3c41b88df6ee382dc7be39a06eb8d"
EXPECTED_FILES = 13094
EXPECTED_BYTES = 161740237
EXTENSIONS = (".ts", ".tsx", ".mts", ".cts")

# The standard Go type checker resolves promoted methods and selector receiver
# types. Export metadata is built by the pinned local Go compiler, not a model of
# binder syntax. No bridge is inserted into the canonical upstream checkout.
GO_INSPECTOR = r'''
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"go/types"
	"golang.org/x/tools/go/gcexportdata"
	"io"
	"os"
	"path/filepath"
	"strings"
)

type Package struct {
	ImportPath, Dir, Export string
	GoFiles                 []string
}
type exportImporter struct {
	fset    *token.FileSet
	exports map[string]string
	imports map[string]*types.Package
}

func (i exportImporter) Import(path string) (*types.Package, error) {
	if path == "unsafe" {
		return types.Unsafe, nil
	}
	if p := i.imports[path]; p != nil && p.Complete() {
		return p, nil
	}
	f, e := os.Open(i.exports[path])
	if e != nil {
		return nil, e
	}
	defer f.Close()
	r, e := gcexportdata.NewReader(f)
	if e != nil {
		return nil, e
	}
	return gcexportdata.Read(r, i.fset, i.imports, path)
}

type Anchor struct {
	File      string `json:"file"`
	Line      int    `json:"line"`
	Column    int    `json:"column"`
	EndLine   int    `json:"end_line"`
	EndColumn int    `json:"end_column"`
}
type Call struct {
	Anchor
	Package    string `json:"package"`
	Receiver   string `json:"receiver"`
	Name       string `json:"name"`
	Expression string `json:"expression"`
	Dynamic    bool   `json:"dynamic"`
}
type Write struct {
	Anchor
	Function     string `json:"function"`
	Target       string `json:"target"`
	Receiver     string `json:"receiver"`
	ReceiverType string `json:"receiver_type"`
	ValueType    string `json:"value_type"`
	Field        string `json:"field"`
	Operation    string `json:"operation"`
	Statement    string `json:"statement"`
	Indirect     bool   `json:"indirect"`
}

func main() {
	fset := token.NewFileSet()
	input, err := os.Open(os.Args[1])
	must(err)
	defer input.Close()
	exports := map[string]string{}
	var binder Package
	dec := json.NewDecoder(input)
	for {
		var p Package
		err := dec.Decode(&p)
		if err == io.EOF {
			break
		}
		must(err)
		exports[p.ImportPath] = p.Export
		if strings.HasSuffix(p.ImportPath, "/internal/binder") {
			binder = p
		}
	}
	imp := exportImporter{fset, exports, map[string]*types.Package{}}
	info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Uses: map[*ast.Ident]types.Object{}, Defs: map[*ast.Ident]types.Object{}, Selections: map[*ast.SelectorExpr]*types.Selection{}}
	files := []*ast.File{}
	for _, name := range binder.GoFiles {
		f, e := parser.ParseFile(fset, filepath.Join(binder.Dir, name), nil, 0)
		must(e)
		files = append(files, f)
	}
	config := types.Config{Importer: imp, Sizes: types.SizesFor("gc", "arm64")}
	_, err = config.Check(binder.ImportPath, fset, files, info)
	must(err)
	render := func(n ast.Node) string { var b bytes.Buffer; must(format.Node(&b, fset, n)); return b.String() }
	anchor := func(n ast.Node) Anchor {
		a, b := fset.Position(n.Pos()), fset.Position(n.End())
		return Anchor{"tsc/internal/binder/" + filepath.Base(a.Filename), a.Line, a.Column, b.Line, b.Column}
	}
	typ := func(e ast.Expr) string {
		t := info.TypeOf(e)
		if t == nil {
			return "unknown"
		}
		return types.TypeString(t, func(p *types.Package) string { return p.Path() })
	}
	calls := []Call{}
	writes := []Write{}
	for _, file := range files {
		for _, decl := range file.Decls {
			var body ast.Node = decl
			name := "<package-initializer>"
			if fn, ok := decl.(*ast.FuncDecl); ok {
				if fn.Body == nil {
					continue
				}
				body = fn.Body
				obj := info.Defs[fn.Name].(*types.Func)
				name = obj.Name()
				if rec := obj.Type().(*types.Signature).Recv(); rec != nil {
					name = types.TypeString(rec.Type(), func(*types.Package) string { return "" }) + "." + name
				}
			}
			ast.Inspect(body, func(n ast.Node) bool {
				if n == nil {
					return true
				}
				if call, ok := n.(*ast.CallExpr); ok {
					expr := call.Fun
					if x, ok := expr.(*ast.IndexExpr); ok {
						expr = x.X
					}
					if x, ok := expr.(*ast.IndexListExpr); ok {
						expr = x.X
					}
					var obj types.Object
					switch e := expr.(type) {
					case *ast.Ident:
						obj = info.Uses[e]
					case *ast.SelectorExpr:
						obj = info.Uses[e.Sel]
					}
					if f, ok := obj.(*types.Func); ok {
						pkg := ""
						if f.Pkg() != nil {
							pkg = f.Pkg().Path()
						}
						rec := ""
						if r := f.Type().(*types.Signature).Recv(); r != nil {
							rec = types.TypeString(r.Type(), func(*types.Package) string { return "" })
						}
						calls = append(calls, Call{Anchor: anchor(call), Package: pkg, Receiver: rec, Name: f.Name(), Expression: render(call.Fun)})
					} else if _, ok := obj.(*types.Var); ok || obj == nil {
						calls = append(calls, Call{Anchor: anchor(call), Expression: render(call.Fun), Dynamic: true})
					}
				}
				var targets []ast.Expr
				op := ""
				switch x := n.(type) {
				case *ast.AssignStmt:
					targets = x.Lhs
					op = x.Tok.String()
				case *ast.IncDecStmt:
					targets = []ast.Expr{x.X}
					op = x.Tok.String()
				}
				for _, target := range targets {
					w := Write{Anchor: anchor(target), Function: name, Target: render(target), Operation: op, Statement: render(n), ValueType: typ(target)}
					switch x := target.(type) {
					case *ast.SelectorExpr:
						w.Receiver = render(x.X)
						w.ReceiverType = typ(x.X)
						w.Field = x.Sel.Name
					case *ast.IndexExpr:
						w.Receiver = render(x.X)
						w.ReceiverType = typ(x.X)
						w.Field = "[element]"
					case *ast.StarExpr:
						w.Receiver = render(x.X)
						w.ReceiverType = typ(x.X)
						w.Field = "[indirect]"
						w.Indirect = true
					default:
						continue
					}
					writes = append(writes, w)
				}
				return true
			})
		}
	}
	must(json.NewEncoder(os.Stdout).Encode(map[string]any{"calls": calls, "writes": writes}))
}
func must(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
'''


def command(args, cwd=ROOT):
    return subprocess.check_output(args, cwd=cwd, env={**os.environ, "GOTOOLCHAIN": "local"})


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_or_check(path, content, write):
    if write:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    elif not path.exists() or path.read_bytes() != content:
        raise ValueError(f"inventory drift: {path.relative_to(ROOT)} (review and use --write)")


def json_artifact(name, data, write):
    write_or_check(ROOT / "data/s07" / name,
                   (json.dumps(data, indent=2, ensure_ascii=True) + "\n").encode(), write)


def upstream_pin():
    expected = tomllib.loads((ROOT / "PORTS.toml").read_text())["pin"]
    actual = command(["git", "rev-parse", "HEAD"], ROOT / "upstream").decode().strip()
    if actual != expected or command(["git", "status", "--porcelain"], ROOT / "upstream").strip():
        raise ValueError("inventory requires the clean pinned upstream checkout")
    version = command(["go", "version"]).decode().split()[2]
    if version != tomllib.loads((ROOT / "data/s04/toolchains.toml").read_text())["go"]:
        raise ValueError("inventory requires the manifest-pinned local Go toolchain")
    return expected, version


def normalized_receiver(value):
    return re.sub(r"\[.*\]", "", value.lstrip("*"))


def inventory(write):
    pin, version = upstream_pin()
    ledger = {f["go"]: f for f in tomllib.loads((ROOT / "PORTS.toml").read_text())["file"]}
    with (ROOT / "data/go-functions.tsv").open() as source:
        functions = list(csv.DictReader((line for line in source if not line.startswith("#")), delimiter="\t"))
    by_name = collections.defaultdict(list)
    for f in functions:
        by_name[(f["package"], f["receiver"], f["name"])].append(f)
    mappings = collections.defaultdict(list)
    provenance = collections.defaultdict(list)
    for source in sorted((ROOT / "crates").rglob("*.rs")):
        for line, text in enumerate(source.read_text().splitlines(), 1):
            for marker in re.findall(r"\bport:\s+(tsc/\S+)", text):
                mappings[marker].append({"file": source.relative_to(ROOT).as_posix(), "line": line})
            for marker in re.findall(r"\bupstream:\s+(tsc/\S+:\S+)", text):
                provenance[marker].append({"file": source.relative_to(ROOT).as_posix(), "line": line})
    with tempfile.TemporaryDirectory(prefix="s07-inventory-") as temp:
        temp = Path(temp)
        (temp / "packages.json").write_bytes(command(["go", "list", "-export", "-deps", "-json", "./internal/binder"], ROOT / "upstream/tsc"))
        (temp / "inspect.go").write_text(GO_INSPECTOR)
        observations = json.loads(command(["go", "run", str(temp / "inspect.go"), str(temp / "packages.json")], ROOT / "upstream/tsc"))
    def row(f):
        l = ledger.get(f["file"], {})
        return {"id": f["id"], "file": f["file"], "package": f["package"],
                "receiver": f["receiver"], "name": f["name"],
                "start_line": int(f["start"]), "end_line": int(f["end"]),
                "kind": l.get("kind", "unknown"), "planned_crate": l.get("crate"),
                "rust_mappings": mappings.get(f["id"], []),
                "rust_generated_provenance": provenance.get(f["id"], [])}
    closure = {}; unresolved = []; dynamic = []; standard = []
    for call in observations["calls"]:
        if call["dynamic"]:
            dynamic.append(call)
            continue
        package = call["package"].removeprefix("github.com/microsoft/TypeScript/tsc/")
        candidates = by_name[(package, normalized_receiver(call["receiver"]), call["name"])]
        if len(candidates) == 1:
            f = candidates[0]
            entry = closure.setdefault(f["id"], {**row(f), "call_sites": []})
            entry["call_sites"].append(call)
        elif call["package"].startswith("github.com/microsoft/TypeScript/tsc/"):
            unresolved.append({**call, "candidate_ids": [f["id"] for f in candidates]})
        else:
            standard.append(call)
    source_functions = [row(f) for f in functions if f["package"] == "internal/binder" and ledger[f["file"]]["kind"] == "source"]
    document = {"version": 1, "upstream_pin": pin, "go_toolchain": version,
                "authority": "Pinned Go parser/type checker; data/go-functions.tsv and SOURCE ledger classification",
                "scope": "All binder source functions and direct statically resolved calls, including promoted methods; this is not a transitive helper closure or semantic coverage proof.",
                "rust_mapping_semantics": "Observed port comments only; empty means no current mapping, never proof that equivalent code is absent.",
                "source_function_count": len(source_functions), "source_functions": source_functions,
                "direct_call_dependencies": [closure[key] for key in sorted(closure)],
                "unresolved_project_calls": unresolved, "dynamic_calls": dynamic, "standard_library_calls": standard}
    json_artifact("dependency-functions.json", document, write)
    observed = []; internal = []
    for event in observations["writes"]:
        typ = event["receiver_type"]
        if "/internal/ast." not in typ:
            internal.append(event)
            continue
        field = event["field"]
        if "SymbolTable" in typ:
            family = "symbol_tables"
        elif re.search(r"ast\.Symbol(?:$|\])", typ):
            family = "symbols_and_declaration_lists"
        elif re.search(r"ast\.(?:FlowNode|FlowLabel|FlowList)$", typ):
            family = "flow_nodes_and_lists"
        elif field in ("Symbol", "LocalSymbol", "locals", "NextContainer"):
            family = "node_symbol_and_container_side_tables"
        elif "Flow" in field:
            family = "node_flow_side_tables"
        elif field == "Flags":
            family = "bound_node_flags_overlay"
        elif "ast.SourceFile" in typ:
            family = "bound_source_file_overlay"
        else:
            family = "bound_node_payload_overlay_review_required"
        observed.append({**event, "intended_overlay_family": family,
                         "classification": "proposed_from_resolved_receiver_type_and_field"})
    helper_specs = {
        "tsc/internal/ast/utilities.go:GetSymbolTable": ("symbol_tables", "Lazily writes an allocated-empty table through *data when nil; pointer arguments can name SourceFile.GlobalExports as well as symbol/node tables."),
        "tsc/internal/ast/utilities.go:GetMembers": ("symbols_and_declaration_lists", "Calls GetSymbolTable with &symbol.Members."),
        "tsc/internal/ast/utilities.go:GetExports": ("symbols_and_declaration_lists", "Calls GetSymbolTable with &symbol.Exports."),
        "tsc/internal/ast/utilities.go:GetLocals": ("node_symbol_and_container_side_tables", "Calls GetSymbolTable with &container.LocalsContainerData().Locals."),
        "tsc/internal/ast/ast.go:SourceFile.SetBindDiagnostics": ("bound_source_file_overlay", "Replaces SourceFile.bindDiagnostics with the appended diagnostic slice supplied by the binder."),
        "tsc/internal/ast/ast.go:SourceFile.BindOnce": ("file_binding_lifecycle", "Consumes sync.Once even on panic; writes isBound only after the initializer returns successfully."),
        "tsc/internal/ast/utilities.go:GetNodeId": ("identity_observation", "May increment process-global nextNodeId and CAS Node.id; binder-derived symbol names embed this identity."),
        "tsc/internal/ast/utilities.go:GetSymbolId": ("identity_observation", "May increment process-global nextSymbolId and CAS Symbol.id; private-identifier symbol names embed this identity."),
        "tsc/internal/ast/flow.go:NewFlowSwitchClauseData": ("synthetic_flow_payload", "Allocates an AST node and initializes SwitchStatement, ClauseStart and ClauseEnd; source clause indices narrow to int32."),
        "tsc/internal/ast/flow.go:NewFlowReduceLabelData": ("synthetic_flow_payload", "Allocates an AST node and initializes Target and Antecedents; retains their graph identities through flow ownership."),
    }
    helpers = []
    for identifier, (family, effect) in helper_specs.items():
        definition = next(f for f in functions if f["id"] == identifier)
        helpers.append({**row(definition), "source_sha256": sha((ROOT / "upstream" / definition["file"]).read_bytes()),
                        "intended_overlay_family": family, "effect": effect,
                        "direct_call_sites": closure.get(identifier, {}).get("call_sites", [])})
    writes = {"version": 1, "upstream_pin": pin, "go_toolchain": version,
              "scope": "Every assignment/incdec selector, indexed or indirect target in the three binder source files. Local-variable reassignment is excluded. Helper-induced mutations require auditing the separately inventoried callees.",
              "limitations": "Overlay families are design proposals, not generated runtime layout. Non-AST targets are retained for audit rather than discarded. Calls such as GetLocals/GetMembers/GetExports can allocate/mutate through their source helpers and are not disguised as direct assignments.",
              "ast_write_count": len(observed), "ast_writes": observed,
              "reviewed_helper_mutations": helpers,
              "identity_derived_name_sites": [
                  {"file": "tsc/internal/binder/binder.go", "line": 314, "function": "Binder.getDeclarationName", "identity": "ast.GetNodeId(attributes)", "observation_requirement": "Represent the referenced node identity separately from the literal internal-name prefix/module-name/pattern suffix; comparing raw process-global numeric text is not allocation-independent."},
                  {"file": "tsc/internal/binder/binder.go", "line": 375, "function": "GetSymbolNameForPrivateIdentifier", "identity": "ast.GetSymbolId(containingClassSymbol)", "observation_requirement": "Represent the containing class symbol identity and description separately; preserve ordinary raw symbol names exactly."}],
              "other_write_count": len(internal), "other_writes": internal}
    json_artifact("binder-writes.json", writes, write)
    print(json.dumps({"binder_source_functions": len(source_functions), "resolved_direct_functions": len(closure),
                      "unresolved_calls": len(unresolved), "ast_writes": len(observed), "other_writes": len(internal)}))


def workload(write, cache, offline):
    cache.mkdir(parents=True, exist_ok=True)
    archive = cache / f"vscode-{COMMIT}.tar.gz"
    if not archive.exists():
        if offline:
            raise ValueError(f"workload archive is not provisioned: {archive}")
        with urllib.request.urlopen(ARCHIVE_URL) as response, tempfile.NamedTemporaryFile(dir=cache, delete=False) as output:
            temporary = Path(output.name)
            try:
                while chunk := response.read(1024 * 1024):
                    output.write(chunk)
            except BaseException:
                temporary.unlink(missing_ok=True)
                raise
        temporary.replace(archive)
    with archive.open("rb") as source:
        archive_hash = hashlib.file_digest(source, "sha256").hexdigest()
    if archive_hash != ARCHIVE_SHA256:
        raise ValueError("workload archive SHA-256 differs from the frozen archive")
    prefix = f"vscode-{COMMIT}/"
    rows = []; seen = set(); license_hash = None
    with tarfile.open(archive, "r:gz") as source:
        for entry in source:
            if not entry.name.startswith(prefix) and entry.name != prefix.rstrip("/"):
                raise ValueError("archive has an unexpected root")
            path = entry.name.removeprefix(prefix)
            if path == "LICENSE.txt" and entry.isfile():
                license_hash = sha(source.extractfile(entry).read())
            if not entry.isfile() or not path.endswith(EXTENSIONS):
                continue
            if path in seen or path.startswith("/") or ".." in Path(path).parts:
                raise ValueError(f"duplicate/unsafe source path: {path}")
            seen.add(path)
            data = source.extractfile(entry).read()
            rows.append({"path": path, "bytes": len(data), "sha256": sha(data)})
    rows.sort(key=lambda row: row["path"])
    if len(rows) != EXPECTED_FILES or sum(row["bytes"] for row in rows) != EXPECTED_BYTES or license_hash is None:
        raise ValueError("archive bytes do not reproduce the independently observed tree totals/license")
    document = {"version": 1, "repository": "https://github.com/microsoft/vscode", "tag": "1.136.1", "commit": COMMIT,
                "archive_url": ARCHIVE_URL, "archive_sha256": archive_hash,
                "license_path": "LICENSE.txt", "license_sha256": license_hash,
                "selection": "Every regular tracked source-tree file ending .ts, .tsx, .mts or .cts; includes declarations and tests; each physical file once; no external dependency installation or generated build output.",
                "source_loading": "Raw archive bytes; runtime loaded-byte hashes/effective parser options remain a separate preflight obligation, not certified by this byte inventory.",
                "file_count": len(rows), "source_bytes": sum(row["bytes"] for row in rows),
                "extension_counts": {ext: sum(row["path"].endswith(ext) for row in rows) for ext in EXTENSIONS}, "files": rows}
    json_artifact("vscode-files.json", document, write)
    options_path = ROOT / "data/s07/vscode-parse-options.json"
    if not options_path.is_file():
        raise ValueError("workload parser options require the independent pinned-Go benchmark preflight")
    option_rows = json.loads(options_path.read_bytes())["files"]
    if [row["filename"] for row in option_rows] != ["/vscode/" + row["path"] for row in rows]:
        raise ValueError("workload parser options have missing or reordered files")
    toml = f'''# Source-byte inventory only; this does not certify parser/binder execution.
[vscode]
repository = "https://github.com/microsoft/vscode"
tag = "1.136.1"
commit = "{COMMIT}"
archive_url = "{ARCHIVE_URL}"
archive_sha256 = "{archive_hash}"
license = "MIT"
license_path = "LICENSE.txt"
license_sha256 = "{license_hash}"
manifest = "data/s07/vscode-files.json"
manifest_sha256 = "{sha((json.dumps(document, indent=2, ensure_ascii=True) + chr(10)).encode())}"
extensions = [".ts", ".tsx", ".mts", ".cts"]
file_count = {len(rows)}
source_bytes = {sum(row["bytes"] for row in rows)}
upstream_library_pin = "1f70213d4922b434345f639b441681e470c7cfc1"
intended_target = "ESNext"
intended_script_kind = "pinned extension-derived"
effective_parse_options = "data/s07/vscode-parse-options.json"
effective_parse_options_sha256 = "{sha(options_path.read_bytes())}"
effective_parse_options_authority = "pinned Go GetExternalModuleIndicatorOptions and GetScriptKindFromFileName; ESNext compiler option with empty package metadata"
'''
    write_or_check(ROOT / "data/workloads.toml", toml.encode(), write)
    print(json.dumps({"archive_sha256": archive_hash, "files": len(rows), "bytes": document["source_bytes"]}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("inventory", "workload"))
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--cache-dir", type=Path, default=ROOT.parent / ".ts-rust-workloads")
    args = parser.parse_args()
    if args.operation == "inventory":
        inventory(args.write)
    else:
        workload(args.write, args.cache_dir, args.offline)


if __name__ == "__main__":
    main()
