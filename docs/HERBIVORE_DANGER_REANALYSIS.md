# Refuge-lab V1 measurement reanalysis

**Purpose:** correct a validator cross-check defect without rerunning or
changing the registered biological experiment. The original V1 raw output,
probe, protocol copy, and failed terminal manifest remain immutable under
`data/experiments/herbivore-lab-v1-20261023/`.

The formal V1 probe completed all eight registered seeds, but the first
analysis rejected the raw capture while comparing closure enum spellings. The
observer serializes closures in snake case (`target_changed`, `threat_lost`),
while the engine event uses PascalCase (`TargetChanged`, `ThreatLost`); the
validator lowercased the engine string without converting its case boundary.
The probe and biological measurements are unaffected. The raw probe output
and every world tick remain unchanged.

Original-run identity is revision `60ab151`, raw SHA-256
`deee6bbcb78f2af731ef20a476cd43c3f3c8d127bf7eb76a00d14d734eb7f10a`, and
probe SHA-256
`02432ca516a5afbfa50b5f5ce602db5c2c4cabe8ef23d7a018e95cb781c9b88a`. The probe
exited 0 in 0.517 seconds after capturing all eight seeds; the first manifest
is failed only because the validator compared enum names with incompatible
spelling conventions. The first mismatch was seed 45002, tick 65, episode
started tick 61: observer `target_changed` versus engine `TargetChanged`.
Equivalent `threat_lost`/`ThreatLost` mismatches also occurred at seeds 45004
and 45008. Seed 45001, tick 4, was a legitimate ambiguous observer endpoint
and was already handled by the old validator as unresolved rather than
accepted as a matching observation.

## Reanalysis rule

Re-run validation and summaries from the original `raw.jsonl` only. Do not
invoke or rebuild the biological probe, change seeds, edit source decisions,
replay ticks, or tune parameters or thresholds. The corrected validator may
cross-check an engine closure against the observer ledger only when the
observer resolved exactly one visible, currently valid shelter path from the
chosen movement prefix. If the observer recorded `ambiguous_prefix`,
`unmatched`, or `no_move_path`, keep the endpoint unknown and retain the engine
closure in the unmapped-event audit. Do not infer the observer's endpoint from
the engine event, the eventual burrow position, or hidden world state.

Both arms use `episode_audit` as the symmetric episode denominator. Reports
must retain every completed and censored observer closure, show unresolved
endpoints and unavailable comparisons explicitly, and derive first/last
comparisons only from the preregistered endpoint-independent visible context
key. The feature-owned engine event is an additional implementation audit for
the enabled arm, not a baseline substitute and not an outcome source when its
endpoint cannot be mapped uniquely.

## Provenance and bounds

Write analysis products to the new directory
`data/experiments/herbivore-lab-v1-reanalysis-20261023/`; never overwrite the
original V1 run. The reanalysis manifest must record:

- the original run path, raw SHA-256, probe/source/protocol hashes, original
  manifest status, and original validator error;
- the analysis-only validator revision and SHA-256, the exact command, start
  and finish times, exit status, and output hashes;
- that no biological probe was launched, no simulation ticks were run, and no
  source, seed, world state, or raw data changed.

The original registered ceilings remain 1,600 formal ticks plus the separate
200-tick pre-held-out calibration (1,800 total maximum). Reanalysis consumes
zero additional biological ticks. A reanalysis failure leaves the V1 run
rejected; a new experiment or changed policy would require a fresh
preregistration before any simulation.
