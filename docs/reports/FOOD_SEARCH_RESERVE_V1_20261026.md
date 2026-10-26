# Food search reserve guard V1 — 300-tick paired screen

**Status: complete descriptive screen. No general ecology qualification or live acceptance.** The registered question was whether a default-off energy cap on discretionary `SearchFood` movement reduces the unique-food access failure mode instead of merely delaying starvation. The result is limited to these fixtures, seeds, and 300-tick horizon.

## Run identity and evidence checks

Registration `food-search-reserve-v1-20261026` was `registered` and `root_reviewed=true`. The run used source revision `786ad0455743fe5a23ae8126271a92cbbef71383`, clean at launch, source content SHA-256 `6a79ab79f6cae4486a7cf16e646041c79d09b343e9b80f59dab8e21fe0b5034b`, and binary SHA-256 `99c301444e46e89571cd6ebe230d2d31e9e2d313d111bf5d591ce954f21b8656`. The registered protocol SHA-256 is `fb4db4bde28f9af5e1b47ec1e58940e80d50604b123c5100298014237948c61c`; fixture hashes and per-raw SHA-256/byte counts are in the [JSON companion](FOOD_SEARCH_RESERVE_V1_20261026.json).

The run completed 36 Worlds (three fixtures × six seed identities × two arms), 300 ticks each, or 10,800 World ticks, in 50.056 seconds. The manifest recorded aggregate output as 27,528,180 bytes before its final rewrite; the completed run directory is 29,704,573 bytes, 2,176,393 bytes larger after that rewrite. Seed inventory checked 1,496 text files and 34 verified compressed-JSONL indexes (194,639,942 compressed bytes hashed); seeds 62401–62406 were unseen. Those same six seed identities were reused across the three fixtures, so the design has 18 paired fixture comparisons, not 18 independent seed identities.

Read-only validation confirmed all 36 raw hashes and byte counts against the manifest, 300 ordered rows per World, empty stderr files, endpoint checkpoint restore for all Worlds, and population balance for all Worlds. The lineage stream reconciles 484 birth records with 484 mating records and includes 514 deaths; there are zero lineage issues and zero unknown-generation births. The 18 paired initial States match after normalizing only the reserve flag. Meadow and sparse-resource tick rows match exactly in all six paired seeds per fixture; the candidate caused no reserve interventions in either fixture. Raw output and the full manifest remain under local, ignored `data/experiments/food-search-reserve-v1-20261026/`.

## Aggregate outcomes

Plant food energy is gross resolver intake; movement energy is the resolver's paid movement cost. Neither is net energy, and metabolism, water use, and other costs remain separate.

| Fixture | Candidate reserve interventions | Living individual-ticks, baseline/candidate | Plant-food energy, baseline/candidate | Movement energy, baseline/candidate | F1/F2 births, baseline/candidate | Endpoint live, baseline/candidate |
|---|---:|---:|---:|---:|---:|---:|
| V1 unique-food stress | 111 | 624 / 812 | 566 / 794 | 594 / 592 | 0/0 / 0/0 | 0 / 0 |
| Meadow | 0 | 24,287 / 24,287 | 48,274 / 48,274 | 16,435 / 16,435 | 48/35 / 48/35 | 87 / 87 |
| Sparse-resource | 0 | 5,959 / 5,959 | 4,908 / 4,908 | 2,988 / 2,988 | 17/0 / 17/0 | 0 / 0 |

All six V1 focal individuals died by tick 199 in the candidate arm and by tick 123 in baseline. Candidate minus baseline living individual-ticks were `[0, 0, +25, +93, +46, +24]` (median +24.5, range 0–93); gross plant-food energy deltas were `[0, 0, +34, +102, +58, +34]` (median +34, range 0–102). Movement-energy deltas were `[0, −8, +1, +1, +1, +3]`, totaling −2. The focal zero-energy, visible-food approach-failure diagnostic fell from 75 events to 17; the stricter underfunded partial-movement diagnostic was zero in both arms for this fixture. These counts are budget-deficit diagnostics, not exclusive causal attribution.

### V1 unique-food stress, per seed

| Seed | Living individual-ticks B/C | Plant-food energy B/C | Movement energy B/C | Zero-energy exposure B/C | Zero-energy visible-food failures B/C | Candidate cap calls | Focal death tick/cause B/C |
|---:|---:|---:|---:|---:|---:|---:|---|
| 62401 | 92 / 92 | 80 / 80 | 97 / 97 | 30 / 30 | 17 / 17 | 1 | 93 Dehydration / 93 Dehydration |
| 62402 | 98 / 98 | 100 / 100 | 108 / 100 | 27 / 19 | 0 / 0 | 9 | 99 Dehydration / 99 Dehydration |
| 62403 | 103 / 128 | 92 / 126 | 98 / 99 | 30 / 22 | 14 / 0 | 16 | 104 Dehydration / 129 Starvation |
| 62404 | 105 / 198 | 90 / 192 | 94 / 95 | 30 / 22 | 12 / 0 | 44 | 106 Starvation / 199 Starvation |
| 62405 | 122 / 168 | 108 / 166 | 97 / 98 | 32 / 21 | 18 / 0 | 25 | 123 Starvation / 169 Starvation |
| 62406 | 104 / 128 | 96 / 130 | 100 / 103 | 29 / 22 | 14 / 0 | 16 | 105 Starvation / 129 Starvation |

In four seed pairs (62403–62406), a cap intervention was followed by an observed plant meal within one to two ticks, with higher total plant intake and more living individual-ticks than the paired baseline. In seed 62402, a plant meal followed the first cap at tick 62, but total intake and survival exposure were unchanged. In seed 62401, no plant meal followed its sole cap; food intake, movement cost, survival exposure, and death time were unchanged.

### Trace witnesses and remaining failure

- In seed 62401 at tick 61, energy was 5, water 19, and stamina 2. Baseline selected a three-step search; the cap shortened it to two steps. Resolver stamina also limited baseline to two actual steps, so both arms paid 2 movement energy and reached the same position. The cap call itself produced no physical benefit in that instance.
- In seed 62404 at tick 75, the candidate had energy 2 and water 15. The cap was zero and the candidate idled; baseline spent 2 energy on its search move. Both arms ate 2 plant energy at tick 76. At tick 78, the candidate ate another 2; baseline selected `Eat Down,Down` and received `Approach/OutOfRange`. This was the first candidate-only meal in that pair.
- Seed 62404 still reached a different energy lock. At tick 89, with energy 2 and water 1, its unchanged `LearnedWater` decision attempted a three-step water approach, paid 2 movement energy, and drank 10 water. From ticks 191–198 it had energy and water at zero, repeatedly chose the water approach while visible plant food was adjacent, and received `Approach/OutOfRange`; starvation death was recorded at tick 199. The guard only caps discretionary `SearchFood` movement, so it does not change water-seeking decisions. This later episode is visible in the raw decision trace and is outside the guard's action scope.

The two seed pairs without survival or plant-intake improvement, plus eventual death in all six candidate stress runs, show that the guard did not solve the full lifetime food/energy problem. The four improved pairs did include additional resolver-confirmed meals, so their gains were not only a later death tick. Water depletion and other causes remain part of the observed stress; the final death label does not assign all preceding health loss to one cause.

## Other fixtures and reproduction

The meadow arms were physically identical across all six seeds: 24,287 living individual-ticks, 48 F1 births, 35 F2 births, and 87 endpoint live individuals per arm in total. Across those runs, 43 F1 identities and 28 F2 identities bred; the per-seed living descendant-breeder and generation counts are retained in JSON. No candidate cap activated, so this is a no-exposure comparison rather than transfer evidence.

All twelve sparse-resource Worlds ended with zero live individuals. Each arm recorded 17 F1 births, no F2 births, and no descendant breeders. The candidate activated no cap, and every paired tick row was identical. This confirms the fixture's sparse-resource collapse under the registered settings; it does not demonstrate a reserve-guard effect.

## Interpretation boundary

This screen gives descriptive evidence that limiting discretionary search movement can preserve energy for additional food acquisition in four of six V1 stress seed pairs. It also identifies where the default-off guard does not apply: a later water-seeking route can still consume the remaining movement budget while plant food is visible. It does not establish universal ecological benefit, survival beyond the observed horizon, or a live-use qualification. The candidate remains default-off; any broader claim needs a new held-out protocol with explicit food access, water stress, lifetime survival, and reproduction outcomes.

Related context: [energy and package diagnosis](ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md), the [frozen V1 protocol](../FOOD_SEARCH_RESERVE_GUARD_V1_PROTOCOL_20261026.md), and the [current behavior contract](../FOOD_SEARCH_RESERVE.md). Machine-readable per-seed vectors, paired differences, trace witnesses, archive audit summary, and raw hashes are in [FOOD_SEARCH_RESERVE_V1_20261026.json](FOOD_SEARCH_RESERVE_V1_20261026.json).

## Implementation and verification handoff

The candidate is a V20 decision-policy constraint, not machine learning. It stays default-off; pre-V20 checkpoints restore it off, V20 retains an explicit opt-in, and unsupported V21 is rejected. SQLite schema remains V2. All empirical Worlds in this screen contain herbivores only: carnivore/omnivore effects, long-horizon mixed coexistence, and interactions with other opt-in features were not evaluated. Runtime is phase wall time including inventory/supervision, not a tick-throughput benchmark.

Root ran `cargo test --all-targets --locked --jobs 1`: 421 passed, zero failed, one pre-existing long-soak test ignored. The separately added two-tick physical integration test passed (1 test). Python discovery passed 207 tests; after the last runner edits, its six tests passed again (not six additional unique tests). Release CLI and lab builds passed. `scripts/check_control.py` passed HTTP/auth/audit/writer-lock and SIGKILL recovery checks against a private temporary directory/port. Regression-test simulation work is separate from the registered 10,800 lab ticks. Relative links/JSON and `git diff --check` were checked; no browser UI change was made.

Independent audit: local `data/operations/food-reserve-20261026/independent-lab-audit.json` checks 72 registered source hashes against `git show 786ad04`, the frozen binary, all 36 raw hashes, 18 initial pairs, 111 cap records, focal diagnostics and lineage. V1 is one individual, so food/movement event totals can be reconciled fully for that fixture. Population fixtures retain all-population tick aggregates and lineage plus full focal trace; event-level verification for other actors is not claimed.

After all Cargo/rustc/lab processes had exited, only `/tmp/curious-food-reserve-20261026` was removed. Its file-size sum was 705,508,752 bytes (nominal file sizes, not a filesystem free-block measurement). Registered CLI/lab binaries, the source tar, raw files and checkpoints were preserved. Cleanup, build/test logs and runtime observations remain locally under `data/operations/food-reserve-20261026/`; they are not uploaded Git data.

The read-only after-check at the time stored in `live-after.json` saw live V12 tick 14255, 577 living individuals, `paused=false`, and `storage_error=null`. Coexistence PID119175 and replay PID1137 were unchanged, both active with NRestarts=0; qualified binary SHA-256 remained `35930838981679ab1b937324df6dda73d2b3bcc6c57706d69f2c11d2803959af`. No live deployment, pause, speed change, data migration or restart was performed.
