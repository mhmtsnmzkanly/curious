# Habitat and heritable-trait selection — design draft

**Status: design only. Not preregistered and not executable.** This draft
specifies a bounded next study for root review. It assigns no seeds and does not
authorize a build, World construction, calibration, formal run, service change,
or live deployment.

## Question

In the current accepted-profile fixture, does expressing a naturally inherited
vision allele change herbivore descendant-breeder representation over 500 ticks,
relative to the same inherited allele acting only as a neutral marker? This is a
selection question for one allele, one species, and one short horizon. A second
habitat profile is deferred to a separately reviewed protocol. This is not a
test of general adaptation, all traits, long-term coexistence, or open-ended
evolution.

The directional prediction is deliberately weak: if additional vision helps
herbivores locate food enough to repay its maintenance cost, vision-1 should
gain more representation among living herbivore descendants that have
themselves reproduced in the expressed-trait arm than in the neutral-marker
arm. A result in the opposite direction, no measurable difference, or a result
that varies across seeds is informative and does not trigger retuning.

## Why this is the next study

The current V17 fixture report establishes only a short-horizon viability
baseline under explicit accepted-profile settings: calibration 53099 ran for
200 ticks, and seeds 53001–53003 each ran for 500 ticks. Those seed roles are
seen and must not be reused as held-out evidence. The baseline had extra traits
disabled; therefore it says nothing about trait fitness. The old 80-world trait
matrix did not produce breeding descendant lineages and did not isolate a
selection effect. Repeating a broad matrix before establishing polymorphic
founders, expression, paid costs, transmission, and descendant reproduction
would leave the same causal gap.

The existing code provides a narrow test candidate. `Genome.vision` is a
0..2 inherited allele; with extra traits enabled, expression changes
`vision_range` and assigns a per-tick `vision_metabolic_cost`. Metabolism pays
basal demand first and records requested and paid trait maintenance separately.
Offspring inherit one parent's genome, with optional mutation. This supports a
single-allele study, but does not itself prove the trait changes food discovery,
that requested maintenance is fully paid, or that allele differences cause a
reproductive advantage. Those must be measured from Worlds and audit events.

## Proposed design

### Arms and controlled contrast

Use only the accepted-profile habitat: the archived qualification-both fixture
and explicit current profile from `ECOLOGY_FIXTURE_V1`. In both arms initialize
herbivore founders with a 1:1 vision-0/vision-1 split. Fix mobility and stamina
alleles at 0 for all founders, and fix carnivore genomes. Assign both vision
alleles evenly across spatial strata and founder ordering; do not cluster
genotypes by location or assign all members of a family one allele. Set
mutation to zero in both arms. Do not reset bodies or add founders after tick 0.

- **Expressed-trait arm:** enable extra traits. Vision alleles change the
  expressed vision range and carry their configured maintenance cost.
- **Neutral-marker control:** use the same genomes, founder positions, species,
  policy, resource map, and seed, but keep extra traits disabled. Vision alleles
  are still inherited and counted but do not alter phenotype or physiology.

This paired contrast controls for initial allele transmission and the neutral
marker drift observed under the same fixture, seed, and founder allocation. A
repeatable arm difference supports an expressed-feature-associated change in
this experiment. It does not identify a pure vision-phenotype effect: the
feature toggle also changes RNG draws during reproduction, and later random
streams and world trajectories diverge. The measured contrast therefore
includes that implementation-path difference. It does not isolate the
mechanism (food discovery versus maintenance, survival, or reproduction) or
prove advantage in another habitat. Record the divergence and do not describe
the arms as exact coupled counterfactuals. Use multiple paired seed blocks to
distinguish a repeatable arm difference from one-seed founder/lineage drift.

Formal seed values and roles remain **UNSET** until root audits manifests and
freezes the protocol. Four paired seed blocks at 500 ticks per arm are only an
initial design proposal, not a budget guarantee, adequate acceptance sample, or
reliability claim. Root must decide sample size after the capture-inclusive
feasibility forecast and review.

### Outcomes

The primary outcome is the expressed-minus-neutral difference in change from
tick-0 founder frequency to tick-500 frequency of vision-1 among living
herbivore descendants that have themselves produced offspring, reported for
each paired seed block. Also report the numerator, denominator, and number of
families represented in each arm; a proportion without counts is not
interpretable. Do not pool founders with descendants or count a living
nonbreeder as a reproducing descendant. A seed with too few relevant
descendants or extinction is a biological outcome with an undefined or
non-estimable primary measure, not a reason to replace the seed.

Required interpretation measures:

- allele counts among founders, all living herbivores, all living descendants,
  living descendant breeders, and births, with parent-to-child transmission
  checked against lineage records;
- actual mating and birth counts, generations, species alive, deaths, and
  living descendant breeders, separated by species and arm;
- expressed vision range and its observed distribution, plus food discovery /
  acquisition events by genotype where the raw event contract supports
  attribution;
- trait maintenance requested, paid, unpaid, and residual body-energy changes
  by genotype if attribution is available; basal metabolism remains a separate
  component and must not be counted again as trait cost;
- per-tick food stock, regeneration, and consumption accounting, and the exact
  initial/final resource totals;
- runtime, peak RSS, raw bytes, and audit row/frame counts.

Treat action-cost attribution, basal metabolism, genotype-specific food
opportunities, and any missing event linkage as unknown where not directly
recorded. A higher allele frequency alone is not proof of a useful phenotype;
the claim requires verified expression, transmission, paid maintenance,
reproduction, and living reproducing descendants.

### Gates and stopping rules

Before a formal phase can be considered, a separate root-reviewed technical
check must confirm the intended V18/current candidate source and verify its
default-off behavior, checkpoint/resume behavior, and accepted-profile control
settings. The old V17 fixture result cannot stand in for this check.

Then one calibration role per arm may be used only to confirm measurement and
observer validity, not to tune the trait. Calibration must show: both founder
alleles are present and spatially balanced; allele expression matches the
configured genomes in the expressed arm and remains phenotypically neutral in
the control; child transmission is auditable; trait payment requested/paid
totals reconcile; births and lineage events reconcile; and resource-flow
accounting passes. If those conditions cannot be measured, the formal selection
question is non-identifiable and formal work must not begin.

For each formal seed, separately report biological and infrastructure outcomes.
Natural extinction is a biological negative, not a runner failure. Stop the
registered batch on source/protocol mismatch, corrupt or incomplete audit,
unreconciled resource or lineage accounting, cap breach, or an infrastructure
failure; preserve all partial evidence. Do not replace a failed seed, add
founders, relax a gate, alter trait costs, change the resource distribution,
or automatically rerun after seeing outcomes.

Do not claim a selection result if any of the following holds: the starting
allele balance is absent; the allele is not expressed; there is no measurable
trait exposure or paid-cost evidence; parent-to-child transmission cannot be
verified; the relevant descendants do not reproduce; allele-specific counts
cannot be reconstructed; or the audit does not distinguish natural extinction
from infrastructure failure. No advantage claim is permitted if the only
evidence is founder survival, mating exposure, a single generation, or
genotype-frequency change without verified breeding descendants.

## Run and evidence budget proposed for root review

Use the already measured current-fixture probe cost as a planning reference,
not a guarantee: 200 calibration ticks took about 14 seconds of probe time;
three 500-tick formal seeds took about 134 seconds and produced 94,073,278 raw
bytes. That is about 62.7 kB/tick, or 31.4 MB per 500-tick world under that
source and those settings. Four 500-tick worlds per arm project to about 125.4 MB raw before any
additional genotype audit rows, stderr, manifests, or capture overhead. Both
arms together project to about 250.9 MB raw. One four-seed arm already leaves
little room under a 128 MiB aggregate-output cap; this is not evidence that
four paired seeds fit.

Proposed hard cap per phase: 300 seconds wall time, 1 GiB address space, and
128 MiB aggregate output. Measure capture-inclusive output and runtime during
calibration, including stderr, manifests, audit summaries, and fixed-frame
capture. Forecast the full proposed arm/seed batch from those measurements with
headroom; calibration measurements do not guarantee formal fit. If the four
paired seed blocks do not fit with headroom, root must narrow the sample or
revise the cap in a separately reviewed protocol before preregistration. Do
not silently drop frames, omit raw events, or exceed a cap. Cap raw detail rows,
retain per-tick stock/flow counters and lineage totals, and retain full
state/event frames only at fixed preregistered checkpoints. State exactly which
events are omitted so no audit is described as a full replay.

Use separate output directories for calibration and formal evidence. Freeze
source revision/content hash, scenario hashes, protocol hash, runner hash,
command and flags, seed roles, raw schema, output caps, and analysis before any
formal seed is instantiated. Seeds in this draft are intentionally unset.

## Decision after this study

This bounded study can estimate the expressed-versus-neutral allele-frequency
contrast for the selected fixture, horizon, and paired seeds, alongside
observed expression and energetic cost. It cannot identify the exact
physiological/behavioral mechanism or show long-term stability or habitat
robustness. A null, mixed, or adverse result is reported as such; it does not
authorize another trait, habitat, or parameter matrix. Any follow-up requires
a new question, protocol, and root review.

## Known measurement gaps to close before preregistration

- Verify that the current candidate records trait request/payment per
  individual/genotype or add a root-reviewed audit extension; species-level
  totals alone cannot estimate genotype-specific costs in this polymorphic
  design.
- Verify a direct, attributable measure of food encounters/discovery by
  expressed vision. If unavailable, report this pathway as unknown and limit
  conclusions to allele transmission and descendant representation.
- Confirm that the existing lineage audit can join founder genotype, both
  parents, child genotype, expression, births, and descendant-breeder status
  without inferring missing identifiers.
- Confirm checkpointed state carries genomes and expressed life costs
  consistently across resume; do not assume serialization support proves
  behavioral continuity.
- Verify the neutral-marker control's lineage and allele accounting remain
  intact with extra traits disabled, and measure how differing RNG draw counts
  affect arm coupling after reproduction.

## Deferred habitat question

A mild food redistribution that preserves total initial and renewable food
could later test whether the expressed-versus-neutral contrast changes with
spatial resource unevenness. It is deliberately excluded from this first study:
it adds a second environmental contrast before the trait counterfactual and
capture-inclusive budget are validated. That question requires its own frozen
transformation, paired seeds, resource-accounting checks, and root-reviewed
protocol.
