"""Independently classified library panics; unknown panics never satisfy parity."""
from s04 import panic_class as leaf_panic_class
from s06_protocol import hex_bytes, sha256


def panic_identity(request, stage, message, runtime, decoded_nil_wires):
    """Return a comparable identity, or None for an unrecognized failure.

    `decoded_nil_wires` maps each runtime to hashes of wire bytes for which its
    prior DecodeNodes call successfully returned a nil root. A nil-pointer panic
    without this independent witness cannot be relabeled as the nil-root contract.
    Raw messages remain in the adapter frames and failure artifacts.
    """
    if runtime not in ("oracle", "rust") or type(message) is not str:
        raise ValueError("invalid panic classification input")
    if leaf_panic_class(message, runtime) == "bounds":
        return ("bounds",)
    if request["op"] == "decode" and stage == "decode" and request["entrypoint"] == "source_file":
        expected = {"oracle": "runtime error: invalid memory address or nil pointer dereference",
                    "rust": "nil SourceFile root returned by DecodeNodes"}
        digest = sha256(hex_bytes(request["wire_hex"], "nil-root wire"))
        if message == expected[runtime] and digest in decoded_nil_wires.get(runtime, set()):
            return ("decode_source_file_nil_root", digest)
    if request["op"] == "codec" and stage == "reencode":
        key = "codec:" + request["id"]
        expected = {"oracle": "runtime error: invalid memory address or nil pointer dereference",
                    "rust": "nil root passed to EncodeNode"}
        if message == expected[runtime] and key in decoded_nil_wires.get(runtime, set()):
            return ("encode_node_nil_root", key)
    expected = {"decode": "SyntheticExpression should never be decoded",
                "encode_source_file": "SyntheticExpression should never be encoded",
                "encode": "SyntheticExpression should never be encoded",
                "reencode": "SyntheticExpression should never be encoded"}
    if stage in expected and message == expected[stage]:
        return ("synthetic_expression", message)
    # ScriptKindUnknown is rejected explicitly by initializeState. This does not
    # permit other Debug failure assertions, even when both runtimes panic.
    if request["op"] == "parse" and stage == "parse" and request["script_kind"] == 0:
        expected = "ScriptKind must be specified when parsing source file: " + request["filename"]
        if message == expected:
            return ("script_kind_required", request["filename"])
    return None
