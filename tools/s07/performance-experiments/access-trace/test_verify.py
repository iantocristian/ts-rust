import copy
import gzip
import hashlib
import importlib.util
import io
from pathlib import Path
import struct
import sys
import tempfile
import unittest

PATH = Path(__file__).with_name("verify.py")
SPEC = importlib.util.spec_from_file_location("access_trace_verifier_tests", PATH)
verify = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = verify
SPEC.loader.exec_module(verify)
REGISTRIES = [PATH.with_name(name + "-registry.json") for name in ("protocol", "state", "hooks")]
NODE = (7 << 32) | 1


def record(op, *, site=0, file=1, domain=0, a=0, b=0, c=0, d=0, reserved=0, blob=b""):
    body = struct.pack("<HHIIIQQQQ", op, site, file, domain, reserved, a, b, c, d) + blob
    return struct.pack("<I", len(body)) + body


def file_records(file=1, *, node=NODE, kind=3):
    return [record(1, file=file, a=file - 1, b=3), record(2, file=file),
            record(3, file=file, a=node, b=2), record(4, file=file),
            record(10, file=file, domain=1, a=node >> 32, b=8, d=3),
            record(11, site=1, file=file, domain=1, a=node >> 32, b=1, c=1),
            record(11, site=2, file=file, domain=1, a=8),
            record(14, site=1, file=file, domain=1, a=node, b=kind),
            record(79, file=file, domain=1, a=1),
            record(5, file=file), record(6, file=file),
            record(100, site=100, file=file, domain=2, a=node),
            record(101, site=101, file=file, domain=2, a=node, b=kind & 65535),
            record(102, site=103, file=file, domain=2, a=node),
            record(103, site=106, file=file, domain=2, a=node, b=5),
            record(102, site=104, file=file, domain=2, a=node, b=5),
            record(105, site=108, file=file, domain=2, a=17, b=(7 << 32) | 2, c=1, d=node),
            record(107, site=110, file=file, domain=2, a=node),
            record(7, file=file, a=2, b=1), record(8, file=file, a=1)]


def fixture_records(files=1):
    result = []
    for file in range(1, files + 1):
        result.extend(file_records(file))
    result.append(record(9, file=0, a=files, b=3 * files, c=2 * files, d=files))
    return result


def encoded(records, *, chunk=5, change_block=None, change_footer=None, trailing=b""):
    blocks, hashes = [], []
    for sequence, start in enumerate(range(0, len(records), chunk)):
        rows = records[start:start + chunk]
        payload = b"".join(rows)
        digest = hashlib.sha256(payload).digest()
        header = struct.pack("<QII32s", sequence, len(rows), len(payload), digest)
        block = b"BLK1" + header + payload
        blocks.append(change_block(sequence, block) if change_block else block)
        hashes.append(digest)
    footer = b"END1" + struct.pack("<QQQ32s", len(blocks), len(records), sum(map(len, records)),
                                  hashlib.sha256(b"".join(hashes)).digest())
    if change_footer:
        footer = change_footer(footer)
    return b"S07TRC01" + b"".join(blocks) + footer + trailing


class BoundedReader(io.BytesIO):
    def read(self, size=-1):
        if not 0 < size <= verify.READ_CHUNK:
            raise AssertionError("unbounded compressed input read")
        return super().read(min(size, 11))


class TraceVerifierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.documents = [verify.strict_json(path.read_bytes()) for path in REGISTRIES]
        cls.events = verify.registry(cls.documents)

    def check(self, rows=None, **options):
        raw = encoded(fixture_records() if rows is None else rows)
        return verify.verify(BoundedReader(gzip.compress(raw)), self.events, **options)

    def rejects(self, raw):
        with self.assertRaises(verify.InvalidTrace):
            verify.verify(io.BytesIO(gzip.compress(raw)), self.events)

    def test_actual_registry_complete_trace_and_phase_specific_counters(self):
        result = self.check(expected={"files": 1, "source_bytes": 3, "nodes": 2, "symbols": 1})
        self.assertEqual(result["records"], 21)
        self.assertEqual(result["totals"], {"files": 1, "source_bytes": 3, "nodes": 2, "symbols": 1})
        self.assertEqual(result["by_domain"], {"0": 9, "1": 5, "2": 7})
        self.assertEqual(result["binder_operations"], {"lookup": 1, "read": 4, "write": 2, "other": 0})
        self.assertEqual(result["files"][0]["state_node_headers"], 1)
        self.assertEqual(result["by_op_shape"]["100"], {"1": 1})
        self.assertEqual(result["by_op_site"]["102"], {"103": 1, "104": 1})
        self.assertEqual(result["by_op_site"]["11"], {"1": 1, "2": 1})
        self.assertFalse(result["full_semantic_replay"])
        self.assertIsNone(result["unobserved_operation_count"])

    def test_file_local_header_state_does_not_escape_and_unmapped_is_visible(self):
        rows = fixture_records(2)
        rows.insert(12, record(100, site=100, domain=2, a=(8 << 32) | 1))
        result = self.check(rows)
        self.assertEqual(result["by_op_shape"]["100"], {"1": 2, "unmapped": 1})
        self.assertEqual([file["unmapped_node_operations"] for file in result["files"]], [1, 0])
        self.assertEqual([file["state_node_headers"] for file in result["files"]], [1, 1])

    def test_named_kind_flags_and_sign_extended_state_kind(self):
        rows = file_records(kind=2**64 - 1) + [record(9, file=0, a=1, b=3, c=2, d=1)]
        self.assertTrue(self.check(rows)["complete"])
        for index, replacement in ((12, record(101, site=101, domain=2, a=NODE, b=4)),
                                   (13, record(102, site=103, domain=2, a=NODE, b=1)),
                                   (15, record(102, site=104, domain=2, a=NODE, b=0)),
                                   (7, record(14, site=1, domain=1, a=NODE, b=65535))):
            changed = fixture_records()
            changed[index] = replacement
            self.rejects(encoded(changed))

    def test_duplicate_initial_header_and_owner_qualified_node_ids(self):
        rows = fixture_records()
        rows.insert(8, rows[7])
        self.rejects(encoded(rows))
        for invalid in (0, 1, 7 << 32):
            rows = fixture_records()
            rows[11] = record(100, site=100, domain=2, a=invalid)
            self.rejects(encoded(rows))

    def test_initial_state_requires_owner_end_and_exact_core_header_count(self):
        base = fixture_records()
        for index in (4, 5, 6, 7, 8):
            self.rejects(encoded(base[:index] + base[index + 1:]))
        for index, replacement in ((4, record(10, domain=1, a=6, b=8, d=3)),
                                   (4, record(10, domain=1, a=7, b=8, d=4)),
                                   (5, record(11, site=1, domain=1, a=7, b=2, c=2)),
                                   (8, record(79, domain=1, a=2)),
                                   (8, record(79, domain=1, a=1, b=1)),
                                   (8, record(79, domain=1, a=1, d=1))):
            rows = list(base)
            rows[index] = replacement
            self.rejects(encoded(rows))
        self.rejects(encoded(base[:8] + [base[4]] + base[8:]))
        self.rejects(encoded(base[:9] + [base[7]] + base[9:]))

    def test_blob_chunks_are_contiguous_complete_and_counted(self):
        base = fixture_records()
        base[8] = record(79, domain=1, a=1, d=3)
        chunks = [record(23, domain=1, a=7, d=3, blob=b"ab"),
                  record(23, domain=1, a=7, c=2, d=3, blob=b"c")]
        self.assertTrue(self.check(base[:8] + chunks + base[8:])["complete"])
        for second in (record(23, domain=1, a=7, c=1, d=3, blob=b"c"),
                       record(23, domain=1, a=8, c=2, d=3, blob=b"c"),
                       record(23, domain=1, a=7, c=2, d=4, blob=b"c"),
                       record(23, domain=1, a=7, c=2, d=3, blob=b"cd"),
                       record(23, domain=1, a=7, c=2, d=3)):
            self.rejects(encoded(base[:8] + [chunks[0], second] + base[8:]))
        self.rejects(encoded(base[:8] + chunks[:1] + base[8:]))
        self.rejects(encoded(base[:8] + chunks[1:] + base[8:]))

    def test_physical_node_and_auxiliary_order_identity_and_missing_slots(self):
        base = fixture_records()
        base[6] = record(11, site=2, domain=1, a=8, b=2, c=2)
        base[8] = record(79, domain=1, a=1, b=2)
        aux = [record(24, site=2, domain=1, a=(8 << 32) | slot) for slot in (1, 2)]
        self.assertTrue(self.check(base[:8] + aux + base[8:])["complete"])
        for invalid in (0, 8 << 32, (7 << 32) | 1, (8 << 32) | 2):
            first = record(24, site=2, domain=1, a=invalid)
            self.rejects(encoded(base[:8] + [first, aux[1]] + base[8:]))
        self.rejects(encoded(base[:8] + aux[:1] * 2 + base[8:]))
        base = fixture_records()
        base[5] = record(11, site=1, domain=1, a=7, b=2, c=2)
        base[8] = record(79, domain=1, a=2)
        second = record(14, site=1, domain=1, a=NODE + 1, b=3)
        self.assertTrue(self.check(base[:8] + [second] + base[8:])["complete"])
        self.rejects(encoded(base[:7] + [second, base[7]] + base[8:]))

    def test_progress_reports_observed_blocks_without_skipping_checks(self):
        progress = []
        result = self.check(progress=progress.append, progress_records=6)
        self.assertEqual([row["records"] for row in progress], [10, 15, 20])
        self.assertEqual(result["records"], 21)
        with self.assertRaises(verify.InvalidTrace):
            self.check(progress_records=0)

    def test_phase_order_file_identity_domains_and_completion(self):
        base = fixture_records()
        variants = [base[:-1], base[1:], base + [base[-1]],
                    base[:5] + [base[11]] + base[5:], base[:11] + [base[7]] + base[11:],
                    base[:2] + [record(4)] + base[3:],
                    base[:-1] + [record(9, file=0, a=1, b=4, c=2, d=1)]]
        for index, replacement in ((0, record(1, file=2, a=0, b=3)),
                                   (11, record(100, site=100, file=2, domain=2, a=NODE)),
                                   (11, record(100, site=100, domain=1, a=NODE)),
                                   (19, record(8, a=2))):
            changed = list(base)
            changed[index] = replacement
            variants.append(changed)
        for rows in variants:
            self.rejects(encoded(rows))

    def test_unknown_ops_sites_reserved_fields_blobs_and_index_constraints(self):
        for replacement in (record(99, domain=2), record(100, site=99, domain=2, a=NODE),
                            record(100, site=100, domain=2, a=NODE, reserved=1),
                            record(100, site=100, domain=2, a=NODE, blob=b"unexpected"),
                            record(105, site=108, domain=2, b=2, c=2),
                            record(101, site=101, domain=2, a=NODE, b=65536)):
            rows = fixture_records()
            rows[11] = replacement
            self.rejects(encoded(rows))

    def test_blocks_hashes_sequences_record_lengths_counts_and_footer(self):
        raw = encoded(fixture_records())
        mutations = [raw[:7], b"BADMAGIC" + raw[8:], raw[:-1], raw + b"trailing",
                     raw + raw[-60:]]
        for offset in (8, 12, 20, 24, 28, 60):
            changed = bytearray(raw)
            changed[offset] ^= 1
            mutations.append(bytes(changed))
        for offset in (4, 12, 20, 28):
            mutations.append(encoded(fixture_records(), change_footer=lambda footer, offset=offset:
                footer[:offset] + bytes([footer[offset] ^ 1]) + footer[offset + 1:]))
        for bad_length in (0, 47, 2**32 - 1):
            rows = fixture_records()
            rows[0] = struct.pack("<I", bad_length) + rows[0][4:]
            mutations.append(encoded(rows))
        # A valid block hash cannot excuse an understated record count.
        mutations.append(encoded(fixture_records(), change_block=lambda seq, block:
            block[:12] + struct.pack("<I", 1) + block[16:] if seq == 0 else block))
        for changed in mutations:
            self.rejects(changed)

    def test_gzip_crc_truncation_trailing_and_multiple_members(self):
        compressed = gzip.compress(encoded(fixture_records()))
        changed_crc = compressed[:-8] + bytes([compressed[-8] ^ 1]) + compressed[-7:]
        for invalid in (compressed[:-1], compressed[:15], changed_crc, compressed + b"garbage",
                        compressed + gzip.compress(b"")):
            with self.assertRaises(verify.InvalidTrace):
                verify.verify(BoundedReader(invalid), self.events)

    def test_explicit_limits_expected_totals_and_empty_writer_fixture(self):
        raw = encoded(fixture_records())
        compressed = gzip.compress(raw)
        for limits in (verify.Limits(payload_bytes=51),
                       verify.Limits(compressed_bytes=len(compressed) - 1),
                       verify.Limits(block_bytes=52), verify.Limits(record_bytes=52, active_nodes=0),
                       verify.Limits(files=0)):
            with self.assertRaises(verify.InvalidTrace):
                verify.verify(BoundedReader(compressed), self.events, limits=limits)
        with self.assertRaises(verify.InvalidTrace):
            self.check(expected={"files": 2})
        with self.assertRaises(verify.InvalidTrace):
            self.check(fixture_records(0))
        self.assertEqual(self.check(fixture_records(0), allow_empty=True)["totals"]["files"], 0)

    def test_large_logical_blob_is_streamed_and_chunk_and_record_caps_include_prefix(self):
        rows = fixture_records()
        blob = b"x" * verify.READ_CHUNK
        logical_length = verify.MIB
        rows[8] = record(79, domain=1, a=1, d=logical_length)
        chunks = [record(23, domain=1, a=7, c=offset, d=logical_length, blob=blob)
                  for offset in range(0, logical_length, len(blob))]
        rows = rows[:8] + chunks + rows[8:]
        result = verify.verify(BoundedReader(gzip.compress(encoded(rows, chunk=1))), self.events)
        self.assertGreater(result["payload_bytes"], verify.MIB)
        with self.assertRaises(verify.InvalidTrace):
            verify.verify(BoundedReader(gzip.compress(encoded(rows, chunk=1))), self.events,
                          limits=verify.Limits(record_bytes=verify.READ_CHUNK + 51))
        for invalid_size in (verify.READ_CHUNK + 1, verify.MIB):
            rows[8] = record(23, domain=1, a=7, d=invalid_size, blob=b"x" * invalid_size)
            with self.assertRaises(verify.InvalidTrace):
                verify.verify(BoundedReader(gzip.compress(encoded(rows, chunk=1))), self.events)

    def test_registry_duplicate_domains_and_unknown_constraints_fail(self):
        for mutation in (lambda docs: docs[0]["events"].append(docs[0]["events"][0]),
                         lambda docs: docs[2]["events"][0].update(domain=1),
                         lambda docs: docs[2]["events"][0].update(constraints=[{"rule": "unknown"}]),
                         lambda docs: docs[0]["events"][0].pop("sites")):
            docs = copy.deepcopy(self.documents)
            mutation(docs)
            with self.assertRaises(verify.InvalidTrace):
                verify.registry(docs)
        for raw in (b'{"version":1,"version":1}', b'{"value":NaN}'):
            with self.assertRaises(verify.InvalidTrace):
                verify.strict_json(raw)

    def test_file_entry_point(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "trace.bin.gz"
            path.write_bytes(gzip.compress(encoded(fixture_records())))
            result = verify.verify_file(path, REGISTRIES, expected={"files": 1})
            self.assertEqual(result["records"], 21)


if __name__ == "__main__":
    unittest.main()
