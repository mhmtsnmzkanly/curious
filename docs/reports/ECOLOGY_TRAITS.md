# Ecology cycles and paid-trait bounded screen

**Result:** schedule mechanics passed; the registered short fitness screen did
not demonstrate descendant lineage continuation or a trait advantage. This is a
bounded mechanism/screening result, not ecological qualification.

## Study and immutable evidence

The preregistered design crossed five fixed genotypes (control, mobility,
vision, stamina, vision+stamina) with static and geographically phased resource
availability. Each of eight held-out seeds (50001–50008) ran all ten arms in
separate fresh herbivore-only worlds with eight founders, no later spawns or
rescue, zero mutation, and a fixed 2,000-tick duration: 80 worlds and 160,000
evaluation ticks. The eight seeds had not appeared in the experiment-file audit
recorded in the manifest.

The run used source revision `260f853e0f8fce53f927fc25a47a9146e90b6390`,
source SHA-256 `02a5b298536ae5fbc8b53319be33ce5fcfa663c5aab54beedf058c3d2bdf9d3c`,
immutable probe SHA-256
`fbba5587e311a0a6e101befe7049ce68fca60bfa543bf7f7e9ea7ac521fb00ed`, and
frozen protocol SHA-256
`53b5af9d9563d39715d379883e2127e1e9bbdde1ea397ff75e39a1697d0686c5`.
Evaluation raw SHA-256 is
`c981f94d5aadd479f0b5158d9034be40943976ec609649dee2a05cefca108784`;
summary SHA-256 is
`5eff8e49d7cd443fa65ccb2d37f4c17b45b3a0df5885d805968aaf3acd906bb1`.
The run finished normally in 1.795 seconds, exit code 0, with empty stderr and
no timeout/output-limit error. Full manifest, protocol copy, immutable binary,
raw lines, tick samples, event counters, final states and summary remain under
`data/experiments/ecology-traits-v1-20261023/`.

Before evaluation, the complete 40-world calibration batch (seeds 50041–50044,
all ten arms, 80,000 ticks) was rerun with the same probe and settings. Both
calibration raw files have the exact SHA-256
`9eee0369fc44a934c18d526e85ceab32bc849ca2ad15b47ba0a7e14a15493b7d`;
`repeat-proof.json` records `match: true`, exit code 0 and the same probe hash.
The first calibration service invocation failed before the probe ran because
its service environment lacked `PATH`; it produced no world output or biological
result. The corrected service invocation completed, and only that successful
calibration plus its exact replay gated evaluation.

The frozen copy of `protocol.md` preserves its pre-run preregistration header;
the evaluation manifest and `runreport.md` record the completed execution. No
protocol, seed, parameter or threshold was changed after results were seen.

## Registered mechanism result

The 48-tick schedule check passed: 32 renewal events were checked against the
registered schedule, RNG state was unchanged, and a checkpoint at tick 17
restored and continued to exactly the same full state and RNG as uninterrupted
execution. Schedule settings were period 12, four active ticks, geographic
phase step 1 and offset 0. This supports the deterministic schedule and resume
mechanism for this check; it says nothing about population fitness.

## Fitness and lineage outcome

The primary outcome was the number of living descendants that had themselves
bred at tick 2,000. It was **zero in all 80 worlds**. All eight founders were
dead in every world; no F1 offspring had bred, there were no F2 births, and all
final checkpoints had empty entity lists. Thus no arm produced a living
reproducing descendant lineage. Births below are included as demography, not as
fitness success. Since all ten arms had the same zero primary outcome, this
screen cannot rank the trait arms by descendant breeding fitness.

F1 birth counts for every seed and arm are shown below. For every cell, living
descendant breeders at tick 2,000 = 0 and F1 offspring that bred = 0.

| Seed | Static control | mobility | vision | stamina | vision+stamina | Cycle control | mobility | vision | stamina | vision+stamina |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 50001 | 4 | 2 | 3 | 2 | 0 | 3 | 0 | 0 | 0 | 0 |
| 50002 | 3 | 3 | 2 | 2 | 0 | 2 | 0 | 1 | 0 | 0 |
| 50003 | 3 | 3 | 2 | 0 | 0 | 3 | 0 | 1 | 0 | 0 |
| 50004 | 3 | 2 | 2 | 0 | 0 | 2 | 1 | 0 | 0 | 0 |
| 50005 | 3 | 3 | 3 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| 50006 | 3 | 3 | 3 | 2 | 0 | 4 | 0 | 0 | 0 | 0 |
| 50007 | 3 | 4 | 3 | 2 | 0 | 2 | 0 | 0 | 0 | 0 |
| 50008 | 3 | 2 | 2 | 1 | 0 | 1 | 0 | 0 | 0 | 0 |

Across seeds, the static arms produced median F1 counts of 3 control, 3
mobility, 2.5 vision, 1.5 stamina and 0 vision+stamina. The cycle arms produced
2, 0, 0, 0 and 0 respectively. Every arm had median F2 births 0 and median
living descendant breeders 0. The static control had the most consistent F1
production (3–4); this did not yield a breeding descendant. The combined
vision+stamina arm had no births in either environment. These small descriptive
differences do not identify a broad trait effect, and there is no within-world
selection evidence: each genotype was tested in a separate fixed-genotype
population and mutation was zero.

Across the eight replicates, total births by static arm were 25 control, 22
mobility, 20 vision, 9 stamina and 0 vision+stamina; cycle totals were 18, 1,
2, 0 and 0. They are all F1 births in this run. The summary's `max_generation`
field is computed from entities remaining in the final state, so its zero value
in these empty worlds is not a historical maximum-generation measure.

## Requested and actually paid maintenance

Values are per-world medians with `[minimum–maximum]` across eight seeds. “Paid”
is energy actually charged after basal metabolism; “requested” is the registered
trait maintenance request. The shortfall is expected when energy cannot cover
the full request and must not be reported as a fully paid cost.

| Environment / arm | Vision requested | Vision paid | Stamina requested | Stamina paid |
|---|---:|---:|---:|---:|
| static / control | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| static / mobility | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| static / vision | 724 [660–762] | 515 [447–558] | 0 [0–0] | 0 [0–0] |
| static / stamina | 0 [0–0] | 0 [0–0] | 707.5 [634–728] | 554 [419–565] |
| static / vision+stamina | 576 [550–582] | 380 [340–386] | 576 [550–582] | 318.5 [282–332] |
| cycle / control | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| cycle / mobility | 0 [0–0] | 0 [0–0] | 0 [0–0] | 0 [0–0] |
| cycle / vision | 524.5 [516–547] | 239.5 [221–253] | 0 [0–0] | 0 [0–0] |
| cycle / stamina | 0 [0–0] | 0 [0–0] | 520.5 [513–532] | 225 [211–241] |
| cycle / vision+stamina | 479 [475–484] | 186 [179–206] | 479 [475–484] | 165 [159–168] |

The raw event-row metabolized counts agree with the number of maintenance audit
events. A single `EntityMetabolized` event records both vision and stamina
requests/payments when a body expresses both traits.
The recorded requested and paid totals show that the traits were not cost-free,
but paid amounts were materially below requests in every expressing arm. The
report does not infer an advantage from lower movement energy or raw food totals;
those are downstream outcomes in populations that all went extinct.

## Interpretation and limits

This screen is negative for the preregistered lineage-continuity outcome and
inconclusive about relative long-term fitness of the traits. It shows that
births alone overstate success: some arms produced F1 young, but no F1 bred and
none survived to the endpoint. It does not show a second generation, persistent
selection, population-independent evolutionary change, open-ended evolution,
or mixed-species coexistence. Eight seeds and 2,000 ticks are a bounded
mechanism/fitness screen, not long-term selection evidence or an acceptance
gate. Because mutation was disabled and the genotype arms were separate, no
adaptive evolution claim is supported. Most worlds were empty for part of the
2,000-tick interval, so the 1.795-second batch runtime is not a throughput
estimate for a sustained living ecosystem.

All individual seed/arm outcomes and source, binary, protocol, raw and summary
identities are indexed in [ECOLOGY_TRAITS.json](ECOLOGY_TRAITS.json). The
complete immutable evidence is in the experiment directories listed above.
