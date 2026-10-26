# Resolver Order V1 runner repair record

**Purpose:** register one fail-closed supervisor schema correction before any replacement run. This note does not modify the frozen protocol or its registration clarification:
[protocol](RESOLVER_ORDER_V1_PROTOCOL_20261026.md), SHA-256
`e8357e8d2d0795ae110579befb416585f50161355f140ff5d80252dc4402f6ff`;
[registration addendum](RESOLVER_ORDER_V1_REGISTRATION_ADDENDUM_20261026.md),
SHA-256 `7a271bbc907cb940b4190f1a734896b304f202a92c11282772c3719b4158a65b`.
The replacement run registration must record this note's own SHA-256 before
launching any World.

## Preserved failed attempt

The first registered attempt is retained at
`/root/curious/data/experiments/resolver-order-v1-20261026/`. Its manifest is
SHA-256 `9b8c43fcef5e4eb3025375f6cf7e40483d2a52c39c04fb753a2c48192b17fb28`
and remains `failed`. The first child completed its case output, including its
prefix replay through tick 93, the shared target preamble, A/B/C and native
off/on references. This consumed 98 resolved-tick invocations and one shared
preamble. Its output reports the internal target, branch-roundtrip, intent,
learner, and reference gates as true. The supervisor then rejected the JSON
because it expected `prefix_replay.from_initial_state_through`; the child
actually emitted `prefix_replay.from_initial_state: true` and
`prefix_replay.validated_through_tick: 93`. This is a runner/schema mismatch,
not a completed experiment result or acceptance evidence. The case output SHA-256
is `d2bffb4143f7902703ed42ab408f9721837a6fb272afd9e9a5fde0a5cc0bc4dd`.

The failed manifest, copied input files, child output, and log are immutable.
Do not overwrite, append to, retry, repair, or relabel that run as complete. An
offline validator review of its retained first case is permitted and is not a
new World execution.

## Narrow repair and next run identity

The supervisor now requires the schema actually emitted by the child:
`from_initial_state == true` and
`validated_through_tick == target_tick - 1`, plus all existing per-tick raw-row
checks. The replacement identity is `resolver-order-v1-runnerfix-20261026`,
with a separate new, non-overwriting output directory. It retains the same five
fixed seed/tick cases, three arms, reference checks, metrics, interpretation
limits, per-child budgets, 180-second phase limit, and aggregate 64 MiB cap.
Only the supervisor's accepted prefix-field names and the run identity change.

Before any World in the replacement run, registration must pin this repair-note
hash together with the unchanged protocol/addendum hashes, clean source and
supervisor identities, binary hash, raw manifest hash, and all five input
hashes. The old failed attempt remains an infrastructure failure. Replacement
results, if any, are separate evidence and still have no utility or acceptance
threshold.
