# Refuge represented-context opportunity — diagnosis draft

**Status: draft; the zero-tick diagnosis has not been run.** This report is
reserved for one read-only analysis of the archived seed-123 repair-validation
raw data. It will describe exact candidate-to-row matches, source threat IDs,
row age/TTL, and focal age/max-age from recorded decisions. It makes no new
World run, formal seed, utility claim, or biological conclusion.

The analysis tool is
[`diagnose_refuge_opportunity.py`](../../scripts/diagnose_refuge_opportunity.py).
It streams JSONL or Zstandard JSONL, enforces 128 MiB decompressed input, 16 MiB
per-line, and 100,000 candidate/row comparison limits, hashes input bytes, and
checks a neighboring verified archive index when available. It scopes prefix
completed samples as common history and clones that history independently at
each treatment start; suffix samples never leak between arms. Candidate support
is recalculated only from rows present in that choice's recorded focal table.
The script includes four pure synthetic fixtures for closure/row source linkage,
exact-key different-ID support, TTL boundary behavior, and key mismatch fields.
Root should run those fixtures and freeze the source before running the archived
diagnosis; they have not been run by this draft's author.

Input evidence is the verified archive
`data/experiments/refuge-repair-validation-20261024/raw.jsonl.zst` and its index.
The analysis must restore raw JSONL only if a frozen reader requires the
original path. It must not change or overwrite either historical run. The
existing root summary reports zero post-prefix supported fresh greedy choices,
zero different-threat-ID supported choices, and zero aligned online/frozen
contrasts; the diagnosis will verify per-choice support counts against the
recorded summary and provide candidate-level key differences/source IDs.

The focal is age-limited to 105 ticks. The completed run reaches age 90; an
unchanged four-arm continuation could add at most 15 ticks per arm before the
age boundary, for 330 physical ticks total including its 30-tick shared prefix.
The strict represented key also includes changing body, threat, route, and
estimated-cost values, and rows expire after 100 ticks. This makes another
World run on this fixture unpromising; the archived-data diagnosis is the
decision gate. A zero-tick finding of no exact different-ID support means this
fixture remains non-identifying, not that transfer is generally impossible.

**Results pending root execution.** Do not fill in counts until the tool's
output, input hashes, and root review are available. The protocol and rationale
are in [`REFUGE_OPPORTUNITY_PROTOCOL.md`](../REFUGE_OPPORTUNITY_PROTOCOL.md).
