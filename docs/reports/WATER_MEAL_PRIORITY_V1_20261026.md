# Water–meal priority V1: registered factorial screen

**Status: complete descriptive screen; no utility qualification or live acceptance.** The candidate remains default-off; live V12 was unchanged.

## Run and scope

The root-reviewed registration completed 72 Worlds × 300 ticks (21,600 scheduled ticks) in 61.404 seconds. Source revision `edb76316d7002f0533d4c038b664a796ec293c41` was clean at launch; source content SHA-256 `d55d6b1d4fb27ec1abc87d088a8d7616c1dad96a442c7144e9cc3a4f6a075d5a`; binary SHA-256 `81d6044ca3f838234643d072e23d07ce4856f70953d91c0642e783883e1f28e0`; protocol SHA-256 `2b602b2dc294b49585dc2235c669af80d598479090774fce9390ee92bd0a6e4b`. The [JSON companion](WATER_MEAL_PRIORITY_V1_20261026.json) records fixture hashes and every raw-file SHA-256 and byte count.

The factorial screen used control, food-reserve-only, water-meal-only, and bundle arms in unique-food stress, meadow, and sparse-resource fixtures, with seeds 62501–62506. The same six seed identities recur across fixtures: 18 paired fixture blocks, not 18 independent seed identities. The prior runner attempt at `6f033f6` stopped at `shutil.copy2` with a `NameError` before launching any World (0 Worlds / 0 ticks). Its failed artifacts remain unchanged; see the [runner addendum](../WATER_MEAL_PRIORITY_V1_RUNNER_ADDENDUM_20261026.md).

## Aggregate outcomes

Values sum six Worlds per arm. Plant-food energy is gross resolver intake, separate from water and movement energy; it is not net energy.

| Fixture | Water-meal-only − control: living ticks / food / water / movement / endpoint live | Bundle − food-reserve-only: living ticks / food / water / movement / endpoint live |
|---|---|---|
| Unique-food | 0 / 0 / 0 / 0 / 0 | **+97 / +168 / +160 / +11 / 0** |
| Meadow | **+214 / +1,036 / +172 / +611 / +3** | **+214 / +1,036 / +172 / +611 / +3** |
| Sparse-resource | **−6 / +2 / −30 / +8 / 0** | **−6 / +2 / −30 / +8 / 0** |

Unique-food water-meal-only had zero interventions and matched control. Bundle had 10 interventions and outperformed food-reserve-only on those aggregate measures, but only two of six Worlds were alive at tick 300 in both arms. Those survivors are right-censored at the horizon, not durable survival evidence; the other four Worlds in each arm died.

Meadow had 73 interventions per water-policy-on arm. Aggregate births were F1/F2 50/32 in control and food-reserve-only, versus 49/35 with water-meal priority; final population totals were 89 versus 92. This aggregate gain was heterogeneous. Sparse-resource had 47 interventions per water-policy-on arm, but all six Worlds were extinct at tick 300 in every arm.

### Per-seed survival exposure

Vectors follow seed order 62501–62506. `W−C` is water-meal-only minus control; `B−F` is bundle minus food-reserve-only. Complete per-seed physical vectors for plant food, water, movement and diagnostics are in JSON.

| Fixture / contrast | Living individual-tick deltas | Endpoint-alive deltas |
|---|---|---|
| Unique-food / W−C | [0, 0, 0, 0, 0, 0] | [0, 0, 0, 0, 0, 0] |
| Unique-food / B−F | [0, 33, 0, 31, 0, 33] | [0, 0, 0, 0, 0, 0] |
| Meadow / W−C | [266, −106, −173, 268, −167, 126] | [−2, 1, 1, −1, 1, 3] |
| Meadow / B−F | [266, −106, −173, 268, −167, 126] | [−2, 1, 1, −1, 1, 3] |
| Sparse-resource / W−C | [0, 2, −8, 0, −1, 1] | [0, 0, 0, 0, 0, 0] |
| Sparse-resource / B−F | [0, 2, −8, 0, −1, 1] | [0, 0, 0, 0, 0, 0] |

## Intervention attribution and integrity

There were 130 unique `(fixture, seed, tick, actor)` intervention decisions: 10 in unique-food (bundle only), 73 in meadow, and 47 in sparse-resource. Meadow and sparse-resource decisions appear in both water-policy-on arms, yielding 250 raw intervention rows; all passed the runner’s visible-water/actual-plant-meal validator.

A focal witness in `v1_unique_food-seed62502-bundle.jsonl` at tick 230 had energy 0, move cost 1, visible water amount 12 on route `Right,Right,DownRight`, and adjacent plant food amount 2. `EatBeforeWater` resolved a 2-energy meal and no same-tick drink. The active water episode closed as `Interrupted` with `reward=null`, water 0; focal learner tables were unchanged. The actor later moved one paid step and drank 10 at tick 233. This verifies that witness only: learner snapshots are focal-only, and the learner’s single `last_episode` slot does not support a unique population-wide closure count.

All 72 raw hashes and sizes matched the manifest; each stream had 300 ordered ticks, stderr was empty, and all-actor resolver totals reconciled to plant-food, water, and movement totals. Runner validation passed checkpoint restore and lineage/population balance for all Worlds (zero lineage issues and unknown-generation births). Raw evidence remains local/ignored at `data/experiments/water-meal-priority-v1-runnerfix-20261026/`.

The screen is limited to these fixtures, six held-out seed identities, and 300 ticks. The new guard alone had no unique-food exposure; meadow showed seed-level losses and higher movement cost; sparse-resource still ended in extinction. These results do not establish general ecological benefit, durable survival, or Pareto dominance; no utility qualification or deployment was accepted. The frozen design remains in the [V1 protocol](../WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md); the prior zero-tick runner error is a separate infrastructure failure, not a biological result.

## Validation and operating handoff

All fixtures contained herbivores; carnivore/omnivore behavior and long-horizon mixed coexistence were not evaluated. The registered `focal_underfunded_water_approaches` field counts **partial paid water approaches** (`0 < paid < full approach cost`). Speed, stamina or other constraints can contribute, so that field is not proof of insufficient energy. The strict intervention gate (`energy < one movement cost`, otherwise available first step) is verified separately from the recorded original decision and body.

Root verification: 431 Rust tests passed with zero failures and one pre-existing long-soak ignored; Python discovery passed 214 tests, then the latest runner module passed all 9 tests (overlapping checks, not 223 distinct tests). Release CLI/example compilation and the isolated HTTP/auth/audit/writer-lock/SIGKILL recovery check passed. Initial test-compilation/JSON-macro issues were corrected before registration; the runner-copy failure was separately preserved as a zero-tick attempt. No browser UI code changed. Phase wall time includes inventory and supervision; 21,600 is the registered lab workload, separate from regression-test simulations.

The independent audit at local `data/operations/water-meal-20261026/independent-lab-audit.json` checked 73 source-file hashes against `git show edb7631`, the source digest, scenario/protocol/binary hashes, all 72 raw files and exact output-size accounting. The completed directory contains 92,376,296 bytes; its manifest reports that same final size. Raw data, source tars and executed CLI/lab binaries remain locally preserved, not uploaded as Git data.

After no Cargo/rustc/task process remained, only `/tmp/curious-water-meal-main-20261026` was removed (711,962,863 nominal file-size bytes; not a free-block measurement). The read-only after-check recorded live V12 tick14927, 588 alive, `paused=false`, `storage_error=null`, unchanged coexistence/replay PIDs119175/1137, both active with NRestarts=0. The qualified binary hash remained `35930838981679ab1b937324df6dda73d2b3bcc6c57706d69f2c11d2803959af`. Logs, timestamps and cleanup proof are under local `data/operations/water-meal-20261026/`. No live pause, acceleration, restart or deployment was performed.
