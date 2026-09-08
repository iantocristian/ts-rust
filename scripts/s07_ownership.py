"""Executed binder and program ownership groups, separate from full future E3."""
from pathlib import Path
import re
import sys

from s04_common import strict_json_loads
from s06_ownership import validate_output

GROUPS = ("shared_bound_file", "retained_snapshot_edit")


def load_cases(root):
    manifest = strict_json_loads((Path(root) / "data/s07/ownership-cases.json").read_bytes())
    if set(manifest) != {"version", "common", "groups"} or manifest["version"] != 1 or set(manifest["groups"]) != set(GROUPS):
        raise ValueError("invalid S07 ownership inventory")
    suites = [manifest["common"], *manifest["groups"].values()]
    for suite in suites:
        if (set(suite) != {"package", "filter", "exact", "cases"}
                or suite["package"] not in {"ts_ast", "ts_compiler"}
                or type(suite["exact"]) is not bool or not suite["cases"]
                or suite["cases"] != sorted(set(suite["cases"]))
                or any(not isinstance(name, str) or not re.fullmatch(r"[a-z0-9_]+::[a-z0-9_]+", name) for name in suite["cases"])
                or any(not name.startswith(suite["filter"]) for name in suite["cases"])
                or suite["exact"] and suite["cases"] != [suite["filter"]]):
            raise ValueError("invalid exact S07 ownership suite")
    return manifest


def measure(root, invoke, prefix, options, env, manifest, mode):
    def execute(name, suite):
        tail = ["--", "--test-threads=1", "--nocapture"]
        if suite["exact"]:
            tail.append("--exact")
        args = [*prefix, "test", "--package", suite["package"], "--lib", "--locked", *options, suite["filter"], *tail]
        try:
            validate_output(invoke(root, args, env), suite["cases"], mode, f"S07 ownership {name}")
        except (RuntimeError, ValueError) as error:
            print(f"S07 ownership {name} {mode} failed: {error}", file=sys.stderr)
            return False
        return True
    common = execute("binding publication", manifest["common"])
    outcomes = {name: execute(name, manifest["groups"][name]) for name in GROUPS}
    return {name: common and outcomes[name] for name in GROUPS}


def publish_metrics(report, modes, manifest):
    required = {"debug", "release", "miri", "address_sanitizer"}
    if set(modes) != required or any(set(outcomes) != set(GROUPS) for outcomes in modes.values()):
        raise ValueError("S07 ownership must execute every group in all four modes")
    for name in GROUPS:
        for mode, outcomes in modes.items():
            report["metrics"][f"{name}_{mode}"] = outcomes[name]
        report["metrics"][name] = all(outcomes[name] for outcomes in modes.values())
    report["metrics"]["program_ownership_tests"] = len(manifest["common"]["cases"]) + sum(len(group["cases"]) for group in manifest["groups"].values())
