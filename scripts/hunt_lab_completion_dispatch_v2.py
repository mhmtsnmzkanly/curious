#!/usr/bin/env python3
"""Queue one completion message for an explicitly watched bounded lab.

The runtime config is operator-owned and lives outside the repository. This
versioned helper only reads explicitly allowlisted lab manifests and stores a
durable outbox; it never starts, resumes, or changes an experiment.
"""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

PROJECT = Path("/root/curious")
EXPERIMENTS = PROJECT / "data/experiments"
CODEX = "/usr/bin/codex"
NODE = "/usr/bin/node"
NOTIFY = "/root/notify/bin/notify.js"
NOTIFY_TOKEN = "/root/notify/token.json"
CONFIG_SCHEMA = "curious-bounded-lab-completion-v2"
HELPER_NAME = "hunt_lab_completion_dispatch_v2.py"
LAB_SCHEMA = re.compile(
    r"^curious-(hunt-lab|lifetime-lab|herbivore-lab|ecology-traits)-run-v([1-9][0-9]*)$"
)
RUN_NAME = {
    "hunt-lab": re.compile(r"^hunt-lab-v([1-9][0-9]*)-[A-Za-z0-9][A-Za-z0-9._-]{0,63}$"),
    "lifetime-lab": re.compile(r"^lifetime-lab-v([1-9][0-9]*)-[A-Za-z0-9][A-Za-z0-9._-]{0,63}$"),
    "herbivore-lab": re.compile(r"^herbivore-lab-v([1-9][0-9]*)-[A-Za-z0-9][A-Za-z0-9._-]{0,63}$"),
    "ecology-traits": re.compile(r"^ecology-traits-v([1-9][0-9]*)-[A-Za-z0-9][A-Za-z0-9._-]{0,63}$"),
}
ARTIFACT_NAMES = frozenset({"hunt-lab-probe", "lab-probe"})
RUN_KEY = re.compile(r"^[a-z0-9][a-z0-9._-]{0,63}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
REVISION = re.compile(r"^[0-9a-f]{40}$")
UUID = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
    re.I,
)
QUEUE_ID = re.compile(r"^[A-Za-z0-9_-]{6,128}$")


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def parse_time(value: Any, label: str) -> datetime:
    if not isinstance(value, str):
        raise ValueError(f"{label} must be an ISO-8601 UTC string")
    try:
        result = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ValueError(f"{label} must be an ISO-8601 UTC string") from exc
    if result.tzinfo is None or result.utcoffset() is None:
        raise ValueError(f"{label} must include a timezone")
    return result.astimezone(timezone.utc)


def canonical_hash(value: Any) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(data).hexdigest()


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def helper_hash() -> str:
    return file_hash(Path(__file__).resolve(strict=True))


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} must contain a JSON object")
    return value


def validate_config(config: dict[str, Any], actual_helper_hash: str) -> tuple[list[dict[str, str]], Path]:
    if config.get("schema") != CONFIG_SCHEMA:
        raise ValueError("unsupported completion config schema")
    if config.get("helper_sha256") != actual_helper_hash:
        raise ValueError("frozen completion helper SHA does not match runtime config")
    thread_id = config.get("thread_id")
    if not isinstance(thread_id, str) or not UUID.fullmatch(thread_id):
        raise ValueError("thread_id must be a UUID")
    parse_time(config.get("expires_at"), "expires_at")
    outbox_value = config.get("outbox_dir")
    if not isinstance(outbox_value, str) or not Path(outbox_value).is_absolute():
        raise ValueError("outbox_dir must be an absolute path")
    outbox = Path(outbox_value)
    if outbox.exists() and outbox.is_symlink():
        raise ValueError("outbox_dir must not be a symlink")

    allowed_schema = config.get("allowed_schema")
    protocol_version = config.get("protocol_version")
    schema_match = LAB_SCHEMA.fullmatch(allowed_schema) if isinstance(allowed_schema, str) else None
    if (schema_match is None or type(protocol_version) is not int or protocol_version < 1
            or int(schema_match.group(2)) != protocol_version):
        raise ValueError("allowed_schema and protocol_version must identify the same allowed lab version")
    lab_kind = schema_match.group(1)
    artifact_name = config.get("artifact_name")
    if not isinstance(artifact_name, str) or artifact_name not in ARTIFACT_NAMES:
        raise ValueError("artifact_name must be an explicitly allowed immutable probe filename")
    runs = config.get("watched_runs")
    if not isinstance(runs, list) or len(runs) != 1:
        raise ValueError("watched_runs must explicitly name exactly one bounded run")
    validated: list[dict[str, str]] = []
    seen: set[str] = set()
    for row in runs:
        if not isinstance(row, dict):
            raise ValueError("watched_runs entries must be objects")
        name, output_dir = row.get("name"), row.get("output_dir")
        if not isinstance(name, str) or not RUN_KEY.fullmatch(name) or name in seen:
            raise ValueError("watched run names must be unique safe keys")
        if not isinstance(output_dir, str) or not Path(output_dir).is_absolute():
            raise ValueError(f"{name}: output_dir must be absolute")
        output = Path(output_dir)
        run_match = RUN_NAME[lab_kind].fullmatch(output.name)
        if (output.name != name or run_match is None
                or int(run_match.group(1)) != protocol_version):
            raise ValueError(f"{name}: output_dir must name an allowlisted {lab_kind} v{protocol_version} run")
        try:
            output.relative_to(EXPERIMENTS)
        except ValueError as exc:
            raise ValueError(f"{name}: output_dir must be under {EXPERIMENTS}") from exc
        if output.exists() and output.is_symlink():
            raise ValueError(f"{name}: output_dir must not be a symlink")
        validated.append({"name": name, "output_dir": str(output)})
        seen.add(name)
    return validated, outbox


def _safe_manifest(output: Path) -> Path | None:
    if not output.exists() or output.is_symlink() or not output.is_dir():
        return None
    try:
        resolved = output.resolve(strict=True)
        resolved.relative_to(EXPERIMENTS.resolve(strict=True))
    except (OSError, ValueError):
        return None
    manifest = output / "manifest.json"
    if manifest.is_symlink() or not manifest.is_file():
        return None
    return manifest


def terminal_identity(output: Path, manifest: dict[str, Any], allowed_schema: str,
                      protocol_version: int, artifact_name: str) -> tuple[str, dict[str, Any]] | None:
    status = manifest.get("status")
    if status not in ("completed", "failed"):
        return None
    if manifest.get("schema") != allowed_schema or type(manifest.get("protocol_version")) is not int \
            or manifest.get("protocol_version") != protocol_version:
        return None
    revision, binary_sha, protocol_sha = (
        manifest.get("revision"), manifest.get("binary_sha256"), manifest.get("protocol_sha256")
    )
    finished = manifest.get("finished_at")
    if not isinstance(revision, str) or not REVISION.fullmatch(revision):
        return None
    if not all(isinstance(value, str) and SHA256.fullmatch(value) for value in (binary_sha, protocol_sha)):
        return None
    try:
        parse_time(finished, "finished_at")
    except ValueError:
        return None
    raw_sha = manifest.get("raw_sha256")
    summary_sha = manifest.get("summary_sha256")
    if status == "completed":
        if not all(isinstance(value, str) and SHA256.fullmatch(value) for value in (raw_sha, summary_sha)):
            return None
        for name, expected in (("raw.jsonl", raw_sha), ("summary.json", summary_sha)):
            path = output / name
            if path.is_symlink() or not path.is_file():
                return None
            if file_hash(path) != expected:
                return None
    else:
        raw_sha = raw_sha if isinstance(raw_sha, str) and SHA256.fullmatch(raw_sha) else None
        summary_sha = summary_sha if isinstance(summary_sha, str) and SHA256.fullmatch(summary_sha) else None
    if artifact_name not in ARTIFACT_NAMES:
        return None
    probe = output / artifact_name
    protocol = output / "protocol.md"
    if probe.is_symlink() or not probe.is_file() or file_hash(probe) != binary_sha:
        return None
    if protocol.is_symlink() or not protocol.is_file() or file_hash(protocol) != protocol_sha:
        return None
    identity = {
        "status": status,
        "revision": revision,
        "artifact_name": artifact_name,
        "binary_sha256": binary_sha,
        "protocol_sha256": protocol_sha,
        "raw_sha256": raw_sha,
        "summary_sha256": summary_sha,
        "finished_at": finished,
    }
    return status, identity


def atomic_save(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    if path.parent.is_symlink():
        raise ValueError("outbox directory must not be a symlink")
    fd, temporary = tempfile.mkstemp(prefix=".outbox-", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            json.dump(value, stream, sort_keys=True, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.chmod(temporary, 0o600)
        os.replace(temporary, path)
        directory_fd = os.open(path.parent, os.O_DIRECTORY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def queue_id_from(stdout: str, expected_thread: str) -> str | None:
    # The native CLI has no JSON mode. Accept only its full success sentence,
    # including the requested thread UUID, so unrelated stdout cannot mark it sent.
    match = re.fullmatch(
        r"\s*Queued message ([0-9a-f-]{36}) for thread ([0-9a-f-]{36})\.\s*",
        stdout,
        re.I,
    )
    if not match or match.group(2).lower() != expected_thread.lower():
        return None
    if not UUID.fullmatch(match.group(1)):
        return None
    return match.group(1)


def default_call(argv: list[str], timeout: int) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, capture_output=True, text=True, timeout=timeout, check=False)


def send_notify(run_name: str, identity: str,
                call: Callable[[list[str], int], subprocess.CompletedProcess[str]]) -> tuple[bool, str]:
    payload = {
        "title": "Curious bounded lab completion queue error",
        "message": f"Run {run_name} has a pending completion queue failure. Inspect the durable bounded-lab outbox.",
        "type": "error",
        "target": {"user": "u_e049630155fc"},
    }
    descriptor, filename = tempfile.mkstemp(prefix="hunt-lab-notify-", suffix=".json")
    path = Path(filename)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(payload, stream)
        os.chmod(path, 0o600)
        result = call([NODE, NOTIFY, "-t", NOTIFY_TOKEN, "send", str(path), "--json", "--timeout", "10",
                       "--idempotency-key", f"bounded-lab-queue-error:{run_name}:{identity}"], 15)
        if result.returncode == 0:
            return True, ""
        return False, f"notify exit {result.returncode}"
    except (OSError, subprocess.TimeoutExpired) as exc:
        return False, type(exc).__name__
    finally:
        try:
            path.unlink()
        except FileNotFoundError:
            pass


def dispatch_one(
    entry: dict[str, str], thread_id: str, allowed_schema: str, protocol_version: int,
    artifact_name: str, outbox: Path,
    call: Callable[[list[str], int], subprocess.CompletedProcess[str]],
) -> str:
    output = Path(entry["output_dir"])
    manifest_path = _safe_manifest(output)
    if manifest_path is None:
        return "waiting"
    try:
        manifest = read_json(manifest_path)
    except (OSError, ValueError, json.JSONDecodeError):
        return "waiting"
    terminal = terminal_identity(output, manifest, allowed_schema, protocol_version, artifact_name)
    if terminal is None:
        if manifest.get("status") not in ("completed", "failed"):
            return "waiting"
        invalid_path = outbox / f"{entry['name']}.invalid.json"
        manifest_digest = file_hash(manifest_path)
        if invalid_path.exists() and invalid_path.is_symlink():
            raise ValueError(f"{entry['name']}: invalid manifest marker must not be a symlink")
        if invalid_path.exists():
            try:
                previous = read_json(invalid_path)
            except (OSError, ValueError, json.JSONDecodeError):
                previous = {}
            if previous.get("manifest_sha256") == manifest_digest:
                return "invalid_terminal_manifest"
        marker = {
            "schema": "curious-hunt-lab-completion-outbox-v1",
            "run": entry["name"],
            "state": "invalid_terminal_manifest",
            "manifest": str(manifest_path),
            "status": manifest.get("status"),
            "manifest_sha256": manifest_digest,
            "observed_at": utc_now().isoformat(),
        }
        atomic_save(invalid_path, marker)
        return "invalid_terminal_manifest"
    status, identity_fields = terminal
    identity = canonical_hash({"run": entry["name"], **identity_fields})
    state_path = outbox / f"{entry['name']}.json"
    if state_path.exists():
        if state_path.is_symlink():
            raise ValueError(f"{entry['name']}: outbox marker must not be a symlink")
        try:
            state = read_json(state_path)
        except (OSError, ValueError, json.JSONDecodeError):
            return "outbox_corrupt"
        if state.get("identity_sha256") != identity:
            state.update(state="identity_changed", observed_identity_sha256=identity)
            atomic_save(state_path, state)
            return "identity_changed"
        if state.get("state") == "queued" and state.get("queue_id"):
            return "already_queued"
    else:
        state = {
            "schema": "curious-hunt-lab-completion-outbox-v1",
            "run": entry["name"],
            "state": "pending",
            "identity_sha256": identity,
            "status": status,
            "manifest": str(manifest_path),
            "attempts": 0,
            "created_at": utc_now().isoformat(),
            "at_least_once": True,
        }
        atomic_save(state_path, state)
    token = f"hunt-lab-v3:{entry['name']}:{identity}"
    message = (
        "Bounded hunt lab reached terminal manifest status. Recheck the allowlisted "
        f"manifest before acting. run={entry['name']} status={status} "
        f"identity_sha256={identity} completion_token={token} "
        f"manifest={manifest_path}"
    )
    state["attempts"] = int(state.get("attempts", 0)) + 1
    state["last_attempt_at"] = utc_now().isoformat()
    try:
        result = call([CODEX, "queue", "--thread", thread_id, "--message", message], 10)
        stdout = result.stdout or ""
        stderr = result.stderr or ""
        queue_id = queue_id_from(stdout, thread_id) if result.returncode == 0 else None
        if result.returncode == 0 and queue_id:
            if state.get("last_error"):
                state["previous_queue_error"] = state["last_error"]
            state.update(state="queued", queue_id=queue_id, queued_at=utc_now().isoformat())
            state.pop("last_error", None)
            atomic_save(state_path, state)
            return "queued"
        reason = f"codex queue exit {result.returncode}; queue ID not confirmed"
        if result.returncode != 0:
            reason = f"codex queue exit {result.returncode}"
        state.update(state="pending", last_error=reason)
    except subprocess.TimeoutExpired:
        # Queue may have accepted the request before the transport timed out.
        # Retrying is at-least-once; completion_token lets the receiving thread dedupe.
        reason = "codex queue timed out; delivery is ambiguous"
        state.update(state="pending", last_error=reason, delivery_ambiguous=True)
    except OSError as exc:
        reason = f"codex queue unavailable ({type(exc).__name__})"
        state.update(state="pending", last_error=reason)
    if not state.get("notify_sent"):
        sent, notify_error = send_notify(entry["name"], identity, call)
        state["notify_sent"] = sent
        if not sent:
            state["notify_error"] = notify_error
    atomic_save(state_path, state)
    return "pending"


def run(config_path: Path, now: datetime | None = None,
        call: Callable[[list[str], int], subprocess.CompletedProcess[str]] = default_call) -> dict[str, Any]:
    if config_path.is_symlink() or not config_path.is_file():
        raise ValueError("runtime config must be a regular file, not a symlink")
    config = read_json(config_path)
    watched, outbox = validate_config(config, helper_hash())
    expires_at = parse_time(config["expires_at"], "expires_at")
    if (now or utc_now()).astimezone(timezone.utc) >= expires_at:
        outbox.mkdir(parents=True, exist_ok=True, mode=0o700)
        atomic_save(outbox / "expired.json", {
            "schema": "curious-hunt-lab-completion-outbox-v1",
            "state": "expired", "expired_at": (now or utc_now()).isoformat(),
            "config": str(config_path),
        })
        return {"status": "expired", "runs": {}}
    outbox.mkdir(parents=True, exist_ok=True, mode=0o700)
    if outbox.is_symlink():
        raise ValueError("outbox_dir must not be a symlink")
    lock_path = outbox / ".dispatch.lock"
    with lock_path.open("a+") as lock:
        os.chmod(lock_path, 0o600)
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        results = {
            entry["name"]: dispatch_one(entry, config["thread_id"], config["allowed_schema"],
                                         config["protocol_version"], config["artifact_name"], outbox, call)
            for entry in watched
        }
    return {"status": "checked", "runs": results}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    args = parser.parse_args(argv)
    try:
        result = run(args.config)
    except (OSError, ValueError, TimeoutError, BlockingIOError) as exc:
        print(json.dumps({"status": "error", "error": str(exc)}), file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
