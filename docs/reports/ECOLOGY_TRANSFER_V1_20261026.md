# Ecology transfer V1 — 600-tick paired feature screen

**Status: complete descriptive screen. No ecological utility or live-upgrade acceptance.** The exact registered protocol remains [ECOLOGY_TRANSFER_V1_PROTOCOL_20261026](../ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md); the scanner-cap correction is recorded separately in [ECOLOGY_TRANSFER_V1_SCAN_ADDENDUM_20261026](../ECOLOGY_TRANSFER_V1_SCAN_ADDENDUM_20261026.md). The protocol was not edited after registration.

## Run and evidence checks

Run identity: `ecology-transfer-v1-scanfix-20261026`; source revision `bc60fa60ac060b7cc0dfae75f20fe16df5173461`, clean at launch; release binary SHA-256 `826686da814665dfc94d7994a3e34cc3936092f7a05221338764a9254a00afc5`. All 70 source-file hashes from the manifest match the current checkout. The run completed all 45 registered worlds (3 existing habitats × 3 seed blocks × 5 arms), 600 ticks each, 27,000 World ticks total, in 69.387 seconds. The registered phase ceiling was 300 seconds; the runner recorded 12,566,379 aggregate output bytes before its final manifest rewrite (which adds 69 bytes; final directory total 12,566,448 bytes).

Before launch, the seed inventory scanner searched 1,444 text files and found all nine registered `623xx` seeds unseen. The earlier scanner-size preflight rejection remains a separate failed attempt; it produced no World outcome. This completed run used a distinct `scanfix` identity.

Raw evidence and `manifest.json` are in local ignored directory `data/experiments/ecology-transfer-v1-scanfix-20261026/`. Manifest SHA-256: `ddcf5c4f3c107fdf1d71be09bdb3cf3d309659690157b4fe94d191d82080bfd3`. Scenario SHA-256 values are in the JSON companion. The registration and manifest identify the frozen arms and seeds.

Independent read-only checks confirmed all 45 raw file SHA-256/byte counts, 600 ordered tick rows per world, and 45 endpoint checkpoint restore checks. The tick-level lineage stream contains 4,036 birth records paired one-for-one with 4,036 mating records, plus 4,120 death records. Species and generation counts reconcile; all 45 population balances satisfy initial founders + births − deaths = endpoint live. There are zero lineage issues and zero unknown-generation births; all 45 stderr files are empty. Across the 45 worlds the balances total 810 initial founders, 4,036 births, 4,120 deaths, and 726 endpoint live individuals.

## Per-habitat results

Each absolute value below is the median across the three registered seeds, followed by the observed seed range in brackets. Paired deltas are computed per seed against that habitat’s same-seed `control`; their medians and ranges describe only these three pairs. The JSON companion preserves every seed value and the full numeric delta vector, including resource stocks, stress counts, species-specific counts, and feature diagnostics.

### Meadow — resource-rich herbivore fixture

| Arm | Final live H/C | Births H/C | Deaths H/C | Living individual-ticks | F1 births | F1 breeders | Living F1 breeders | F2 births | Living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Control | 13/0 [H 12–13; C 0–0] | 76/0 [H 76–77; C 0–0] | 71/0 [H 71–73; C 0–0] | 8076 [8064–8096] | 8 [7–8] | 6 [6–6] | 0 [0–0] | 5 [4–5] | 9 [8–10] |
| Climate only | 12/0 [H 10–12; C 0–0] | 62/0 [H 59–64; C 0–0] | 58/0 [H 57–60; C 0–0] | 6641 [6454–6803] | 7 [7–8] | 5 [4–6] | 0 [0–0] | 4 [3–5] | 8 [7–8] |
| Ambush only | 13/0 [H 12–14; C 0–0] | 77/0 [H 76–77; C 0–0] | 72/0 [H 70–73; C 0–0] | 8121 [8110–8246] | 8 [7–8] | 7 [6–8] | 0 [0–0] | 6 [5–7] | 9 [8–11] |
| Alarm only | 13/0 [H 12–13; C 0–0] | 76/0 [H 76–77; C 0–0] | 71/0 [H 71–73; C 0–0] | 8076 [8064–8096] | 8 [7–8] | 6 [6–6] | 0 [0–0] | 5 [4–5] | 9 [8–10] |
| Extra traits only | 12/0 [H 12–13; C 0–0] | 73/0 [H 62–75; C 0–0] | 69/0 [H 57–71; C 0–0] | 7833 [6598–7958] | 7 [6–7] | 4 [4–5] | 0 [0–0] | 4 [2–6] | 8 [7–8] |

| Treatment vs control | Δ final H/C | Δ births H/C | Δ deaths H/C | Δ living individual-ticks | Δ F1 breeders | Δ F2 births | Δ living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|
| Climate only | -1/0 [H -2–-1; C 0–0] | -14/0 [H -18–-12; C 0–0] | -13/0 [H -16–-11; C 0–0] | -1423 [-1622–-1293] | -1 [-2–0] | 0 [-2–0] | -1 [-2–-1] |
| Ambush only | 1/0 [H -1–1; C 0–0] | 0/0 [H 0–1; C 0–0] | -1/0 [H -1–2; C 0–0] | 34 [25–182] | 1 [0–2] | 1 [1–2] | 0 [0–1] |
| Alarm only | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Extra traits only | 0/0 [H -1–0; C 0–0] | -3/0 [H -14–-2; C 0–0] | -2/0 [H -14–-2; C 0–0] | -231 [-1498–-118] | -2 [-2–-1] | -1 [-2–1] | -1 [-2–-1] |

### Sparse-resource — ecology-traits-lab fixture

| Arm | Final live H/C | Births H/C | Deaths H/C | Living individual-ticks | F1 births | F1 breeders | Living F1 breeders | F2 births | Living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Control | 0/0 [H 0–0; C 0–0] | 3/0 [H 3–3; C 0–0] | 11/0 [H 11–11; C 0–0] | 988 [967–1028] | 3 [3–3] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Climate only | 0/0 [H 0–0; C 0–0] | 3/0 [H 3–3; C 0–0] | 11/0 [H 11–11; C 0–0] | 891 [886–893] | 3 [3–3] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Ambush only | 0/0 [H 0–0; C 0–0] | 3/0 [H 3–3; C 0–0] | 11/0 [H 11–11; C 0–0] | 988 [942–1028] | 3 [3–3] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Alarm only | 0/0 [H 0–0; C 0–0] | 3/0 [H 3–3; C 0–0] | 11/0 [H 11–11; C 0–0] | 988 [967–1028] | 3 [3–3] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Extra traits only | 0/0 [H 0–0; C 0–0] | 2/0 [H 1–2; C 0–0] | 10/0 [H 9–10; C 0–0] | 780 [701–843] | 2 [1–2] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |

| Treatment vs control | Δ final H/C | Δ births H/C | Δ deaths H/C | Δ living individual-ticks | Δ F1 breeders | Δ F2 births | Δ living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|
| Climate only | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | -97 [-135–-81] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Ambush only | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0 [-25–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Alarm only | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0/0 [H 0–0; C 0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| Extra traits only | 0/0 [H 0–0; C 0–0] | -1/0 [H -2–-1; C 0–0] | -1/0 [H -2–-1; C 0–0] | -248 [-266–-145] | 0 [0–0] | 0 [0–0] | 0 [0–0] |

### Mixed refuge — 36 herbivores and 2 carnivores at tick 0

| Arm | Final live H/C | Births H/C | Deaths H/C | Living individual-ticks | F1 births | F1 breeders | Living F1 breeders | F2 births | Living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Control | 41/0 [H 37–43; C 0–0] | 221/1 [H 211–226; C 0–1] | 216/3 [H 210–219; C 2–3] | 24365 [23630–24707] | 25 [24–25] | 16 [14–17] | 0 [0–0] | 12 [10–13] | 29 [23–29] |
| Climate only | 37/0 [H 36–37; C 0–0] | 177/0 [H 168–180; C 0–1] | 176/2 [H 168–179; C 2–3] | 19798 [18909–20182] | 24 [24–24] | 13 [11–16] | 0 [0–0] | 9 [9–10] | 24 [22–25] |
| Ambush only | 39/0 [H 37–43; C 0–0] | 224/0 [H 217–225; C 0–1] | 217/2 [H 216–222; C 2–3] | 24570 [24491–24791] | 26 [25–29] | 16 [15–17] | 0 [0–0] | 10 [9–12] | 25 [25–30] |
| Alarm only | 36/0 [H 0–42; C 0–0] | 152/3 [H 38–225; C 0–5] | 152/5 [H 74–219; C 2–7] | 17390 [7087–24741] | 22 [22–24] | 13 [11–16] | 0 [0–0] | 9 [9–13] | 21 [0–26] |
| Extra traits only | 38/0 [H 36–38; C 0–0] | 212/0 [H 209–215; C 0–0] | 210/2 [H 209–213; C 2–2] | 23255 [23235–23657] | 23 [21–23] | 9 [7–9] | 0 [0–0] | 6 [5–7] | 24 [23–26] |

| Treatment vs control | Δ final H/C | Δ births H/C | Δ deaths H/C | Δ living individual-ticks | Δ F1 breeders | Δ F2 births | Δ living descendant breeders |
|---|---:|---:|---:|---:|---:|---:|---:|
| Climate only | -5/0 [H -6–0; C 0–0] | -46/0 [H -53–-34; C -1–0] | -40/0 [H -48–-34; C -1–0] | -4909 [-5456–-3448] | -3 [-4–0] | -3 [-3–-1] | -4 [-7–1] |
| Ambush only | -4/0 [H -4–6; C 0–0] | -1/-1 [H -4–13; C -1–1] | 3/-1 [H 0–7; C -1–1] | 126 [84–940] | 0 [0–1] | -1 [-3–0] | -4 [-4–7] |
| Alarm only | -1/0 [H -43–1; C 0–0] | -59/2 [H -188–4; C 0–4] | -58/2 [H -145–3; C 0–4] | -6240 [-17620–376] | -3 [-3–-1] | -1 [-4–1] | -3 [-29–-2] |
| Extra traits only | -3/0 [H -7–1; C 0–0] | -9/-1 [H -17–4; C -1–0] | -6/-1 [H -10–3; C -1–0] | -1110 [-1472–27] | -8 [-9–-5] | -6 [-8–-3] | -3 [-6–1] |

## What the contrasts show

- **Meadow:** climate reduced herbivore births in all three paired seeds (median Δ −14, range −18 to −12), and living individual-ticks in all three (median Δ −1,423, range −1,622 to −1,293). Endpoint herbivores were median 12 versus 13 for control. Alarm-only emitted no signals and matched control across the recorded ecological outcomes. Ambush-only emitted no attacks, yet its trajectories and some breeding counts differed; this is explained by a visibility side effect described below and is not evidence of predation benefit. Extra traits were a joint vision/stamina expression package: median living individual-ticks were 231 lower than control, with a wide paired range (−1,498 to −118).
- **Sparse-resource fixture:** all 15 runs (five arms × three seeds) ended with zero living individuals. Control and three feature arms produced 3 herbivore births per seed but no F2 births or descendant breeders; extra-trait runs produced 1–2 births per seed and no F2 births. Climate reduced living individual-ticks by a median 97 (range −135 to −81), but extinction at tick 600 bounds interpretation. Lower population here is a stress outcome, not a standalone score.
- **Mixed refuge:** no arm preserved a carnivore at tick 600; all founders were dead by the endpoint in every habitat/arm/seed block. Control produced median 221 herbivore births and 12 F2 births, with 24,365 living individual-ticks. Climate had paired median changes of −46 herbivore births, −4,909 living individual-ticks, and −3 F2 births. Alarm-only had the widest spread: seed 62323 ended with both species extinct, while the other two seeds retained herbivores. It emitted 2,635–4,325 sound signals across its three seeds; reception remains unknown. The run does not establish mixed-species coexistence.

**Ambush scope caveat.** In the frozen source, `src/systems/perception.rs` applies ambush-cover concealment to any fully rested living entity in cover; the perception condition has no predator-prey or species check. `is_in_ambush_cover` treats food cells and adjacent food/burrow/wall cells as cover. Consequently, even herbivore-only meadow/sparse worlds can hide conspecifics and alter mate perception when the ambush flag is on. The meadow ambush arm had zero attack events in every seed but differed from control; sparse-resource had zero attacks and one seed differed by 25 living individual-ticks before both endpoints were extinct. Those changes cannot be attributed to protection from predation. This is a measured side effect of the current implementation; whether its scope is intended remains a design question. No correction or repeat was made.

**Alarm and traits measurement limits.** No captured decision trace in any world included a `Perception`, so all alarm reception counts are null/Unknown, not zero. Alarm-only emitted no signals in the herbivore-only fixtures. In the mixed fixture, calls were emitted but their receipt and downstream hearing are not measured. For extra traits, `EntityMetabolized` event-reported requested/paid sums are preserved per seed; any actual total outside emitted-event coverage stays Unknown. The traits arm changes vision and stamina jointly; it does not isolate either trait and is not a selection study. Its birth path also consumes two additional RNG draws per mating even with mutation set to zero, so downstream RNG trajectories can diverge independently of expressed-trait effects.

## Interpretation boundary

This is a small paired screen of current source behavior across three fixed fixtures, nine seed blocks, and 600 ticks. Arms share the initial map, positions, genomes, and RNG seed, with the registered phenotype differences allowed by the traits arm. Later RNG trajectories diverge after behavior changes (including two extra mating-path RNG draws in that arm), so these are paired starting conditions, not independent replicates. Births, deaths, living individual-ticks, reproduction, stress, resource trajectories, and endpoint populations are reported separately. No single reward or general Pareto ordering is supported. The results do not qualify a live upgrade or establish universal habitat transfer. No post-result tuning or additional World run was performed for this report.

For per-seed vectors, 100-tick resource/stress trajectories, generation-specific counts, alarm/attack totals, trait event sums, raw SHA-256 values, and machine-readable validation checks, see [ECOLOGY_TRANSFER_V1_20261026.json](ECOLOGY_TRANSFER_V1_20261026.json). The original protocol and all raw run artifacts remain unchanged.

Related evidence: [energy and package diagnosis](ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md), the [registered protocol](../ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md), and the [seed-scan addendum](../ECOLOGY_TRANSFER_V1_SCAN_ADDENDUM_20261026.md).
