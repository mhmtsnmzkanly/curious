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
| 2 | Same individual retains its actual identity, age, body history, and learner through resource/predator reversal and checkpoint/resume. Audit raw actor state and RNG equality across uninterrupted/resumed branches. | Prepared preregistration `LIFETIME_CONTINUITY_PROTOCOL.md` is pending root review/commit; no source changes or evaluation before that commit. |
| 3 | Extend existing danger memory for herbivore escape/shelter choices after observed threat. Measure real movement cost, own received damage, escape success, and blocked/occupied destinations; do not reveal future threat positions. | Prepared preregistration `HERBIVORE_DANGER_PROTOCOL.md` is pending root review/commit; saved-state V16 must integrate after phase-4 V15. |
| 4 | Population-independent spatial/temporal resource schedule with deterministic site offsets and a closed-feature no-RNG equivalence check. | `ecology_traits_luna6` worktree; pending phase 3 coordination/review. |
| 5 | Individual decision timeline for seen/remembered evidence, selected action, observed cost, and resulting update; compare first/last genuinely similar subjective encounters. | Implemented as the read-only documentary viewer in source `4c2cef7`; independent browser and 75 Python UI tests passed. The current view presents observed history rows, not a claim of complete lifetime cognition. Read-only deployment health showed 3 individual-history rows and zero world restarts; no Rust server upgrade. |
| 6 | Separate inherited-allele lab for sight, speed, and endurance. Charge expressed traits through live perception/resolver/metabolism budgets, then measure survival and reproduction costs. | `ecology_traits_luna6` worktree; pending phase 5 coordination/review. |

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
`codex queue` delivery to the configured user/thread. The active frozen V1
helper/config still watches the terminal V3 hunt run. A separate V2 helper
source and 12-test suite now accept only the explicit bounded lab schemas for
hunt, lifetime, herbivore, and ecology traits, one exact watched run, and an
explicit allowlisted immutable artifact filename (`hunt-lab-probe` or
`lab-probe`). V2 is not installed: root must review its SHA, freeze it, and
update the external config before any later lab relies on completion delivery.
Timer completion only queues a message; root must recheck run identity and
evidence.

The live worlds on 7878–7883, frozen/bin releases, and historic/raw lab
archives are outside this work. Biology defaults, closed-state behavior and
RNG order are acceptance constraints. New saved-state fields require an
explicit version and migration; learned experience never transfers to offspring.
