#!/usr/bin/env python3
"""Generate PORTS.toml, the port ledger, from the pinned upstream checkout.

Usage: scripts/ledger-init.py [path-to-TypeScript-checkout] [--pin COMMIT]

One entry per non-test Go file under tsc/internal and tsc/cmd. Existing statuses
and rust paths in a current PORTS.toml are preserved when the file is regenerated,
so this can be re-run after a pin bump without losing progress.
"""
import os, re, subprocess, sys

UPSTREAM = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("--") else os.path.expanduser("~/git/TypeScript")
PIN = None
if "--pin" in sys.argv:
    PIN = sys.argv[sys.argv.index("--pin") + 1]
if PIN is None:
    PIN = subprocess.check_output(["git", "-C", UPSTREAM, "rev-parse", "--short", "HEAD"], text=True).strip()
ROOT = os.path.join(UPSTREAM, "tsc")
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "PORTS.toml")

# Longest-prefix map from Go package (relative to tsc/) to (Rust crate, parity phase).
CRATES = {
    "cmd/tsc": ("tsc", 4),
    "internal/api/encoder": ("ts_encoder", 0),
    "internal/api": ("ts_api", 6),
    "internal/ast": ("ts_ast", 0),
    "internal/astnav": ("ts_astnav", 1),
    "internal/binder": ("ts_binder", 1),
    "internal/bundled": ("ts_bundled", 1),
    "internal/checker": ("ts_checker", 2),
    "internal/collections": ("ts_collections", 1),
    "internal/compiler": ("ts_compiler", 4),
    "internal/contentmapper": ("ts_contentmapper", 5),
    "internal/core": ("ts_core", 1),
    "internal/debug": ("ts_core", 1),
    "internal/diagnostics": ("ts_diagnostics", 1),
    "internal/diagnosticwriter": ("ts_diagnosticwriter", 4),
    "internal/evaluator": ("ts_evaluator", 1),
    "internal/execute/build": ("ts_build", 4),
    "internal/execute/incremental": ("ts_incremental", 4),
    "internal/execute/tsc": ("ts_execute", 4),
    "internal/execute/tsctests": ("ts_tsctests", 4),
    "internal/execute/watchmanager": ("ts_execute", 4),
    "internal/execute": ("ts_execute", 4),
    "internal/format": ("ts_format", 5),
    "internal/fourslash": ("ts_fourslash", 5),
    "internal/fswatch": ("ts_fswatch", 4),
    "internal/glob": ("ts_glob", 1),
    "internal/ipc": ("ts_ipc", 6),
    "internal/jsnum": ("ts_jsnum", 1),
    "internal/json": ("ts_json", 1),
    "internal/jsonrpc": ("ts_jsonrpc", 6),
    "internal/locale": ("ts_locale", 1),
    "internal/ls/autoimport": ("ts_autoimport", 5),
    "internal/ls": ("ts_ls", 5),
    "internal/lsp/lsproto": ("ts_lsproto", 5),
    "internal/lsp": ("ts_lsp", 5),
    "internal/module": ("ts_module", 1),
    "internal/modulespecifiers": ("ts_modulespecifiers", 2),
    "internal/nativepath": ("ts_nativepath", 4),
    "internal/nodebuilder": ("ts_nodebuilder", 2),
    "internal/osutil": ("ts_core", 1),
    "internal/outputpaths": ("ts_outputpaths", 3),
    "internal/packagejson": ("ts_packagejson", 1),
    "internal/parser": ("ts_parser", 0),
    "internal/pprof": ("ts_pprof", 4),
    "internal/printer": ("ts_printer", 3),
    "internal/project": ("ts_project", 5),
    "internal/pseudochecker": ("ts_pseudochecker", 3),
    "internal/repo": ("ts_testutil", 1),
    "internal/scanner": ("ts_scanner", 0),
    "internal/semver": ("ts_semver", 1),
    "internal/sourcemap": ("ts_sourcemap", 3),
    "internal/spanmap": ("ts_spanmap", 5),
    "internal/stringutil": ("ts_stringutil", 1),
    "internal/symlinks": ("ts_core", 1),
    "internal/testrunner": ("ts_testrunner", 1),
    "internal/testutil": ("ts_testutil", 1),
    "internal/tracing": ("ts_tracing", 4),
    "internal/transformers/declarations": ("ts_declarations", 3),
    "internal/transformers": ("ts_transformers", 3),
    "internal/transpile": ("ts_transpile", 3),
    "internal/tsoptions": ("ts_tsoptions", 1),
    "internal/tspath": ("ts_tspath", 1),
    "internal/vfs": ("ts_vfs", 1),
}
HARNESS_PREFIXES = ("internal/testutil", "internal/testrunner", "internal/fourslash", "internal/execute/tsctests",
                    "internal/repo", "internal/vfs/vfstest", "internal/vfs/vfsmock", "internal/tsoptions/tsoptionstest")
TARGETS = [("darwin", "arm64"), ("darwin", "amd64"), ("linux", "amd64"), ("linux", "arm64")]
KNOWN_OS = {"aix", "android", "darwin", "dragonfly", "freebsd", "illumos", "ios", "js", "linux", "netbsd", "openbsd", "plan9", "solaris", "wasip1", "windows"}
KNOWN_ARCH = {"386", "amd64", "arm", "arm64", "loong64", "mips", "mips64", "mips64le", "mipsle", "ppc64", "ppc64le", "riscv64", "s390x", "wasm"}


def crate_for(pkg):
    best = None
    for prefix, val in CRATES.items():
        if pkg == prefix or pkg.startswith(prefix + "/"):
            if best is None or len(prefix) > len(best[0]):
                best = (prefix, val)
    return best[1] if best else ("unmapped", 9)


def eval_constraint(expr, goos, goarch):
    """Evaluate a //go:build expression for one target. Unknown tags are false."""
    tokens = re.findall(r"\(|\)|&&|\|\||!|[A-Za-z0-9_.]+", expr)
    pos = 0

    def tag(t):
        if t == goos or t == goarch:
            return True
        if t == "unix":
            return goos in {"darwin", "linux", "freebsd", "netbsd", "openbsd", "dragonfly", "aix", "solaris", "illumos", "ios", "android"}
        if t.startswith("go1."):
            return True
        return False

    def parse_or():
        nonlocal pos
        v = parse_and()
        while pos < len(tokens) and tokens[pos] == "||":
            pos += 1
            v = parse_and() or v
        return v

    def parse_and():
        nonlocal pos
        v = parse_not()
        while pos < len(tokens) and tokens[pos] == "&&":
            pos += 1
            v = parse_not() and v
        return v

    def parse_not():
        nonlocal pos
        if tokens[pos] == "!":
            pos += 1
            return not parse_not()
        if tokens[pos] == "(":
            pos += 1
            v = parse_or()
            pos += 1  # ')'
            return v
        t = tokens[pos]
        pos += 1
        return tag(t)

    return parse_or()


def in_scope(path, head):
    base = os.path.basename(path)[:-3]
    parts = base.split("_")
    suffix_os = suffix_arch = None
    if len(parts) >= 3 and parts[-2] in KNOWN_OS and parts[-1] in KNOWN_ARCH:
        suffix_os, suffix_arch = parts[-2], parts[-1]
    elif len(parts) >= 2 and parts[-1] in KNOWN_OS:
        suffix_os = parts[-1]
    elif len(parts) >= 2 and parts[-1] in KNOWN_ARCH:
        suffix_arch = parts[-1]
    m = re.search(r"^//go:build (.+)$", head, re.M)
    for goos, goarch in TARGETS:
        if suffix_os and suffix_os != goos:
            continue
        if suffix_arch and suffix_arch != goarch:
            continue
        if m and not eval_constraint(m.group(1).strip(), goos, goarch):
            continue
        return True
    return False


def load_existing():
    keep = {}
    if not os.path.exists(OUT):
        return keep
    cur = None
    for line in open(OUT, encoding="utf-8"):
        line = line.strip()
        if line == "[[file]]":
            cur = {}
        elif cur is not None and "=" in line:
            k, v = line.split("=", 1)
            cur[k.strip()] = v.strip().strip('"')
            if k.strip() == "go":
                keep[cur["go"]] = cur
    return keep


existing = load_existing()
entries = []
for dirpath, dirnames, filenames in os.walk(ROOT):
    rel_dir = os.path.relpath(dirpath, ROOT).replace(os.sep, "/")
    dirnames[:] = [d for d in dirnames if d != "testdata" and not d.startswith("_") and not d.startswith(".")
                   and not (rel_dir == "internal/fourslash" and d == "tests")]
    if not (rel_dir.startswith("internal") or rel_dir.startswith("cmd")):
        continue
    for fn in sorted(filenames):
        if not fn.endswith(".go") or fn.endswith("_test.go"):
            continue
        path = os.path.join(dirpath, fn)
        rel = "tsc/" + rel_dir + "/" + fn
        with open(path, encoding="utf-8", errors="replace") as f:
            text = f.read()
        head = "\n".join(text.splitlines()[:15])
        loc = text.count("\n")
        pkg = rel_dir
        krate, phase = crate_for(pkg)
        if any(pkg == p or pkg.startswith(p + "/") for p in HARNESS_PREFIXES):
            kind = "harness"
        elif fn.endswith("_generated.go") or "Code generated" in head:
            kind = "generated"
        else:
            kind = "source"
        if not in_scope(path, head):
            kind = "out-of-scope"
        prev = existing.get(rel, {})
        status = prev.get("status", "out-of-scope" if kind == "out-of-scope" else "planned")
        entries.append({"go": rel, "package": pkg, "crate": krate, "phase": phase, "kind": kind,
                        "status": status, "rust": prev.get("rust", ""), "pin": prev.get("pin", PIN), "loc": loc})

entries.sort(key=lambda e: e["go"])
with open(OUT, "w", encoding="utf-8") as out:
    out.write("# Port ledger: one entry per non-test Go file of the pinned upstream tsc module.\n")
    out.write("# Regenerate with scripts/ledger-init.py; statuses and rust paths are preserved.\n")
    out.write("# status: planned | in-progress | ported | verified | out-of-scope\n")
    out.write("# kind: source | generated | harness | out-of-scope\n")
    out.write(f'pin = "{PIN}"\n\n')
    for e in entries:
        out.write("[[file]]\n")
        for k in ("go", "package", "crate", "phase", "kind", "status", "rust", "pin", "loc"):
            v = e[k]
            out.write(f"{k} = {v}\n" if isinstance(v, int) else f'{k} = "{v}"\n')
        out.write("\n")

from collections import Counter
kinds = Counter(e["kind"] for e in entries)
print(f"{len(entries)} entries -> {OUT}")
print("by kind:", dict(kinds))
print("in-scope LOC:", sum(e["loc"] for e in entries if e["kind"] != "out-of-scope"))
