# Herbivore danger, escape, and refuge experience — preregistration

**Status: prepared for root review; do not run a lab before root commits this
protocol with the source revision.** This lab asks whether a herbivore can use its
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

Update a record only after an observed threat episode ends with a resolver-
verified actual final position. A safe arrival requires that the individual
physically enters the selected refuge and receives no damage from that
observed threat during the route. Charge the movement energy actually paid.
Damage from another predator or unattributed damage censors that episode for
the threat-specific success/damage rate; report it separately. An unfinished,
interrupted, or lab-boundary route is censored and does not count as safe.
Expire/evict deterministically by oldest observation and stable key. The
feature stays disabled by default, consumes no additional RNG, and must leave
closed-state behavior identical.

## Scenario and comparison

Use one herbivore per seed for at most four consecutive observed-threat
episodes in the same world, with no body, age, identity, memory, or learner
reset. Use eight fixed seeds 45001–45008 and no experience transfer. Pin a 7×7
open arena with the subject at `(3,3)`, two `Burrow` refuge endpoints at `(1,3)`
and `(5,3)`, renewable food at `(3,6)` (amount/capacity 20, replenish 1 every
2 ticks), and water at `(6,6)` (amount 12, capacity 30, replenish 4 every 2
ticks). The controlled carnivore starts at `(3,0)` and follows the repeating
waypoints `(3,0)→(1,0)→(1,2)→(5,2)→(5,4)→(1,4)→(1,2)→(1,0)→(3,0)` using one-cell
`Move` intents through the ordinary resolver. If the subject is currently
visible and adjacent (Manhattan distance ≤1), the carnivore instead submits
`Attack { target_id: subject_id }` through the normal resolver; otherwise it
continues its next route step. This same environment controller applies to
both arms. No position teleport or hidden future route is allowed. Each
encounter starts on the first ordinary `SeekShelter` decision while the
subject's perception contains the predator. It closes when the subject
reaches a currently visible valid burrow, the predator has not been visible
for two consecutive ticks, the subject dies, or 20 ticks elapse. The initial
fixture presents both shelters at equal path length; later occupancy and
blockage arise only through actual world state. A comparable context requires
the same visible threat ID/species, relative-bearing and threat-distance
bucket, subject energy/stamina buckets, and the same pair of currently visible
valid refuge endpoints. The harness logs current perception on every choice;
later predator positions are never exposed.

Compare a default-off baseline with the opt-in outcome-memory branch cloned
from the same complete initial state before its first threat observation. Both
branches receive identical event scripts and begin with identical actor IDs,
body, age, ordinary memory, learner, and RNG. Neither individual is reset
between episodes; stop on death rather than replacing it. The opt-in branch
updates its own refuge records only after completed observed routes. Separate
replay of the baseline must match itself across checkpoint/resume; no
assertion requires experimental and baseline RNG streams to remain equal after
their policy choices diverge. A clone represents a counterfactual, not a
second co-present individual. This phase follows the phase-4 V15 saved-state
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

Primary descriptive outcomes are successful completed routes per eligible
observed-threat episode, threat-attributed damage, movement energy, and
resource-seeking time/cost after escape. Show the first versus last genuinely
similar repeated context for each seed, with all seeds and absolute counts.
Do not combine safety, energy, and food into a weighted reward. If the two
branches do not encounter an eligible same-context pair or if the subject dies
before a later comparison, report that seed as unavailable/dead rather than
substituting another animal or fabricating a match.

Each branch runs continuously for at most 100 world ticks and ends early on
subject death. An episode may start only from a real observed threat; the
runner does not force four encounters. Report how many eligible contexts and
encounters were actually available. Hard bound: eight seeds × two branches ×
100 ticks = 1,600 ticks. Stop on the first mismatch in paired initial state, hidden-state
leak into observations, invalid escape attribution, memory-bound violation,
schema error, or timeout; preserve partial output as failed. The committed
protocol must pin exact map/route coordinates, tick schedule, seed mapping,
binary/source SHA, command, runner/protocol SHA, manifest schema, and fresh
output path. Root reviews the protocol and clean source commit before any
evaluation. Keep the module opt-in and do not edit live worlds, frozen releases,
or historic/raw archives.
