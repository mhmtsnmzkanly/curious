# Bounded ecosystem learning work

This is a six-phase, modular research program. Each feature remains opt-in and
default-off, uses resolver/perception evidence available to the acting
individual, and requires a bounded preregistered lab plus a separate review.
No phase implies ecological acceptance or enables deployment. Luna6 owns code,
tests, lab runner, and analysis tooling; root owns protocol approval, source
commits, evaluation review, and operational installation. No commits, long
batches, `cargo clean`, pushes, live-world edits, or frozen-release edits are
performed by the coding agent.

| Phase | Feature and minimum evidence | State |
|---|---|---|
| 1 | Opt-in hunt injury-risk history, causally tied to resolver-confirmed prey defense and a choice rule with unknown evidence kept unknown. Preregistered six-arm lab compares food, injury, deaths/final health, and energy. | V3 source and bounded runner completed at `67a4e14`; 131 Rust/77 Python and disabled state/RNG fidelity passed. Bounded evaluation completed; risk policy did not reduce injury outcomes (see `reports/HUNT_LAB_V3.md`/JSON). Feature stays default-off; no tuning or deployment. |
| 2 | Same individual retains its actual identity, age, body history, and learner through resource/predator reversal and checkpoint/resume. Audit raw actor state and RNG equality across uninterrupted/resumed branches. | Bounded run completed at source `ba71b7f`: 1,529/1,664 physical ticks; all eight checkpoint/resume states and RNG matched. Seven subjects died naturally; four had sparse post-reversal water/table updates. Online and frozen had identical decisions and physical outcomes. The result supports continuity only, not a lifespan or learning advantage; see `reports/LIFETIME_CONTINUITY.md`/JSON. |
| 3 | Extend existing danger memory for herbivore escape/shelter choices after observed threat. Measure resolver-paid movement, own received damage, safe arrival, censoring, and blocked/occupied destinations; do not reveal future threat positions. | Root-approved V16 core committed at `970b974`; 153 Rust tests, 11 refuge tests, 2 physical patrol tests, HTTP/SIGKILL control checks, and 100/200-tick frozen-source/RNG fidelity pass. The bounded example adds the same observation-only episode ledger to both arms because feature-off baseline does not emit learning events. The preregistration now includes one 45999 two-arm/200-tick serde-measurement calibration before the 45001–45008 held-out/1,600-tick batch; no calibration or formal result has started. Runner/validator are under review; only measurement/serialization fixes may follow calibration, and the feature stays default-off. |
| 4 | Population-independent spatial/temporal resource schedule with deterministic site offsets and a closed-feature no-RNG equivalence check. | V15 schedule mechanism check passed. The preregistered 80-world / 160,000-tick screen had zero reproducing descendant lineages in every arm; no relative fitness ranking is supported. Exact repeated calibration raw hashes matched before evaluation. Keep schedules opt-in; see `reports/ECOLOGY_TRAITS.md`/JSON. |
| 5 | Individual decision timeline for seen/remembered evidence, selected action, observed cost, and resulting update; compare first/last genuinely similar subjective encounters. | Implemented as the read-only documentary viewer in source `4c2cef7`; independent browser and 75 Python UI tests passed. The current view presents observed history rows, not a claim of complete lifetime cognition. Read-only deployment health showed 3 individual-history rows and zero world restarts; no Rust server upgrade. |
| 6 | Separate inherited-allele lab for sight, speed, and endurance. Charge expressed traits through live perception/resolver/metabolism budgets, then measure survival and reproduction costs. | Separate V15 paid-trait arms were included in the ecology-traits preregistration; requested and actually paid vision/stamina costs were measured. The combined vision+stamina arm had no births, and no arm produced a reproducing descendant lineage. No trait advantage or evolutionary benefit is accepted; keep the feature default-off and see the ecology-traits report. |

Each phase produces the smallest implementation that can answer its stated
question, tests its causal measurement, and pins commands, seeds, source and
protocol SHAs, manifest schema, tick ceiling, and stop rule before evaluation.
No parameter sweep or tuning after seeing outcomes. If a lab cannot identify
the claimed effect with current measurements, narrow the claim or design a
separate preregistered lab before changing the feature.

## Phase 1 protocol boundary

V3 counts an exposure only for a completed observed-policy closure with attack
energy and exact incoming-damage attribution to the episode's visible prey.
A lab-forced unfinished pursuit is censored, not a safe observation. Risk
evidence is independent of food/action-cost Q; non-exploration choices compare
observed candidates lexicographically by injury rate, mean prey-defense damage,
then existing Q and rule tie-breaks. Unknown candidates are excluded only after
some candidate risk is known; the existing exploration branch can still sample
unknown contexts. The frozen risk arm and same-table risk-disabled ablation
must begin with identical food and risk tables.

Root committed the preregistration before source implementation. The V3 run
completed in 12 seconds: 10,240 trials and 99,729 physical ticks. The report
and summary are preserved under `reports/HUNT_LAB_V3.md` and `.json`; original
raw output, probe, and manifest remain under
`data/experiments/hunt-lab-v3-20261023/`. Results reject the preregistered
risk-benefit hypothesis for this policy: on the defensive normal window,
risk-online had 248/320 injured trials and 3,558 prey-defense damage versus
215/320 and 3,276 for plain experienced-online, with higher energy spent
(7,505 vs 6,043). After reversal, risk-online had 246/320 injured trials and
1,476 damage versus 218/320 and 1,308. Keep risk choice disabled in live
settings; do not tune these parameters based on this result.

The bounded completion timer is installed and active
(`curious-hunt-lab-completion.timer`); root owns systemd changes. Its actual
V3 terminal poll queued exactly once (queue ID
`01a16693-86a7-7953-91c2-7b9656de2478`, one attempt; root verified the queue
database and retained completion journal/dedup evidence). It uses
a frozen helper, external runtime configuration, explicit watched run paths,
terminal manifest and content-hash checks, durable outbox, and native
`codex queue` delivery to the configured user/thread. The retired V1 helper/config remains as V3 delivery evidence. The frozen V2
helper is installed with three active completion timers for lifetime,
herbivore, and ecology-traits labs. Its source and 12-test suite accept only
explicit bounded lab schemas, one exact watched run, and an allowlisted
immutable artifact filename (`hunt-lab-probe` or `lab-probe`). Runtime configs
and frozen SHA are outside git and root-owned. New lab runs must use the
matching schema, watched path, and probe filename.
Timer completion only queues a message; root must recheck run identity and
evidence.

The live worlds on 7878–7883, frozen/bin releases, and historic/raw lab
archives are outside this work. Biology defaults, closed-state behavior and
RNG order are acceptance constraints. New saved-state fields require an
explicit version and migration; learned experience never transfers to offspring.
