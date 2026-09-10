"""Access-only S08 overlays over the verified source pin; no checkout edits."""

import hashlib
import json
from pathlib import Path

from s04 import go_environment, verified_upstream
from s04_common import command, strict_json_loads

ROOT = Path(__file__).resolve().parents[1]


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run_overlay(directory, package, source, request, test_name):
    """Keep the exact request/overlay/output; failed Go execution yields no result."""
    directory = Path(directory).resolve()
    directory.mkdir(parents=True, exist_ok=False)
    upstream = verified_upstream()
    env = go_environment()
    source_path = directory / "export_test.go"
    source_path.write_text(source)
    request_path = directory / "requests.json"
    request_bytes = canonical(request) + b"\n"
    request_path.write_bytes(request_bytes)
    output = directory / "observations.json"
    virtual = upstream / "tsc/internal" / package / "codex_s08_export_test.go"
    if virtual.exists():
        raise ValueError(f"overlay would replace a source file: {virtual}")
    overlay = directory / "overlay.json"
    overlay.write_bytes(canonical({"Replace": {str(virtual): str(source_path)}}))
    env.update(S08_REQUESTS=str(request_path), S08_OUTPUT=str(output))
    stdout = command(["go", "test", "-trimpath", "-mod=readonly", "-overlay", str(overlay),
                      f"./internal/{package}", "-run", f"^{test_name}$", "-count=1",
                      "-timeout=5m"], cwd=upstream / "tsc", env=env)
    (directory / "go-test.stdout").write_bytes(stdout)
    verified_upstream()
    report = strict_json_loads(output.read_bytes())
    if report["request_sha256"] != digest(request_bytes):
        raise ValueError("Go observed a different request inventory")
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    (directory / "provenance.json").write_bytes(canonical({
        "pin": pin, "source_sha256": digest(source.encode()),
        "request_sha256": digest(request_bytes), "output_sha256": digest(output.read_bytes()),
        "go": report["go"], "goos": report["goos"], "goarch": report["goarch"],
        "toolchain_local": env["GOTOOLCHAIN"] == "local",
    }) + b"\n")
    return report
