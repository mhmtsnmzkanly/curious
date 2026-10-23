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
| 1 | Opt-in hunt injury-risk history, causally tied to resolver-confirmed prey defense and a choice rule with unknown evidence kept unknown. Preregistered six-arm lab compares food, injury, deaths/final health, and energy. | V3 protocol at `c14a778`, censoring/protocol amendment `f12da9a`; source/tests and V3 bounded runner in progress; no evaluation run. |
| 2 | Same individual retains its actual identity, age, body history, and learner through resource/predator reversal and checkpoint/resume. Audit raw actor state and RNG equality across uninterrupted/resumed branches. | Luna6; pending phase 1 review. |
| 3 | Extend existing danger memory for herbivore escape/shelter choices after observed threat. Measure real movement cost, own received damage, escape success, and blocked/occupied destinations; do not reveal future threat positions. | Luna6; pending phase 2. |
| 4 | Population-independent spatial/temporal resource schedule with deterministic site offsets and a closed-feature no-RNG equivalence check. | `ecology_traits_luna6` worktree; pending phase 3 coordination/review. |
| 5 | Individual decision timeline for seen/remembered evidence, selected action, observed cost, and resulting update; compare first/last genuinely similar subjective encounters. | `experience_documentary_luna6` worktree; pending phase 4 coordination/review. |
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

Root committed the preregistration before source implementation. Raw V3 output
does not yet exist. The existing bounded completion timer is installed and
active (`curious-hunt-lab-completion.timer`); root owns systemd changes. It uses
a frozen helper, external runtime configuration, explicit watched run paths,
terminal manifest and content-hash checks, durable outbox, and native
`codex queue` delivery to the configured user/thread. Its current config watches
`hunt-lab-v3-*`; later phase labs need explicit compatible run names or a
reviewed ops-config update. Timer completion only queues a message; root must
recheck run identity and evidence.

The live worlds on 7878–7883, frozen/bin releases, and historic/raw lab
archives are outside this work. Biology defaults, closed-state behavior and
RNG order are acceptance constraints. New saved-state fields require an
explicit version and migration; learned experience never transfers to offspring.
