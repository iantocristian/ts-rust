# Distance to historical Go budgets

This small arithmetic diagnostic derives [result.json](result.json) from exact
retained reports. It runs no compiler, benchmark or native child, recalculates no
confidence intervals, and emits no sprint metrics. It contextualizes the distance
remaining after A0-b and CP1; it is not a fresh Go-relative acceptance result.

The four compressed inputs preserve the original report bytes. Their raw SHA-256
identities are pinned in `replay.py`. The original control manifest independently
binds its baseline report; the A0-b and CP1 source manifests identify their actual
Rust control/candidate roles. Their checkpoint medians are also checked against
all seven retained values per variant/domain/worker. Full graph, raw-sample and
instrumentation replay belongs to the earlier capture archives, not this helper.

| Input | Origin | Raw SHA-256 |
| --- | --- | --- |
| Original Go report | Immutable original control's `evidence/baseline-report.json` | `1c8dc74fb5c3d6d3ef5e6a8f37595c8d9dbacc7ba1f1ed0b7bef21bfb5fad5dd` |
| Original control manifest | Immutable control bundle | `c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363` |
| A0-b screen | [Retained A0-b archive](../results/2026-09-08/a0b-README.md), `screen/report.json` | `0f70cf1705dc7006695d5622bd00356b3912ab29aedc051f57ef6e1e18fe513f` |
| CP1 screen | [Retained CP1 archive](../results/2026-09-08-cp1/README.md), `screen/report.json` | `66b6547a29aafaddbbc01cc834ee7460a79126dcb0dba133d5e6606b144cfc6b` |

Each worker mode has its own historical Go denominator:

| Workers | Go wall median | Wall budget | Go allocated | Allocation budget (×0.7) | Go peak RSS | RSS budget (×0.7) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2.937627 s | 2.937627 s | 2.907466 GB | 2.035226 GB | 3.155640 GB | 2.208948 GB |
| 8 | 0.633963 s | 0.633963 s | 2.908238 GB | 2.035767 GB | 3.167207 GB | 2.217045 GB |

The [A0-b table](../../../../docs/S07-bis-A0.md#distance-to-historical-go-budgets)
and [CP1 table](../../../../docs/S07-bis-CP1.md#distance-to-historical-go-budgets)
show absolute median gaps and ratios. The JSON preserves base units (nanoseconds
and bytes), exact rational historical budgets, and each checkpoint's same-screen
Rust/control timing bound separately. Decimal GB means 1,000,000,000 bytes.

The CP1 one-worker wall gap is 1.653226 seconds to the old Go median. Its phase
origin is unmeasured by this diagnostic; an old binding profile cannot establish
that binding dominates the current remainder. Allocation remains about 2.607 GB
above the historical allocation budgets, while RSS remains about 2.292–2.298 GB
above its separate budgets. Retained requested-live storage is not an RSS measure.

Final CPU acceptance requires fresh Rust/Go median ratios and bootstrap upper
95% ratio bounds at most 1.0, with both runtimes' relative MAD at most 5%, at one
and eight workers. Final memory ratios use the worse worker-mode median ratio
and must be at most 0.7 for allocation and RSS. An old Go denominator and a newer
Rust median do not establish these requirements. The stored Go-relative timing
upper bounds for newer checkpoints are explicitly `null`, not inferred from
their Rust/control bounds. Independent checkpoint percentages are never compounded.

```sh
python3 tools/s07/performance-experiments/gate-distance/replay.py
python3 -m unittest discover -s tools/s07/performance-experiments/gate-distance -p test_replay.py -v
```

`--write` regenerates only the derived `result.json` from its pinned inputs.
Adding a future checkpoint requires a new retained source report and explicit
identity/role entry; this diagnostic does not discover or substitute current
workspace results. Historical reports and existing capture helpers stay unchanged.
