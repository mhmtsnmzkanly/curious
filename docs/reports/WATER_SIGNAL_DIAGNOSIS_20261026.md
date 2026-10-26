# Water signal diagnosis — 26 October 2026

This retrospective analysis asks why the V19 context-water residual is usually
zero in the completed V2 water-adaptation fixture. It reads the six existing
formal raw records one seed at a time. It ran no World and changed no engine,
protocol, or raw evidence. The primary area is K3 (individual learning), with
K1/K2 measurement dependencies.

## Result

The records do not show a residual-computation defect. They show a mostly
uninformative residual target in this fixture: of 147 context observations
accepted by the online learner across seeds 62201–62206, 146 were exactly zero
and one was +67. The per-seed counts were 24/21/23/29/29/21; the sole nonzero
observation was seed 62204, source `(-2, 0)`, at tick 386. The frozen trained
tables did not change, as intended.

The code computes a *correction* to the current foraging estimate. At choice
time, `foraging::value` already uses currently visible water amount, the current
body's water capacity and movement cost, speed, and perceived route length. A
completed visit's physical value is computed from water actually gained,
resolver-recorded movement energy, and elapsed ticks. The recorded context
residual is actual minus that choice-time baseline. When the source still
provides the visible amount and the route/cost estimate matches resolution,
these terms cancel to zero. That agreement accounts for the observed zeros; it
does not show that all environments or source histories are predictable.

The fixture has two fixed source locations and controlled replenishment
periods, and the outcome horizon is a short, verified source visit. It is
food-rich and deliberately separates thirst from food access; it does not
create a broad changing source-reliability problem. The one +67 record is a
real correction, not proof of usefulness. At that visit the stored baseline
was -200 and the realized physical value was -133 (water 1, movement energy 3,
3 ticks). The observation table keeps only four recent values, so the resulting
prediction was +16 at ticks 396 and 398. At both choice opportunities the
visible-amount baseline already ranked `(4, 0)` above `(-2, 0)` by 900 points;
adding +16 did not change that order. At tick 400 the same prediction again
did not change the ranking, while the decision was marked `ongoing` and thus
was not a fresh selection.

The analyzer sees 46 two-candidate observations in the trained-online suffix.
Three are `ongoing` continuation ticks for an already selected water target;
they are candidate-visible snapshots, not fresh choice opportunities. The
remaining 43 are fresh two-source choices (10, 4, 8, 7, 5, 9 by seed). The
ranking comparison below is restricted to those fresh choices. This reconciles
the 46 snapshots with the V2 report's 43 online suffix choices. In seed 62204,
the two fresh nonzero-prediction opportunities were ticks 396 and 398; tick
400 was the third nonzero-prediction snapshot, but was an ongoing continuation.

| Seed | Two-candidate observations | Fresh choices | Ongoing continuations | Context-covered fresh choices | Accepted context observations | Residuals | Fresh-choice rankings changed |
|---:|---:|---:|---:|---:|---:|---|---:|
| 62201 | 12 | 10 | 2 | 10 | 24 | 24 × 0 | 0 |
| 62202 | 4 | 4 | 0 | 4 | 21 | 21 × 0 | 0 |
| 62203 | 8 | 8 | 0 | 8 | 23 | 23 × 0 | 0 |
| 62204 | 8 | 7 | 1 | 7 | 29 | 28 × 0, 1 × +67 | 0 |
| 62205 | 5 | 5 | 0 | 5 | 29 | 29 × 0 | 0 |
| 62206 | 9 | 9 | 0 | 9 | 21 | 21 × 0 | 0 |
| **Total** | **46** | **43** | **3** | **43** | **147** | **146 × 0, 1 × +67** | **0** |

The choice-time context lookup is exact-keyed by source position, speed, and
movement cost, averages at most four own observations, and ignores entries
older than 48 ticks. The visible baseline changes with current amount, capacity,
route, and body state; those changing values are not features in the residual
memory key. Thus the current system asks whether a source has a stable additive
correction after those visible factors are accounted for. It is not a direct
model of source reliability conditional on amount and elapsed time. The V2
records have little evidence that this correction can alter candidate ranking.

## Interpretation and next learning target

No current code bug was found in the inspected residual path: the choice stores
the foraging baseline, resolver evidence closes the episode, and only a
verified completed visit (or explicitly matched missing-resource failure) can
produce a context observation. Unresolved, interrupted, or censored attempts
are not turned into successful zero-cost labels. The analysis mirrors the
integer arithmetic, including Rust's truncation toward zero, and compares
baseline candidate ranking with the recorded context-adjusted ranking.

The narrow conclusion is that this V2 fixture does not make the present
residual a useful learning target; it is not evidence that individual source
reliability is unlearnable. A separate, bounded, default-off own-observation
target is proposed in the canonical
[source-reliability contract](../SOURCE_RELIABILITY_TARGET_20261026.md). The
contract labels that quantity as an individual's visible-service forecast
residual, not pure source reliability, and specifies observable inputs,
censoring, memory ownership, and a possible future protocol. It remains a
proposal; no candidate, protocol execution, or new World run was created here.

The present result is compatible with the V2 formal report: trained-online,
trained-frozen, and naive-online produced identical physical trajectories, and
the additional learning effect was not accepted. This diagnosis explains a
mechanism-level reason consistent with those observations, without attributing
the trajectory match solely to one cause or claiming utility from the lone
nonzero label.

## Reproducibility and limits

Analyzer: [`scripts/analyze_water_signal.py`](../../scripts/analyze_water_signal.py).
Tests: [`scripts/test_analyze_water_signal.py`](../../scripts/test_analyze_water_signal.py).
Compact report summary: [`WATER_SIGNAL_DIAGNOSIS_20261026.json`](WATER_SIGNAL_DIAGNOSIS_20261026.json).
Compact machine-readable results, per-seed rows, SHA-256 values, and nonzero
prediction details: [`analysis.json`](../../data/operations/water-signal-diagnosis-20261026/analysis.json).
Raw input: `data/experiments/water-adaptation-v2-formal-20261025/`.

The manifest-declared formal raw hashes were checked by the analyzer. The
manifest identifies source revision `dd7c65ddc90ff83157de49409b5f1a0c6e370570`,
source SHA-256 `168899d4ce675db6bc4795643dcca773a85417df52332a7ed35adabe02098cbc`,
and protocol SHA-256 `8c48d01aeb42aaa1e95add396726c8dcffb01b888ebf811468153b9415b59a76`.
The six inspected evaluation seeds are now seen and should not be presented as
fresh held-out evidence. This is a descriptive reanalysis of one fixture and
six seeds, not an acceptance study or general statement about ecological
adaptation.

Validation performed: `python3 scripts/analyze_water_signal.py` and
`python3 scripts/test_analyze_water_signal.py -v` (4 tests passed). No Rust
build, engine test, experiment, live operation, or World run was performed.
