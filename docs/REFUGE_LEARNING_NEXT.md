# Refuge learning next — single-candidate natural transfer gate

**Status: design candidate for review only.** This proposes the smallest
default-off extension that could test whether an individual's own completed
refuge outcome naturally recurs under a different threat identity, without
claiming route-ranking benefit. It does not alter or replace V18, revise any
failed or completed evidence, assign a seed, or authorize a World run.

## Evidence and question

V18's exact generalized key and table remain intact. On the completed seed-123
repair-validation fixture, the zero-tick diagnosis saw eight fresh choice
records and 17 candidate-to-row comparisons, all with threat ID 37; there were
zero exact matches, zero different-ID support, and zero all-candidate
supported post-prefix choices. This is non-identifying evidence for that
fixture, not evidence of global impossibility. The earlier incomplete run
remains marked failed; the later technical validation remains a separate
artifact.

The next narrow question is: **does any ordinary World produce even one
same-individual, exact-key, resolver-completed single-route outcome that is
queried later under a different threat ID while still fresh?** A single
candidate removes the all-candidate ranking requirement, but does not weaken
the exact context match. Since the only available route is selected either
way, this is an opportunity and outcome-transfer gate; it cannot demonstrate
route-ranking benefit or a behavioral effect.

## Candidate: default-off single-candidate evidence probe

Keep the existing V18 key fields and 100-tick TTL unchanged. Add a separate,
default-off probe mode with a distinct, bounded table; do not reinterpret V18
or legacy rows. The probe is eligible only when the normal current decision is
`SeekShelter`, the V18 candidate builder yields exactly one valid route with a
complete exact key, and the focal individual's query has an exact fresh row
from at least one completed resolver arrival. The current threat ID is
retained in the active attempt and its event provenance, but excluded from
exact key equality exactly as in V18.

The probe never changes candidate generation, refuge routing, or the current
choice. With one candidate, ordinary fallback already selects that route. It
does not call an outcome "safe" based on a past mean, convert missing history
to zero, or allow a one-sample result to override current perception. An
unsupported or ambiguous query stays `unknown` and takes the ordinary current
fallback unchanged. This lets a future lab test whether the exact stored
context recurs naturally and measure the next resolver outcome, while
preserving the current-observation baseline.

### Query, update, censor, and uncertainty contract

- **Query:** use the exact V18 candidate key from the current focal body,
  threat observation, origin, and route. Require one valid candidate, one
  unique row, `completed > 0`, `last_completed_tick <= current_tick`, and age
  at most 100 ticks. A cross-ID opportunity additionally requires resolver
  provenance for a completed source whose threat ID differs from the current
  threat ID. No source identity is inferred from an aggregate count alone.
- **Update:** add an outcome only after the resolver confirms arrival at the
  selected burrow and attribution is known. Retain exact threat damage and
  actual movement energy separately. Retain one route-key row per individual;
  preserve sample count and source-ID provenance. A bounded provenance summary
  must distinguish a single source ID from multiple distinct source IDs
  without changing key equality. Retain deterministic eviction, overflow
  refusal, and TTL behavior; capacity and counters must be root-reviewed
  before implementation.
- **Censor:** threat loss, timeout, death, invalid route, interruption,
  ambiguous arrival, another attacker's damage, or unexplained health loss
  does not update the completed outcome row. Keep these attempts in the raw
  ledger with closure reason and source IDs; they are unknown/censored, not
  safe or zero-damage observations.
- **Uncertainty:** report completed, attributed arrivals; safe-arrival count;
  threat-attributed damage; actual movement energy; number of distinct source
  threat IDs; and censored counts. A row with one successful sample is only a
  mechanism-level observation. Neither a safe sample nor an average establishes
  that the current route is safe. Do not pool across focal individuals, seeds,
  route keys, or arms. No utility threshold or confidence cutoff is proposed.
- **Physical measurements:** at each query record focal health, energy,
  stamina, maxima, movement cost and speed; current threat ID, species, health,
  power, bearing, and distance; the complete observed route; row age; resolver
  arrival; threat damage; other damage; unexplained health loss; actual
  movement energy; and elapsed ticks. Keep current observed threat conditions
  separate from historical outcomes. Food, water, or unrelated encounter
  events are not evidence that the refuge route was safe.

The query is observational only. It does not rank candidates because it is
restricted to one, and it cannot support claims that a learned choice
outperforms the conservative current-observation/fallback policy.

## Scenario decision

No defensible finite tick-zero scenario is frozen for this gate. A lone
herbivore, one burrow route, food nearby, and two identical carnivores in a
fixed approach corridor is a plausible geometry sketch, but not a credible
prediction of the required trajectory: the first predator may remain the
visible threat, take damage, block the route, or change the focal's body state;
the second may never present the same health, power, bearing, and distance.
The focal must also return naturally with exactly matching health, energy,
stamina, and route features before the first row expires. Designing initial
positions to schedule those events would create an unsupported scripted
sequence. Do not run that sketch as a World lab.

The single-candidate gate removes the need to sample and support every
alternative route, but leaves exact body/threat-state and route equality
unchanged. If a future reviewed scenario cannot credibly repeat that exact
context, the contract that would need empirical reconsideration is V18's
dynamic exact-key representation (not its all-candidate rule alone). The
current evidence does not identify which fields can be relaxed safely. Any
future compatibility proposal must calibrate field-level near matches against
resolver outcomes on a separate, frozen data set; do not invent numeric bands
from the failed opportunity run or treat a historically safe mean as a current
safety guarantee.

## Compatibility and migration

Keep all V18 and legacy exact tables, readers, and update semantics unchanged.
The new probe flag and update flag default off; new history starts empty and
is not inherited by offspring. Do not write probe samples into the V18 table.
Introduce a separate checkpoint version only after specifying explicit
migration from every supported prior version. Old checkpoints migrate with
the probe disabled and an empty new table; unknown future versions remain
rejected. SQLite compatibility must be separately reviewed before any
persistence change.

Feature-off execution must preserve the existing V18-default-off behavior,
random-number consumption, decisions, checkpoint continuation, and resolver
events exactly. Probe reads may use only the current `DecisionTrace`, normal
perception, frozen key builder, and resolver-produced events. The probe must
not inject memory into AI decisions or add random draws. A build and behavior
compatibility review must bind the source, binary, protocol, fixture, and
capture schema before any future data collection.

## Bounded acceptance sequence if a scenario is later reviewed

This is a stop-gated acceptance outline, not a run authorization:

1. First review and freeze a finite t0 scenario with exact founder bodies,
   positions, resource cells/sites, and one ordinary-AI focal. Explain why a
   distinct second threat can naturally recur with the same complete V18 key
   while the focal remains alive. No state, body, memory, route, threat, or
   resource is changed after t0; all agents use normal AI, perception, intent,
   and resolver paths.
2. Before World work, require pure tests for exact-key match/mismatch,
   different-ID provenance, inclusive TTL boundary, censor/no-update cases,
   bounded retention, and feature-off checkpoint/RNG/decision compatibility.
   These tests validate code mechanics only.
3. If root approves a physical engineering gate, use seen engineering seed 123
   only, one fixed scenario, and one normal World trajectory. Capture every
   focal query and resolver closure. Stop at the first of 1,000 physical ticks,
   300 seconds, 1 GiB address space, 128 MiB raw output, focal death, or the
   frozen opportunity horizon; core dumps are disabled. Preserve partial raw
   data on infrastructure failure. No retry, parameter sweep, seed
   replacement, table injection, age bonus, or post-result scenario change.
4. The opportunity gate passes only if a completed, attributed row from the
   focal is later queried with exactly the same full key, exactly one current
   valid candidate, a different current threat ID, and row age no more than
   100 ticks. Report the subsequent physical closure/outcome independently.
   If absent, label this scenario `opportunity_not_identified` and stop. A pass
   establishes one observed transfer opportunity only; it does not establish
   route-ranking improvement, safety, utility, causality, ecological
   qualification, or biological benefit.

Any multi-candidate ranking test, formal seed use, safety threshold, or
compatibility broadening needs a separate question and frozen protocol. This
design preserves the historical failed refuge artifact and the separate
repair-validation result without upgrading either claim.
