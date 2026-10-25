---
name: curious-operations
description: Inspect Curious worlds, services, archive growth, monitoring, and experiment progress; perform scoped maintenance while protecting durable history and frozen releases. Use for status and operations work.
---

# Curious operations

Locate the checkout root three parent levels above this skill directory. Read
[AGENTS.md](../../../AGENTS.md), [CONTROL](../../../docs/CONTROL.md), and the
relevant deployment/observation record. On another host discover its actual
configuration; do not assume this repository's example addresses are installed.

Operational work usually maps to [K8 motor and operations](../../../docs/ROADMAP.md#motor-isletim);
read-only replay/archive viewing may also touch
[K7 observation and interface](../../../docs/ROADMAP.md#gozlem-arayuz).
These routing labels do not change the scope authorized by the user.

## Establish what is actually running

A request to check status is read-only. Discover executable, unit, data path,
endpoint, target TPS, pause state, and frozen source from actual service settings
and runtime/deployment manifests. Avoid printing secret-bearing environment files.
A new main-branch commit or successful build does not upgrade a running service.

Where systemd is available, inspect service activity, restart count/start time,
process memory/CPU, recent journal failures, and timer schedules. Query documented
status/observation endpoints with required authentication using a safe local client.
Do not expose tokens in command lines, output, Git, or reports. A paused world is
not necessarily a crash; low target TPS may be intentional.

Check tick progress across timestamped samples, species counts, storage errors,
archive size/budget, disk/WAL/temp headroom, sample freshness, monitor heartbeat,
watchdog incidents, and pending notification queues. Read experiment manifests and
actual worker/service state before calling a batch running or completed. A missing
result is not a completed study. Monitor health is separate from world health.

Distinguish current memory, peak sample, process RSS, and service-cgroup memory.
Use measured elapsed time and actual ticks for speed estimates; report sampling
interval and uncertainty for CPU/disk forecasts. Do not infer model quality from
population, age, generation, or service uptime.

## Preserve state during maintenance

Do not restart, resume, accelerate, change founder/scenario settings, replace a
binary, or redeploy a world merely because a check found an issue. Apply authorized
repairs within scope; explain a missing capability if it blocks work. Before a
stateful upgrade, establish checkpoint compatibility, a consistent backup,
immutable executable identity, verification, and rollback that preserves failed
data. Never open newer data with an older incompatible binary. Use strict resume
for existing worlds; a missing checkpoint must not silently create new founders.

Control requests such as step/spawn are not inherently idempotent. If a response
is lost, inspect durable state before retrying. API pause, process stop, browser
replay pause, and experiment completion mean different things.

Only clean proven disposable build/task cache that is not used by active work.
The default is to preserve `data/`, raw results, SQLite/WAL/checkpoints,
gzip/zstd archives, backups, `bin/releases/`, and frozen source worktrees.

There are two narrow, evidence-preserving exceptions when the user explicitly
authorized the exact scope or an already reviewed host policy configures it:

- The Curious history-retention service may compact only uniquely represented,
  closed coexistence `history-N.sqlite.gz`/`.zst` shards after it has verified a
  deterministic summary and source SHA-256/signature proof. It keeps the newest
  shard and all raw SQLite/WAL/SHM files, and refuses unknown/corrupt inputs.
  The configured keep-days and compressed-closed-shard byte target are not a
  total-world storage bound. Never generalize this to other worlds or data.
- The experiment JSONL archive helper may replace one explicitly selected raw
  JSONL with a lossless Zstandard stream only after verifying the compressed
  stream, original bytes/hash, and seed-role index, while preserving its index
  and all protocol/manifest files. Restore the raw file from its index before
  running frozen analyzers or scanners that require the original path. Keep the
  compressed archive/index after restore.

A new table does not shrink SQLite; follow the established mechanism and verify
its proof before any scoped removal. These exceptions do not authorize broad
data deletion, cleaning unrelated caches, or automatic live binary upgrades.

## Handoff

Report timestamp, world/version identity, tick/population, failures, resources,
monitoring health, and any actual intervention. State a justified next check time
when asked. The configured services may continue after an agent exits; the agent
itself does not keep coding in the background. Use the project's authorized
notification channel for meaningful incidents/milestones, with retry evidence if
available. Routine checks need no notification spam or empty commits.
