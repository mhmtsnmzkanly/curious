# Ecology measurement audit

This report describes the measurement contract implemented by the bounded
`ecology_candidate` diagnostic. It does not report a biological evaluation or
acceptance result.

## Accounting contract

At each physical tick, the observer records opening and ending plant-food and
water stock, renewal events, and resolver-backed intake events. It computes

`food residual = ending food − (opening food + food renewal − plant meals)`

and

`water residual = ending water − (opening water + water renewal − drinks)`.

Corpse meals are recorded separately and do not subtract from map plant-food
stock. A nonzero residual is preserved as unexplained. Resource cells are also
compared by location and any food↔water replacement is explicitly listed.
Residuals identify missing accounting evidence; they do not identify a cause.

## Identity, events, and traits

The observer retains an ID ledger for each tick-zero founder and each observed
birth, including species, generation, parents, death cause, and a genotype plus
expressed-phenotype snapshot. The ledger survives a slot being removed from the
world. `EntityMated` events identify actual breeders and parentage; `EntityBorn`
events are checked against those parent events. Reports distinguish generation
one/two births, generation-one breeders, and living descendants that themselves
bred. Unknown parents, unparented births, and slot/event parent mismatches remain
visible as audit issues.

Food, corpse food, and water acquisition, resolver-paid movement energy, and
trait maintenance requested/paid are grouped by the actor's recorded species.
Basal metabolism, reproduction transfer, healing, starvation/dehydration body
loss, and other body-energy changes cannot be fully reconstructed from the
existing event stream. Those quantities are labeled unknown; the observer does
not infer them or claim body-energy conservation. Snapshot and last-observed
health, energy, and water are exposed as states, while the un-attributed change
between them remains explicitly unknown.

## Verification boundary

The targeted runtime tests use seed 123 and thirteen physical ticks total. They
exercise resource renewal/reconciliation, a deliberate unexplained resource-kind
replacement, distinct plant/corpse meal pools, and persistence of founder identity
after world-slot removal. An audited and unaudited twin also match captured
checkpoint state and RNG after four continuation ticks. A synthetic two-generation
parent/birth event fixture verifies the F1 breeder, F2 birth, living
bred-descendant, and genotype/phenotype snapshot counters; a separate World
resolver fixture performs one real mating and captures its child snapshot and
trait maintenance. Synthetic parent-event fixtures cover unknown parentage and a
generation/slot mismatch; those fixtures do not simulate mating. The detail-cap
fixture checks 2,001 audit observations without advancing the World.
Detailed reconciliation and replacement rows retain at most 2,000 entries;
cumulative checked/mismatch counts, signed residual totals, and first/last
anomaly summaries are retained after the detail cap. The configured diagnostic
cap remains 100,000 ticks, so full per-tick reconciliation detail is available
only for the first 2,000 ticks; aggregate residual accounting continues after
that cap. Positive and negative residual magnitudes are totaled separately so
unexplained losses cannot cancel out against unexplained gains.

These fixture checks do not establish sustainable habitat, trait fitness,
species persistence, or evolutionary advantage. No seed sweep, population
batch, or registered science evaluation was run. The candidate remains a
diagnostic, and its CLI tick limit is not an overall memory, disk, or wall-time
budget: the identity ledger, breeder sets, and issue vectors grow with observed
individuals and events. Any future large batch needs runner-enforced RSS,
output-size, and identity-count limits under a separately declared budget.
