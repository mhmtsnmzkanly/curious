# Herbivore danger, escape, and refuge experience — preregistration

**Status: preregistered by root; bounded execution requires a clean phase-3
source commit and root review.** This lab asks whether a herbivore can use its
own observed danger and realized escape/refuge outcomes to make a safer choice
in a later similar encounter. It is not a general predation or ecological
balance study.

## Minimal candidate and causal boundary

Reuse the existing bounded `Memory.dangers` observations and current
`SeekShelter` resolver action; do not add a second danger subsystem. Add one
default-off `RefugeOutcome` collection to each individual's existing `Memory`,
capped at eight records and expiring 100 ticks after its most recent completed
observation. Each record is keyed by the *currently perceived* threat ID and
species, relative-bearing and threat-distance buckets, subject energy/stamina
buckets, and the currently perceived refuge endpoint. It stores completed-route count,
safe-arrival count, threat-attributed damage, and resolver movement energy.
On `SeekShelter` only, after the existing closed/occupied-path filters run,
rank visible valid shelters lexicographically by higher completed safe-arrival
rate, lower mean damage from that observed threat, lower mean actual movement
energy, then the existing shortest path and endpoint order. If all candidates
are unobserved, retain the existing nearest-shelter behavior. If any candidate
has observations, unobserved candidates remain unknown and are ordered after
observed candidates; their missing counters are never read as zero risk. No
other decision reason or resource action uses this table. Memory belongs only
to the individual and is cleared for offspring.

Update a record only after an observed threat episode reaches its selected
refuge at a resolver-verified actual final position. A safe arrival requires
that the individual physically enters the selected refuge and receives no
damage from that observed threat during the route. Charge the movement energy
actually paid. Damage from another predator or unattributed damage censors
that episode for the threat-specific success/damage rate; report it separately.
At each resolver boundary compare net health loss with observed attack-event
damage. Excess net loss is recorded as unattributed and censors model updating.
`safe_arrival` means no observed attack damage from the keyed threat, not a
general absence of injury.
Loss of the currently eligible threat before arrival closes and audits the
episode but censors it: the model does not update the completed-route table.
Eligibility means that the same hunted individual remains in current
perception within two steps; it can leave this range while still visible at a
greater distance. A higher-priority decision, invalid refuge options, death,
timeout, and lab-boundary interruption are also censored. This feature requires
an eligible currently perceived threat while following a route.
Expire/evict deterministically by oldest observation and stable key. The
feature stays disabled by default, consumes no additional RNG, and must leave
closed-state behavior identical.

The exact context buckets are fixed: bearing is the first perceived step in
the eight `Direction` compass buckets (`Up`, `UpRight`, `Right`, `DownRight`,
`Down`, `DownLeft`, `Left`, `UpLeft`); threat distance is 1, 2, or 3+ perceived
steps; energy and stamina are quartiles
`min(3, floor(4 * current / max(1, maximum)))`. The selected threat is the first
hunted species in perception order within the existing two-step threat range,
matching ordinary instinct selection. The key also includes the sorted set of
all currently visible, nonempty, unoccupied refuge endpoints, so evidence from
a single endpoint is reused only against the same alternative set. A route target change censors the old
episode and starts a fresh episode only if the new decision is also
`SeekShelter` under a currently visible threat; evidence never transfers
between endpoints or contexts.

Each outcome record has a joint cap of 1,000,000 completed episodes. At the
cap, freeze all four counters together; never continue a numerator while
freezing its denominator. `RefugeEpisodeEnded` (or an exactly equivalent
stable audit record) is emitted once per closed episode, including censored or
unknown-attribution closure, so reports count closure events rather than
re-reading a persistent last-result field. Death, foreign damage, and
unattributed damage produce censored closures, never zero threat damage or a
safe arrival. When parent memory is disabled, refuge records and active
episodes are cleared under the existing memory-disable semantics. Offspring
inherit the feature flag only when both parents have it enabled; learned
outcomes and pending episodes are always empty in the child.

The lab writes an external `episode_audit` closure ledger for both the
default-off and opt-in branches from each full current `DecisionTrace`, visible
shelter/entity paths, resolver events, and post-resolver state. This audit is
not a decision input and does not mutate the world or learner. It supplies an
equal episode denominator because default-off memory emits no learning-event
closures. A selected endpoint is accepted only when exactly one currently
valid perceived refuge path matches the chosen movement prefix; ambiguous
prefixes and burrow arrivals without a unique endpoint stay censored and
unresolved. Each tick also records complete state and subject-memory snapshots
before and after the resolver, so episode-boundary memory can be reconstructed.

## Scenario and comparison

Use one herbivore per seed and observe all naturally occurring threat episodes
within the registered continuous branch; do not force a target encounter or
stop after a chosen number of closures. The subject's body, age, identity,
memory, and learner are never reset. Use eight fixed seeds 45001–45008 and no
experience transfer. Pin a 7×7
open arena with the subject at `(3,3)`, two `Burrow` refuge endpoints at `(1,3)`
and `(5,3)`, renewable food at `(3,6)` (amount/capacity 20, replenish 1 every
2 ticks), and water at `(6,6)` (amount 12, capacity 30, replenish 4 every 2
ticks). The controlled carnivore starts at `(3,0)` and follows the repeating
fixed one-cell loop `(3,0)→(2,0)→(1,0)→(1,1)→(1,2)→(2,2)→(3,2)→(4,2)→(5,2)→(6,2)→(6,3)→(6,4)→(5,4)→(4,4)→(3,4)→(2,4)→(1,4)→(0,4)→(0,3)→(0,2)→(1,2)→(1,1)→(1,0)→(2,0)` and then back to `(3,0)`, using one-cell `Move` intents through the ordinary resolver. The loop avoids both burrow cells; the carnivore starts with health/max-health, energy/max-energy, and water/max-water all 2,000 and speed 1. If the subject is currently
visible and adjacent (Manhattan distance ≤1), the carnivore instead submits
`Attack { target_id: subject_id }` through the normal resolver; otherwise it
continues its next route step. This same environment controller applies to
both arms. No position teleport or hidden future route is allowed. Each
encounter starts on the first ordinary `SeekShelter` decision while the
subject's perception contains the predator within two steps. It closes when
the subject reaches a currently visible valid burrow, the predator leaves the
eligible two-step perception range, a higher-priority action interrupts,
refuge options become invalid, the subject dies, or 20 ticks elapse. The initial
fixture presents both shelters at equal path length; later occupancy and
blockage arise only through actual world state. A comparable context requires
the same visible threat ID/species, relative-bearing and threat-distance
bucket, subject energy/stamina buckets, and the same pair of currently visible
valid refuge endpoints. The harness logs current perception on every choice;
later predator positions are never exposed. The resolver tick, perception,
selected path, post-resolver position, and resolver costs are recorded in that
order. Controller state and future interventions never enter the subject's
observation.

Compare a default-off baseline with the opt-in outcome-memory branch cloned
from the same complete initial state before its first threat observation. Both
branches receive identical event scripts and begin with identical actor IDs,
body, age, ordinary memory, learner, and RNG. Neither individual is reset
between episodes; stop on death rather than replacing it. The opt-in branch
updates its own refuge records only after completed observed routes. At tick
50 of each existing 100-tick branch, serialize and restore that same branch in
place, then require complete state and RNG equality before continuing. This is
a serialization roundtrip check, not a repeated trajectory proof, and adds no
physiological ticks. A separate checkpoint continuation test may compare a
repeated suffix outside the biological budget. No assertion requires
experimental and baseline RNG streams to remain equal after their policy
choices diverge. A clone represents a counterfactual, not a second co-present
individual. This phase follows the phase-4 V15 saved-state
change and, if it is merged as designed, adds the memory field in checkpoint
V16. The V16 migration must preserve every V15 field; an earlier checkpoint
must produce empty refuge records and leave the feature disabled.

## Outcomes, validation, and bounds

For each threat episode record visible threat identity/location from perception
(not hidden world state), choice/reason, selected refuge endpoint, whether the
route was physically completed, actual final position, movement energy paid,
damage received and source attribution, time to a refuge, failed/rejected
actions, blocked/occupied options, health/energy/stamina, and the exact memory
before/after. An episode counts as a successful escape only if the subject
physically reaches a currently valid refuge and receives no threat-attributed
damage during the measured route; report partial progress and any other damage
separately. Lab-boundary interruptions are censored.

Primary descriptive outcomes are resolver-verified safe arrivals and
threat-attributed damage per eligible observed-threat episode; also report
every censored closure, movement energy, and resource-seeking time/cost after
escape. The model learns only completed arrivals, so this lab cannot establish
reduced mortality or safer outcomes across censored/death episodes. Show the
first versus last genuinely
similar repeated context for each seed, with all seeds and absolute counts.
Do not combine safety, energy, and food into a weighted reward. If the two
branches do not encounter an eligible same-context pair or if the subject dies
before a later comparison, report that seed as unavailable/dead rather than
substituting another animal or fabricating a match.

Each branch runs continuously for at most 100 world ticks and ends early on
subject death. An episode may start only from a real observed threat. Report
how many eligible contexts and naturally observed episodes were actually
available. Hard bound: eight seeds × two branches ×
100 ticks = 1,600 ticks. Stop on the first mismatch in paired initial state, hidden-state
leak into observations, invalid escape attribution, memory-bound violation,
schema error, or timeout; preserve partial output as failed. The committed
protocol must pin exact map/route coordinates, tick schedule, seed mapping,
binary/source SHA, command, runner/protocol SHA, manifest schema, and fresh
output path. Root reviews the protocol and clean source commit before any
evaluation. Keep the module opt-in and do not edit live worlds, frozen releases,
or historic/raw archives.

## Exact execution record

Before the held-out seeds, run one fixed serialization/measurement calibration
from the clean, reviewed source and immutable `lab-probe`:

```sh
/root/curious/data/experiments/herbivore-lab-v1-calibration-20261023/lab-probe \
  --calibration
```

The single calibration seed is 45999, distinct from held-out seeds 45001–45008.
It emits the same raw schema and runs both branches for at most 100 ticks each,
for no more than 200 physical ticks. Store its copied probe, protocol/source
hashes, seed audit, raw JSONL, stderr, manifest, and validation summary under
`data/experiments/herbivore-lab-v1-calibration-20261023/`. Validate its one row
with `validate_initial_pair`, `validate_branch` for both arms, and the physical
tick sum check (≤200). Calibration cannot change biological parameters, seeds,
or thresholds and cannot be presented as held-out evidence. Only a demonstrated
measurement or serialization defect may be corrected before held-out execution;
any such correction requires a reviewed source/protocol hash and a fresh
calibration on the replacement immutable probe.

After root has committed and reviewed the clean source and built the immutable
release example, run exactly:

```sh
python3 scripts/run_herbivore_danger_lab.py \
  --binary /root/curious/target/release/examples/herbivore_danger_lab \
  --output /root/curious/data/experiments/herbivore-lab-v1-20261023 \
  --seed-registry /root/curious/data/experiments
```

The runner refuses a dirty source tree, a reused seed, or an existing output
directory. It copies the release binary to `lab-probe`, the protocol to
`protocol.md`, and the exact Rust/Python sources to `source/`, then hashes those
files and writes the `curious-herbivore-lab-run-v1` manifest with
`protocol_version: 1` before launching `./lab-probe` with no arguments. Seed
audit must show 45001–45008 unused. Formal evaluation limits are 300 seconds,
1 GiB raw output, 1 GiB child memory, and 1,600 physical ticks; the registered
calibration plus formal evaluation has a combined hard ceiling of 1,800
physical ticks. The manifest records the
probe, source, protocol, harness, runner, seed-scan hashes, exact command and
terminal status. A failed or partial run is retained and never overwritten.
