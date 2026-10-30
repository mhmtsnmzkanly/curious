# Refuge represented-context opportunity — zero-tick feasibility protocol

**Status: design proposal only; no new World run recommended.** The smallest
justified next step is one read-only diagnosis of the complete, archived repair
validation output. The current evidence does not make another physical lab
credible under the requested 1,000-tick ceiling: the fixture's focal max age
leaves only 15 additional ticks per branch beyond the completed run, while
the learner requires exact represented-key equality and a 100-tick row TTL.
Do not launch a World invocation under this proposal.

The bounded offline reader is
[`scripts/diagnose_refuge_opportunity.py`](../scripts/diagnose_refuge_opportunity.py);
the output report draft is
[`REFUGE_OPPORTUNITY_DIAGNOSIS.md`](reports/REFUGE_OPPORTUNITY_DIAGNOSIS.md).
The reader has not been run against experiment data; root should freeze and
review its source before running it.

## Question

Do the complete seed-123 records contain any natural post-sample candidate
opportunity in which a different observed threat ID has at least one candidate
whose represented key exactly matches a completed, still-fresh sample, and do
they contain all-candidate support or an aligned online/frozen choice contrast?
If none exists in the existing full record, is there enough remaining
same-individual lifetime to justify another fixed-scenario World run?

This is an opportunity-identifiability question. It is not a learner utility,
causal transfer-safety, ecological qualification, or biological rejection
test.

## Evidence and identity

Read the immutable evidence; do not replace or amend it:

- Original incomplete run: `data/experiments/refuge-escape-v1-engineering-20261024/`.
  Its manifest remains `failed`; raw SHA-256 is
  `15ce046035061e24fbd969cc9056ae32cc8f423926705b5e168c55a0a830c4a0`.
- Completed technical validation: `data/experiments/refuge-repair-validation-20261024/`.
  Its raw archive index is
  `raw.jsonl.archive.json`; original raw SHA-256 is
  `de40e6f9a14acdb62a0be0d7ea93371511265e7423b0691f3e858cff06e2946a`.
  Restore the verified JSONL before using any frozen reader that requires the
  original path.
- Repair source: `3c2a83d`; fixed probe SHA-256:
  `9e2a0e25c1c992a49ba429b53a27e5d33f15e55e39b97708a6965a78e8586248`.
- Fixed fixture: `scenarios/mixed-refuge.json`; seen engineering seed 123,
  focal herbivore 15. No seed role is added or reclassified.

The complete validation recorded 30 shared-prefix ticks and four 60-tick
suffixes: 270 physical tick records but world tick/age labels only through 90.
It matched all 235 common JSONL records, including the earlier 225 tick
records. The root audit reports zero post-prefix supported fresh greedy
choices, zero different-ID supported choices, and zero aligned online/frozen
contrasts. The original partial diagnostic saw only threat ID 37 and one
post-prefix online choice at tick 56; its four candidates differed from the
completed row from tick 7 in eight or nine exact-key fields. In the complete
validation's fresh choice records, the selected threat was also ID 37.

The represented key excludes IDs and the current alternative set, but exact
equality still requires the same species, threat health/power/bearing/distance,
subject health/energy/stamina/speed and maxima, move cost, origin habitat,
endpoint sector, ordered route cell kinds, refuge kind, turn count, and
estimated movement energy. Rows expire after 100 ticks. The fixed fixture's
focal starts at age 0 with max age 105. No evidence shows a different ID
producing an exact reuse opportunity in the existing record.

## Proposed zero-tick audit

This diagnosis reads only existing JSONL records, their archived metadata, and
the frozen key implementation. It executes no `World::tick`, invokes no policy
decision, and changes no source, simulation state, learner table, fixture, or
service. It may write a separate derived audit; that output is descriptive and
must retain input hashes and the raw archive index.

For every focal fresh shelter choice and every candidate, report:

1. Current threat ID, decision tick, branch/arm, candidate index, and whether
   it is fresh greedy or fallback.
2. Every earlier completed row's source threat ID, completion tick, age at
   choice, and whether its row is within the 100-tick TTL.
3. Exact full-key equality and the exact list of differing fields when unequal.
   A different ID only counts as possible reuse if the candidate key is exactly
   equal and its source sample has known resolver-backed attribution.
4. Whether all candidates have unique fresh exact support; whether a
   post-prefix greedy choice would have been supported; and whether online and
   frozen records contain the preregistered aligned-state contrast. Do not
   impute missing source IDs, matching keys, or physical outcomes.
5. The latest tick where a completed row is still eligible and the focal's
   recorded age/max-age margin at that point.

Reconcile derived counts to the existing root audit and summary. If any count
disagrees, stop the diagnosis and report the discrepancy; do not repair raw or
change the original failed manifest. No parameter sweep, new seed, fixture
edit, table injection, threshold change, or utility analysis is included.

After root freezes the source, the intended invocation is:

```sh
python3 scripts/diagnose_refuge_opportunity.py \
  data/experiments/refuge-repair-validation-20261024/raw.jsonl.zst
```

The pure synthetic fixtures are available with `--self-test`; they test exact
key matching, source-ID linkage, and the inclusive 100-tick TTL boundary, and
do not create a World.

## Feasibility decision and stopping gate

For the unchanged four-arm design, extending each suffix from 60 ticks to the
focal's remaining lifetime allows at most 75 suffix ticks after the 30-tick
prefix: `30 + 4 × 75 = 330` physical ticks. That reaches age 105, the fixture's
max age. Compared with the completed validation, this adds only 15 world ticks
per branch. The example completed row at tick 7 remains within its 100-tick TTL
through tick 107, but the focal reaches max age at 105. Thus at most 15 more
ticks are available in the current fixture before the focal's age boundary;
spending a 1,000-tick budget on longer branches cannot extend that individual's
usable lifetime. The observed run found no ID 38 choice and no exact candidate
support, while its later candidate keys were far from the prior row.

This makes a new run on the unchanged scenario a poor use of the authorized
budget. Do the zero-tick audit and stop. If it confirms the existing audit's
zero different-ID exact-key opportunities, mark the current fixture
non-identifying and do not run another World. If it finds a genuine recorded
exact-key, different-ID opportunity overlooked by the root summary, report it
for root review; that finding alone still does not authorize a new World or
support a utility claim.

No scenario change at tick zero is proposed. Moving founders or resources to
stage successive threats at the same represented context would be a new
environmental intervention whose realism and opportunity rationale are not
established by the existing evidence. A separately reviewed scenario could be
considered later, but it would need its exact finite layout and rationale frozen
before any output. In any future World lab, agents must use ordinary AI and the
normal perception, intent, resolver, and event paths; from tick zero onward no
body, age, resource, learner context, or memory may be changed. No attacks,
routes, threat targets, candidate keys, or table rows may be scripted or
injected.

## Limits if a later World protocol is separately proposed

Any later proposal must be reviewed and frozen before data collection, use one
finite fixed scenario and seen engineering seed 123 only for engineering, and
cap total physical ticks at 1,000, wall time at 300 seconds, address space at
1 GiB, and raw JSONL at 128 MiB, with core dumps disabled. It must stop at the
first cap, focal death, or predetermined horizon; preserve partial raw evidence
on infrastructure failure; and prohibit automatic retries, parameter sweeps,
new seeds, formal-seed reuse, or post-result retuning. These are upper bounds,
not a recommendation to consume them. This document does not register formal
seeds, preregister utility thresholds, authorize a World run, or authorize
deployment.
