# Continuous-lifetime water learning — preregistration

**Protocol version:** 2  
**Status:** fixture approved by root; protocol awaiting its source commit. No calibration or formal world run is authorized by this draft.  
**Question:** Can one focal individual’s own water-choice experience, accumulated during one continuous life, change a later decision and improve actual water/energy/survival after a real renewal reversal?

This is a bounded individual-learning demonstration. It is not an ecology,
population, inheritance, general intelligence, or long-term fitness acceptance.
The focal actor is created once at age zero. Its ID, body, age, health, energy,
water, position, memory, and learner history are never reset during its life.
There are no replacement actors, teleports, scripted decisions, forced
successes, world-state observations in the policy, or post-result parameter
sweeps.

## Hypothesis and contrasts

The focal actor learns during ticks 1–200 while a second finite herbivore
consumes water through ordinary AI decisions and the normal resolver. At the
fixed checkpoint after tick 200, the one shared world state is cloned into four
counterfactual suffixes. Before decision tick 201, the water-site renewal rates
are reversed in every suffix. All branches retain their tick-200 world, actor
IDs, bodies, age, positions, resources, memory, RNG and external schedule cursor.

| Branch | Focal water learner at fork | Purpose |
|---|---|---|
| `online` | Experienced values retained; updates enabled | Adaptation after reversal |
| `trained_frozen` | Same experienced values retained; updates frozen | Primary same-history counterfactual |
| `ablated_frozen` | Values cleared; updates frozen; all other state retained | Effect of the focal individual’s accumulated water table |
| `rule_based` | Learner disabled; its unused values retained | Descriptive current-rule baseline |

The first contrast for adaptation is `online` versus `trained_frozen`.
`trained_frozen` versus `ablated_frozen` asks whether the focal individual’s
pre-reversal water table affects choices and outcomes. `rule_based` supplies
the existing policy baseline; it is not treated as an independent animal.
Each branch starts from the same tick-200 body and world but can diverge
thereafter as choices alter physical outcomes and RNG use.

The existential hypothesis is supported only if at least one held-out seed has
all of the following in a single focal lifetime: (1) the online table changes
the focal actor’s selected site relative to `trained_frozen` at a fresh choice
with two visible sources and a registered causal attribution check below;
(2) the altered choice is followed by a resolver-confirmed physical difference
in acquired water; and (3) the online branch has a favorable physical outcome
vector without any negative component at the common paired horizon. The
common horizon is the shorter of the two focal suffix lifetimes, capped at 300
ticks after the fork. The vector is cumulative acquired water (higher is
better), cumulative movement energy paid (lower is better), and health and
energy at the common horizon (higher is better). Survival ticks are reported
separately and are nondeteriorating only if online survival is at least as
long as the paired control. A water gain paired with a larger movement-energy
cost, lower health/energy, or shorter survival is a tradeoff, not a net
benefit. Report full-suffix totals and survival separately too. No learner
reward, table value, or action difference alone counts as benefit. One seed
supports only an existence example; it does not support a general expected
advantage.

## Fixed fixture

- World bounds: `x=0..4`, `y=0..4`; open cells, with no predators, walls,
  burrows, or animals other than the two registered herbivores.
- Focal herbivore: ID 1, AI controller, initial position `(2,2)`.
- Competing herbivore: ID 2, AI controller, initial position `(2,3)`. It has
  the same ordinary default body as the focal subject and its water learner is
  disabled for the entire run. It moves and drinks only through its ordinary
  AI policy and resolver. It has finite health, energy, water, and lifespan; it
  is never reset or replaced.
- Both animals use the ordinary herbivore body values for all fields except
  `max_age=512` and `maturity_age=501`. This documented initial lifespan
  setting keeps the actors below reproductive maturity through the 500-tick
  ceiling; it is not an age bonus. All other body fields are default values,
  including `max_water=60`, `max_energy=80`, `speed=3`, and unit movement
  energy cost. Genetics, world policy, and ordinary memory settings remain at
  their defaults. The focal water learner is enabled with updates enabled from
  birth.
- Water site A at `(0,2)` and site B at `(4,2)`, each initially amount 12,
  capacity 12, with no availability schedule. Before tick 201, A replenishes
  4 every 2 ticks and B replenishes 1 every 4 ticks. After completed tick 200,
  swap only those two `(replenish, interval)` pairs before the next world tick:
  A becomes `1 every 4`, B becomes `4 every 2`. Do not directly edit either
  water cell or replenish a site outside the normal environment update.
- Four renewable food sites at `(0,0)`, `(4,0)`, `(0,4)`, and `(4,4)`; each
  starts at amount 20, has capacity 20, and replenishes 2 every 2 ticks. These
  support ordinary energy metabolism and are identical across all branches.
  They do not restore health, water, or actors.
- Environment renewal occurs only through `Environment::update` in normal
  world ticks. The only harness intervention is the registered renewal-rate
  reversal after tick 200. No entity receives an external command.

The two water sites are equidistant from the focal actor at creation. Their
finite capacity plus actual drinks should make renewal differences visible as
different experienced availability. The registered calibration gate below
checks that this intended contrast actually occurs; a source remaining full or
an unavailable comparison is not assumed away.

## Timeline and fixed bounds

Each seed has exactly one continuous prefix of 200 world ticks. Record the
complete `State`, RNG, learner, actor bodies, environment, and external
schedule cursor after completed tick 200. If either actor is dead at that
checkpoint, the seed is invalid for the registered branch comparison; do not
revive or replace it.

Make four copies of that checkpoint. In `online`, retain the learner unchanged.
In `trained_frozen`, change only `update_enabled` to false. In
`ablated_frozen`, change `update_enabled` to false and clear only the focal
learner’s `values`; retain its update-independent fields, including any active
episode, last result, and RNG state. In `rule_based`, change the focal learner’s
`enabled` and `update_enabled` flags to false; its table remains stored but
unused. The rule-based branch’s normal disabled-learner interruption is part of
this declared treatment. Do not normalize later branch states or force matching
actions. Tests and raw fork audit must assert each branch’s exact
`(enabled, update_enabled)` pair: online `(true,true)`, trained-frozen
`(true,false)`, ablated-frozen `(true,false)`, and rule-based `(false,false)`.

Attribute a decision effect to post-reversal updates only when one of these
predeclared checks succeeds. **Paired check:** at the first fresh choice where
online and trained-frozen select different sources, their complete pre-choice
world/body/perception/RNG records must be equal after normalizing only the
focal learner’s values and the two learner flags. The selected source must
differ at that same subjective exposure. **Same-exposure shadow check:** if
the paired worlds have already diverged, use the online branch’s recorded
perception and RNG state to calculate its greedy choice from its current table
and from the checkpoint’s trained table, without consuming RNG or mutating
either table. The realized online choice must be the online-table greedy
winner, not exploration, and the shadow trained-table winner must be a
different source. A later choice in different worlds by itself is not evidence
that learning caused the decision difference.

The reversal is applied at the fork boundary in all branches, before decision
tick 201, by changing the registered renewal rates only. Each branch continues
until the focal actor dies naturally or completes tick 500. The other actor
continues under the same rules until its natural death or the same ceiling.
There is no branch-specific reset, state injection, direct resource refill,
or survival intervention. Maximum suffix duration is 300 ticks per branch.

Calibration is exactly one separate seed, `51099`, with the same fixture,
timeline, branch definitions, and 1,400-tick hard cap. Formal evaluation is
exactly seeds `51001`–`51008`, with a maximum of 1,400 physical world ticks per
seed (200 prefix + 4 × 300 suffix), 11,200 formal ticks total. Combined
calibration and formal maximum: 12,600 physical ticks. Each batch has a
300-second wall limit, 1 GiB maximum captured raw output, and 1 GiB maximum
child-process memory per batch. Stop on the first
identity/checkpoint mismatch, malformed row, invalid state, unexpected actor
creation, or runner error; preserve partial output and mark the batch failed.
No selective reruns or seed replacement.

Before any evaluation, audit these seeds against all available experiment
manifests and raw data. Any previously evaluated seed is marked seen and
excluded before the protocol is committed; do not replace it after results are
visible. Commit this protocol and the clean lab source before the calibration
run. The held-out run requires a separate clean source identity and root review
of calibration evidence.

## Calibration validity gate and stop rule

Calibration is a fixture-validity check, not a tuning batch. Do not change the
map, bodies, rates, resources, branch definitions, seeds, gates, or analysis
after seeing calibration. Calibration passes only if the `online` focal actor
and competitor both reach the fork alive and the focal actor has, in the
single continuous life:

- at least 6 **fresh** post-metabolism thirst decisions before tick 200,
  each with two currently perceived positive-water sites;
- at least 3 resolver-confirmed focal drinks before tick 200;
- in the `online` suffix, at least 6 such fresh two-source thirst decisions
  and 3 resolver-confirmed focal drinks after tick 200 and before death or
  tick 500; and
- at least one of the focal actor’s two currently perceived source amounts is
  below its registered capacity at each of at least 3 such two-source
  decisions in each regime, so the comparison is not only between perpetually
  full sites.

A fresh choice is a decision where the focal actor’s pre-learner decision is
`SeekWater` and current perception contains both water coordinates with
positive amounts, and either there was no active water episode before intent
collection or its previous target is no longer among those perceived sources.
In the latter case the existing learner interrupts the old episode before
choosing again. Repeated per-tick continuation of an active, still-perceived
target is not a new choice. In this fixture, record the pre-learner reason from
the final trace: the water learner emits `LearnedWater` only when the
pre-learner reason was `SeekWater`, and it otherwise leaves `SeekWater` intact.
Any final reason outside that pair is not a fresh water choice. This
classification does not re-run the policy or consume RNG. A completed drink
must match the focal ID and selected source in a resolver `EntityDrank` event.

Before using the registered fixture, add isolated software tests that prove
checkpoint serialization/restore and per-tick state-plus-RNG replay equality
for a representative fixture state. Add fork tests that compare all four
branch start states against the tick-200 checkpoint and allow only the branch
table/learner-flag differences specified above. These tests use development
fixtures and do not consume calibration or held-out world ticks.

If any calibration gate fails, stop. Mark the fixture non-identifiable and do
not run held-out seeds or tune it. Report observed exposure counts and which
gate failed. A calibration pass only permits root to review the evidence and
authorize the formal run; it does not accept the learning hypothesis. A rank
reversal is not a calibration gate. If the formal run shows no reversal in the
online table, adaptation is unsupported or non-identifiable under this fixture;
do not modify the protocol or fixture.

## Registered observations and analysis

Retain raw data for every physical tick and each branch. Record:

- source revision, binary/protocol/fixture/runner hashes, command, seed,
  runtime and resource bounds;
- complete pre- and post-tick `State` and RNG state/hash, actor IDs, positions,
  ages, body fields, learner values/visits/options, memory, environment rates,
  actual water and food contents, and actor liveness/death causes;
- every focal subjective choice exposure: the current visible water positions,
  positive amounts, path lengths, base reason, final reason/intent, active
  versus fresh episode, pre-choice value for each candidate, selected target,
  and whether the policy used exploration or greedy selection;
- resolver events for both actors, especially movement and its paid energy,
  drink position and exact water amount, rejected/missing-resource drinks,
  metabolized state and death; and
- initial and fork state hashes with a narrow declaration of branch-only
  learner fields. The four initial suffix world/body/RNG states must be equal
  before applying the listed learner treatment. Report any difference caused
  by the branch’s first normal tick rather than masking it.

From recorded pre-choice state, calculate the candidate argmax from saved
values and existing deterministic tie-breaks without consuming RNG or
mutating the learner. Log the actual exploration outcome separately. Do not
call policy helpers a second time or use hidden world state as a subject input.

Report every seed and branch separately: fresh choice counts, selected source
counts, source amounts, drinks and water acquired, movement energy, time,
energy/water/health at fork and termination, death/censoring, and actual
survival ticks. Compare `online` with `trained_frozen` for reversal adaptation;
compare `trained_frozen` with `ablated_frozen` for the causal effect of the
pre-reversal individual table; show `rule_based` as context. Include source
rank reversals and the decisions they can affect. A learning-table update,
reward increase, more time alive in accelerated ticks, or changed action by
itself is not success. Missing choice exposure is non-identifiability, not a
negative learning outcome. Do not infer real time lived, age advantage,
offspring transfer, evolutionary benefit, or ecosystem persistence.

The immutable raw capture, stderr, manifest, summary, and report belong in
`data/experiments/lifetime-lab-v2-20261023/`; the separate calibration belongs
in `data/experiments/lifetime-lab-v2-calibration-20261023/`. The completion
manifest must use schema `curious-lifetime-lab-run-v2`, `protocol_version: 2`,
and immutable helper artifact name `lab-probe`. Root controls protocol
commit/review, seed audit, formal-run authorization, service/timer/runtime
configuration, and commits. This document alone authorizes no world run.
