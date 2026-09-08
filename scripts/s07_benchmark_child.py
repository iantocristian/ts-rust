#!/usr/bin/env python3
"""One benchmark child and its own OS resource accounting, never prior children."""
import json
import os
import subprocess
import sys
import tempfile
import time

from s04_common import strict_json_loads


def capture(argv, timeout=600):
    if sys.platform not in {"darwin", "linux"}:
        raise ValueError("peak RSS normalization is implemented for macOS and Linux only")
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as error:
        started = time.monotonic_ns()
        child = subprocess.Popen(argv, stdout=output, stderr=error)
        deadline = time.monotonic() + timeout
        try:
            while True:
                pid, status, usage = os.wait4(child.pid, os.WNOHANG)
                if pid:
                    child.returncode = os.waitstatus_to_exitcode(status)
                    break
                if time.monotonic() >= deadline:
                    raise ValueError("benchmark child exceeded the fixed process deadline")
                time.sleep(0.02)
        finally:
            # A failed/interrupting capture cannot leave its owned benchmark
            # child competing with a later sample on the same host.
            if child.returncode is None:
                child.kill()
                _, status, _ = os.wait4(child.pid, 0)
                child.returncode = os.waitstatus_to_exitcode(status)
        process_ns = time.monotonic_ns() - started
        output.seek(0)
        error.seek(0)
        raw = output.read(65537)
        stderr = error.read(65537)
        if len(raw) > 65536 or len(stderr) > 65536:
            raise ValueError("scalar benchmark child exceeded bounded output size")
        if child.returncode != 0:
            raise ValueError(f"benchmark child exited {child.returncode}: {stderr.decode(errors='backslashreplace')}")
        report = strict_json_loads(raw)
        rss = usage.ru_maxrss * (1024 if sys.platform == "linux" else 1)
        if type(rss) is not int or rss <= 0:
            raise ValueError("OS returned an invalid peak RSS sample")
        return {"report": report, "peak_rss_bytes": rss, "process_time_ns": process_ns,
                "user_time_ns": round(usage.ru_utime * 1_000_000_000),
                "system_time_ns": round(usage.ru_stime * 1_000_000_000),
                "stderr": stderr.decode("utf-8", errors="backslashreplace")}


if __name__ == "__main__":
    try:
        if len(sys.argv) != 4:
            raise ValueError("usage: s07_benchmark_child.py BINARY INPUTS WORKERS")
        print(json.dumps(capture(sys.argv[1:]), allow_nan=False, separators=(",", ":")))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1) from error
