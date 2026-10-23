# Lifetime water calibration: validator reanalysis

## Status

The calibration capture for seed `51099` completed all 1,400 preregistered
physical ticks on source revision `8f58d14265eba2d0a41999be7de91a9b76363686`.
The first Python validation then failed at `initial RNG must equal the
registered seed`. That assertion was wrong: the probe creates `World::with_seed`
and only then replaces its generated map with the fixed lab map. The constructor
populates resources first, consuming the world's seeded RNG. The probe and
biology behaved as committed; this is a validator assumption failure, not a
biological result.

The initial capture remains immutable at
`data/experiments/lifetime-lab-v2-calibration-20261023/`. Its original manifest
is terminal `failed`, and its raw file records 1,400 physical ticks. SHA-256
identities recorded at capture:

| Artifact | SHA-256 |
| --- | --- |
| Original manifest | `a4cd5232b966d5aaebc4f1db083f1a0ccc90b69d83919479e7447f4d2495c20b` |
| Raw capture | `47e57608eb28dc4da3cfed512cf4278b94769be28df42634d6810750f1b80b23` |
| Probe binary | `bf59d25881e85c7a648c3dd9c6af450511dbbcf824bb9d462d52dc523ae7a693` |
| Bound protocol copy | `49d6653a63464a43679d68c275b15c8090e95f57a77d9067ce9fa4a91e5d2047` |

The original run has no `summary.json`; validation stopped before producing one.
Its first manifest and logs are retained as evidence of the failed validator.

## Correct RNG check

`World::with_seed` initializes `Random(seed)`, creates the map, and calls
`populate_resources(0.05, &mut rng)`. For this fixture the map is 5 by 5, so
resource population checks 25 in-bounds cells. Each cell consumes one random
value for the 5-percent spawn roll. For each successful roll it consumes two
additional values for amount and resource type. The seed `51099` has no
successful spawn rolls in those 25 checks, so the constructor consumes exactly
25 calls. The committed LCG returns its previous state, then advances by
`state = state * 6364136223846793005 + 1` modulo 2^64. Replaying those calls
gives `15382464887130188488`, which matches `initial_state.rng` in the raw
capture. Validation must replay this constructor sequence and compare the
recorded state; it must not skip RNG validation or demand the raw state equal
the input seed.

## First zero-tick reanalysis

The first corrected validator read the original raw capture and its provenance
only. It performed no world ticks and launched no probe. Its sidecar belongs in
`data/experiments/lifetime-lab-v2-calibration-reanalysis-20261023/`; it records
the original manifest/raw/probe/protocol/source/runner identities, the corrected
validator identity, expected initial RNG by seed, and the validation and
calibration-gate results. The original calibration manifest/raw/probe/protocol
remain untouched. The sidecar's validation pass value is evidence only when it
matches an independent recomputation from the original raw data.

The first formal preflight then found a separate seed-scan bound mismatch and
stopped before launching a probe or creating a formal run directory. The
registered calibration JSONL line is 19,443,712 bytes, while the shared seed
scanner's default was 16 MiB; the protocol and run manifest allow 64 MiB per
seed record. Because fixing the scanner changes the validator/current source
identity, root is repeating this validator-only analysis against the same
immutable raw capture in
`data/experiments/lifetime-lab-v2-calibration-reanalysis2-20261023/`. See
[`LIFELONG_WATER_PREFLIGHT_REPAIR.md`](LIFELONG_WATER_PREFLIGHT_REPAIR.md).
The first sidecar and all original capture artifacts remain preserved.

The reanalysis does not change the registered fixture, biological parameters,
gates, budgets, or seed plan. Seed `51099` is used only as calibration. Formal
seeds `51001`–`51008` have not been run and remain unobserved; no formal run is
authorized until root reviews the current source-bound reanalysis and calibration gate.
This document makes no claim about behavior or efficacy.
