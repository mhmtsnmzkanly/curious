# Ecology fixture validity protocol (draft for root review)

**Status:** Draft. No registered World run is authorized by this file. The
runner rejects the draft until root fills seed roles, pins the clean source
identity and changes the protocol to `registered` with `root_reviewed: true`.

## Question and scope

Does the archived accepted-profile fixture, with its accepted policy settings
explicitly carried into the current Scenario API, provide observable feeding,
survival, and reproductive opportunity over a short F1-oriented horizon, and
can the ecology audit retain internally consistent resource, lineage, and body
snapshot evidence within a fixed runtime and storage cap?

This first stage measures fixture validity only. It does not compare traits,
shocks, fitness, treatment advantage, or sustained coexistence. If a species
goes extinct, record that as a biological negative. If a species survives but
the run observes no eligible adjacent mature same-species pair, the breeding
question is nonidentifiable for that run, not proof of reproductive failure.
Eligible adjacent pairs are an opportunity proxy: agents can choose other
actions, and the measure does not guarantee a birth space or mating intent.
Actual `EntityMated`/`EntityBorn` evidence remains authoritative for breeding.

## Immutable starting fixture and accepted settings

Input: `/root/curious/data/experiments/reserve-qualification/qualification-both/scenario.json`,
SHA-256 `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`.
It contains 608 founders (576 herbivores, 32 carnivores), 976 renewable sites,
and 1,232 initial resource cells. The raw scenario is read-only. No body repair,
resource refill, replacement, rescue, or parameter tuning is allowed.

The qualification manifest binds revision
`b4dcef20fdef830c2895806076d972df62741eea`, clean binary SHA-256
`e1ffa03754fc964d503b6a6e04bffe19ad868b3d2090e0a8b2f9c48e3506f636`, with
`reserve_opportunities=true` and `mate_rendezvous=true`. Those world policies
are absent from the old scenario JSON, so the new harness must supply both
flags explicitly. It also forces the newly introduced water-residual policy
off. The archived run used the old code defaults for genetics (enabled with
20-per-mille mobility mutation); the fixture has no extra-trait enablement and
all listed founder mobility alleles are zero.

Current source has evolved since the accepted binary. Schedule-less resource
renewal remains the default path; extra vision/stamina traits are opt-in and
remain off; the water-residual flag is explicit off. However, current World
ticks also invoke new default learner/outcome paths and emit additional event
types. This protocol does **not** claim bit-for-bit reproduction of the old
qualification or that disabled-path equivalence has been proven across every
seed. The run will be labeled a current-source fixture-validity diagnostic
bound to both source and old accepted-profile manifest.

## Stages and roles

The proposed calibration role is one fresh seed at at most 200 ticks. It checks
source/fixture identity, command flags, basic stock reconciliation, and whether
the measurement output stays bounded. It is diagnostic and does not qualify the
fixture biologically. Its technical gate also requires zero food and water
resource residuals, no kind replacements, no parent/unattributed/trait-payment
audit issues, no species extinction, and at least one actual birth of either
species to exercise the observation path. Failure stops before formal roles;
parameters and seed roles are not changed in response.

If calibration passes and root accepts the measured cost, the proposed formal
stage has three new seed roles, each at 500 ticks. This horizon spans 25 default
maturity intervals (founders mature at age 20 and have maximum age 105) and is
intended to expose F1 birth and breeder opportunity. It is too short to require
F2 breeders as a general acceptance condition. Do not reuse previously inspected
seeds. Root chooses all seed numbers only after inventorying all experiment
manifests and raw files. The current draft records calibration 53099 and formal
roles 53001, 53002, and 53003 from the root-owned scan of 1,093 existing
experiment text sources; the scan reported no matches for these values. Its
SHA-256 is `567208ec739e54153ad964f1c1465386dcf2b35a6428c97d135854b268400dbe`.
These role numbers remain a draft until the source and protocol are reviewed
and frozen; no numbered run has been launched.

The formal stage is three seeds × 500 ticks = 1,500 ticks total; calibration
adds 200 ticks, for 1,700 total. A formal seed is a bounded viability pass only
if all four preregistered checks pass: neither species goes extinct; both
species have at least two living individuals at every tick in the final 50;
each species has at least one living descendant that itself bred at tick 500;
and each species has at least one birth in the final 50 ticks. All three seeds
must pass for this small fixture-validity gate. If any species goes extinct,
record a biological negative and stop that seed at extinction while continuing
the other registered seeds for bounded variation. If neither species is extinct
but breeding evidence is absent, report the gate failure; the adjacency proxy
describes exposure only and does not turn it into proof that mating was possible.

Before formal seeds are consumed, freeze the feasibility forecast from the
calibration probe time: `calibration_probe_seconds × (1500 / calibration_ticks)
× 2 + 60 seconds` for a safety factor and seed scan/analysis reserve. Require
the forecast to be at most 300 seconds. This linear projection is only a
preregister feasibility gate and is not a runtime guarantee; the parent/child
300-second and 1-GiB combined address-space supervisors remain hard limits.

Only the accepted-profile arm is present. Later habitat-matrix work is gated on
clean source review and fixture-validity evidence and needs a separate protocol.

## Measurements and gates

Record opening/ending stocks, renewals, resolver-backed plant meals, corpse
meals, water intake, signed and directional resource residuals, resource-kind
replacement, per-species live founders/descendants, births by generation,
actual breeders and living bred descendants, death causes, parent/event issues,
and genotype/phenotype snapshots. Basal metabolism and unattributed body-energy
changes remain unknown. Source identity, scenario hash, explicit flags, protocol
hash, binary hash, seed role, runtime and output sizes accompany raw records.

Gate sequence: accepted fixture hash and explicit flags match; source and
protocol identity are clean and pinned; resource/event/lineage measurements are
internally coherent; each species' outcome is classified as surviving or
extinct; breeding interpretation is conditional on observed eligible adjacent
pair exposure and actual mating/birth events. A failed calibration gate blocks
the formal phase. During formal work, extinction stops that seed and records a
biological negative; the other already registered seeds still run to describe
bounded variation. Any infrastructure error or capture-cap hit aborts the
phase. There is no automatic retry, seed replacement, or retuning.

## Budgets and capture bounds

Per phase: 300 seconds wall clock, 1 GiB address-space limit, and at most
128 MiB aggregate output. Single-process preflight (including calibration raw
revalidation and the complete seed scan) may use up to 1 GiB. While one probe
child is active, the parent is capped at 256 MiB and the child at 768 MiB; the
parent returns to a 1-GiB ceiling for analysis after the child exits. One build
worker is used. Before
formal root registration, root should measure a short local cost sample using
the seed-123 engineering fixture (at most 2,000 physical ticks total), estimate
the planned tick/actor workload from the measured rate, and include the frozen
safety factor. If the projected formal phase exceeds the cap, stop before any
formal seed starts and return to root for a separate decision. Do not change
the frozen horizon or replace the registered seed roles to fit the cap.

The probe aborts before audit capture if the identity estimate or world slot
count exceeds 50,000, a single tick emits over 20,000 events, complete mating,
birth, and death lineage capture would exceed 50,000 records, or combined
resource reconciliation/replacement/lineage-issue detail would exceed 2,000
rows. It never truncates vital lineage and labels the remainder complete. The
per-tick stream retains species counts, stock/flow counters and lineage events;
five fixed audit frames (ticks 1, 20, 100, 200, and 500 when within the phase)
retain full checkpoint State before/after and all typed events at that tick.
The runner monitors wall time and aggregate output bytes and applies the
parent/child address-space limits. These are hard aborts, not truncation
presented as complete evidence. A killed or aborted run retains stderr, status,
and flushed partial JSONL under a new output directory.

## Runner contract

`examples/ecology_validity_lab.rs` reads the scenario and explicitly sets the
accepted profile in memory: `mode=both`, companions enabled, genetics enabled
with mutation rate 20 per mille, extra traits off, water learning/update on,
water-residual/hunt/refuge features off, `reserve_opportunities=true` and
`mate_rendezvous=true`. It does not write or repair the scenario. The Python
runner requires a root-reviewed, registered protocol JSON with distinct seed
roles and matching clean source hashes. The machine-readable calibration and
formal draft role files are [`ECOLOGY_FIXTURE_CALIBRATION.json`](ECOLOGY_FIXTURE_CALIBRATION.json)
and [`ECOLOGY_FIXTURE_FORMAL.json`](ECOLOGY_FIXTURE_FORMAL.json); they record
the root-selected roles but still lack source registration and remain
unrunnable. The runner writes flushed JSONL
tick rows so partial biological negatives and infrastructure stops remain
inspectable. It builds with one Cargo job, records the executable SHA, and
creates output files exclusively. Root must review and
commit this protocol before any registered World run. Unit fixture work is
limited to the nonregistered seed-123 engineering check and at most 2,000 total
physical ticks.
