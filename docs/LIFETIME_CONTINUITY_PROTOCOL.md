# Lifetime continuity and reversal — preregistration

**Status: preregistered by root; bounded execution requires a clean source commit and root review.** This lab asks whether one real individual keeps
its own body state, age, identity, and learner experience across an observed
environmental reversal and checkpoint/resume. It is a continuity and bounded
adaptation study, not evidence for a general lifespan-learning claim.

## Question and design

Use one herbivore subject per seed, created once at age zero with the ordinary
species body and `max_age`; do not reset its body, age, identity, memory, or
learner between phases. This is a continuity test of the existing individual
water-choice learner plus physical-state persistence, not a claim that every
resource has a learned policy. The subject is the only herbivore; its initial
state is captured before the first world tick. It remains the same actor until
natural death or the registered tick ceiling. No replacement individual is
substituted after death. Use exactly eight independent lab seeds, 44001–44008;
there is no separate training-seed cohort or transfer of experience between
individuals.

Pin the arena to a 7×7 open map with subject start `(3,3)`, water sites at
`(1,3)` and `(5,3)`, and renewable food at `(3,0)` and `(3,6)` to sustain
ordinary metabolism. Both food sites start at amount/capacity 20, replenish 1
every 2 ticks. Both water sites start at amount 12 and capacity 30. For the
first 52 completed world ticks, the west water site replenishes 4 every 2
ticks and the east site 1 every 4 ticks. After completed tick 52, reverse
those two site rates before decision tick 53. After completed tick 78, reverse
them again before decision tick 79. These are actual environment/resource changes,
not trial resets; the individual sees only current water amounts and its
ordinary perception. One controlled carnivore with stable actor ID starts at
`(0,1)` and patrols the 7×7 rectangle clockwise using one-cell `Move` intents
through the normal resolver: east along `y=1` to `(6,1)`, south to `(6,5)`,
west to `(0,5)`, north to `(0,1)`, and repeat. This brings the predator within
the ordinary two-cell threat range along the middle portions of both passes.
After completed tick 52 it reverses the route direction before decision tick
53 through the same resolver; it is
never moved by writing its position. The subject observes
only the predator's current perceived position. The route and resource
intervention are harness-side exogenous actions and are not exposed as future
information. The exact tick ordering (world update, perception, intervention,
resolver) must be fixed in the runner and asserted in its event log.

If the subject dies during tick 52, record `died_at_checkpoint` and do not create a checkpoint or counterfactual branches. Earlier death is `died_before_checkpoint`. If it survives, after the tick-52 resolver has completed, apply the first reversal and capture
the exact complete `State`, RNG state, and external script cursor before
decision tick 53. Record one uninterrupted online continuation and one
restore-from-checkpoint online continuation for the same remaining 52 ticks.
They must match in every event,
actor field, memory/learner field, serialized state, and RNG state after every
tick. From the same tick-52 checkpoint, make a third counterfactual branch
that freezes only learner updates while retaining the exact same identity,
body, age, memories, learned table, RNG state, and environment. It is a
counterfactual, not a second animal in the same world.

The subject has only one continuous 104-tick life history. This is a bounded
readout of existing lifetime state, so it adds no age bonus,
offspring memory, or new learning signal. A code change is justified only if
the audit shows that the ordinary motor or persistence layer loses individual
state across a real reversal or checkpoint resume. The lab must not reset the
subject to manufacture additional trials.

## Registered outputs and bounds

Record the raw actor ID and species; age/max age; health, energy, water,
stamina, position, and body/genome fields; current visible food/water/predator
observations; action and reason; resolver movement/interaction cost and
damage; resource contents; learner table, visit counts and update flag; memory
including observed dangers; complete state/RNG hashes; and any death or
reproduction event. Include the same fields at creation, tick 52, after each
post-reversal tick, and at termination. Assert ID, birth age, and age
progression rather than merely trusting a `no_reset` manifest flag. Assert
that no new subject replaces a dead one and that each paired branch begins
with byte-equivalent body and learner state.

The physical tick ceiling is 8 × (104 uninterrupted + 52 restored replay + 52
frozen counterfactual) = 1,664 ticks. Stop on the first invalid identity,
age/body discontinuity, state/RNG mismatch in the replay pair, missing event,
schema error, or timeout. Preserve partial output and mark the run failed;
never rerun selectively by seed. Do not run a parameter sweep. Report all
eight seeds, each reversal window, survival/death, actual resource use,
realized costs/damage, and learner state deltas. Descriptive online/frozen
differences are exploratory and are not a deployment criterion.

Before execution, the committed protocol must pin the concrete arena and
predator script, binary/source SHA, command, runner/protocol SHA, output path,
schema and stop rule. Use a fresh run directory under `data/experiments/` and
preserve raw JSONL, validated summary, report, and manifest. Evaluation belongs
to root after a clean source commit; no live world, frozen release, or historic
archive is in scope.
