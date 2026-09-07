"""Strict wire decoding and subprocess capture shared by the S04 producers."""

import json
import math
import subprocess
import sys


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


def command(args, *, cwd, env=None, data=None):
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
