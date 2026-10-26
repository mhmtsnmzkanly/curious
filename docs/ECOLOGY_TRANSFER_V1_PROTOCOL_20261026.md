# Ecology transfer V1 — bounded per-feature screen

**Status: protocol contract. A phase is registered only when its matching
registration JSON says `status=registered` and `root_reviewed=true`, and pins
the clean source revision and hashes.** This is a small transfer screen of implemented,
default-off features. It is not an ecological qualification, deployment
decision, or claim that denser populations are better. Primary category K5,
with K1/K2/K3/K4/K6 dependencies.

## Question and fixed inputs

Across three existing habitat fixtures, do climate, ambush, local alarm, or
vision/stamina trait expression change physical survival and reproduction over
600 ticks relative to the same habitat's control? A lower population under environmental
stress is an observed response, not automatically a failure. Interpret births,
deaths, survival exposure, body stress, and offspring reproduction together.

| Habitat role | Frozen scenario | Founders | Resources | Seeds |
|---|---|---:|---:|---|
| Resource-rich herbivore baseline | `scenarios/meadow.json` (`herbivore-meadow`) | 8 herbivores | 20 sites / 20 initial resource cells | 62301–62303 |
| Sparse-resource herbivore | `scenarios/ecology-traits-lab.json` (`ecology-traits-lab-v1`) | 8 herbivores | 8 sites / 8 initial resource cells | 62311–62313 |
| Mixed predator/refuge | `scenarios/mixed-refuge.json` (`mixed-refuge`) | 36 herbivores, 2 carnivores | 61 sites / 77 initial resource cells | 62321–62323 |

The sparse-resource fixture is not classified as seasonal. Scenarios are read
and hashed, never edited on disk. For every arm, genetics is enabled with
mutation 0 per mille. Founder templates are the same across arms: mobility 0,
vision/stamina pairs `(0,0)`, `(1,0)`, `(0,1)`, `(1,1)` repeated by founder
index. The extra-trait arm enables expression of the vision/stamina alleles.
The same genome alleles are inherited through the ordinary `Genome::child`
path in every arm; `extra_traits_enabled` gates their expression and the
extra-trait mutation loop. With mutation fixed at zero, this contrast measures
expressing inherited variation against dormant alleles, not inheritance versus
no inheritance. It does not isolate vision from stamina or establish
within-population selection. The extra-trait path still consumes two additional
RNG draws at every mating, one per allele, even at zero mutation. Thus any
extra-trait-arm outcome includes that RNG-stream difference as well as changed
body expression; it is not a clean estimate of phenotype alone. Equal seed and
initial RNG state are recorded, but post-mating RNG alignment is not claimed.
In other arms the alleles remain dormant, and offspring body state follows the
ordinary extra-trait-off path. These initial bodies otherwise match the
scenario's default life state.

Root must scan the experiment inventory for all nine seeds before registration.
If a fixed seed is already recorded, stop this design and bring a replacement
protocol back for review; do not substitute a seed after outcomes are known.
Seeds pair arms within habitat. After behavior differs, later RNG draws need not
match.

## Arms, feature settings, and workload

Run five arms for each habitat/seed block, exactly 600 ticks per World. If a
population becomes empty, continue the empty World to tick 600.

| Arm | Climate | Ambush | Local alarm | Extra-trait expression |
|---|---:|---:|---:|---:|
| `control` | off | off | off | off |
| `climate_only` | on | off | off | off |
| `ambush_only` | off | on | off | off |
| `alarm_only` | off | off | on | off |
| `extra_traits_only` | off | off | off | on |

Climate uses the fixed `SeasonCycle { period: 80, winter_bmr_surcharge: 2,
winter_plant_regen_divisor: 2 }`; no day/night feature is added. All arms
explicitly disable mate rendezvous, reserve opportunities, water residual,
injury-capacity penalty, and juvenile care. No combined-feature arm, parameter
sweep, replacement seed, rescue, post-start resource addition, body reset,
threshold tuning, or automatic retry is allowed.

The primary screen is 3 habitats × 3 seeds × 5 arms = 45 Worlds × 600 ticks
(27,000 primary ticks). Each invocation also advances an isolated clone by one
untimed Store-smoke tick, so the full registered workload is at most 27,045
World ticks across 45 child processes. The supervisor has a 300-second phase
limit, a 60-second child
limit per World, a 1 GiB child address-space limit, a 1 GiB aggregate output
cap, a 10,000 identity cap, and a 20,000 event-per-tick cap. It builds one
release example with one Cargo job. A phase timeout, source/config/hash
mismatch, invalid State, identity/event/output cap, incomplete tick stream, or
runner error stops the phase. Partial raw output and stderr remain marked
incomplete. Biological extinction, no mating, and zero births are outcomes;
they do not authorize retry or seed replacement.

The runner requires a root-reviewed registration JSON (`status=registered`,
`root_reviewed=true`) that binds clean source revision/content and file hashes,
scenario hashes, these nine habitat/seed roles, exact arms/horizon/settings,
and all resource ceilings. Its required top-level fields are `status`,
`root_reviewed`, `source`, `scenario_sha256`, `seed_roles`, `ticks_per_world`,
`arms`, `habitats`, `world_count`, `phase_seconds`, `child_seconds`,
`child_address_space_bytes`, `aggregate_output_bytes`, `max_identities`,
`max_events_per_tick`, `mutation_per_mille`, and `genetics_enabled`. It rescans the existing experiment archive before
creating a new, non-overwriting output directory. The registration JSON, runner binary, and run output belong to the registered
phase artifacts; this contract does not claim they exist before registration.

## Measurements and analysis

Each raw JSONL contains an initial State, then one abundance/resource/body
stress row per tick and an endpoint summary with final checkpoint State. Tick
rows also retain compact `birth_lineage`, `mating_events`, and `death_lineage`
records from resolver events and child slots: IDs, species, generation, parent
IDs/generations, and death cause, with missing values left unknown. The runner
does not copy the full event stream. This permits independent reconciliation
of F1/F2 birth counts and breeder identities after dead descendants are removed.
The supervisor recomputes birth/death species and generation totals and F1 breeder
IDs from those rows before accepting a World summary.
The summary validates endpoint restore without advancing another World tick.
Record species abundance, founder survival, births/deaths by species and generation,
living individual-ticks, resource stocks, zero-energy and low-health exposure,
attacks/damage/kills, alarm signal emissions, and final genotypes/phenotypes.
Every 100 ticks, preserve interval and cumulative survival exposure and
interval births/deaths, food/water stocks, winter ticks, and body-stress counts.

Lineage measures include F1/F2 birth counts, identities of F1 breeders, living
F1 breeders at tick 600, and living descendant breeders. Birth generation and
parent identities are taken from endpoint slots and mating/birth events; any
missing parent/generation attribution is reported as unknown. The screen does
not count adjacency as successful mating.

For the extra-trait arm, retain expressed founder/final body requests and
`EntityMetabolized` event-reported requested/paid values when emitted. The event
stream does not guarantee an event for every zero or unlogged body-maintenance
term, so unobserved actual total paid cost stays `Unknown`; do not infer exact
paid totals from the phenotype request. For alarm, count emitted sound traces
by `emitted_tick`. Count receptions only where an actual decision Perception
trace exposes them; otherwise report `null`/Unknown, never zero. For climate,
report fixed seasonal exposure and observed body stress; do not label an
abundance change itself as climatic harm.

Primary comparison is the per-seed arm-minus-control vector within each
habitat: births, deaths, final live by species, founder survival, living
individual-ticks, F1 breeders, F2 births, resource exposure, and feature-specific
physical outcomes. Show absolute values and per-habitat median/range across the
three paired seeds. This small screen is descriptive; do not collapse it into a
single reward, rank habitats universally, or claim general Pareto improvement.
Environmental stress can reduce abundance while changing reproductive
outcomes in either direction. State those outcomes and their limits explicitly.

## Stop and interpretation boundary

Any incomplete/censored World remains so in summaries. Validate the tick
sequence and endpoint checkpoint restore; check population balance against
recorded births/deaths, and retain unattributed lineage/cause values as
Unknown. Preserve source/scenario/protocol/binary hashes, exact config, seed
scan, stdout/stderr, raw per-tick records, summaries, and endpoint State under
a new output directory. A complete run supports claims only for these three
fixtures, nine paired seed blocks, exact flags, and 600-tick horizon. Feature
mechanism presence or allele transmission alone is not utility acceptance; any broader
candidate needs a new held-out protocol and explicit outcomes.

Related evidence: [energy and package diagnosis](reports/ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md),
[ecology traits protocol](ECOLOGY_TRAITS_PROTOCOL.md), and
[ecology fixture protocol](ECOLOGY_FIXTURE_PROTOCOL.md).
