---
name: curious-operations
description: Inspect Curious worlds, services, archive growth, monitoring, and experiment progress; perform scoped maintenance while protecting durable history and frozen releases. Use for status and operations work.
---

# Curious operations

Locate the checkout root three parent levels above this skill directory. Read
[AGENTS.md](../../../AGENTS.md), [CONTROL](../../../docs/CONTROL.md), and the
relevant deployment/observation record. On another host discover its actual
configuration; do not assume this repository's example addresses are installed.

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
Preserve `data/`, raw results, SQLite/WAL/checkpoints, gzip/zstd archives, backups,
`bin/releases/`, and frozen source worktrees. A new table does not shrink SQLite;
use the established shard/compression/budget mechanism. Do not remove evidence to
make a world fit its budget or broadly delete shared caches/unrelated projects.

## Handoff

Report timestamp, world/version identity, tick/population, failures, resources,
monitoring health, and any actual intervention. State a justified next check time
when asked. The configured services may continue after an agent exits; the agent
itself does not keep coding in the background. Use the project's authorized
notification channel for meaningful incidents/milestones, with retry evidence if
available. Routine checks need no notification spam or empty commits.
