# Refuge physical baseline: one-run observation protocol

**Status: root-reviewed and frozen for one engineering run.** This protocol
measures current-route effort and observes physical outcomes in one ordinary
World run. It does not enable refuge learning or emit an expected damage
estimate. It authorizes only the fixed seen-seed run below; any source, fixture,
scenario, or command mismatch invalidates the run and stops without retry.

## Frozen setup

- Scenario: `scenarios/mixed-refuge.json`, SHA-256
  `9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2`.
- Seed: `123` (previously seen engineering seed). This is one descriptive
  engineering measurement, not independent or held-out seed evidence.
- Horizon: exactly 1,000 physical ticks, unless a registered stop condition is
  reached.
- Limits: 300 seconds wall time, 1 GiB address space, and 128 MiB uncompressed
  JSONL output. The output writer rejects a row that would exceed its cap.
- One no-argument release executable. Before launch, the external runner must
  verify and record the scenario, source, executable, protocol, output-path,
  and resource-limit hashes/settings. The executable embeds the frozen
  scenario SHA but does not implement SHA itself.
- Frozen base engine source: V18 commit
  `117a440431deead231ac6d4edbe324be2345936a`. The runner verifies that the
  experiment commit descends from this revision and that all refuge/decision,
  resolver, perception, scenario, and checkpoint engine files still match
  their V18 contents. New observer files and this protocol are separately
  included in the experiment source hash.
- No sweep, retry, replacement seed, fixture edit, learner edit, body edit,
  command injection, forced encounter, or post-tick intervention. On any
  scenario/intent mismatch, writer cap, timeout, memory-limit event, missing
  full perception, or incomplete horizon, stop and retain the partial output
  as an incomplete run. Do not automatically retry.

## Observation-only configuration

At tick zero, set `world.observation.full_all = true` once, solely so the
decision trace retains every herbivore's current perception for this audit.
Source inspection confirms this flag controls trace storage and is not read by
the decision or resolver. It does not change World state, RNG, or actions.
Refuge outcome/generalization flags and tables must remain disabled/empty; the
observer checks that condition initially and after each tick. No learner table
is populated or copied.

The lab tracks any herbivore with a normal `SeekShelter` decision, including
descendants born during the run, by its actual entity ID. It does not designate
a focal herbivore or predator. A route row contains every currently perceived,
nonempty shelter path with an unoccupied endpoint, in perception order. It
does not apply the opt-in learner's eight-row memory cap to the ordinary
behavior. The observer identifies the current threat with the ordinary
threat-instinct predicate: first perceived species that hunts herbivores and
is within two path steps. An empty perceived path is preserved if present;
the opt-in learner's additional nonempty-path condition does not define this
baseline. The ordinary fallback route is the shortest path, with
perception order as the tie-break. The emitted `SeekShelter` move must match
that route's first `speed` directions; otherwise the lab stops as a
baseline/decision mismatch.

For each route, record endpoint, perception order, directions, step count,
turn count, per-step observed cell values (missing cells are explicit `null`),
and static current-position threat clearance. These are current observations,
not forecasts. Conditional nominal paid movement energy is
`path_steps * current move_energy_cost`; conditional nominal time is
`ceil(path_steps / current speed)`. They assume every step resolves on the
current route with no interruption, invalidation, or energy/stamina shortfall.
The observer separately records resolver-paid movement energy and actual
elapsed ticks.

Expected threat damage remains **Unknown** for every route. Perceived threat
`power` is health plus energy, not attack damage; optional perceived health is
not target capacity. Public species-profile attack damage may be included only
as descriptive context. The helper does not use it as an expected value or
rank routes.

## Episode accounting and censoring

The observer maintains a declared audit-attempt lifecycle adapted from the
current opt-in episode closure rules, without writing to World memory. Since
the refuge flags stay off, these rows are observer-created attempts, not core
`RefugeEpisode`s:

1. At each decision boundary, reconcile health since the prior resolver
boundary before checking whether the prior attempt continues. A missing
qualifying threat closes as `ThreatLost`; a changed decision reason closes as
`PriorityInterrupted`; no valid route closes as `OptionsInvalid`; a changed
threat/endpoint closes as `TargetChanged`. These decision-time closures are
   queued and emitted before the observer records current active-attempt
   resolver effects.
2. Start an attempt only for a `SeekShelter` trace with a qualifying current
   threat, valid candidates, and a matching ordinary fallback move. Record the
   pre-resolver body state and every current candidate route.
3. For every active herbivore and tick, preserve all global `EntityMoved` and
   `EntityAttacked` resolver witnesses, indexed in the original event vector.
   For the attempt, sum movement energy charged to that herbivore and attacks
   against it by the selected threat ID or by other attackers. Reconcile
   health loss only after collecting all attack events, using the same
   saturating subtraction rule as the core. Each attempt tick records before
   and after position/health, the attributed events, cumulative physical
   quantities, and elapsed ticks.
4. After resolver effects, close with the core's observed priority: death, physical arrival at the
   selected burrow, then the 20-tick timeout. If an active owner disappears
   from the entity list, close it as `ObservationMissing`; at tick 1,000, close
   remaining active attempts as `HorizonBoundary`.

Only a resolver-verified `Arrived` episode with no other-attacker damage and no
unattributed health loss is an attributable completed damage observation.
Death, timeout, threat loss, priority interruption, invalid options, target
change, missing observation, and horizon boundary are censored for damage;
their observed partial events remain in raw records and never become a
zero-damage success. An arrival with other/unattributed damage counts as a
physical arrival for effort/time summaries but is excluded from attributable
damage summaries. This is observational attribution to the selected threat,
not a causal or counterfactual claim.

## Descriptive diagnostics and gates

For the eligible completed-arrival damage rows only, report a fixed-mean
predictor diagnostic with five deterministic entity-ID folds:
`fold = owner_id % 5`. All episodes from one owner stay in the same fold. Each
fold's training mean predicts the threat damage for its held-out rows; report
counts of agents/episodes, training and held-out means, held-out mean absolute
error, and signed error (`prediction - observed`). Empty folds are `Unknown`,
not imputed. Censored rows and arrivals with ambiguous damage attribution are
excluded from this diagnostic. This same-world grouped split is a descriptive
check for repeated-owner leakage, not independent-world generalization,
calibration proof, or a model to deploy.

Also report all attempt starts/closures, completed arrivals, attributable
arrivals, censor reasons, unmatched refuge decisions, nominal-energy
exact-match count and mean absolute error, nominal-time comparison counts,
and actual elapsed time. No numerical success threshold or utility threshold
is preregistered: this run answers whether the fixture naturally provides
enough observed physical outcomes to characterize effort and an empirical
damage sample. Sparse/empty outcomes stay Unknown and recommend a revised
design; they do not establish that risk is zero or that the mechanism is
impossible.

Raw JSONL includes a header, each tick, all resolver movement/attack witness
rows, per-attempt rows, a summary, and full initial/final `State` checkpoints
with RNG state. The full checkpoints are bounded by the same 128 MiB raw cap
and allow post-run verification without repeating the World run.

## Frozen invocation and output schema

Build once with:

```sh
CARGO_TARGET_DIR=/tmp/curious-wave9-baseline-target cargo build --release --example refuge_risk_baseline_lab --jobs 1
```

The only authorized launch command is:

```sh
python3 data/operations/wave9-20261025/run_refuge_baseline.py
```

The runner invokes the prebuilt release executable with no arguments, under
`/usr/bin/time -v`, from the repository root. The runner's prelaunch manifest
records the source commit and input hashes, binary SHA-256, fixed command,
environment, output paths, and limits before it starts the process. It uses
`RLIMIT_AS=1,073,741,824` bytes, `RLIMIT_CORE=0`, `RLIMIT_FSIZE=134,217,728`
bytes, and a 300-second subprocess deadline. It writes to a new,
exclusive directory:
`data/experiments/refuge-risk-baseline-v1-engineering-20261025/`.

`raw.jsonl` contains one JSON object per line with exactly one of these
`record_type` values and corresponding payloads:

- `header`: protocol/run identity, input scenario hash, seed, horizon, raw
  cap, observation setting, refuge-feature flag state, evidence role, and
  expected-damage status.
- `state_checkpoint`: `when`, tick, RNG state, and a complete serialized
  `State`; emitted initially after the storage-only observation setting and
  finally after tick 1,000 and observer horizon censoring.
- `episode_start`: owner/threat IDs and species, start tick/position/body,
  decision/selected move, selected route index, and all observed route rows.
- `episode_tick`: owner/threat IDs, tick, positions and health before/after
  resolution, per-attempt movement/attack witnesses, cumulative physical
  quantities, and elapsed ticks.
- `episode_closure`: full start context, end state and tick, observer closure
  reason, completion/attribution flags, and accumulated outcomes.
- `resolver_events`: tick and every `EntityMoved` / `EntityAttacked` witness
  in the original resolver-event vector with its event index and physical
  fields. This allows independent audit of per-attempt event filtering.
- `tick`: tick number, active herbivore count, `SeekShelter` count, active
  attempt count, and total resolver-event count.
- `summary`: completed tick count, observer attempt/closure counts, eligible
  attributable-arrival sample count, five entity-ID grouped constant-mean
  diagnostics, effort/time diagnostics, and the interpretation boundary.

The lab exits successfully only after exactly 1,000 World ticks and final
summary/checkpoint emission. The runner records raw/stderr/resource-log hashes,
sizes, exit code, measured elapsed time, and terminal status for success,
error, signal, timeout, or byte-cap failure. It never retries. Technical run
completion is not an acceptance threshold for the damage-baseline model: the
runtime expectation remains `expected_threat_damage: null` and
`unknown_uncalibrated` for every route. There is no preregistered minimum
episode count, error threshold, utility threshold, route-ranking threshold,
or safety threshold. The only positive claim available is that the fixed
observation completed and produced the recorded physical measurements; sparse
or empty outcome samples remain Unknown.
