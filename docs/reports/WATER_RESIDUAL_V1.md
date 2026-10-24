# V17 continuous-life water residual lab — bounded result

**Result:** calibration passed its preregistered opportunity gate. The formal
eight-seed phase completed all 11,200 physical ticks. Three seeds produced
independently audited causal mechanism witnesses; the preregistered bounded
utility criterion did **not** pass against either trained-frozen or rule-based.
The aggregate results include adverse tradeoffs and do not support a general
performance claim.

## Provenance and run integrity

The source-integrated candidate is revision `506ed6f4b3ec3bab8bb65ecfdb8ea3bed2797602`.
Calibration seed `52099` completed once on 2026-10-24 UTC with 1,400 physical
ticks, exit code 0, empty stderr, and no output-limit violation. Its preparation
directory retains the `20261023` protocol identifier; the actual run timestamp
is in the immutable manifest. Root verified the artifact hashes and separately
audited one completed residual-mean update at tick 433: source `(4,2)`, mean
`12 → 11`, matched water `8`, paid movement `3`, sample residual `0`.

| Artifact | Identity |
| --- | --- |
| Source revision | `506ed6f4b3ec3bab8bb65ecfdb8ea3bed2797602` |
| Source SHA-256 | `09e4890d23df8ea4267e032d440b73743493b40040f48207d344eb061b6ecf49` |
| Probe SHA-256 | `0c0f1aebc82baf75dc0eac925289e71741e770df46b3ba6d637a8fc59a1f7213` |
| Protocol SHA-256 | `0410576adef7474ec23dd9e92a73eb5c9306f87c0f6450696c3ca5e424a22bc0` |
| Runner SHA-256 | `8869d804429295830f0c6b847a216cdad3696e20c07e48dd0382cfb8b65dc3eb` |
| Calibration raw SHA-256 | `12f3ab2edcf092930af7c385dec457761294b7a48d88a4f4ac61962ee9beae39` |
| Calibration summary SHA-256 | `9b1305b8586baf5aecf4b095ee2f582194aa2724df640fca75b9d3ca854ada4c` |
| Calibration manifest SHA-256 | `7cb7bd7f857bcea7b100e0d613ffd97ece675a6cfa232ded5cc7b5c44f2debd3` |
| Formal raw SHA-256 | `c10e7d55431f6c10e96acf1f84d7c4295d67208b54ab6ab788ae210d30fcf78c` |
| Formal summary SHA-256 | `255aa41c6abd12f37fce07561799c142c043580f20fe7c63ff9d0f26db242d52` |
| Formal manifest SHA-256 | `d95a37b0f363e2f2172c02812c32768b0318e75a980498539ef4d46a559520c4` |
| Root formal audit SHA-256 | `9c25548db4bd66219d2f14c8c891fafa152c76ea48c79fa8036a39df2ed30601` |

The calibration record used the registered protocol/schema and a phase-only
seed scan: seed `52099` had no prior matches among 1,068 scanned archive text
files. Limits were 300 seconds, 1 GiB child address space, 1 GiB raw output,
and 64 MiB per seed record. The exact immutable manifest, raw capture, summary,
and root's independent mean-update audit are preserved under
`data/experiments/water-residual-v1-calibration-20261023/`.

Formal seeds `52001`–`52008` completed once on 2026-10-24 from
`00:41:33Z` to `00:41:44Z`: 11,200 ticks, exit code 0, empty stderr, no
output-limit violation, and no runner error. The raw file is 164,554,619 bytes.
Root verified the artifacts and independently
recomputed all 32 physical vectors from the raw record; all focal branches
reached tick 500 alive. The formal directory is
`data/experiments/water-residual-v1-20261023/`.

Before registration, two unregistered seed-123 engineering runs each produced
1,400 ticks. Across all 1,400 corresponding frames, pre/post State, RNG,
resolver events, and decisions matched; the second added closure-audit fields.
The mechanism audit did not qualify. These 2,800 engineering ticks are not
evaluation evidence and did not provide an acceptance result. The records are
retained at `data/experiments/water-residual-v1-engineering-20261023/` as
`unit-seed123-v1.jsonl` and `unit-seed123-v2.jsonl`; their original temporary
paths were outside the canonical archive.

## Registered comparison

The fixture is a fixed predator-free 5×5 World with ordinary AI actors. One
continuous 200-tick prefix trains the focal subject, and the tick-200 checkpoint
forks into four 300-tick suffixes. There are no body/age resets, offspring,
replacement actors, rescue, forced choices, or external refill. Water renewal
rates reverse before tick 201. Bodies, policy, environment, and resolver remain
fixed as registered.

| Arm | Fork treatment |
| --- | --- |
| `rule_based` | Focal learner disabled; ordinary rule policy remains active. |
| `naive_online` | Clear only residual entries; preserve legacy values and active episode; update online. |
| `trained_online` | Keep prefix-trained tables and update online. |
| `trained_frozen` | Keep the same trained tables; freeze legacy and residual updates. |

The rule-based arm has no learner epsilon draw; the three learner-enabled arms
use 10% epsilon exploration. Disabling the focal learner also changes later
RNG consumption. The comparison against `rule_based` is therefore an
end-to-end policy comparison, not an isolated causal estimate of residual
learning alone. The trained-online versus trained-frozen mechanism check
matches pre-choice RNG and the other physical inputs, so its three witnesses
support a narrower, state-matched choice effect.

The primary mechanism comparison is the first fresh greedy
`trained_online`/`trained_frozen` choice with matched physical state,
perception, and RNG after normalizing only the learner tables and registered
update setting. It also requires a different own resolver event after the
choice. This mechanism test is separate from utility. Utility is evaluated only
for `trained_online`: all four arms must complete all 300 suffix ticks alive;
its unweighted aggregate physical vector must Pareto-dominate both `rule_based`
and `trained_frozen`, with no per-seed survival or health deficit. Death before
the horizon remains a biological negative and prevents a utility claim. Naive
online is descriptive. No weighted score or general statistical claim is used.

## Calibration result

Calibration passed all eight preregistered opportunity checks. Both actors were
alive at the fork, the trained residual table was nonempty, and the prefix had
20 fresh choices with two positive visible sources while the body was nonfull.
There were 20 matched completed drinks and 20 learner closures in the prefix.
In the trained-online suffix there were 35 such fresh choices, 35 completed
drinks, 35 `Reached` closures, and at least one residual sum/count update. The
independent tick-433 audit verified that an own completed update changed the
selected source's exact mean; the opportunity gate therefore passed.

| Seed 52099 | Ticks | Water acquired | Paid movement energy | Survival | Terminal energy / water / health |
| --- | ---: | ---: | ---: | ---: | --- |
| Prefix to tick 200 | 200 | 184 | 139 | 200 observed | focal 67 / 44 / 120 |
| `rule_based` | 300 | 305 | 234 | 300 | 67 / 49 / 120 |
| `naive_online` | 300 | 305 | 238 | 300 | 57 / 49 / 120 |
| `trained_online` | 300 | 305 | 238 | 300 | 57 / 49 / 120 |
| `trained_frozen` | 300 | 305 | 238 | 300 | 57 / 49 / 120 |

All actors survived their observed calibration horizons. There were no
censored episode closures. The mechanism comparison did not qualify in this
calibration seed: no qualifying greedy divergence was recorded. Calibration
establishes that the registered preference-relevant learning opportunity was
present; it is not utility evidence and is not counted as formal evidence.

## Formal causal mechanism

The registered same-state greedy choice plus different own resolver event gate
passed for three seeds: `52002` at tick 247, `52003` at tick 415, and `52006` at
tick 241. Root's raw audit independently matched full normalized prestate,
RNG, perception, and body, recomputed the greedy scores, and confirmed that the
online and frozen targets differed and their own resolver events differed.
Online selected `(0,2)` and frozen selected `(4,2)` in each witness. The other
five seeds did not yield a qualifying witness. This is mechanism evidence in
three fixed runs, not a utility result.

## Formal physical vectors

Every row below is a 300-tick living suffix. Each vector is `water acquired /
paid movement energy / terminal energy-water-health / observed survival ticks`.
Choices and drinks are included as exposure context, not as utility components.
All branches had zero focal deaths; no terminal value was imputed.

| Seed | Arm | Choices / drinks | Physical vector | Censor closures |
|---:|---|---:|---|---:|
| 52001 | `rule_based` | 31 / 31 | 298 / 183 / 60-49-120 / 300 | 0 |
| 52001 | `naive_online` | 37 / 38 | 296 / 208 / 65-47-120 / 300 | 0 |
| 52001 | `trained_online` | 37 / 38 | 296 / 208 / 65-47-120 / 300 | 0 |
| 52001 | `trained_frozen` | 37 / 38 | 296 / 208 / 65-47-120 / 300 | 0 |
| 52002 | `rule_based` | 37 / 36 | 302 / 213 / 62-54-120 / 300 | 0 |
| 52002 | `naive_online` | 37 / 38 | 302 / 206 / 65-54-120 / 300 | 0 |
| 52002 | `trained_online` | 34 / 34 | 294 / 204 / 63-46-120 / 300 | 0 |
| 52002 | `trained_frozen` | 32 / 33 | 296 / 210 / 61-48-120 / 300 | 0 |
| 52003 | `rule_based` | 35 / 35 | 298 / 202 / 62-50-120 / 300 | 0 |
| 52003 | `naive_online` | 34 / 34 | 301 / 207 / 69-53-120 / 300 | 1 |
| 52003 | `trained_online` | 36 / 36 | 301 / 206 / 64-53-120 / 300 | 1 |
| 52003 | `trained_frozen` | 34 / 34 | 295 / 206 / 64-47-120 / 300 | 1 |
| 52004 | `rule_based` | 37 / 35 | 292 / 209 / 62-46-120 / 300 | 0 |
| 52004 | `naive_online` | 34 / 35 | 300 / 192 / 59-54-120 / 300 | 0 |
| 52004 | `trained_online` | 35 / 35 | 297 / 174 / 61-51-120 / 300 | 0 |
| 52004 | `trained_frozen` | 35 / 35 | 297 / 174 / 61-51-120 / 300 | 0 |
| 52005 | `rule_based` | 33 / 34 | 298 / 221 / 67-49-120 / 300 | 0 |
| 52005 | `naive_online` | 34 / 34 | 295 / 180 / 60-46-120 / 300 | 0 |
| 52005 | `trained_online` | 34 / 34 | 295 / 180 / 60-46-120 / 300 | 0 |
| 52005 | `trained_frozen` | 34 / 34 | 295 / 180 / 60-46-120 / 300 | 0 |
| 52006 | `rule_based` | 38 / 37 | 296 / 206 / 60-48-120 / 300 | 0 |
| 52006 | `naive_online` | 38 / 39 | 295 / 208 / 68-47-120 / 300 | 0 |
| 52006 | `trained_online` | 38 / 38 | 297 / 218 / 60-49-120 / 300 | 0 |
| 52006 | `trained_frozen` | 34 / 34 | 296 / 225 / 65-48-120 / 300 | 0 |
| 52007 | `rule_based` | 39 / 39 | 301 / 185 / 66-47-120 / 300 | 0 |
| 52007 | `naive_online` | 32 / 34 | 298 / 231 / 68-44-120 / 300 | 0 |
| 52007 | `trained_online` | 32 / 34 | 298 / 231 / 68-44-120 / 300 | 0 |
| 52007 | `trained_frozen` | 32 / 34 | 298 / 231 / 68-44-120 / 300 | 0 |
| 52008 | `rule_based` | 31 / 30 | 298 / 200 / 65-48-120 / 300 | 0 |
| 52008 | `naive_online` | 36 / 36 | 296 / 230 / 67-46-120 / 300 | 0 |
| 52008 | `trained_online` | 36 / 36 | 296 / 230 / 67-46-120 / 300 | 0 |
| 52008 | `trained_frozen` | 36 / 36 | 296 / 230 / 67-46-120 / 300 | 0 |

Pareto relations use the six preregistered physical components, with paid
movement energy oriented lower-is-better. `Identical` means all six components
matched. `Tradeoff` means neither arm dominated the other.

| Seed | `trained_online` vs rules | `trained_online` vs frozen | `naive_online` vs rules (descriptive) |
|---:|---|---|---|
| 52001 | Tradeoff | Identical | Tradeoff |
| 52002 | Tradeoff | Tradeoff | Naive dominates |
| 52003 | Tradeoff | Online dominates | Tradeoff |
| 52004 | Tradeoff | Identical | Tradeoff |
| 52005 | Tradeoff | Identical | Tradeoff |
| 52006 | Tradeoff | Tradeoff | Tradeoff |
| 52007 | Tradeoff | Identical | Tradeoff |
| 52008 | Tradeoff | Identical | Tradeoff |

No trained-online seed Pareto-dominated the strong rule policy: all eight were
tradeoffs. Against trained-frozen, online dominated once (`52003`), traded off
twice (`52002`, `52006`), and was identical five times. Naive online is
descriptive: it dominated rules once (`52002`) and traded off in seven seeds.

## Aggregate result and registered utility decision

Across the eight seeds, all four arms reached the 300-tick ceiling in all
branches. Aggregate vectors sum each component across seeds without weights:

| Arm | Water acquired | Paid movement energy | Terminal energy | Terminal water | Health | Survival ticks |
|---|---:|---:|---:|---:|---:|---:|
| `trained_online` | 2,374 | 1,651 | 508 | 382 | 960 | 2,400 |
| `trained_frozen` | 2,369 | 1,664 | 511 | 377 | 960 | 2,400 |
| `rule_based` | 2,383 | 1,619 | 504 | 391 | 960 | 2,400 |
| `naive_online` (descriptive) | 2,383 | 1,662 | 521 | 391 | 960 | 2,400 |

`trained_online − rule_based` is `−9` water, `+32` paid movement energy,
`+4` terminal energy, `−9` terminal water, `0` health, and `0` survival.
`trained_online − trained_frozen` is `+5` water, `−13` movement energy,
`−3` terminal energy, `+5` terminal water, `0` health, and `0` survival.
Each comparison contains adverse components, so neither aggregate vector
Pareto-dominates its comparator. The preregistered bounded-utility result is
**false** against both strong rules and trained-frozen. There was no per-seed
health or survival deficit, but that no-harm condition alone does not meet the
registered utility rule.

Naive online is not a primary candidate. Descriptively, its aggregate
comparison to rules has `0` water, `+43` movement energy, `+17` terminal energy,
`0` terminal water, and unchanged health/survival; it does not dominate rules.
Its aggregate comparison to trained-frozen is `+14` water, `−2` movement
energy, `+10` terminal energy, `+14` terminal water, and unchanged health and
survival. That descriptive relation does not change the registered
trained-online criterion.

## Scope

These results show a resolver-confirmed, state-matched greedy choice effect in
three of eight fixed seeds, while the preregistered aggregate utility test
failed against both controls. The rule-based result compares complete policies;
it does not isolate residual learning from exploration and subsequent RNG
consumption. The findings are limited to one small fixture and eight
descriptive seeds. They do not establish population fitness, longevity,
ecological balance, inheritance, or broad learning efficacy. Accelerated ticks
are simulation ticks, not elapsed real lifetime. No automatic deployment or
change to the live policy follows from this report.
