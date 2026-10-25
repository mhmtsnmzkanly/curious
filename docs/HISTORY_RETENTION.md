# Closed history shard retention

`scripts/history_retention.py` summarizes old, closed Curious history archives
and optionally removes their detail. It is an external maintenance process; it
does not change the simulation binary or SQLite schema.

## Retention contract

- `--keep-days` defaults to 7; `--byte-budget` defaults to 3 GiB of compressed
  closed-shard bytes. Both are validated as positive integers in the CLI and
  callable API. Archives at least this old are eligible. If compressed closed
  shard bytes exceed the budget, oldest eligible closed archives are also
  selected until the projected compressed-shard total fits. This is not a strict
  bound on total world storage: raw/newest protected files may exceed the cap,
  and permanent summaries grow over time.
- Every run processes at most two archives and has a 45-second internal deadline.
  The systemd unit adds a 60-second wall clock limit and 512 MiB memory cap.
- Only a uniquely named, closed `history-N.sqlite.gz` or
  `history-N.sqlite.zst` is eligible. The newest shard number is always kept.
  Raw SQLite files, their WAL/SHM neighbors, duplicate shard representations,
  symlinks, and unrecognized files are not eligible. If the byte budget cannot
  be met while preserving these files, the report can remain over budget.
- SQLite schema versions 1 and 2 are recognized; version 0 is rejected because
  it has no established retention proof. Unknown versions, corrupt
  archives, unexpected table layouts, failed integrity checks, oversized
  decompression, or missing zstd support stop that candidate without deletion.
  Temporary unpacking is capped at 256 MiB. The zstd child has CPU, address
  space, file size, and descriptor limits.
- Each evidence record contains source name, source SHA-256 and byte count,
  SQLite user version, record count, event totals by type, the first and last
  full `state_json` checkpoints, last metrics, and at most 20 deterministic
  life/error event payload examples. It states whether examples were truncated
  and how many were omitted. `history-retention/` stores the gzip summary and a
  separate proof manifest with source identity/signature and summary SHA-256.
  Both are written atomically, fsynced, and verified before source removal.
- If the process stops after writing evidence but before unlinking, a later run
  accepts only a summary and proof that still match the candidate's bytes and
  source signature. If a summary exists without its proof, it is regenerated
  from the still-present source and compared/replaced before proof creation; a
  self-declared source hash alone is insufficient. Summary/proof symlinks and
  any mismatch preserve the source. Full checkpoints must have State versions
  1–18 and a tick matching their record.

This path is for the selected live coexistence history directory only. It does
not delete raw experiment evidence or clean stopped observation worlds; those
remain separate root operations. Summary evidence is permanent and itself is
not included in the compressed closed-shard byte budget.

## Running

Preview a batch first. `--apply` is required to write evidence and remove detail.

```sh
python3 scripts/history_retention.py --data-dir /root/curious/data/coexistence
python3 scripts/history_retention.py --data-dir /root/curious/data/coexistence --apply
```

The units target the verified host paths `/root/curious/data/coexistence` and
`/root/curious/data/history-retention-runtime.env`. Install the script as an
immutable copy at `/root/curious-runtime/releases/history-retention-<commit>/history_retention.py`
and set `CURIOUS_RETENTION_SCRIPT` in the environment file to that absolute
path. The oneshot runs as root because the host has no `curious` service account.
`ProtectHome=read-only` and `ProtectSystem=strict` keep source/control paths
read-only; the only writable exception is the coexistence data directory, whose
parent permissions are needed for archive unlinking and evidence creation. The
timer runs every 15 minutes with up to 30 seconds randomized delay, so its
2-shard cap can keep pace with the observed history growth. Validate the actual
environment file and immutable copy before enabling the timer.

Before the 25 October 2026 storage correction, external compaction could leave
a running engine's `Store.archived_bytes` cache stale. That pre-deployment
behavior and the later V12 backport are recorded below; retention itself still
does not restart or redeploy a live world.

## Historical deployment and cleanup — 24 October 2026

At main `904d4cc`, root independently passed all 8 retention tests and a real
closed-Zstandard integration check over 17 records and 15,876 events. The live
source was not changed by those checks. Frozen runtime copy:
`/root/curious-runtime/releases/history-retention-904d4cc/history_retention.py`,
SHA-256 `2331196cec3c827f03546102265bd1b331b2c7d81bcf1cff8f5ecd6d3de8bee9`;
runtime path setting is in `data/history-retention-runtime.env`. The 15-minute
timer is enabled and its first run succeeded with no eligible closed shards.

Root separately retired detailed histories for stopped `main`, `meadow`, `lab`,
and `evolution`, retaining four minimal resume SQLite files and checkpoint
proofs. Final ticks: 10, 77,974, 30, 74,075. The plan and source hashes are in
`data/retired-worlds/20261024T015208Z/plan.json`; net detailed-history removal
was 929,698,848 bytes and checkpoint backup was 292,787 bytes. PIDs on 7882 and
7883 were unchanged. Four prior services are off, the old watchdog is disabled,
and no wake is pending. No biological/ML experiment or storage batch remains running. All 21 selected
experiment JSONL files were losslessly archived: 1,060,085,984 raw bytes became
52,274,679 compressed bytes. All 710 original metadata files retained their
hashes; the existing scanner still sees all 2,607 indexed seed values. A real
200-tick calibration was restored, verified by its frozen reader, and repacked.
The resumed full Python suite passed 177 tests.
See [the verified report](reports/HISTORY_RETENTION_20261024.md).

At that time the live engine still had stale cached accounting after external
pruning. This condition was corrected by the separately reviewed deployment on
25 October below. The old retention report remains historical. Restore archived
raw JSONL before using frozen analyzers or scanners that require original paths;
archive and protocol metadata remain unchanged.

## Archive-budget accounting fix and live deployment — 25 October 2026

The earlier V18 source candidate was backported narrowly to the frozen V12 live
source and deployed as immutable release
`curious-observation-budget-8dc23de` (source `8dc23de`), SHA-256
`35930838981679ab1b937324df6dda73d2b3bcc6c57706d69f2c11d2803959af`. The
previous qualified V12 binary remains preserved. Before deployment, old and
candidate binaries matched the live checkpoint and eight isolated continuation
ticks: 54 of 54 checks passed across nine full State/RNG snapshots, persisted
records, events, decisions, traces, metrics, and interventions. The V12
backport passed 89 all-target Rust tests, 52 Python tests, a release app build,
and HTTP/auth/writer-lock/SIGKILL checks.

Before rejecting a write using cached archive totals, the store now refreshes
closed-shard metadata using the selected file inventory. It reads directory
entries and metadata, not archive contents. A shard disappearing during the
pass triggers one re-inventory; persistent enumeration/stat errors fail closed.
The live checkpoint remained State V12 and SQLite schema V2. The candidate
restarted with exact paused State/RNG equality at tick 10670, then resumed at
tick 10671 without storage error. The retention, monitor, and sampler timers
were restored. No rollback was needed; a verified stopped-data backup remains
at `data/backups/wave9-20261025/coexistence-before-budget-20261025T002809Z`.
A live check at 00:29:30 UTC confirmed tick 10673, exact API/latest-checkpoint
equality, V12/V2, the candidate hash, no storage error, and all three timers
active. The replay process remained unchanged.

The history limit remains a soft guard, not a strict disk quota. External files
added after a metadata snapshot are not continuously detected, concurrent
external changes can race a write, and WAL or temporary archive space is
outside the logical active-pages plus selected-history estimate. The comparison
covers this checkpoint and eight ticks; it does not prove every runtime path or
make a biological claim. Deployment evidence and hashes are in
[`ARCHIVE_BUDGET_DEPLOYMENT_20261025`](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
and its JSON companion. Earlier V18 source-only notes and the 24 October
retention report remain historical records.
