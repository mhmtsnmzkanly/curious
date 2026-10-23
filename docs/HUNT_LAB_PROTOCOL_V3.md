# Hunt laboratory v3 — preregistration

This preregistered study tests whether a bounded individual can use its own resolver-confirmed
prey-defense injury history to avoid riskier visible prey contexts. It does not
test survival, reproduction, lifetime learning, or mixed-ecosystem balance.
Evaluation has not run. This file must be committed with the source revision
before the registered runner is executed.

## Hypothesis and candidate rule

When two eligible prey candidates have different recorded injury histories,
the risk-aware learner will prefer the candidate context with the lower observed
injury rate, while retaining the existing learned food/action-cost score as the
next ordering criterion. The risk ordering is lexicographic: fewer injury
episodes per eligible exposed episode, then lower mean prey-defense damage,
then the existing learned net-return value, then the existing rule-based
estimated-cost, distance, and identity order. No scalar injury-to-food or
injury-to-energy coefficient is introduced.

Risk evidence is a separate, bounded per-context record. An episode contributes
one exposure only when the resolver ledger contains at least one attack-energy
cost and all incoming damage is attributed to that episode's visible prey.
An exposed episode with positive prey-defense damage is an injury. Episodes
with no executed attack, third-party or unattributed damage, missing/mismatched
damage totals, or ambiguous audit are not risk observations. Actual policy
closures record injury within the observed window. A laboratory-forced boundary
does not count as an exposure, even if an attack occurred; no injury before
that boundary is not evidence of a safe completed pursuit. Counts and total
damage are updated once at eligible closure. Evidence is not copied to
offspring. The existing food/action-energy/tick reward and coefficients remain
unchanged. Risk statistics are bounded to 108 contexts, one million eligible
exposures per context, and at most one million damage units per exposure; their
damage accumulator remains exact within that bound.

Unknown risk is not zero risk. Before any candidate context has an eligible
observation, the existing learned-score and rule ordering applies. Once at least
one candidate has a risk observation, non-exploration choice is limited to
candidates with observed risk; the lexicographic rule above selects among
them. The fixed existing 10% exploration branch may select an unobserved
candidate to gather evidence. It is reported separately. This conservative
gate is a policy hypothesis, not a claim that missing evidence means safety.

## Arms, seeds, and bounds

The trial map, bodies, resource settings, prey scripts, 20-tick limit, reversals,
and resolver event accounting remain those defined in V1/V2. Existing training
seeds remain 1, 2, 3, 4, 5, 42, 12345, and 2026. Evaluation pairs use new
protocol seeds 43001–43008 in that order; they are disjoint from V1/V2's
41001–41008 and 42001–42008. No project-wide claim is made that these numeric
seeds were never used elsewhere.

For each of passive and defensive tracks, run 80 trials per training seed for
the plain learner and 80 for the risk-aware learner, with the same generated
world-seed schedule. Evaluation uses six arms on identical per-trial world
seeds: rule-based, naive-online, experienced-online (plain learned table),
experienced-risk-online, the risk-trained table with updates frozen, and
`experienced_risk_ablation_frozen`. The last arm uses the exact same initial
food/action-cost table and risk statistics as the risk-frozen arm, with updates
frozen in both, but disables the risk choice rule. This isolates the effect of
the choice rule. Both frozen arms must leave values, risk counts, and risk
damage totals byte-for-byte unchanged. The first 40 evaluation trials are
normal and the last 40 reverse which visible prey flees or defends, as in the
existing contract. The upper bound is 2 tracks × 8 seed pairs × (80 plain +
80 risk training + 6 × 80 evaluation) × 20 ticks = 204,800 physical ticks.
The runner stops on the first
schema/seed/audit validation failure and preserves all raw files. No parameter
sweep, automatic tuning, continuation after a biological result, or live
deployment is part of this protocol.

## Outcomes and analysis

Record per trial and per tick: the actor's initial visible contexts, decisions,
selected target, actual attack/movement energy from resolver events, food,
prey-defense/external/unattributed damage, rejected actions, final energy and
health, time to first food, failed ticks, episode closures and reasons, learned
food/action-cost values, risk exposures/injuries/damage before and after, and
whether exploration selected an unobserved context. Validate that damage
categories sum to incoming damage and that food/energy totals reconcile with
resolver events and body accounting. Count each episode closure once, including
two in one tick. Preserve censored episodes without treating them as safe or
successful.

Primary policy outcome is injury incidence per evaluation trial and exposed
episode. Report it jointly with food acquired, deaths/final health, movement and
attack energy, total energy spent, failed ticks, first-food tick, and
unscored/censored endings. Show each seed and normal/reversed window; compare
risk-online with experienced-online, risk-frozen with risk-online, and
risk-frozen with the same-table `experienced_risk_ablation_frozen` arm. Do not
collapse food and injury into a new weighted reward or infer an injury cause
from unrelated damage. A risk reduction without its food and survival tradeoff
is not a successful result. A passed mechanism test is not ecological
acceptance.

## Version, command, and stopping rule

V3 adds bounded risk evidence to the individual's saved hunt learner. The new
`risk_enabled` setting defaults false on creation and migration. Checkpoint V14
migrates V13 risk evidence to empty and keeps the feature disabled, preserving
any existing pending episode and its ambiguity flags. SQLite schema V2 is
independently versioned and remains unchanged because it stores the versioned
checkpoint JSON. Tests must confirm checkpoint continuation of state plus RNG,
disabled no-op compatibility, risk evidence bounds, and no offspring transfer.

The exact command, source commit, executable SHA, runner revision, protocol SHA,
fresh output path, start time, and stop rule must be captured in the run
manifest. The V3 runner must require a fresh output path and the V3 selector;
it must not reuse a V1/V2 result. No run is authorized until this protocol and
the clean source revision are committed and reviewed.
