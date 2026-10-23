# Continuous-lifetime water learning — V2 bounded result

**Result:** The formal run contains one preregistered, causally supported individual learning example (seed `51007`). Online learning changed the focal subject’s greedy water-site choice in all eight seeds relative to the same experienced table held frozen. A resolver-confirmed physical difference followed each first paired divergence. At the common horizon, one seed (`51007`) met the preregistered favorable physical-vector condition, six showed tradeoffs, and in seed `51001` the frozen arm Pareto-dominated online. This supports one existence example, not a general performance advantage.

## Run integrity and provenance

The calibration capture ran once for seed `51099` (1,400 world ticks). Its original validator incorrectly required the captured initial RNG to equal the input seed. `World::with_seed` first populates the temporary 5×5 map: 25 spawn-roll calls for seed `51099`, with zero successful spawn rolls, advance the LCG to the captured state `15382464887130188488`. The validation failure was an assertion defect; the original raw capture and failed manifest are preserved. First and second reanalyses read that same raw capture, each replaying zero simulation ticks.

The first formal preflight then failed before output-directory creation or probe launch: the seed scanner’s 16 MiB default rejected the 19,443,712-byte calibration JSONL line despite the preregistered 64 MiB per-seed bound. The scanner was repaired to accept a per-call limit and perform a complete fail-closed 64 MiB scan. Reanalysis 2, bound to source revision `41c3a9fd7e450ab04f844197584f0912af7415cc`, passed with zero new ticks; the original calibration raw/probe/protocol hashes still matched. Its validity observations were 23 fresh two-source choices, 23 drinks and 21 nonfull choices in the prefix, and 35 fresh choices, 34 drinks and 33 nonfull choices in the online suffix; both actors were alive at the fork. No formal seed was consumed by the failed preflight.

Formal evaluation then completed all eight preregistered seeds and four counterfactual suffixes per seed: 11,200 physical ticks, 1 GiB child-memory limit, 300-second batch limit, empty stderr, and no runner error. Each subject reached the tick-200 fork alive; all four focal branches in every seed remained alive to the 300-tick suffix ceiling. The rival actors also remained alive. No seed replacement, body/age reset, world reset, external refill, forced choice, or parameter change occurred.

| Artifact | Identity |
| --- | --- |
| Source revision / source SHA-256 | `41c3a9fd7e450ab04f844197584f0912af7415cc` / `3b7fb339505624205ad9e736f80130a7a69bc48cea353c198249086c15f6c5da` |
| Probe SHA-256 | `bf59d25881e85c7a648c3dd9c6af450511dbbcf824bb9d462d52dc523ae7a693` |
| Protocol SHA-256 | `49d6653a63464a43679d68c275b15c8090e95f57a77d9067ce9fa4a91e5d2047` |
| Runner SHA-256 | `2f37dda843250b7cd5d5e0383c185bc694b47135937a41f67a7b8a6d61be444a` |
| Formal raw SHA-256 | `03aab983f69ec4543eb7d035e6d3e7b7c0e247102fd5480f38f9bb4ab2e1000c` |
| Formal summary SHA-256 | `35dcb3de9c87555de0a18a3fe9dc1f7ec93113e8499ce6b4c259092050370e01` |

## Study design

This was a predator-free 5×5 safe-habitat fixture with a finite, ordinary AI rival. Both actors were configured at birth with `max_age=512` and `maturity_age=501`; all other body fields used defaults. The focal individual experienced a continuous 200-tick prefix, then the same tick-200 checkpoint was forked into online, trained-frozen, ablated-frozen, and rule-based 300-tick suffixes. Renewal rates reversed before tick 201. No body or age reset, replacement, forced position, external refill, mating, or offspring occurred. The longer fixture-specific age limit allowed the preregistered 500-tick study window; it is not a longevity or age-bonus result.

## Causal choice evidence

At the first fresh two-source choice where `online` and `trained_frozen` differed, their pre-choice world, focal body, perceived candidates, and RNG matched after normalizing only the registered learner values and flags. The subject selected site B `(4,2)` online and site A `(0,2)` with its same table frozen. The same-exposure shadow calculation confirmed the online choice was greedy under the current table and the opposite source was greedy under the tick-200 table; the choice was not exploration. This paired check passed in all eight seeds. Each choice was followed by a resolver-confirmed difference in acquired water, paid movement energy, or both. The chosen sources immediately had different available amounts/path lengths in ordinary perception; the harness supplied no hidden world information.

| Seed | First divergent tick | Online / frozen source | Matched post-choice ticks | Water delta | Movement-energy-paid delta | Same-world/RNG + greedy shadow |
|---:|---:|---|---:|---:|---:|---|
| 51001 | 236 | B / A | 265 | -1 | +13 | pass |
| 51002 | 228 | B / A | 273 | +2 | -2 | pass |
| 51003 | 241 | B / A | 260 | -2 | -17 | pass |
| 51004 | 240 | B / A | 261 | -2 | +0 | pass |
| 51005 | 244 | B / A | 257 | +6 | -17 | pass |
| 51006 | 236 | B / A | 265 | +5 | -52 | pass |
| 51007 | 213 | B / A | 288 | +3 | -19 | pass |
| 51008 | 233 | B / A | 268 | +2 | -3 | pass |

The resource-rate reversal changed which site renewed faster at the fork. At every first divergent choice, the online current-table greedy winner was B and the frozen tick-200 table’s winner was A. All eight seeds also recorded a post-reversal argmax reversal. The same-world comparison at that first fresh choice, rather than later decisions in already divergent worlds, is the causal basis for attributing the selected action to updated experience.

## Physical outcomes across every seed and arm

The primary physical comparison is online minus same-history `trained_frozen` at the registered common horizon (the shorter paired survival, capped at 300 ticks). Positive acquired water, terminal energy, terminal health, and survival are favorable; lower paid movement energy is favorable. The horizon was 300 ticks in every pair. Health and survival were identical across these pairs.

| Seed | Online − frozen: water | Movement energy paid | Terminal energy | Health | Survival | Classification |
|---:|---:|---:|---:|---:|---:|---|
| 51001 | -1 | +13 | -3 | +0 | +0 | frozen dominates online |
| 51002 | +2 | -2 | -8 | +0 | +0 | tradeoff |
| 51003 | -2 | -17 | +1 | +0 | +0 | tradeoff |
| 51004 | -2 | +0 | +2 | +0 | +0 | tradeoff |
| 51005 | +6 | -17 | -7 | +0 | +0 | tradeoff |
| 51006 | +5 | -52 | -4 | +0 | +0 | tradeoff |
| 51007 | +3 | -19 | +1 | +0 | +0 | online favorable vector |
| 51008 | +2 | -3 | -1 | +0 | +0 | tradeoff |

Seed `51007` is the only complete favorable vector: `+3` water, `−19` movement energy, `+1` terminal energy, unchanged health, and unchanged 300-tick survival. An independent root audit of its tick-213 choice recorded the same pre-choice body/perception/RNG, greedy (not exploratory) selection, and the first resolver events: online drank 10 units at B; frozen moved at a cost of 2 energy and drank 1 unit at A. At the full common horizon, online acquired 302 water and paid 214 movement energy, compared with 299 and 233 for frozen; terminal energy was 63 versus 62.

Six vectors have tradeoffs: some gained more water or paid less movement energy while ending with lower energy, while other combinations moved one component favorably and another adversely. In seed `51001`, the trained-frozen arm was better on water, paid movement energy, and terminal energy, with health and survival equal. It therefore Pareto-dominated online; this is a physical loss for online under the registered components, not a tradeoff. No aggregate utility score was calculated. The decision mechanism occurred in all eight seeds, but one favorable existence example does not estimate an expected advantage.

Every branch’s focal physical metrics are listed below. “Water” is the sum of resolver-confirmed acquired amounts; movement energy is resolver-paid cost; terminal energy/health/water are the subject’s state at the 300-tick ceiling. “Choices / drinks” counts fresh perceived two-source choices and resolver-confirmed drinks. `rule_based` is descriptive context, not the primary control.

| Seed | Arm | Choices / drinks | Water acquired | Movement energy paid | End energy / health / water | Survival ticks |
|---:|---|---:|---:|---:|---|---:|
| 51001 | online | 35 / 37 | 296 | 229 | 64 / 120 / 48 | 300 |
| 51001 | trained_frozen | 41 / 62 | 297 | 216 | 67 / 120 / 49 | 300 |
| 51001 | ablated_frozen | 41 / 51 | 296 | 181 | 66 / 120 / 48 | 300 |
| 51001 | rule_based | 33 / 33 | 294 | 201 | 68 / 120 / 46 | 300 |
| 51002 | online | 37 / 39 | 302 | 219 | 59 / 120 / 54 | 300 |
| 51002 | trained_frozen | 41 / 61 | 300 | 221 | 67 / 120 / 52 | 300 |
| 51002 | ablated_frozen | 39 / 49 | 296 | 205 | 61 / 120 / 48 | 300 |
| 51002 | rule_based | 33 / 33 | 294 | 206 | 66 / 120 / 46 | 300 |
| 51003 | online | 35 / 36 | 293 | 228 | 67 / 120 / 47 | 300 |
| 51003 | trained_frozen | 40 / 61 | 295 | 245 | 66 / 120 / 49 | 300 |
| 51003 | ablated_frozen | 39 / 54 | 291 | 236 | 63 / 120 / 45 | 300 |
| 51003 | rule_based | 30 / 30 | 296 | 218 | 69 / 120 / 50 | 300 |
| 51004 | online | 39 / 41 | 296 | 222 | 68 / 120 / 50 | 300 |
| 51004 | trained_frozen | 45 / 66 | 298 | 222 | 66 / 120 / 52 | 300 |
| 51004 | ablated_frozen | 40 / 52 | 296 | 230 | 62 / 120 / 50 | 300 |
| 51004 | rule_based | 30 / 30 | 299 | 202 | 62 / 120 / 53 | 300 |
| 51005 | online | 35 / 37 | 305 | 218 | 60 / 120 / 52 | 300 |
| 51005 | trained_frozen | 41 / 63 | 299 | 235 | 67 / 120 / 46 | 300 |
| 51005 | ablated_frozen | 45 / 55 | 300 | 199 | 65 / 120 / 47 | 300 |
| 51005 | rule_based | 31 / 31 | 303 | 203 | 57 / 120 / 50 | 300 |
| 51006 | online | 32 / 34 | 300 | 194 | 62 / 120 / 54 | 300 |
| 51006 | trained_frozen | 39 / 64 | 295 | 246 | 66 / 120 / 49 | 300 |
| 51006 | ablated_frozen | 39 / 48 | 294 | 154 | 60 / 120 / 48 | 300 |
| 51006 | rule_based | 30 / 30 | 300 | 194 | 64 / 120 / 54 | 300 |
| 51007 | online | 35 / 35 | 302 | 214 | 63 / 120 / 49 | 300 |
| 51007 | trained_frozen | 40 / 60 | 299 | 233 | 62 / 120 / 46 | 300 |
| 51007 | ablated_frozen | 41 / 51 | 301 | 199 | 64 / 120 / 48 | 300 |
| 51007 | rule_based | 35 / 35 | 305 | 198 | 65 / 120 / 52 | 300 |
| 51008 | online | 32 / 33 | 306 | 218 | 59 / 120 / 54 | 300 |
| 51008 | trained_frozen | 41 / 60 | 304 | 221 | 60 / 120 / 52 | 300 |
| 51008 | ablated_frozen | 40 / 48 | 300 | 167 | 60 / 120 / 48 | 300 |
| 51008 | rule_based | 31 / 30 | 300 | 223 | 66 / 120 / 48 | 300 |

The existing rule-based arm is a strong baseline. Against it, the online arm has **no Pareto-dominating seed** on the same unweighted physical components: rule-based outcomes dominate online in seeds `51003`, `51006`, and `51007`; the other five seeds are tradeoffs. In particular, the one online-versus-frozen favorable vector at seed `51007` is not evidence that learning beats the ordinary policy. That subject acquired 302 water, paid 214 movement energy, and ended with energy 63 online, versus 305 water, 198 movement energy, and energy 65 rule-based.

| Seed | Online − rule water | Online − rule movement energy paid | Online − rule terminal energy | Relation |
|---:|---:|---:|---:|---|
| 51001 | +2 | +28 | −4 | tradeoff |
| 51002 | +8 | +13 | −7 | tradeoff |
| 51003 | −3 | +10 | −2 | rule-based dominates |
| 51004 | −3 | +20 | +6 | tradeoff |
| 51005 | +2 | +15 | +3 | tradeoff |
| 51006 | 0 | 0 | −2 | rule-based dominates |
| 51007 | −3 | +16 | −2 | rule-based dominates |
| 51008 | +6 | −5 | −7 | tradeoff |

## Interpretation and scope

This lab demonstrates that the same individual’s post-reversal learning history changed a later, ordinary greedy choice and that real resolver outcomes followed. Against trained-frozen, outcomes were one favorable vector, six tradeoffs, and one frozen-dominant loss. The rule-based arm provides context and does not establish that learning is generally superior to the existing policy. The ablated arm is the declared counterfactual that clears only learned values at the fork.

The finding is limited to an existence example in one fixed small water-competition fixture with an explicit 500-tick life cap. It does not establish typical benefit, long-term survival, lifespan extension, population fitness, inheritance, ecological balance, or broad learning efficacy. The rule-based comparison shows no policy-level Pareto win in this sample. Tick counts are accelerated simulation time. No live default, biological policy, or Rust core behavior was changed.

## Follow-up design question

The selected site followed the learned table’s stored value, while ordinary perception also exposed current amount and path length. In the seed-51007 root audit, the candidates were A `(0,2)` with amount 1 and path length 3, and B `(4,2)` with amount 12 and path length 1. Online selected B; the frozen learned table selected A. A future, separately preregistered candidate could test a residual or reliability-adjusted memory score against current visible resource amount and route cost. This report proposes no implementation, parameter change, or new run.

## Preserved evidence

- Preregistration: [LIFELONG_WATER_PROTOCOL.md](../LIFELONG_WATER_PROTOCOL.md).
- Original calibration capture and failed manifest: [calibration directory](../../data/experiments/lifetime-lab-v2-calibration-20261023/).
- First zero-tick calibration reanalysis: [reanalysis 1](../../data/experiments/lifetime-lab-v2-calibration-reanalysis-20261023/reanalysis.json).
- Source-bound second zero-tick calibration reanalysis: [reanalysis 2](../../data/experiments/lifetime-lab-v2-calibration-reanalysis2-20261023/reanalysis.json).
- Formal manifest, raw trace, summary, runner output, and independent focal example audit: [formal directory](../../data/experiments/lifetime-lab-v2-20261023/).
- Machine-readable result: [LIFELONG_WATER_V2.json](LIFELONG_WATER_V2.json).
