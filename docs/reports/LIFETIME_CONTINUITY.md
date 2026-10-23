# Lifetime continuity and reversal — bounded result

**Result:** individual identity/state continuity and exact online checkpoint
replay passed in all eight seeds. The study does not demonstrate a lifespan or
learning advantage. A few post-reversal water-learning updates occurred, but
online and frozen branches had identical decisions and physical outcomes in
these eight lifetimes.

## Design and evidence

One herbivore subject was created at age zero per seed (44001–44008), kept as
the same actor, and observed through the registered resource/predator route
reversals. No replacement or body/age reset was made after death. At completed
tick 52, each surviving subject was checkpointed; its post-checkpoint history
was continued uninterrupted, restored into an online branch, and cloned into a
counterfactual with only water-learner updates disabled. The restored branch
had to match the uninterrupted branch at every post-checkpoint tick.

The registered physical ceiling was 1,664 ticks (8 × [104 uninterrupted + 52
restored + 52 frozen]). The run completed normally at 1,529 physical ticks in
0.653 seconds, with empty stderr. Its manifest records clean source revision
`ba71b7fd63268201ed7889f9a3c572c6c0dae215`, binary SHA-256
`257bcf6aa48e74a85201ae5ad3890d9409949c43abf509dbd26f8eda87fc3a93`, and
protocol SHA-256
`4acef82df8d66dc3fd84b8682f6cc8534fe9ed78221ebaa4ba2ab216045d0ca1`.
Evaluation raw SHA-256 is
`5549f6b0bad9fc75a5216d523b51c184dbd81dc83479e04b6fb62008e59130ae`;
summary SHA-256 is
`42bfa131fb64de14e47e90a7fb9ddacee1f4cdaf23251c227d7e5d77df96cdb1`.
The immutable probe, source-file hashes, raw actor traces and full states are
preserved under `data/experiments/lifetime-lab-v1-20261023/`.

## Identity, continuity and death audit

For every seed, actor ID 1 began at age zero, reached the tick-52 checkpoint at
age 52, and its age advanced continuously in each branch. All eight checkpoint
states exactly equal their restored-online starting states, including RNG. The
resumed-online trace matches the uninterrupted trace in events, decisions,
complete serialized state, controller/intervention state and RNG on every
post-checkpoint tick. The frozen checkpoint has the same body/world/RNG and
learner table as the online checkpoint; only
`learner.update_enabled` is false.

| Seed | End status | Last age | End/death tick | Death cause | Online / resumed / frozen ticks |
|---:|---|---:|---:|---|---:|
| 44001 | died after checkpoint | 95 | 95 | Starvation | 95 / 43 / 43 |
| 44002 | died after checkpoint | 97 | 97 | Dehydration | 97 / 45 / 45 |
| 44003 | died after checkpoint | 103 | 103 | Starvation | 103 / 51 / 51 |
| 44004 | reached tick ceiling alive | 104 | 104 | — | 104 / 52 / 52 |
| 44005 | died after checkpoint | 99 | 99 | Starvation | 99 / 47 / 47 |
| 44006 | died after checkpoint | 94 | 94 | Dehydration | 94 / 42 / 42 |
| 44007 | died after checkpoint | 101 | 101 | Dehydration | 101 / 49 / 49 |
| 44008 | died after checkpoint | 94 | 94 | Dehydration | 94 / 42 / 42 |

Seven subjects died naturally before the 104-tick ceiling, from starvation or
dehydration. Seed 44004 remained alive at the cutoff with health 1 and energy 0.
No subject received predator damage in the recorded windows. These outcomes
show that one subject's identity, body, age and learner state remained attached
to that individual through checkpointing and continuation. They do not show an
age-related fitness or capability advantage, nor long-term survival.

Across ticks 53–78, movement cost ranged from 8 to 32 energy per subject and
health ended between 60 and 99; no damage was attributed to the predator.
Across ticks 79–104, movement cost was zero in every history while health fell
further, consistent with late depletion rather than a predator injury effect.
The exact per-window health, energy, water and movement values are retained in
the JSON audit.

## Resource reversal and learner observations

The first 52-tick window yielded 10–30 water per individual and online learner
updates in all eight seeds. After the tick-52 reversal, water acquisition and
learner updates were sparse but not absent. The table lists water acquired in
the two later windows and online learner-table deltas. Resumed-online deltas
match uninterrupted-online deltas. Frozen branches acquire the same water but
hold learned values and visits fixed.

| Seed | Ticks 53–78: online water / table delta | Ticks 79–104: online water / table delta |
|---:|---|---|
| 44001 | 0 / none | 0 / none |
| 44002 | 0 / none | 0 / none |
| 44003 | 0 / none | 0 / none |
| 44004 | 0 / none | 30 / (x=5,y=3: +1 value, +3 visits) |
| 44005 | 0 / none | 20 / (x=5,y=3: +99 value, +2 visits) |
| 44006 | 10 / (x=5,y=3: +95, +1) | 10 / (x=5,y=3: +1, +1) |
| 44007 | 0 / none | 0 / none |
| 44008 | 10 / (x=5,y=3: +95, +1) | 10 / (x=5,y=3: +1, +1) |

Thus four of eight subjects had post-checkpoint water acquisitions and online
table updates; the other four had no post-checkpoint water/update. The frozen
counterfactual confirms the learner updates themselves: it recorded the same
body, resource, health, energy, movement, events and decisions as online after
the checkpoint, while leaving learner values/visits unchanged. Across all
eight paired histories, normalizing only learner values/visits and the update
flag makes full world states identical; action choices and physical events are
identical as recorded. The study therefore provides no observed behavioral or
survival benefit from these updates. The few updates do not establish robust
adaptation: water opportunities were limited, seven lifetimes ended before the
registered ceiling, and the fixed observations cannot establish a general
learning advantage.

Before the checkpoint, acquired water by seed was 20, 20, 30, 20, 20, 10, 30
and 10 respectively. Initial-window learning establishes that the table could
update, while the later windows show only sparse post-reversal updating. This
is not a transfer test across multiple independent lifetimes; each animal had
one continuous life history, and the frozen branch is a same-checkpoint
counterfactual rather than another animal.

## Interpretation limits

The demonstrated result is **continuity and deterministic resume**, not a
lifespan-learning acceptance. The registered 104-tick histories are short,
seven end from resource-related causes, and one merely reaches the cutoff.
Observed updates are few and do not change decisions or physical outcomes in
this sample. No age advantage, broad adaptation, population fitness, or
long-term ecological benefit is supported. The 1,529 physical ticks are
accelerated lab execution and do not represent real time lived or a throughput
measurement for a sustained ecosystem.

The machine-readable per-seed and per-window audit, identity/state replay checks
and artifact hashes are in [LIFETIME_CONTINUITY.json](LIFETIME_CONTINUITY.json).
The [preregistration](../LIFETIME_CONTINUITY_PROTOCOL.md),
[manifest](../../data/experiments/lifetime-lab-v1-20261023/manifest.json),
[raw traces](../../data/experiments/lifetime-lab-v1-20261023/raw.jsonl) and
[summary](../../data/experiments/lifetime-lab-v1-20261023/summary.json) remain
available with the immutable probe in the experiment directory.
