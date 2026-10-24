# V17 continuous-life water residual lab — preregistration

**Status:** preregistration choices selected; execution requires root's source
review and commit. No calibration or formal World batch has run. The frozen
choices below cannot be revised after seeing candidate outcomes.

Before preregistration, two explicit unit-fixture runs used unregistered seed
123 to check the serialized record, Python validation boundary, and closure
replay observer. Each produced 1,400 physical ticks. All 1,400 world frames
matched across runs in before/after State, RNG, events, and decisions; only the
new closure audit fields were added in the second run. The mechanism audit did
not qualify. These 2,800 engineering ticks are not evaluation evidence. Both
raw records are outside the canonical archive at
`/tmp/curious-water-residual-unit-seed123.jsonl` and
`/tmp/curious-water-residual-unit-seed123-v2.jsonl`.

The calibration opportunity gate was refined during engineering review to
require a completed post-fork update that changes the selected source's exact
residual mean, so count-only updates cannot make the candidate identifiable.
This was set before any registered seed outcome was observed. No acceptance
rule or model parameter was tuned from unit results; the fixture, physical
utility criterion, and registered seed roles remain fixed. No registered seed
has been read or consumed, and the unit result is not an acceptance result.

## Question and scope

Does the V17 per-source exact sum/count residual, added to the current visible
foraging score, change an individual's later greedy water choice and produce a
resolver-confirmed physical difference? Does the resulting physical vector
improve on the ordinary rule-based policy under a root-selected criterion?
This is a bounded individual learning comparison, not ecological qualification,
lifespan evidence, or a general performance claim.

The candidate is enabled only through the actual global
`PolicySettings.water_residual_enabled` flag. Candidate arms also enable the
focal learner. The rule-based arm keeps the global option available but sets
the focal learner disabled, so it uses the ordinary policy instead of the
legacy absolute-value bandit. The ordinary-AI rival has its learner disabled
in every arm.

## Fixture and continuity

Use the 5×5 predator-free fixture and ordinary World/resolver execution from
`examples/water_residual_lab.rs`. Actors 1 and 2 are ordinary AI herbivores at
`(2,2)` and `(2,3)`. Both use the normal default body except `max_age=512` and
`maturity_age=501`; age starts at zero and never resets. Initial water cells at
`(0,2)` and `(4,2)` each have amount/capacity 12 and renewal `(4,2 ticks)` and
`(1,4 ticks)` respectively. Food cells at `(0,0)`, `(4,0)`, `(0,4)`, `(4,4)`
each have amount/capacity 20 and renewal `(2,2 ticks)`. Water renewal rates
swap before tick 201: A becomes `(1,4 ticks)`, B `(4,2 ticks)`. Use a continuous
prefix through tick 200 and fork the same tick-200 checkpoint into four
300-tick suffixes. No body or age reset, new actor, offspring, rescue, forced
choice, outside refill, or hidden world input is permitted. The rival retains
ordinary AI behavior and may die naturally. Record all deaths and censoring.

The prefix has global residual selection enabled and the focal learner enabled
with updates enabled, so its own residual table and active episode history are
the trained history. At the fork, preserve the active episode and its captured
choice-time baseline in every arm unless the disabled rule-based learner
naturally closes it. Do not reset age or bodies at the fork.

## Four registered arms

| Arm | Global V17 option | Focal learner | Updates | Fork treatment |
|---|---:|---:|---:|---|
| `rule_based` | true | disabled | disabled | Use ordinary rules; learner values remain unused. |
| `naive_online` | true | enabled | enabled | Clear only residual entries; preserve legacy values, active episode, and captured baseline. |
| `trained_online` | true | enabled | enabled | Retain prefix-trained legacy and residual tables. |
| `trained_frozen` | true | enabled | disabled | Retain the exact trained tables; both legacy values/visits and residual sum/count must stay unchanged. |

Naive online isolates the residual table by starting with no residual samples;
its legacy water values and episode state remain present but are ignored by the
V17 score. No arm changes exploration, body, world, resolver, or environment
except the registered rate reversal. All arms fork from identical checkpoint
state and RNG before treatment-specific learner settings.

## Seed roles and fixed limits

The shared complete seed scanner audited 1,068 text files under
`/root/curious/data/experiments` at the registered 64 MiB line bound.
Calibration seed: **52099**. Ordered formal seeds: **52001, 52002, 52003,
52004, 52005, 52006, 52007, 52008**. The scan found no matches for any of
these seeds. The runner rescans only the seeds for the current phase before
each invocation and records that phase-only audit in its manifest. A conflict
stops the phase; no replacement is allowed.

Each calibration seed is one continuous prefix plus four suffix forks, capped
at 1,400 physical ticks. Each formal seed has the same cap. There is exactly
one calibration invocation and one formal invocation, with no retries,
replacements, parameter changes, or post-result tuning. Calibration checks
that both actors are alive at the fork, the trained residual table is nonempty,
the prefix contains at least two fresh choices with two positive visible
sources while focal body water is below maximum, the trained-online suffix contains at least two
such choices, at least one matched completed drink occurs in the prefix and
suffix, at least one suffix residual sum/count update occurs, and at least one
own completed suffix update changes the selected source's exact sum/count mean.
This last condition verifies a preference-relevant learning opportunity; it
does not set or tune an outcome coefficient or acceptance threshold. Failure
of any calibration condition is terminal non-identifiability and blocks formal
evaluation without tuning. The runner imposes a
300-second batch limit, 1 GiB
child-memory ceiling, 1 GiB raw-output ceiling, and a complete 64 MiB per-seed
record scan. Output directories are unique and failed raw records are retained.

## Measurements and causal audit

For every tick retain full pre/post `State`, RNG before/after, decisions,
perception, and resolver events. For every focal perceived source record current
amount, route length, current foraging baseline, residual sum/count/mean,
final candidate score, legacy value/visits, selected target, and greedy versus
exploration status. Record active-episode baseline and residual table before
and after each decision. Verify the actual world policy flag is true in all
arms; verify the rival learner remains disabled. At fork require a nonempty
trained residual table. For `trained_frozen`, verify both learned tables remain
unchanged throughout the suffix. For `naive_online`, verify only residuals
were cleared and legacy values plus active episode/baseline were retained.

The primary mechanism comparison is the first fresh greedy choice where
`trained_online` and `trained_frozen` begin from the same physical pre-world
state, focal body, candidate perception, and RNG after normalizing only focal
legacy/residual tables and the registered update setting. The active episode,
captured baseline, body, rival, environment, and all other physical state must
still match. Compare the current foraging baseline plus the actual current
online mean and frozen mean at that same exposure; do not use legacy absolute
Q values to audit the candidate score. Record the different greedy targets and
confirm an actual own drink or paid movement event differs afterward. This is
a mechanism gate, not a physical benefit claim.

Report per seed and arm, with no weighted utility: resolver-confirmed water
acquired (higher), own resolver-paid movement energy (lower), observed survival
ticks (higher), terminal energy/water/health (higher), deaths, censoring, and
exposure counts. Compare each candidate arm against both `trained_frozen` and
`rule_based`. Recompute physical totals from raw resolver events and state;
missing or ambiguous event evidence is unknown, never success.

Separate mechanism evidence from bounded utility evidence. Mechanism requires
at least one qualifying greedy same-prestate/perception/RNG trained-online vs
trained-frozen target change followed by a different own drink or paid movement
event. Bounded utility requires all four focal branches to complete the full
300-tick suffix alive, then the **trained_online** componentwise sum vector
across the fixed eight formal seeds to Pareto-dominate both trained-frozen and rule-based (all
components at least as good, at least one strictly better against each), with
no per-seed health or observed-survival deficit against either comparator. A
death before the horizon is a biological negative / insufficient common
horizon for utility, not infrastructure failure; report observed components
and death separately without imputing terminal components. Report every
per-seed Pareto, tradeoff, or dominated result and aggregate vector. Do not
hide adverse water, energy, or movement tradeoffs or make a general robustness
claim. The eight seeds are descriptive, not a general statistical claim. No
automatic deployment follows.
`naive_online` comparisons are descriptive only and do not contribute to the
bounded-utility gate.

## Identity, output, and stopping

Use an independent schema/version (`curious-water-residual-seed-v1`),
protocol, runner, and binary binding; do not edit old lifetime-water files.
Bind the clean candidate revision, all source hashes, protocol hash, runner
hash, binary hash, exact command/flags, seed roles, limits, timestamps, and
raw/summary hashes in an immutable manifest. A source/protocol mismatch blocks
formal execution. Preserve raw, manifest, stderr, summary, and report on every
failure. A failed fixture gate is terminal and prevents formal execution.

No old formal/calibration seed is run here, no live or persistent world is
opened, no source under `src/` is changed, and no acceptance claim or deployment
follows from the harness alone.

## Registered invocation and external stop

Root runs each phase from this committed checkout under the host supervisor:

```sh
/usr/bin/timeout --signal=TERM --kill-after=5s 300s \
  /usr/bin/prlimit --as=1073741824 -- \
  python3 scripts/run_water_residual_lab.py calibration
```

The formal invocation substitutes `formal` for `calibration`. The supervisor
caps the complete parent-plus-analysis wall time at 300 seconds and the runner
caps the probe at 1 GiB address space, 1 GiB raw output, and 64 MiB per seed
record. Root retains the supervisor log and exit status. A timeout or other
infrastructure failure is terminal for that phase: preserve partial output and
do not retry or replace seeds.
