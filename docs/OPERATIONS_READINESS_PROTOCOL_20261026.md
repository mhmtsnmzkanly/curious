# Offline operational-readiness probe — 26 October 2026

## Purpose and current decision

This protocol measures a bounded offline V12-to-V19 checkpoint continuation,
State/RNG round-trip, SQLite write/rotation/readback, and process resource use.
It does not test a live service or qualify a deployment. The current utility gate
has not passed; `promotion_allowed` is hard-coded `false` for every mode.

The latest [qualification diagnosis](reports/QUALIFICATION_DIAGNOSTIC_20261025.md)
found two improvements and eight neutral outcomes for care and care-plus-refuge
on the previously seen 401–410 seed set, with no refuge-only improvements. The
offline 1,000-tick V19 bundle shadow from the V12 archive ended with 13 more
living entities than closed V19 control but 1,428 fewer cumulative living
entity-ticks. This is a tradeoff, not accepted utility or Pareto qualification.
The V12 live world has not been upgraded.

## Frozen input and scope

- Read only `data/experiments/qualification-diagnostic-20261025/input-v12.json`.
  Its expected SHA-256 is
  `357870bd9a3d414704ed6befbea050852028f54af304c27cdca482dc31abdad2`;
  the probe checks this before creating output.
- The checkpoint is V12 at tick 12639 with 630 entities. `State::restore`
  migrates an in-memory copy to V19. Requested flags are applied only after
  restore. The input bytes are never rewritten.
- `off` is the required control mode. `care`, `refuge`, and `bundle` are optional
  offline comparisons, not deployment recommendations. All comparisons use
  this one checkpoint and do not create independent population samples.
- Maximum simulation work is 121 World ticks per invocation: up to 120 timed
  primary ticks plus one additional isolated storage-smoke tick. Default primary
  horizon is 120; shorter fixed runs may be selected before execution. No
  retries, seed changes, or threshold tuning are part of the protocol.
- Output must be a new directory. The executable refuses an existing path and
  checks output size between bounded phases and before writing the final
  manifest. Its 256 MiB ceiling is a checked limit, not a preventive disk quota;
  it covers files inside the run directory only. Preserve incomplete output as
  evidence; use a different directory for a new invocation.
- The primary world is advanced without `Store::record` in the timed loop so
  `Instant` encloses only `World::tick`. Frame 0/final checkpoint capture,
  serde, and final restore/recapture are timed and reported separately.
- A separate isolated `storage/` smoke uses the restored checkpoint, a 1 MiB
  shard threshold and an 8 MiB history budget. It records an initial state and
  one tick, reopens Store, checks the latest State, runs SQLite integrity checks,
  and reads a closed zstd shard through `archive::Reader`. This does not test
  runtime writer locking, production retention, or a live database.

## Build and run

Freeze the source/protocol identity before the run. Build in the repository with
the documented single-job limit:

```bash
cargo build --release --locked --example operations_readiness --jobs 1
sha256sum examples/operations_readiness.rs \
  docs/OPERATIONS_READINESS_PROTOCOL_20261026.md \
  data/experiments/qualification-diagnostic-20261025/input-v12.json \
  target/release/examples/operations_readiness
git rev-parse HEAD
git status --short
```

Run the required control into a unique, previously absent directory. Check the
run directory and both sibling sidecar paths before writing anything: the
program protects only the run directory, while shell redirections could
otherwise overwrite the freeze, stdout, or time files. `noclobber` also makes
the redirections fail if a path already exists. `/usr/bin/time` captures process
maximum RSS separately from per-tick timings. The virtual address space ceiling
is 1 GiB; tick count and output size have separate checked limits.

```bash
mkdir -p data/experiments/operations-readiness-20261026
run_id="off-$(date -u +%Y%m%dT%H%M%SZ)"
run_dir="data/experiments/operations-readiness-20261026/$run_id"
freeze_file="data/experiments/operations-readiness-20261026/$run_id.freeze.sha256"
stdout_file="$run_dir.stdout.json"
time_file="$run_dir.time.txt"
for path in "$run_dir" "$freeze_file" "$stdout_file" "$time_file"; do
  if [[ -e "$path" ]]; then
    printf 'Refusing existing output path: %s\n' "$path" >&2
    exit 1
  fi
done
set -o noclobber
sha256sum examples/operations_readiness.rs \
  docs/OPERATIONS_READINESS_PROTOCOL_20261026.md \
  data/experiments/qualification-diagnostic-20261025/input-v12.json \
  target/release/examples/operations_readiness \
  > "$freeze_file"
git rev-parse HEAD \
  >> "$freeze_file"
git status --short \
  >> "$freeze_file"
ulimit -v 1048576
/usr/bin/time -v target/release/examples/operations_readiness \
  data/experiments/qualification-diagnostic-20261025/input-v12.json \
  "$run_dir" off 120 \
  > "$stdout_file" \
  2> "$time_file"
```

The timestamp identifier must be unused; the program refuses an existing run
directory. Do not reuse an output or log path. If comparative modes are later
requested, invoke each mode in its own new directory with the same input hash
and tick count; do not combine their results into an acceptance claim.

`run.json` records argv, source revision/status, executable and example-source
hashes, input identity, configured flags, initial/final checkpoint hashes,
State/RNG round-trip equality, timing phases, and storage-smoke evidence. Keep
the external `/usr/bin/time -v` output beside it as the process RSS record.
Record the source and protocol hashes from the pre-run freeze with the result
bundle; a dirty or changing source invalidates the run identity.

## Pass/fail interpretation

Operational smoke passes only if the program exits successfully, the final
V19 State/RNG restore and recapture are exact, Store reopens with the expected
latest State, each listed SQLite shard passes `PRAGMA integrity_check`, and a
closed archive is read through `archive::Reader`. Resource ceilings must also
hold: at most 121 World ticks total (120 primary plus one storage-smoke tick),
at most 256 MiB under the run directory as checked between phases and before
the final manifest, and at most 1 GiB virtual address space; report measured
peak RSS from `/usr/bin/time -v`. The output-size check is not a disk quota and
does not include sibling stdout/time files.

Passing these checks means only that this bounded offline probe completed and
its output was readable. It does not change `promotion_allowed=false`, establish
behavioral utility, or authorize flags in production. A storage smoke pass does
not qualify a world upgrade.

## Future live-change runbook gates

No live action is performed by this probe. Any later deployment task must first
verify the actual process, binary, checkpoint, State/RNG, storage, and backup on
the host. The documented pause request is `POST /control` with
`{"action":"pause"}`; `/control/pause` is not the route. The control API has
no general command to enable the care/refuge bundle. Never imply these offline
flags can be set through HTTP.

Before an independently authorized upgrade, preserve a verified immutable V12
backup; test migration and configuration on an output copy; record the exact
binary/source/checkpoint hashes; verify resume continuity; and prepare rollback
that restores the backup with a compatible binary. Since current behavioral
utility is unaccepted, the present decision gate remains **blocked for
promotion**. This protocol itself does not grant deployment authority.
