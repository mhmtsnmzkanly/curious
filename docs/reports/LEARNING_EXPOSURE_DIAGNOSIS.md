# Historical learning exposure diagnosis

**Purpose.** This read-only diagnosis audits the preserved lifetime water-choice
and herbivore refuge labs for opportunities where a learned estimate could affect
a choice. It is a mechanism/exposure audit, not an acceptance result or evidence
of survival benefit. Counts below come from the old raw traces; no simulation was
run for this report.

## Lifetime water-choice lab

The formal raw file contains eight uninterrupted single-subject histories
(seeds 44001–44008), plus checkpoint-resumed and frozen branches. For the primary
count, I used only each seed's uninterrupted history. A **fresh multi-option
water choice** means the subject's decision reason was `LearnedWater`, its prior
state had no active water episode targeting any currently perceived candidate,
and the recorded ordinary perception held at least two water candidates. The
prior learner table was read from the previous tick's complete state. Candidate
values use the learner's implemented default of 200 when a site has no table
entry.

There were **27** fresh multi-option choices across the eight uninterrupted
histories: 15 in ticks 1–52, 2 in ticks 53–78, and 10 in ticks 79–104. At 14 of
those points, the candidate Q estimates differed. At 9, the learner's
deterministic exploitation winner differed from the shortest-path reference
winner. That comparison is **not** the rule-based policy: the actual rule-based
water chooser ranks candidates with
`foraging::value(amount, path_length, life, true)`. This report makes no claim
that the learned policy selected a different site than the actual rule-based
policy.

The paired post-checkpoint comparison recorded no decision, resolver-event,
physical-state, or RNG differences between resumed-online and frozen-update
branches in any seed after excluding the learner table and update flag, which
are the intended intervention. Four of eight uninterrupted subjects acquired
water and updated their table after tick 52, for 90 units total and 9 visit
increments. The post-checkpoint value changes therefore did not create an
observed choice difference in the paired histories. Updating was enabled in the
resumed branch; freezing updates did not freeze the ordinary exploration policy.

The resource budget constrained the opportunity to express a changed choice.
All eight uninterrupted subjects reached zero energy during ticks 53–78. Their
movement energy in that window ranged from 8 to 32; movement energy was zero for
all subjects in ticks 79–104. Seven subjects died naturally before tick 104
(starvation or dehydration); the eighth reached tick 104 alive with health 1 and
energy 0. The post-reversal windows produced only 12 fresh multi-option water
choices in total. These observations are consistent with resource depletion
limiting repeated, mobile choice opportunities; they do not prove that depletion
alone caused the identical paired decisions.

## Herbivore refuge lab

The formal refuge raw file contains eight paired histories. In each branch, 94
`SeekShelter` decisions had at least two visible, unoccupied shelter endpoints.
Reconstructing the feature's current context from the recorded perception and
pre-decision body state (threat identity/species, bearing and distance, energy
and stamina quartiles, exact visible endpoint set) found **zero** decisions with
even one matching completed outcome record. Therefore none had two eligible
learned candidates, and none had a data-driven learned ordering between
alternatives. The comparison rule had no matching evidence to rank at these
choice points.

Across each branch, the raw observer ledger recorded 65 episode closures, 52
verified safe arrivals and 13 censored or unresolved closures. The sole repeated
matching visible context reported by the raw audit was seed 45002 at ticks 66
and 82; both closures were censored (`threat_lost` and `died`). All 16 subject
histories ended in natural death between ticks 62 and 82. These results show
recorded refuge outcomes but do not establish safer choices or reduced
mortality. The very specific context key and the short, resource-depleting
histories left no completed outcome reusable at a later multi-option choice.

## Implications for a next bounded water lab

These are design constraints inferred from the audited histories, not measured
acceptance thresholds:

- Sustain ordinary metabolism with renewable food and water, and account for
  measured movement/interaction costs as well as basal costs. In the existing
  rules, basal energy and water costs are each 1 per tick, and a drink action
  yields at most 10 units.
- Keep at least two non-empty, walkable water choices simultaneously visible at
  repeated decision points, with observable differences that can reverse over
  time. Record only ordinary perception as policy input.
- Ensure food renewal is close and reliable enough that the subject retains
  energy to travel between water choices. Do not use body resets, rescue, or
  replenishment invisible to the resolver.
- Place any distant reserve outside ordinary perception (or behind existing
  ordinary occlusion) so the subject cannot use hidden current/future
  availability. Do not let the reserve accidentally become a third visible
  candidate at registered choice points.
- Preregister counts of fresh multi-option decisions, completed learning
  updates, pre-decision candidate estimates, and later choices after those
  updates. Report actual rule-based, online-learning, trained-frozen, and any
  ablation branches separately; keep exploration semantics explicit.

The water system takes the `SeekWater` decision from ordinary thirst behavior,
then the learner ranks visible candidates using its bounded table and 10%
exploration. A higher-priority threat or food decision suppresses that water
choice. This makes a predator unnecessary for the water-learning question and a
potential source of fewer eligible water decisions.

## Provenance and limits

The lifetime raw traces are
[`lifetime-lab-v1-20261023/raw.jsonl`](../../data/experiments/lifetime-lab-v1-20261023/raw.jsonl),
source revision `ba71b7fd63268201ed7889f9a3c572c6c0dae215`, protocol SHA-256
`4acef82df8d66dc3fd84b8682f6cc8534fe9ed78221ebaa4ba2ab216045d0ca1`, and raw
SHA-256 `5549f6b0bad9fc75a5216d523b51c184dbd81dc83479e04b6fb62008e59130ae`.
The refuge raw traces are
[`herbivore-lab-v1-20261023/raw.jsonl`](../../data/experiments/herbivore-lab-v1-20261023/raw.jsonl),
source revision `60ab1519022bb50dfe91977892b67be3da178e9f`, raw SHA-256
`deee6bbcb78f2af731ef20a476cd43c3f3c8d127bf7eb76a00d14d734eb7f10a`.
The refuge manifest's original validator status is failed; the preserved
analysis-only reanalysis validated the same raw file without additional
simulation ticks.

These bounded labs establish specific persistence and event-accounting facts.
They do not establish general learning benefit, a lifetime advantage, or
population-level/ecological benefit. Historical seed sets are already observed
and must not be presented as unseen acceptance seeds.
