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

External compaction leaves a running engine's `Store.archived_bytes` cache
stale. Do not restart or redeploy a live world as part of this maintenance.
Coordinate any later restart separately; until the store is reopened, its
cached budget accounting may overcount removed archives.

## Current deployment and cleanup — 24 October 2026

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

The engine's `Store.archived_bytes` cache is stale after external pruning, so its
32 GiB ceiling is not a durability guarantee. Do not restart the engine as part
of retention. Restore archived raw JSONL before using frozen analyzers or
scanners that require original paths. The archive helper verifies lossless bytes
and preserves the seed index; protocol and manifest files remain unchanged.

## Candidate engine accounting fix — not deployed

The V18 source candidate refreshes closed-shard byte totals from the selected
`Store.files()` inventory when the cached projection would reject a record. It
reads directory entries and file metadata only; it does not open archive
contents. Enumeration, metadata, and byte-sum errors fail closed. This permits
the already-open writer to continue after external retention removed older
closed shards, while preserving the budget stop when the measured total still
reaches the configured limit. The writer lock continues to protect the active
SQLite file and its WAL; retention must preserve the newest shard as before.

This remains a soft guard, not an atomic disk quota: another process can change
files after the metadata snapshot, externally added files are not continuously
detected, and WAL or temporary archive space is outside the logical active-pages
plus selected-history estimate. A shard disappearing during the metadata pass
causes one fresh inventory pass; persistent stat/permission errors still stop the
writer. The change is a source candidate only and has not been deployed or tested
against a live world. The frozen live world remains V12; do not describe it as
fixed.
