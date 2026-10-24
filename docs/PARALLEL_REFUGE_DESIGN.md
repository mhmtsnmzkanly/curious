# Refuge context generalization — design proposal

**Status: default-off V18 implementation candidate; no empirical qualification.**
The implementation preserves the registered V16 exact-key table and SQLite V2.
The eight historical seeds and all 94 multi-shelter decisions are
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
The historical evidence is 94 eligible decisions and zero matching completed
records, so there is no demonstrated reuse opportunity or benefit.

## Candidate context representation

The initial prototype separates reusable physical features from active episode
attribution:

1. **Active episode attribution:** retain the currently visible threat ID and
   species while an escape episode is active, so resolver attack events can be
   attributed to that threat. Threat ID is not part of the reusable outcome
   key.
2. **Observable physical situation:** include subject species, threat species,
   bearing and distance bands, energy/stamina bands, local habitat, and current
   subject speed. All are derived from present perception/body state.
3. **Candidate route:** include each visible route's relative endpoint sector,
   observed step directions and terrain, refuge kind, turns, and estimated
   movement energy derived from visible path and current speed. Record actual
   movement energy after execution; the estimate is not an outcome.

The current pure prototype implements exact equality over its selected feature
representation. The threat ID and current alternative-set membership are
outside the reusable key, so only those two facts may vary at the represented
feature level. Bearing, distance, energy, and stamina use coarse bands; raw
physical values may differ within the same band. Such aliases are an unvalidated
model risk, not evidence of physical or causal equivalence. Missing features,
different represented species, bands, habitat, speed, route, refuge kind,
turns, or estimated cost mean **no transfer**. No tolerance or similarity
radius is introduced without new preregistered empirical calibration.

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

The offline helper is checked against a finite fixed synthetic contract catalog:
identity/alternative-set-only changes are intended compatible cases, while
independent selected-feature changes and missing values are incompatible.
These fixtures test code categorization only; they are neither a held-out
empirical test nor calibration from world outcomes. Future empirical
calibration must test raw-value differences within each shared coarse band.
A future study must report how often the gate shares evidence, rejects it, and
falls back. If the historical-style audit finds zero transfer opportunities,
report no opportunity and stop. If a preregistered calibration criterion fails,
report failure and do not retune the gate against that failed set.

## Phased work and acceptance

### Phase A — isolated representation prototype (historical)

Implement only a pure helper under `examples/support/refuge_candidate.rs` with
no production imports or checkpoint dependencies. Unit tests use a finite
synthetic contract catalog, test exact selected-feature matching with threat
identity and alternative-set metadata outside the reusable key, and cover missing and
censored evidence, no-evidence fallback, distinct-route identity, no row reuse,
stable tie-breaking, and bounded deterministic retention. The helper does not
select actions or run a world. The schema and migration proposal are in
[`REFUGE_GENERALIZATION_SCHEMA.md`](REFUGE_GENERALIZATION_SCHEMA.md).

### Phase B — root-reviewed implementation candidate

Root approved a default-off per-individual mode, separate bounded table, exact
raw body/distance representation, and V18 checkpoint with SQLite V2 unchanged.
The implementation uses the ordinary perception, decision, resolver, event,
and checkpoint boundaries. It has not established safe transfer or ecological
benefit. Root still reviews the final schema, closure behavior, and V18 migration
before any experiment binary or protocol is considered.

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
