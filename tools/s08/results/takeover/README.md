# S08 first P0/P1 observations

`recorded-results.tar.xz` contains every local flag, phase-policy and constructor
capture from this increment, including the rejected first Go retained-memory
endpoint, plus validation logs and the final storage diagnostic's source snapshot.
`manifest.json` records every member's size/SHA-256 and the archive hash.

The current phase record is `p0-phases-source-closure`, checked independently in
`p0-phases-source-closure-check`. The current constructor record is
`storage-source-closure`. Earlier records are history, not interchangeable final
evidence. `storage-initial` has constructor parity but an unusable retained-memory
endpoint; its negative Go delta must not enter a memory comparison.

The full interpretation and remaining work are in `docs/S08-P0-P1.md`.
Reproduction commands are in `data/s08/README.md`. Native Rust binaries are for
the recorded darwin/arm64 host. Go overlays record their original absolute paths;
use the reproduction commands to create new paths rather than executing an
archived overlay unchanged. Go source and frozen S07 inputs remain in the pinned
repository; this archive is not a standalone Go toolchain or a sprint producer.

Verify integrity before inspecting a local extraction:

```python
import hashlib, json, tarfile
from pathlib import Path
p = Path("tools/s08/results/takeover")
m = json.loads((p / "manifest.json").read_bytes())
assert hashlib.sha256((p / m["archive"]).read_bytes()).hexdigest() == m["archive_sha256"]
with tarfile.open(p / m["archive"], "r:xz") as archive:
    actual = {}
    for member in archive:
        assert member.isfile()
        raw = archive.extractfile(member).read()
        actual[member.name] = {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
    assert actual == m["files"]
```
