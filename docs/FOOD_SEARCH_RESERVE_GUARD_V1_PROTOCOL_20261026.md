# Food search reserve guard V1 — paired descriptive screen

**Protocol contract.** A phase is registered only when its registration JSON
has `status=registered` and `root_reviewed=true`, and pins the clean source,
scenario, and binary identities. This protocol describes a default-off K2
body/decision-boundary candidate. Its purpose is to test whether energy-bounded
discretionary food-search movement avoids the recorded food-access failure mode,
rather than merely moving starvation later. It is a small descriptive screen,
not a utility qualification or deployment decision.

## Frozen question and candidate

Across a unique-food stress fixture, a resource-rich herbivore meadow, and a
sparse-resource herbivore fixture, does enabling the `food_search_reserve_enabled`
policy change physical food access, survival exposure, and reproduction relative
to the same baseline World at the same seed and horizon?

The candidate is the one isolated policy flag in the current source. It applies
only to AI `SearchFood` decisions whose intent is `Move`; it does not change
direct `SeekFood`, water seeking, threat response, or resolver rules. Given
current energy `E`, known next-tick basal plus expressed vision/stamina
maintenance `b`, movement energy cost `c`, selected path length `p`, and
effective speed `s`, it caps the already selected path at
`min(p, max(s-m, 0), floor(max(E-b, 0)/(2*c)))`, where `m` is movement
already used this tick. A zero cap produces a one-tick idle;
otherwise the path is shortened only when necessary. It uses no extra RNG,
future resources, route search, or resource-availability guarantee. The
resolver still charges actual movement and metabolism. The option is default-off.

## Fixtures and seeds

| Role | Frozen start | Seeds | Ticks per World |
|---|---|---:|---:|
| Unique-food stress | Exact V1 lab geometry/body: one AI at `(0,0)`, default herbivore with `max_age=2000` and `maturity_age=2000`, food `(0,-1)` capacity 30/refill 2 every 2 ticks, water at `(-2,0)` and `(4,0)` with the original capacities/refill schedules. Preserve focal learning enabled, updates enabled, context TTL enabled, and water residual disabled in both arms. Seed 62199 remains historical diagnosis only and is never rerun. | 62401–62406 | 300 |
| Resource-rich meadow | Read-only `scenarios/meadow.json` fixture | 62401–62406 | 300 |
| Sparse-resource herbivore | Read-only `scenarios/ecology-traits-lab.json` fixture | 62401–62406 | 300 |

The scenario files are read without edits. For those fixtures, enable genetics
with mutation 0 per mille and extra-trait expression off in both arms. In all
fixtures explicitly disable unrelated
policy options (mate rendezvous, reserve opportunities, water residual, injury
capacity penalty, juvenile care, ambush predation, and local coordination).
The V1 single-founder fixture retains its original learner settings. Both arms
use the same fixture, body, map, resources, seed, and initial RNG state; the
only arm difference is `food_search_reserve_enabled`. Compare normalized
initial States after setting that one flag to false. RNG streams may diverge
after behavior differs.

The design has 3 fixtures × 6 seeds × 2 arms = 36 Worlds and 10,800 World ticks.
Every World runs 300 ticks, including an empty population. No rescue, resource
addition, body reset, post-start intervention, baseline weakening, parameter
sweep, seed replacement after outcomes, retry, or automatic tuning is allowed.
Before registration, scan the experiment archive's recognized text inventory
with a 64 MiB maximum line bound. For every `*.jsonl.zst` payload, verify its
sibling `*.jsonl.archive.json` index is marked verified and its recorded archive
size/SHA-256 match the payload; include each index's declared seed roles in the
inventory result. The scanner code, index verifier, and archive contract are
source-pinned. This verifies indexed seed coverage without decompressing the
payloads. Seeds 62401–62406 must be unseen in the text records and verified
indexes before registration. SQLite payloads are not decoded by this scan;
their declared seed roles are taken from retained run manifests and experiment
directory metadata. This is the scan's SQLite coverage limit.

## Measures and interpretation

The primary outcome is living individual-ticks, reported as absolute baseline
and candidate totals and paired differences for each fixture/seed. No composite
utility or post-result pass threshold is defined.

Report independently for every World:

- Resolver food-energy gain, movement energy actually paid, and their ratio
  where movement cost is nonzero. Food intake is not net energy; metabolism,
  water stress, and other costs remain separate.
- Each focal founder decision, position, pre-choice body state, perceived food,
  intended target/path, resolver outcome, and relevant movement/eat/approach
  events. Count `Approach/OutOfRange` as such; that event alone does not prove
  an energy block. Attribute an energy-limited access episode only when the
  pre-choice energy and full perceived path cost, partial paid movement, and
  failed approach jointly support it. Otherwise attribution is Unknown.
- For the V1 focal founder, separately count zero-energy visible-plant-food
  approach failures (pre-choice energy 0, nonzero Eat approach path, zero paid
  movement) and underfunded visible-plant-food approach failures (pre-choice
  energy below the Eat approach path cost and partial paid movement strictly
  between zero and that cost). Both also require the selected non-corpse Eat
  target to match perceived positive plant food and an `Approach/OutOfRange`
  event. Treat these as budget-deficit diagnostics, not exclusive causal
  attribution: obstruction, stamina, and other blockers may also contribute.
  Other approach rejections remain unattributed.
- Zero-energy living entity-ticks, first zero-energy observation, death tick and
  cause, food/resource stocks, and health/water context. Report time from a
  focal individual's first zero-energy tick to its death when observable. A
  later death without a successful meal and without increased survival exposure
  is a delay, not evidence that food access was solved.
- Births and deaths by generation, F1 births and breeder identities, F2 births
  and breeder identities, living F1/F2 breeders, and living descendant
  breeders at tick 300. Preserve
  unknown lineage/cause values; never infer reproduction from mate proximity.

Summarize the 6 within-fixture paired differences with median and range, while
retaining all per-seed vectors and negative outcomes. Reproduction, resource
use, food access, and survival exposure remain separate measures. The short
300-tick screen may not expose later-generation outcomes in every fixture; no
claim beyond the observed horizon is supported.

## Raw evidence and execution limits

The Rust child writes a header with full initial checkpoint State, one ordered
row per tick with compact population/resource/body totals, per-tick reason
counts, intervention rows for every `FoodSearchReserve` decision (pre-choice
energy, maintenance, movement cost, energy cap, remaining-speed cap, applied
cap, chosen path/idle and speed; the proposed pre-cap path length is not
available from the post-intervention decision trace and stays Unknown),
full focal-founder decision traces and relevant resolver events, compact
birth/mating/death lineage, and a summary with final checkpoint State. Plant
food gain is resolver `EntityAte.amount` excluding corpse meals; movement cost
is resolver `EntityMoved.energy_cost`. Endpoint restore must
preserve State exactly and must not advance an extra World tick. The supervisor
validates ordered ticks, per-tick record shape, lineage totals and population
balance, arm pairing, normalized initial State identity, and checkpoint restore.

The root-prebuilt executable, protocol registration, source/scenario hashes,
seed scan (including each verified compressed-JSONL index path/hash/size),
command, stdout/stderr, raw rows, manifest, and timestamps are
preserved under a new non-overwriting experiment directory. Evaluation requires
a clean committed source tree and a root-reviewed registration binding the
binary SHA-256. Limits: 300 seconds for the phase, 60 seconds and 1 GiB address
space per World, 256 MiB aggregate output, 10,000 identities, 20,000 events per
tick, and 16 MiB raw output per World. Any identity/hash mismatch, timeout,
output/event cap, incomplete tick stream, checkpoint mismatch, or invalid
lineage stops the phase; preserve partial evidence as incomplete. Biological
extinction, zero births, and candidate harm are outcomes, not infrastructure
failures.

## Decision boundary

This screen can describe the fixed fixtures, 6 paired seed blocks, exact source,
flags, and 300-tick horizon. It cannot establish general ecology, benefit in
unseen habitats, long-term survival, or qualification for live use. A signal
that the guard shortens food-search excursions while preserving access is
mechanism evidence; survival and descendant reproduction are separate physical
outcomes. A follow-up acceptance protocol would require a new held-out seed
allocation and explicit review.
