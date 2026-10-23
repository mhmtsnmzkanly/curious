# Bounded hunt-lab completion wake-up

The optional one-minute timer reads only the run directory explicitly listed in
an operator-owned config outside Git. It never starts or resumes a run. The
versioned helper verifies its own SHA, config TTL, run manifest schema/status,
probe and protocol hashes, and completed raw/summary hashes before queuing a
message for the configured Codex thread. `running`, missing, or malformed
manifests wait. Only `completed` and `failed` are terminal.

`deploy/curious-hunt-lab-completion.service` and `.timer` are installation
templates. Before enabling them, copy the reviewed helper to an immutable
runtime release path whose directory is its SHA-256:

```sh
sha256sum scripts/hunt_lab_completion_dispatch_v1.py
install -d -m 0755 /root/curious-runtime/releases/<helper-sha256>
install -m 0555 scripts/hunt_lab_completion_dispatch_v1.py \
  /root/curious-runtime/releases/<helper-sha256>/hunt_lab_completion_dispatch_v1.py
```

Use that same SHA in the service `ExecStart` path and the external config's
`helper_sha256`. Copy `deploy/hunt-lab-completion-config.example.json` to
`/root/curious-runtime/hunt-lab-completion-v1.json`, then replace the thread
UUID, expiry, output run name/path, allowed schema, and protocol version. Keep
the config and outbox outside Git. `watched_runs` must contain exactly one
absolute path under `/root/curious/data/experiments`; the manifest never supplies
a path or command to execute. The current V3 entry is explicit in the example;
a later lab version requires an intentional config change to its matching
`allowed_schema` and integer `protocol_version`.

The helper calls `codex queue --thread UUID --message TEXT` as an argument list,
without a shell. It records the returned queue ID only when stdout exactly
matches the CLI's success sentence and names the configured thread. A stable
completion token includes the run identity so the receiving thread can recheck
the manifest and deduplicate. If transport times out after acceptance, delivery
is ambiguous; the durable outbox retries at least once and records that risk.
Queue errors produce a user-scoped Notify alert with a stable idempotency key;
the outbox retains the error even if Notify is unavailable. The Notify token
remains at its host path and is never copied into this repository.

The timer is not installed or enabled by these templates. Test the service
inside its exact systemd sandbox before enabling it. The one-run allowlist, 10s
queue timeout, 15s Notify timeout, and 30s unit timeout keep each poll bounded.
