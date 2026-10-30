# Vision allele selection: accepted fixture, paired arms

**Status: FROZEN BY ROOT.** Root approved the protocol after independent review. Execution remains blocked until root commits this protocol and the reviewed driver source together, freezes source/binary identities, and records a capture-inclusive calibration budget decision.

## Question and prediction

In the accepted qualification-both fixture, does expressing vision allele 0/1
change the allele-1 frequency among living herbivore descendants that have
themselves reproduced by tick 500, compared with the same inherited alleles
acting only as neutral markers?

The original directional prediction is retained: when the additional vision
range helps herbivores locate food enough to repay its maintenance demand,
vision-1 may have higher representation among living descendant breeders in
the expressed arm than in the neutral arm. Opposite, null, fixed, mixed, or
seed-varying outcomes are reported with exact counts and are not grounds for
retuning or rerunning.

The paired seed block is the replication unit. For seed `s`, report
`D[s] = (p_breeder[expressed,s] - p_founder[expressed,s]) -
(p_breeder[neutral,s] - p_founder[neutral,s])`, where each breeder frequency is
the number of living herbivore descendants that have themselves produced an
offspring and carry vision allele 1 divided by all such living descendant
breeders. Both founder proportions should be 0.5; preserve them explicitly so
allocation imbalance cannot be hidden. A denominator of zero makes that arm's
breeder frequency and paired `D[s]` not estimable (JSON `null`). Allele
frequency 0 or 1 with a positive denominator is valid and must be reported as
allele loss/fixation, not as missing data. Report each arm's founder and
breeder numerator, denominator, frequency, and `D[s]`; also report descriptive
mean, median, and range of the four paired values when estimable. Never pool
individuals across seeds or substitute pooled counts for the per-seed results.

For family support, report both (a) distinct recorded immediate parent-pair
families among allele-0 and allele-1 living descendant breeders, and (b) the
number of distinct original herbivore founder ancestors represented among
those breeders. Pair keys are the unordered actual parent ID pair. An
individual may descend from multiple founders, so ancestry-union counts are
non-additive and neither family count is an extra replicate. Preserve exact
ancestry membership in the raw lineage record. A 576-founder bitset (nine
64-bit words) per retained identity bounds ancestry propagation.

This is an expressed-feature-associated contrast, not a pure causal estimate
of vision. The feature setting can consume different RNG draws during
reproduction, and the paths can diverge. Food encounters, food acquisition,
maintenance costs, survival, and reproduction are measured separately when
the events support those claims; do not infer a mechanism from the primary
frequency alone.

## Fixture, controls, and arms

Use only the archived accepted-profile scenario
`/root/curious/data/experiments/reserve-qualification/qualification-both/scenario.json`,
SHA-256
`33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`.
Use the explicit accepted profile from `ECOLOGY_FIXTURE_V1`: mode `both`,
companions enabled, mate rendezvous and reserve opportunities enabled, water
residual disabled, water learning and updates enabled, hunt learning and
refuge outcomes disabled. Do not alter the scenario, bodies, resources,
positions, or habitat.

Create a fresh independent world for each arm and seed, with exactly 500 World
ticks. Both arms use genetics enabled, mutation rate zero, and identical
scenario bytes, positions, policy, bodies, resource map, and founder genomes.
All mobility and stamina alleles are zero; all carnivore vision alleles are
zero. Herbivore founders have exactly 288 vision-0 and 288 vision-1 genomes.
Assign them with the existing deterministic allocation: 4x4 tiles by inclusive
scenario bounds, row-major tiles, founder sorting `(y, x, original index)`,
alternating alleles globally across tiles. Record each founder ID, coordinates,
tile, genome and per-tile allele counts. Check initial State before tick 1.

- **Expressed:** `extra_traits_enabled=true`. Record actual life-state vision
  range and requested maintenance by genotype. Verify allele expression
  against current engine rules before tick 1 and at every observed state.
- **Neutral marker:** `extra_traits_enabled=false`, same genomes. Verify all
  founders retain base vision range and zero vision/stamina trait requests.

There is no post-tick-0 founder, body, resource, or genotype reset. Use normal
AI decisions and resolver execution. Capture full perception in memory only;
serialize compact observed encounter counts from actual `DecisionTrace` values
and never feed audit data to AI. Repeated food entries remain repeated
encounters; corpse and plant counts stay separate. Missing trace or perception
means unknown coverage, not zero.

## Roles and order

1. **Calibration:** exactly one paired seed-123 run, 500 ticks per arm. Seed
   123 is seen and previously used for the separate 200-tick measurement
   engineering run. This 500-tick calibration checks measurement/capture and
   cannot tune the feature or establish formal evidence. Preserve both results
   even if viability or the primary endpoint is negative.
2. **Formal:** four paired seeds, run once in this order: `54001`, `54002`,
   `54003`, `54004`. Root's frozen seed inventory reports these unseen after a
   scan of 1,226 experiment text files. Inventory SHA-256:
   `71e421a91eeb3163ae17b6db160083bf36f0637cb557c22ee196d19ecd1da579`;
   retained at `data/operations/wave7-checks-20261024/seed-inventory.json`.
   Before formal execution, root must recheck the inventory, record an
   immutable copy/hash in the formal manifest, and freeze source, protocol,
   binary, fixture, runner, and output identities. No other seed is allowed.

Formal seed allowlist: `54001, 54002, 54003, 54004`.

The driver accepts only `calibration 123` or `formal` with one of the four
listed formal seeds. It refuses World construction unless this document is
marked `FROZEN BY ROOT`. Each invocation must be launched by the root
supervisor, which verifies the frozen source/binary/protocol and exact scenario
SHA before spawn, records a durable `prepared` then `running` manifest, captures
stdout/stderr/status and timestamps, enforces resource limits, exclusive
output paths, and both per-pair and whole-phase aggregate caps. The driver does
not independently hash the scenario or enforce aggregate caps; a direct
invocation outside that supervisor is not a preregistered run. No calibration
or formal World runs are permitted before root commits the reviewed protocol
and driver together. Pure hand-built fixtures may run before freeze; they must
not construct a World or advance ticks. Root audits source/binary/protocol
identity before each role. Each formal seed is run once; infrastructure failure
stops the batch, preserves partial evidence, and triggers no retry or
replacement. A completed seed with a biological negative does not stop the
remaining registered formal seeds.

## Outcomes, audit, and calibration gate

For each arm/seed report the primary numerator, denominator, frequency,
per-founder-ancestor support, births, matings, deaths, generations, species
alive, founders, living herbivores, descendants, and living descendant
breeders, with allele counts for founders, all living herbivores, all living
descendants, living descendant breeders, and births. Check child genome,
species, generation and both parents against actual mating/birth events; a
missing or unattributed join is an audit issue. Mutation is zero, so child
genomes must match one actual genetic parent's genome. Report any violation as
an issue and stop the run; do not reinterpret it as mutation.

Use `genotype_audit` for individual/genotype costs, acquisitions, trace counts,
birth/death/mating/lineage evidence; use `ecology_audit` for per-tick food and
water resource-flow accounting. Reconcile aggregate observer totals to actual
typed events. Record vision/stamina requested, paid, and unpaid separately;
payment below request is a valid shortfall, while payment above request or
typed-event/observer disagreement is an issue. Record plant and corpse food,
water acquisition, and encounter traces by full genotype. Perception counts are
not validated discovery. Basal metabolism and body-energy residuals remain
unknown; do not infer them from trait costs. Do not claim causal trait benefit.

For each arm, count living genotype-specific individual-ticks from the exact
pre-tick State: `life.is_alive()` and phase is not `Corpse` or `Removed`, which
includes sleeping individuals. Stream tick-level counts by full genotype and
retain their cumulative sums. Report unique individuals and these
individual-ticks separately. Normalize requested/paid/unpaid vision
maintenance, actual plant/corpse food and water acquisition by the matching
genotype individual-ticks; these are descriptive per-exposure rates, not
causal estimates. Plant/corpse encounters need their own trace-coverage counts;
never use trace rows as a substitute for individual-tick exposure. Missing
perception remains unknown and is reported alongside encounter counts.

Retain each tick's species and genotype population counts, resource stocks and
flows, typed event totals, compact event rows needed to reconstruct costs,
meals, water, mating, births/deaths, child genome/generation/parents, lineage
ancestry, and trace rows with explicit missing coverage. Retain complete State
plus all events only at fixed frames `t0`, `t1`, `t100`, `t200`, and `t500`.
At t100, serialize/restore State plus RNG, compare equality, then continue that
same arm to t500. This checks a recorded round trip, not equivalence to an
uninterrupted trajectory or RNG transition replay. No frame may be dropped
after seeing output. State precisely that movement/action event payloads and
full perception are not retained on non-frame ticks; the evidence is not a
full replay.

Calibration must pass measurement gates before formal seeds are considered:
source and fixture identity; balanced initial genomes and declared phenotype
expression/neutrality; actual event-backed cost/acquisition totals; complete
parent-child-generation joins with zero unexpected transmission; no observer,
lineage, or unattributed-actor issues; and zero unexplained per-tick food or
water stock-flow residuals/replacements. Additionally, **each arm must have at
least one living descendant breeder of both species at tick 500.** This is a
biological viability gate for calibration. It does not require both herbivore
alleles to remain at tick 500; allele loss is valid observed biology. A zero
herbivore-breeder denominator is also biological/non-estimable and blocks
formal work. Failure of any measurement gate or this calibration biological
gate blocks formal work. There is no tuning, extension, or repeat calibration.

In formal runs, natural extinction, zero herbivore-breeder denominator, or
allele loss is a biological result. Report an `NA` paired endpoint only when an
arm's denominator is zero; otherwise report frequency 0 or 1 exactly. Do not
replace seeds. Formal seeds continue after biological negatives and stop only
for registered infrastructure/source/capture/audit failures.

## Fixed frames, caps, and stopping

Use separate new output directories for calibration and each formal seed. The
driver creates output files exclusively and flushes partial JSONL/status on
failure. Hard capture caps are 50,000 identities per arm, 20,000 events per
tick, 50,000 lineage events per arm, 2,000 detailed audit/issue rows per arm,
and 240 MiB raw per paired seed. Preflight cap projections before mutating
bounded audits. Stop immediately on source/profile mismatch, incorrect founder
or phenotype state, observer/lineage/transmission error, unreconciled food or
water flow, resume mismatch, cap breach, or infrastructure failure. Do not
label truncated evidence complete.

Calibration limits: 300 seconds wall, 1 GiB address space, 256 MiB aggregate
output (at most 240 MiB raw; reserve at least 16 MiB for status, manifest,
stderr and audit metadata). Formal batch limits: 900 seconds wall total across
sequential workers, 1 GiB address space per worker, 1 GiB aggregate output for
all four paired seeds, and 240 MiB raw per paired seed. Every formal output
must fit both per-pair raw and aggregate limits. Use one Cargo job and the
root-designated target directory.

The previous measured 400-tick paired capture wrote 80,367,647 bytes in
26.259 seconds. Linear planning for a 1,000-tick pair (500 per arm) is about
201 MB raw and 65.6 seconds; four pairs are about 804 MB and 263 seconds.
These forecasts are approximate because population, lineage, event, and trace
volume vary. The calibration capture-inclusive result is the preregistered
forecast basis. Before formal execution, root records measured raw, metadata,
and wall time and applies these gates:

- Forecast raw per pair as `calibration_raw_bytes * 1.20`; it must be at most
  240 MiB.
- Forecast total raw as `4 * calibration_raw_bytes * 1.20`; after adding
  measured/per-run metadata, stderr, manifests, audits, and status, it must fit
  within the 1-GiB formal aggregate cap.
- Forecast batch wall time as `4 * calibration_elapsed_seconds * 1.50`; add the
  separately measured compile/runner/audit overhead as applicable, and require
  the total to fit within 900 seconds.

The calibration capture already includes the fixed extra t200 frame, so the
forecast adds no further frame allowance. If any gate fails, do not instantiate
a formal seed. Request a separately reviewed protocol update; do not tune,
drop required records, or exceed a cap.

For estimable formal pairs, report `N_valid`, mean, median, and range of the
paired `D[s]` values as descriptive summaries only. Retain all four per-seed
results, including `NA`; do not pool individuals, claim significance, or apply
a positive threshold.

## Interpretation

Report the original positive prediction, every per-seed delta, raw allele and
family counts, missing coverage, and adverse/null/mixed outcomes. A positive
primary delta is an expressed-feature-associated observation under these
fixture conditions, not proof of vision utility, a food-discovery mechanism,
pure causal effect, long-term coexistence, habitat robustness, or general
fitness. A negative or null outcome is retained without retuning. Any follow-up
trait, habitat, or parameter question requires a new protocol and root review.
