# Continuous-life refuge escape study — engineering protocol proposal

**Status: memory-footprint source candidate based on `5ae985e4df219b2fae17d9e1497d7bc4774f1008`; this candidate has not been run.** The earlier frozen proposal
`6a50312c3c6a9a697dc472b0f242025a61c1fa9f` was also never run. One engineering
attempt using source `5ae985e` emitted 225 complete JSONL tick records against
270 planned physical ticks before an address-space allocation failure. The
number of physical ticks executed cannot be established from the record count
alone. This candidate changes only how
pre-tick state witnesses are retained in memory; focal selection, fixture,
horizon, and learning behavior are unchanged. No retry or new seed is
authorized. The attempt was incomplete and provides no full-run utility or
acceptance claim.

The partial evidence is recorded in the [manifest](../data/experiments/refuge-escape-v1-engineering-20261024/manifest.json)
and [root partial audit](../data/experiments/refuge-escape-v1-engineering-20261024/root-partial-audit.json).
The preserved `protocol.snapshot.md` remains the original run protocol.
The separate `refuge_escape_replay` example provides an offline, bounded
zero-tick representation check over preserved JSONL. It does not execute or
resume the simulation; this check has not yet been run on the preserved file.

This document describes one bounded engineering fixture only; it does not
register formal evaluation seeds or support a refuge-utility claim.

## Prior engineering attempt — incomplete infrastructure stop

The single run returned code `-6` after 2.912 seconds when a 632-byte address-
space allocation failed. It emitted 225 complete JSONL tick records: the 30-tick
prefix, 60 ticks each for trained-online, trained-frozen, and naive-online, and
15 ticks of the rule arm. The planned 270 ticks and final summary were not
completed. Treat this as infrastructure-incomplete, not as a biological
failure or completed fixture result.

The independent partial audit matched all 225 recorded body/age transitions,
table count deltas, unchanged frozen rows, and fork differences. It found one
prefix update, one trained-online update, no frozen update, and one naive-online
update. Each of the three completed suffix arms had zero supported greedy
choices; the rule arm is incomplete. Across those three completed suffixes the
recorded totals were terminal health 120, energy 34, water 27, paid movement
energy 36, water intake 56, and food intake 92. These partial totals do not
establish utility. No full vector across four completed arms is available.

## Question

Can the V18 represented-context refuge learner produce a later, resolver-backed
choice opportunity for the same continuously living herbivore when the observed
threat has a different individual ID, and can the trained table alter a later
greedy choice relative to the identical table with updates frozen?

This is a feasibility question. It is not a claim that refuge generalization is
safe, useful, or beneficial. A missing opportunity is reported as
non-identifiability under this fixture. It is not evidence of an engine defect.

## Fixed fixture and engineering role

- Fixture: the checked-in `scenarios/mixed-refuge.json`, built without scenario
  edits. It has 36 herbivore and 2 carnivore founders, renewable resource sites,
  and burrows. All actors use ordinary AI. Predators move and act through normal
  perception, intent, resolver, and event paths.
- Focal selection: before any tick, select the existing herbivore founder with
  the minimum Manhattan distance to any founder whose species hunts herbivores;
  ties go to the lowest entity ID. On this unchanged fixture, focal ID 15 is at
  `(-1,-1)`, distance 1 from carnivore 37 at `(-1,0)` and distance 3 from
  carnivore 38 at `(1,0)`. This is observer selection only: no body, position,
  threat, or policy is changed.
- Focal: entity ID 15, the selected existing herbivore founder. Its body and
  age are not set or changed. Only its default-off
  `refuge_generalization_enabled` flag is enabled; updates remain enabled. Full
  observation is retained for this focal ID so its production decision trace
  supplies the measured perception. All other founders retain default-off
  refuge generalization.
- Engineering seed: 123 only. This seed is already seen and is not an
  evaluation, calibration, or held-out seed. Do not try another seed or alter
  the fixture if gates fail.
- Scope: exactly 30 prefix World ticks, followed by four 60-tick suffix
  branches forked from one captured prefix state: 270 physical World ticks
  maximum across the invocation. The fixture focal starts at age 0 and has
  `max_age=105`; each branch therefore ends at age 90 if alive. This fixed
  horizon avoids a protocol-defined old-age death during the lab while
  preserving the existing body and age rules. Natural combat, hunger, or other
  ordinary deaths remain valid negative outcomes. There is no early success stop and no reset
  within a branch. If the focal dies, its same entity remains dead; no rescue,
  replacement, or respawn is allowed.
- Every branch receives the same actual prefix State, including map, RNG,
  entity IDs, all bodies and ages, focal memory, active episode, and pending
  closures. This is a fork of one lived trajectory, not four newly initialized
  bodies. Different suffix choices may naturally produce different futures.
- No scripted attacks, target forcing, hidden-state action inputs, body/resource
  replenishment, artificial candidate/table injection, or post-result retuning.
  Natural AI reproduction and ordinary resource renewal follow the fixture's
  existing rules.

Before the first seed-123 World tick, root reviews this protocol, the fixed
fixture, and the exact probe/capture source. The reviewed source and protocol
hashes, command, executable hash, environment, and raw output path must then be
recorded in a new immutable engineering-evidence directory. A failed run is
preserved and never overwritten.

## Prefix and four suffix treatments

The one prefix uses the real production World path with the focal learner online.
At tick 30, capture `State::capture` once and restore four independent worlds
from that exact snapshot. Record the complete common checkpoint and prove each
restored arm starts with the same state/RNG before its declared treatment.

1. **Trained online:** retain the table and all state; updates remain enabled.
2. **Trained frozen:** retain the exact table and all other state; change only
   `refuge_generalization_update_enabled` to `false`. Expiry filtering may make
   old rows ineligible for a choice, but rows, counters, timestamps, and
   retention must not mutate.
3. **Naive online:** clear only `refuge_generalized_outcomes`; preserve the
   active episode, pending closures, body, age, map, RNG, identity, and both
   flags. It means a table-blank suffix, not a reset individual. If the inherited
   in-flight prefix episode completes and supplies an update, label that
   carry-over explicitly; it is not independent naive training.
4. **Strong rule baseline:** change only
   `refuge_generalization_enabled` to `false` (the legacy refuge outcome mode
   remains at its fixture default). The production fallback/rules choose
   shelters. The inherited generalized episode remains at the fork and is
   closed by the normal feature-mode boundary; preserve and report that closure.

The online/frozen pair is aligned at the fork, including the active episode.
After their choices diverge, RNG and physical states are expected to diverge.
Choice contrasts are evaluated only at a post-prefix tick where both traces
show byte-equal complete pre-tick State after normalizing only the focal
generalized table and update flag, and the same focal body, position, full
perception/candidate keys, candidate ordering, RNG-before-choice, and fresh
greedy choice. The selected route and its actual resolver movement or received
damage event must differ. The trained table must have unique fresh exact support
for every candidate. Exploration is not a greedy contrast.

## Measurement and opportunity gates

The probe records each production focal decision from `World::last_decisions`
and each resolver event/episode closure once. It does not replay a decision or
call `World::tick` to obtain extra evidence. The audit includes the actual
pre-choice body and RNG, perceived threats/routes/cells, exact candidate keys,
support rows and expiry status, active episode, selected intent, post-tick body,
RNG, focal events, and table snapshots at the prefix fork and branch end.

Report these gates per arm and per suffix tick:

1. **Own completed learning:** at least one focal `RefugeEpisodeEnded` with
   resolver-verified `Arrived`, known attribution, and a corresponding table
   change in an update-enabled arm. A censored closure or row rejection does
   not count.
2. **Different-ID reuse opportunity:** at least one *post-prefix fresh greedy*
   choice under an active observed threat ID that differs from the threat ID
   associated in the event ledger with the supporting completed sample(s),
   where every current candidate has unique, unexpired exact-key support. Keep
   the candidate-level source IDs in the audit; a missing historical source ID
   is unknown support, never presumed transfer.
3. **Online/frozen contrast:** at least one matched, post-prefix fresh greedy
   choice as defined above where online and frozen select different candidate
   routes from aligned pre-choice state and the online table changed through
   resolver-backed evidence. Also report whether the frozen row values and
   metadata remained byte-equivalent for the suffix.

Failure of any gate means the fixture did not identify learned refuge choice.
Stop at the end of the fixed engineering run, retain the complete output, and
report the unmet gate(s). Do not search other seeds, extend the horizon, modify
the fixture, weaken exact compatibility, inject table entries, or infer an
implementation bug from the absence of opportunity.

## Outcomes (kept as a vector)

Per branch and focal individual, report survival and terminal health separately;
threat-attributed damage, other damage, and unattributed health loss separately;
number of completed arrivals and censored closures by reason; actual
resolver-paid movement energy; refuge movement ticks/time-to-arrival; food and
water intake from resolver events; and generalized table values/counts and
changes. Validate each episode cost against movement events and body/event
accounting, charging a movement or damage event once. A completed arrival is
not itself a utility improvement. Reward, row count, or survival alone cannot
support an effectiveness claim.

This engineering run has no utility threshold and is not accepted as formal
evidence. A later formal protocol requires root review after the opportunity
gates are audited. Any proposed 3–4 fresh-seed study, registered-seed role,
formal stopping threshold, and full-run resource budget remains unassigned.

## Resource and evidence limits

The source probe hard-caps itself at 270 physical World ticks and 128 MiB of
JSONL output. The post-review engineering invocation is capped at 300 seconds
and 1 GiB memory; hitting any cap is an infrastructure stop, not a biological
failure. Preserve stdout, stderr, exit status, start/end times, protocol/source/
scenario/executable hashes, and partial raw output on failure. Do not touch
live worlds or deploy. Before any separately authorized invocation, set the
shell core-size limit to zero (`ulimit -c 0`) to prevent an abort from creating
a large core file. No formal seed selection or formal runner is authorized
by this engineering proposal.
