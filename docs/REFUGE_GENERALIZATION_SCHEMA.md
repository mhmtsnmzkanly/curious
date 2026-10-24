# Refuge generalization schema proposal

**Status: default-off V18 implementation candidate; no empirical qualification.**
The historical record contains 94 eligible decisions and zero completed records
matching their exact keys. That establishes no observed transfer opportunity
and no demonstrated benefit. This implementation does not change the V16 exact
key table's reader/update semantics or SQLite V2.

## Intended transfer boundary

The initial hypothesis is deliberately narrow at the feature-representation
level: an individual may reuse its own completed refuge outcome when the
selected situation and candidate-route fields below match, even if the active
threat has a different individual identity or the visible set of valid
alternatives has changed. This does not establish raw physical equality or
causal equivalence.

V18 supports herbivores only because current burrow perception and shelter
choice are herbivore capabilities. The key carries the actual subject species
from intent collection and rejects non-herbivore saved contexts; it does not
pretend the subject species is interchangeable.

The new reusable key contains only current observable features:

- subject species and currently observed threat species;
- threat health and perceived power, eight-sector bearing, exact observed step
  distance, and exact subject health/max-health, energy/max-energy, and
  stamina/max-stamina values;
- subject speed and movement energy cost;
- the observed cell kind at the subject's origin;
- candidate route identity: endpoint sector relative to the subject, each
  observed step direction and terrain class, refuge kind, and turn count;
- every route direction and the observed cell kind at every route step, plus
  endpoint sector, refuge kind, turns, and current estimated movement energy.

All listed numeric values are exact-match fields in this implementation. Any
missing required field or route cell rejects generalized support for the whole
choice. Bearing remains an eight-sector representation, so raw directions within
one sector alias. Exact equality in these selected features does not establish
causal or physical equivalence and has no empirical safety calibration. Route
cost is path length times the subject's current movement energy cost; actual
movement energy remains a separate resolver-measured outcome.

Threat ID belongs only to the active episode attribution/audit record used to
match resolver attack events to the threat being escaped. It is not stored in
the reusable outcome key. The current alternative set is likewise decision
context, not reusable evidence identity. The active choice still consists of
currently perceived and currently valid routes. A route's observed path
signature distinguishes different choices even when scalar costs match. If two
current options have the same feature key, or a stored row would have to support
two options, ranking fails closed and the established choice is used.

At the selected-feature level, only threat identity and current alternative-set
membership may vary. Threat species and current visible threat health/power,
represented bearing, exact observed distance, subject body values, habitat,
speed, movement cost, route shape, refuge kind, turns, and estimated cost must
match exactly. This is a representation boundary, not a claim of physical
equivalence or safe transfer.

## Outcome and decision rules

Only a resolver-verified arrival with complete attribution can update reusable
evidence. Threat damage and movement energy are kept as separate measures.
Other-attacker damage, unexplained health loss, death, threat loss, timeout,
invalid route, priority interruption, ambiguous arrival, or lab boundary is
censored and remains in the audit ledger. Censored evidence is neither success
nor zero damage.

Use evidence only when every currently valid candidate has unique compatible
support. The implementation's one completed sample is a mechanics gate only,
not empirical qualification. If any candidate is unsupported,
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

The V18 implementation keeps the legacy exact-key outcome table and its reader
semantics unchanged; legacy rows are never reinterpreted or merged. A distinct,
default-empty generalized-outcome collection is bounded to eight rows and each
row expires for eligibility after 100 ticks. These are retained mechanics
bounds, not tuned empirical thresholds. Frozen updates leave rows, counters,
timestamps, expiry cleanup, and retention unchanged; current age still excludes
expired evidence at decision time. Online updates use checked atomic counters
and deterministic oldest-row eviction. Children may inherit the enable/update
flags only when both parents have them enabled; learned rows and episodes start
empty.

The single active refuge episode stores its captured learning mode and optional
generalized key. The threat ID remains in active attribution state. Each close
emits one existing `RefugeEpisodeEnded` event; generalized events include the
optional generalized context. The resolver credits at most one table according
to the episode's captured mode. If any current candidate lacks a complete key,
the whole choice uses the ordinary refuge fallback and opens no generalized
episode. Checkpoints V1–V17 migrate by turning the
new mode off and clearing only new generalized state; legacy refuge and V17
water state are preserved. SQLite remains V2. Unknown future checkpoint versions
remain rejected.

Checkpoint migration and SQLite persistence are separate interfaces and need
independent review. This proposal does not assign a version number or prescribe
SQLite columns; integration must inspect the then-current formats, specify
round-trip and old-reader behavior, and test state plus RNG continuation before
any immutable lab build. Learned experience remains individual and is not
inherited.
