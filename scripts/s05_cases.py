"""Version-one S05 requests. Frozen before the scanner behavior implementation."""

from collections import Counter
import hashlib
import json
from pathlib import Path
import random

from s04_common import command, strict_json_loads

ROOT = Path(__file__).resolve().parents[1]
TARGETS = tuple(range(13)) + (99, 100)
TOKEN_OPS = {
    "scan", "scan_all", "rescan_less_than", "rescan_greater_than", "rescan_asterisk_equals",
    "rescan_hash", "rescan_question", "rescan_template", "rescan_slash", "rescan_jsx",
    "scan_jsx", "scan_jsx_ex", "scan_jsx_identifier", "scan_jsx_attribute",
    "rescan_jsx_attribute", "scan_jsdoc", "scan_jsdoc_text", "snapshot",
}
# Each operation has exactly these argument fields, in addition to op.
ACTION_FIELDS = {op: {} for op in TOKEN_OPS | {
    "reset", "mark", "rewind", "commit", "can_follow_jsdoc_at", "identifier_token",
    "valid_identifier", "intrinsic_jsx_name", "string_to_token", "keyword_suggestions",
    "shebang", "normalize_jsdoc", "number_from_string", "pseudo_bigint",
}}
for op in ("rescan_template", "rescan_jsx", "scan_jsx_ex", "scan_jsdoc_text", "set_skip_trivia",
           "set_skip_jsdoc_asterisks", "set_on_error"):
    ACTION_FIELDS[op] = {"flag": bool}
ACTION_FIELDS.update({
    "rescan_slash": {"report_errors": str},
    "set_text": {"text_hex": str}, "reset_pos": {"pos": int}, "reset_token_state": {"pos": int},
    "set_variant": {"value": int}, "set_target": {"value": int}, "observe": {"getter": str},
    "identifier_block": {"first": int, "count": int}, "identifier_point": {"point": int},
    "identifier_text": {"variant": int}, "equal_fold": {"other_hex": str},
    "token_to_string": {"kind": int}, "number_format": {"bits": str},
    "skip_trivia": {"pos": int, "options": bool, "stop_after_line_break": bool,
                    "stop_at_comments": bool, "in_jsdoc": bool},
    "comment_ranges": {"pos": int, "trailing": bool},
})
GETTERS = {"text", "token", "flags", "full_start", "start", "end", "token_text", "value", "range", "directives", "predicates"}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def hex_bytes(value):
    if type(value) is not str or len(value) % 2 or any(ch not in "0123456789abcdef" for ch in value):
        raise ValueError("bytes must be canonical lowercase hexadecimal")
    return bytes.fromhex(value)


def validate_request(request):
    fields = {"version", "id", "source_hex", "decode_source", "target", "variant", "skip_trivia", "actions"}
    if type(request) is not dict or set(request) != fields:
        raise ValueError("invalid request fields")
    if type(request["version"]) is not int or request["version"] != 1:
        raise ValueError("unknown request version")
    if type(request["id"]) is not str or not request["id"] or len(request["id"]) > 1024:
        raise ValueError("invalid request identity")
    if type(request["target"]) is not int or request["target"] not in TARGETS:
        raise ValueError("invalid script target")
    if type(request["variant"]) is not int or request["variant"] not in (0, 1):
        raise ValueError("invalid language variant")
    if type(request["skip_trivia"]) is not bool or type(request["decode_source"]) is not bool:
        raise ValueError("invalid request boolean")
    if len(hex_bytes(request["source_hex"])) > 4 * 1024 * 1024:
        raise ValueError("source exceeds the frozen 4 MiB limit")
    actions = request["actions"]
    if type(actions) is not list or not 1 <= len(actions) <= 4096:
        raise ValueError("invalid action count")
    marks = 0
    for action in actions:
        if type(action) is not dict or type(action.get("op")) is not str or action["op"] not in ACTION_FIELDS:
            raise ValueError("unknown action")
        op = action["op"]
        expected = ACTION_FIELDS[op]
        if set(action) != {"op", *expected} or any(type(action[key]) is not kind for key, kind in expected.items()):
            raise ValueError(f"invalid {op} action fields/types")
        for key in ("text_hex", "other_hex"):
            if key in action and len(hex_bytes(action[key])) > 4 * 1024 * 1024:
                raise ValueError("action source exceeds 4 MiB")
        for key, value in action.items():
            if type(value) is int and not -(1 << 63) <= value < 1 << 63:
                raise ValueError("integer outside signed Go-int range")
        if op == "rescan_slash" and action["report_errors"] not in ("omitted", "false", "true"):
            raise ValueError("invalid slash reporting mode")
        if op == "observe" and action["getter"] not in GETTERS:
            raise ValueError("unknown getter")
        if op in ("set_variant", "identifier_text") and action.get("value", action.get("variant")) not in (0, 1):
            raise ValueError("invalid action variant")
        if op == "set_target" and action["value"] not in TARGETS:
            raise ValueError("invalid action target")
        if op == "identifier_block" and not (0 <= action["first"] <= 0x10FFFF and 1 <= action["count"] <= 4096 and action["first"] + action["count"] <= 0x110000):
            raise ValueError("invalid identifier block")
        if op == "identifier_point" and not -(1 << 31) <= action["point"] < 1 << 31:
            raise ValueError("point outside Go rune range")
        if op == "token_to_string" and not 0 <= action["kind"] < 351:
            raise ValueError("unknown syntax kind")
        if op == "number_format" and len(hex_bytes(action["bits"])) != 8:
            raise ValueError("number bits must encode eight big-endian bytes")
        if op == "mark":
            marks += 1
        elif op in ("rewind", "commit"):
            marks -= 1
            if marks < 0:
                raise ValueError("checkpoint stack underflow")
    if marks:
        raise ValueError("unclosed checkpoints")
    return request


def action(op, **args):
    return {"op": op, **args}


def case(case_id, source=b"", actions=None, groups=("scanner", "diagnostics", "values"), **options):
    if type(source) is str:
        source = source.encode()
    request = {"version": 1, "id": case_id, "source_hex": source.hex(), "decode_source": False,
               "target": 0, "variant": 0, "skip_trivia": True,
               "actions": [action("scan_all")] if actions is None else actions}
    request.update(options)
    validate_request(request)
    return {"request": request, "groups": sorted(groups), "witness": None, "expected_panic": None}


def corpus_inventory(upstream):
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    listing = command(["git", "ls-tree", "-rz", "--format=%(objectmode) %(path)", pin, "--",
                       "tsc/testdata/tests/cases", "tsc/internal/bundled/libs"], cwd=upstream)
    result = []
    for entry in listing.split(b"\0"):
        if not entry:
            continue
        mode, name = entry.decode().split(" ", 1)
        path = upstream / name
        if path.suffix not in (".ts", ".tsx", ".js", ".jsx"):
            continue
        if mode not in ("100644", "100755") or path.is_symlink():
            raise ValueError(f"unsupported pinned corpus entry: {name}")
        content = path.read_bytes()
        result.append({"path": name, "sha256": hashlib.sha256(content).hexdigest(),
                       "bytes": len(content), "variant": int(path.suffix in (".tsx", ".jsx"))})
    return sorted(result, key=lambda row: row["path"])


def fixtures(upstream, tables):
    result = []
    def add(*args, **kwargs):
        item = case(*args, **kwargs)
        result.append(item)
        return item
    # Independently selected byte contexts; delimiter bytes are intentionally included.
    for byte in range(256):
        for context, prefix, suffix in (("bare", b"", b""), ("double", b'"', b'"'),
                                        ("single", b"'", b"'"), ("template", b"`", b"`"),
                                        ("comment", b"/*", b"*/")):
            for skip in (False, True):
                add(f"byte/{byte:02x}/{context}/{int(skip)}", prefix + bytes([byte]) + suffix, skip_trivia=skip)
    special = [b"\xc0\xaf", b"\xe2\x82", b"\xf0\x9f\x98", b"\xff\xfe", b"\xed\xa0\x80",
               b"\xed\xb0\x80", b"\xef\xbf\xbd", "🦀".encode(), "\u2028\u2029".encode()]
    for index, text in enumerate(special):
        for context, prefix, suffix in (("bare", b"", b""), ("string", b'"', b'"'), ("comment", b"//", b"\n"), ("template", b"`", b"`")):
            add(f"bytes/{index}/{context}", prefix + text + suffix)
    for name, text in (("utf8bom", b"\xef\xbb\xbfconst x=1"), ("utf16le", b"\xff\xfe\x22\x00\x00\xd8\x22\x00"),
                       ("utf16be-odd", b"\xfe\xff\x00a\x00")):
        add(f"decode/{name}", text, decode_source=True)
    texts = {
        "escapes": r'''"🦀\ud7ff\ud800\ud801\uD83E\uDD80" '\x00\x7f\u{10ffff}' ''',
        "escape-errors": r'''"\x" '\u0' "\u{}" '\u{110000}' "\8\9\01\377\400"''',
        "numbers": "0 01 08 - /*x*/ 077 0x 0x_1 0b2 0o8 1__2 1_ 1.e+ 1e 1e+n 1.2n 2e3n 1nfoo 1_2.3_4e5_6",
        "punctuation": "<<= >>>= >> >= </ *= **= ?? ?.1 ?.x #foo # ",
        "directives": "#!/usr/bin/env node\n// @ts-ignore\n/** @deprecated @link x */ a\n/* first\n @ts-expect-error */ b",
        "conflicts": "<<<<<<< HEAD\na\n=======\nb\n>>>>>>> other\n",
        "jsdoc-tags": "/** @deprecatedx @deprecated} @linkcode* @linkage */ a\n/** @seeX */ b",
        "linebreaks": "a\r\nb\rc\nd\u2028e\u2029f\u0085g",
    }
    for name, text in texts.items():
        for skip in (False, True):
            add(f"lexical/{name}/{int(skip)}", text, skip_trivia=skip)
    # Public rescan operations, including legal no-op branches.
    for op, text in (("rescan_less_than", "<<="), ("rescan_greater_than", ">>>="),
                     ("rescan_asterisk_equals", "*="), ("rescan_hash", "#foo"), ("rescan_question", "??")):
        add(f"rescan/{op}", text, [action("scan"), action(op), action("scan_all")], groups=("rescan", "diagnostics", "values"))
    for op in ("rescan_less_than", "rescan_greater_than", "rescan_hash"):
        add(f"rescan/{op}/noop", "name", [action("scan"), action(op)], groups=("rescan", "diagnostics", "values"))
    for op, message in (("rescan_asterisk_equals", "'ReScanAsteriskEqualsToken' should only be called on a '*='"),
                        ("rescan_question", "'reScanQuestionToken' should only be called on a '??'")):
        item = add(f"rescan/{op}/contract", "name", [action("scan"), action(op)], groups=("rescan",))
        item["expected_panic"] = {"action": 1, "class": "contract", "message": message}
    for tagged in (False, True):
        add(f"rescan/template/{int(tagged)}", r"}`\u{}${x}", [action("scan"), action("rescan_template", flag=tagged), action("scan_all")], groups=("rescan", "diagnostics", "values"))
    jsx = [("text", "  a\n b > } <div", [action("scan_jsx_ex", flag=False), action("scan_jsx")]),
           ("rescan", "  hello <x>", [action("scan"), action("rescan_jsx", flag=True)]),
           ("identifier", "foo-bar:baz", [action("scan"), action("scan_jsx_identifier"), action("scan_all")]),
           ("attribute", '  "a\\nb"', [action("scan_jsx_attribute"), action("rescan_jsx_attribute")])]
    for name, text, actions in jsx:
        add(f"rescan/jsx/{name}", text, actions, variant=1, groups=("rescan", "diagnostics", "values"))
    for backticks in (False, True):
        add(f"rescan/jsdoc/text/{int(backticks)}", "hello @param {x} `foo`\n* @", [action("scan_jsdoc_text", flag=backticks), action("scan_jsdoc"), action("can_follow_jsdoc_at"), action("scan_jsdoc")], groups=("rescan", "diagnostics", "values"))
    add("rescan/jsdoc/tokens", " \t@foo-bar\r\n*{}[]()<>.,`#=\\u0041", [action("scan_jsdoc") for _ in range(24)], groups=("rescan", "diagnostics", "values"))
    state = [action("scan"), action("mark"), action("scan"), action("mark"), action("scan"), action("rewind"),
             action("snapshot"), action("commit"), action("set_text", text_hex=b"// @ts-ignore\nx".hex()), action("scan"),
             action("mark"), action("set_text", text_hex=b"other".hex()), action("scan"),
             action("set_text", text_hex=b"// @ts-ignore\nx".hex()), action("rewind"), action("snapshot")]
    add("state/checkpoints-and-text", "a /* @ts-expect-error */ b c", state, groups=("rescan", "diagnostics", "values"))
    add("state/reset", "0xabc", [action("scan"), action("mark"), action("reset"), action("commit"), action("set_text", text_hex=b"\xff".hex()), action("scan"), action("set_on_error", flag=True), action("reset_token_state", pos=0), action("scan")], groups=("rescan", "diagnostics", "values"))
    add("state/positions", "abc", [action("scan"), action("reset_pos", pos=4), action("observe", getter="end"), action("reset_token_state", pos=0), action("snapshot"), action("scan")], groups=("rescan", "diagnostics", "values"))
    item = add("state/negative-reset", "abc", [action("reset_pos", pos=-1)], groups=("rescan",))
    item["expected_panic"] = {"action": 0, "class": "contract", "message": "Cannot reset token state to negative position"}
    item = add("state/bounds-token-text", "abc", [action("reset_pos", pos=4), action("observe", getter="token_text")], groups=("rescan",))
    item["expected_panic"] = {"action": 1, "class": "bounds"}
    add("state/nested-asterisks", "\n * a\n * b", [action("set_skip_jsdoc_asterisks", flag=True), action("set_skip_jsdoc_asterisks", flag=True), action("scan"), action("set_skip_jsdoc_asterisks", flag=False), action("scan"), action("set_skip_jsdoc_asterisks", flag=False), action("scan")], groups=("rescan", "diagnostics", "values"))
    for suffix in ("before", "after"):
        add(f"state/fresh/{suffix}", "// @ts-ignore\nlet x='a'", groups=("rescan", "diagnostics", "values"))
        if suffix == "before":
            add("state/poison", "x", [action("set_skip_trivia", flag=False), action("set_target", value=1), action("set_variant", value=1), action("set_on_error", flag=False), action("set_skip_jsdoc_asterisks", flag=True), action("scan")], groups=("rescan", "diagnostics", "values"))
    regexes = [r"/foo/visualstudiocode", r"/(?med-ium:bar)/", r"/\1/", r"/(hi)(hello)\2/", r"/\9(?<foo>x)\k<bar>/u",
               r"/[z-a]/", r"/[[]/v", r"/[/]/", r"/\c1[\c0]/", r"/\p{InvalidProperty=Value}/u", r"/\p{ascii}/u",
               r"/\p{sc=unknownX}/u", r"/\p{Script_Declensions=Inherited}/u", r"/\p{RGI_Emoji}/v", r"/\P{RGI_Emoji}/v",
               r"/[[a-z]&&[^aeiou]]/v", r"/[\q{foo|bar}--\q{foo}]/v", r"/[^\q{foo}]/v", r"/a{32,16}/", r"/a{999999999999999999999999999999999999999}/",
               r"/(?<a>x)|(?<a>y)/", r"/(?<a>x)(?<a>y)/", r"/(?i-m:x)/", r"/\k<missing>*\9(?<present>a)/z",
               "/\\k<İ>(?<i>a)/u", "/\\k<K>(?<k>a)/u", "/🦀[🦀-😀]/", "/🦀[🦀-😀]/u", "/foo", "/[foo ; }", r"/foo/uv", r"/foo/dd"]
    for index, text in enumerate(regexes):
        for report in ("omitted", "false", "true"):
            for target in (1, 4, 5, 8, 9, 10, 11, 12, 0):
                add(f"regexp/explicit/{index}/{report}/{target}", text, [action("scan"), action("rescan_slash", report_errors=report)], target=target, groups=("regexp", "rescan", "diagnostics", "values"))
    # Every accepted pinned property/alias/value is exercised through actual regexp parsing.
    scanner_tables = tables["scanner"]
    properties = list(scanner_tables["binary"]) + list(scanner_tables["strings"])
    for alias, canonical_name in sorted(scanner_tables["non_binary"].items()):
        properties += [alias + "=" + value for value in scanner_tables["values"][canonical_name]]
    for index, prop in enumerate(properties):
        for mode in ("u", "v"):
            add(f"regexp/property/{index}/{mode}", "/\\p{" + prop + "}/" + mode, [action("scan"), action("rescan_slash", report_errors="true")], groups=("regexp", "diagnostics", "values"))
    for index, text in enumerate((b"\xff", b"\xef\xbf\xbd", b"\xed\xa0\x80")):
        for mode in (b"", b"u", b"v"):
            add(f"regexp/raw/{index}/{mode.decode() or 'annexb'}", b"/[" + text + b"]" + b"/" + mode, [action("scan"), action("rescan_slash", report_errors="true")], groups=("regexp", "diagnostics", "values"))
    # Public recursion stress covers both entry points, valid and missing closures.
    for name, body in (("groups", b"(" * 20000 + b"a" + b")" * 20000),
                       ("groups-unclosed", b"(" * 20000 + b"a"),
                       ("lookahead", b"(?=" * 10000 + b"a" + b")" * 10000),
                       ("sets", b"[" * 20000 + b"a" + b"]" * 20000),
                       ("sets-unclosed", b"[" * 20000 + b"a"),
                       ("sets-partially-closed", b"[" * 20000 + b"a]"),
                       ("group-to-set", b"(" * 1000 + b"[" * 10000 + b"a" + b"]" * 10000 + b")" * 1000)):
        add(f"regexp/deep/{name}", b"/" + body + b"/v", [action("scan"), action("rescan_slash", report_errors="true")], groups=("regexp", "diagnostics", "values"))
    for first in range(0, 0x110000, 4096):
        add(f"identifier/block/{first:06x}", actions=[action("identifier_block", first=first, count=4096)], groups=("identifier",))
    for point in (-2147483648, -1, 0x110000, 2147483647):
        add(f"identifier/point/{point}", actions=[action("identifier_point", point=point)], groups=("identifier",))
    for index, (left, right) in enumerate([(b"a", b"A"), ("İ".encode(), b"i"), ("K".encode(), b"k"), ("ſ".encode(), b"S"), (b"\xff", b"\xfe"), (b"\xff", "�".encode()), (b"\xc0\xaf", b"\xff\xff"), ("Σ".encode(), "ς".encode())]):
        add(f"equal-fold/{index}", left, [action("equal_fold", other_hex=right.hex())], groups=("identifier",))
    for index, text in enumerate([b"", b"foo", b"foo-bar", b"foo:bar", b"async", b"\xff", "🦀".encode(), "α".encode(), "a\u200c".encode()]):
        add(f"identifier/text/{index}", text, [action("identifier_token"), action("valid_identifier"), action("identifier_text", variant=0), action("identifier_text", variant=1), action("intrinsic_jsx_name"), action("string_to_token")], groups=("identifier",))
    for kind in range(351):
        add(f"token/text/{kind}", actions=[action("token_to_string", kind=kind)], groups=("identifier",))
    add("token/suggestions", actions=[action("keyword_suggestions")], groups=("identifier",))
    for index, text in enumerate([b"", b"#!/bin/node\n //one\n/*two*/ x", "\u2028/** @x */\u2029* x".encode(), b"<<<<<<< HEAD\na\n=======\nb\n>>>>>>> theirs\n", b" * Foo\r\n * *\n Bar\r\n"]):
        actions = [action("shebang"), action("normalize_jsdoc")]
        for pos in (-1, 0, len(text), len(text) + 1):
            for mask in range(8):
                actions.append(action("skip_trivia", pos=pos, options=True, stop_after_line_break=bool(mask&1), stop_at_comments=bool(mask&2), in_jsdoc=bool(mask&4)))
            actions += [action("comment_ranges", pos=pos, trailing=trailing) for trailing in (False, True)]
        add(f"trivia/utilities/{index}", text, actions, groups=("rescan",))
    numeric = [b"", b" ", b"-0", b"+0", b"Infinity", b"-Infinity", b"NaN", b".1", b"1.", b"1e", b"1e+", b"1e309", b"1e-400", b"0x", b"0x10", b"-0x10", b"0b11", b"0o11", b"01", b"1_2", b"1n", b"9007199254740993", b"0x"+b"f"*300, b"0b1"+b"0"*1024, b"\xff", b"\xc2\x85"+b"1"]
    numeric += [str(value).encode() for value in (2**53-1, 2**53, 2**53+1, 2**64-1, 2**64, 10**21, 10**22)]
    for index, text in enumerate(numeric):
        add(f"number/from-string/{index}", text, [action("number_from_string")], groups=("numbers",))
    bits = [0, 1, 0x8000000000000000, 0x7ff0000000000000, 0xfff0000000000000, 0x7ff8000000000000, 0x7ff0000000000001, 0x3eb0c6f7a0b5ed8d, 0x444b1ae4d6e2ef50, 0x0010000000000000, 0x7fefffffffffffff]
    rng = random.Random(0x5052026)
    bits += [rng.getrandbits(64) for _ in range(256)]
    for value in bits:
        add(f"number/bits/{value:016x}", actions=[action("number_format", bits=f"{value:016x}")], groups=("numbers",))
    for index, text in enumerate([b"0n", b"123n", b"00123n", b"0xffn", b"0XFFn", b"0b101n", b"0o77n", b"1_2n", b"", b"n", b"12", b"0b2n", b"0xzn", b"-1n"]):
        item = add(f"number/pseudo/{index}", text, [action("pseudo_bigint")], groups=("numbers",))
        if text in (b"0b2n", b"0xzn"):
            item["expected_panic"] = {"action": 0, "class": "invalid_bigint", "input_hex": text.removesuffix(b"n").hex()}
    # Additive review regressions; initial cases retain their IDs and obligations.
    for name, text, panics in (("double-underscore", b"0x__1n", True),
                               ("trailing-underscore", b"0x1_n", True),
                               ("second-byte-radix", b"abc", True),
                               ("leading-hex-underscore", b"0x_1n", False),
                               ("leading-octal-underscore", b"0o_77n", False),
                               ("opaque", b"hello", False)):
        item = add(f"number/pseudo-review/{name}", text, [action("pseudo_bigint")], groups=("numbers",))
        if panics:
            item["expected_panic"] = {"action": 0, "class": "invalid_bigint", "input_hex": text.removesuffix(b"n").hex()}
    item = add("number/pseudo-decoded-utf16le", b"\xff\xfe" + "0b2n".encode("utf-16-le"), [action("pseudo_bigint")], decode_source=True, groups=("numbers",))
    item["expected_panic"] = {"action": 0, "class": "invalid_bigint", "input_hex": "306232"}
    for index, (text, target_mode) in enumerate((("/\\\u2028/", "annexb"), ("/\\\u2028/u", "u"), ("/\\\u2029/v", "v"))):
        item = add(f"regexp/expected-assertion/{index}/{target_mode}", text, [action("scan"), action("rescan_slash", report_errors="true")], groups=("regexp", "rescan"))
        item["expected_panic"] = {"action": 1, "class": "upstream_assertion", "message": "Debug failure. False expression."}
    for name, source, reporting, codes in (("grammar-true", b"/a/zz", "true", [1499,1499]),
                                          ("grammar-false", b"/a/zz", "false", []),
                                          ("unterminated-true", b"/abc", "true", [1161]),
                                          ("unterminated-false", b"/abc", "false", [1161])):
        item = add(f"witness/{name}", source, [action("scan"), action("rescan_slash", report_errors=reporting)], groups=("regexp", "rescan", "diagnostics", "values"))
        item["witness"] = {"action": 1, "codes": codes}
    item = add("witness/binary", b"\xff")
    item["witness"] = {"action": 0, "codes": [1490]}
    return result


def requests(upstream, corpus, supplemental):
    for row in corpus:
        content = (upstream / row["path"]).read_bytes()
        if len(content) != row["bytes"] or hashlib.sha256(content).hexdigest() != row["sha256"]:
            raise ValueError(f"frozen corpus source changed: {row['path']}")
        for skip in (False, True):
            yield case(f"corpus/{row['path']}/{int(skip)}", content, variant=row["variant"], skip_trivia=skip)
    yield from supplemental


def inventory(upstream, corpus, supplemental):
    digest = hashlib.sha256()
    ids, groups, operations = [], Counter(), Counter()
    for item in requests(upstream, corpus, supplemental):
        request = validate_request(item["request"])
        ids.append(request["id"])
        groups.update(item["groups"])
        operations.update(action["op"] for action in request["actions"])
        digest.update(canonical(item) + b"\n")
    if len(set(ids)) != len(ids):
        raise ValueError("duplicate frozen case IDs")
    return ids, {"version": 1, "requests": len(ids), "sha256": digest.hexdigest(),
                 "groups": dict(sorted(groups.items())), "operations": dict(sorted(operations.items())),
                 "source_files": len(corpus), "source_bytes": sum(row["bytes"] for row in corpus),
                 "raw_file_cases": 2 * len(corpus)}


def freeze(upstream, tables, write=False):
    corpus = corpus_inventory(upstream)
    supplemental = fixtures(upstream, tables)
    ids, probes = inventory(upstream, corpus, supplemental)
    outputs = {"corpus.json": corpus, "fixtures.json": supplemental, "cases.json": ids, "probes.json": probes}
    for name, value in outputs.items():
        path = ROOT / "data/s05" / name
        content = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=True) + "\n"
        if write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        elif not path.exists() or path.read_text() != content:
            raise ValueError(f"frozen S05 inventory drift: {path}; review before explicit --write-manifest")
    return corpus, supplemental, probes
