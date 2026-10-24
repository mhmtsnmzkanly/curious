# Genotype measurement V1 — engineering result

**Result: engineering measurement audit passed.** The one seen-seed paired
capture completed 200 ticks per arm and passed the independent event, lineage,
and resource-flow audit. This is evidence that the bounded observer recorded
the specified quantities for this run. It is not a fitness test, selection
result, or evidence that vision was useful, harmful, or causally responsible
for any difference between arms.

## Run and provenance

The run used seen seed 123, with the accepted-profile fixture and source
revision `0c06d723317301eca507ce01ff4419fdd8360485`. The two arms started from
the same 608-founder scenario (576 herbivores, 32 carnivores) and balanced
vision genotypes; only extra-trait expression differed as specified in the
frozen protocol. No formal seed role was consumed and there was no live
deployment. The run completed 400 physical ticks in 26.259 seconds, with
476,252 KiB peak RSS and 80,367,647 raw bytes, below the 300-second, 1-GiB,
128-MiB aggregate-output, and 112-MiB raw-output limits.

| Evidence | SHA-256 / size |
| --- | --- |
| Source revision | `0c06d723317301eca507ce01ff4419fdd8360485` |
| Source tree | `09f21808aea2dbee1af7c960d33fffd0cc0b73db` |
| Driver binary | `1444c7ba62223f33efc9be9cd39e5c877bc76e80a8b32c3ed4776e9a02d81ee8`; retained at `/root/curious/bin/releases/curious-observation-7b157af` |
| Frozen protocol | `7227c4ffc0579ce1817aa208acc1632d2495427f7f52201ed82243763d175ed7` |
| Scenario | `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac` |
| Manifest | `cf04d228863c7981c1faebbd381bad9017bd5d26e23471e4bd9e5c8d9ba3ec84` |
| Root audit | `1262c3997257e866d6bda2eee097536b2c5b3c1940f30b863b6aa0bfbcc9996d` |
| Raw JSONL | 80,367,647 bytes; `fc12b41731c2114ec9e32f4f1331af0ce710e9e336583924b8bcfcacfc93bb22` |
| Zstandard archive | 4,607,904 bytes; `47e2d1d9751ed2fd83f1863d2caee0c191f44b38005e99284c4659168b4a3ae5` |

The experiment artifacts are in
[`genotype-measurement-v1-engineering-20261024`](../../data/experiments/genotype-measurement-v1-engineering-20261024/).
The raw archive index verifies the compressed data against the original raw
hash. Restore the JSONL before running readers that require the original
uncompressed path. The exact archive index is
[`raw.jsonl.archive.json`](../../data/experiments/genotype-measurement-v1-engineering-20261024/raw.jsonl.archive.json);
the root check manifest is
[`manifest.json`](../../data/operations/wave6-checks-20261024/manifest.json).

## Audit findings

Both arms began with 288 herbivores of each vision allele, with mobility and
stamina alleles zero; carnivores had vision allele zero. At tick 0, the
expressed arm recorded vision range 6 / maintenance request 0 for allele 0 and
range 7 / request 1 for allele 1. The neutral arm kept base range 6 and zero
vision requests for both marker alleles. The trait settings therefore produced
the declared phenotype and neutral control.

| Arm | Births | Deaths | End living H/C | Living herbivore descendant breeders, allele 0/1 | Vision requested / paid / unpaid |
| --- | ---: | ---: | ---: | ---: | ---: |
| Expressed | 1,193 | 1,172 | 593 / 36 | 323 / 90 (413 total) | 45,395 / 42,070 / 3,325 |
| Neutral | 1,190 | 1,213 | 549 / 36 | 165 / 190 (355 total) | 0 / 0 / 0 |

The requested-versus-paid difference is retained as unpaid maintenance, not
treated as an audit failure. Across the two arms, the independent audit
recomputed all 400 per-tick typed-event totals and food/water stock-flow ticks
(200 per arm). It found zero unexplained stock-flow residual, zero lineage
issues, zero unattributed actor
joins, zero trait-payment issues, and zero observer issues. Birth and death
counts reconciled to the ledger. The observer separately retained herbivore
plant/corpse encounter counts by genotype, with missing trace ticks marked as
unknown coverage rather than zero encounters. These are recorded perceptions,
not validated discoveries or consumed food.

Both arms passed the tick-100 State-plus-RNG serialization/restore equality
check and continued through tick 200. This verifies the recorded round trip and
resume path; it does not compare against uninterrupted execution. Full State
and typed event frames were retained at fixed checkpoints, with event-count
checks on six frames. The non-frame capture is sufficient for the independent
event and lineage reconciliation but is not a full simulation replay or RNG
transition audit.

## Interpretation and limits

This was one previously seen seed, and the arms are not exact counterfactuals:
extra-trait expression can change random-number consumption during later
simulation, allowing trajectories to diverge. The different endpoint
population and breeder counts are descriptive outcomes of these paths. They
do not establish an allele advantage or disadvantage, a causal effect, general
ecological viability, long-run coexistence, or performance in another habitat.
The capture does not validate food discovery, and basal metabolism is not
inferred from trait cost or body-energy residuals. Missing perception coverage
remains unknown.

Root reports final source checks of 246 Rust tests / 0 failures across 47
summary groups, 177 Python tests / 0 failures, release app and probe passed,
and scoped Rust formatting passed. No experiment was rerun.

The runtime resume check recorded a host reboot at 2026-10-24 14:58:05 UTC;
the services started at 14:58:08 UTC. Root did not deploy or restart them.
After the reboot, coexistence and replay were observed active as PIDs 1136 and
1137, with restart count zero; the retention timer was also active. The
`/tmp/curious-wave6-target` cache was already absent on resume, so no cleanup
amount is attributed to this handoff.
