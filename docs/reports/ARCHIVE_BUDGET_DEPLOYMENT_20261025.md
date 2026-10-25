# Archive budget correction deployment — 25 October 2026

## Result

The archive-budget correction was backported to the frozen V12 live source and
deployed on 25 October. The running binary is
`/root/curious/bin/releases/curious-observation-budget-8dc23de`, SHA-256
`35930838981679ab1b937324df6dda73d2b3bcc6c57706d69f2c11d2803959af`. The
qualified symlink now points to that immutable release. The prior live binary
and source remain preserved.

The change refreshes closed-shard metadata totals before a cached history
budget rejection. It reads file metadata and directory entries, not archive
contents. If metadata enumeration or stat fails, the write fails closed. A
single re-inventory handles a closed archive disappearing during the pass.
Checkpoint format, SQLite schema, and biological source were unchanged. The
active world was V12 at deployment and remains V12; the recorded checkpoint
and eight-tick continuation comparison confirmed exact State/RNG equality.

## Validation and deployment evidence

Before deployment, the old binary and candidate were started against isolated
copies of the latest live shard. The original checkpoint matched both API
states, and eight successive ticks matched across nine full State/RNG
snapshots. All 54 checks passed for state, persisted records, events,
decisions, traces, metrics, and interventions; only wall-clock timestamps and
`tick_us` were excluded. The first comparison attempt failed before starting
either engine due to a SQLite reader row-factory error; the corrected run
passed. Neither comparison touched the live service or database.

The V12 backport passed 89 all-target Rust tests, 52 Python tests, a release
app build, and HTTP, auth, writer-lock, and SIGKILL checks. The deployment
paused the live world, stopped and drained the
retention, monitor, and sampler timers, then stopped the service. A stopped
copy of the full data tree was verified before switching the symlink. The
backup contains 740 entries and 2,760,680,943 file bytes; its inventory hash
is `9530624092d56359da16946824f2c975787b5df015c8d427bd9efc26c4be8b3d`.
SQLite schema V2 integrity checks passed, and the latest checkpoint matched
the paused API State/RNG at tick 10670.

The candidate restarted paused on the same full V12 State/RNG, with no storage
error, and its process executable matched the expected hash. The original
0.03 TPS running mode was restored; the service resumed at tick 10671 with no
storage error. The retention, monitor, and sampler timers were restored, and
the temporary systemd drop-in was removed. Replay remained unchanged. The
pre-deployment data backup remains available; rollback was not needed.

A follow-up live check at 00:29:30 UTC observed tick 10673, three normal ticks
after the paused candidate checkpoint. The API State matched the latest
persisted SQLite checkpoint; State V12, schema V2, candidate executable hash,
0.03 TPS, all three active timers, no storage error, no drop-ins, and unchanged
replay PID were confirmed. Evidence:
[`live-postdeploy-verification.json`](../../data/operations/wave9-20261025/live-postdeploy-verification.json).

Deployment record:
[`deployment.json`](../../data/operations/wave9-20261025/deploy-20261025T002809Z/deployment.json).
Comparison evidence:
[`comparison.json`](../../data/operations/wave9-20261025/binary-compare-v2/comparison.json).
The earlier failed attempt is retained at
[`comparison.json`](../../data/operations/wave9-20261025/binary-compare-reviewed/comparison.json).

## Limits and remaining work

The history limit remains a soft guard, not a strict disk quota. External
files added after a metadata snapshot are not continuously detected, and
concurrent external changes can race a write. WAL and temporary archive space
are outside the logical history estimate. The comparison verifies continuation
compatibility over this recorded checkpoint and eight ticks; it is not a proof
for every possible runtime path. This storage correction provides no biological
or experimental result. Further biology work requires separate protocol and
measurement review; no new seed or experiment was run for this deployment.
