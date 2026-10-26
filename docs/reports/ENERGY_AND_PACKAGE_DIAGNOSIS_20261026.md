# Energy lock and package tradeoff diagnosis — 26 October 2026

This read-only diagnosis covers the failed V1 water calibration and the four
1,000-tick qualification soak arms. It does not change the source, run a World,
or accept a candidate package. Structured values are in
[ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.json](ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.json).

## Energy lock in calibration seed 62199

The raw record has one ordinary AI herbivore, initial energy 80/80, water 60/60,
food at `(0,-1)` with capacity 30/refill 2 every 2 ticks, and two water sites.
The agent's first `SeekFood` intent is tick 15 at energy 59; it pays 1 movement
energy and eats 10. The detailed resolver sequence at the lock is:

| Tick | Energy at choice | Position at choice | Intent and observed outcome |
|---:|---:|---|---|
| 65 | 20 | (1,-2) | `SearchFood`: moves 3 left, cost 3, ends (-2,-2) |
| 66 | 16 | (-2,-2) | `SeekFood`: path has 2 steps; moves 1, cost 1, eats 2, ends (-1,-2) |
| 67 | 16 | (-1,-2) | `SearchFood`: moves 2 right, cost 2, ends (1,-2) |
| 68 | 13 | (1,-2) | `SeekFood`: eats 2 in place |
| 69 | 14 | (1,-2) | `SearchFood`: moves 3 down, cost 3, ends (1,1) |
| 70 | 10 | (1,1) | `SeekFood`: moves 1 up, cost 1, eats 2, ends (1,0) |
| 71 | 10 | (1,0) | `SearchFood`: moves 3 up, cost 3, ends (1,-3) |
| 72 | 6 | (1,-3) | `SeekFood`: moves 1 down, cost 1, eats 2, ends (1,-2) |
| 73 | 6 | (1,-2) | `SearchFood`: moves 1 down-left, cost 1, reaches (0,-1) |
| 74 | 4 | (0,-1) | At the food site; eats 2 in place |
| 75 | 5 | (0,-1) | `SearchFood`: moves 3 down-right, cost 3, leaves the site for (3,2) |
| 76 | 1 | (3,2) | `SeekFood`: path has 3 steps; moves only 1, cost 1, reaches (2,1); approach rejected `OutOfRange` |
| 77 | 0 | (2,1) | `SeekFood`: path has 2 steps; no movement is paid, approach rejected `OutOfRange` |

Tick 65 is exactly the quarter-full low-energy threshold (`max_energy/4`), but
food-seeking was already observable at tick 15. Tick 74 really is at `(0,-1)`;
tick 75 starts there and leaves after consuming the available 2 units, so the
recorded movement is continuous. The route lock occurs at ticks 76–77: after
tick 75's three-step search movement, the remaining energy pays only one of
the three steps back toward newly replenished food. At zero energy the final
two-step approach cannot be paid. The site's stock then refills to 30 by tick
106, when starvation death is recorded.

`foraging::value` scores only visible current resource amount, path distance,
current energy/water capacity, move-energy cost, speed-derived travel ticks,
and the current meal cap. It has no term for expected future starvation,
body-specific reserve requirement, health trajectory, or whether the agent can
pay the full route before eating. It does not inspect hidden future resources.
The instinct policy asks for food well before zero (its food target is 75% of
maximum energy, or 90% for an injured individual), but selection of the
highest current forage score is not a route-feasibility guarantee. At zero
energy, `can_move`/the resolver cannot pay a positive movement cost, so the
visible food does not rescue the individual. The simultaneous later water
depletion and health loss are additional stresses; the final `Starvation`
label does not attribute every preceding health loss to starvation.

### Reserve planning boundary

A candidate reserve guard is worth designing as a bounded, default-off
comparison only if it uses current body state and currently perceived food/path
information. It must reserve enough energy to physically pay the planned
movement to the food and any registered body/maintenance requirement before
choosing discretionary movement or another costly action. The reserve is a
planning constraint, not energy creation: it cannot authorize an unpaid move,
forage beyond perception, predict future replenishment, or bypass resolver and
metabolism costs. A test must show that it preserves a reachable food option
without reducing reproduction eligibility or merely moving starvation to a
later tick. There is not enough evidence here to select a reserve formula or
change the engine.

## Four-arm package over 1,000 ticks

The diagnostic starts each arm from the same 616-live checkpoint at tick 12639;
the 10 samples then summarize each successive 100-tick interval. These are
paired deterministic trajectories, not independent population replicates.
Interval births, deaths, and living individual-ticks fluctuate in direction;
the endpoint alone hides those differences. Over the full horizon:

| Arm | Births | Deaths | End live | Founder live | Living individual-ticks | Δ individual-ticks vs control |
|---|---:|---:|---:|---:|---:|---:|
| Control | 5161 | 5197 | 580 | 15 | 595736 | 0 |
| Parental care | 5101 | 5142 | 575 | 12 | 591932 | -3804 |
| First-interval refuge | 5121 | 5153 | 584 | 15 | 593113 | -2623 |
| Candidate bundle | 5146 | 5169 | 593 | 15 | 594308 | -1428 |

The bundle ends with 13 more live individuals than control, but produces 1,428
fewer living individual-ticks. It also has 15 fewer births and 28 fewer deaths.
The structured companion records, at each 100-tick boundary, each arm's
birth, death, living individual-tick, and live-count delta against control.
The traces show local reversals; for example, the bundle has 38 fewer live at
tick 12839 but 36 more at tick 13439 than control. No arm is a general
Pareto improvement across endpoint population and accumulated survival. These
population differences do not imply that lower population is inherently bad:
environmental stress can reduce abundance while improving or worsening other
ecological outcomes. They establish a tradeoff that needs explicit outcomes,
including reproduction, for each bounded ecology study.

## Decision and next bounded evidence

No code or package change is recommended from these data. The approved next
work is a fixed 600-tick ecology transfer study with control and
one-feature-at-a-time arms for climate, ambush, alarm, and extra traits,
crossed with three named habitats and three fresh seeds per habitat. Measure
births, deaths, survival exposure, and offspring that themselves reproduce;
retain physical stress and resource outcomes alongside population counts. The
[Ecology transfer V1 protocol](../ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md)
is approved in scope but awaits frozen source identity and run registration;
no evaluation World has been run.

## Inputs and limits

- `data/experiments/water-adaptation-v1-calibration-20261025/seed-62199.raw.json`
  and its manifest; original V1 report records that calibration failed and
  formal seeds were not run.
- `data/experiments/qualification-diagnostic-20261025/soak-four-arms.json`;
  comparison and raw interval samples are preserved in the local experiment
  directory.
- Source inspected: `src/entity/foraging.rs`, `src/entity/instinct.rs`,
  `src/entity/behavior.rs`, `src/entity/lifestate.rs`, and
  `src/systems/metabolism.rs`.

This is a source-and-artifact inspection, not a rerun. It cannot establish that
a reserve guard improves outcomes, nor that a package or feature transfers to
other worlds.
