# Refuge generalization schema proposal

**Status: offline proposal and pure helper.** The historical record contains 94
eligible decisions and zero completed records matching their exact keys. That
establishes no observed transfer opportunity and no demonstrated benefit. This
proposal does not change the registered V16 learner, current memory, checkpoint,
SQLite, lab source, or live worlds.

## Intended transfer boundary

The initial hypothesis is deliberately narrow at the feature-representation
level: an individual may reuse its own completed refuge outcome when the
selected situation and candidate-route fields below match, even if the active
threat has a different individual identity or the visible set of valid
alternatives has changed. This does not establish raw physical equality or
causal equivalence.

The reusable key contains only observable physical features:

- subject species and currently observed threat species;
- threat bearing and distance bands, subject energy and stamina bands, and
  locally observed habitat class;
- subject movement speed;
- candidate route identity: endpoint sector relative to the subject, each
  observed step direction and terrain class, refuge kind, and turn count;
- current estimated movement energy for that candidate route.

These fields use exact equality in this first proposal. Any absent required
feature rejects compatibility. No band radius, route-length tolerance, or other
unvalidated threshold is introduced. However, bearing, distance, energy, and
stamina are represented by coarse bands. Different raw distances or body values
can fall in the same band and therefore alias under this key. That aliasing is
an unvalidated model risk; matching the key does not establish that physics or
outcomes are equal. Route cost is route-specific and must be derived only from
visible path features and the subject's own current speed; actual movement
energy remains a separate resolver-measured outcome.

Threat ID belongs only to the active episode attribution/audit record used to
match resolver attack events to the threat being escaped. It is not stored in
the reusable outcome key. The current alternative set is likewise decision
context, not reusable evidence identity. The active choice still consists of
currently perceived and currently valid routes. A route's observed path
signature distinguishes different choices even when scalar costs match. If two
current options have the same feature key, or a stored row would have to support
two options, ranking fails closed and the established choice is used.

At the selected-feature level, only individual threat identity and the
surrounding alternative-set membership may vary. Species, represented danger
geometry, body bands, habitat, speed, route shape, refuge kind, turns, and
estimated cost must match exactly. Raw physical values within equal bands may
still differ, so the schema does not claim exact physical matching.

## Outcome and decision rules

Only a resolver-verified arrival with complete attribution can update reusable
evidence. Threat damage and movement energy are kept as separate measures.
Other-attacker damage, unexplained health loss, death, threat loss, timeout,
invalid route, priority interruption, ambiguous arrival, or lab boundary is
censored and remains in the audit ledger. Censored evidence is neither success
nor zero damage.

Use evidence only when every currently valid candidate has unique compatible
support above a separately reviewed minimum. If any candidate is unsupported,
if evidence is ambiguous, or if two candidates map to the same row, use the
existing fallback for the whole decision. Never treat a missing row as zero
risk. Rank supported evidence by safe-arrival rate, then threat-attributed
damage rate, then actual movement-energy cost, with the existing deterministic
route order as the final tie break.

If a future preregistered study has zero transfer opportunities, the result is
“no opportunity to test transfer”; retain fallback and stop that analysis. If
empirical calibration finds false-compatible physical mismatches, including
within-band differences, or otherwise fails its declared criterion, report
failure and stop. Do not broaden the compatibility region or retune against
the failed calibration set.

## Offline fixture acceptance

`tests/refuge_candidate.rs` contains a finite fixed synthetic contract catalog
and verifies that outcomes from two external threat identities update the same
reusable row and that the aggregate affects ranking.
Positive fixtures vary only the external active threat ID and visible
alternative set while retaining the same selected feature key. Negative
fixtures independently change subject species, threat species, represented
bearing, distance band, energy band, stamina band, habitat, subject speed,
estimated route cost, observed route path, refuge kind, or turns. Missing speed
or cost also fails closed. These fixtures verify implementation categorization
only. They are not a held-out empirical test, do not measure within-band alias
risk, and do not calibrate transfer from world outcomes. Future empirical
calibration must include raw-value pairs that differ within each shared band,
as well as route and cost variation. No physical cutoff is selected here.

The pure helper additionally checks censored evidence, complete candidate
support, no row reuse, deterministic safety/cost ranking, bounded retention,
and aggregate overflow. It does not make a world decision or read hidden state.

## Persistence migration proposal only

Production integration must wait for root's serial review after the water lab
source V17 is frozen. Preserve the legacy exact-key outcome table and its reader semantics;
never reinterpret or merge its rows as generalized evidence. Add a distinct,
default-empty generalized-outcome collection with this exact feature schema.
Existing checkpoints migrate to an empty collection, while unknown future
schema revisions remain rejected. Active episode state keeps the threat ID and
species required for attribution; reusable rows do not contain either threat
ID or the current alternative set. Disabled/default behavior remains unchanged.

Checkpoint migration and SQLite persistence are separate interfaces and need
independent review. This proposal does not assign a version number or prescribe
SQLite columns; integration must inspect the then-current formats, specify
round-trip and old-reader behavior, and test state plus RNG continuation before
any immutable lab build. Learned experience remains individual and is not
inherited.
