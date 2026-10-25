# Shadow engine bridge probe

`scripts/support/shadow_engine_bridge.rs` is a standalone diagnostic runner that
can be compiled unchanged against the current V19 source and the frozen V12
source at `/root/curious-research/archive-live-backport-wave9` (commit
`8dc23de785dcfcd248611c17de0e9351b0116222`, verified with `git rev-parse
HEAD`). It only restores a JSON engine
checkpoint and calls `World::tick`; it has no network, service, SQLite, or live
world integration. The source uses checkpoint and event serialization APIs
shared by both revisions and deliberately does not refer to V19-only settings.

Use an external crate directory for each build so neither engine's manifest or
the frozen worktree changes. For example, make `/tmp/shadow-v12/Cargo.toml`:

```toml
[package]
name = "shadow-v12"
version = "0.1.0"
edition = "2024"

[dependencies]
curious = { path = "/root/curious-research/archive-live-backport-wave9" }
serde_json = "1"
```

Put the exact shared source in that crate's `src/main.rs`, then run:

```sh
cargo run --release --manifest-path /tmp/shadow-v12/Cargo.toml -- \
  /path/to/checkpoint.json /tmp/shadow-v12-output 1000
```

Repeat in a separate `/tmp/shadow-v19` crate with `curious = { path =
"/root/curious" }`. Save and compare `sha256sum` output for the source, input
checkpoint, both `Cargo.toml` files, and resulting binaries externally. Never
point either runner at service data or a live endpoint. Output directories must
be new and ticks must be an integer from 0 through 1000.

Each run writes `summary.json` with the checkpoint State after restore, final
State, RNG values, alive count by species at the initial point and final point,
every 100 ticks plus the end, cumulative survived ticks per entity, event counts,
species births/deaths, movement count and energy sum, attack count and damage
sum. Every tick's event payloads stream to `events.jsonl`; only five event arrays
and five State snapshots are retained in memory for resume validation. Inputs
over 10,000 entity slots fail before the run. Active and sleeping entities count
as alive when `LifeState::is_alive()` is true; dead, removed, and other inactive
phases are excluded. State fingerprints use FNV-1a for a quick
same-run comparison and are not cryptographic hashes; hash the full files with
SHA-256 externally.

For runs of at least five ticks, a five-tick branch is restored from the initial
State (or tick 100 when the run is at least 105 ticks). It compares all five
per-tick event arrays and serialized States, including RNG, against the main
uninterrupted trajectory. This invokes five additional `World::tick` calls on
the validation branch; `validation_tick_calls_added` discloses the calls.
`elapsed_ms` records runner time through simulation, validation, and event-log
flush; it is descriptive and not a benchmark.
Runs shorter than five ticks report `checkpoint_resume_exact: false` because
the five-tick check cannot be completed. These diagnostics support a bounded
bridge comparison; they do not qualify a behavioral improvement or alter the
separate pre-registered outcome protocol.
