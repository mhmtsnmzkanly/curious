# Refuge represented-context opportunity diagnosis

**Result: the archived fixture contains no exact different-ID reuse opportunity.**
The bounded, read-only diagnosis reconciled to the complete repair-validation
summary: across eight fresh choice records, it found zero exact candidate
matches to a completed row, zero candidate support from a different threat ID,
and zero post-prefix choices with all candidates supported. This is a
descriptive result for the one seen engineering fixture. It does not show that
represented-context reuse is generally impossible, establish utility, or
change the status of the earlier failed run.

## Evidence and method

The reader was committed as `1c326ad6def2ae70e0cc7416ed8a88f737f91b4c` and
ran against the verified archived raw from the seed-123, focal-15 technical
validation. It streamed archived records only; it did not run a World tick,
invoke a policy decision, or mutate simulation state. The output status is
`witness_reconciled`.

The original JSONL SHA-256 is
`de40e6f9a14acdb62a0be0d7ea93371511265e7423b0691f3e858cff06e2946a` (59,886,938
bytes). Its Zstandard archive SHA-256 is
`b32208ad3c36f1710911360dee8c60e57553f8ab3281938ca342a925075b8749` (3,353,735
bytes); the neighboring archive index verified the restoration metadata. The
derived diagnosis JSON SHA-256 is
`80faff2cc4f31f82e92f63add890f77f0cfb2f5ee2d972ebad79bac36bc8878c`.
Restore the original JSONL from that verified archive before using frozen
readers that require the uncompressed path.

The reader compared each recorded fresh-choice candidate with completed rows
present in that choice's recorded focal table. It kept common-prefix samples
separate from each suffix arm and reconciled calculated support witnesses with
the raw summary. The input contained 30 prefix ticks, 240 suffix ticks, and all
four branch ends. All eight fresh choices selected threat ID 37. The report
contains 17 candidate-to-row comparisons; all 17 rows were still within the
100-tick TTL, but none had an exact represented-key match. Their source
closures identified threat ID 37. The near matches differed in route,
endpoint, threat geometry or power, movement estimate, subject energy, or
turn count, as listed per candidate in the machine-readable output.

## Reconciled counts

| Measure | Diagnosis | Recorded summary | Reconciled |
|---|---:|---:|---|
| Post-prefix choices with every candidate supported and fresh | 0 | 0 | Yes |
| Post-prefix choices with different-threat-ID support | 0 | 0 | Yes |
| Candidate exact different-ID supports | 0 | — | Yes; no exact key match |
| Candidate witness mismatch choices / fields | 0 / 0 | — | Yes |
| Aligned online/frozen contrasts | 0 recorded | 0 | Recorded count agrees |

The aligned online/frozen contrast was **not independently recomputed** by this
diagnosis; only the recorded zero was checked against the summary. Likewise,
the result is restricted to the recorded choices, candidate tables, and
completed rows. No claim is made about unrecorded states or other scenarios.

The focal's recorded age reached 90 of max age 105 in each suffix arm. Thus the
unchanged fixture had only 15 more focal-age ticks available per arm after the
completed run. This supports stopping further physical work on this fixture;
the diagnosis does not establish a general limit on transfer or reuse.

## Historical run status

The original run remains an infrastructure failure with its original manifest
marked `failed`; the repair-validation run is a separate technical validation,
and this diagnosis is a separate derived read-only artifact. None resumes or
retroactively completes the failed run. The repair-validation report is
[REFUGE_REPAIR_VALIDATION](REFUGE_REPAIR_VALIDATION.md), and the controlling
design proposal is [REFUGE_OPPORTUNITY_PROTOCOL](../REFUGE_OPPORTUNITY_PROTOCOL.md).

Machine-readable counts, per-choice candidate comparisons, and hashes are in
[REFUGE_OPPORTUNITY_DIAGNOSIS.json](REFUGE_OPPORTUNITY_DIAGNOSIS.json). The
source reader is [diagnose_refuge_opportunity.py](../../scripts/diagnose_refuge_opportunity.py).
