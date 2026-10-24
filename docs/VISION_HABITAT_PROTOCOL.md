# Vision expression across accepted and patched habitats

**Status: FROZEN BY ROOT.** Root approved this protocol and the reviewed driver for root-controlled execution. Local or direct World execution is not authorized by this status.

## Question and scope

Does the expressed-versus-neutral vision-allele contrast differ between the
accepted qualification habitat and one deterministic habitat with food
concentrated into eight spatial patches? The intervention changes only the
initial placement and potential renewable distribution of food. It is a
bounded habitat-opportunity comparison, not a favorable-result search. The
study does not predict which habitat or allele will have higher breeder
representation.

The directional prediction from the earlier vision study is retained as
context: vision-1 might gain representation if its extra range helps locate
food enough to repay its maintenance demand. The previous four-pair study
instead observed negative expressed-minus-neutral contrasts in the accepted
habitat. Neither result determines the outcome here. Report every arm and the
habitat interaction, including adverse, null, fixed, or non-estimable outcomes,
with exact counts. A positive contrast is not a gate for proceeding or
acceptance.

The planned primary endpoint for the conditional formal phase, if root
authorizes it after the registered engineering and forecast gates, is the
descriptive interaction at the registered endpoint:

`I = D_patched - D_accepted`, where `D_habitat = (p_breeder,expressed - p_founder,expressed) - (p_breeder,neutral - p_founder,neutral)`.

Each frequency uses vision-1 herbivore founders or living herbivore descendants
that themselves reproduced as numerator and all such members as denominator.
The calibration below is a seen-seed measurement/exposure check, not a formal
estimate. It reports its interaction exactly but does not establish a
replicable effect. Paired arms share seed and initial founder allocation, but
are not exact counterfactuals: behavior, feature cost, food placement, and RNG
paths can diverge after tick 0. Do not claim pure causal vision utility,
general fitness, or long-term habitat robustness.

## Frozen inputs and four-cell engineering calibration

Use only the archived accepted-profile scenario
`/root/curious/data/experiments/reserve-qualification/qualification-both/scenario.json`,
SHA-256 `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`.
Use the explicit accepted profile in `ECOLOGY_FIXTURE_V1`: mode `both`,
companions enabled, mate rendezvous and reserve opportunities enabled, water
residual disabled, water learning and updates enabled, hunt learning and
refuge outcomes disabled. Keep the V1 4x4 spatially stratified, globally
alternating herbivore assignment: 288 vision-0 and 288 vision-1 founders;
mobility/stamina 0 for all; all non-herbivore genomes default (including
carnivores vision 0). Mutation is zero. Both habitats and both feature settings
use exactly the same founders, assigned genomes, positions, bodies, policy,
seed, and normal AI/resolver execution. No founder or resource reset occurs
after tick 0.

Run exactly four fresh worlds on seen seed `123`, 200 ticks each, in this
registered order: accepted-neutral, accepted-expressed,
patched-neutral, patched-expressed. Use the same seed and founder assignment
for all four cells. Engineering is limited to measurement, exposure, and
resource-flow feasibility. Preserve results even if either species or the
primary denominator is absent. The conditional formal phase is fixed in this
protocol as four new paired seed blocks, `55001`, `55002`, `55003`, and `55004`,
confirmed unseen by root's scan of 1,281 text files (12.676 seconds), with no
matches. Inventory SHA-256 is
`5026aeb4c1499e58ff94627328f33a322e5d86ab39ef760dcdb673668b4cbff7`, retained
at `data/operations/wave8-checks-20261024/seed-inventory.json`. Root records an
immutable inventory copy/hash in the frozen run manifest before engineering
starts. Formal execution remains conditional on every engineering and forecast
gate below passing and root authorizing the registered phase.

Each formal block uses four fresh worlds, one per habitat × feature cell, with
500 ticks each (2,000 ticks per block; 8,000 ticks total across four blocks).
Use the same seed and founder assignment for all four cells of a block. Formal
cell order is the same as engineering: accepted-neutral,
accepted-expressed, patched-neutral, patched-expressed. No seed is replaced,
and formal roles execute once in registered order.

- **Neutral:** `extra_traits_enabled=false`; inherited vision alleles remain
  markers and all extra-trait requests must be zero.
- **Expressed:** `extra_traits_enabled=true`; verify actual vision range and
  requested cost by genome before tick 1 and every recorded state.
- **Accepted habitat:** scenario bytes and resources unchanged.
- **Patched habitat:** apply only the deterministic Food transformation below.

All other genetics, ecology, control, body, map-bound, founder-position,
water, burrow, and wall settings are unchanged. Capture full perceptions in
memory for actual decisions. Audit data never feeds the AI. Report missing
trace/perception coverage as unknown, not zero. Use the genotype and ecology
auditors; validate actual typed events and stock flow rather than deriving
realized resource use from the scenario's potential site totals.

## Exact deterministic food transformation

The accepted fixture has bounds `[-6, 48] × [-6, 48]`, 15,680 initial Food,
784 Food sites with total potential capacity 15,680 and total nominal
replenish 1,568 per tick (interval 1, no schedules), 5,760 initial Water, and
192 Water sites. There are no initial Food-site overlaps with founders, Water,
burrows, or walls. The transform preserves all those non-Food fields and
positions exactly.

1. Compute eight anchor coordinates in row-major order over inclusive bounds.
   For `i=0..3`, `x_i=min_x + floor((2*i+1)*width/8)`; for `j=0..1`,
   `y_j=min_y + floor((2*j+1)*height/4)`, where `width=max_x-min_x+1` and
   `height=max_y-min_y+1`. For this fixture anchors are `(0,7)`, `(14,7)`,
   `(28,7)`, `(42,7)`, `(0,35)`, `(14,35)`, `(28,35)`, `(42,35)`.
2. Start with the original Food-site positions sorted by no implicit order.
   For each anchor in row-major order, select 16 remaining positions by
   `(Manhattan distance to anchor, y, x)` ascending and remove them from the
   candidate set. These 128 existing Food positions form eight patches. For
   the accepted fixture each patch's positions fit within a 9-by-9 bounding
   box, and the eight groups are disjoint.
3. Order selected sites by patch order then `(y, x)`. Set the first 64
   capacities to 123 and the remaining 64 to 122 (sum 15,680). Set the first
   32 replenish values to 13 and the remaining 96 to 12 (sum 1,568). Keep
   each selected site at interval 1 and with no schedule. Set its initial
   Food-cell amount equal to its new capacity.
4. Set the other 656 original Food cells to Empty and remove their Food sites.
   Do not add or move sites to any other position. Do not alter any Water,
   Burrow, Wall, founder, or other map cell.

Pure transformation checks must verify source eligibility, unique positions,
all eight 16-site groups and exact anchors/order, no prohibited overlaps,
unchanged non-Food scenario fields, equal initial Food total, equal total
potential Food capacity and nominal replenish sum, and equal Water/site,
founder, wall, and burrow data. Capacity and replenish totals are potential
site properties only: local stock and consumption can clamp the realized
per-tick renewal. Record actual `ResourceReplenished` events and run the
per-tick food/water stock-flow audit in all four worlds. Do not treat equality
of potential totals as proof of equal realized flows.

## Resource opportunity measurement

For every herbivore decision trace with full perception, count actual recorded
food paths split into plant and corpse entries. `PerceivedFood.steps.len()` is
the path length in the recorded bounded BFS result; it is not a discovery,
successful trip, meal, or causal utility measure. Report counts with path
length `<=6` and `>6` separately for plant and corpse, plus whether at least
one plant food entry has recorded path length `<=6`. The threshold is the
base herbivore range 6 in both trait arms; retain all observed lengths and
report trace/perception coverage separately. Repeated perception entries are
repeated observations. Never call these discovery events.

Under the raw JSONL key `trace_food_path_histograms`, stream non-frame trace
records as the exact compact JSON array
`[id, perception_known, vision_range, plant_histogram, corpse_histogram]`.
Each histogram is a sorted array of `[steps_len, entry_count]` pairs; both
histograms are `null` when full perception is missing. Do not repeat genotype,
species, derived at/beyond-six totals, or boolean plant flags in these rows.
Join ID to species and full genotype from the retained founder/birth lineage
ledger; derive threshold counts and `any plant at <=6` from the histograms.
Retain missing trace IDs and missing perception IDs separately. At fixed
ticks, the raw key `food_path_samples` stores one object per herbivore trace as
`{"id":…, "perception_known":…, "food_paths":…}`. Each known food-path entry
is `{"is_corpse":…, "steps":[Direction,…]}`; a missing perception has a null
`food_paths` value. This preserves the actual individual ID, food kind, and
exact directions for independent spot checks. Also report t0 founder geometry
from the scenario without a World:
nearest selected-patch food-site Manhattan distance by assigned genotype and
counts of founders with a selected site within distances 6 and 7. Label this
as geometry, not bounded-path perception. Report initial/final Food and Water
stocks, all actual typed event totals, births/deaths/matings, living species,
lineage, exposure, trait requested/paid/unpaid costs, and missing coverage.

## Measurement gates and stopping

Before freeze, the new driver's pure tests must cover the exact food transform,
input rejection and conservation, geometry counts, trace bins at path lengths
6 and 7 for plant/corpse, repeated entries, and missing trace/perception as
unknown. These tests must not construct a World or tick it.

After root freezes both protocol and source, root may run the one four-world
engineering calibration once. Stop on source/profile/genotype mismatch,
incorrect founder allocation or expression, missing/ambiguous lineage joins,
observer error, event/payment mismatch, stock-flow residual, output/time/memory
cap, or infrastructure failure. Preserve partial evidence; do not retry,
replace, tune, or broaden the transform. Natural extinction, allele loss, no
breeder denominator, low or zero extra-range food paths, and a negative or null
interaction are biological/exposure observations, not reasons to retune or
repeat. No positive effect or benefit gate is permitted.

Before any formal seed, root also compares the accepted-neutral and
accepted-expressed tick-200 final State+RNG trees against the corresponding
archived Wave 6 seed-123 200-tick genotype-measurement endings. This is an
exact compatibility gate for the unchanged accepted controls and their
tick-100 restore path, not a new World run or an uninterrupted-trajectory
claim. The new exposure observers must remain read-only and must not alter
policy, state, or RNG. Any mismatch stops before formal work; do not waive or
explain it away after seeing results.

Before formal execution, every engineering cell must pass source/profile,
founder/genotype assignment and phenotype, child transmission, genotype cost
and acquisition/event reconciliation, lineage, missingness, and per-tick
resource-flow gates. Each engineering cell must also have both species alive
and at least one living herbivore descendant breeder at tick 200. The primary
endpoint is herbivore allele representation, so the engineering gate does not
require carnivore descendant breeders to have matured by that short horizon.
Earlier accepted-control evidence recorded no carnivore living descendant
breeders at 200 ticks in either arm, while the 500-tick vision calibration
recorded 14 per arm; report engineering carnivore births and descendant
breeders descriptively with this known horizon limitation. This study is not a
new coexistence qualification. There is no positive-effect gate and no
requirement that both herbivore alleles remain among living breeders; allele
loss is a valid observation. At the formal 500-tick endpoint, report both
species' living descendant breeders descriptively. Formal endpoints with
positive herbivore breeder denominators remain frequencies 0 or 1. A zero
herbivore descendant-breeder denominator is a biological non-estimable
endpoint, not an infrastructure failure or a replacement-seed reason; other
registered blocks continue after biological negatives.

Formal seed roles are the four fixed values above, contingent on root's
inventory verification before engineering. After any engineering output is
observed, no seed, transform, endpoint, threshold, or design change is allowed.
If a source/profile/audit/capture/infrastructure gate fails, stop the phase and
preserve partial evidence; do not retry. Root reviews the complete calibration,
event and flow audits, exposure by genotype, and capture-inclusive budget before
allowing the one formal batch.

## Capture and resource limits

Engineering limits: 300 seconds wall, 1 GiB address space, 256 MiB aggregate
output, with raw JSONL capped at 240 MiB to reserve at least 16 MiB for
metadata, stderr, and status. Formal limits: 1,200 seconds total across
sequential blocks, 1 GiB address space per worker, 2 GiB aggregate output for
all blocks, and 480 MiB raw per four-cell seed block. Use one Cargo job and the
root-designated target. Retain full State plus typed events at engineering
frames `t0`, `t1`, `t100`, `t200`; formal frames are `t0`, `t1`, `t100`, `t500`
(no formal t200 frame). Retain compact sufficient per-tick event, trace,
genotype, lineage, and resource-flow rows otherwise. Do not drop required frames
or evidence after seeing output. State clearly that movement/action details and
full perception are not retained away from fixed frames, and this is not a full
replay. Root supervisor must verify exact source, protocol, binary, scenario,
runner, inventory, and output identities, prepare a durable manifest before
spawn, and capture logs, status, and timestamps; direct driver invocation alone
is not a registered run.

Formal execution is permitted only if all forecasts pass from the single
engineering capture: per-block raw forecast is
`engineering_raw_bytes * (500 / 200) * 1.20` and must be at most 480 MiB;
aggregate raw forecast is four times that value, and after measured metadata,
manifests, stderr, audits, and status it must fit 2 GiB; batch wall forecast is
`4 * engineering_elapsed_seconds * (500 / 200) * 1.50`, plus separately
measured compile, runner, and audit overhead, and must fit 1,200 seconds. Fixed
frames are included in engineering capture; do not add or remove frames after
observing output. If any forecast or gate fails, do not instantiate a formal
seed and do not tune or retry.

The pure engineering-size preflight uses the archived Wave 6 seen-seed capture
as an intentionally conservative reference: 80,367,647 raw bytes and 258,470
trace rows over 400 ticks. For 800 engineering ticks, it scales the baseline
to 160,735,294 bytes and predicts 516,940 trace rows. The representative
compact trace array is 62 serialized bytes. Add the full projected compact
rows and a reserved 16 MiB for all fixed-frame food path samples to the scaled
baseline, then apply 10% headroom: 230,519,069 raw bytes (219.84 MiB), below
the 240 MiB raw cap. This deliberately counts new trace rows on top of a
baseline that already had older trace rows. The driver enforces the 16 MiB
fixed-frame path-sample budget and calculates this forecast before it
constructs a World. Root independently applies the measured calibration
forecast formulas above before formal execution.

## Interpretation

Report the earlier positive prediction and the prior accepted-habitat negative
result, but make no directional success gate. For all four engineering cells,
report exact founder/breeder counts, frequencies, costs, acquisitions,
path-length exposure, and resource flows. Report the descriptive calibration
interaction only with explicit seen-seed status. For formal blocks, report
each seed-level interaction, each habitat-arm numerator/denominator, and
descriptive mean, median, range, and `N_valid`; never pool individuals across
seeds or use significance testing. The paired seed block is the replication
unit. Neither engineering nor formal results are a pure causal vision effect,
proof of food discovery, general fitness, or habitat robustness. No new batch,
retuning, or transform change follows automatically.
