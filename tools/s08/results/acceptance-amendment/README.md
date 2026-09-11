# Owner-approved E2 acceptance amendment

See [the accepted decision and results](../../../../docs/S08-acceptance-amendment.md).
These are source/Go observations and verification records; Rust semantic checker
parity remains unimplemented. Informational results are not an allow-list and
cannot affect E2 acceptance.

Two archives preserve different parts of the same work:

- `baselines.tar.xz`: two full native captures after the input correction,
  original raw baseline bytes, ordered query observations, complete diagnostic
  payloads, commands, overlays, reports and the per-tier summary. All 9,369
  acceptance cases complete; 1,325 informational cases complete and 34 retain
  their native option-parsing failures.
- `provenance.tar.xz`: source snapshots from that capture; raw S06/S07 extraction
  and loader observations; compiled native policy/phase captures and independent
  regeneration checks; exact source-diff audit; source/policy changes and
  packaging scripts. Earlier diagnostic failures are retained (the first
  extraction compile missed a retained config index; the first binder freeze
  encountered the S06 producer lock). Both were corrected before final checks.

The `*-01` and `*-02` policy captures precede the final portability correction.
`acceptance-policy-portable` and `acceptance-policy-portable-check` are the final
matching policy records: host metadata stays in the full native reports, outside
the portable frozen policy. The corresponding `phase-amendment-portable` freeze
and check outputs match. Both baseline captures have identical semantic outputs
and ordered query actions; twelve native numeric type IDs differ in four cases,
recorded in `baseline-recapture-comparison.json`. No numeric IDs were normalized
in the raw observations. The historical baseline-authority archive in the
sibling directory remains unchanged.

The frozen manifests live in `data/s06`, `data/s07` and `data/s08`. The E1, E2,
binder, program and self-test results are recorded in the ordinary immutable
`status/evidence` records; `validation.json` identifies the records belonging to
this increment. Historical or stale metrics do not become current by being
copied into an archive.

Each archive has its own `.manifest.json` listing every member's bytes/SHA-256
and the archive hash. Verify before inspecting an extraction:

```python
import hashlib, json, tarfile
from pathlib import Path
p = Path("tools/s08/results/acceptance-amendment")
for manifest in p.glob("*.tar.xz.manifest.json"):
    m = json.loads(manifest.read_bytes())
    assert hashlib.sha256((p / m["archive"]).read_bytes()).hexdigest() == m["archive_sha256"]
    with tarfile.open(p / m["archive"], "r:xz") as archive:
        actual = {}
        for member in archive:
            assert member.isfile()
            raw = archive.extractfile(member).read()
            actual[member.name] = {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        assert actual == m["files"]
```

Archived Go overlays contain original absolute paths. Reproduce through the
scripts named in the decision document, using new output directories and the
pinned checkout/toolchain. Do not run archived absolute-path overlays unchanged.
