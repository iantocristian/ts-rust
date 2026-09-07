"""Strict, bounded S05 stream validation and independent semantic comparison."""

from collections import Counter
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import selectors
import struct
import subprocess
import time

from s04 import panic_class as leaf_panic_class
from s04_common import strict_json_loads
from s05_cases import TOKEN_OPS, canonical, hex_bytes, validate_request

MAX_RECORD = 32 * 1024 * 1024
DEADLINE = 120


def shape(value, fields):
    if type(value) is not dict or set(value) != set(fields.split()):
        raise ValueError(f"invalid record fields; expected {fields}")


def integer(value, low=-(1 << 63), high=(1 << 63)-1):
    if type(value) is not int or not low <= value <= high:
        raise ValueError("invalid integer value")


def boolean(value):
    if type(value) is not bool:
        raise ValueError("invalid boolean value")


def predicates(value):
    if type(value) is not list or len(value) != 7:
        raise ValueError("invalid predicate inventory")
    for item in value:
        boolean(item)


def directives(value):
    if type(value) is not list:
        raise ValueError("invalid directives")
    for item in value:
        shape(item, "start end kind")
        integer(item["start"]); integer(item["end"]); integer(item["kind"], 0, 2)


def diagnostics(value):
    if type(value) is not list:
        raise ValueError("invalid diagnostic inventory")
    for item in value:
        shape(item, "code category key start length args")
        integer(item["code"], -(1 << 31), (1 << 31)-1); integer(item["category"], 0, 3)
        integer(item["start"]); integer(item["length"])
        if type(item["key"]) is not str or not item["key"] or type(item["args"]) is not list:
            raise ValueError("invalid diagnostic identity/arguments")
        for arg in item["args"]:
            if type(arg) is not dict or arg.get("kind") not in ("string", "integer"):
                raise ValueError("invalid diagnostic argument")
            if arg["kind"] == "string":
                shape(arg, "kind hex"); hex_bytes(arg["hex"])
            else:
                shape(arg, "kind value"); integer(arg["value"])


def range_value(value):
    if type(value) is not list or len(value) != 2:
        raise ValueError("invalid range")
    for item in value:
        integer(item)


def token_value(value):
    shape(value, "kind token full_start start end text_hex value_hex flags range predicates directives")
    integer(value["kind"], 0, 350); integer(value["token"], 0, 350)
    for field in ("full_start", "start", "end"):
        integer(value[field])
    integer(value["flags"], -(1 << 31), (1 << 31)-1)
    hex_bytes(value["text_hex"]); hex_bytes(value["value_hex"])
    range_value(value["range"]); predicates(value["predicates"]); directives(value["directives"])


def validate_value(action, value):
    op = action["op"]
    if op in TOKEN_OPS:
        token_value(value)
    elif op in ("reset", "set_text", "reset_pos", "reset_token_state", "set_skip_trivia", "set_skip_jsdoc_asterisks", "set_variant", "set_target", "set_on_error", "mark", "rewind", "commit"):
        if value is not None:
            raise ValueError("state action must return null")
    elif op in ("can_follow_jsdoc_at", "equal_fold", "valid_identifier", "identifier_text", "intrinsic_jsx_name"):
        boolean(value)
    elif op in ("identifier_token", "string_to_token"):
        integer(value, 0, 350)
    elif op == "identifier_point":
        if type(value) is not list or len(value) != 3:
            raise ValueError("invalid identifier point")
        for item in value:
            boolean(item)
    elif op == "identifier_block":
        shape(value, "start_hex part_hex jsx_hex")
        count = action["count"]
        for item in value.values():
            raw = hex_bytes(item)
            if len(raw) != (count+7)//8 or count % 8 and raw[-1] >> (count % 8):
                raise ValueError("invalid identifier bitset length/padding")
    elif op in ("token_to_string", "shebang", "normalize_jsdoc", "pseudo_bigint"):
        hex_bytes(value)
    elif op == "keyword_suggestions":
        if type(value) is not list:
            raise ValueError("invalid keyword suggestions")
        for item in value:
            hex_bytes(item)
        if value != sorted(set(value)):
            raise ValueError("keyword suggestions are not a sorted unique set")
    elif op == "comment_ranges":
        if type(value) is not list:
            raise ValueError("invalid comment range list")
        for item in value:
            shape(item, "start end kind trailing_newline")
            integer(item["start"]); integer(item["end"]); integer(item["kind"], 0, 350); boolean(item["trailing_newline"])
    elif op == "skip_trivia":
        integer(value)
    elif op in ("number_from_string", "number_format"):
        shape(value, "number_class bits text_hex")
        hex_bytes(value["text_hex"])
        if value["number_class"] == "nan":
            if value["bits"] is not None:
                raise ValueError("NaN payload must not become numeric parity")
        else:
            raw = hex_bytes(value["bits"])
            if len(raw) != 8:
                raise ValueError("invalid number bits")
            number = struct.unpack(">d", raw)[0]
            wanted = "finite" if math.isfinite(number) else "positive_infinity" if number == math.inf else "negative_infinity" if number == -math.inf else "nan"
            if wanted != value["number_class"]:
                raise ValueError("number classification contradicts bits")
    elif op == "observe":
        getter = action["getter"]
        if getter in ("text", "token_text", "value"):
            hex_bytes(value)
        elif getter == "range":
            range_value(value)
        elif getter == "directives":
            directives(value)
        elif getter == "predicates":
            predicates(value)
        elif getter == "token":
            integer(value, 0, 350)
        elif getter == "flags":
            integer(value, -(1 << 31), (1 << 31)-1)
        else:
            integer(value)
    else:
        raise ValueError("validated operation lacks output schema")


def decode_go_quoted(quoted):
    """Decode a Go %q string without assuming Unicode text or Go IsPrint data."""
    if type(quoted) is not str or len(quoted) < 2 or quoted[0] != '"' or quoted[-1] != '"':
        raise ValueError("invalid Go quoted string")
    out = bytearray()
    position, end = 1, len(quoted)-1
    controls = {"a": 7, "b": 8, "f": 12, "n": 10, "r": 13, "t": 9, "v": 11, "\\": 92, '"': 34}
    while position < end:
        ch = quoted[position]
        position += 1
        if ch != "\\":
            if ch == '"' or ord(ch) < 32:
                raise ValueError("invalid character in Go quoted string")
            out.extend(ch.encode("utf-8"))
            continue
        if position >= end:
            raise ValueError("truncated Go escape")
        escape = quoted[position]
        position += 1
        if escape in controls:
            out.append(controls[escape])
            continue
        if escape in "xXuU":
            # Go supports lowercase x only, and lowercase u / uppercase U.
            widths = {"x": 2, "u": 4, "U": 8}
            if escape not in widths:
                raise ValueError("invalid Go hexadecimal escape")
            width = widths[escape]
            digits = quoted[position:position+width]
            if position+width > end or len(digits) != width or any(c not in "0123456789abcdefABCDEF" for c in digits):
                raise ValueError("invalid Go hexadecimal escape")
            position += width
            number = int(digits, 16)
            if escape == "x":
                out.append(number)
            elif number > 0x10FFFF or 0xD800 <= number <= 0xDFFF:
                raise ValueError("invalid Go Unicode scalar escape")
            else:
                out.extend(chr(number).encode("utf-8"))
            continue
        if escape in "01234567":
            digits = escape + quoted[position:position+2]
            if position+2 > end or len(digits) != 3 or any(c not in "01234567" for c in digits) or int(digits, 8) > 255:
                raise ValueError("invalid Go octal escape")
            out.append(int(digits, 8)); position += 2
            continue
        raise ValueError("unknown Go escape")
    return bytes(out)


def classify_panic(action, message, runtime, source_hex):
    contracts = {
        "Cannot reset token state to negative position": {"reset_pos", "reset_token_state"},
        "'ReScanAsteriskEqualsToken' should only be called on a '*='": {"rescan_asterisk_equals"},
        "'reScanQuestionToken' should only be called on a '??'": {"rescan_question"},
    }
    if action["op"] in contracts.get(message, set()):
        return "contract", None
    if action["op"] == "rescan_slash" and message == "Debug failure. False expression.":
        return "upstream_assertion", None
    if leaf_panic_class(message, runtime) == "bounds":
        return "bounds", None
    if action["op"] == "pseudo_bigint":
        input_hex = hex_bytes(source_hex).removesuffix(b"n").hex()
        if runtime == "oracle" and message.startswith('Failed to parse big int: '):
            try:
                recovered = decode_go_quoted(message.removeprefix("Failed to parse big int: "))
            except (ValueError, UnicodeError):
                return "unexpected", None
            if recovered.hex() == input_hex:
                return "invalid_bigint", recovered.hex()
            return "unexpected", None
        if runtime == "rust" and message == f"Failed to parse big int (hex): {input_hex}":
            return "invalid_bigint", input_hex
    return "unexpected", None


def decoded_request_source(raw):
    """Protocol-bound input bookkeeping, mirroring the pinned BOM contract.

    This establishes panic payload identity and token-count limits; source-decoder
    parity itself remains S04 evidence, not a scanner metric derived from this model.
    """
    if raw.startswith((b"\xff\xfe", b"\xfe\xff")):
        encoding = "utf-16-le" if raw[:2] == b"\xff\xfe" else "utf-16-be"
        payload = raw[2:]
        payload = payload[:len(payload)//2*2]  # Go ignores one incomplete word.
        return payload.decode(encoding, errors="replace").encode("utf-8")
    return raw.removeprefix(b"\xef\xbb\xbf")


class StreamCase:
    """One stream's framing; token counts need not match the other implementation."""
    def __init__(self, request, runtime):
        self.request = validate_request(request)
        self.runtime = runtime
        self.action = self.ordinal = self.action_count = 0
        self.panic = False
        source = hex_bytes(request["source_hex"])
        if request["decode_source"]:
            source = decoded_request_source(source)
        self.source_hex = source.hex()
        self.source_bound = len(source) + 2

    def begin(self, record):
        shape(record, "event id version")
        if record["event"] != "begin" or record["id"] != self.request["id"] or type(record["version"]) is not int or record["version"] != 1:
            raise ValueError("invalid begin record")

    def observation(self, record):
        shape(record, "event id action ordinal status value diagnostics")
        integer(record["action"], 0); integer(record["ordinal"], 0)
        if self.panic or record["event"] != "observation" or record["id"] != self.request["id"] or record["action"] != self.action or record["ordinal"] != self.ordinal or self.action >= len(self.request["actions"]):
            raise ValueError("missing/extra/reordered action or observation")
        action = self.request["actions"][self.action]
        diagnostics(record["diagnostics"])
        if record["status"] == "panic":
            shape(record["value"], "message class input_hex")
            message = record["value"]["message"]
            if type(message) is not str or not message:
                raise ValueError("invalid panic payload")
            expected_class, expected_input = classify_panic(action, message, self.runtime, self.source_hex)
            if (record["value"]["class"], record["value"]["input_hex"]) != (expected_class, expected_input):
                raise ValueError("panic classification does not match raw payload/operation")
            self.panic = True
        elif record["status"] == "ok":
            validate_value(action, record["value"])
        else:
            raise ValueError("invalid observation status")
        self.ordinal += 1
        self.action_count += 1
        if action["op"] == "scan_all" and self.action_count > self.source_bound:
            raise ValueError("scan token bound exceeded")
        if action["op"] == "set_text" and not self.panic:
            self.source_hex = action["text_hex"]
            self.source_bound = len(hex_bytes(self.source_hex)) + 2
        if action["op"] == "reset" and not self.panic:
            self.source_hex = ""
            self.source_bound = 2
        if self.panic or action["op"] != "scan_all" or record["value"]["kind"] == 1:
            self.action += 1
            self.action_count = 0

    def end(self, record):
        shape(record, "event id observations completed_actions")
        integer(record["observations"], 0); integer(record["completed_actions"], 0)
        if record["event"] != "end" or record["id"] != self.request["id"] or record["observations"] != self.ordinal or record["completed_actions"] != self.action or self.action_count or not self.panic and self.action != len(self.request["actions"]):
            raise ValueError("incomplete case, missing EOF or inconsistent end counts")


class Process:
    def __init__(self, command, stderr_path, env=None, deadline=DEADLINE, max_record=MAX_RECORD):
        self.log = open(stderr_path, "wb")
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log, env=env)
        self.deadline, self.max_record = deadline, max_record
        self.buffer = bytearray()
        self.digest = hashlib.sha256()
        os.set_blocking(self.process.stdin.fileno(), False)
        os.set_blocking(self.process.stdout.fileno(), False)

    def wait(self, fd, event, expires):
        with selectors.DefaultSelector() as selector:
            selector.register(fd, event)
            remaining = expires - time.monotonic()
            if remaining <= 0 or not selector.select(remaining):
                raise TimeoutError("S05 adapter progress deadline exceeded")

    def send(self, request):
        raw = canonical(validate_request(request)) + b"\n"
        if len(raw) > self.max_record:
            raise ValueError("oversized request")
        view = memoryview(raw)
        expires = time.monotonic() + self.deadline
        while view:
            self.wait(self.process.stdin, selectors.EVENT_WRITE, expires)
            try:
                count = os.write(self.process.stdin.fileno(), view)
            except BlockingIOError:
                continue
            view = view[count:]

    def read(self, allow_eof=False):
        expires = time.monotonic() + self.deadline
        while True:
            end = self.buffer.find(b"\n")
            if end >= 0:
                if end + 1 > self.max_record:
                    raise ValueError("oversized response record")
                raw = bytes(self.buffer[:end+1]); del self.buffer[:end+1]
                value = strict_json_loads(raw.decode("utf-8"))
                self.digest.update(canonical(value)+b"\n")
                return value
            if len(self.buffer) > self.max_record:
                raise ValueError("oversized response record")
            self.wait(self.process.stdout, selectors.EVENT_READ, expires)
            try:
                raw = os.read(self.process.stdout.fileno(), 64*1024)
            except BlockingIOError:
                continue
            if not raw:
                if self.buffer or not allow_eof:
                    raise ValueError("truncated adapter output")
                return None
            self.buffer.extend(raw)

    def observations(self, request, runtime):
        state = StreamCase(request, runtime)
        state.begin(self.read())
        while True:
            record = self.read()
            if type(record) is dict and record.get("event") == "end":
                state.end(record)
                return
            state.observation(record)
            yield record

    def finish(self):
        self.process.stdin.close()
        if self.read(allow_eof=True) is not None:
            raise ValueError("unexpected trailing protocol records")
        if self.process.wait(timeout=self.deadline):
            raise RuntimeError("adapter exited unsuccessfully")

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=10)
        for stream in (self.process.stdin, self.process.stdout):
            if stream and not stream.closed:
                stream.close()
        self.log.close()


def expected_panic_matches(item, record):
    expected = item["expected_panic"]
    if expected is None or record["action"] != expected["action"] or record["value"]["class"] != expected["class"]:
        return False
    # StreamCase independently checked each raw payload against its runtime syntax.
    # Go and Rust choose different bounds details and wording for the same operation.
    if expected["class"] == "bounds":
        return True
    key = "input_hex" if expected["class"] == "invalid_bigint" else "message"
    return record["value"][key] == expected[key]


def compare_case(item, oracle, rust):
    missing = object()
    first_failure = None
    matched = True
    complete = True
    diagnostic_digests = [hashlib.sha256(), hashlib.sha256()]
    value_digests = [hashlib.sha256(), hashlib.sha256()]
    counts = [0, 0]
    witness_codes = [[], []]
    panics = [False, False]
    for left, right in itertools.zip_longest(oracle, rust, fillvalue=missing):
        pair_matches = left is not missing and right is not missing
        for index, record in enumerate((left, right)):
            if record is missing:
                continue
            counts[index] += 1
            for diagnostic in record["diagnostics"]:
                diagnostic_digests[index].update(canonical([record["action"], diagnostic])+b"\n")
                if item["witness"] and record["action"] == item["witness"]["action"]:
                    witness_codes[index].append(diagnostic["code"])
            if record["status"] == "panic":
                panics[index] = True
                if not expected_panic_matches(item, record):
                    pair_matches = False
                    complete = False
            elif item["request"]["actions"][record["action"]]["op"] in TOKEN_OPS:
                value_digests[index].update(canonical([record["action"], record["value"]["text_hex"], record["value"]["value_hex"]])+b"\n")
        if pair_matches:
            if left["status"] == right["status"] == "panic":
                pair_matches = left["diagnostics"] == right["diagnostics"] and expected_panic_matches(item, left) and expected_panic_matches(item, right)
            else:
                pair_matches = canonical(left) == canonical(right)
        if not pair_matches:
            matched = False
            if first_failure is None:
                first_failure = {"expected": None if left is missing else left, "actual": None if right is missing else right}
    if item["expected_panic"] and not all(panics):
        matched = complete = False
        first_failure = first_failure or {"error": "expected contract panic did not occur"}
    witness = item["witness"]
    witness_ok = not witness or all(codes == witness["codes"] for codes in witness_codes)
    if not witness_ok:
        matched = False
        first_failure = first_failure or {"error": "diagnostic witness not observed", "codes": witness_codes}
    return {"pass": matched, "diagnostics": complete and witness_ok and diagnostic_digests[0].digest() == diagnostic_digests[1].digest(),
            "values": complete and value_digests[0].digest() == value_digests[1].digest(),
            "observations": counts, "failure": first_failure}


def report(items, results):
    if not items or len(items) != len(results):
        raise ValueError("incomplete result inventory")
    groups = Counter(group for item in items for group in item["groups"])
    if any(groups[group] == 0 for group in ("regexp", "rescan", "diagnostics", "values", "identifier", "numbers")):
        raise ValueError("empty required measurement subset")
    if {item["request"]["id"] for item in items if item["witness"]} != {"witness/binary", "witness/grammar-true", "witness/grammar-false", "witness/unterminated-true", "witness/unterminated-false"}:
        raise ValueError("missing/extra diagnostic witness inventory")
    metrics = {"probes": len(items), "failed_cases": sum(not result["pass"] for result in results),
               "observations": sum(result["observations"][0] for result in results),
               "rust_observations": sum(result["observations"][1] for result in results)}
    for group in ("regexp", "rescan"):
        selected = [result for item, result in zip(items, results) if group in item["groups"]]
        metrics[group+"_parity"] = sum(result["pass"] for result in selected) / len(selected)
        metrics[group+"_cases"] = len(selected)
    for group, metric in (("diagnostics", "diagnostics"), ("values", "token_value_bytes")):
        selected = [result for item, result in zip(items, results) if group in item["groups"]]
        metrics[metric] = all(result[group] for result in selected)
        metrics[metric+"_cases"] = len(selected)
    return {"metrics": metrics, "tests": {item["request"]["id"]: "pass" if result["pass"] else "fail" for item, result in zip(items, results)}}
