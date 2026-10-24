# Refuge observer memory repair — technical validation protocol

**Status: preregistered technical validation only.** This document authorizes
one bounded execution of the already committed memory-repair probe. It does not
resume, replace, or rehabilitate the failed `5ae985e` run. The original failed
manifest and 225-record raw artifact remain failed historical evidence. This
validation is not a new seed role, independent learning evidence, formal study,
or utility evaluation.

## Purpose and fixed inputs

Determine whether the committed observer-memory repair can complete the exact
previously attempted engineering workload within its already specified resource
limits, while preserving the earlier raw records and all declared treatments.
The only allowed probe/source is the committed fixed-probe snapshot associated
with repair source `3c2a83d` (main integration `e551263`), using retained binary
SHA-256
`9e2a0e25c1c992a49ba429b53a27e5d33f15e55e39b97708a6965a78e8586248`.
Record and verify the actual executable and snapshot hashes before execution.
No Rust build is needed or allowed as a substitute for that retained binary.

Use the existing immutable fixture `scenarios/mixed-refuge.json`, seen
engineering seed 123, focal herbivore 15, and the original protocol's unchanged
30-tick shared prefix plus four 60-tick suffixes (trained-online,
trained-frozen, naive-online, and rule-based), for exactly 270 planned World
ticks maximum. The original protocol and fixture remain authoritative for focal
selection, fork contents, treatment definitions, episode handling, and normal
World/resolver behavior. The source repair is limited to observer memory
representation. Do not alter the probe, source, binary, fixture, focal, seed,
body, age, flags, table, treatments, horizon, thresholds, or simulation behavior.

This is the one specifically authorized technical retry following the prior
allocation failure. It does not authorize another attempt if this validation
fails or stops.

## Pass criteria

Technical validation passes only if all of these hold in the same invocation:

1. Exit status is zero and the probe emits its completed-run summary.
2. Raw output contains all 270 complete, correctly ordered tick records: the
   shared prefix ticks 1–30 and 60 suffix ticks for each of the four declared
   arms, with no duplicate, missing, malformed, or out-of-order records. All
   four branch-end records and the final summary are present.
3. Every field in the original common 225-record prefix is exactly equal after
   parsing the original restored JSONL and the new JSONL record-by-record.
   The archived original has no source-hash/provenance fields requiring
   normalization, so this comparison is exact, including values and ordering.
   Any prefix mismatch fails validation; do not waive or normalize it.
4. The frozen arm's table values and metadata remain unchanged throughout its
   suffix, and the treatments and fork state match the original declarations.
   The full-run audit must confirm the original probe's frozen-value, body/age,
   transition, and branch-end checks against the new raw output.
5. The invocation stays within 300 seconds elapsed, a 1 GiB address-space
   ceiling, and 128 MiB raw JSONL. Capture peak RSS/resource usage as well as
   elapsed time and output bytes. Set the shell core-size limit to zero before
   starting the process.

These are technical completion and integrity gates only. The original refuge
opportunity gates remain descriptive outcomes to report: own completed
resolver-backed learning, different-threat-ID exact-support reuse, and an
aligned online/frozen greedy contrast. They carry no preregistered utility
threshold here. Whether those gates pass or fail does not change the technical
completion classification and cannot establish utility, benefit, ineffectiveness,
or biological rejection.

## Stop and evidence handling

Use a new, separately identified evidence directory and manifest. Record the
command, source/protocol/fixture/executable hashes, resource limits, raw output,
stdout, stderr, exit code, and start/end timestamps. Preserve partial output and
classify any nonzero exit, resource-limit hit, malformed or missing record,
prefix mismatch, treatment mismatch, or audit failure as a failed technical
validation. Stop immediately on such a failure; do not automatically retry,
rebuild, tune, change the fixture, or run another seed.

Keep the original `data/experiments/refuge-escape-v1-engineering-20261024/`
artifacts, original failed status, raw archive, snapshots, and the historical
`docs/REFUGE_ESCAPE_PROTOCOL.md` text unchanged. The successful result, if any,
belongs only to the new validation record and does not retroactively complete
the previous run. No live world, service, deployment, formal seed list, or
learning-evidence status may be changed by this validation.
