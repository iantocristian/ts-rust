"""S06 direct parser request contract; no file decoding occurs at this boundary."""

import hashlib
import json
import re

from s06_codec_fixtures import CODEC_SCENARIOS, CODEC_STAGES

VERSION = 1
MAX_REQUEST_BYTES = 16 * 1024 * 1024
REQUEST_FIELDS = {"version", "id", "primary", "op", "source_hex", "filename", "path",
                  "script_kind", "jsx", "force", "operations"}
PRIMARY_OPERATIONS = ["parse", "node_index_before", "encode_source_file", "node_index_after"]
FACTORY_SCENARIOS = (
    "hooks-counts-update-clone", "raw-slice-same", "visitor-nil-flatten-disable",
    "visitor-lift-contracts", "visitor-role-hooks", "deep-clone-locations-and-parents",
    "subtree-cache-and-exclusions", "token-subtree-and-precedence",
    "source-file-clone-omissions", "source-cache-panic-once", "node-index-nil-after-sort",
    "source-file-hooks",
)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def exact_keys(value, fields, context):
    if type(value) is not dict or set(value) != set(fields):
        raise ValueError(f"{context}: missing or unexpected fields")


def text(value, context, *, empty=True):
    if type(value) is not str or (not empty and not value):
        raise ValueError(f"{context}: expected {'nonempty ' if not empty else ''}string")
    try:
        value.encode("utf-8")
    except UnicodeEncodeError as error:
        raise ValueError(f"{context}: JSON strings must contain Unicode scalar values") from error
    return value


def hex_bytes(value, context):
    text(value, context)
    if len(value) % 2 or re.fullmatch(r"[0-9a-f]*", value) is None:
        raise ValueError(f"{context}: expected canonical lowercase byte hex")
    return bytes.fromhex(value)


def integer(value, low, high, context):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"{context}: integer outside [{low},{high}]")
    return value


def validate_request(request):
    if type(request) is not dict:
        raise ValueError("request must be an object")
    if request.get("op") == "codec":
        exact_keys(request, {"version", "id", "primary", "op", "scenario"}, "codec request")
        integer(request["version"], VERSION, VERSION, "version")
        text(request["id"], "id", empty=False)
        if request["primary"] is not None or request["scenario"] not in CODEC_SCENARIOS:
            raise ValueError("unknown codec scenario or nonnull primary")
        if len(canonical(request)) > MAX_REQUEST_BYTES:
            raise ValueError("codec request exceeds byte limit")
        return
    if request.get("op") == "factory":
        exact_keys(request, {"version", "id", "primary", "op", "scenario"}, "factory request")
        integer(request["version"], VERSION, VERSION, "version")
        text(request["id"], "id", empty=False)
        if request["primary"] is not None or request["scenario"] not in FACTORY_SCENARIOS:
            raise ValueError("unknown factory scenario or nonnull primary")
        if len(canonical(request)) > MAX_REQUEST_BYTES:
            raise ValueError("factory request exceeds byte limit")
        return
    if request.get("op") == "path":
        exact_keys(request, {"version", "id", "primary", "op", "path_hex"}, "path request")
        integer(request["version"], VERSION, VERSION, "version")
        text(request["id"], "id", empty=False)
        if request["primary"] is not None:
            raise ValueError("supplemental paths cannot contribute primary rows")
        hex_bytes(request["path_hex"], "path bytes")
        if len(canonical(request)) > MAX_REQUEST_BYTES:
            raise ValueError("path request exceeds byte limit")
        return
    if request.get("op") == "kind_names":
        exact_keys(request, {"version", "id", "primary", "op", "first", "count"}, "kind-name request")
        integer(request["version"], VERSION, VERSION, "version")
        text(request["id"], "id", empty=False)
        if request["primary"] is not None:
            raise ValueError("supplemental kind names cannot contribute primary rows")
        first = integer(request["first"], -32768, 32767, "first kind")
        integer(request["count"], 1, 32768-first, "kind count")
        return
    if request.get("op") == "decode":
        exact_keys(request, {"version", "id", "primary", "op", "wire_hex", "entrypoint"}, "decode request")
        integer(request["version"], VERSION, VERSION, "version")
        text(request["id"], "id", empty=False)
        if request["primary"] is not None:
            raise ValueError("supplemental decoding cannot contribute primary rows")
        hex_bytes(request["wire_hex"], "wire bytes")
        if request["entrypoint"] not in ("nodes", "source_file"):
            raise ValueError("unknown decoder entry point")
        if len(canonical(request)) > MAX_REQUEST_BYTES:
            raise ValueError("decode request exceeds byte limit")
        return
    exact_keys(request, REQUEST_FIELDS, "parser request")
    integer(request["version"], VERSION, VERSION, "version")
    if request["op"] != "parse":
        raise ValueError("unsupported request operation")
    text(request["id"], "id", empty=False)
    if request["primary"] is not None:
        text(request["primary"], "primary", empty=False)
    hex_bytes(request["source_hex"], "source_hex")
    text(request["filename"], "filename")
    text(request["path"], "path")
    integer(request["script_kind"], -(2**31), 2**31-1, "script_kind")
    for key in ("jsx", "force"):
        if type(request[key]) is not bool:
            raise ValueError(f"{key}: expected boolean")
    if request["operations"] != PRIMARY_OPERATIONS:
        raise ValueError("unsupported parser operation sequence")
    if len(canonical(request)) > MAX_REQUEST_BYTES:
        raise ValueError("parser request exceeds frozen byte limit")


def parser_request(identifier, primary, source, filename, path, script_kind=3, jsx=False, force=False):
    request = {"version": VERSION, "id": identifier, "primary": primary, "op": "parse",
               "source_hex": source.hex(), "filename": filename, "path": path,
               "script_kind": script_kind, "jsx": jsx, "force": force,
               "operations": list(PRIMARY_OPERATIONS)}
    validate_request(request)
    return request

# Stream framing validates each adapter independently before comparing outcomes.
MAX_RESPONSE_BYTES = 1024 * 1024
# Only these pinned API calls return an error. A matching invented error from
# parse, traversal or factory work must not skip their unmeasured obligations.
RETURNED_ERROR_STAGES = ("decode", "encode_source_file", "encode", "reencode")
OBSERVATION_FIELDS = {
    "kind": "raw name", "path": "encoded_root_length normalized_hex declaration_file",
    "source_file": "kind pos end flags node_count text_count identifier_count script_kind language_variant declaration_file hash",
    "diagnostic": "collection index code category pos end key_hex text_hex args_hex",
    "table": "length", "node": "index node", "absent_lookup": "index",
    "bytes": "length", "chunk": "offset hex", "root": "node",
    "tree_node": "index kind pos end flags children",
    "factory": "label ints bools strings",
}


def stages_for(request):
    if request["op"] == "codec": return CODEC_STAGES
    return PRIMARY_OPERATIONS if request["op"] == "parse" else (["decode", "decoded_tree"] if request["op"] == "decode" else [request["op"]])


def validate_node(value, *, indexed):
    if value is None:
        return
    fields = "kind pos end flags parent lookup" if indexed else "kind pos end flags"
    exact_keys(value, fields.split(), "node observation")
    integer(value["kind"], -32768, 32767, "node kind")
    for key in ("pos", "end"):
        integer(value[key], -(2**31), 2**31-1, key)
    for key in ("flags", "parent", "lookup"):
        if key in value:
            integer(value[key], 0, 2**32-1, key)


class StreamCase:
    def __init__(self, request):
        validate_request(request)
        self.request = request
        self.stages = stages_for(request)
        self.stage_index = self.ordinal = 0
        self.started = self.ended = self.stopped = False
        self.counts = {}
        self.table_length = self.byte_length = None
        self.bytes_received = 0
        self.diagnostic_indices = {}
        self.outcomes = []
        self.tree_pending = 0

    def accept(self, record):
        if type(record) is not dict or self.ended:
            raise ValueError("invalid frame or frame after end")
        if record.get("version") != 1 or type(record["version"]) is not int or record.get("id") != self.request["id"]:
            raise ValueError("wrong response version or identity")
        tag = record.get("tag")
        if not self.started:
            exact_keys(record, "version id tag op".split(), "begin frame")
            if tag != "begin" or record["op"] != self.request["op"]:
                raise ValueError("missing or wrong begin frame")
            self.started = True
            return
        if tag == "end":
            exact_keys(record, "version id tag observations stages".split(), "end frame")
            integer(record["observations"], 0, 2**63-1, "observation count")
            integer(record["stages"], 0, len(self.stages), "stage count")
            if record["observations"] != self.ordinal or record["stages"] != self.stage_index or self.counts or not self.stopped and self.stage_index != len(self.stages):
                raise ValueError("missing stages/observations or inconsistent end counts")
            self.ended = True
            return
        if self.stopped or self.stage_index >= len(self.stages) or record.get("stage") != self.stages[self.stage_index]:
            raise ValueError("wrong stage order or work after failure")
        stage = self.stages[self.stage_index]
        if tag == "stage":
            exact_keys(record, "version id tag stage outcome message_hex".split(), "stage frame")
            outcome = record["outcome"]
            message = hex_bytes(record["message_hex"], "stage message")
            if outcome not in ("ok", "error", "panic") or outcome == "ok" and message:
                raise ValueError("invalid stage outcome")
            if outcome == "error" and stage not in RETURNED_ERROR_STAGES:
                raise ValueError(f"{stage} cannot return an error in the pinned API")
            if outcome == "ok":
                self.complete_stage(stage)
            self.outcomes.append((stage, outcome, record["message_hex"]))
            self.stopped = outcome != "ok"
            self.stage_index += 1
            self.counts = {}; self.table_length = self.byte_length = None
            self.bytes_received = 0; self.diagnostic_indices = {}
            return
        exact_keys(record, "version id tag seq stage kind value".split(), "observation frame")
        if tag != "observation" or type(record["seq"]) is not int or record["seq"] != self.ordinal:
            raise ValueError("wrong observation sequence")
        self.observation(stage, record["kind"], record["value"])
        self.ordinal += 1

    def observation(self, stage, kind, value):
        allowed = {"parse": {"source_file", "diagnostic"}, "node_index_before": {"identity", "table", "node", "absent_lookup"},
                   "node_index_after": {"identity", "table", "node", "absent_lookup"}, "encode_source_file": {"bytes", "chunk"},
                   "kind_names": {"kind"}, "path": {"path"}, "decode": {"root"}, "decoded_tree": {"tree_node"}, "factory": {"factory"}, "encode": {"bytes", "chunk"}, "reencode": {"bytes", "chunk"}}
        if type(kind) is not str or kind not in allowed[stage]:
            raise ValueError("unknown observation or observation in wrong stage")
        previous = self.counts.get(kind, 0)
        self.counts[kind] = previous+1
        fields = OBSERVATION_FIELDS.get(kind)
        if kind == "identity":
            fields = "independent_order_equal cache_reused" if stage == "node_index_before" else "before_reused encoded_reused"
        exact_keys(value, fields.split(), "observation payload")
        bool_fields = {"declaration_file", "independent_order_equal", "cache_reused", "before_reused", "encoded_reused"}
        text_fields = {"name", "collection", "hash", "label"}
        for key, item in value.items():
            if key in bool_fields:
                if type(item) is not bool: raise ValueError("nonboolean observation")
            elif key.endswith("_hex") or key == "hex":
                if key == "args_hex":
                    if type(item) is not list: raise ValueError("nonarray diagnostic arguments")
                    for arg in item: hex_bytes(arg, "diagnostic argument")
                else: hex_bytes(item, key)
            elif key in text_fields:
                text(item, key)
            elif kind == "factory" and key in ("ints", "bools", "strings"):
                if type(item) is not list: raise ValueError("nonarray factory witness")
                for element in item:
                    if key == "ints": integer(element, -(2**63), 2**63-1, "factory integer")
                    elif key == "strings": text(element, "factory string")
                    elif type(element) is not bool: raise ValueError("nonboolean factory witness")
            elif key == "node":
                validate_node(item, indexed=kind == "node")
            else:
                integer(item, -(2**63), 2**63-1, key)
        if kind in ("identity", "table", "bytes", "source_file", "absent_lookup", "path", "root") and previous:
            raise ValueError("duplicate singular observation")
        if kind == "table":
            self.table_length = integer(value["length"], 1, 2**32-1, "table length")
        elif kind == "node":
            if self.table_length is None or value["index"] != previous or previous >= self.table_length:
                raise ValueError("missing table or wrong node ordinal")
            if previous == 0 and value["node"] is not None:
                raise ValueError("node table lacks nil sentinel")
        elif kind == "bytes":
            self.byte_length = integer(value["length"], 0, 2**32-1, "encoded byte length")
        elif kind == "chunk":
            data = hex_bytes(value["hex"], "encoded chunk")
            if self.byte_length is None or value["offset"] != self.bytes_received or not data or len(data) > 65536:
                raise ValueError("missing byte header, empty chunk or wrong byte offset")
            self.bytes_received += len(data)
            if self.bytes_received > self.byte_length: raise ValueError("extra encoded bytes")
        elif kind == "diagnostic":
            collection = value["collection"]
            if collection not in ("parse", "js", "jsdoc") or value["index"] != self.diagnostic_indices.get(collection, 0):
                raise ValueError("wrong diagnostic collection/ordinal")
            self.diagnostic_indices[collection] = value["index"]+1
        elif kind == "kind":
            if value["raw"] != self.request["first"]+previous or previous >= self.request["count"]:
                raise ValueError("wrong kind domain/ordinal")
        elif kind == "tree_node" and value["index"] != previous:
            raise ValueError("wrong decoded tree ordinal")
        if kind == "root":
            self.tree_pending = int(value["node"] is not None)
        elif kind == "tree_node":
            if self.tree_pending < 1 or value["children"] < 0:
                raise ValueError("extra reconstructed tree node")
            self.tree_pending += value["children"]-1

    def complete_stage(self, stage):
        count = self.counts.get
        if stage == "parse" and count("source_file", 0) != 1:
            raise ValueError("successful parser omitted source-file state")
        if stage.startswith("node_index_") and (count("identity", 0) != 1 or count("table", 0) != 1 or count("absent_lookup", 0) != 1 or count("node", 0) != self.table_length):
            raise ValueError("successful node-index stage omitted members")
        if stage in ("encode_source_file", "encode", "reencode") and (count("bytes", 0) != 1 or self.bytes_received != self.byte_length):
            raise ValueError("successful encoder omitted bytes")
        if stage == "kind_names" and count("kind", 0) != self.request["count"]:
            raise ValueError("successful stringer omitted kind values")
        if stage == "path" and count("path", 0) != 1 or stage == "decode" and count("root", 0) != 1:
            raise ValueError("successful operation omitted its result")
        if stage == "decoded_tree" and self.tree_pending:
            raise ValueError("reconstructed tree omitted descendants")
        if stage == "factory" and not count("factory", 0):
            raise ValueError("successful factory scenario omitted witnesses")
