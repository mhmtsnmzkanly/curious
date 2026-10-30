# Refuge natural opportunity — feasibility review

**Status: design-only no-go recommendation; no new World run is justified by
the current design.** The previous seed-123 scenario is non-identifying and
must not be rerun unchanged. I did not find a finite tick-zero layout whose
ordinary-AI behavior is presently credible enough to expect repeated
same-individual, all-candidate exact support from different threat IDs. This
is a feasibility conclusion about the proposed next lab, not a claim that
natural exact reuse is impossible in general.

## Question and stopping scope

The desired observation is a fresh shelter choice by one individual where
every currently valid candidate has its own unique, resolver-backed, completed
row that is still within the 100-tick TTL, and at least one such row came from a
different threat ID. Repeated supported choices require the same situation to
recur again under a new threat ID. A useful feasibility check must explain how
the fixed scenario and ordinary AI can naturally create the completed rows
first, then recreate every represented key while the evidence remains fresh.

This is not an invitation to inject a training table, script actors or routes,
reset a body, grant an age bonus, relax the represented key, or retune after
observing outcomes. No seed is registered, no source changes are proposed, and
no World execution is authorized by this review. Any later proposal remains
subject to the tentative ceilings of 1,000 physical ticks, 300 seconds, 1 GiB
address space, and 128 MiB raw output. These are limits, not a target horizon.

## Existing evidence

The original refuge engineering run remains failed. The separately identified
repair validation completed its 270 planned physical ticks on the same seen
engineering seed and fixture; it did not replace or complete the failed
artifact. The subsequent zero-tick diagnosis read only the completed archived
records. It found eight fresh choice records and 17 candidate-to-row
comparisons, all with current threat ID 37, zero exact candidate-row matches,
zero different-ID candidate support, and zero all-candidate supported
post-prefix choices. The eligible comparison rows were still within TTL, but
their exact keys differed. The focal ended at recorded age 90 of max age 105.
The diagnosis is in
[`REFUGE_OPPORTUNITY_DIAGNOSIS`](reports/REFUGE_OPPORTUNITY_DIAGNOSIS.md).

The pure tests prove only the bounded helper mechanics. For example,
`tests/refuge_learning.rs::resolver_finish_fixture_reuses_context_for_a_different_threat_id`
manually constructs two keys and resolver-finish outcomes, then verifies a
later memory choice with a changed threat ID. The fixture does not drive
ordinary World behavior or demonstrate a naturally recurring context.
`tests/refuge_candidate.rs` likewise checks a synthetic contract catalog.
Neither is evidence that a physical scenario will supply the needed sequence.

## Why a tick-zero layout is not yet a credible opportunity design

The V18 reusable key in
[`refuge_generalization.rs`](../src/entity/refuge_generalization.rs) includes
exact current values for subject health, energy, stamina, maxima, speed, and
move cost, plus exact threat health, power, bearing sector, and distance. It
also includes the origin cell kind, endpoint sector, ordered route cell kinds
and directions, refuge kind, turns, and estimated movement energy. IDs and the
current alternative set alone are omitted. The TTL is 100 ticks and retention
is bounded to eight rows.

`rank_candidates` returns a ranking only when **every** candidate has exactly
one fresh row; it rejects missing, duplicate, stale, or reused support.
Resolver outcomes enter the table only for an attributed arrival at the
selected burrow. Thus with two candidates, at least two distinct route-key
outcomes must first be completed by the same individual and remain fresh. If
one route is unsupported, ordinary fallback chooses by shortest route and
stable input order; this can repeatedly sample one route rather than naturally
populate the other. A later third exposure can then be the first
all-candidate-supported choice. To observe a repeated supported choice from
another ID requires another successful recurrence before either row expires.

A symmetric chain of burrows and a line of identical carnivore founders could
repeat route geometry on paper. It does not establish that ordinary agents will
arrive at the exact same subject energy/stamina and threat health/power values,
select each alternative route in turn, complete attributed arrivals, and then
recreate both keys within 100 ticks. Making these states predictable by
blocking one chosen burrow at each step, timing predators to a particular
range, or arranging exact feeding cycles would make the experiment depend on
an unvalidated scripted sequence in all but name. A static initial layout is
allowed, but no source-based or existing-data evidence presently justifies
that sequence as a natural consequence of ordinary behavior.

The repaired fixture supplies direct negative evidence for the old layout,
not for every possible layout. The proposed repeated-burrow geometry is only
a sketch and does not clear the dynamic-state feasibility gap. I therefore do
not freeze a new scenario or recommend spending the World budget on it.

## Narrow next step

Before reconsidering a physical lab, perform a zero-tick, read-only feasibility
review over already-retained records that actually contain refuge choice keys
and resolver closures. For each same-individual sequence, enumerate:

1. candidate route keys at each choice and the exact source closure for each
   completed row;
2. subject energy, stamina, and health; threat ID, health, power, bearing, and
   distance; and row age at each subsequent choice;
3. how many distinct current candidate keys have completed, attributed,
   still-fresh rows at the same choice, and whether source threat IDs differ;
4. whether a later recorded choice repeats the complete support under another
   threat ID.

Use only existing archives; do not launch an unchanged or replacement World
to create input for this check. If the archives contain no adequate decision
and closure records, report that instrumentation is insufficient and stop.
Do not infer opportunity from synthetic tests, spatial symmetry alone, or
repeated IDs. A future scenario proposal would need an explicit finite tick-zero
map and founder list, a non-scripted causal account of how ordinary behavior
can sample all routes and repeat exact body/threat values, and a frozen
opportunity stopping rule before any World is considered.

## Interpretation boundary

No exact different-ID reuse in the old fixed scenario is a non-identifying
result for that scenario. A future zero-tick overlap found in retained records
would establish only that the recorded opportunity exists; it would not
establish utility or transfer safety. Any future natural run must preserve the
exact key and TTL, use ordinary perception, AI, intent, and resolver paths,
and stop at its first cap or preregistered opportunity horizon. No formal
seeds, utility thresholds, parameter sweeps, retries, or deployment are
proposed here.
