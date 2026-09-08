"""Executed binder and program ownership groups, separate from full future E3."""
from pathlib import Path
import re
import sys

from s04_common import strict_json_loads
from s06_ownership import validate_output

GROUPS = ("shared_bound_file", "retained_snapshot_edit")
COMMON = {
    "binding_publication": ("ts_ast", "bind_tests::"),
    "exclusive_binding": ("ts_binder", "exclusive_tests::"),
    "core_validation_proof": ("ts_ast", "storage::validation_proof_tests::"),
}
MODES = {"debug", "release", "miri", "address_sanitizer"}


def load_cases(root):
    manifest = strict_json_loads((Path(root) / "data/s07/ownership-cases.json").read_bytes())
    return validate_manifest(manifest)


def validate_manifest(manifest):
    if (type(manifest) is not dict or set(manifest) != {"version", "common", "groups"}
            or type(manifest["version"]) is not int or manifest["version"] != 2
            or type(manifest["common"]) is not dict or set(manifest["common"]) != set(COMMON)
            or type(manifest["groups"]) is not dict or set(manifest["groups"]) != set(GROUPS)):
        raise ValueError("invalid S07 ownership inventory")
    suites = [*manifest["common"].values(), *manifest["groups"].values()]
    identities = set()
    for suite in suites:
        if (type(suite) is not dict or set(suite) != {"package", "filter", "exact", "cases"}
                or type(suite["package"]) is not str or suite["package"] not in {"ts_ast", "ts_binder", "ts_compiler"}
                or type(suite["filter"]) is not str or not suite["filter"]
                or type(suite["exact"]) is not bool or type(suite["cases"]) is not list or not suite["cases"]
                or any(type(name) is not str or not re.fullmatch(r"(?:[a-z0-9_]+::)+[a-z0-9_]+", name) for name in suite["cases"])
                or suite["cases"] != sorted(set(suite["cases"]))
                or any(not name.startswith(suite["filter"]) for name in suite["cases"])
                or suite["exact"] and suite["cases"] != [suite["filter"]]):
            raise ValueError("invalid exact S07 ownership suite")
        for name in suite["cases"]:
            identity = suite["package"], name
            if identity in identities:
                raise ValueError("duplicate S07 ownership test across suites")
            identities.add(identity)
    for name, (package, prefix) in COMMON.items():
        suite = manifest["common"][name]
        if (suite["package"], suite["filter"], suite["exact"]) != (package, prefix, False):
            raise ValueError("S07 common ownership suite changed scope: " + name)
    if any(suite["package"] != "ts_compiler" or not suite["exact"]
           or not suite["filter"].startswith("ownership_tests::") for suite in manifest["groups"].values()):
        raise ValueError("S07 program ownership group changed scope")
    return manifest


def measure(root, invoke, prefix, options, env, manifest, mode):
    validate_manifest(manifest)
    if mode not in MODES:
        raise ValueError("invalid S07 ownership measurement mode")

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
    # Keep each independently observed common outcome. A missing new suite must
    # not be indistinguishable from the old two-group success shape. The shared
    # E3 caller can still use all(outcomes.values()) for its instrumentation flag.
    common = {name: execute(name, manifest["common"][name]) for name in COMMON}
    outcomes = {name: execute(name, manifest["groups"][name]) for name in GROUPS}
    return {**common, **outcomes}


def publish_metrics(report, modes, manifest):
    validate_manifest(manifest)
    required_outcomes = {*COMMON, *GROUPS}
    if (type(modes) is not dict or set(modes) != MODES
            or any(type(outcomes) is not dict or set(outcomes) != required_outcomes
                   or any(type(value) is not bool for value in outcomes.values()) for outcomes in modes.values())):
        raise ValueError("S07 ownership must execute every common suite and group in all four modes")
    for name in GROUPS:
        for mode, outcomes in modes.items():
            report["metrics"][f"{name}_{mode}"] = outcomes[name] and all(outcomes[common] for common in COMMON)
        report["metrics"][name] = all(report["metrics"][f"{name}_{mode}"] for mode in MODES)
    # Count distinct cases with complete validated observations in every mode.
    # Missing/failed suite output cannot inflate this informational denominator.
    suites = {**manifest["common"], **manifest["groups"]}
    report["metrics"]["program_ownership_tests"] = sum(
        len(suite["cases"]) for name, suite in suites.items() if all(outcomes[name] for outcomes in modes.values()))
