# Refuge residual candidate — pure research prototype

**Status: unqualified, default-off research helper.** This prototype is not
connected to `World`, the learner, refuge intent selection, checkpoint
serialization, SQLite, or RNG. It cannot change a live or simulated action.
V18 remains frozen. The old failed run and its separate technical repair
validation retain their original meanings and artifacts.

The helper is in [`refuge_residual_candidate.rs`](../examples/support/refuge_residual_candidate.rs);
its hand-built pure checks are in
[`refuge_residual_candidate_check.rs`](../examples/refuge_residual_candidate_check.rs).
The helper lets a future integration test a small per-individual residual
predictor without requiring an exact key match for every current route. This
is a mechanism prototype, not evidence that generalization is reliable, safe,
useful, or biologically beneficial.

## Inputs and current baseline contract

The caller supplies each currently perceived route's endpoint and refuge kind,
current observable context, a baseline physical prediction, its calibration
status, and its stable order under the existing route rule. The helper does
not invent an expected-damage value or silently substitute zero. A missing
baseline component makes that candidate unrankable and the query falls back
to the baseline route order. Every reported result carries the supplied
baseline calibration status.

The baseline vector keeps three physical outcomes separate:

- resolver-attributed threat damage;
- resolver-paid movement energy; and
- elapsed ticks to refuge arrival.

The current route gives a direct estimate of movement effort and travel time,
but this workspace has no calibrated current-observation expected-damage
predictor for every candidate route. A future integration must identify and
validate that predictor before presenting residual-adjusted values as physical
expectations. The helper's fixtures use explicitly uncalibrated baseline
vectors to check mechanics only.

The fixed nine-element feature vector uses continuous Q16 ratios from the
choice-time observation: subject health, energy, and stamina relative to their
maxima; threat distance relative to the subject's known vision range; perceived threat power
and observed threat health each relative to the subject's own maximum health;
route steps relative to that vision range; turns relative to route steps; and
estimated route energy relative to subject maximum energy. Threat maxima are
not inputs: the current perceived-entity observation does not expose them.
Using the subject's known maximum health for threat power/health is arithmetic
normalization only, not a claim of physical equivalence between subjects and
threats. Missing observed threat health, zero denominators, invalid route-turn
counts, or ratios above the fixed numeric range produce missing features and
an `Unknown` result; the helper does not infer hidden target capacity.
There are no exact-match bands. Q16 is the arithmetic representation, not a
claim that nearby physical situations are equivalent. The route row key is
only endpoint `(x, y)` plus refuge kind; the model instance is private to one
owner. Source threat IDs are retained only as bounded provenance and never
enter the feature vector.

## Per-individual update and query

An enabled, unfrozen model can update only from that same owner's own
resolver-verified `Arrived` episode with a known threat ID and complete route
features and baseline. The training target for each output is the separate
physical residual `observed outcome - choice-time baseline`. Any other closure,
missing attribution, other-attacker damage, or unexplained health loss censors
the whole episode and makes no update. Censored outcomes are never converted
to zero damage or successful arrivals.

Each route row holds three independent, fixed-point online linear residual
predictors over the nine features plus bias. The prototype uses Q16 signed
weights, a fixed 1/16 update step, a per-observation residual clip of 256
physical units per component, and a prediction clip of the same size. Checked
arithmetic, a 1,000,000-unit physical-input bound, eight route rows, a 256
update cap for this prototype instance, and a 32-entry provenance ring keep
the helper bounded. An invalid value, full route table, update cap, or
arithmetic-bound failure leaves the model unchanged. These are engineering
bounds for this pure prototype, not fitted hyperparameters, evidence
thresholds, or a proposed lifetime-learning guarantee for an integrated
system.

An untrained route returns the supplied baseline unchanged, residual zero,
and status `Unknown`; that status does not mean safe. A route with learned
updates returns status `Provisional` with its update count and the explicit
baseline calibration status. The prototype has no calibrated confidence
interval: sample count is support metadata only. Frozen mode reads its current
model but rejects updates. Disabled mode returns baseline values and stable
baseline-order selection unchanged. These claims apply only to this pure
helper; engine, checkpoint, and State+RNG compatibility have not been
implemented or tested.

For diagnostic comparison only, the helper orders available predicted vectors
lexicographically by threat damage, paid movement energy, elapsed ticks, then
the current baseline order. This is an explicit physical-vector ordering, not
a scalar reward and not a live policy. If no physical prediction differs from
its supplied baseline, including an enabled empty model or zero-residual
updates, the helper preserves the exact baseline choice and values instead of
re-ranking baseline vectors. An unknown route may remain in the
comparison at its unchanged supplied baseline, with `Unknown` visible in its
row. If any candidate lacks a baseline vector or model arithmetic fails, the
helper returns the unchanged baseline choice for the whole query.

## Pure checks and limits

The standalone checker covers: own attributed adverse versus undamaged
episodes shifting only the diagnostic ranking; default-off exact baseline
choice and values; unknown route status; owner isolation; censor, frozen, and
disabled no-update behavior; missing-feature/baseline fallback; route and
update caps with transactional refusal; bounded clipping; and provenance
retention. All episodes in the ranking fixture enter through the public
completed-episode update API; no learned row or coefficient is injected.

This checker constructs no `World` and runs zero simulation ticks. The
prototype establishes only that the proposed bounded mechanism can express a
route-specific residual and produce a deterministic diagnostic comparison.
It does not establish transfer validity, physical accuracy, safety, refuge
utility, route-selection benefit, fitness, or production readiness.

Before any integration, root must review the baseline damage model, feature
validity, outcome attribution, persistence/migration design, disabled-path
compatibility, and the exact action-selection contract. Any World acceptance
requires a separate frozen protocol with finite scenario, resource caps, and
root audit. Do not infer an action-policy result from these pure mechanics
checks.
