# Operations-readiness timing correction — 26 October 2026

## Finding

The original offline probe at
`data/experiments/operations-readiness-20261026/run.json` times 120 calls to
`World::tick`, but its p50/p95 helper selected ranks from the original execution
order instead of a sorted copy, and the individual durations were not stored.
The stored p50 (`41,701 µs`) exceeds the stored
p95 (`35,965 µs`), so **both percentile fields are invalid and must not be
interpreted**. Individual tick durations were not persisted in that run, so the
percentiles cannot be recomputed from its output. The original manifest and
timing file remain unchanged.

The total (`4,686,542 µs`), arithmetic mean (`39,054.5167 µs`) and observed
maximum (`48,774 µs`) are order-independent fields from the same 120 timed
`World::tick` calls and remain usable within this one V12-derived, `off`-mode,
120-tick offline probe. The operation's scope, fixed input identity, migration,
State/RNG round-trip and isolated storage-smoke results are unaffected by the
percentile calculation defect.

## Correction for future runs

The source helper now sorts a copy before selecting the existing nearest-rank
percentile and includes an unsorted-input unit test. Future `run.json` files
also retain the per-tick duration vector so p50/p95 can be audited from raw
samples. This correction does not rewrite or qualify the historical run; no
World was rerun for it.

## Evidence and limit

- Historical output: `data/experiments/operations-readiness-20261026/run.json`
- External process timing: `data/operations/consolidation-20261026/operations.time.txt`
- Corrected source: `examples/operations_readiness.rs`

The `/usr/bin/time -v` file measures whole-process resource use. It does not
contain the per-tick observations needed to recover percentiles. Do not infer
tick p50/p95 from process elapsed time or replace the invalid fields with
estimates.
