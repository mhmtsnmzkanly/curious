# Herbivore danger and refuge experience — bounded reanalysis

**Result:** the original probe captured all registered seeds, but its first
summary attempt failed on a validator naming mismatch. A corrected validator
then passed on the same raw file, with no new simulation ticks. The validated
paired physical outcomes are identical across the two branches. The lab shows
recorded refuge learning updates, but no change in decisions, physical state or
survival in these eight histories; it does not establish a mortality benefit.

## Registered study and provenance

The preregistered study compared default off refuge memory with the opt in
individual outcome memory. It used one continuous herbivore per seed, a
controlled predator, two burrows, current perceived threat context, resolver
costs, and a symmetric episode ledger. Neither branch reset or replaced its
subject. Branches ran for at most 100 ticks and ended at natural death. The
registered held out seeds were 45001–45008; the manifest's seed audit found
each unused before the run.

Calibration seed 45999 completed both branches. The independent Python
validation passed and counted 134 physical ticks under its 200 tick ceiling.
Calibration raw SHA 256:
`77ed0d56d576daead06cfdb65ec553880f2a7c7c30ca02aee2c54b4afe972756`.
Calibration was not used to tune any source, parameters, seeds or thresholds.

The formal probe used clean revision
`60ab1519022bb50dfe91977892b67be3da178e9f`, binary SHA 256
`02432ca516a5afbfa50b5f5ce602db5c2c4cabe8ef23d7a018e95cb781c9b88a`, and
protocol SHA 256
`537901bac12ac406b4895f315b09ee60f38a512e5e10ab0d5631c0e836c4a9d6`. It
captured all eight seeds in 1,148 ticks. The original manifest remains
`failed`: the probe exited 0, but the first summary validator compared
snake_case observer closure labels with PascalCase engine labels and rejected
`target_changed` versus `TargetChanged` at seed 45002, tick 65. The original
raw SHA 256 is
`deee6bbcb78f2af731ef20a476cd43c3f3c8d127bf7eb76a00d14d734eb7f10a`.

Reanalysis revision `976683f` corrected that comparison and validated the
original raw rows. It did not launch the probe, change biology or add any
simulation ticks. Its validator SHA 256 is
`e6a3e6264e2bf205b14a4754e9ff838bf2afa5633d4664819762a831b579f28b`, analysis
protocol SHA 256
`a96a19e7e876b3f1725178747b45c95c79145d03c73b284b7f6db9cd026b7d66`, and
summary SHA 256
`7d199ee26918e582e8551e61dc9bd5402900952dc9b87b97aee8d530fd33085e`. The
original failure, corrected analysis and their separate identities are
preserved under `data/experiments/herbivore-lab-v1-20261023/` and
`data/experiments/herbivore-lab-v1-reanalysis-20261023/`.

The completed calibration plus formal capture used 1,282 of the registered
1,800 combined physical tick ceiling; reanalysis used zero biological ticks.
These eight seeds are now seen. This reanalysis is not an independent second
study.

## Episode outcomes

The shared observer ledger counted every closure, including death, loss of the
currently eligible threat within two steps (it may remain visible farther
away), target change, and unresolved refuge prefixes. For each branch,
it records 65 closures: 52 resolver verified safe arrivals and 13 censored or
unresolved closures. The safe arrival rate is therefore **52/65 (80.0%) per
branch**, using the registered closure denominator. No endpoint is imputed for
an ambiguous prefix.

| Seed | End cause | Ticks per branch | Baseline safe / closures | Refuge memory safe / closures | Censored or unresolved per branch | Movement energy per branch | Whole life threat damage per branch |
|---:|---|---:|---:|---:|---:|---:|---:|
| 45001 | Starvation | 67 | 8/10 (80.0%) | 8/10 (80.0%) | 2 | 69 | 60 |
| 45002 | Starvation | 82 | 7/10 (70.0%) | 7/10 (70.0%) | 3 | 72 | 0 |
| 45003 | Combat | 67 | 5/6 (83.3%) | 5/6 (83.3%) | 1 | 71 | 60 |
| 45004 | Dehydration | 80 | 7/8 (87.5%) | 7/8 (87.5%) | 1 | 71 | 0 |
| 45005 | Combat | 67 | 6/8 (75.0%) | 6/8 (75.0%) | 2 | 70 | 80 |
| 45006 | Combat | 69 | 8/10 (80.0%) | 8/10 (80.0%) | 2 | 70 | 60 |
| 45007 | Combat | 62 | 6/7 (85.7%) | 6/7 (85.7%) | 1 | 68 | 100 |
| 45008 | Dehydration | 80 | 5/6 (83.3%) | 5/6 (83.3%) | 1 | 71 | 0 |

The damage column is the total threat attributed damage across the entire
branch, not a per closure rate. Separately, the observer episode ledgers sum to
360 threat damage, zero other attributed damage, 134 unattributed health loss,
and 101 movement energy per branch. The reanalysis summary's
`threat_damage_per_closure` divides the whole life damage total by closure
count; this report does not interpret that quotient as conditional episode
risk. Across branches after the first safe arrival, resource seeking used 369
decision ticks, 366 movement energy, and acquired 295 food and 80 water per
branch. These totals are descriptive costs, not a combined reward.

All sixteen branch histories ended in natural death between ticks 62 and 82;
the paired branch had the same cause and duration for every seed. Death causes
were combat, starvation or dehydration as shown above. Every
paired branch began from the same state except for the feature flag. Root's
independent comparison found the same decisions, resolver physical events,
controller cursor, full physical state and RNG across all 574 paired ticks,
after excluding only the four new refuge memory fields and the
feature owned `RefugeEpisodeEnded` audit events. Thus the memory updates did
not change observed shelter choices, physical costs or these deaths.

## Feature updates and observer limits

The opt in engine emitted 55 closure events marked `safe_arrival=true` across
the eight seeds. The shared observer independently verified 52 safe arrivals;
it left three engine arrivals unmapped because the chosen `Move` prefix was
shared by two visible refuge routes: seed 45001 tick 4, seed 45005 tick 10 and
seed 45006 tick 10. Those endpoints remain unknown in the common observer
outcomes.

| Seed | Engine safe arrival events | Observer verified safe arrivals | Engine events with observer endpoint unresolved | Terminal memory completed/safe counters retained |
|---:|---:|---:|---:|---:|
| 45001 | 9 | 8 | 1 | 8 |
| 45002 | 7 | 7 | 0 | 7 |
| 45003 | 5 | 5 | 0 | 5 |
| 45004 | 7 | 7 | 0 | 7 |
| 45005 | 7 | 6 | 1 | 7 |
| 45006 | 9 | 8 | 1 | 8 |
| 45007 | 6 | 6 | 0 | 6 |
| 45008 | 5 | 5 | 0 | 5 |

Root's source trace confirms that the core selects and stores its full endpoint
before truncating the movement path to the subject's
speed, then records arrival only when the resolver leaves the subject at that
stored burrow endpoint. The internal selected target can therefore be known to
the agent while remaining unresolvable from the observer's current path prefix.
The reviewed paths are [refuge_learning.rs](../../src/entity/refuge_learning.rs)
and [refuge_outcomes.rs](../../src/systems/refuge_outcomes.rs).
The report keeps the 55 core records separate from the 52 observer verified
arrivals; it does not fill in the three observer endpoints.

At termination the bounded eight record memory contains 53 aggregate
completed/safe counters. This is not a cumulative lifetime update total: the
eight record cap had evicted older records in seeds 45001 and 45006. The
terminal table count must not replace either the 55 core event total or the 52
common observer verified safe arrivals.

Only seed 45002 had a repeated genuinely matching visible context in either
branch. In both branches its first and last matching episodes closed at ticks
66 and 82; both were censored (`threat_lost` then `died`). A censored closure
with `safe_arrival=false` means safety was not verified, not that an unsafe
route was established. The matching context
was the same visible carnivore, relative bearing and distance, subject energy
and stamina buckets, and two visible burrow alternatives. The other seven
seeds had no repeated same context before death and remain unavailable; no
individual was substituted.

## Interpretation

The lab establishes that the feature recorded refuge outcome data and that its
decision/persistence path left the paired physical history unchanged in this
sample. It does not demonstrate improved escape behavior or reduced mortality:
all subjects died, both branches were physically identical, and the sole
repeated context comparison was censored in both. It does not establish a general
learning benefit, an age advantage, long term safety, or ecological balance.
Safe arrival is a route outcome, not a survival outcome. The short continuous
lives and the single repeated context make the study a bounded mechanism screen
only.

The [compact audit JSON](HERBIVORE_DANGER.json) links the original failed
manifest, calibration, reanalysis and physical comparison proof. The complete
raw traces remain in [the original run](../../data/experiments/herbivore-lab-v1-20261023/raw.jsonl).
The [reanalysis summary](../../data/experiments/herbivore-lab-v1-reanalysis-20261023/summary.json),
[analysis protocol](../../data/experiments/herbivore-lab-v1-reanalysis-20261023/analysis-protocol.md),
[formal manifest](../../data/experiments/herbivore-lab-v1-20261023/manifest.json),
and [preregistration](../HERBIVORE_DANGER_PROTOCOL.md) are also preserved.
