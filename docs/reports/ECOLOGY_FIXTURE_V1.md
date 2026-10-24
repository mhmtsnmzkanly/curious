# Ecology fixture validity — bounded V1 result

**Result:** The registered calibration passed its technical and observation checks. All three preregistered 500-tick formal seeds passed the bounded fixture-viability gate. This is a small, fixed-horizon diagnostic for the current V17 source under explicitly carried accepted-profile settings. It is not a reproduction of the old qualification binary, a V18 result, an all-habitat result, a trait-fitness comparison, or a general coexistence or reliability claim.

## Scope and provenance

The starting fixture is the archived `qualification-both/scenario.json`: 608 founders (576 herbivores and 32 carnivores), 976 renewable sites, and 1,232 initial resource cells. Its SHA-256 is `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`. The old accepted profile was revision `b4dcef20fdef830c2895806076d972df62741eea`, binary SHA-256 `e1ffa03754fc964d503b6a6e04bffe19ad868b3d2090e0a8b2f9c48e3506f636`, with `reserve_opportunities=true` and `mate_rendezvous=true`.

The measured runs used current source revision `466f9a0f6af18d12469e1d7761b62ce5cd771f49` (source content SHA-256 `a37d6e396b22ba68c7a454d78e98d18606d422f905c074331a6d4f54fa49ca5d`) and frozen probe SHA-256 `b4b5830bea49ed2e8532840880e5fadb526440363adad292220ac224e72accc0`. The harness explicitly used `mode=both`, companions enabled, genetics enabled with mutation rate 20 per mille, extra traits and hunt learning disabled, water learner/updates enabled, water-residual and refuge features disabled, and both accepted policy flags enabled. Because the source has changed, these results are conditional on the explicit profile settings; they do not prove bit-for-bit or complete disabled-path equivalence with the old accepted binary.

The selected seed roles had no archive matches in the registered scans (1,093 text sources for calibration and 1,185 for formal). A preflight process was interrupted before an output directory or `World` was created; root verified that it had consumed no seed. The retained calibration directory contains the one completed calibration run, not a rerun of a world. The formal directory contains one completed invocation of all three registered seeds.

| Artifact | Calibration | Formal |
| --- | --- | --- |
| Directory | `data/experiments/ecology-fixture-v1-calibration-20261024/` | `data/experiments/ecology-fixture-v1-20261024/` |
| Registered protocol SHA-256 | `9168da984cfcb413bc74c5108db8198f2edb060e5558640c4e22f0a52a3d74cf` | `6aaeec85d720939ab745837d21c580bdd75325f3f22ce0d4f5a5f727c405e816` |
| Manifest SHA-256 | `4a8c8622ee327aa71b1fc1f2105e4cf8ef17a7e29109bffaf24ff843adf4e3f4` | `5c6e0352b78d6a776e95848564b0d7aa250493940c919d6d7b573809481dfd91` |
| Root audit SHA-256 | `d363f102ee419b66cdafcee875748ed65b3cfd5eeb471644f7d131b86765d8d4` | `9e5ce12bb3a543437a00a74a94c1828a7efb6ed1d4bbe7fccd1bc70d3af3208b` |
| Probe SHA-256 | `b4b5830bea49ed2e8532840880e5fadb526440363adad292220ac224e72accc0` | same |

The frozen protocol used 300 seconds per phase, a 1-GiB combined address-space ceiling, and 128 MiB aggregate output. The calibration took 98.851 seconds overall; its measured probe time was 13.9839 seconds. The frozen formal feasibility forecast was 269.759 seconds (under the 300-second limit); the formal phase completed in 134.229 seconds. Formal raw JSONL totaled 94,073,278 bytes. There was no retuning or seed replacement.

## Calibration

Seed `53099` completed 200 ticks. Calibration passed the registered technical checks: no extinction, no food/water residuals or resource-kind replacements, consistent resource flow and lineage, no parent/unattributed/trait-payment audit issues, and at least one actual birth. It recorded 1,236 actual mating events and an adjacent-pair exposure proxy of 1,193. The proxy describes exposure and is not proof of mating opportunity by itself. The independent root audit checked stock/flow arithmetic at all 200 ticks, 1,844 lineage identities, and all four full audit frames.

There were 1,233 herbivore births and 3 carnivore births. At tick 200, 592 herbivores and 35 carnivores remained; 404 living herbivore descendants had bred, while no living carnivore descendant breeder was recorded. This is consistent with calibration being a technical/observer gate only; it was not the formal viability criterion.

Calibration raw JSONL SHA-256: `29d23f7b7a140beac70519faf18eb6392466e7dab764824d72c04a835f1491bd` (24,728,247 bytes).

## Formal bounded viability result

Each registered 500-tick seed passed all preregistered checks: both species avoided extinction; each species had at least two living individuals on every tick in the final 50; each had a living descendant that had itself bred at tick 500; each had at least one birth in the final 50; and the technical resource and lineage gates passed. The root's independent audit checked all 1,500 stock/flow ticks, all genealogy, the 15 full audit frames including typed resource events and stocks, and the final living-descendant-breeder counts. Root audit records aggregate counters bound to source plus captured frames; they are not a replay of every unretained movement/action event.

| Seed | Herbivore / carnivore births | Final alive H / C | Minimum alive H / C in final 50 | Living descendant breeders H / C at end | Births H / C in final 50 | Lineage identities audited |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 53001 | 2,866 / 23 | 571 / 49 | 563 / 47 | 375 / 15 | 276 / 3 | 3,497 |
| 53002 | 2,866 / 19 | 569 / 45 | 565 / 44 | 369 / 8 | 258 / 2 | 3,493 |
| 53003 | 2,887 / 18 | 562 / 44 | 553 / 43 | 353 / 11 | 270 / 2 | 3,513 |

All three seeds passed the bounded fixture-viability gate at the registered horizon. The raw files and their hashes are listed in the JSON companion. There were no biological negatives or infrastructure failures in the formal phase. This small three-seed result establishes only that this fixed fixture produced the registered short-horizon lineage and survival observations under these current-source settings. It does not establish long-run coexistence, fitness effects, trait advantage, habitat robustness, or production reliability. Basal metabolism and unattributed body-energy changes remain unmeasured/unknown in this audit.

## Retained evidence

The experiment directories retain registered protocol snapshots, manifests, status, immutable probe copies, raw JSONL, stderr, and root audit summaries. The root audit helper and provenance remain under `data/operations/ecology-wave3-root-audit.py` and its provenance record. The structured companion [ECOLOGY_FIXTURE_V1.json](ECOLOGY_FIXTURE_V1.json) records source, protocol, manifest, root-audit, and raw hashes with the per-seed results.
