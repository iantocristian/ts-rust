#!/usr/bin/env python3
"""Bounded streaming verification of recorded S07 access operations, not semantic replay."""
import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

MAGIC = b"S07TRC01"
BLOCK = struct.Struct("<QII32s")
RECORD = struct.Struct("<HHIIIQQQQ")
FOOTER = struct.Struct("<QQQ32s")
MIB = 1024 * 1024
READ_CHUNK = 64 * 1024
FIELD_TYPES = {"u64", "u32", "u16", "bool", "zero", "node_id", "optional_node_id"}


class InvalidTrace(ValueError):
    """Malformed, incomplete, over-budget or registry-incompatible trace."""


def require(condition, message):
    if not condition:
        raise InvalidTrace(message)


def strict_json(raw):
    def pairs(values):
        result = {}
        for key, value in values:
            require(key not in result, "duplicate JSON key: " + key)
            result[key] = value
        return result

    def constant(value):
        raise InvalidTrace("nonfinite JSON value: " + value)

    return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)


def integer(value, maximum, label):
    require(type(value) is int and 0 <= value <= maximum, "invalid " + label)
    return value


@dataclass(frozen=True)
class Limits:
    payload_bytes: int = 32 * 1024**3
    compressed_bytes: int = 3 * 1024**3
    block_bytes: int = MIB
    record_bytes: int = MIB
    files: int = 13094
    active_nodes: int = 2_000_000

    def validate(self):
        for name in ("payload_bytes", "compressed_bytes", "files", "active_nodes"):
            require(type(getattr(self, name)) is int and getattr(self, name) >= 0,
                    "invalid verifier limit: " + name)
        for name in ("block_bytes", "record_bytes"):
            require(type(getattr(self, name)) is int and 52 <= getattr(self, name) <= MIB,
                    "invalid verifier limit: " + name)


class GzipReader:
    """One gzip member, bounded reads/decompression, strict compressed EOF."""

    def __init__(self, source, maximum):
        self.source = source
        self.maximum = maximum
        self.compressed_bytes = 0
        self.compressed_sha = hashlib.sha256()
        self.stream_sha = hashlib.sha256()
        self.decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
        self.pending = b""
        self.finished = False

    def raw_read(self, maximum):
        value = self.source.read(min(maximum, self.maximum - self.compressed_bytes + 1))
        require(type(value) is bytes, "trace input must be a binary stream")
        self.compressed_bytes += len(value)
        require(self.compressed_bytes <= self.maximum, "compressed-byte limit exceeded")
        self.compressed_sha.update(value)
        return value

    def read(self, count):
        require(0 < count <= MIB, "internal unbounded trace read")
        if self.finished:
            return b""
        while True:
            value = self.pending or self.raw_read(READ_CHUNK)
            require(bool(value), "truncated gzip stream")
            try:
                result = self.decoder.decompress(value, min(count, READ_CHUNK))
            except zlib.error as error:
                raise InvalidTrace("invalid gzip stream: " + str(error)) from error
            self.pending = self.decoder.unconsumed_tail
            if self.decoder.eof:
                require(not self.decoder.unused_data and not self.pending,
                        "trailing compressed data or multiple gzip members")
                require(self.raw_read(1) == b"", "trailing compressed data or multiple gzip members")
                self.finished = True
            self.stream_sha.update(result)
            if result or self.finished:
                return result

    def exact(self, count):
        require(0 <= count <= MIB, "internal unbounded trace read")
        result = bytearray()
        while len(result) < count:
            value = self.read(count - len(result))
            require(bool(value), "truncated trace structure")
            result.extend(value)
        return bytes(result)


def registry(documents):
    events = {}
    for document in documents:
        require(type(document) is dict and type(document.get("version")) is int
                and document["version"] == 1 and type(document.get("events")) is list,
                "invalid event registry")
        for event in document["events"]:
            require(type(event) is dict, "invalid event definition")
            op = integer(event.get("id"), 65535, "event ID")
            require(op not in events, "duplicate event ID")
            expected_domain = 0 if 1 <= op <= 9 else 1 if 10 <= op <= 79 else 2 if 100 <= op <= 149 else None
            require(expected_domain is not None and type(event.get("domain")) is int
                    and event["domain"] == expected_domain, "event ID/domain range mismatch")
            require(type(event.get("name")) is str and bool(event["name"]), "event name is required")
            sites = event.get("sites")
            if sites is None:
                require(type(event.get("site_semantics")) is str and bool(event["site_semantics"]),
                        "wildcard sites require documented site_semantics")
            else:
                require(type(sites) is list and bool(sites), "event sites must be a nonempty list")
                ids = [integer(site["id"], 65535, "site ID") for site in sites]
                require(len(ids) == len(set(ids)), "duplicate site ID for event")
                sites = frozenset(ids)
            fields = event.get("fields")
            require(type(fields) is dict and set(fields) == {"a", "b", "c", "d"},
                    "event must describe all four fields")
            fields = {name: {"name": field, "type": "u64"} if type(field) is str else field
                      for name, field in fields.items()}
            for field in fields.values():
                require(type(field) is dict and type(field.get("name")) is str
                        and field.get("type") in FIELD_TYPES, "invalid event field definition")
                for bound in ("min", "max", "exact_value"):
                    if bound in field:
                        integer(field[bound], 2**64 - 1, "field bound")
            blob = event.get("blob", "none")
            if type(blob) is bool:
                blob = "bytes" if blob else "none"
            require(blob in ("none", "bytes"), "unsupported event blob contract")
            category = event.get("category", "other")
            require(category in ("lookup", "read", "write", "other", "driver", "state"),
                    "invalid event category")
            require(category not in ("lookup", "read", "write") or expected_domain == 2,
                    "observer/driver operations cannot count as binder operations")
            constraints = event.get("constraints", [])
            require(type(constraints) is list, "invalid event constraints")
            for constraint in constraints:
                require(type(constraint) is dict and set(constraint) == {"rule", "index_field", "descriptor_field"}
                        and constraint["rule"] == "relative_index_less_than_packed_len"
                        and constraint["index_field"] in fields and constraint["descriptor_field"] in fields,
                        "unknown event constraint")
            checks = []
            for index, name in enumerate(("a", "b", "c", "d")):
                field = fields[name]
                if field["type"] != "u64" or set(field) & {"min", "max", "exact_value"}:
                    checks.append((index, field))
            events[op] = {**event, "fields": fields, "checks": tuple(checks), "sites": sites,
                          "blob": blob, "category": category,
                          "constraints": tuple((("a", "b", "c", "d").index(value["index_field"]),
                                                ("a", "b", "c", "d").index(value["descriptor_field"]))
                                               for value in constraints)}
    require(set(range(1, 10)).issubset(events), "driver phase registry is incomplete")
    return events


def check_field(value, field):
    kind = field["type"]
    maximum = {"u16": 65535, "u32": 2**32 - 1, "bool": 1, "zero": 0}.get(kind, 2**64 - 1)
    require(value <= maximum, "field exceeds " + kind + ": " + field["name"])
    if kind in ("node_id", "optional_node_id") and not (kind == "optional_node_id" and value == 0):
        require(value >> 32 != 0 and value & (2**32 - 1) != 0,
                "node ID requires a nonzero arena and slot: " + field["name"])
    if "exact_value" in field:
        require(value == field["exact_value"], "field differs from exact_value: " + field["name"])
    require(value >= field.get("min", 0) and value <= field.get("max", 2**64 - 1),
            "field outside registry bounds: " + field["name"])


class Observations:
    def __init__(self, events, limits, allow_empty, expected):
        self.events, self.limits, self.allow_empty, self.expected = events, limits, allow_empty, expected
        self.by_op, self.by_site, self.by_domain, self.by_file = Counter(), Counter(), Counter(), Counter()
        self.by_op_site = defaultdict(Counter)
        self.binder = Counter({"lookup": 0, "read": 0, "write": 0, "other": 0})
        self.by_op_shape = defaultdict(Counter)
        self.headers = {}
        self.files = []
        self.active = None
        self.phase = 0
        self.capture_ended = False
        self.state = None
        self.pending_blob = None
        self.totals = {"files": 0, "source_bytes": 0, "nodes": 0, "symbols": 0}

    def event(self, op, site, file, domain, reserved, values, blob):
        require(not self.capture_ended, "event after capture_end")
        require(op in self.events, "unknown event operation: " + str(op))
        spec = self.events[op]
        require(domain == spec["domain"], "event domain differs from registry")
        require(spec["sites"] is None or site in spec["sites"], "unknown site for event")
        require(reserved == 0, "record reserved field is nonzero")
        require(spec["blob"] == "bytes" or len(blob) == 0, "unexpected event blob")
        for index, field in spec["checks"]:
            check_field(values[index], field)
        for index, descriptor in spec["constraints"]:
            require(values[index] < values[descriptor] & (2**32 - 1),
                    "successful slice index is outside the selected descriptor")
        if domain == 0:
            self.driver(op, file, values)
        else:
            require(self.active is not None and file == self.active["file"], "event outside its active file")
            require(self.phase == (4 if domain == 1 else 6), "event outside its declared phase")
            if domain == 2:
                category = spec["category"] if spec["category"] in self.binder else "other"
                self.binder[category] += 1
                self.active["binder_operations"][category] += 1
                self.node_operation(op, values)
            else:
                self.active["state_operations"] += 1
                self.state_event(op, site, values, blob, spec["blob"] == "bytes")
                if op == 14:
                    node, kind, parent, flags = values
                    check_field(node, {"name": "state node", "type": "node_id"})
                    check_field(parent, {"name": "state parent", "type": "optional_node_id"})
                    check_field(flags, {"name": "state flags", "type": "u32"})
                    normalized = kind & 65535
                    signed = normalized if normalized < 32768 else normalized - 65536
                    require(kind == signed & (2**64 - 1), "state kind is not sign-extended i16")
                    require(node not in self.headers, "duplicate initial node header")
                    require(len(self.headers) < self.limits.active_nodes, "active-node limit exceeded")
                    self.headers[node] = [normalized, flags, site]
        self.by_op[op] += 1
        self.by_site[site] += 1
        self.by_op_site[op][site] += 1
        self.by_domain[domain] += 1
        self.by_file[file] += 1

    def state_event(self, op, site, values, blob, is_blob):
        a, b, c, d = values
        if op == 10:
            require(self.state is None, "duplicate state owner")
            require(0 < a < 2**32 and 0 < b < 2**32 and a != b,
                    "invalid core arena identities")
            require(self.active["root"] >> 32 == a, "root differs from declared core owner")
            require(d == self.active["source_bytes"], "state source length differs from driver")
            self.state = {"node_arena": a, "aux_arena": b, "nodes": None, "aux": None,
                          "aux_headers": 0, "blob_bytes": 0, "ended": False}
        require(self.state is not None and not self.state["ended"], "state record outside owner/end markers")
        state = self.state
        if is_blob:
            require(len(blob) <= READ_CHUNK, "blob chunk exceeds declared 64KiB limit")
            identity = (op, site, a, b, d)
            if self.pending_blob is None:
                require(c == 0, "blob begins at nonzero offset")
            else:
                old_identity, offset = self.pending_blob
                require(identity == old_identity and c == offset, "blob chunk identity/offset changed")
            require(c + len(blob) <= d and (len(blob) > 0 or d == 0), "invalid blob chunk extent")
            state["blob_bytes"] += len(blob)
            self.pending_blob = (identity, c + len(blob)) if c + len(blob) < d else None
        else:
            require(self.pending_blob is None, "interrupted or truncated blob chunks")
        if op == 11:
            require(site in (1, 2), "unknown core arena layout")
            key, arena = ("nodes", "node_arena") if site == 1 else ("aux", "aux_arena")
            require(state[key] is None and a == state[arena] and b <= c,
                    "duplicate or inconsistent core arena layout")
            state[key] = b
        elif op == 14:
            require(state["nodes"] is not None and a >> 32 == state["node_arena"]
                    and (a & (2**32 - 1)) == len(self.headers) + 1
                    and len(self.headers) < state["nodes"], "node header differs from core physical slot order")
        elif op == 24:
            require(state["aux"] is not None and a >> 32 == state["aux_arena"]
                    and (a & (2**32 - 1)) == state["aux_headers"] + 1
                    and state["aux_headers"] < state["aux"], "auxiliary header differs from core physical slot order")
            state["aux_headers"] += 1
        elif op == 79:
            require(state["nodes"] == a == len(self.headers) and self.active["root"] in self.headers,
                    "state end/core layout/header counts or root differ")
            require(state["aux"] == b == state["aux_headers"], "state auxiliary counts differ")
            require(d == state["blob_bytes"], "state selected-byte count differs")
            state["ended"] = True

    def node_operation(self, op, values):
        if op not in (100, 101, 102, 103, 107):
            return
        header = self.headers.get(values[0])
        if header is None:
            self.by_op_shape[op]["unmapped"] += 1
            self.active["unmapped_node_operations"] += 1
            return
        kind, flags, shape = header
        self.by_op_shape[op][str(shape)] += 1
        if op == 101:
            require(values[1] == kind, "named kind read differs from initial node header")
        elif op == 102:
            require(values[1] == flags, "named flags read differs from tracked node flags")
        elif op == 103:
            header[1] = values[1]

    def driver(self, op, file, values):
        a, b, c, d = values
        if op == 9:
            require(self.active is None and file == 0, "capture_end while a file is active or file is nonzero")
            require(self.allow_empty or self.totals["files"] > 0, "empty capture is not an actual workload")
            require(values == tuple(self.totals[key] for key in ("files", "source_bytes", "nodes", "symbols")),
                    "capture_end totals differ from completed files")
            for name, expected in self.expected.items():
                require(self.totals[name] == expected, "capture differs from expected " + name)
            self.capture_ended = True
            return
        if op == 1:
            require(self.active is None and file == self.totals["files"] + 1 and a == file - 1,
                    "file_begin order/identity changed")
            require(file <= self.limits.files, "file-count limit exceeded")
            self.active = {"file": file, "input_index": a, "source_bytes": b,
                           "binder_operations": {name: 0 for name in self.binder}, "state_operations": 0,
                           "unmapped_node_operations": 0}
            self.phase = 1
            require(c == 0 and d == 0, "file_begin unused fields are nonzero")
            return
        require(self.active is not None and file == self.active["file"] and op == self.phase + 1,
                "driver phase order/identity changed")
        if op in (2, 4, 5, 6):
            require(values == (0, 0, 0, 0), "phase marker fields are nonzero")
            if op == 5:
                require(self.state is not None and self.state["ended"], "incomplete initial-state export")
        elif op == 3:
            check_field(a, {"name": "root", "type": "node_id"})
            require(c == 0 and d == 0, "parse_end unused fields are nonzero")
            self.active.update(root=a, parsed_nodes=b)
        elif op == 7:
            self.active.update(nodes=a, symbols=b, parse_diagnostics=c, bind_diagnostics=d)
        elif op == 8:
            require(a in (0, 1) and (b, c, d) == (0, 0, 0), "invalid file_end fields")
            self.active["bound_in_place"] = bool(a)
            self.active["state_node_headers"] = len(self.headers)
            self.totals["files"] += 1
            for key in ("source_bytes", "nodes", "symbols"):
                self.totals[key] += self.active[key]
            self.files.append(self.active)
            self.active = None
            self.headers.clear()
            self.state = None
        self.phase = op


def verify(source, events, *, limits=Limits(), allow_empty=False, expected=None,
           progress=None, progress_records=1_000_000):
    limits.validate()
    require(type(progress_records) is int and progress_records > 0, "invalid progress interval")
    expected = {} if expected is None else expected
    require(set(expected) <= {"files", "source_bytes", "nodes", "symbols"}, "unknown expected total")
    for name, value in expected.items():
        integer(value, 2**64 - 1, "expected " + name)
    reader = GzipReader(source, limits.compressed_bytes)
    observations = Observations(events, limits, allow_empty, expected)
    require(reader.exact(8) == MAGIC, "trace magic/version mismatch")
    blocks = records = payload_bytes = 0
    next_progress = progress_records
    hashes = hashlib.sha256()
    while True:
        tag = reader.exact(4)
        if tag == b"END1":
            recorded_blocks, recorded_records, recorded_bytes, digest = FOOTER.unpack(reader.exact(FOOTER.size))
            require((recorded_blocks, recorded_records, recorded_bytes) == (blocks, records, payload_bytes),
                    "footer counters differ from observed blocks")
            require(digest == hashes.digest(), "footer block-hash digest mismatch")
            require(observations.capture_ended, "missing capture_end")
            require(reader.read(1) == b"", "duplicate footer or trailing trace bytes")
            break
        require(tag == b"BLK1", "unknown block tag or missing footer")
        sequence, count, length, digest = BLOCK.unpack(reader.exact(BLOCK.size))
        require(sequence == blocks, "block sequence is missing, duplicated or reordered")
        require(0 < length <= limits.block_bytes and 0 < count <= length // 52,
                "invalid block length or record count")
        require(payload_bytes + length <= limits.payload_bytes, "raw-payload limit exceeded")
        payload = reader.exact(length)
        require(hashlib.sha256(payload).digest() == digest, "block payload hash mismatch")
        offset = 0
        for _ in range(count):
            require(offset + 4 <= length, "truncated record length")
            size = struct.unpack_from("<I", payload, offset)[0]
            require(RECORD.size <= size <= limits.record_bytes - 4 and offset + 4 + size <= length,
                    "invalid or truncated record length")
            op, site, file, domain, reserved, a, b, c, d = RECORD.unpack_from(payload, offset + 4)
            blob = memoryview(payload)[offset + 4 + RECORD.size:offset + 4 + size] if size > RECORD.size else b""
            observations.event(op, site, file, domain, reserved, (a, b, c, d), blob)
            offset += 4 + size
        require(offset == length, "block has uncounted records or trailing payload")
        blocks += 1
        records += count
        payload_bytes += length
        hashes.update(digest)
        if progress is not None and records >= next_progress:
            progress({"records": records, "blocks": blocks, "files": observations.totals["files"],
                      "payload_bytes": payload_bytes, "compressed_bytes": reader.compressed_bytes})
            next_progress = (records // progress_records + 1) * progress_records
    ordered = lambda counts: {str(key): value for key, value in sorted(counts.items())}
    return {"version": 1, "format": MAGIC.decode(), "complete": True,
            "blocks": blocks, "records": records, "payload_bytes": payload_bytes,
            "compressed_bytes": reader.compressed_bytes,
            "compressed_sha256": reader.compressed_sha.hexdigest(), "stream_sha256": reader.stream_sha.hexdigest(),
            "block_hashes_sha256": hashes.hexdigest(), "totals": observations.totals,
            "by_op": ordered(observations.by_op), "by_site": ordered(observations.by_site),
            "by_op_site": {str(op): ordered(counts) for op, counts in sorted(observations.by_op_site.items())},
            "by_domain": ordered(observations.by_domain), "by_file": ordered(observations.by_file),
            "by_op_shape": {str(op): dict(sorted(counts.items()))
                            for op, counts in sorted(observations.by_op_shape.items())},
            "binder_operations": dict(observations.binder), "files": observations.files,
            "full_semantic_replay": False, "unobserved_operation_count": None,
            "scope": "Framing, integrity, phase/physical-header/blob-count checks and named kind/flag checks for mapped initial headers; no hardware, list-value replay or complete payload coverage"}


def verify_file(path, registries, max_payload=32 * 1024**3, max_compressed=3 * 1024**3, **options):
    documents = [strict_json(Path(value).read_bytes()) if isinstance(value, (str, Path)) else value
                 for value in registries]
    events = registry(documents)
    limits = Limits(payload_bytes=max_payload, compressed_bytes=max_compressed,
                    files=options.pop("max_files", Limits.files),
                    active_nodes=options.pop("max_active_nodes", Limits.active_nodes))
    with Path(path).open("rb") as source:
        return verify(source, events, limits=limits, **options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path)
    parser.add_argument("--registry", type=Path, nargs=3, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--max-payload-bytes", type=int, default=Limits.payload_bytes)
    parser.add_argument("--max-compressed-bytes", type=int, default=Limits.compressed_bytes)
    parser.add_argument("--max-files", type=int, default=Limits.files)
    parser.add_argument("--max-active-nodes", type=int, default=Limits.active_nodes)
    parser.add_argument("--progress-records", type=int, default=1_000_000)
    parser.add_argument("--allow-empty", action="store_true", help="writer-unit fixtures only, not an actual capture")
    for name in ("files", "source-bytes", "nodes", "symbols"):
        parser.add_argument("--expected-" + name, type=int)
    args = parser.parse_args()
    events = registry(strict_json(path.read_bytes()) for path in args.registry)
    expected = {name: getattr(args, "expected_" + name) for name in ("files", "source_bytes", "nodes", "symbols")
                if getattr(args, "expected_" + name) is not None}
    with args.trace.open("rb") as source:
        result = verify(source, events, limits=Limits(payload_bytes=args.max_payload_bytes,
                        compressed_bytes=args.max_compressed_bytes, files=args.max_files,
                        active_nodes=args.max_active_nodes),
                        allow_empty=args.allow_empty, expected=expected,
                        progress=lambda value: print(json.dumps({"verify_progress": value}), file=sys.stderr, flush=True),
                        progress_records=args.progress_records)
    raw = json.dumps(result, indent=2, sort_keys=True, allow_nan=False) + "\n"
    if args.output:
        args.output.write_text(raw)
    else:
        print(raw, end="")


if __name__ == "__main__":
    main()
