# Refuge physical baseline: one-run engineering observation

**Result:** the fixed run completed technically, but it did not calibrate an
expected-damage baseline. All 182 attributable arrivals had zero observed
threat damage, while all six attempts with positive threat damage were
censored. The damage value therefore remains **Unknown** for every route;
the zero-error grouped-mean diagnostic is uninformative under this
success-conditioned support.

This is one seen-seed (`123`) normal-World observation on the unchanged
`mixed-refuge.json` fixture. It does not change or replace the earlier failed
evidence, and it makes no safety, utility, causal, counterfactual, fitness, or
route-ranking claim. The refuge learner remained disabled; no expected-damage
model was accepted.

## Run and evidence

- Frozen experiment source: `0377d4c66d4ea81bf1ea83e0c5305c00849f963d`;
  engine source V18: `117a440431deead231ac6d4edbe324be2345936a`.
- Scenario SHA-256:
  `9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2`.
- Binary SHA-256:
  `73d9781001ea0cdd0644e1bc261afbd36687041093e4526a6192c7d0d46c753d`.
- Technical result: exit 0, 1,000/1,000 ticks, 1.929288 seconds runner
  elapsed (1.89 seconds `/usr/bin/time` elapsed), 7,680 KiB peak RSS,
  5,448,580 raw JSONL bytes. No timeout or cap was reached.
- Limits were 300 seconds, 1 GiB address space, zero core size, and 128 MiB
  raw/file-size cap. `observation.full_all=true` was set at tick zero for
  trace storage only. Refuge learning flags/tables remained disabled/empty.

Evidence files:

- [Raw JSONL](../../data/experiments/refuge-risk-baseline-v1-engineering-20261025/raw.jsonl)
  SHA-256 `d9c7e00e976c2fe31e0ad07868df924cabd423b91c6f5c32540b4b2cd43b4874`.
- [Run manifest](../../data/experiments/refuge-risk-baseline-v1-engineering-20261025/manifest.json)
  SHA-256 `b5161962b0939b6389f07f64d19458572703e55b721a3383b7eaf0202b2aef12`.
- [Independent audit](../../data/operations/wave9-20261025/baseline-independent-audit.json)
  SHA-256 `8e22728c1347a7b2fe91bd7c322591cf99424a999d6597e76a2c4361d55e00de`;
  status `passed`.
- [Resource log](../../data/experiments/refuge-risk-baseline-v1-engineering-20261025/resources.log)
  SHA-256 `2baa84f80d04ec4be0a08fbd39e56fcd382e91f5670f7219a3eff4bc20f4931e`.
- [Archived runner copy](../../data/experiments/refuge-risk-baseline-v1-engineering-20261025/run_refuge_baseline.py)
  SHA-256 `5c3b4614848ac2a7bf7e0420757944aa4e935d7ee26e325ca52138e1a0115ad8`.

The independent audit reconciled all 623 starts and closures, all 631
attempt-tick rows against the global movement/attack witnesses, accumulated
damage/health/energy/time, arrival endpoints, 1,353 route rows, five grouped
folds, and V18 initial/final checkpoints. Its stated limitation is that it did
not independently reconstruct the entire choice-time perception and route
eligibility; those were checked through frozen source and runtime assertions.

## Physical support and censoring

There were 631 ordinary `SeekShelter` decisions, 623 observer attempts and
closures, 182 arrivals (29.2%), and 441 censored attempts (70.8%). The 631
attempt-tick rows include repeated decisions during active attempts; they are
not independent samples.

| Closure | Attempts | Positive threat damage | Threat damage total | Positive other damage | Positive unexplained health loss |
| --- | ---: | ---: | ---: | ---: | ---: |
| Arrived | 182 | 0 | 0 | 0 | 0 |
| Threat lost | 165 | 0 | 0 | 0 | 6 |
| Target changed | 169 | 1 | 20 | 1 | 22 |
| Priority interrupted | 105 | 3 | 60 | 0 | 9 |
| Died | 2 | 2 | 40 | 1 | 0 |

All 182 arrived episodes were attributable and had zero threat damage. Among
all 623 attempts, six (0.96%) had positive threat damage, totaling 120; all
six were censored. Two attempts (0.32%) had positive other-attacker damage,
totaling 40, and 37 (5.94%) had unexplained health loss, totaling 132. Partial
damage and losses remain in the raw evidence but were excluded from completed
damage support. Censored attempts were never converted to zero-damage
successes.

Static contact geometry below is the selected route's current observed
Chebyshev proximity to the currently perceived threat cell. It describes the
decision-time layout, not predicted threat movement or attack exposure.

| Selected route cells currently within one cell | Attempts | Arrivals | Positive partial threat-damage attempts |
| --- | ---: | ---: | ---: |
| 0 | 460 | 150 | 3 |
| 1 | 94 | 27 | 3 |
| 2+ | 69 | 5 | 0 |

In a separate minimum-clearance grouping, 163 selected routes had current
minimum clearance 1 (32 arrivals, 3 positive partial-damage attempts), and
460 had clearance 2 or greater (150 arrivals, 3 positive partial-damage
attempts). There were no starts with clearance 0. The positive observations
are censored in every stratum. These small, selected-route counts do not
estimate route risk.

## Effort and damage diagnostics

For the 182 completed arrivals, nominal movement energy exactly matched
resolver-paid movement energy in 182/182 cases: 382 units total, mean 2.099,
mean absolute error 0.0. Nominal elapsed ticks also equaled observed time in
182/182 arrivals: 181 took one tick and one took two; mean 1.0055 ticks. These
checks validate the conditional effort arithmetic on this selected set. They
do not validate any damage prediction.

The five deterministic `owner_id % 5` grouped mean folds had 28–54 held-out
episodes each and training means, held-out means, MAE, and signed error all
equal to zero. Since every eligible outcome was zero and every positive
threat-damage observation was censored, **zero MAE is not evidence of model
accuracy, calibration, safety, or transfer**. It is the expected result of a
zero-valued target sample with informative censoring.

Every candidate route's expected threat damage remains `null`, status
`unknown_uncalibrated`. The public species attack amount was descriptive only;
no model or route ranking used it.

## Population context and next status

The initial checkpoint had 36 active herbivores and 2 active carnivores. The
final tick-1,000 checkpoint contained 41 herbivore entity records: 40 active
with positive health and 1 removed with zero health; no carnivore records
remained. Both checkpoints are V18 and the independent audit verified refuge
flags/tables remained empty. These counts contextualize this single run and
are not fitness outcomes.

The smallest measured gap is now concrete: selected-route effort and time have
descriptive validation, while expected threat damage has no positive
completed-attribution support. The existing residual prototype remains
unintegrated and default-off; its baseline requirement is still unmet. Any
next damage-calibration design needs informative, naturally observed completed
outcomes without changing bodies, scripted encounters, or treating censors as
successes. This run supplies no authority to tune, retry, or activate a
learner.

Cleanup after the run removed 1,289,639,393 logical bytes from inactive
wave9 targets and registry caches; measured filesystem usage fell by
1,195,802,624 bytes (about 1.11 GiB). Evidence:
[cleanup proof](../../data/operations/wave9-20261025/cleanup.json), SHA-256
`0f141c6354fcca897b92de92720938f13378eec41c530fe7918eff732a13ccc7`. The
release binary, raw data, frozen source, and backups were preserved.
