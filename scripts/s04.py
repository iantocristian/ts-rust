#!/usr/bin/env python3
"""Execute the S04 leaf contracts. JSON stdout is reserved for runner evidence.

The Go oracle is rebuilt from a clean exported pin plus small access wrappers in
scripts/s04_oracle. The upstream submodule itself is never modified. E4 compares
every frozen probe, including prescribed panic messages, then aggregates only the
five implemented criteria. E3 covers arena contracts only.
"""

import argparse
from collections import Counter, defaultdict
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import random
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parent.parent
TEXT_CRITERIA = (
    "source_decoding", "helper_semantics", "slice_validity", "utf8_positions", "utf16_positions"
)


def strict_json_loads(data):
    def reject_constant(value):
        raise ValueError(f"non-finite JSON number {value}")

    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON object key {key}")
            result[key] = value
        return result

    def finite_float(text):
        value = float(text)
        if not math.isfinite(value):
            raise ValueError(f"non-finite JSON number {text}")
        return value

    return json.loads(data, parse_constant=reject_constant, parse_float=finite_float,
                      object_pairs_hook=unique_object)


def validate_json_value(value):
    kind = type(value)
    if value is None or kind in (str, bool, int):
        return
    if kind is float:
        if not math.isfinite(value):
            raise ValueError("non-finite result value")
        return
    if kind is list:
        for item in value:
            validate_json_value(item)
        return
    if kind is dict and all(type(key) is str for key in value):
        for item in value.values():
            validate_json_value(item)
        return
    raise ValueError("result contains a value outside the JSON data model")


def same_json_value(expected, actual):
    """JSON booleans, integers and floating-point numbers are distinct outcomes."""
    if type(expected) is not type(actual):
        return False
    if type(expected) is list:
        return len(expected) == len(actual) and all(
            same_json_value(left, right) for left, right in zip(expected, actual)
        )
    if type(expected) is dict:
        return expected.keys() == actual.keys() and all(
            same_json_value(expected[key], actual[key]) for key in expected
        )
    return expected == actual


def probe_inventory(probes):
    """Freeze both scenario coverage and every request, including probe arguments."""
    if not probes or len({probe["id"] for probe in probes}) != len(probes):
        raise ValueError("empty or duplicate probe inventory")
    payload = json.dumps(probes, sort_keys=True, separators=(",", ":")).encode()
    return {
        "schema_version": 1,
        "probes": len(probes),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "scenario_probes": dict(sorted(Counter(probe["group"] for probe in probes).items())),
    }


def validate_probe_inventory(probes, frozen):
    if not same_json_value(probe_inventory(probes), frozen):
        raise ValueError("E4 probe inventory drift; review and regenerate with --write-manifest")


def command(args, *, cwd=ROOT, env=None, data=None):
    print("+ " + " ".join(map(str, args)), file=sys.stderr)
    result = subprocess.run(args, cwd=cwd, env=env, input=data, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=False)
    if result.stderr:
        sys.stderr.buffer.write(result.stderr)
        sys.stderr.flush()
    if result.returncode:
        if result.stdout:
            sys.stderr.buffer.write(result.stdout)
        raise RuntimeError(f"command exited {result.returncode}: {args}")
    return result.stdout


def verified_upstream():
    spec = importlib.util.spec_from_file_location("s04_bootstrap", ROOT / "scripts/tracking-bootstrap.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.verify_upstream(ROOT)


def go_environment():
    env = os.environ.copy()
    for key in ("GOOS", "GOARCH", "GOARM", "GOARM64", "GOAMD64", "GO386", "GOMIPS", "GOMIPS64", "GOPPC64", "GORISCV64"):
        env.pop(key, None)
    env.update(CGO_ENABLED="0", GOWORK="off", GOFLAGS="", GOCACHE=str(ROOT / "target/go-build"))
    wanted = tomllib.loads((ROOT / "data/s04/toolchains.toml").read_text())["go"]
    version = command(["go", "version"], env=env).decode().strip()
    if version.split()[2] != wanted:
        raise ValueError(f"S04 oracle requires {wanted}; got {version}")
    print(version, file=sys.stderr)
    return env


def go_oracle(upstream, env):
    """Build in a fresh export so a modified cached oracle cannot become evidence."""
    pin = json.loads((ROOT / "data/upstream.json").read_text())["pin"]
    destination = ROOT / "target/s04-oracle"
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="export-", dir=destination) as temporary:
        checkout = Path(temporary)
        archive = subprocess.Popen(["git", "archive", pin, "tsc"], cwd=upstream, stdout=subprocess.PIPE)
        try:
            with tarfile.open(fileobj=archive.stdout, mode="r|") as stream:
                stream.extractall(checkout, filter="data")
        finally:
            archive.stdout.close()
        if archive.wait():
            raise RuntimeError("cannot export pinned Go source")
        bridges = {
            "decode_bridge.go": "internal/vfs/internal/s04_bridge.go",
            "printer_bridge.go": "internal/printer/s04_bridge.go",
            "lsp_bridge.go": "internal/ls/lsconv/s04_bridge.go",
            "main.go": "internal/vfs/s04oracle/main.go",
            "main_test.go": "internal/vfs/s04oracle/main_test.go",
        }
        for source, target in bridges.items():
            target = checkout / "tsc" / target
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / "scripts/s04_oracle" / source, target)
        executable = destination / "oracle"
        sys.stderr.buffer.write(command(["go", "test", "-mod=readonly", "./internal/vfs/s04oracle"],
                                        cwd=checkout / "tsc", env=env))
        command(["go", "build", "-mod=readonly", "-o", str(executable), "./internal/vfs/s04oracle"],
                cwd=checkout / "tsc", env=env)
    print(f"Go oracle executable sha256={hashlib.sha256(executable.read_bytes()).hexdigest()}", file=sys.stderr)
    return executable


def sigma_contexts(table):
    """Sample property-range edges, stride members and holes in sigma contexts."""
    points = set()
    for name in ("unicodeCasedRanges", "unicodeCaseIgnorableRanges"):
        block = re.search(r"var " + name + r" = &unicode\.RangeTable\{(.*?)\n\}", table, re.DOTALL)
        if not block:
            raise ValueError(f"missing pinned property table {name}")
        rows = re.findall(r"\{(0x[0-9a-fA-F]+),\s*(0x[0-9a-fA-F]+),\s*(\d+)\}", block[1])
        if not rows:
            raise ValueError(f"empty pinned property table {name}")
        for start, end, step in rows:
            lo, hi, stride = int(start, 16), int(end, 16), int(step)
            if lo > hi or stride <= 0 or (hi - lo) % stride:
                raise ValueError(f"invalid pinned property range in {name}")
            points.update((lo - 1, lo, lo + 1, lo + stride, hi - stride, hi - 1, hi, hi + 1))
    for code in sorted(points):
        if 0 <= code <= 0x10FFFF and not 0xD800 <= code <= 0xDFFF:
            char = chr(code)
            for context in ("AΣ" + char, char + "Σ", "A" + char + "Σ", "AΣ" + char + "B"):
                yield context.encode()


def text_probes(upstream):
    corpus = json.loads((ROOT / "data/s04/corpus.json").read_text())
    if corpus["schema_version"] != 1:
        raise ValueError("unknown S04 corpus version")
    texts = [(item["id"], bytes.fromhex(item["hex"])) for item in corpus["texts"]]
    rng = random.Random(corpus["random_seed"])
    texts.extend((f"random-{i:02}", rng.randbytes(rng.randrange(corpus["random_max_bytes"] + 1)))
                 for i in range(corpus["random_cases"]))
    probes = []
    counts = Counter()

    def add(group, criterion, op, text=b"", a=0, b=0, flag=False, panic_message=False):
        counts[group] += 1
        probe = {"id": f"{group}/{counts[group]:05}", "group": group, "criterion": criterion,
                 "op": op, "text": text.hex(), "a": a, "b": b, "flag": flag}
        if panic_message:
            probe["panic_message"] = True
        probes.append(probe)

    for name, text in texts:
        add(f"source/{name}", "source_decoding", "source", text)
        for start in range(-1, len(text) + 2):
            for end in range(-1, len(text) + 2):
                add(f"slice/{name}", "slice_validity", "slice", text, start, end)
        # A UTF-16 code unit can become three bytes after replacement. This is a
        # coverage bound only; the expected decoded bytes come from the Go oracle.
        source_bound = len(text)
        if text.startswith((b"\xff\xfe", b"\xfe\xff")):
            source_bound = max(source_bound, ((len(text) - 2) // 2) * 3)
        for start in range(-1, source_bound + 2):
            for end in range(-1, source_bound + 2):
                add(f"source-slice/{name}", "slice_validity", "source_slice", text, start, end)
        for op in ("lower", "upper", "lower_first", "combine", "decode_js", "decode_utf8"):
            add(f"helpers/{name}", "helper_semantics", op, text)
        for length in range(-2, len(text) + 3):
            add(f"helpers/{name}", "helper_semantics", "truncate", text, length)
        for quote in (34, 39, 96):
            for flags in range(4):
                add(f"helpers/{name}", "helper_semantics", "escape", text, quote, flags)
        add(f"utf8/{name}", "utf8_positions", "ecma_lines", text)
        add(f"utf8/{name}", "utf8_positions", "lsp_lines", text)
        add(f"utf16/{name}", "utf16_positions", "api_ascii", text)
        add(f"utf16/{name}", "utf16_positions", "utf16_len", text)
        for offset in [*range(-2, len(text) + 3), -(2**31), 2**31 - 1, -(2**63), 2**63 - 1]:
            for op in ("api_to_utf16", "api_to_utf8", "scanner_from_position"):
                add(f"utf16/{name}", "utf16_positions", op, text, offset)
            for op in ("scanner_line", "byte_from_position", "lsp_line_index"):
                add(f"utf8/{name}", "utf8_positions", op, text, offset)
            for flag, criterion in ((False, "utf16_positions"), (True, "utf8_positions")):
                group = ("utf8/" if flag else "utf16/") + name
                add(group, criterion, "lsp_from_position", text, offset, flag=flag)
        # Includes signed Go offsets and u32 wire values whose TextPos conversion wraps.
        lines = [-2, -1, 0, 1, 2, 3, len(text) + 1, 2**31, 2**32 - 1]
        characters = [-2, -1, 0, 1, 2, 3, len(text), len(text) + 2, 2**31 - 1, 2**32 - 1]
        for line in lines:
            add(f"utf8/{name}", "utf8_positions", "scanner_end_line", text, line)
            for character in characters:
                add(f"utf8/{name}", "utf8_positions", "scanner_byte_position", text, line, character)
                for flag in (False, True):
                    add(f"utf16/{name}", "utf16_positions", "scanner_to_position", text, line, character, flag)
                    criterion = "utf8_positions" if flag else "utf16_positions"
                    group = ("utf8/" if flag else "utf16/") + name
                    add(group, criterion, "lsp_to_position", text, line, character, flag)
    table = (upstream / "tsc/internal/stringutil/js_case_generated.go").read_text()
    for code in sorted(set(int(value, 16) for value in re.findall(r"^\s*(0x[0-9a-fA-F]+):", table, re.MULTILINE))):
        text = chr(code).encode()
        for op in ("lower", "upper", "lower_first"):
            add("helpers/case-table", "helper_semantics", op, text)
    for text in sigma_contexts(table):
        add("helpers/sigma-ranges", "helper_semantics", "lower", text)
    for byte in range(256):
        for op in ("lower", "upper", "lower_first", "decode_js", "decode_utf8", "combine"):
            add("helpers/all-bytes", "helper_semantics", op, bytes([byte]))
    for code in [-1, 0, 1, 0x7F, 0x80, 0x7FF, 0x800, 0xD7FF, *range(0xD800, 0xE000), 0xE000, 0xFFFF, 0x10000, 0x10FFFF, 0x110000, 2**31 - 1]:
        add("helpers/rune-encoding", "helper_semantics", "encode", a=code)
    # Explicit contract panics have stable payloads; runtime bounds panics do not.
    for line in [-1, 2, -(2**63), 2**63 - 1]:
        add("utf16/bad-line-message", "utf16_positions", "scanner_to_position",
            b"a\nb", a=line, panic_message=True)
    return probes


def compare(probes, oracle, rust):
    """Require complete, ordered, unique results; aggregate real comparisons only."""
    if type(probes) is not list or any(
        type(probe) is not dict
        or any(type(probe.get(key)) is not str or not probe[key]
               for key in ("id", "group", "criterion"))
        for probe in probes
    ):
        raise ValueError("invalid probe inventory")
    ids = [probe["id"] for probe in probes]
    if not ids or len(set(ids)) != len(ids):
        raise ValueError("empty or duplicate probe inventory")
    for label, results in (("oracle", oracle), ("rust", rust)):
        if type(results) is not list or any(type(result) is not dict for result in results):
            raise ValueError(f"invalid {label} result inventory")
        if [r.get("id") for r in results] != ids:
            raise ValueError(f"{label} returned incomplete, reordered or unknown probes")
        for probe, result in zip(probes, results):
            if set(result) != {"id", "panic", "value"} or type(result["panic"]) is not bool:
                raise ValueError(f"invalid {label} result shape")
            if result["panic"]:
                if probe.get("panic_message", False):
                    if type(result["value"]) is not str or not result["value"]:
                        raise ValueError(f"invalid {label} panic message")
                elif result["value"] is not None:
                    raise ValueError(f"invalid {label} panic result")
            validate_json_value(result["value"])
    groups = {}
    failures = []
    for probe, expected, actual in zip(probes, oracle, rust):
        group = probe["group"]
        criterion = probe["criterion"]
        if criterion not in TEXT_CRITERIA:
            raise ValueError(f"unsupported text criterion {criterion}")
        if group in groups and groups[group]["criterion"] != criterion:
            raise ValueError("a scenario cannot mix criteria")
        entry = groups.setdefault(group, {"criterion": criterion, "pass": True, "probes": 0})
        entry["probes"] += 1
        if not same_json_value(expected, actual):
            entry["pass"] = False
            failures.append({"probe": probe, "expected": expected, "actual": actual})
    metrics = {}
    for criterion in TEXT_CRITERIA:
        selected = [g for g in groups.values() if g["criterion"] == criterion]
        metrics[criterion] = bool(selected) and all(g["pass"] for g in selected)
        metrics[f"{criterion}_probes"] = sum(g["probes"] for g in selected)
    return {"metrics": metrics, "tests": {group: "pass" if values["pass"] else "fail"
                                          for group, values in sorted(groups.items())}}, failures


def e4():
    upstream = verified_upstream()
    probes = text_probes(upstream)
    manifest = json.loads((ROOT / "data/s04/e4-cases.json").read_text())
    if manifest != sorted({probe["group"] for probe in probes}):
        raise ValueError("E4 case manifest drift; review and regenerate with --write-manifest")
    validate_probe_inventory(probes, strict_json_loads((ROOT / "data/s04/e4-probes.json").read_text()))
    payload = json.dumps(probes, separators=(",", ":")).encode()
    env = go_environment()
    command([sys.executable, str(ROOT / "crates/ts_jsstring/tools/generate_case_tables.py"), "--check"], env=env)
    expected_bytes = command([str(go_oracle(upstream, env))], data=payload, env=env)
    sys.stderr.buffer.write(command(["cargo", "test", "--package", "ts_jsstring", "--all-targets", "--locked"]))
    # Let Cargo select and run the artifact it just built, including when the
    # caller configured CARGO_TARGET_DIR, CARGO_BUILD_TARGET or build.target-dir.
    actual_bytes = command(["cargo", "run", "--package", "ts_jsstring", "--example", "e4", "--release", "--locked"], data=payload)
    expected, actual = strict_json_loads(expected_bytes), strict_json_loads(actual_bytes)
    report, failures = compare(probes, expected, actual)
    report["metrics"]["probes"] = len(probes)
    report["metrics"]["failed_probes"] = len(failures)
    for label, results in (("oracle", expected), ("rust", actual)):
        digest = hashlib.sha256(json.dumps(results, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        print(f"{label}: {len(results)} probes; output sha256={digest}", file=sys.stderr)
    if failures:
        target = ROOT / "target/s04-e4-failures.json"
        target.write_text(json.dumps(failures, indent=2) + "\n")
        print(f"{len(failures)} mismatches; first failures: {json.dumps(failures[:8])}; full report: {target}", file=sys.stderr)
    verified_upstream()
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("e3", "e4"))
    parser.add_argument("--write-manifest", action="store_true")
    args = parser.parse_args()
    try:
        if args.write_manifest:
            if args.mode != "e4":
                raise ValueError("only E4 has a generated probe manifest")
            probes = text_probes(verified_upstream())
            (ROOT / "data/s04/e4-cases.json").write_text(json.dumps(sorted({p["group"] for p in probes}), indent=2) + "\n")
            (ROOT / "data/s04/e4-probes.json").write_text(json.dumps(probe_inventory(probes), indent=2) + "\n")
            print(f"Frozen {len(probes)} probes in {len({p['group'] for p in probes})} scenarios.", file=sys.stderr)
            return 0
        if args.mode == "e3":
            from s04_ownership import run
            print(json.dumps(run(ROOT), sort_keys=True))
            return 0
        print(json.dumps(e4(), sort_keys=True))
        return 0
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"S04 producer failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
