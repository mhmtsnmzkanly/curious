# Parallel water policy candidate

## Scope and hypothesis

The rule policy already compares the water sites visible now with
`entity::foraging::value(amount, path_len, life, true)`. The current learner
then replaces that ranking with an absolute stored reward; it only uses path
length as a tie-break. A once-good source can therefore outrank a currently
more useful one after its visible amount or route changes.

**Hypothesis:** a learned source-specific correction, added to the current
foraging score, will let an individual's own resolver-confirmed experience
change choices while keeping visible amount, current route cost, and current
body state in the score on every choice. A learned correction may change the
winner; it must not discard candidates through a hard dominance rule.

The candidate is a decision-layer policy experiment. It does not change water
availability, renewal, movement, drinking, thirst priority, or resolver rules.
It is default-off. With it off, decision, state, and RNG behavior must match the
legacy learner. This work does not change shared `src/`, checkpoint or SQLite
versions, a live binary/world, or start a world batch.

## Candidate score and outcome currency

For every currently perceived water candidate `i`, calculate the ordinary
current score `B_i = foraging::value(current_amount, current_path_len,
current_life, true)`. Add only that location's learned residual `R_i`:

```text
score_i = B_i + R_i
```

An unexperienced location has residual zero. The existing deterministic tie
break (shorter route, then perception order) remains after score comparison.
The existing exploration rate and candidate set remain unchanged.

The residual is an online running-mean approximation of
`actual_score - choice_time_baseline` for completed, attributable attempts at
that coordinate. The baseline is captured
when the learner first selects the source and uses the same visible amount,
path, and body snapshot used to form `B_i`. `actual_score` must use the same
water-foraging currency as `B_i`: water benefit weight 2, resolver-paid own
movement energy, elapsed episode ticks, and the same per-occupancy-tick
normalization. This makes the correction an outcome residual instead of adding
the existing raw learner reward (water weight 10, movement weight 2, time
weight 1) directly to the differently scaled foraging score. Resolver events,
not net body change or an inferred world amount, supply actual drink and paid
movement. Only the subject's own matching `EntityDrank` event counts.

The existing reward can remain in the legacy learner for parity and diagnostics;
it is not an input to the candidate score. Episodes censored by a higher
priority decision, external command, death, or ambiguous attribution do not
update residuals. A resolver-confirmed missing-resource drink rejection may
count as a completed failure with zero acquired water, if the current accounting
contract confirms that the attempt actually ended there. Repeated continuation
of an active target keeps its original baseline; it does not rescore or start a
second episode.

An additive residual still carries past outcome context across changes in
current amount, route, or body state; the current baseline makes those inputs
visible in every score but does not guarantee that history cannot outweigh
them. This is a model risk to measure, not a claim that the candidate solves
staleness. A reliability-adjusted meal estimate is a possible later alternative
if an explicit mechanism comparison justifies it; it is outside this candidate.

Residual arithmetic, sample count, and serialized memory must be bounded.
Resolver water is always a nonnegative acquired amount (zero on a confirmed
missing-resource failure); a negative score means only that paid movement/time
exceeded acquired-water benefit. It is not a prediction of a negative water
quantity. The example rejects rather than clamps outcomes outside the existing
episode accounting caps: `MAX_WATER_TICKS` is 20 and the learner's `MAX_ACCOUNT`
is 1,000,000; `WaterResult.water` is validated at 100. Its current outcome
formula therefore has conservative score range `[-100,000,100, 19,900]`: the
lower endpoint charges the maximum movement account in one tick; the upper
endpoint uses 100 units, zero movement cost, and one tick. The baseline uses
the bounded body and perception inputs to `foraging::value`; integration must
preserve checked or saturating arithmetic across both score terms. A small-sample shrinkage or decay rule is not part of
the first candidate; if analysis shows it is required, it needs a model-based
reason and a new preregistered version. The per-source table remains bounded by
the existing water memory limit. Source identity is position, as in the current
learner; the value is not inherited by offspring.

The prototype's integer recursive mean is bounded but is not an exact
arithmetic mean: integer division truncates each adjustment, so small corrections
can stop changing the stored value after enough samples. Before core integration,
choose and test a representation that states its precision and overflow bounds
(for example, a fixed-point mean or a bounded sum/count). No exact-mean or
long-run precision claim is made here.

## Boundaries and dependencies

- **Input:** only current `Perception.waters`, current `LifeState`, and this
  individual's bounded learned residuals.
- **Experience:** only this individual's choice baseline plus its own
  resolver-confirmed outcomes. No hidden map lookup, other actor's outcomes,
  future RNG, or inferred renewal schedule.
- **Physics:** unchanged `World` / resolver; choice cannot bypass a rejected
  or unaffordable action.
- **Persistence:** no core persistence change in this prototype. If approved
  for integration, the episode needs a validated choice-time baseline and the
  residual table needs independently reviewed checkpoint migration/defaults.
  SQLite event rows do not gain fields unless separately justified.
- **Other tracks:** refuge and ecology changes remain isolated; no shared
  source files are edited in this phase.

## Preregistered evaluation outline (not authorization to run)

1. Before source integration, root audits all manifests/raw records and freezes
   unseen seed roles, scenario, candidate/control source hashes, limits, and
   analysis. Seeds `51001..51008` and existing water-study seeds are seen and
   cannot be reused as held-out evaluation. The historical V2 report remains
   immutable context, not a new control result.
2. A calibration fixture must produce multiple fresh two-source choices with
   changing current amounts and route costs, plus resolver-confirmed outcomes
   for both sources, without body/age resets or replacements. Calibration can
   reject an unidentifiable fixture; it cannot tune coefficients or thresholds.
3. The formal arms are rule-based, naive online candidate, candidate trained
   online, and that exact trained candidate with updates frozen. They share
   initial world/body/age/perception/RNG and the same episode schedule; the
   trained/frozen fork differs only in the registered update flag. Include the
   legacy online learner only if root preregisters it as an additional
   mechanism comparator. All four required arms retain normal world/resolver
   execution.
4. Primary mechanism gate: at a same-world, same-RNG, same-perception fresh
   choice, the online residual must change a greedy winner versus its own
   frozen table, and a resolver-confirmed physical difference must follow.
   Record candidates' current amount, route length, baseline score, residual,
   final score, selected source, and exploration/greedy status.
5. Physical outcomes are reported per seed and arm without a weighted score:
   water acquired, own movement energy paid, survival/common-horizon ticks,
   terminal energy/water/health, death/censoring, and costs. Report Pareto
   relations against both the same-trained-table frozen arm and the strong
   rule baseline. Mechanism success alone does not pass policy utility.
6. A candidate passes only if the registered mechanism gate is met, no
   invariant/parity/persistence gate fails, and the preregistered physical
   criterion beats the rule baseline without hiding a dominated seed or
   adverse component. Root must choose that physical criterion before seeing
   evaluation results. No post-result tuning, seed replacement, or selective
   rerun.

## Acceptance and model cost

The example prototype is reviewable only when its score computation and
residual update have focused unit tests for current amount/cost scoring,
historical residual changing a choice without filtering either candidate,
own-target outcome attribution, censored outcomes, bounded values, and exact
default-off parity. Tests are local to the owned example; they do not establish
world behavior, improvement over the rule baseline, or correct end-to-end event
accounting. The current admission API distinguishes `CompletedDrink` from
`Censored`, leaves censored and actor/source-mismatched evidence unchanged, and
rejects out-of-contract physical totals; integration still has to bind it to
the resolver ledger.

The computational cost per choice is `O(W × V)` for scoring `W` currently
visible water sites against a linear scan of the bounded learned table `V` (at
most 32 entries). An indexed map could reduce lookup but adds serialization and
validation complexity and is not proposed. Each active episode adds one
choice-time score snapshot. No neural model, global cache, or additional RNG
draw is required. Candidate tie-breaking and the 10% exploration draw must
remain byte-for-byte the legacy sequence when the new option is disabled.

## Current status

Design/prototype only. No source integration, checkpoint version change,
preregistration, world ticks, or deployment is authorized by this document.
