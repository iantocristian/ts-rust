"""Run the same arena scenarios in release, debug, Miri and AddressSanitizer.

Only the seven implemented S04 ownership scenarios become evidence. Instrumented
runs use a separately pinned nightly, native target and workspace-local caches.
No synthetic success or skipped-test count can satisfy an instrumentation gate.
"""

import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib


SCENARIOS = (
    "id_exhaustion", "wrong_owner_rejected", "stale_and_recycled_ids_rejected",
    "concurrent_lazy_storage", "mapper_bundle_disposal", "owners_return_to_baseline",
    "allocations_return_to_baseline",
)


def strict_json_loads(data):
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate ownership JSON key: {key}")
            result[key] = value
        return result

    def invalid_constant(value):
        raise ValueError(f"non-finite ownership JSON number: {value}")

    return json.loads(data, object_pairs_hook=unique_object, parse_constant=invalid_constant)


def invoke(root, args, env=None):
    print("+ " + " ".join(args), file=sys.stderr)
    result = subprocess.run(args, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    sys.stderr.buffer.write(result.stderr)
    if result.returncode:
        sys.stderr.buffer.write(result.stdout)
        raise RuntimeError(f"ownership command exited {result.returncode}: {args}")
    return result.stdout


def validate_test_output(output):
    text = output.decode()
    sys.stderr.write(text)
    summary = re.findall(r"test result: ok\. (\d+) passed; 0 failed; (\d+) ignored;", text)
    if not summary or any(int(passed) < len(SCENARIOS) or int(ignored) for passed, ignored in summary):
        raise ValueError("ownership suite must run every required scenario without ignores")
    for scenario in SCENARIOS:
        if not re.search(rf"^test [\w:]*e3_{scenario} \.\.\. ok$", text, re.MULTILINE):
            raise ValueError(f"ownership suite did not pass {scenario}")


def validate_measurements(report):
    if not isinstance(report, dict) or set(report) != {"metrics", "tests"}:
        raise ValueError("invalid ownership report envelope")
    tests = report["tests"]
    if not isinstance(tests, list) or sorted(test["id"] for test in tests) != sorted(SCENARIOS):
        raise ValueError("ownership report must contain exactly the frozen scenarios")
    if any(set(test) != {"id", "result"} or test["result"] != "pass" for test in tests):
        raise ValueError("ownership scenario failed or was skipped")
    metrics = report["metrics"]
    expected_metrics = set(SCENARIOS[:5]) | {"live_owner_delta", "live_allocation_delta"}
    if not isinstance(metrics, dict) or set(metrics) != expected_metrics:
        raise ValueError("native example may emit only the measured S04 metrics")
    for scenario in SCENARIOS[:5]:
        if metrics.get(scenario) is not True:
            raise ValueError(f"missing or failed ownership measurement {scenario}")
    for metric in ("live_owner_delta", "live_allocation_delta"):
        value = metrics.get(metric)
        if type(value) is not int or value < 0:
            raise ValueError(f"missing or invalid measured counter {metric}")
    if "miri" in metrics or "address_sanitizer" in metrics:
        raise ValueError("native example cannot assert instrumentation completion")
    return report


def run(root):
    root = Path(root)
    manifest = json.loads((root / "data/s04/e3-cases.json").read_text())
    if manifest != sorted(SCENARIOS):
        raise ValueError("ownership case manifest drift")
    base = os.environ.copy()
    base["CARGO_TERM_COLOR"] = "never"
    tests = ["test", "--package", "ts_arena", "--lib", "--locked"]
    tail = ["--", "--test-threads=1"]
    validate_test_output(invoke(root, ["cargo", *tests, *tail], base))
    validate_test_output(invoke(root, ["cargo", *tests, "--release", *tail], base))
    report = validate_measurements(strict_json_loads(invoke(root, [
        "cargo", "run", "--quiet", "--package", "ts_arena", "--example", "e3",
        "--features", "harness", "--release", "--locked",
    ], base)))
    toolchains = tomllib.loads((root / "data/s04/toolchains.toml").read_text())
    nightly = toolchains["nightly"]
    if not re.fullmatch(r"nightly-\d{4}-\d{2}-\d{2}", nightly):
        raise ValueError("ownership instrumentation requires a dated nightly")
    native = invoke(root, ["rustc", "-Vv"], base).decode()
    print(native, file=sys.stderr)
    host = next(line.removeprefix("host: ") for line in native.splitlines() if line.startswith("host: "))
    instrument = {**base, "RUSTUP_HOME": str(root / "target/s04-rustup"),
                  "CARGO_HOME": str(root / "target/s04-cargo-home")}
    print(invoke(root, ["rustup", "run", nightly, "rustc", "-Vv"], instrument).decode(), file=sys.stderr)
    # Cargo gives its encoded variable precedence even when it is empty. Remove
    # it so the ASan RUSTFLAGS below cannot silently become an uninstrumented run.
    instrument.pop("CARGO_ENCODED_RUSTFLAGS", None)
    # parking_lot_core reconstructs queue pointers from tagged integers. Run
    # Miri's default exposed-provenance model, retaining its warning and all
    # other checks, against the real production lock implementation. Strict
    # provenance rejects that dependency operation before testing our storage.
    miri = {**instrument, "MIRI_SYSROOT": str(root / "target/s04-miri-sysroot"),
            "CARGO_TARGET_DIR": str(root / "target/s04-miri"),
            "MIRIFLAGS": "", "RUSTFLAGS": ""}
    invoke(root, ["cargo", f"+{nightly}", "miri", "setup", "--target", host], miri)
    validate_test_output(invoke(root, ["cargo", f"+{nightly}", "miri", *tests, "--target", host, *tail], miri))
    report["metrics"]["miri"] = True
    asan = {**instrument, "CARGO_TARGET_DIR": str(root / "target/s04-asan"),
            "RUSTFLAGS": "-Zsanitizer=address",
            # macOS ASan does not support LeakSanitizer. Miri and measured
            # ownership counters still require all roots/allocations to drop.
            "ASAN_OPTIONS": "detect_leaks=0" if "apple" in host else "detect_leaks=1"}
    validate_test_output(invoke(root, ["cargo", f"+{nightly}", *tests, "-Zbuild-std", "--target", host, *tail], asan))
    report["metrics"]["address_sanitizer"] = True
    # The native example's row inventory has already been checked for duplicate
    # IDs. The evidence runner requires its public id -> outcome map schema.
    report["tests"] = {test["id"]: test["result"] for test in report["tests"]}
    return report
