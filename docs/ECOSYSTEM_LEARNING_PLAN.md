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
| 1 | Opt-in hunt injury-risk history, causally tied to resolver-confirmed prey defense and a choice rule with unknown evidence kept unknown. Preregistered six-arm lab compares food, injury, deaths/final health, and energy. | V3 source and bounded runner completed at `67a4e14`; 131 Rust/77 Python and disabled state/RNG fidelity passed. Bounded evaluation completed; risk policy did not reduce injury outcomes (see `reports/HUNT_LAB_V3.md`/JSON). Feature stays default-off; no tuning or deployment. |
| 2 | Same individual retains its actual identity, age, body history, and learner through resource/predator reversal and checkpoint/resume. Audit raw actor state and RNG equality across uninterrupted/resumed branches. | Bounded run completed at source `ba71b7f`: 1,529/1,664 physical ticks; all eight checkpoint/resume states and RNG matched. Seven subjects died naturally; four had sparse post-reversal water/table updates. Online and frozen had identical decisions and physical outcomes. The result supports continuity only, not a lifespan or learning advantage; see `reports/LIFETIME_CONTINUITY.md`/JSON. |
| 3 | Extend existing danger memory for herbivore escape/shelter choices after observed threat. Measure resolver-paid movement, own received damage, safe arrival, censoring, and blocked/occupied destinations; do not reveal future threat positions. | Source/protocol `60ab151`; 153 Rust tests, 5 harness tests, calibration (seed 45999, 134 ticks), formal held-out run (1,148/1,600 ticks), and immutable-raw validator-only reanalysis completed. Every seed's paired summaries and all 574 paired physical ticks matched. The only repeated comparable context was seed 45002 at ticks 66 and 82; both episodes were censored (`threat_lost`, then `died`), so neither was verified safe and no improvement comparison is identifiable. No behavioral or physical benefit was observed; keep the feature default-off. See `reports/HERBIVORE_DANGER.md` and `HERBIVORE_DANGER_REANALYSIS.md`. |
| 4 | Population-independent spatial/temporal resource schedule with deterministic site offsets and a closed-feature no-RNG equivalence check. | V15 schedule mechanism check passed. The preregistered 80-world / 160,000-tick screen had zero reproducing descendant lineages in every arm; no relative fitness ranking is supported. Exact repeated calibration raw hashes matched before evaluation. Keep schedules opt-in; see `reports/ECOLOGY_TRAITS.md`/JSON. |
| 5 | Individual decision timeline for seen/remembered evidence, selected action, observed cost, and resulting update; compare first/last genuinely similar subjective encounters. | Implemented as the read-only documentary viewer in `4c2cef7`, with bounded-panel refinement in `5736e6d`; independent browser and 75 Python UI tests passed. The deployed read-only viewer uses the frozen `ba71b7f` release and presents observed history rows, not a claim of complete lifetime cognition. Health showed 3 individual-history rows and zero world restarts; no Rust server upgrade. |
| 6 | Separate inherited-allele lab for sight, speed, and endurance. Charge expressed traits through live perception/resolver/metabolism budgets, then measure survival and reproduction costs. | Separate V15 paid-trait arms were included in the ecology-traits preregistration; requested and actually paid vision/stamina costs were measured. The combined vision+stamina arm had no births, and no arm produced a reproducing descendant lineage. No trait advantage or evolutionary benefit is accepted; keep the feature default-off and see the ecology-traits report. |

Each phase produces the smallest implementation that can answer its stated
question, tests its causal measurement, and pins commands, seeds, source and
protocol SHAs, manifest schema, tick ceiling, and stop rule before evaluation.
No parameter sweep or tuning after seeing outcomes. If a lab cannot identify
the claimed effect with current measurements, narrow the claim or design a
separate preregistered lab before changing the feature.

## Phase 2 follow-up — continuous-lifetime water choices

The first lifetime-continuity result above established same-ID/body/age/learner
checkpoint replay but found no decision or physical difference in eight short
reversal histories. A bounded follow-up is preregistered in
[`LIFELONG_WATER_PROTOCOL.md`](LIFELONG_WATER_PROTOCOL.md), source commit
`02b3d89`. It uses one 200-tick uninterrupted experience prefix, a checkpoint,
and four counterfactual suffixes (online, same-table frozen, table-ablated
frozen, rule-based) in the same physically renewable water-competition
fixture. The sole candidate implementation is a lab probe and isolated
serialization/replay tests; no core policy or saved-state behavior changes.

The follow-up aims only to demonstrate a causally attributable decision and a
physical water/energy/survival effect in at least one held-out continuous
lifetime. It does not establish general learning efficacy. Root audited
calibration seed `51099` and formal seeds `51001`–`51008` as unused before any
new run. The calibration probe then captured 1,400 ticks for seed `51099`, but
the first validator failed on an incorrect assertion that initial RNG equal the
input seed. `World::with_seed` consumes deterministic RNG calls while
populating its temporary map before the probe installs the fixed fixture map.
The raw capture, failed manifest, probe, and protocol remain unchanged. A
zero-tick validator reanalysis is documented in
[`LIFELONG_WATER_REANALYSIS.md`](LIFELONG_WATER_REANALYSIS.md), with separate
artifacts under
`data/experiments/lifetime-lab-v2-calibration-reanalysis-20261023/`. No biology,
fixture, threshold, gate, or budget is being changed. The first formal
preflight stopped before the probe because the shared seed scanner's default
16 MiB line cap rejected the valid 19.44 MB calibration record, despite the
registered 64 MiB per-seed limit. No formal output directory or world ticks
were created. Root is fixing the scanner's configurable bound and will repeat
the calibration-only raw validation with zero world ticks in
`data/experiments/lifetime-lab-v2-calibration-reanalysis2-20261023/`; see
[`LIFELONG_WATER_PREFLIGHT_REPAIR.md`](LIFELONG_WATER_PREFLIGHT_REPAIR.md).
Seed `51099` is used only for calibration; formal seeds `51001`–`51008` remain
unused. Root must review the current source-bound reanalysis and full seed audit
before authorizing another formal attempt.
Root owns source/protocol commits, seed freshness scans, formal authorization, and
the installed completion timer. The v2 completion helper watches only
`lifetime-lab-v2-20261023` and schema
`curious-lifetime-lab-run-v2`/protocol 2; its active runtime is external to
this repository.

## Phase 3 evidence and limits

The fixed calibration used 134 physical ticks of its 200-tick ceiling. The
formal run used all eight held-out seeds and 1,148 ticks of the 1,600-tick
ceiling; all 16 branches ended by natural death at ticks 62–82. A validator
enum-spelling defect initially marked the run failed after the probe had
completed. The analysis was corrected and rerun from the immutable original
raw capture only: zero new simulation ticks, no probe rerun, and no biological
parameter changes. The original failed manifest and raw data remain preserved;
the corrected analysis and provenance are in
`data/experiments/herbivore-lab-v1-reanalysis-20261023/`.

Across 574 paired ticks, physical state and RNG matched after removing only
the four refuge-memory fields. Decisions, perception, body state, and physical
resolver events also matched after excluding the new memory audit event. All
paired seed summaries matched. The only repeated comparable visible context
was seed 45002 at ticks 66 and 82. The closures were `threat_lost` and then
`died`, and both were censored: `safe_arrival=false` does not establish that an
episode was unsafe. Thus no first/last improvement comparison is identifiable.
The lab found no observed behavioral or physical benefit, with little
repeated-context evidence for assessing learned improvement. Keep refuge
outcome learning default-off; do not infer efficacy from the mechanism or
update counts.

The overall measured phase-3 work used 134 calibration ticks plus 1,148 formal
ticks (1,282 total), below the preregistered 1,800-tick combined ceiling.
Validator-only reanalysis consumed no additional biological ticks.

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

Root committed the preregistration before source implementation. The V3 run
completed in 12 seconds: 10,240 trials and 99,729 physical ticks. The report
and summary are preserved under `reports/HUNT_LAB_V3.md` and `.json`; original
raw output, probe, and manifest remain under
`data/experiments/hunt-lab-v3-20261023/`. Results reject the preregistered
risk-benefit hypothesis for this policy: on the defensive normal window,
risk-online had 248/320 injured trials and 3,558 prey-defense damage versus
215/320 and 3,276 for plain experienced-online, with higher energy spent
(7,505 vs 6,043). After reversal, risk-online had 246/320 injured trials and
1,476 damage versus 218/320 and 1,308. Keep risk choice disabled in live
settings; do not tune these parameters based on this result.

The bounded completion timer is installed and active
(`curious-hunt-lab-completion.timer`); root owns systemd changes. Its actual
V3 terminal poll queued exactly once (queue ID
`01a16693-86a7-7953-91c2-7b9656de2478`, one attempt; root verified the queue
database and retained completion journal/dedup evidence). It uses
a frozen helper, external runtime configuration, explicit watched run paths,
terminal manifest and content-hash checks, durable outbox, and native
`codex queue` delivery to the configured user/thread. The retired V1 helper/config remains as V3 delivery evidence. The frozen V2
helper is installed with three active completion timers for lifetime,
herbivore, and ecology-traits labs. Its source and 12-test suite accept only
explicit bounded lab schemas, one exact watched run, and an allowlisted
immutable artifact filename (`hunt-lab-probe` or `lab-probe`). Runtime configs
and frozen SHA are outside git and root-owned. New lab runs must use the
matching schema, watched path, and probe filename.
Timer completion only queues a message; root must recheck run identity and
evidence.

The live worlds on 7878–7883, frozen/bin releases, and historic/raw lab
archives are outside this work. Biology defaults, closed-state behavior and
RNG order are acceptance constraints. New saved-state fields require an
explicit version and migration; learned experience never transfers to offspring.
