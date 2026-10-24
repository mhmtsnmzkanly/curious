# Vision selection study — independent design review

**Status: review and design only.** This review authorizes no World run, source
change, seed assignment, or deployment. It addresses the draft
[`HABITAT_SELECTION_PROTOCOL.md`](HABITAT_SELECTION_PROTOCOL.md) using the
seen-seed measurement report and root audit. The protocol needs explicit
measurement and interpretation gates before preregistration; its central
question remains useful as a bounded descriptive study.

## Evidence that should shape the frozen interpretation

The prior genotype measurement used seen engineering seed 123, not formal
evidence. It began with 288 herbivores per vision allele in both arms. At tick
200, living herbivore descendant breeders were:

| Arm | Vision 0 | Vision 1 | Vision-1 share |
|---|---:|---:|---:|
| Expressed | 323 | 90 | 21.8% |
| Neutral marker | 165 | 190 | 53.5% |

For this one descriptive snapshot, the expressed-minus-neutral breeder-share
contrast is **−31.7 percentage points**. Vision-1 expressed range and
maintenance as configured, but the run had one seen seed, divergent RNG paths,
and family-correlated descendants. This is not a selection estimate and must
not be used to retune the study. It does mean the draft's positive directional
prediction must remain a prediction, while the analysis and report remain
direction-neutral. Preserve and report a negative, null, mixed, or positive
formal result as observed.

The measurement report also records 45,395 vision-maintenance units requested,
42,070 paid, and 3,325 unpaid in the expressed arm. Those are aggregate totals,
not per-exposure rates. Genotype-specific raw plant-food totals likewise reflect
different numbers of individuals and lived ticks. Recorded plant encounters
are perceptions, not validated discoveries or consumed food. Do not compare
these totals as rates without their appropriate exposure denominators.

## Recommended primary descriptive statistic

Use the **paired seed block** as the replication unit. For each seed block
`s` and arm `a`, define `p_founder[a,s]` as the tick-zero vision-1 fraction
among herbivore founders and `p_breeder[a,s]` as the vision-1 fraction among
living herbivore descendants that have themselves reproduced at tick 500.
Report the raw counts and the within-block contrast:

```text
D[s] = (p_breeder[expressed,s] - p_founder[expressed,s])
     - (p_breeder[neutral,s]  - p_founder[neutral,s])
```

With a balanced 1:1 founder allocation in each arm, the founder terms are both
0.5; retain them in the calculation and report them to make any imbalance
visible. Report all four `D[s]` values, their mean, median, and range, alongside
each numerator, denominator, and number of represented parent-pair families.
Do not pool individuals across seeds, treat individual breeders as independent
replicates, or turn the four blocks into a thresholded significance test. Four
pairs support a bounded descriptive comparison, not a general selection or
fitness conclusion.

If an arm has at least one qualifying breeder, an endpoint allele frequency of
0 or 1 is a valid observed fixation value. Mark the endpoint `not estimable`
only when its breeder denominator is zero. Preserve the seed pair and its
biological outcome; do not replace it, impute a value, or condition the batch
on retaining both alleles. Report founder survival, all living herbivores,
living descendants, and living descendant breeders separately so the
breeder-conditioned primary measure cannot hide survival or reproduction
differences.

## Required interpretation controls

### RNG divergence and causal scope

The trait toggle is known to change random-number consumption during
reproduction. Pairing by seed and initial allocation controls the declared
starting conditions, but later arms are not exact counterfactual trajectories.
Freeze this implementation path and report it as part of the treatment. The
estimand is the paired difference between these two complete implementation
arms under the registered fixture and seeds. It cannot isolate vision range
from maintenance cost, altered RNG path, or their downstream effects. Do not
describe the difference as a pure causal effect of vision or as general
adaptation.

### Genotype, phenotype, and paid-cost denominators

For each arm and vision genotype, report:

- the number of unique individuals and the number of living genotype-specific
  individual-ticks over which the trait was expressed;
- expressed vision-range distribution and range-by-genotype checks;
- requested, paid, and unpaid maintenance totals **and per expressed
  genotype-individual-tick rates**, with payment reconciliation;
- plant food and water actually acquired per genotype-individual-tick, with
  trace coverage; keep corpse food separate;
- plant/corpse encounter counts as observed perceptions only, never as
  discoveries or intake.

The individual-tick denominator must be explicitly defined (alive and
phenotype-recorded at a tick, not a count of unique individuals or trace rows).
Report missing trace coverage as unknown exposure, not zero. Keep basal
metabolism separate from trait maintenance and do not infer paid cost from body
energy residuals. If genotype-linked individual-tick denominators or paid-cost
joins cannot be reconstructed from the frozen schema, that pathway remains
unknown and the report must not make a per-genotype cost or acquisition-rate
claim.

### Descendant and family accounting

Keep founders, all living herbivores, all living descendants, births, and
living descendant breeders as distinct denominators. Verify each qualifying
breeder's lineage and genotype through the recorded birth and parent joins.
Report counts by arm/genotype and the number of distinct recorded parent-pair
families represented among qualifying breeders. Where family IDs are not
unambiguously recoverable, report that limitation rather than inventing
families. Family summaries are dependence diagnostics, not extra replicates;
the seed pair remains the unit for the primary comparison.

## Gates and run bounds

1. **Freeze the analysis and implementation contract first.** Lock source,
   scenario and founder allocation, protocol, observer schema, trait settings,
   seed roles, output caps, and the primary formula before any 500-tick
   calibration. No adjustment based on the seed-123 negative snapshot.
2. **Use the seen seed only for measurement calibration.** The planned paired
   500-tick seed-123 run can verify phenotype, lineage, exposure denominators,
   cost payment, food-flow, and artifact capture. It is not a formal block and
   contributes no selection evidence. A failure of measurement or lineage
   reconciliation stops the study; it does not justify tuning the trait or
   changing the fixture.
3. **Do not require polymorphism at tick 500.** Calibration must verify both
   alleles at the start, their configured expression/control behavior, and
   auditable inheritance. A later allele frequency of 0 or 1 is data, not a
   calibration failure. A zero breeder denominator is non-estimable for that
   arm, not a failed run.
4. **Keep biological and infrastructure outcomes distinct.** Natural
   extinction or zero breeders is a biological result. Corrupt/incomplete
   capture, failed lineage joins, source drift, or cap breach is an
   infrastructure/measurement failure. Preserve partial evidence and stop;
   no automatic retry, seed replacement, or post-result retuning.
5. **Verify the capture-inclusive budget before formal seeds.** The prior
   measurement produced 80,367,647 raw bytes and 476,252 KiB peak RSS over 400
   ticks. The planned limits are 256 MiB aggregate calibration output (240 MiB
   raw), 1 GiB address space, and, for formal work, four paired seed blocks at
   500 ticks per arm, sequential workers, 1 GiB total output, and 900 seconds.
   Include fixed frames, summaries, stderr, manifests, and audit files in the
   calibration measurement and formal forecast. Stop at a cap; do not drop
   required records or retry. If forecast headroom is insufficient, do not
   instantiate formal seeds; any revised design needs a new review and freeze.
6. **Formal replication is four unseen seed pairs only after calibration
   passes.** Keep each pair's initial genotype allocation balanced and
   spatially interleaved, with mutation fixed at zero as specified. Run
   sequentially and preserve each pair separately. Report every pair, including
   adverse or non-estimable biological outcomes; no additional seeds are
   authorized by this review.

The protocol's current broad interpretations are appropriately cautious. The
remaining gates are to specify genotype-individual-tick denominators and
parent-pair-family summaries in the frozen measurement schema, then verify
that the actual capture and budget forecast support the proposed batch. No
advantage threshold is recommended or preregistered here.

## Evidence references

- [Habitat selection protocol](HABITAT_SELECTION_PROTOCOL.md)
- [Genotype measurement V1 result](reports/GENOTYPE_MEASUREMENT_V1.md)
- Engineering evidence: seen seed 123, source `0c06d723317301eca507ce01ff4419fdd8360485`,
  raw SHA-256 `fc12b41731c2114ec9e32f4f1331af0ce710e9e336583924b8bcfcacfc93bb22`,
  root audit SHA-256 `1262c3997257e866d6bda2eee097536b2c5b3c1940f30b863b6aa0bfbcc9996d`.
  Full raw is losslessly archived; restore the verified archive before using
  readers that require uncompressed JSONL.
