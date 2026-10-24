# Wave 5 genotype lifecycle observer

**Status: bounded observer helper integrated and validated.** This work adds a
pure observer helper for future genotype and lineage audits. It is read-only
input: it consumes explicit pre-tick and
post-tick checkpoint states, simulation events, and recorded decision traces.
It does not construct or advance a `World`, feed information into policy, alter
traits, or perform trait-selection analysis. The three helper/contract files
were committed as source revision `6f60d9c`.

## Scope

The helper keeps an ID-keyed ledger of the initial cohort and observed
individuals, including species, mobility/vision/stamina alleles, generation,
parents, current expressed vision and stamina, lineage status, births, deaths,
and mating. It aggregates event-reported requested and paid vision/stamina
costs, plant and corpse food, water, and food entries present in recorded
perception traces. Missing traces or missing perception are marked as unknown
coverage rather than interpreted as zero encounters.

Lineage joins use the supplied event stream and before/after states. Mismatches
and missing joins are retained as explicit issues. Mutation-aware transmission
checks are limited to what the configured genetics settings permit; the helper
does not infer which mutation occurred. It does not infer basal metabolism,
causal trait benefit, neutral phenotype, or inheritance of learned information.
The observer caps retained IDs at 50,000 and issue records at 2,000; failed
bounded observations leave the prior ledger unchanged.

The three helper and contract paths are:

- [genotype audit observer](../../examples/support/genotype_audit.rs)
- [pure fixture check](../../examples/genotype_audit_check.rs)
- [observer contract](../GENOTYPE_AUDIT.md)

These files define a reusable measurement component and hand-built pure
fixtures. They are not an experiment runner, a World driver, or a trait-selection
implementation. No genotype experiment, seed assignment, fitness claim, or live
change is part of this work. The six new genotype fixture tests construct no
World and execute zero World ticks; the full suite includes existing World
tests.

Root-bound input hashes are recorded in
[`wave5-integration-inputs.json`](../../data/operations/wave5-integration-inputs.json):
the observer source hash is
`1bc4def4bc28844b7c68ae9c22d54314d48c0180daabd7544c67900502240ea0`, the pure
fixture source hash is
`99726dd0c96dfa2435547a47e8e99c22dea77402dcb83e9011e67c8d4de35f68`, and the
contract hash is
`bc8c682f535b2fa8f211bdee0cff084a86c821017128bdf7fe6bdfe502a9cb87`.
Root's review fixes cover reseed/preseed guards, invalid alleles, configured
genetic transmission rules, exclusion of unknown lineage from descendant
counts, private ledger accessors, and atomic issue-budget rollback.

## Validation status

The fixture file contains focused cases for event-backed cost and acquisition
totals, missing-trace coverage, same-tick lineage joins, mutation-aware genome
transmission, error-cap atomicity, invalid inputs, and retained deaths. Root
reports `cargo test --all-targets` passed with 241 tests across 46 summary
groups, zero failures; Python passed 177 tests, zero failures. Rust formatting
for the two Rust files and `git diff --check` passed. The release build passed
with exit 0 in 1 minute 44 seconds; its log is
`data/operations/wave5-release.log`. The proposed focused command is documented
in [`GENOTYPE_AUDIT.md`](../GENOTYPE_AUDIT.md); the pure observer fixtures
execute zero World ticks and do not establish World behavior or trait fitness;
existing World tests in the full suite are separate.

Cleanup removed 608,011,561 logical bytes (about 579.84 MiB) from the inactive
`/tmp/genotype-audit-wave5-target`. Proof:
`data/operations/wave5-cleanup-20261024.json`. Live PIDs 1126 and 75038 remained
active and unchanged, with zero restarts. No live deployment occurred.

This observer is only a prerequisite measurement helper. Next work is a new
refuge opportunity protocol and a separate genotype observer driver with
validated genotype-linked food and paid-cost gates before any habitat or
trait-selection study. The driver, study protocol, and World execution are not
included here.
