# Water–meal priority V1: preregistered paired factorial screen

**Status: candidate formula and factorial design approved; source integration and registration review pending.** This is a bounded descriptive experiment, not a utility qualification or deployment decision. Primary roadmap area: K2 decision/body boundary; supporting measures concern K5 resource access and K6 reproduction. The decision unit is the individual; survival and reproduction are population outcomes.

## Question and candidate

In the exact unique-food stress fixture, when the normal final policy selects a water intent (`SeekWater`, `LearnedWater`, or `RecallWater`) while positive water is visible and a positive, diet-eligible non-corpse plant meal is immediately reachable, does a strict energy-blocked water-vs-meal priority rule increase successful drinking and reduce matched futile water approaches without sacrificing food access or survival exposure?

The candidate flag is `water_meal_priority_enabled` (default off), with intervention reason `EatBeforeWater`. Per the approved formula it may act only on final normal-policy decisions with reason `SeekWater`, `LearnedWater`, or `RecallWater`, a matched visible positive-water target whose route has at least one movement step, adequate first-step speed/stamina, and pre-decision energy below one movement-energy cost. It changes the immediate intent to Eat only when positive, diet-eligible, non-corpse plant food is available at at most one step. It draws no RNG. A positive-payable water prefix and adjacent zero-movement-cost Drink remain unchanged; corpse food is ineligible. It interrupts the water learner before resolver handling. The transient `DecisionTrace.water_meal_override` audit field records the original selected water decision and matched visible resource amounts; the lab stores those values with the resulting actual `EatBeforeWater` decision.

## Arms, fixtures, seeds and horizon

| Arm | `food_search_reserve_enabled` | `water_meal_priority_enabled` |
|---|---:|---:|
| control | off | off |
| food_reserve_only | on | off |
| water_meal_only | off | on |
| bundle | on | on |

This 2×2 factorial design estimates each flag's isolated contrast and the bundle interaction. All four arms share initial body, world, scenario, seed and RNG state; normalized initial States may differ only in the two registered flags. The food reserve remains its prior V20 behavior.

| Fixture | Start | Seeds | Ticks per World |
|---|---|---:|---:|
| Unique-food stress | Prior V1 geometry; one AI herbivore at `(0,0)`, max age and maturity age 2000, food `(0,-1)`, water at `(-2,0)` and `(4,0)`; learner enabled, updates enabled, context TTL enabled | 62501–62506 | 300 |
| Meadow | Read-only `scenarios/meadow.json` | 62501–62506 | 300 |
| Sparse-resource | Read-only `scenarios/ecology-traits-lab.json` | 62501–62506 | 300 |

Genetics is enabled with mutation 0 per mille and extra traits off. Other opt-in policies are explicitly off. Total: 3 fixtures × 6 seeds × 4 arms = 72 Worlds and 21,600 World ticks. The same six seed identities recur across the three fixtures, so the design has 18 paired fixture blocks, not 18 independent seed identities. Every World runs through tick 300, including extinction. No rescue, reset, resource addition, seed replacement, retry, tuning or intervention is allowed. Prior seeds 62401–62406 are not acceptance evidence; 62404 remains diagnostic context only.

Before registration scan the experiment archive's recognized text inventory with a 64 MiB maximum line bound. For each `*.jsonl.zst`, verify its sibling archive index status, size and SHA-256, and preserve every index's declared seed roles. The scanner is source-pinned. Seeds 62501–62506 must be unseen. SQLite payloads are not decoded; use their retained manifests/directory metadata for declared seed roles and state this coverage limit.

## Outcomes and raw evidence

Report each World and paired seed contrast separately. Primary physical measures are successful resolver-confirmed drink amount, plant-food energy gained excluding corpse meals, movement energy actually paid, living individual-ticks, zero-energy living ticks, deaths and cause, endpoint population, births by generation, F1/F2 breeders and living descendant breeders. Keep food, water, movement, metabolism and other costs separate; do not use reward as success.

For every tick retain compact all-actor `EntityAte`, `EntityDrank`, and `EntityMoved` resolver events by actor and physical amount/cost, along with population/resource/body totals. Preserve the focal founder's full decision trace and before/after learner snapshots (episode, last result, visits/value table, residual table, context entries and snapshot hash) from ordinary World state; do not alter learner state for measurement. Record actual `EatBeforeWater` decisions with pre-choice body state and matched visible water/food amounts. `DecisionTrace.water_meal_override` must retain the original selected water decision; do not reconstruct an unrecorded counterfactual target.

Count matched futile water approaches only where the focal trace selected a Drink whose target matches visible positive water and the resolver records `Approach/OutOfRange`. Separately report the matched energy-block diagnostic when the paid movement is below `(route_steps - 1) × movement_energy_cost`, with zero-paid and partial-paid counts; this is diagnostic, not exclusive causal attribution. Count zero-energy and underfunded adjacent-food approach failures using the prior food-lab definition, independently from futile water approaches. Preserve every unresolved rejection as unattributed.

Record existing learner episode and last-result transitions as observed state, without synthesizing closure events, changing learner logic, or claiming unique closure counts unsupported by source events. A learner with context TTL disabled or no qualifying water candidate may have no active episode; a replaced base `SeekWater` choice alone is not evidence that an episode was interrupted. Attribute an interruption only when the before/after learner state or a normal learner result supports it. TTL pruning during the normal choice can change context tables; do not require unchanged table hashes. The bounded single `last_episode` slot can overwrite earlier results, so it does not support unique closure totals. An episode still open at tick 300 is right-censored at the horizon for analysis only; do not write a synthetic world event. Death or naturally observed learner closure remains as recorded. Missing start/end or uncertain identity means unknown. Time from first zero-energy observation to death is null/unknown if either endpoint is absent; never turn right-censoring into a measured death interval.

Retain all tick rows, per-actor resolver-event records, full focal traces, lineage records with parent IDs/generations, and initial/final checkpoint State. Validate event totals against resolver events, lineage/population balance, arm pairing, ordered ticks, exact endpoints, and restore equality without an extra World tick. Numeric counts must remain numeric; identity-list metrics are separately named counts. Report per-seed vectors, medians and ranges with support denominators, missing/censored values, negative outcomes and any zero-exposure fixtures. No composite pass score or post-result threshold is defined.

## Execution limits and interpretation

Use a clean committed source tree and root-reviewed registration binding source, scenarios, protocol and prebuilt binary hashes. Preserve command, stdout/stderr, raw records, manifest and timestamps under a new non-overwriting directory. Limits: 300 seconds per phase, 60 seconds and 1 GiB address space per World, 256 MiB aggregate output, 16 MiB per World, 10,000 identities and 20,000 events per tick. Identity/hash mismatch, timeout, cap breach, incomplete stream, failed checkpoint restore or invalid lineage stops the phase; partial evidence stays marked incomplete. Extinction, no births, no intervention and candidate harm are outcomes, not infrastructure failures. The result can support only these fixtures, six held-out seed identities and 300-tick horizon; it cannot establish general ecology, lifetime survival, long-term reproduction or live qualification.
