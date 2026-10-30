# Vision contrast across accepted and patched habitats — completed paired study

**Status: the engineering calibration and all four registered formal seed blocks completed, passed independent audits, and have verified lossless raw archives.**

## Question and interpretation

This study compares the expressed-minus-neutral vision-1 breeder-frequency
change between the accepted qualification habitat and one deterministic food
patch layout. The primary interaction is `I = D_patched - D_accepted`, with
each `D` subtracting each arm's 0.5 tick-0 founder frequency from its
tick-200/500 vision-1 frequency among living herbivore descendant breeders.
The engineering calculation below is from one seen seed and is only a
measurement/exposure check. Each formal paired seed block is the replication
unit (`N=4`); family counts are dependence diagnostics, not extra replicates.

The prior positive prediction was that vision-1 might gain representation if
its added range helps herbivores locate food enough to repay maintenance. The
prior accepted-habitat study reported negative expressed-minus-neutral deltas.
This single-seed engineering run was also negative (`I=-0.096121`), but is
only a measurement check. The four formal pairs here had a mixed interaction:
mean `+0.023547`, median `-0.016539`, range `[-0.160442, +0.287709]`, with
one positive and three negative pairs. All eight within-habitat deltas were
negative. The positive mean is not a consistent vision advantage.
The feature toggle changes RNG paths and the
habitat changes the spatial food opportunity, so the arms are not exact
counterfactuals. Recorded food paths are observations from bounded perception,
not discoveries, meals, or causal utility.

## Engineering calibration — seen seed 123

The run created four fresh worlds at 200 ticks each (800 physical ticks):
accepted-neutral, accepted-expressed, patched-neutral, and patched-expressed.
Each began with the same 608 founder IDs/positions and balanced herbivore
genomes (288 vision-0, 288 vision-1). There were no post-tick-0 resets.

| Cell | Living herbivores / carnivores at 200 | Births | Deaths | Herbivore descendant breeders v0/v1 | v1 frequency | Within-cell change from founder 0.5 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Accepted-neutral | 549 / 36 | 1,190 | 1,213 | 165 / 190 (355) | 0.535211 | +0.035211 |
| Accepted-expressed | 593 / 36 | 1,193 | 1,172 | 323 / 90 (413) | 0.217918 | −0.282082 |
| Patched-neutral | 327 / 35 | 744 | 990 | 102 / 121 (223) | 0.542601 | +0.042601 |
| Patched-expressed | 316 / 35 | 700 | 957 | 182 / 27 (209) | 0.129187 | −0.370813 |

| Calibration contrast | Value |
| --- | ---: |
| `D_accepted = expressed change − neutral change` | −0.317294 |
| `D_patched = expressed change − neutral change` | −0.413414 |
| `I = D_patched − D_accepted` | −0.096121 |

Every cell retained living herbivores and carnivores and a positive herbivore
living-descendant-breeder denominator. Carnivore descendant breeders were 0,
0, 0, and 1 in the four cells, respectively. This was a known short-horizon
limitation, not an engineering gate: previous accepted-control evidence had
zero carnivore descendant breeders at 200 ticks, while the 500-tick vision
calibration had 14 per arm. This is not a coexistence qualification.

Family dependence was substantial. Parent-pair family counts by v0/v1 were
accepted-neutral `164/189`, accepted-expressed `320/90`, patched-neutral
`100/119`, and patched-expressed `179/27`. Distinct herbivore founder-ancestor
unions were `319/329`, `346/268`, `223/235`, and `206/103`, respectively.
Ancestry unions can overlap; neither family measure is an independent sample.

## Genotype-linked exposure

The denominator is the exact pre-tick living genotype-specific individual-tick
count (`life.is_alive()` and not Corpse/Removed). Acquisition and costs below
are divided by that exposure. Missing trace/perception coverage remains
unknown. Food-path histograms retain repeated entries split between plants and
corpses; path length greater than six means the recorded bounded-BFS path
exceeded the herbivore base radius. These rows do not establish discovery.

| Cell | Genotype | Living individual-ticks | Vision requested / paid / unpaid per tick | Plant food / tick | Water / tick | Known / missing-perception traces | Plant paths ≤6 / >6 | Corpse paths ≤6 / >6 | Known traces with any plant path ≤6 / known |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Accepted-neutral | v0 | 61,216 | 0 / 0 / 0 | 2.2482 | 1.1353 | 60,624 / 4,226 | 1,183,388 / 0 | 52,452 / 0 | 60,624 / 60,624 |
| Accepted-neutral | v1 | 64,651 | 0 / 0 / 0 | 2.2840 | 1.1565 | 64,050 / 4,359 | 1,256,492 / 0 | 56,097 / 0 | 64,050 / 64,050 |
| Accepted-expressed | v0 | 75,674 | 0 / 0 / 0 | 2.3864 | 1.2678 | 75,061 / 4,322 | 1,489,044 / 0 | 60,470 / 0 | 75,061 / 75,061 |
| Accepted-expressed | v1 | 45,796 | 0.9912 / 0.9186 / 0.0726 | 2.6207 | 0.9727 | 45,265 / 3,784 | 1,105,783 / 48,894 | 45,745 / 2,412 | 45,265 / 45,265 |
| Patched-neutral | v0 | 42,961 | 0 / 0 / 0 | 1.9010 | 0.9606 | 42,489 / 3,384 | 251,294 / 0 | 39,669 / 0 | 37,037 / 42,489 |
| Patched-neutral | v1 | 43,282 | 0 / 0 / 0 | 1.9195 | 0.9685 | 42,812 / 3,345 | 259,620 / 0 | 40,299 / 0 | 37,969 / 42,812 |
| Patched-expressed | v0 | 52,628 | 0 / 0 / 0 | 2.1801 | 1.1004 | 52,123 / 3,462 | 333,966 / 0 | 47,214 / 0 | 47,231 / 52,123 |
| Patched-expressed | v1 | 28,591 | 0.9917 / 0.7948 / 0.1969 | 2.0375 | 0.7008 | 28,194 / 2,953 | 192,686 / 11,902 | 30,770 / 1,885 | 24,929 / 28,194 |

Expressed v1 maintenance totaled 45,395 requested, 42,070 paid, 3,325 unpaid
in accepted habitat; and 28,353 requested, 22,723 paid, 5,630 unpaid in the
patched habitat. Short payment is retained as a biological cost outcome; it
is not an audit error. Basal metabolism is not inferred from these trait
costs.

The independent root audit spot-checked 5,720 exact fixed-frame food-path
samples. Its genotype-linked path histogram totals reconcile to the compact
raw records. All four cells passed typed-event and food/water stock-flow,
genotype, lineage, and exposure reconciliation. Accepted-neutral and
accepted-expressed tick-200 final State+RNG trees exactly matched the archived
Wave 6 seed-123 endings; the compatibility comparison added zero World ticks.

## Food layout and engineering gates

| Habitat | Initial Food | Food sites | Potential capacity | Nominal replenish per tick |
| --- | ---: | ---: | ---: | ---: |
| Accepted | 15,680 | 784 | 15,680 | 1,568 |
| Patched | 15,680 | 128 | 15,680 | 1,568 |

The patched layout used the preregistered eight deterministic 16-site patches.
The geometry audit verified unchanged bounds, 608 founder IDs/genomes/positions,
water, burrows, walls, and other non-Food cells. Capacity and nominal refill
are potential site totals; realized renewals remain an audited outcome because
stock clipping and consumption can differ by location.

All engineering gates passed: measurement and resource-flow accounting,
biology gate, and capture/budget forecast. The formal forecast was 416,248,554
raw bytes per four-cell block with margin, 1,680,121,788 bytes including
metadata for the four blocks, and 788.715 seconds including overhead; each
fits its preregistered limits (480 MiB per block, 2 GiB aggregate, 1,200
seconds). The forecast did not select or tune the food transform.

| Measure | Observed | Limit |
| --- | ---: | ---: |
| Runtime | 40.581 s | 300 s |
| Peak RSS | 499,948 KiB | 1 GiB address-space cap |
| Raw JSONL | 138,749,518 bytes | 240 MiB |
| Aggregate output cap | 256 MiB applied by runner | 256 MiB |

## Formal phase — four registered paired seed blocks

The formal phase used the preregistered seed allowlist `55001–55004`; each
block ran the four habitat-by-marker cells for 500 ticks each. The seen-seed
engineering calibration is excluded. Every cell passed the independent event,
lineage, genotype, exposure, food/water flow, geometry/founder, and phenotype
audits. Each cell had living herbivores and carnivores plus living descendant
breeders of both species.

| Seed | Accepted D | Patched D | Interaction I |
| --- | ---: | ---: | ---: |
| 55001 | −0.513873 | −0.545889 | −0.032016 |
| 55002 | −0.629524 | −0.341815 | +0.287709 |
| 55003 | −0.547321 | −0.707763 | −0.160442 |
| 55004 | −0.501330 | −0.502392 | −0.001062 |
| **Mean** |  |  | **+0.023547** |
| **Median** |  |  | **−0.016539** |
| **Range** |  |  | **[−0.160442, +0.287709]** |

`D` is the expressed-minus-neutral change from the 0.5 founder frequency to
the vision-1 frequency among living herbivore descendant breeders. `I` is
`D_patched − D_accepted`. All four founder cohorts started at 288 vision-0
and 288 vision-1 herbivores. The mean interaction is positive, but three of
four seed-pair interactions are negative and all eight habitat-specific `D`
values are negative. This small descriptive batch does not support a
consistent benefit from the patch layout for expressed vision. No significance
test was performed, and individuals were not pooled across seed blocks.

The table reports endpoint allele counts in both the full living herbivore
population and the living descendant-breeder cohort. A zero vision-1 breeder
count with a positive total denominator is a valid cohort frequency of zero;
it does not establish absence from all living herbivores.

| Seed | Cell | Living H / C | Living herbivores v0 / v1 | Descendant breeders v0 / v1 | Parent-pair families v0 / v1 | Founder-ancestor union v0 / v1 |
| --- | --- | ---: | ---: | ---: | ---: |
| 55001 | Accepted neutral | 573 / 46 | 275 / 298 | 192 / 205 | 192 / 204 | 387 / 394 |
|  | Accepted expressed | 587 / 44 | 584 / 3 | 399 / 1 | 396 / 1 | 360 / 106 |
|  | Patched neutral | 288 / 47 | 121 / 167 | 79 / 109 | 79 / 109 | 261 / 261 |
|  | Patched expressed | 333 / 38 | 320 / 13 | 228 / 8 | 225 / 8 | 212 / 206 |
| 55002 | Accepted neutral | 582 / 45 | 196 / 386 | 134 / 268 | 133 / 267 | 385 / 384 |
|  | Accepted expressed | 566 / 49 | 542 / 24 | 337 / 13 | 335 / 13 | 356 / 305 |
|  | Patched neutral | 296 / 42 | 185 / 111 | 130 / 75 | 129 / 75 | 266 / 266 |
|  | Patched expressed | 300 / 43 | 292 / 8 | 203 / 5 | 200 / 5 | 227 / 218 |
| 55003 | Accepted neutral | 580 / 48 | 262 / 318 | 170 / 212 | 169 / 212 | 392 / 396 |
|  | Accepted expressed | 580 / 49 | 576 / 4 | 389 / 3 | 389 / 3 | 371 / 273 |
|  | Patched neutral | 317 / 46 | 91 / 226 | 64 / 155 | 62 / 153 | 245 / 245 |
|  | Patched expressed | 298 / 43 | 298 / 0 | 196 / 0 | 193 / 0 | 228 / 0 |
| 55004 | Accepted neutral | 584 / 48 | 295 / 289 | 193 / 200 | 192 / 199 | 373 / 376 |
|  | Accepted expressed | 579 / 47 | 574 / 5 | 393 / 3 | 391 / 3 | 357 / 193 |
|  | Patched neutral | 315 / 46 | 152 / 163 | 104 / 105 | 102 / 105 | 246 / 246 |
|  | Patched expressed | 294 / 44 | 294 / 0 | 201 / 0 | 197 / 0 | 231 / 0 |

Founder-ancestor unions can overlap, and parent-pair families are not
independent observations. These diagnostics do not increase the replication
count beyond four paired seed blocks. Habitat and marker-expression arms can
follow different RNG paths; the interaction is not a pure causal vision
effect or general fitness estimate.

The formal batch completed 8,000 World ticks in 423.626 seconds. Total formal
raw capture was 978,040,989 bytes; peak worker RSS was 510,588 KiB under a 1 GiB
address-space cap. The recorded 992,150,499-byte aggregate snapshot was
measured after execution and audits, before aggregate summary writing and
archival; it is not a post-archive directory-size measurement.

## Provenance and scope

| Input or audit | SHA-256 |
| --- | --- |
| Frozen source revision | `cdb349f76cc2a82c966925295a8769b9fe7b7ed5` |
| Source tree | `996753e313290baa85851d07be69c02887889c86` |
| Probe binary | `c26f17a22491ce68c3828880c1ebe18e3838470e302ace2cdbad12b240c3f52f` |
| Protocol | `8f90a94a04c02d5cae374f57f6c1cc2208d66c3cbf6519fffc0db470c3eb78fd` |
| Scenario | `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac` |
| Seed inventory | `5026aeb4c1499e58ff94627328f33a322e5d86ab39ef760dcdb673668b4cbff7` |
| Runner | `02ad90bee4695518c9469b0bbcc3c9c1298310888604f2159fe97a6087056712` |
| Root event/exposure audit | `40220cfbde805f7532d40ad96f46d8505b2c0fb25ffa49efee889a25f9804ada` |
| Geometry/founder audit | `5def52667d1a72a17e6eb1f9a5b1f4d15e36245eb979b83be2c10719fd68217b` |
| Engineering raw JSONL | `96176a200400dd571858ceb40248d2038a78502317d9a76cd59361a91ae63ef4` |
| Formal batch manifest | `0cd1ce760a6bb9336c3ce8da2b3781bc7d41ad4f2738275d4b3dc78f6823ec13` |
| Formal batch summary | `062e4502655194f42babdb1db7bbda623d981c8801c5884c2eaf5568b0dff7c2` |
| Archive inventory | `data/operations/wave8-checks-20261024/archives.json` (`e450187df609c679a4d0a35327a92a097019324fd42c1022a051a7f80990bf41`) |
| Check manifest | `data/operations/wave8-checks-20261024/manifest.json` (`4124ae6a3eeaa04f930d3985efaddd0fbc694718914f7471731c227f15a477b3`) |
| Cleanup proof | `data/operations/wave8-checks-20261024/cleanup.json` (`b4eed0e57fb1a1fa223fcbaa8719035af36d03fc1ec0cc2703102182e8fe48d6`) |

Engineering evidence is retained at
`data/experiments/vision-habitat-v1-engineering-20261024/`; formal evidence is
retained at `data/experiments/vision-habitat-v1-formal-20261024/`. The five raw
captures have verified lossless zstd archives: the seen-seed calibration and
formal seeds 55001–55004. Each `raw.jsonl.archive.json` index records original
and compressed hashes; all five are inventoried in
`data/operations/wave8-checks-20261024/archives.json`. Restore each original
`raw.jsonl` before using readers that require uncompressed input. The archive
proof reports 1,116,790,507 original bytes to 87,624,332 compressed bytes and
confirms all original hashes and experiment manifest/audit hashes. The
engineering run used 800 World ticks and no live deployment. The frozen lab
source checks passed 264 Rust tests / 0 failures (49 summary groups), 177 Python
tests / 0 failures, release app/probe, and scoped formatting.
Combined root checks on the integrated main tree reported 275 Rust tests / 0
failures (50 summary groups), 177 Python tests / 0 failures, release app/probe,
and scoped formatting passed. The combined count includes an unrelated shelter
prototype (11 pure tests, commit `3deef3a`); it is not part of this World study.
Cleanup removed 620,910,983 logical bytes (~592.15 MiB) from inactive
`/tmp/curious-wave8-target` only; proof is linked above. Root verified all five
retained probe hashes. No new World jobs are active, and live services were left
untouched. These source checks are separate from the four audited formal seed
blocks.

No result establishes food discovery, a pure causal vision effect, general
fitness, coexistence qualification, long-term persistence, or habitat
robustness. Do not pool individuals across seed blocks or interpret exposure
counts as utility.
