# Refuge V19 decision candidate — baseline veto with directional evidence

**Status: design for root review only.** No implementation, scenario, seed, or
World run is authorized. V18 and the historical failed/repair artifacts remain
unchanged. This candidate narrows the decision change to cases where the
current deterministic refuge baseline has repeated resolver-attributed damage
and a same-effort alternative has repeated resolver-confirmed undamaged
arrivals under a stricter observed context. All other cases retain the current
baseline and an explicit `unknown` or `incomparable` status.

## Why change the decision contract

V18 requires one fresh exact row for every current candidate. The complete
seed-123 repair-validation diagnosis found no exact different-ID candidate
match in eight choices and 17 candidate-to-row comparisons. Repeating that
audit, or storing the same exact rows in a dormant table, does not resolve the
two obstacles: exact dynamic body/threat values rarely recur in the observed
fixture, and all-candidate support requires outcomes for every current route.
This proposal therefore keeps V18 frozen and describes a separate V19
default-off decision candidate. It does not broaden V18's table or relabel its
data.

## Conservative baseline

The baseline is the existing current-observation fallback: generate only
currently perceived and valid refuge routes, then select the shortest route
with the existing deterministic candidate input order as the tie-break. No
history is treated as evidence that an unsupported route is safe or bad. If
the candidate is off, if the comparison is uncertain, or if the evidence gate
below does not pass, execute this baseline unchanged.

At each eligible `SeekShelter` decision, compute and record both the baseline
candidate and the V19 candidate result from the same pre-decision state. Keep
the baseline decision as a shadow witness even when V19 is enabled. Do not
change other AI priorities, candidate generation, perception, or the resolver.

## Proposed represented-context comparison

Do not use arbitrary numeric bands. Keep these structural and categorical
features exact between a stored source and current query: subject and threat
species; subject and threat maximum body values; subject speed and movement
cost; threat power and health; threat bearing sector; origin habitat; exact
ordered route directions and observed cell kinds; endpoint sector; refuge
kind; turn count; and estimated movement energy. Missing values, invalid
routes, changed categorical values, or changed route signatures are
`unsupported`.

For a route with those features equal, compare the following raw dynamic values
by a componentwise partial order, with no bucketization:

- Subject health, energy, and stamina: higher current values are easier; lower
  values are harder.
- Threat distance in observed steps: greater current distance is easier;
  shorter distance is harder.

Define the order on ease values explicitly: higher subject health, energy,
stamina, and threat distance are easier. `current_strictly_easier_than_source`
means every current value is at least its source value and at least one is
greater. `current_no_easier_than_source` means every current value is at most
its source value; exact equality is included. `current_strictly_harder_than_source`
means the latter relation holds with at least one strict decrease. Full equality
is reported as `equal`, and a mix of easier and harder components is
`incomparable`. Threat power/health, body maxima, bearing, and route/cost remain
exact; this design does not assume that changing them is monotone.

These directions are hypotheses about exposure, not calibrated safety laws.
The comparability relation must be checked against an independent,
predeclared baseline data set before it can authorize decisions. Until that
gate passes, non-exact dynamic comparisons are diagnostic only and action-
ineligible, V19 cannot change a route, and the rule baseline controls. A
historically undamaged arrival is never translated into a current guarantee.

## Evidence, residual rule, and minimum support

Keep each outcome tied to its exact route key and source threat ID. Only
resolver-verified burrow arrivals with known attribution are completed
outcomes. Record threat-attributed damage, other damage, unexplained health
loss, actual movement energy, elapsed ticks, and route arrival separately.
Deaths, threat loss, timeouts, route invalidation, interruptions, ambiguous
arrivals, other-attacker damage, and unexplained health loss are censored for
the completed-outcome comparison; preserve their reason and raw exposure, but
do not convert them to zero damage or successful evidence.

For each current route, summarize completed observations by exact key and
distinct threat IDs. Repeated observations from one ID count as one threat
source for the minimum cross-ID gate. Use only fresh rows within the existing
100-tick TTL. Require at least two distinct source threat IDs for either
evidence class, with at least one source ID different from the current threat
ID; otherwise the history does not establish cross-ID transfer. This is the
minimum needed to observe replication across identities, not an empirical
safety or utility threshold.

- **Positive route evidence:** at least two distinct threat IDs produced
  attributed arrivals with zero threat damage, zero other damage, and zero
  unexplained health loss. Each source context must be strictly harder than
  the current query under the partial order above, and at least one source ID
  must differ from the current threat ID. Report all outcomes and
  source IDs; do not call the route safe.
- **Adverse baseline evidence:** at least two distinct threat IDs produced
  attributed arrivals with positive threat damage and no other or unexplained
  health loss. Each source context must be no easier than the current query
  (equal contexts qualify), and at least one source ID must differ from the
  current threat ID.
  Any contradictory completed outcome among the fresh rows considered for
  that route makes the route `uncertain`; censored attempts remain visible but
  do not count as completed outcomes. Incompatible contexts likewise cannot
  support this class.
- **Unknown/incomparable:** insufficient samples, overlapping or mixed
  outcomes, stale rows, missing attribution, any changed exact feature, or
  dynamic values that cross in opposite directions cannot support an override.

The V19 override is deliberately narrow. First compute the current baseline
route `b`. A different route `r` may replace `b` only when all of the
following hold:

1. `b` has adverse baseline evidence under the no-easier relation.
2. `r` has positive route evidence under the strictly-easier relation and
   evidence from at least two distinct threat IDs.
3. `r` is currently valid and its current estimated movement energy is no
   greater than `b`'s. With the shortest-route baseline this normally means an
   equal-cost tie, avoiding an unregistered safety/energy tradeoff.
4. The evidence is fresh, unique to each route, and has no mixed or censored
   observation used as a completed sample.

If multiple alternatives qualify, choose the earliest in the baseline's
existing candidate order. If no alternative qualifies, choose `b`. Unknown
routes never receive a zero-risk score. The override avoids a route with
repeated observed damage only when a no-more-expensive alternative has
repeated undamaged arrivals in harder contexts. This is a bounded decision
residual around the baseline, not a general ranking model.

Allow at most one V19 override in a shelter episode. If the episode is
interrupted or the route becomes invalid, follow the ordinary AI/resolver
behavior and do not grant a second V19 override for that episode. The equal
estimated-energy condition bounds the override's predicted movement cost at
the decision point; actual energy and any later deviation remain measured
outcomes, never assumed costs.

This comparison still does not prove that past outcomes guarantee future
safety. It is an explicit evidence pattern for a limited route change. Record
the current pre-decision state, baseline route, V19 route/status, every source
row and threat ID, comparability result per source, TTL age, current and
historical physical outcomes, and the eventual resolver closure. Report
arrival, damage, and actual movement energy as separate results; never make a
reward-only claim. Evidence classes are eligibility labels, not calibrated
probabilities or confidence bounds. Keep their counts and physical outcome
vectors visible; do not translate them into a current `safe` status.

## Unresolved calibration requirement

The current fixture does not validate the proposed monotone relation. No
existing result demonstrates that higher subject reserves or a more distant
threat preserve the ordering of refuge outcomes when all exact structural
features match. A completed arrival alone is also insufficient to identify
what would have happened on an unselected route.

Before enabling this decision rule, root must freeze a separate baseline
calibration design that records naturally occurring, exact structural route
contexts and their dynamic values, closures, attribution, and outcomes. It must
evaluate the easier/no-easier relation on held-out contexts and report
misordered cases and censoring. Calibration must include the current
rule-selected route and route alternatives where ordinary behavior naturally
samples them; no table injection or scripted route assignment is allowed. If
the calibration cannot evaluate the relation, or contains a false-easier
transfer, V19 remains default-off. Do not retune the comparison after viewing
the held-out outcomes. Root must approve the calibration criterion before
collection; this document assigns no numerical pass threshold.

If no finite fixed scenario can naturally produce the required route samples
and dynamic overlap, the contract requiring reconsideration is the exact
dynamic represented-context relation, not a reason to rerun the old fixture.
The zero-tick fixture diagnosis remains specific to that fixture. No fields
are selected for relaxation beyond the explicit partial-order hypothesis
above, and that hypothesis remains inactive until calibration.

## State, provenance, and default-off compatibility

V19 is a separate feature and storage namespace. Do not read, merge, expire,
or update V18 or legacy exact rows through this candidate. Keep V18 flags,
default behavior, key, TTL, ranking, and checkpoint interpretation unchanged.
The V19 policy and update flags default off; its table starts empty and is not
inherited by offspring. Each retained sample carries the exact key, source
threat ID, completion tick, completion/attribution status, damage fields, and
actual movement energy. Use bounded rows, deterministic retention, atomic
overflow refusal, and explicit duplicate-source accounting. Source IDs are
provenance, not a feature used to claim physical equivalence.

Add a new checkpoint version only after specifying migration for every
supported prior version. Migrate old checkpoints with V19 off and an empty
V19 table; reject unknown future versions. SQLite changes require separate
schema and old-reader review. Learned records remain individual and are never
inherited.

When V19 is off, decisions, events, RNG draws, serialization, and State+RNG
continuation must match the existing baseline/V18-off path exactly. When on,
read only current ordinary observations and the V19 table; never inject
contexts, select targets, manipulate body state, or bypass intent/resolver
execution. The feature must be reversible via its default-off switch.

## Bounded acceptance plan if calibration and root review pass

This is a future stop-gated plan, not authorization:

1. Freeze the exact source, default-off compatibility evidence, V19 table
   schema, current fallback behavior, scenario bytes, output schema, and a
   baseline-calibration criterion before any World.
2. Use one finite fixed scenario and seen engineering seed 123 only for
   calibration. All agents follow ordinary AI, perception, intent, and
   resolver paths. No post-tick-0 state/body/resource/memory changes, scripted
   actors, target paths, injected rows, new seeds, or age bonus.
3. If a separate decision-mechanism acceptance run is later approved, cap it
   at 1,000 physical ticks, 300 seconds, 1 GiB address space, 128 MiB raw
   output, and zero core bytes. Stop at the first cap, focal death, source or
   audit mismatch, or frozen horizon; preserve partial data and do not retry.
4. Before the first decision-changing override, the raw record must show the
   baseline route, every source observation, distinct threat IDs, matching
   structural fields, componentwise comparison, TTL, minimum evidence counts,
   current movement-effort equality, and the chosen V19 route. Root independently
   recomputes each gate from raw event/closure records.
5. For a mechanistic contrast, require at least one pre-decision state where
   the shadow baseline and V19 candidate choose different currently valid
   routes, with the complete override evidence above. Report only the selected
   route's actual resolver arrival, threat/other damage, unexplained health
   loss, and paid movement energy separately; the unselected baseline route
   has no observed outcome and must not be presented as a counterfactual. A
   route change is mechanism evidence only; a single run is not utility or
   safety qualification. If no contrast occurs, report
   `decision_contrast_not_identified` and stop.

Any formal seeds, utility threshold, multi-candidate performance claim,
alternative comparability relation, or broader transfer rule requires a new
reviewed protocol. V18 remains frozen and default-off; the failed original
refuge artifact and the separate repair-validation evidence remain unchanged.
