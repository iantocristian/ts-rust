# Distance to historical Go budgets

This small arithmetic diagnostic derives [result.json](result.json) from exact
retained reports. It runs no compiler, benchmark or native child, recalculates no
confidence intervals, and emits no sprint metrics. It contextualizes the distance
remaining after accepted A0-b and CP1, and for the rejected CP1 list-copy
experiment; it is not a fresh Go-relative acceptance result. Each checkpoint
has an explicit production disposition. A candidate distance row does not imply
that the implementation was promoted.

The five compressed inputs preserve the original report bytes. Their raw SHA-256
identities are pinned in `replay.py`. The original control manifest independently
binds its baseline report; each screen's source manifests identify its actual
Rust control/candidate roles. Their checkpoint medians are also checked against
all seven retained values per variant/domain/worker. Full graph, raw-sample and
instrumentation replay belongs to the earlier capture archives, not this helper.

| Input | Origin | Raw SHA-256 |
| --- | --- | --- |
| Original Go report | Immutable original control's `evidence/baseline-report.json` | `1c8dc74fb5c3d6d3ef5e6a8f37595c8d9dbacc7ba1f1ed0b7bef21bfb5fad5dd` |
| Original control manifest | Immutable control bundle | `c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363` |
| A0-b screen | [Retained A0-b archive](../results/2026-09-08/a0b-README.md), `screen/report.json` | `0f70cf1705dc7006695d5622bd00356b3912ab29aedc051f57ef6e1e18fe513f` |
| CP1 screen | [Retained CP1 archive](../results/2026-09-08-cp1/README.md), `screen/report.json` | `66b6547a29aafaddbbc01cc834ee7460a79126dcb0dba133d5e6606b144cfc6b` |
| Rejected CP1 list-copy screen | [Retained list-copy archive](../results/2026-09-08-list-copy/README.md), `screen/report.json` | `8ccc065b2c33b45ddfed83ca3b678c7d0c87d927f017aaf7e6026ab7680f4f10` |

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
`historical_distance` retains its candidate role; the added
`same_screen_control_historical_distance` prices the explicitly identified
control from the same capture against the same historical denominator. Existing
A0-b and CP1 numeric fields and compressed input bytes are unchanged.

The CP1 one-worker wall gap is 1.653226 seconds to the old Go median. Its phase
origin is unmeasured by this diagnostic; an old binding profile cannot establish
that binding dominates the current remainder. Allocation remains about 2.607 GB
above the historical allocation budgets, while RSS remains about 2.292–2.298 GB
above its separate budgets. Retained requested-live storage is not an RSS measure.

The list-copy experiment was **rejected**: its same-screen wall improvements of
0.825% and 0.191% did not meet the predeclared 5% threshold in either worker mode.
It met the separate nonregression conditions. Its control is the retained CP1
implementation (`3a57976f…`); the rejected candidate is `b03b64eb…`. Their
distances to the historical Go budgets are:

| Workers | Domain | Retained CP1 control median | Rejected list-copy median | Control excess to budget | Rejected excess to budget |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 | Wall, seconds | 4.344062333 | 4.308220625 | 1.406435458 | 1.370593750 |
| 8 | Wall, seconds | 0.970697959 | 0.968843792 | 0.336735126 | 0.334880959 |
| 1 | Allocation, GB | 4.642569475 | 4.642569091 | 2.607343376 | 2.607342992 |
| 8 | Allocation, GB | 4.642570910 | 4.642572046 | 2.606804316 | 2.606805452 |
| 1 | Peak RSS, GB | 4.506697728 | 4.506615808 | 2.297749504 | 2.297667584 |
| 8 | Peak RSS, GB | 4.509220864 | 4.509204480 | 2.292175667 | 2.292159283 |

The contemporaneous retained-control medians above provide the latest observed
distance for the retained implementation. The earlier CP1 screen's 4.590853292
second median and this screen's 4.308220625 second rejected-candidate median are
from separate captures; subtracting them does not measure the list-copy effect.
The source report, role identities and recorded disposition are retained
separately from the arithmetic. The disposition is a reviewed production decision,
not an acceptance conclusion calculated by this helper.

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
