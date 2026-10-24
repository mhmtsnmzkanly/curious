# Refuge observer memory repair — technical validation

**Result: technical validation passed.** The fixed probe completed all 270
planned World ticks within the preregistered resource limits, emitted all four
arms and a summary, and passed the root audit. This validates technical
completion and integrity for this one fixed, previously seen engineering
fixture. It does not complete or replace the original failed run, add
independent learning evidence, or establish refuge utility.

## Run identity and integrity

The separately identified run is
`data/experiments/refuge-repair-validation-20261024/`. It used seen engineering
seed 123, focal herbivore 15, repair source `3c2a83d`, protocol commit
`b082f825a059c9c5a77427b091dfd5cb70399886`, and the retained probe whose SHA-256
is `9e2a0e25c1c992a49ba429b53a27e5d33f15e55e39b97708a6965a78e8586248`. The
protocol hash is
`65eca7de0b774f79d8403ce2af899156fbcc3ca2b41d3c7e400ea4ad2f46dee6`.

The command exited 0. It recorded 270 complete ordered tick records, all four
branch ends, and the final summary. The root audit verified all treatments and
unchanged frozen rows, continuous recorded ages, and matching provenance
replays. The first 235 common JSONL records matched the prior raw exactly,
including all 225 shared tick records. New raw output is 59,886,938 bytes with
SHA-256
`de40e6f9a14acdb62a0be0d7ea93371511265e7423b0691f3e858cff06e2946a`.

The run took 1.578 seconds and used 44,480 KiB peak RSS. Limits were 300
seconds, 1 GiB address space, 128 MiB raw output, and zero core-file bytes.
The raw artifact was losslessly archived from 59,886,938 to 3,353,735 bytes.
Original SHA-256 is
`de40e6f9a14acdb62a0be0d7ea93371511265e7423b0691f3e858cff06e2946a`; archive
SHA-256 is `b32208ad3c36f1710911360dee8c60e57553f8ab3281938ca342a925075b8749`.
The verified index is
`data/experiments/refuge-repair-validation-20261024/raw.jsonl.archive.json`.
The stderr, resource log, manifest, and root audit are also retained; stderr is
empty. Restore `raw.jsonl` from its verified archive before using frozen readers.

## Recorded fixture outcomes

The run had 30 shared-prefix ticks followed by 60 ticks in each suffix arm.
Across the arms, the audit reports 3 focal update events, zero post-prefix
supported fresh greedy choices, zero different-threat-ID supported choices,
and zero aligned online/frozen greedy contrasts. The opportunity gate is
`engineering_opportunity_not_identified`.

| Arm | Ticks | Terminal health / energy / water | Movement energy | Food / water acquired | Arrivals | Accepted updates |
|---|---:|---:|---:|---:|---:|---:|
| Shared prefix, online | 30 | 120 / 58 / 46 | 32 | 40 / 16 | 1 | 1 |
| Trained online | 60 | 120 / 34 / 27 | 36 | 92 / 56 | 1 | 1 |
| Trained frozen | 60 | 120 / 34 / 27 | 36 | 92 / 56 | 1 | 0 |
| Naive online | 60 | 120 / 34 / 27 | 36 | 92 / 56 | 1 | 1 |
| Rule based | 60 | 120 / 34 / 27 | 36 | 92 / 56 | Not listed in audit | 0 |

These are descriptive outcomes from one engineering fixture. No utility
threshold was preregistered for this technical validation. In particular, the
absence of post-prefix supported choices means it did not identify a learned
refuge choice opportunity; it is not evidence of benefit, ineffectiveness,
causal mechanism, or biological rejection.

## Relationship to the earlier failure

The earlier run remains an infrastructure failure: it has 225 tick records,
with the rule arm incomplete at 15/60, and its original manifest remains
`failed`. Its manifest SHA-256 remains
`2ea23a6f62e036c66952a2215fa59ac6dcd66ec5bb5f67514d91b84a040bf395`; its raw
SHA-256 remains
`15ce046035061e24fbd969cc9056ae32cc8f423926705b5e168c55a0a830c4a0`. The new
technical run neither resumes nor retroactively completes that artifact. No
formal seed roles were consumed and no live deployment was made.

The controlling protocol is
[REFUGE_REPAIR_VALIDATION](../REFUGE_REPAIR_VALIDATION.md). Machine-readable
results and evidence hashes are in
[REFUGE_REPAIR_VALIDATION.json](REFUGE_REPAIR_VALIDATION.json).
