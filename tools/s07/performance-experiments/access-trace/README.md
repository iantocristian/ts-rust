# Scoped access trace

This is an isolated, untimed observer of the accepted CP1 implementation. The
production checkout and benchmark source remain unchanged. It prepares the
recorded workload for the next compact-storage experiment; it is not a layout
benchmark or S07 acceptance evidence.

`stage.py` composes checked physical-state accessors and binder hooks in a copied
accepted source bundle. `src/recorder.rs` is installed only in that copy. The
independent `verify.py` reads the compressed trace incrementally, checking binary
framing, hashes, phase order and the declared operation schema.

The three `*-registry.json` files define the observed operations and exclusions.
Physical syntax includes obsolete nodes and list backings. A successful node
lookup does not imply reads of its payload fields. Reads outside named sites,
temporary-copy provenance, full binding-field contents and record births still
need subsequent instrumentation before a broader replay claim.

## Format and limits

All integers are little endian. `S07TRC01` begins the stream. Each `BLK1` block has
a sequence (`u64`, starting0), record count (`u32`), payload length (`u32`) and
SHA256 of that payload. A record has a `u32` body length, then operation/site
(`u16` each), file/domain/reserved (`u32` each; reserved0), four `u64` values and
optional raw blob bytes. The fixed record occupies52 bytes including its length.
`END1` contains block/record/payload totals (`u64` each) and SHA256 of the ordered
block hashes. Completion requires this footer and exact compressed/decoded EOF.

File ordinals are1-based; only the final capture event uses file0. Domains0/1/2
are driver, physical-state observer and binder. Hooks outside the binder domain
are suppressed, so serializing state cannot become apparent binder activity.
Owner-qualified packed IDs retain both original32-bit fields. Observing them
does not assign runtime IDs. Text uses raw bytes; blob chunks preserve offsets
and the total logical length, including empty values.

The initial declarations are32 GiB record payload,3 GiB compressed output,
1 MiB maximum block/record,64 KiB blob chunks and1 GiB free-space reserve.
Output streams directly to gzip level1. Overflow, child failure, framing failure
or graph mismatch leaves raw diagnostics and an unsuccessful receipt.

## Commands

```sh
python3 tools/s07/performance-experiments/access-trace/probe.py build --output target/s07-bis/access-trace-build-1
python3 tools/s07/performance-experiments/access-trace/probe.py capture --build BUILD --build-sha SHA --output SIZING --sizing
python3 tools/s07/performance-experiments/access-trace/probe.py capture --build BUILD --build-sha SHA --output FULL
python3 tools/s07/performance-experiments/access-trace/probe.py record --build BUILD --build-sha SHA --output PENDING
python3 tools/s07/performance-experiments/access-trace/probe.py verify --output FULL/trace.bin.gz
```

The sizing subset is the union of the first16 and largest16 inputs, deduplicated
and kept in original order. Its manifest names every original index. Full mode
uses all13,094 inputs. Both compare every retained graph report with the original
accepted normal executable, preserving the exact raw records without a new
normalization exception. The native source, observer patches, actual executable,
tool closure, loaded-input identity, raw reports and invocation receipts are
recorded. Builds/captures use the existing measurement lock to avoid contaminating
another experiment; their elapsed times are not performance samples.

`record` seals raw data, child receipts and native graph checks with the manifest
kind `s07_bis_access_trace_pending_verification` and `trace_verified: false`.
This permits a separately recorded native decoder to verify large streams. It
does not satisfy the verified-capture consumer; a later result must reference
both the immutable recording and the successful verifier invocation.
