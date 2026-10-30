# Genotype measurement engineering protocol

**Status: FROZEN BY ROOT.** This is one seen-seed measurement exercise, not a selection test. World execution remains gated on root committing this protocol and the reviewed driver together.

## Question and scope

Can the current observer and a bounded capture driver record, on the current
accepted-profile habitat, founder genotype and expression, event-backed
genotype-specific vision/stamina request/payment and food acquisition, repeated
food encounters with explicit unknown coverage, birth/transmission/lineage,
and resource-flow reconciliation for one short paired expressed-versus-neutral
run within the stated capture budget?

This is an engineering measurement check only. It has no fitness endpoint,
formal seed role, pass/fail criterion for biological viability, or selection
claim. Do not interpret between-arm population or allele changes as advantage.
Only the observer gates (balanced starts, phenotype expression/control,
event/lineage joins, and stock-flow consistency) are examined. No tuning,
replacement, extension, or follow-up run is authorized by this protocol.

## Source, fixture, and arms

Use the archived accepted-profile fixture
`/root/curious/data/experiments/reserve-qualification/qualification-both/scenario.json`,
whose recorded SHA-256 is
`33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`.
Require the current-source run to report the current checkpoint schema and
the explicit accepted settings from `ECOLOGY_FIXTURE_V1`: mode `both`,
companions enabled, `mate_rendezvous=true`, `reserve_opportunities=true`,
water residual disabled, water learning and updates enabled, hunt learning and
refuge outcomes disabled. The scenario file is read-only. Do not add/remove
founders, edit body condition, shift positions, refill resources, or alter
sites.

Run exactly two arms in one invocation, both with seen seed 123 and exactly
200 World ticks per arm. There is no calibration role and no formal or held
out seed. Arm runs start from independent builds of the same parsed source
bytes and apply identical founder genomes/positions and policy configuration.
No founder, body, or resource reset occurs after tick 0.

Set genetics enabled and mutation rate zero in both arms. Set all founder
mobility and stamina alleles to zero; set carnivore vision to zero. Herbivore
founders receive an exact 1:1 vision-0/vision-1 split. Allocate deterministically
as follows:

1. Divide the inclusive scenario bounds into a 4-by-4 grid. For coordinate
   `x`, the tile column is
   `min(3, ((x - min_x) * 4) / (max_x - min_x + 1))`; compute the row by the
   same formula for `y`.
2. Visit tiles in row-major `(row, column)` order. Within each tile, sort
   herbivore founders by `(y, x, original_founder_index)`.
3. Assign alleles 0 and 1 alternately, carrying the alternating parity across
   tile boundaries. Require an even total herbivore founder count. This gives
   an exact global half split and at most a one-founder difference within any
   tile. Record each founder ID (`original_founder_index + 1`), position,
   tile, species, and assigned genome, plus per-tile allele counts.

The expressed arm enables extra traits. At tick 0, vision allele `v` must
produce the fixture's starting vision range plus `v`, clamped by the current
engine expression rule, and request vision maintenance `v`; stamina remains
at the starting phenotype with zero trait request. The neutral-marker arm
disables extra traits while retaining the same genomes; every founder's
vision range and trait requests must remain at their base fixture values
(vision and stamina requests zero). In both arms verify actual initial State
against the founder allocation before tick 1. Record the exact phenotype
distribution; do not synthesize a neutral phenotype or subtract costs.

Use normal AI decisions and resolver behavior. Capture full perception in
memory only so the actual per-tick `DecisionTrace` can supply compact
per-individual food encounter counts. Never pass audit data to an agent.
Serialize no full perception payload. Repeated trace entries count as repeated
observed encounters; split corpse and non-corpse. A missing decision trace or
missing full perception is unknown coverage, not zero encounters.

## Checkpoint/resume mechanism check

At tick 100, capture the real `State`, serialize and deserialize it, restore a
World from that State, and compare `State::capture` before and after restore,
including RNG, policy, genetics, genomes, and bodies. Continue the same arm
from that restored World through tick 200. This checks State round-trip and
the resumed execution path. It is not a comparison against uninterrupted
execution and does not establish deterministic fork equivalence.

## Capture and reconciliation

Retain complete State and all typed events only at fixed frames t0, t1, t100,
and t200. At every tick, stream: tick number; species and genotype-level
population counts; food/water stocks; typed event counts; resource
replenishment, actual plant/corpse meals, water intake, actual trait
request/payment, mating, birth, and death events; actor IDs join to the tick-0
founder assignment or a prior birth record, avoiding repeated full genomes;
each birth child's post-tick ID/species/full genome/generation/parents and
expressed vision/stamina; compact per-ID trace rows `[id, full-perception,
vision-range, plant-count-or-null, corpse-count-or-null]` and
coverage; and resource stock-flow reconciliation. This is sufficient for
independent event and lineage reconstruction but is not a complete replay:
non-frame movement/action events and full perception payloads are not retained.

Use the existing `ecology_audit` stock/resource accounting and `genotype_audit`
observer with explicit before State, after State, typed events, and actual
decision traces. Do not infer basal metabolism from trait costs or body energy
residuals. A typed request that disagrees with the actor's expressed
life-state cost, payment above request, missing actor/genome, invalid
parent/generation, unexpected transmission under mutation zero, missing birth
join, or nonzero unexplained food/water stock-flow residual is retained as an
issue and stops the run after flushing evidence. Payment below request is a
valid unpaid amount and is reported explicitly. Totals reconcile to actual
typed events. Positive costs paid by the resolver remain event-backed;
acquisition is not food discovery. Unknown trace coverage remains explicit.

## Bounds and stop rules

The total of both arms' raw JSONL plus driver stderr, status, and manifest must
remain within 128 MiB. The driver raw cap is 112 MiB, reserving 16 MiB for
other outputs. Root supervises combined address space at 1 GiB and wall time
at 300 seconds; compile with one Cargo job. Hard capture limits are 50,000
retained IDs, 20,000 events per tick, 50,000 retained lineage events, and
2,000 retained detailed audit/issue rows. Preflight each cap before mutating
the audit; abort and flush partial evidence on the first breach. Do not truncate
and label capture complete.

The zero-World serialization forecast uses the previous measured 24,728,247
bytes per 200-tick arm (49,456,494 paired), plus 38,230,086 bytes for the
previous fixture's typed resource/meal event counts and an estimated 57,600
expressed vision-maintenance events, plus 17,702,400 bytes for a conservative
compact trace bound of 1,844 identities × 200 ticks × 2 arms × 24 bytes, and
1,000,000 bytes reserved for extra assignment, birth-join, and frame metadata.
The resulting estimate is 106,388,980 raw bytes, below the 112 MiB cap. It is
a planning estimate from a previous observed fixture, not a guarantee for
divergent trajectories; the hard byte limit remains authoritative.

Stop both arms on fixture/profile/source mismatch, invalid or imbalanced
founders, incorrect phenotype expression/neutrality, observer error, any
lineage or transmission issue, stock-flow residual, resume State/RNG mismatch,
capture-cap breach, or infrastructure failure. Preserve partial JSONL and
stderr/status. Do not retry or change parameters after observing output.

## Interpretation and identity

This seed is already seen and the paired arms are not exact counterfactuals:
the expressed-trait path can consume different RNG draws during reproduction,
after which world trajectories can diverge. Record this limitation. A passing
observer gate means only these facts were captured and reconciled in this
engineering fixture. It does not mean vision is useful, neutral, harmful, or
causally responsible for any population change.

Before execution root must commit this protocol and driver together, record
clean source revision/hash, protocol hash, driver
executable hash, exact fixture file hash, command, start time, output path,
and measured resource limits. The driver refuses to run while this document
remains in draft status. No formal seed scan or registration is needed because
this is the explicitly seen engineering seed 123.
