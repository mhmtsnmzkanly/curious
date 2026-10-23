# Refuge context generalization — design proposal

**Status: proposal and isolated prototype only.** This work does not change the
registered V16 learner, its checkpoint, the live world, or the prior refuge
protocol. The eight historical seeds and all 94 multi-shelter decisions are
already observed. Their outcome-memory key matched no completed record at any
choice; they motivate a new hypothesis, not a retrospective claim or a retuned
acceptance result.

## Question and behavior boundary

Can an herbivore reuse a resolver-verified outcome from a *similar observable
escape situation* when the individual predator ID or the exact set of visible
alternatives changes, while still falling back to the established shelter
choice whenever that similarity is uncertain?

The proposal keeps the current instinct as the fallback and only lets an
individual's own bounded, non-inherited memory rank currently perceived,
currently valid shelter routes. It does not alter threat detection, shelter
eligibility, route planning, resolver rules, movement speed, or physical cost.
Disabled mode must preserve state and RNG behavior. The resolver remains the
only source of paid movement and damage facts. No hidden predator location,
future route, physiology, or controller state may enter context or choice.

## Candidate context representation

The prototype deliberately separates three kinds of information:

1. **Episode identity for attribution:** retain the visible threat ID/species
   on the active route so resolver attack events can be attributed to the
   observed threat. This is transient accounting state, not a reusable context
   key.
2. **Observable situation:** describe hunted species, coarse relative threat
   direction and perceived distance, broad subject energy/stamina ranges,
   locally perceived habitat/terrain class, and current route geometry/cost
   features. Use current perception and established memory only. Avoid exact
   threat IDs, exact coordinates, quartiles, and the whole alternative-set
   signature as required equality fields.
3. **Candidate route:** describe each valid refuge independently with route
   length, turns, first-step direction relative to the observed threat, visible
   terrain along the route, endpoint refuge type, and current resolver-cost
   estimate from observable path length and the individual's current speed.
   Record actual movement energy paid after execution; an estimate is not an
   outcome.

The current pure prototype's feature structs omit the subject species and
movement speed. That is an integration gap: before production use, add these
features or preregister and validate a reason they can be omitted. The prototype
does not establish that outcomes transfer safely across different subjects or
speeds.

The candidate comparison may share evidence across changed threat IDs and
changed alternative sets only when the observed situation and route features
remain inside a preregistered compatibility region. A different species,
materially different danger geometry, incompatible habitat/terrain, or route
feature outside support means **no transfer**. Exact feature vocabulary,
normalization, missing-value handling, context radius, and support threshold
are open design choices for review; this proposal does not pick numeric cutoffs.
The prototype uses explicit caller-supplied compatibility decisions so tests
can cover the gate without smuggling in unreviewed thresholds.

Store evidence per candidate under a bounded deterministic table. The table
must not infer safety for an unseen route from a neighboring route merely
because their scalar distances match. Context aliasing is allowed only for
features declared compatible; unknown or missing features fail closed to the
fallback. A table capacity/retention policy must evict by last completed
observation with a stable feature-key tie break. The total storage bound and
serialization migration are separate root-owned decisions.

## Outcome and censoring rules

An update requires a resolver-verified arrival at the selected refuge. Attribute
damage to the observed threat only from resolver events matching the active
threat ID. Damage from another attacker, unexplained health loss, ambiguous
selected endpoint, death, timeout, lost eligible threat, invalid route, priority
interruption, or lab boundary closes the episode for audit and censors it from
the reusable safety estimate. Censored evidence is not a zero-damage or failed
escape sample. Keep closure reason, actual position, movement energy, health
change, and attribution status in the audit ledger.

An arrival with known attribution records separately: safe arrival against the
observed threat, threat-attributed damage, actual movement energy, and time to
arrival. Do not combine safety and energy into one reward. Do not reward kills,
future risk predictions, elapsed age, or another individual's experience.
Unknown data remains unknown and cannot improve a safety estimate.

## Decision gate and fallback

For each current valid refuge route, find only compatible observations with
known attribution. If there is no compatible support for a candidate, retain
the current deterministic instinctive ordering for this choice. Do not insert
an unseen candidate as “zero risk,” and do not compare supported candidates
against unsupported ones as if their estimates had equal confidence. Once the
preregistered support/calibration gate is met, rank candidates by the declared
multi-objective ordering: verified safety evidence first, then observed
threat-damage rate, then actual movement energy, then the existing deterministic
route tie-break. The evidence gate and ordering require root review before
numeric thresholds, model complexity, or formal experiments are selected.

Context similarity must be calibrated against explicit held-out compatible and
incompatible pairs. Report how often the gate shares evidence, rejects it, and
falls back. A context accepted as similar is a model assumption, not proof of
causal equivalence. If support/calibration fails, the decision behavior remains
the fallback and the study reports no reusable learning opportunity.

## Phased work and acceptance

### Phase A — isolated representation prototype (this change)

Implement only a pure helper under `examples/support/refuge_candidate.rs` with
no production imports or checkpoint dependencies. Unit tests cover candidate
evidence matching, ID-independent context aliasing under explicit compatibility,
rejection of incompatible species/habitat/geometry, unknown/censored outcomes,
no-evidence fallback, stable tie-breaking, and bounded deterministic retention.
The helper does not select actions or run a world. Thresholds are supplied by
the caller and remain unset pending root review.

### Phase B — root-reviewed protocol and integration proposal

Review the context schema, safe-transfer boundary, bounded-memory policy, audit
requirements, and proposed support gate. Only after approval, specify a clean
source revision, default-off implementation, persistence migration, and a
fresh preregistration. Integrate through the normal perception/decision,
resolver, outcome ledger, and persistence boundaries. Check disabled exact
state/RNG equivalence and migration before building an immutable lab binary.

### Phase C — bounded refuge generalization experiment

Use fresh seeds and a clean reviewed source. Compare current instinct,
generalized online memory, and a same-trained-memory frozen-update arm where
the question warrants it. Keep each individual's body, age, identity and memory
continuous; do not rescue, replace, or reset the subject. Ensure ordinary
renewable food/water and measure their real movement and interaction costs,
without secretly replenishing body resources. Use actual predator movement and
ordinary perception; no target-forcing or hidden route/physiology input.

Preregister paired initialization, seed roles, scenario bounds, comparison
windows, compatibility positives/negatives, support gate, stop conditions,
and per-seed outcome vectors before evaluation. Require repeated comparable
choices and resolver-verified physical differences to claim an expressed
learning effect. Report safe arrival, threat damage, other damage, censored
episodes, movement energy, resource acquisition, survival, and unavailable
contexts separately. A recorded update or passed unit test is not evidence of
improved safety or ecology.

### Phase D — dependent safe hunt-choice learning

Treat learning that chooses *whether to hunt* as a later, separate capability
after refuge context attribution and censoring are validated. Do not retune the
negative hunt-risk V3 policy from its prior results. First design a new
preregistration that separates visible self-budget cutoffs from unrelated
interruptions and distinguishes the hunter's own prey defense from third-party
damage. Use current visible prey features and the hunter's own resolver costs;
never hidden prey physiology/routes or future state. Keep hunt learning
default-off, bounded, individual-only, and physically costed in the existing
resolver.

Acceptance requires more than a learned-table change: (a) the learned estimate
changes a later choice in a matched visible context; (b) attribution is complete
and each cost is charged exactly once; (c) the selected action completes through
ordinary resolver rules; (d) per-seed physical outcomes show a preregistered
benefit without hiding energy, health, starvation, or censored tradeoffs; (e)
naive, rule-based, online, and trained-frozen controls answer the causal
question; (f) disabled state/RNG and persistence migration pass. If matching
opportunities or outcomes are insufficient, acceptance is unavailable, not
implied. Refuge and hunt changes must not be combined into one initial
experiment.

## Explicit exclusions and limits

No changes to `src/`, shared checkpoint/SQLite schemas, runtime controls,
historical data, formal seeds, registered binaries, live worlds, or deployment
are part of this proposal. No formal World batch is run here. This work does not
promise modular “workers,” social primitives, or emergent intelligence. Such
capabilities require separate bounded specifications, observable inputs,
resolver costs, attribution, and acceptance evidence.
