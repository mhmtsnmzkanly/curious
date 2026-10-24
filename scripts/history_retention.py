#!/usr/bin/env python3
"""Safely replace aged closed Curious history shards with durable summaries."""
from __future__ import annotations

import argparse
import collections
import gzip
import hashlib
import json
import os
import re
import resource
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

SHARD_RE = re.compile(r"^history-(\d{6,})\.sqlite(?:\.(gz|zst))?$")
RAW_RE = re.compile(r"^history-(\d{6,})\.sqlite(?:-wal|-shm)?$")
DEFAULT_KEEP_DAYS = 7
DEFAULT_BUDGET = 3 * 1024**3
MAX_UNPACKED = 256 * 1024**2
MAX_EXAMPLES = 20
MAX_BATCH = 2
MAX_SECONDS = 45
SUPPORTED_VERSIONS = {1, 2}
MIN_STATE_VERSION = 1
MAX_STATE_VERSION = 18


class RetentionError(Exception):
    pass


def _deadline_check(deadline: float) -> None:
    if time.monotonic() > deadline:
        raise RetentionError("maximum runtime exceeded")


def _sha256(path: Path, deadline: float) -> tuple[str, int]:
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        while True:
            _deadline_check(deadline)
            chunk = stream.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
            size += len(chunk)
    return digest.hexdigest(), size


def _inflate(source: Path, target: Path, deadline: float) -> None:
    """Bound decompression output; zstd uses a resource-limited child process."""
    if source.name.endswith(".gz"):
        opener = gzip.open(source, "rb")
        process = None
    else:
        exe = shutil.which("zstd")
        if not exe:
            raise RetentionError("zstd CLI unavailable; preserving candidate")
        def limits() -> None:
            resource.setrlimit(resource.RLIMIT_CPU, (MAX_SECONDS, MAX_SECONDS + 1))
            resource.setrlimit(resource.RLIMIT_FSIZE, (MAX_UNPACKED, MAX_UNPACKED))
            resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2, 512 * 1024**2))
            resource.setrlimit(resource.RLIMIT_NOFILE, (32, 32))
        process = subprocess.Popen([exe, "-q", "-d", "-c", "--", str(source)], stdout=subprocess.PIPE,
                                   stderr=subprocess.DEVNULL, preexec_fn=limits)
        opener = process.stdout
    count = 0
    try:
        with opener, target.open("wb") as output:
            while True:
                _deadline_check(deadline)
                chunk = opener.read(1024 * 1024)
                if not chunk:
                    break
                count += len(chunk)
                if count > MAX_UNPACKED:
                    raise RetentionError("unpacked shard exceeds 256 MiB; preserving candidate")
                output.write(chunk)
            output.flush()
            os.fsync(output.fileno())
        if process and process.wait(timeout=max(0.1, deadline - time.monotonic())) != 0:
            raise RetentionError("zstd decode failed; preserving candidate")
    except Exception as exc:
        if process and process.poll() is None:
            process.kill()
            process.wait()
        target.unlink(missing_ok=True)
        if isinstance(exc, (OSError, EOFError)):
            raise RetentionError("archive decode failed; preserving candidate") from exc
        raise


def _parse_json(raw: Any, what: str) -> Any:
    if not isinstance(raw, str):
        raise RetentionError(f"invalid {what}; preserving candidate")
    try:
        return json.loads(raw)
    except (ValueError, TypeError) as exc:
        raise RetentionError(f"invalid {what}; preserving candidate") from exc


def _event_class(kind: str) -> str | None:
    lowered = kind.lower()
    if any(word in lowered for word in ("reject", "error", "fail", "invalid")):
        return "error"
    if any(word in lowered for word in ("born", "died", "mate", "hunt", "ate", "drank", "sleep", "wake", "attack", "corpse")):
        return "life"
    return None


def summarize(db_path: Path, source_name: str, source_hash: str, source_bytes: int, deadline: float) -> dict[str, Any]:
    uri = db_path.resolve().as_uri() + "?mode=ro&immutable=1"
    try:
        db = sqlite3.connect(uri, uri=True, timeout=2)
        db.execute("PRAGMA query_only=ON")
        db.set_progress_handler(lambda: 1 if time.monotonic() > deadline else 0, 1000)
        version = int(db.execute("PRAGMA user_version").fetchone()[0])
        if version not in SUPPORTED_VERSIONS:
            raise RetentionError(f"unsupported history schema version {version}; preserving candidate")
        tables = {r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        if not {"records", "events"}.issubset(tables):
            raise RetentionError("missing required history tables; preserving candidate")
        integrity = db.execute("PRAGMA quick_check").fetchone()[0]
        if integrity != "ok":
            raise RetentionError(f"SQLite quick_check failed ({integrity}); preserving candidate")
        columns = {r[1] for r in db.execute("PRAGMA table_info(records)")}
        required = {"id", "tick", "at_ms", "kind", "state_json", "metrics_json"}
        if not required.issubset(columns):
            raise RetentionError("unsupported records table shape; preserving candidate")
        event_columns = {r[1] for r in db.execute("PRAGMA table_info(events)")}
        if not {"record_id", "kind", "payload"}.issubset(event_columns):
            raise RetentionError("unsupported events table shape; preserving candidate")
        count = int(db.execute("SELECT count(*) FROM records").fetchone()[0])
        if count == 0:
            raise RetentionError("empty history shard; preserving candidate")
        first = db.execute("SELECT id,tick,at_ms,kind,state_json,metrics_json FROM records ORDER BY id ASC LIMIT 1").fetchone()
        last = db.execute("SELECT id,tick,at_ms,kind,state_json,metrics_json FROM records ORDER BY id DESC LIMIT 1").fetchone()
        checkpoints = []
        for row in (first, last):
            state = _parse_json(row[4], "fullState checkpoint")
            metrics = _parse_json(row[5], "metrics")
            if not isinstance(state, dict):
                raise RetentionError("checkpoint is not an object; preserving candidate")
            state_version = state.get("version")
            state_tick = state.get("tick")
            if (isinstance(state_version, bool) or not isinstance(state_version, int)
                    or not MIN_STATE_VERSION <= state_version <= MAX_STATE_VERSION):
                raise RetentionError("unsupported State version; preserving candidate")
            if (isinstance(row[1], bool) or not isinstance(row[1], int)
                    or isinstance(state_tick, bool) or not isinstance(state_tick, int)
                    or state_tick != row[1]):
                raise RetentionError("checkpoint tick does not match record tick; preserving candidate")
            checkpoints.append({"record_id": row[0], "tick": row[1], "at_ms": row[2], "kind": row[3],
                                "full_state": state, "metrics": metrics})
        totals: dict[str, int] = collections.Counter()
        examples = []
        omitted = 0
        cursor = db.execute("SELECT record_id,kind,payload FROM events ORDER BY record_id,kind,payload")
        while True:
            _deadline_check(deadline)
            rows = cursor.fetchmany(512)
            if not rows:
                break
            for record_id, kind, payload in rows:
                kind = str(kind)
                totals[kind] = totals.get(kind, 0) + 1
                category = _event_class(kind)
                if category is None:
                    continue
                if len(examples) >= MAX_EXAMPLES:
                    omitted += 1
                    continue
                parsed = _parse_json(payload, "selected event payload")
                examples.append({"record_id": record_id, "kind": kind, "category": category, "payload": parsed})
        last_metrics = checkpoints[-1]["metrics"]
        return {"format": "curious-history-summary-v1", "source": source_name,
                "source_sha256": source_hash, "source_bytes": source_bytes, "sqlite_user_version": version,
                "record_count": count, "event_type_totals": dict(sorted(totals.items())),
                "first_checkpoint": checkpoints[0], "last_checkpoint": checkpoints[1],
                "last_metrics": last_metrics, "selected_events": examples,
                "selected_events_truncated": omitted > 0,
                "selected_events_omitted_count": omitted,
                "selected_events_limit": MAX_EXAMPLES,
                "event_example_selection": "first 20 life/error events in record_id, kind, payload order"}
    except sqlite3.DatabaseError as exc:
        raise RetentionError(f"SQLite read failed ({type(exc).__name__}); preserving candidate") from exc
    finally:
        try:
            db.close()
        except (UnboundLocalError, sqlite3.Error):
            pass


def _fsync_dir(directory: Path) -> None:
    fd = os.open(directory, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def _artifact_path(evidence_dir: Path, source: Path, source_hash: str) -> Path:
    return evidence_dir / f"{source.name}.{source_hash}.summary.json.gz"


def _atomic_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(mode=0o750, parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.part")
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    try:
        with temp.open("wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temp, path)
        _fsync_dir(path.parent)
    finally:
        temp.unlink(missing_ok=True)


def _read_artifact(path: Path) -> dict[str, Any]:
    try:
        with gzip.open(path, "rt", encoding="utf-8") as stream:
            value = json.load(stream)
        if not isinstance(value, dict) or value.get("format") != "curious-history-summary-v1":
            raise ValueError("bad format")
        return value
    except (OSError, ValueError, TypeError, json.JSONDecodeError) as exc:
        raise RetentionError("existing summary proof is invalid") from exc


def _write_artifact(path: Path, summary: dict[str, Any]) -> None:
    path.parent.mkdir(mode=0o750, parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.part")
    payload = json.dumps(summary, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    try:
        with temp.open("wb") as raw:
            with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as compressed:
                compressed.write(payload)
            raw.flush()
            os.fsync(raw.fileno())
        os.replace(temp, path)
        _fsync_dir(path.parent)
    finally:
        temp.unlink(missing_ok=True)
    verified = _read_artifact(path)
    if verified != summary:
        raise RetentionError("summary artifact verification mismatch")


def _shard_inventory(root: Path) -> tuple[dict[int, list[Path]], int | None]:
    files: dict[int, list[Path]] = collections.defaultdict(list)
    newest = None
    for path in root.iterdir():
        if path.is_symlink() or not path.is_file():
            continue
        match = SHARD_RE.fullmatch(path.name) or RAW_RE.fullmatch(path.name)
        if not match:
            continue
        shard = int(match.group(1))
        newest = shard if newest is None else max(newest, shard)
        files[shard].append(path)
    return files, newest


def run(root: Path, keep_days: int, byte_budget: int, apply: bool) -> list[dict[str, Any]]:
    for name, value in (("keep_days", keep_days), ("byte_budget", byte_budget)):
        if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
            raise RetentionError(f"{name} must be a positive integer")
    deadline = time.monotonic() + MAX_SECONDS
    root = root.resolve(strict=True)
    if not root.is_dir():
        raise RetentionError("data directory is not a directory")
    evidence = root / "history-retention"
    if evidence.is_symlink() or (evidence.exists() and not evidence.is_dir()):
        raise RetentionError("history-retention evidence path is not a safe directory")
    inventory, newest = _shard_inventory(root)
    now = time.time()
    archived_total = sum(p.stat().st_size for paths in inventory.values() for p in paths
                         if p.name.endswith((".sqlite.gz", ".sqlite.zst")))
    candidates = []
    for shard, paths in inventory.items():
        if shard == newest or len(paths) != 1:
            continue
        source = paths[0]
        if not source.name.endswith((".sqlite.gz", ".sqlite.zst")):
            continue
        age = max(0.0, now - source.stat().st_mtime)
        candidates.append((shard, source, age, source.stat().st_size))
    candidates.sort(key=lambda item: (-item[2], item[0]))
    selected = []
    projected = archived_total
    for item in candidates:
        _, _, age, size = item
        old = age >= keep_days * 86400
        over_budget = projected > byte_budget
        if old or over_budget:
            selected.append(item)
            projected -= size
    if not apply:
        return [{"shard": shard, "source": source.name, "source_bytes": size,
                 "age_days": round(age / 86400, 2), "eligible": True, "deleted": False}
                for shard, source, age, size in selected[:MAX_BATCH]]
    results = []
    for shard, source, age, size in selected[:MAX_BATCH]:
        _deadline_check(deadline)
        stat_before = source.stat(follow_symlinks=False)
        sig = {"device": stat_before.st_dev, "inode": stat_before.st_ino, "size": stat_before.st_size,
               "mtime_ns": stat_before.st_mtime_ns}
        sha, hashed_bytes = _sha256(source, deadline)
        if hashed_bytes != sig["size"]:
            raise RetentionError("source changed while hashing; preserving candidate")
        artifact = _artifact_path(evidence, source, sha)
        proof = artifact.with_suffix(artifact.suffix + ".proof.json")
        if artifact.is_symlink() or proof.is_symlink():
            raise RetentionError("summary or proof path is a symlink; preserving candidate")
        proof_exists = proof.exists()
        artifact_exists = artifact.exists()
        if proof_exists and not artifact_exists:
            raise RetentionError("checksum proof has no summary artifact; preserving candidate")
        if artifact_exists and proof_exists:
            summary = _read_artifact(artifact)
        else:
            # With no complete checksum proof, rebuild from the still-present source.
            # Never trust a matching source_hash field inside a potentially replaced artifact.
            with tempfile.TemporaryDirectory(prefix="curious-retention-") as work:
                db_path = Path(work) / "history.sqlite"
                _inflate(source, db_path, deadline)
                summary = summarize(db_path, source.name, sha, hashed_bytes, deadline)
            if artifact_exists:
                try:
                    existing_summary = _read_artifact(artifact)
                except RetentionError:
                    existing_summary = None
                if existing_summary == summary:
                    pass
                else:
                    _write_artifact(artifact, summary)
            else:
                _write_artifact(artifact, summary)
        if summary.get("source_sha256") != sha or summary.get("source_bytes") != hashed_bytes or summary.get("source") != source.name:
            raise RetentionError("summary metadata does not match source; preserving candidate")
        artifact_hash, artifact_bytes = _sha256(artifact, deadline)
        proof_value = {"format": "curious-history-proof-v1", "source": source.name,
                       "source_signature": sig, "source_sha256": sha, "source_bytes": hashed_bytes,
                       "summary": artifact.name, "summary_sha256": artifact_hash,
                       "summary_bytes": artifact_bytes}
        if proof.exists():
            try:
                existing_proof = json.loads(proof.read_text(encoding="utf-8"))
            except (OSError, ValueError) as exc:
                raise RetentionError("existing checksum proof is invalid; preserving candidate") from exc
            if existing_proof != proof_value:
                raise RetentionError("existing checksum proof differs; preserving candidate")
        else:
            _atomic_json(proof, proof_value)
        # Recheck exact identity and digest after proof is durable, immediately before any unlink.
        stat_after = source.stat(follow_symlinks=False)
        if (stat_after.st_dev, stat_after.st_ino, stat_after.st_size, stat_after.st_mtime_ns) != (
                sig["device"], sig["inode"], sig["size"], sig["mtime_ns"]):
            raise RetentionError("source signature changed after summary; preserving candidate")
        sha_after, size_after = _sha256(source, deadline)
        if sha_after != sha or size_after != hashed_bytes:
            raise RetentionError("source hash changed after summary; preserving candidate")
        if apply:
            source.unlink()
            _fsync_dir(root)
        results.append({"shard": shard, "source": source.name, "source_sha256": sha,
                        "source_bytes": size, "age_days": round(age / 86400, 2),
                        "summary": str(artifact), "deleted": apply})
    return results


def positive_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("must be an integer") from exc
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", required=True, type=Path, help="closed shard directory")
    parser.add_argument("--keep-days", type=positive_int, default=DEFAULT_KEEP_DAYS)
    parser.add_argument("--byte-budget", type=positive_int, default=DEFAULT_BUDGET,
                        help="compressed shard byte budget")
    parser.add_argument("--apply", action="store_true", help="write proofs and unlink eligible closed shards")
    args = parser.parse_args(argv)
    try:
        results = run(args.data_dir, args.keep_days, args.byte_budget, args.apply)
        print(json.dumps({"mode": "apply" if args.apply else "dry-run", "max_batch": MAX_BATCH,
                          "max_seconds": MAX_SECONDS, "results": results}, sort_keys=True))
        return 0
    except (OSError, RetentionError, sqlite3.Error, subprocess.SubprocessError) as exc:
        print(f"history-retention: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
