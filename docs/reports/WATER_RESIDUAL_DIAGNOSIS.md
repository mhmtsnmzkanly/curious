# V17 water residual tradeoff diagnosis

**Conclusion:** the frozen eight-seed run contains three state-matched, resolver-confirmed greedy choice witnesses. It does not pass the preregistered physical utility criterion. Trained-online trades off against the strong rules arm in all eight seeds and against trained-frozen in two; five are identical and one favors online. This is a mechanism diagnosis, not a new experiment or efficacy claim.

## What was rechecked

The analysis reads the immutable formal JSONL one seed record at a time. It creates no `World`, calls no probe, and advances no simulation tick. It recomputes focal water and movement totals from resolver events, survival and terminal body state from recorded frames, Pareto relations, current-score rankings, and the residual sample formula. The aggregate physical vectors match the independent root audit. The three paired witnesses reproduce exactly at 52002/247, 52003/415, and 52006/241. The full analysis took 4.8 seconds and peaked at 342,576 KiB RSS in this environment.

It verifies the formal raw, manifest, summary, protocol, probe, root audit, calibration raw/summary/manifest, and all 64 source files in the frozen checkout before analysis. Registered source revision: `506ed6f4b3ec3bab8bb65ecfdb8ea3bed2797602`; source tree SHA-256: `09e4890d23df8ea4267e032d440b73743493b40040f48207d344eb061b6ecf49`. Formal raw SHA-256: `c10e7d55431f6c10e96acf1f84d7c4295d67208b54ab6ab788ae210d30fcf78c`. Formal protocol SHA-256: `0410576adef7474ec23dd9e92a73eb5c9306f87c0f6450696c3ca5e424a22bc0`. Full verified hashes and source binding are in the JSON.

Reproduce with:

```sh
python3 scripts/analyze_water_residual_tradeoffs.py --raw /root/curious/data/experiments/water-residual-v1-20261023/raw.jsonl --manifest /root/curious/data/experiments/water-residual-v1-20261023/manifest.json --source-root /root/curious-research/water-lab-wave2 --output docs/reports/WATER_RESIDUAL_DIAGNOSIS.json
python3 -m unittest scripts/test_water_residual_tradeoffs.py
```

## Choice and learning diagnosis

The three learner arms had 282, 274, and 282 fresh candidate decisions for trained-online, trained-frozen, and naive-online. Of these, their greedy/exploration splits were 257/25, 252/22, and 258/24. The rule-based arm had 281 fresh rule decisions and no epsilon draw. This exploration and the later RNG stream differ across the rule-based comparison, so it is a complete-policy comparison.

At fresh greedy decisions, removing only the residual mean from recorded candidate scores changed the local top-ranked source 6/257 times online, 3/252 frozen, and 7/258 naive. This is a same-perception score-ranking counterfactual with the V17 shorter-route tie-break; it does not replay a policy or predict a physical outcome.

The JSON keeps only those 16 fresh greedy rank-change rows as detailed per-choice records. It omits repeated rows for all other decisions; aggregate and per-seed choice counts, source context ranges, and the three matched witnesses remain. The complete choice trace remains in the immutable raw archive and can be regenerated with the analysis script.

Completed own episode residuals were recomputed from each recorded closure using V17’s integer formula: `trunc0(((2 × water − paid movement energy − episode ticks) × 100) / episode ticks) − captured choice baseline`. Samples require `Reached` and exclude `outcome_overflowed` or `candidate_censored` attempts, even if a malformed record labels one `Reached`. For online arms, the recomputed samples match the exact observed residual-table sum changes.

| Arm | Reached outcome samples | Positive / zero / negative | Recomputed sample sum | Recorded table updates | Recorded sum delta |
|---|---:|---:|---:|---:|---:|
| `trained_online` | 281 | 1 / 261 / 19 | -25000 | 281 | -25000 |
| `trained_frozen` | 273 | 0 / 260 / 13 | -18500 | 0 | 0 |
| `naive_online` | 281 | 0 / 267 / 14 | -19600 | 281 | -19600 |

Every formal suffix had exactly two perceived water candidates at fresh choices. Across trained-online choices, source `(0,2)` was seen with amounts 1–12 and route lengths 0–4; source `(4,2)` with amounts 2–12 and route lengths 1–4. Focal body water ranged 42–44 and energy 59–68 at these choices. Thus one per-source mean spans changing amounts, routes, and body states. The protocol reverses water renewal rates before tick 201. At that fork, 3/8 trained histories had a nonzero residual mean: source `(0,2)` was −76, −84, or −80 in seeds 52003–52005; other source means were zero. That history was carried into the changed renewal regime. These are context/persistence facts, not proof that aliasing caused the aggregate tradeoff.

## State-matched mechanism witnesses

All three matches are fresh greedy choices with equal normalized physical state, body, perception, and pre-choice RNG. Normalization is limited to the preregistered learner value/residual tables and update flag; active episode state stays equal. The selected source and focal resolver events then differ:

| Seed / suffix tick | Trained-online source and events | Trained-frozen source and events |
|---|---|---|
| 52002 / 247 | `0,2`: EntityMoved(1), EntityDrank(7,0,2) | `4,2`: EntityMoved(2), EntityDrank(8,4,2) |
| 52003 / 415 | `0,2`: EntityDrank(7,0,2) | `4,2`: EntityMoved(2), EntityDrank(8,4,2) |
| 52006 / 241 | `0,2`: EntityDrank(6,0,2) | `4,2`: EntityMoved(3), EntityDrank(8,4,2) |

These show that the residual update can change a greedy choice and the own resolver outcome in the same recorded state. They do not show that the different outcome is generally better.

## Per-seed, per-arm outcomes

Water and movement are sums of focal resolver events. Terminal columns are energy/water/health. `G/E` is fresh greedy/exploration choices. `Rank` counts greedy fresh decisions where removing the residual changes the local top-ranked source. `Residual` gives eligible reached episode samples as positive/zero/negative; `Δsum` is the observed table change. `Censor` counts `Interrupted`, `Disabled`, `Died`, or `TimedOut` closures, plus closures whose attempt has `outcome_overflowed` or `candidate_censored`; a verified `Failed`/zero-water result is completed, not censored. The raw formal record has three `Interrupted` closures, no `Failed` closures, and no serialized censor flags. Deaths were zero for every focal branch. The JSON includes exact closure ends, censor reasons, candidate-context ranges, and all per-seed Pareto relations.

| Seed | Arm | G/E | Water | Move energy | Terminal E/W/H | Survival | Residual +/0/− | Rank | Δsum | Censor | Pareto vs rules / frozen |
|---:|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---|
| 52001 | `trained_online` | 36/1 | 296 | 208 | 65 / 47 / 120 | 300 | 0/36/1 | 0 | -1600 | 0 | tradeoff / identical |
| 52001 | `trained_frozen` | 36/1 | 296 | 208 | 65 / 47 / 120 | 300 | 0/36/1 | 0 | 0 | 0 | — |
| 52001 | `naive_online` | 36/1 | 296 | 208 | 65 / 47 / 120 | 300 | 0/36/1 | 0 | -1600 | 0 | descriptive: tradeoff / identical |
| 52001 | `rule_based` | 0/0 | 298 | 183 | 60 / 49 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52002 | `trained_online` | 31/3 | 294 | 204 | 63 / 46 / 120 | 300 | 0/30/4 | 2 | -4700 | 0 | tradeoff / tradeoff |
| 52002 | `trained_frozen` | 31/1 | 296 | 210 | 61 / 48 / 120 | 300 | 0/31/1 | 0 | 0 | 0 | — |
| 52002 | `naive_online` | 35/2 | 302 | 206 | 65 / 54 / 120 | 300 | 0/34/3 | 3 | -4100 | 0 | descriptive: candidate_dominates / candidate_dominates |
| 52002 | `rule_based` | 0/0 | 302 | 213 | 62 / 54 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52003 | `trained_online` | 33/3 | 301 | 206 | 64 / 53 / 120 | 300 | 0/31/4 | 2 | -4450 | 1 | tradeoff / candidate_dominates |
| 52003 | `trained_frozen` | 31/3 | 295 | 206 | 64 / 47 / 120 | 300 | 0/31/2 | 2 | 0 | 1 | — |
| 52003 | `naive_online` | 29/5 | 301 | 207 | 69 / 53 / 120 | 300 | 0/32/1 | 0 | -950 | 1 | descriptive: tradeoff / tradeoff |
| 52003 | `rule_based` | 0/0 | 298 | 202 | 62 / 50 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52004 | `trained_online` | 29/6 | 297 | 174 | 61 / 51 / 120 | 300 | 0/33/2 | 1 | -2950 | 0 | tradeoff / identical |
| 52004 | `trained_frozen` | 29/6 | 297 | 174 | 61 / 51 / 120 | 300 | 0/33/2 | 1 | 0 | 0 | — |
| 52004 | `naive_online` | 30/4 | 300 | 192 | 59 / 54 / 120 | 300 | 0/33/1 | 0 | -1600 | 0 | descriptive: tradeoff / tradeoff |
| 52004 | `rule_based` | 0/0 | 292 | 209 | 62 / 46 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52005 | `trained_online` | 31/3 | 295 | 180 | 60 / 46 / 120 | 300 | 0/32/2 | 0 | -2100 | 0 | tradeoff / identical |
| 52005 | `trained_frozen` | 31/3 | 295 | 180 | 60 / 46 / 120 | 300 | 0/32/2 | 0 | 0 | 0 | — |
| 52005 | `naive_online` | 31/3 | 295 | 180 | 60 / 46 / 120 | 300 | 0/32/2 | 0 | -2100 | 0 | descriptive: tradeoff / identical |
| 52005 | `rule_based` | 0/0 | 298 | 221 | 67 / 49 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52006 | `trained_online` | 34/4 | 297 | 218 | 60 / 49 / 120 | 300 | 1/33/4 | 1 | -6000 | 0 | tradeoff / tradeoff |
| 52006 | `trained_frozen` | 31/3 | 296 | 225 | 65 / 48 / 120 | 300 | 0/31/3 | 0 | 0 | 0 | — |
| 52006 | `naive_online` | 34/4 | 295 | 208 | 68 / 47 / 120 | 300 | 0/34/4 | 4 | -6050 | 0 | descriptive: tradeoff / tradeoff |
| 52006 | `rule_based` | 0/0 | 296 | 206 | 60 / 48 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52007 | `trained_online` | 29/3 | 298 | 231 | 68 / 44 / 120 | 300 | 0/32/0 | 0 | 0 | 0 | tradeoff / identical |
| 52007 | `trained_frozen` | 29/3 | 298 | 231 | 68 / 44 / 120 | 300 | 0/32/0 | 0 | 0 | 0 | — |
| 52007 | `naive_online` | 29/3 | 298 | 231 | 68 / 44 / 120 | 300 | 0/32/0 | 0 | 0 | 0 | descriptive: tradeoff / identical |
| 52007 | `rule_based` | 0/0 | 301 | 185 | 66 / 47 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |
| 52008 | `trained_online` | 34/2 | 296 | 230 | 67 / 46 / 120 | 300 | 0/34/2 | 0 | -3200 | 0 | tradeoff / identical |
| 52008 | `trained_frozen` | 34/2 | 296 | 230 | 67 / 46 / 120 | 300 | 0/34/2 | 0 | 0 | 0 | — |
| 52008 | `naive_online` | 34/2 | 296 | 230 | 67 / 46 / 120 | 300 | 0/34/2 | 0 | -3200 | 0 | descriptive: tradeoff / identical |
| 52008 | `rule_based` | 0/0 | 298 | 200 | 65 / 48 / 120 | 300 | 0/0/0 | 0 | 0 | 0 | — |

## Aggregate physical comparison

All 32 focal branches survived all 300 suffix ticks, with health 120 at the end. The registered utility criterion still fails because the trained-online vector must Pareto-dominate both controls.

| Arm | Water acquired | Movement energy paid | Terminal energy | Terminal water | Health | Survival ticks |
|---|---:|---:|---:|---:|---:|---:|
| `trained_online` | 2374 | 1651 | 508 | 382 | 960 | 2400 |
| `trained_frozen` | 2369 | 1664 | 511 | 377 | 960 | 2400 |
| `rule_based` | 2383 | 1619 | 504 | 391 | 960 | 2400 |
| `naive_online` | 2383 | 1662 | 521 | 391 | 960 | 2400 |

Against rules, trained-online has −9 water, +32 paid movement energy, +4 terminal energy, −9 terminal water, and equal health/survival: tradeoff in 8/8 seeds. Against trained-frozen it has +5 water, −13 movement energy, −3 terminal energy, +5 terminal water, and equal health/survival: one online-dominant seed, two tradeoffs, five identical. The aggregate does not dominate either control. Naive-online is descriptive only; its exploration means it is not a rules-only comparison.

## Narrow next hypothesis

The observed candidate table stores one mean per source while visible amount and route vary broadly, and trained means persist across the preregistered rate reversal. Residual samples were overwhelmingly zero or negative, and removing the residual changed the local greedy rank in only 6 of 257 trained-online fresh greedy decisions. A bounded next hypothesis, if separately preregistered, is that **pooling residuals by source alone loses useful current context and retains stale source history across changing availability**. A future test could stratify the recorded residual by already-visible amount and route-length bands, then compare fresh unseen seeds against unchanged V17, trained-frozen, and rule-based arms while preserving exploration accounting and resolver physical measures. This diagnosis does not authorize implementing that variant, selecting thresholds from these seeds, or running another batch.

## Limits

- This is a read-only diagnosis of one 5×5 fixture and eight fixed seeds; it is not a new independent sample or a general effect estimate.
- Rules versus learner arms differ in 10% exploration and ensuing RNG use. Those outcomes compare complete policies, not isolated residual learning.
- The rank-only “remove residual” check uses each recorded perception and current scores; it does not resimulate decisions or estimate alternate world outcomes.
- Sparse samples, broad current contexts, and a rate reversal make source-history aliasing plausible, but this run does not establish it as the cause of the failed utility criterion.
- No source/policy/protocol/raw input was changed; no World simulation, new seed, build, deployment, or commit was performed.
