"""Supplemental requests frozen independently of primary E1 eligibility."""

from s06_codec_fixtures import document as codec_document
from collections import Counter
import struct

from s06_protocol import (FACTORY_SCENARIOS, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
                          OBSERVATION_FIELDS, RETURNED_ERROR_STAGES, canonical, hex_bytes, parser_request, sha256, validate_request)

REGRESSIONS = {
    "TestHeritageClauseElementKinds": (".ts", "heritage-node-kinds"),
    "TestJSDocImportTypeParentChain": (".js", "jsdoc-import-parents"),
    "TestJSDocTypeSourceSurvivesReparse": (".js", "jsdoc-source-survives-reparse"),
    "TestJSDocTypeSourcePropagatesToConstructedReparse": (".js", "jsdoc-source-propagation"),
    "TestSourceFilePositionMapWithNonASCIIStringLiteral": (".ts", "nonascii-position-map"),
}


def fixture_documents(native, pin, factory):
    if native["kind_first"] != -32768 or native["kind_count"] != 351 or len(native["kind_names"]) != 65536:
        raise ValueError("signed kind observation domain changed")
    if len(native["fuzz_seeds"]) != 10 or {x["name"] for x in native["parser_regressions"]} != set(REGRESSIONS):
        raise ValueError("pinned parser seed/regression inventory changed")
    fixtures = []

    def add(request, group, provenance, **metadata):
        validate_request(request)
        fixtures.append({"request": request, "group": group, "provenance": provenance,
                         "request_sha256": sha256(canonical(request)), **metadata})

    def parse(identifier, source, group="ast_runtime", extension=".ts", kind=None, **options):
        kind = kind if kind is not None else {".ts": 3, ".js": 1, ".tsx": 4, ".json": 6}[extension]
        filename = "/s06/fixture" + extension
        return parser_request(identifier, None, source, filename, filename, kind, **options)

    add({"version": 1, "id": "ast/kind-stringer/all-i16", "primary": None, "op": "kind_names", "first": -32768, "count": 65536},
        "ast_runtime", "ast.Kind.String over every signed i16 value", expected_sha256=sha256(canonical(native["kind_names"])))
    for index, path in enumerate((
        b"", b"/", b"//", b"/a//b", b"/a/./b", b"/a/../b", b"a/../../b/",
        b"C:", b"C:foo", b"C:/foo/../bar", b"c:\\foo\\bar", b"//server/share/../x",
        b"file:///C%3a/../x", b"file:///c:/a/../b", b"https://host/a/../b", b"^/untitled",
        b"x.d.ts", b"x.D.TS", b"x.d.css.ts", b"x.d.mts", b"x.d.cts", b"x.d.ts/",
        b"/a/\xff/../b", b"\xed\xa0\x80.d.ts", b"/a\x00b.ts",
    )):
        add({"version": 1, "id": f"path/boundary/{index}", "primary": None, "op": "path", "path_hex": path.hex()},
            "ast_runtime", "tspath.GetEncodedRootLength/NormalizePath/IsDeclarationFileName direct raw-string boundary")
    for seed in native["fuzz_seeds"]:
        name = "upstream/fuzz/" + seed["name"]
        request = parse(name, hex_bytes(seed["source_hex"], name), extension=seed["extension"], jsx=seed["jsx"], force=seed["force"])
        add(request, "parser_regression", "tsc/internal/parser/testdata/fuzz/FuzzParser/" + seed["name"], source_sha256=seed["raw_sha256"])
    for regression in native["parser_regressions"]:
        extension, name = REGRESSIONS[regression["name"]]
        add(parse("upstream/regression/"+name, hex_bytes(regression["source_hex"], name), extension=extension),
            "parser_regression", "tsc/internal/parser/parser_test.go:"+regression["name"])
    for kind in (-(2**31), -1, 0, 1, 2, 3, 4, 6, 7, 2**31-1):
        add(parse(f"parser/open-script-kind/{kind}", b"const x = <a/>;", kind=kind), "ast_runtime", "parser.initializeState open ScriptKind boundary")
    for name, source in (
        ("utf8-bom", b"\xef\xbb\xbflet x;"), ("double-utf8-bom", b"\xef\xbb\xbf\xef\xbb\xbflet x;"),
        ("raw-utf16le-prefix", b"\xff\xfeA\x00Z"), ("raw-utf16be-prefix", b"\xfe\xff\x00AZ"),
        ("raw-surrogate", b'const x="\xed\xa0\x80";'), ("raw-literal-byte", b'const x="\xff";'),
        ("split-keyword-constructor", b"constructorabcdefghij x;"),
        ("split-keyword-typeof", b"typeofabcdefghijklmno x;"),
    ):
        add(parse("parser/text/"+name, source), "ast_runtime", "reviewed raw parser text; no file decoder at adapter entry")
    # Native JSON uses a generic Token payload for these keyword kinds, unlike
    # the KeywordExpression payload used in ordinary TypeScript parsing.
    for literal in (b"true", b"false", b"null"):
        add(parse("codec/json-token/"+literal.decode(), literal, extension=".json"),
            "encoder", "parser.parseJSONText constructs fieldless keyword kinds with NewToken")
    if factory["pin"] != pin or tuple(case["scenario"] for case in factory["cases"]) != FACTORY_SCENARIOS:
        raise ValueError("factory scenario inventory or pin changed")
    for case in factory["cases"]:
        if case["stages"] != ["factory"]:
            raise ValueError("factory scenario stages changed")
        add(case["request"], "ast_runtime", "data/s06/factory-fixtures.json", steps_sha256=sha256(canonical(case["steps"])))
    # Additive regressions from independent Go/JSDoc review; no original request removed.
    for name, source in (('4', b'/**\n * @callback {Object} A.B.C {@link A.B text}\n * @typedef {?string=} [x] description\n * @type {function(this:T,x:number):string} [x=42] description\n * @typedef { [x] description\n */\nfunction f(x,y) {}'), ('34', b'/**\n * @type {...Object}  ``` @see ignore ```\n */\nfunction f(x,y) {}'), ('169', b'/**\n * @private {(A|B)[]} [x] {@linkcode this.#x}\n * @type {Object} `x` description\n */\nfunction f(x,y) {}')):
        add(parse("encoder/full-signature/"+name, source, extension=".js"), "encoder",
            "reviewed Go NodeVisitor FullSignature traversal; wire mask deliberately omits this Go-only edge")
    depth = 20000
    deep = {
        "parentheses": (b"("*depth+b"0"+b")"*depth, ".ts"),
        "unary": (b"!"*depth+b"x;", ".ts"),
        "binary-left": (b"x+"*depth+b"x;", ".ts"),
        "assignment-right": (b"a="*depth+b"x;", ".ts"),
        "new": (b"new "*depth+b"X;", ".ts"),
        "qualified-namespace": (b"namespace "+b"N."*(depth-1)+b"N {}", ".ts"),
        "jsx": (b"<a>"*depth+b"x"+b"</a>"*depth, ".tsx"),
        "json": (b"["*depth+b"0"+b"]"*depth, ".json"),
        "json-unclosed": (b"["*depth+b"0", ".json"),
        "jsdoc-type": (b"/** @type {"+b"?"*depth+b"number} */ const x=0;", ".js"),
    }
    for name, (source, extension) in deep.items():
        add(parse("depth/"+name, source, extension=extension), "depth", "reviewed depth constructor", depth=depth)
    constants = native["wire_constants"]
    if (constants["version"], constants["header_size"], constants["node_size"]) != (8, 44, 28):
        raise ValueError("protocol-8 wire constants changed")

    def wire(kind, data=0, children=False, pos=0, end=0):
        words = [8 << 24, 0, 0, 0, 0, 0, 44, 44, 44, 44, 44] + [0]*7
        words += [kind, pos, end, 0, 0, data, 0]
        if children:
            words += [constants["comma_token"], 0, 0, 0, 1, 0, 0]
        return struct.pack("<"+"I"*len(words), *words)

    def decode(name, data, entrypoint="nodes", **metadata):
        add({"version": 1, "id": "decode/"+name, "primary": None, "op": "decode", "wire_hex": data.hex(), "entrypoint": entrypoint},
            "decoder", "pinned protocol-8 raw wire boundary", **metadata)

    for length in (0, 1, 43):
        decode(f"short-header/{length}", b"\0"*length)
    for kind in (0, 351, 0xffff, 0x10000, 0x10001, 0x8000, 0xffffffff):
        decode(f"raw-kind/{kind:08x}", wire(kind))
    for name in ("type_alias_declaration", "js_type_alias_declaration", "import_declaration", "js_import_declaration"):
        decode("shared-payload-kind/"+name, wire(constants[name]))
    # Decode widens the wire words before NewTextRange narrows its stored i32
    # fields; the actual observed positions are (-2147483648, -1).
    decode("position-high-bits", wire(0, pos=0x80000000, end=0xffffffff))
    decode("reserved-data-tag", wire(0, 0xc0000000))
    decode("nil-root/source-file", wire(0xffffffff), "source_file", paired_request="decode/raw-kind/ffffffff")
    decode("synthetic-expression", wire(constants["synthetic_expression"]))
    for name in ("syntax_list", "jsdoc_type_literal"):
        decode("generated-list/"+name, wire(constants[name], children=True))
    valid = wire(0)
    decode("trailing-incomplete-node", valid+b"\xff"*27)
    decode("unsupported-version", valid[:3]+b"\x07"+valid[4:])
    for name, offset, value in (("offset-past-end", 24, len(valid)+1), ("offset-not-monotonic", 24, 45),
                                ("ignored-structured-offset", 36, 0xffffffff)):
        data = bytearray(valid); struct.pack_into("<I", data, offset, value)
        decode(name, bytes(data))
    codecs = codec_document(pin)
    for case in codecs["cases"]:
        add(case["request"], "codec", "data/s06/codec-fixtures.json", steps_sha256=sha256(canonical(case["steps"])))
    ids = [item["request"]["id"] for item in fixtures]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate supplemental request")
    documents = {
        "codec-fixtures.json": codecs,
        "fixtures.json": {"version": 1, "pin": pin, "primary_rows_contributed": 0, "fixtures": fixtures},
        "node-kind-observations.json": {"version": 1, "pin": pin, "first": -32768, "names": native["kind_names"]},
        "legacy-option-observations.json": {"version": 1, "pin": pin, "observations": native["legacy_module_none"]},
        "protocol.json": {"version": 1, "request_max_bytes": MAX_REQUEST_BYTES,
                          "response_max_bytes": MAX_RESPONSE_BYTES, "chunk_max_bytes": 65536,
                          "source_contract": "source_hex contains finalized parser bytes; never decode at parser adapter entry",
                          "parse_stages": ["parse", "node_index_before", "encode_source_file", "node_index_after"],
                          "decode_stages": ["decode", "decoded_tree"],
                          "other_stages": {"kind_names": ["kind_names"], "path": ["path"], "factory": ["factory"], "codec": ["encode", "decode", "decoded_tree", "reencode"]},
                          "frame_fields": {"begin": "version id tag op", "observation": "version id tag seq stage kind value",
                                           "stage": "version id tag stage outcome message_hex", "end": "version id tag observations stages"},
                          "observation_fields": OBSERVATION_FIELDS,
                          "identity_fields": {"node_index_before": "independent_order_equal cache_reused", "node_index_after": "before_reused encoded_reused"},
                          "primary_rows": 12829, "supplemental_rows_contributed": 0,
                          "script_kind": "signed i32; zero is an observed parser panic, unknown nonzero values remain valid requests",
                          "framing": "one begin record, ordered stage observations, one end with completed-stage count; panic or returned error ends the request at its actual stage",
                          "behavior_outcomes": ["ok", "error", "panic"],
                          "returned_error_stages": list(RETURNED_ERROR_STAGES),
                          "invalid_capture": ["malformed/duplicate/unknown fields", "wrong identity/order/count", "truncated record", "timeout", "oversized record", "unexpected child exit"],
                          "definitions": "scripts/s06_protocol.py is the executable strict request schema"},
    }
    inventory = {"requests": len(fixtures), "groups": dict(sorted(Counter(item["group"] for item in fixtures).items())),
                 "sha256": sha256(canonical(fixtures)), "kind_stringer_observations": 65536}
    return documents, inventory
