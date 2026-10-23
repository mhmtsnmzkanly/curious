# Ecology cycles and paid-trait laboratory preregistration

Status: preregistered design; evaluation has not started. Source revision under
development: `15356f14f243eba4f5e074e5e251a482d5345d51`. This is a bounded
mechanism and fitness study, not an ecological qualification or deployment
protocol. Update the source identity and commit this protocol before any
registered evaluation.

## Questions and hypotheses

1. A resource site's optional availability schedule produces a deterministic,
geographically staggered renewal pattern from world tick and site coordinates.
The result does not depend on population, entity order, or RNG state. During an
off-window, renewal stops; existing resource remains and ordinary consumption
can deplete it.
2. With `extra_traits_enabled=true`, bounded inherited vision and stamina
alleles change expressed body capabilities, incur the registered energy
maintenance cost through metabolism, and can reach breeding descendants. No
direction of lifetime fitness advantage is assumed.

The primary ecological comparison is descriptive: descendant breeding and
survival by genotype across static and cyclic resource profiles. It cannot
establish general evolutionary advantage, intelligence, or mixed-species
coexistence.

## Frozen implementation profile

The schedule is optional per resource site and is absent by default. The
registered schedule uses period 12, four active phases, geographic phase step
1, and site offset 0. For integer coordinates `x,y`, define the spatial phase
as Euclidean modulo `x + 3*y + offset` by period; a site's renewal is enabled
when `(tick + spatial_phase) mod period < active_ticks`. Tick and coordinate
arithmetic must be checked or use bounded modular arithmetic. The schedule
does not change resource capacity, replenishment amount, consumption, or cell
type rules. A separate unscheduled profile is the control.

`Genetics.enabled` remains true with its existing meaning and defaults.
`extra_traits_enabled` is false by default. The control genotype is mobility 0,
vision 0, stamina 0. The preregistered arms are mobility 1/vision 0/stamina 0;
mobility 0/vision 1/stamina 0; mobility 0/vision 0/stamina 1; and mobility
0/vision 1/stamina 1. Each vision allele step adds one vision-range unit;
each stamina allele step adds four maximum-stamina units. Each expressed step
costs one energy per living tick. Traits are bounded to 0..2. The experiment
sets mutation to zero to attribute offspring effects to inheritance; normal
bounded mutation remains a separate implementation property. Trait cost is
paid in the metabolism system and recorded only when positive. No extra-cost
event is emitted when the opt-in flag is closed.

## Arms and environment

Each replicate is an independent, fresh, 12x12 herbivore-only world with eight
same-genotype, same-age founders, the same base body values, and a fixed
resource layout declared in a committed fixture. Expressed capabilities may
differ only as specified by genotype. There are no spawns, interventions, body
resets, food additions, or population rescue after tick 0. The five genotype
arms above, including the zero-allele control, are crossed with two environment
arms:

- static: the same renewable sites without schedules;
- cycle: the same sites with the registered per-site schedule.

Every arm receives the same seed within a replicate block. RNG divergence after
behavior differs is expected and recorded; pairing applies to starting seed and
world, not to identical later random draws. `extra_traits_enabled` is true in
all trait-study arms, including the zero-allele control, so the allele gate is
held constant.

## Seed roles, trial limits, and stopping

Before evaluation, audit all existing experiment files. Calibration seeds are
`50041..50044`; run each once through all ten arms to validate serialization,
exact schedule timing, event accounting, and runner output. They may not be
used to tune trait sizes, costs, ecological settings, or analysis thresholds.
Held-out evaluation seeds are `50001..50008`; each seed is run once in each of
the ten arms. Scan every existing experiment JSON, JSONL, CSV, manifest,
config, and protocol for these seed values and save the scan result in the new
manifest. If any held-out or calibration seed is already present, stop before
evaluation and return this protocol for a new preregistration review; do not
substitute a seed after seeing outcomes.

Each evaluation world runs for exactly 2,000 ticks unless the world naturally
has no living individuals; in that case continue the empty world to tick 2,000
to retain the fixed duration. There is no parameter search, recovery branch,
long sweep, or automatic follow-up. The cap is 80 evaluation worlds / 160,000
ticks, plus 40 calibration worlds / 80,000 ticks; each batch is limited to 300
seconds wall time and 1 GiB output. Stop the entire batch on a source/protocol
hash mismatch, non-deterministic repeat of a calibration seed, invalid state,
or output/budget failure. Biological losses are results, not runner failures.

## Outcomes and analysis

The schedule mechanism check records per-site amount and renewal events for
every tick of one 48-tick empty-world trace, plus before/after RNG state. It
checks the registered phase formula and a closed-schedule control. The same
world is also checkpointed at tick 17, restored, and continued; final full
State and RNG must match uninterrupted execution.

For each ecology replicate, preserve final State and tick summaries. Report by
seed and arm: alive founders, total births, F1 and F2 births, number of F1
offspring that bred, maximum generation, genotype counts, lifespan/death causes,
food eaten, energy spent on movement, and cumulative paid-trait maintenance.
Validate maintenance against emitted resolver/metabolism audit events and
energy accounting. Report medians and ranges as well as per-seed values. The
fitness outcome is descendant breeders per eight founders at tick 2,000;
survival and births remain separate outcomes. Do not collapse a negative or
null trait result into a success based on allele inheritance or event count.

Genotype arms are separate populations: their differences do not measure
within-population selection or allele-frequency change. The static/cycle
comparison is descriptive because resource timing changes behavior and
therefore later RNG use. The trait arms are paired on initial seed and
conditions, but not assumed to share later random trajectories.

## Evidence and interpretation limits

Use a new output directory under `data/experiments/ecology-traits-v1` only
after the source is clean and reviewed. Preserve the executable, source,
scenario, protocol, and runner hashes; exact command and flags; seed roles;
start/end timestamps; raw per-seed output; stderr; manifest; and summary. Never
overwrite a failed run. A runtime or mechanism check is not a registered
fitness evaluation. No formal evaluation begins before the source change and
this protocol have been committed and reviewed.

Eight evaluation seeds and 2,000 ticks are a small bounded screen. They do not
prove persistent ecological balance, broad adaptation, or long-term stability.
