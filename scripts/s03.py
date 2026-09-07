#!/usr/bin/env python3
"""Run pinned S03 frontends in an isolated, disposable tooling worktree.

Canonical upstream/ is read-only. Rust emission and checked-in drift comparison
live in xtask; this adapter exports source facts and compares client bytes.
"""

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from s04_common import command, strict_json_loads
from s04_runtime import load_toolchains


ROOT = Path(__file__).resolve().parents[1]
CLIENT = "packages/typescript/src"


def read_json(path):
    return strict_json_loads(path.read_bytes())


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n")


def run(args, cwd, env=None):
    return command(list(map(str, args)), cwd=cwd, env=env)


def version_environment(root, tooling):
    package = read_json(root / "upstream/package.json")
    node_pin, npm_pin = package["volta"]["node"], package["volta"]["npm"]
    if package["packageManager"].split("+", 1)[0] != f"npm@{npm_pin}":
        raise ValueError("upstream packageManager and volta npm pins disagree")
    env = os.environ.copy()
    # A local bootstrap is optional; otherwise use the caller's pinned runtime.
    local = root / "target/s03-runtime/node_modules"
    if (local / "node/bin/node").exists():
        env["PATH"] = os.pathsep.join([str(local / "node/bin"), str(local / ".bin"), env.get("PATH", "")])
    for key in ("NODE_OPTIONS", "NODE_PATH", "GOOS", "GOARCH", "GOTOOLCHAIN", "GOFLAGS", "GOWORK"):
        env.pop(key, None)
    env.update(GOTOOLCHAIN="local", GOFLAGS="-mod=readonly -trimpath", GOWORK=str(tooling / "go.work"))
    for tool, expected, args in (("node", f"v{node_pin}", ["node", "--version"]),
                                 ("npm", npm_pin, ["npm", "--version"])):
        found = run(args, root, env).decode().strip()
        if found != expected:
            raise ValueError(f"S03 requires {tool} {expected} from upstream/package.json; found {found}")
    # Read the shared Go pin, keeping GOTOOLCHAIN=local during both check and build.
    expected_go = load_toolchains(root)["go"]
    actual_go = run(["go", "env", "GOVERSION"], root, env).decode().strip()
    if actual_go != expected_go:
        raise ValueError(f"S03 requires {expected_go}; found {actual_go}")
    print(f"S03 runtimes: node {node_pin}, npm {npm_pin}, {actual_go}", file=sys.stderr)
    return env


def check_upstream(root, pin):
    if len(pin) != 40 or any(ch not in "0123456789abcdef" for ch in pin):
        raise ValueError("invalid upstream pin")
    upstream = root / "upstream"
    if not (upstream / ".git").exists():
        raise ValueError("initialize canonical upstream first: git submodule update --init upstream")
    actual = run(["git", "rev-parse", "HEAD"], upstream).decode().strip()
    gitlink = run(["git", "ls-tree", "HEAD", "upstream"], root).decode().split()
    if actual != pin or gitlink[:3] != ["160000", "commit", pin]:
        raise ValueError("canonical upstream HEAD, parent gitlink and ledger pin must agree")
    if run(["git", "status", "--porcelain", "--untracked-files=all"], upstream).strip():
        raise ValueError("canonical upstream must be clean; tooling edits belong only in .s03-tooling/pin")


def prepare_worktree(root, pin):
    # rust-cache prunes arbitrary files from Cargo's target directory. Git
    # metadata and a reusable checkout must stay outside that managed tree.
    # Old target/s03-tooling* remnants are deliberately neither read nor deleted.
    state = root / ".s03-tooling"
    bare = state / "upstream.git"
    tooling = state / "pin"
    for path in (state, bare, tooling):
        if path.is_symlink():
            raise ValueError(f"refusing symlinked tooling path: {path}")
    state.mkdir(exist_ok=True)
    if not bare.exists():
        run(["git", "clone", "--bare", "--shared", root / "upstream", bare], root)
    # Verify ownership before resetting anything, including after a previous run.
    try:
        origin = run(["git", "--git-dir", bare, "config", "--get", "remote.origin.url"], root).decode().strip()
    except RuntimeError as error:
        raise ValueError(f"refusing incomplete tooling repository: {bare}; inspect its Git metadata before retrying") from error
    if Path(origin).resolve() != (root / "upstream").resolve():
        raise ValueError(f"refusing unrelated tooling repository: {bare}")
    if not tooling.exists():
        tooling.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "--git-dir", bare, "worktree", "add", "--detach", "--no-checkout", tooling, pin], root)
    common = run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], tooling).decode().strip()
    if Path(common).resolve() != bare.resolve():
        raise ValueError(f"refusing unrelated worktree: {tooling}")
    run(["git", "sparse-checkout", "set", "tools", "packages", "tsc/internal", "tsc/s03export"], tooling)
    run(["git", "reset", "--hard", pin], tooling)
    run(["git", "clean", "-fd"], tooling)  # Preserve ignored node_modules and its cache stamp.
    patches = sorted((root / "tools/s03/patches").glob("*.patch"))
    if not patches:
        raise ValueError("no carried S03 patches")
    applied = 0
    for patch in patches:
        run(["git", "apply", "--check", patch], tooling)
        run(["git", "apply", patch], tooling)
        applied += 1
    return tooling, applied == len(patches)


def dependencies(root, tooling, env):
    # npm itself validates integrity against the upstream lockfile. Cache the
    # installed closure in this worktree, preserving caller npm registry/cache.
    package = (tooling / "package.json").read_bytes()
    lock = (tooling / "package-lock.json").read_bytes()
    fingerprint = hashlib.sha256(package + b"\0" + lock).hexdigest()
    stamp = tooling / "node_modules/.s03-lock-sha256"
    if not stamp.exists() or stamp.read_text() != fingerprint:
        run(["npm", "ci", "--ignore-scripts", "--no-audit", "--no-fund"], tooling, env)
        stamp.write_text(fingerprint)
    # Exercise the actual pinned formatter, not npx's network fallback.
    run([tooling / "node_modules/.bin/dprint", "--version"], tooling, env)


def copy_adapter(root, tooling, source, destination):
    target = tooling / destination
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(root / source, target)


def client_inventory(root, pin):
    files = run(["git", "ls-tree", "-r", "--name-only", pin, f"{CLIENT}/enums"], root / "upstream").decode().splitlines()
    files = [path for path in files if path.endswith(".ts")]
    if not files:
        raise ValueError("empty pinned enum inventory")
    return [f"{CLIENT}/api/proto.generated.ts", *files]


def compare_client(root, tooling, pin, expected, control):
    actual_enums = {path.relative_to(tooling).as_posix() for path in (tooling / CLIENT / "enums").rglob("*.ts")}
    if actual_enums != set(expected[1:]):
        print("S03 client enum inventory differs from the untouched pin", file=sys.stderr)
        return False, []
    identical, hashes = True, []
    for path in expected:
        original = run(["git", "show", f"{pin}:{path}"], root / "upstream")
        regenerated = (tooling / path).read_bytes()
        matches = regenerated == original
        if not matches:
            print(f"S03 client mismatch: {path}", file=sys.stderr)
        identical = identical and matches
        hashes.append({"path": path, "sha256": hashlib.sha256(regenerated).hexdigest()})
    if control.read_bytes() != (tooling / expected[0]).read_bytes():
        print("S03 JSON client renderer differs from the pinned Go generator", file=sys.stderr)
        identical = False
    return identical, hashes


def handwritten_inputs(root, expected):
    inventory = read_json(root / "data/s03/handwritten-enums.json")
    if inventory["version"] != 1 or not inventory["enums"]:
        raise ValueError("invalid handwritten enum inventory")
    declarations = {f"{CLIENT}/enums/{entry['file']}" for entry in inventory["enums"]}
    if len(declarations) != len(inventory["enums"]):
        raise ValueError("duplicate handwritten enum input")
    for path in declarations:
        if Path(path).parent.as_posix() != f"{CLIENT}/enums" or not path.endswith(".enum.ts"):
            raise ValueError(f"invalid handwritten enum input: {path}")
    paths = declarations | {path.replace(".enum.ts", ".ts") for path in declarations}
    if not paths <= set(expected[1:]):
        raise ValueError("handwritten enum inventory does not match the pin")
    for path in paths:
        source = (root / "upstream" / path).read_bytes()
        if b"Code generated by Herebyfile.mjs" in source:
            raise ValueError(f"generated enum cannot be declared a handwritten input: {path}")
    return paths


def clear_enum_outputs(tooling, expected, inputs=frozenset()):
    # A restored original must never mask an output omitted by the generator.
    for path in expected[1:]:
        relative = Path(path)
        if relative.parent.as_posix() != f"{CLIENT}/enums" or relative.suffix != ".ts":
            raise ValueError(f"unexpected pinned enum output: {path}")
        if path not in inputs:
            (tooling / relative).unlink()


def validate_codecs(root, api, wire):
    inventory = read_json(root / "data/s03/api-special-codecs.json")
    mappings = inventory["mappings"]
    actual = {item["id"] for item in api["specialMappings"]}
    declared = {item["goType"] for item in mappings}
    if (not actual or len(actual) != len(api["specialMappings"])
            or len(declared) != len(mappings) or declared != actual):
        raise ValueError("special codec inventory does not exactly cover the pinned extractor")
    fixtures = wire["fixtures"]
    ids = {item["id"] for item in fixtures}
    if len(ids) != len(fixtures) or not ids:
        raise ValueError("empty or duplicate native wire fixture IDs")
    for item in mappings:
        if not item["fixtureIds"] or not set(item["fixtureIds"]) <= ids:
            raise ValueError(f"missing source wire fixture for {item['goType']}")
    ordinary = inventory["ordinaryFieldFixtures"]
    if not ordinary or len(set(ordinary)) != len(ordinary) or not set(ordinary) <= ids:
        raise ValueError("missing ordinary field wire fixture")


def export_ast(root, tooling, output, env):
    """Measure the pinned resolver separately from later generation stages.

    A resolver that executes but fails or produces no normalized tables is an
    observed schema failure. Missing executables, inaccessible files and invalid
    JSON remain capture errors; they are not evidence about the resolver.
    """
    try:
        run(["node", root / "tools/s03/ast-export.mts", tooling, output], tooling, env)
    except RuntimeError as error:
        print(f"S03 AST schema export failed: {error}", file=sys.stderr)
        return False
    try:
        data = output.read_bytes()
    except FileNotFoundError:
        print("S03 AST schema export failed: resolver produced no ast.json", file=sys.stderr)
        return False
    if not data.strip():
        print("S03 AST schema export failed: resolver produced an empty ast.json", file=sys.stderr)
        return False
    schema = strict_json_loads(data)
    if not isinstance(schema, dict) or type(schema.get("version")) is not int or schema["version"] != 1:
        print("S03 AST schema export failed: unsupported normalized schema version", file=sys.stderr)
        return False
    for name in ("kinds", "markers", "kindAliases", "bases", "nodes"):
        if not isinstance(schema.get(name), list) or not schema[name]:
            print(f"S03 AST schema export failed: missing or empty normalized {name} table", file=sys.stderr)
            return False
    return True


def prepare(root, stage, pin):
    stage.mkdir(parents=True, exist_ok=True)
    if any(stage.iterdir()):
        raise ValueError("S03 export staging directory must be empty; stale exports cannot prove regeneration")
    check_upstream(root, pin)
    tooling, patches_apply = prepare_worktree(root, pin)
    env = version_environment(root, tooling)
    dependencies(root, tooling, env)
    if not export_ast(root, tooling, stage / "ast.json", env):
        return {"patches_apply": patches_apply, "ast_schema": False}
    copy_adapter(root, tooling, "tools/s03/encoder-export.mts", "tools/scripts/tsc/s03-encoder-export.mts")
    run(["node", "tools/scripts/tsc/s03-encoder-export.mts", stage / "encoder-nodes.json"], tooling, env)
    copy_adapter(root, tooling, "tools/s03/diagnostics-export_test.go", "tsc/internal/diagnostics/s03_export_test.go")
    diag_env = dict(env, S03_DIAGNOSTICS_OUTPUT=str(stage / "diagnostics.json"))
    run(["go", "test", "generate.go", "s03_export_test.go", "-run", "^TestS03ExportDiagnostics$", "-count=1"], tooling / "tsc/internal/diagnostics", diag_env)
    copy_adapter(root, tooling, "tools/s03/encoder-constants.go", "tsc/s03export/main.go")
    run(["go", "run", "./tsc/s03export", stage / "encoder-constants.json"], tooling, env)
    nodes, constants = read_json(stage / "encoder-nodes.json"), read_json(stage / "encoder-constants.json")
    write_json(stage / "encoder.json", {"version": 1, "nodes": nodes["nodes"], "constants": constants["constants"]})
    control = tooling / CLIENT / "api/proto.s03-control.ts"
    run(["go", "-C", "tools", "run", "./gen-proto", "../tsc/internal/api/proto.go", control, stage / "api.json"], tooling, env)
    run(["go", "-C", "tools", "test", "./gen-proto"], tooling, env)
    run([sys.executable, root / "tools/s03/proto/render_typescript.py", stage / "api.json", tooling / CLIENT / "api/proto.generated.ts"], tooling, env)
    run([sys.executable, "-m", "unittest", "discover", "-s", str(root / "tools/s03/proto"), "-q"], root, env)
    run([tooling / "node_modules/.bin/dprint", "fmt", control, tooling / CLIENT / "api/proto.generated.ts"], tooling, env)
    # Invoke the pinned enum resolver/emitter, including its independent Go value check.
    expected = client_inventory(root, pin)
    enum_inputs = handwritten_inputs(root, expected)
    clear_enum_outputs(tooling, expected, enum_inputs)
    run([tooling / "node_modules/.bin/hereby", "generate:enums"], tooling, env)
    client_identical, client_hashes = compare_client(root, tooling, pin, expected, control)
    for item in client_hashes:
        item["role"] = "source" if item["path"] in enum_inputs else "generated"
    write_json(stage / "client-files.json", {"version": 1, "upstreamPin": pin, "files": client_hashes})
    copy_adapter(root, tooling, "tools/s03/proto/wire_fixtures.go", "tsc/internal/s03-wire-fixtures/main.go")
    copy_adapter(root, tooling, "tools/s03/proto/wire_fixtures_test.go", "tsc/internal/s03-wire-fixtures/main_test.go")
    run(["go", "-C", "tsc", "test", "./internal/s03-wire-fixtures"], tooling, env)
    wire = strict_json_loads(run(["go", "-C", "tsc", "run", "./internal/s03-wire-fixtures"], tooling, env))
    write_json(stage / "api-wire-fixtures.json", wire)
    api = read_json(stage / "api.json")
    validate_codecs(root, api, wire)
    # Decode every export strictly here before serde consumes normalized snapshots.
    for name in ("ast", "diagnostics", "encoder", "api"):
        write_json(stage / f"{name}.json", read_json(stage / f"{name}.json"))
    return {"patches_apply": patches_apply, "ast_schema": True, "client_identical": client_identical,
            "client_files": len(expected), "api_methods": len(api["methods"]),
            "client_generated_files": len(expected) - len(enum_inputs),
            "client_source_files": len(enum_inputs),
            "api_types": len(api["types"]), "api_special_codecs": len(api["specialMappings"]),
            "api_wire_fixtures": len(wire["fixtures"]),
            "ast_nodes": len(read_json(stage / "ast.json")["nodes"]),
            "diagnostics": len(read_json(stage / "diagnostics.json")["messages"]),
            "encoder_nodes": len(nodes["nodes"])}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("prepare",))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--pin", required=True)
    args = parser.parse_args()
    try:
        (ROOT / "target").mkdir(exist_ok=True)
        with (ROOT / "target/s03-tooling.lock").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            summary = prepare(ROOT, args.output.resolve(), args.pin)
        print(json.dumps(summary, sort_keys=True))
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"S03 generation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
