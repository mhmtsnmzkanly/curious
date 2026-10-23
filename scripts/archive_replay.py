#!/usr/bin/env python3
"""Serve a bounded, read-only browser replay of Curious SQLite history."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import gzip
import json
import os
import re
import shutil
import sqlite3
import subprocess
import tempfile
import threading
import time
from collections import OrderedDict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, quote, urlsplit

MAX_UNPACKED = 2 * 1024 * 1024 * 1024
MAX_CACHE = 256 * 1024 * 1024
MAX_HISTORY_STATE_ROW = 8 * 1024 * 1024
MAX_HISTORY_READ = 64 * 1024 * 1024
MAX_HISTORY_SECONDS = 2.0
SHARD_RE = re.compile(r"^history-(\d{6,})\.sqlite(?:\.(?:gz|zst))?$")


class ReplayError(Exception):
    pass


class ArchiveStore:
    """Resolve one shard at a time and keep at most two unpacked shards on disk."""

    def __init__(self, directory: Path, cache_dir: Path | None = None, max_cache: int = MAX_CACHE):
        self.directory = directory.resolve(strict=True)
        if not self.directory.is_dir():
            raise ReplayError("archive path must be a directory")
        self._owned_cache = tempfile.TemporaryDirectory(prefix="curious-replay-") if cache_dir is None else None
        self.cache = Path(self._owned_cache.name if self._owned_cache else cache_dir).resolve()
        self.cache.mkdir(parents=True, exist_ok=True)
        self.max_cache = max_cache
        self._cache: OrderedDict[int, tuple[Path, int]] = OrderedDict()
        self._lock = threading.RLock()

    def shards(self) -> list[dict]:
        found: dict[int, Path] = {}
        for path in self.directory.iterdir():
            match = SHARD_RE.fullmatch(path.name)
            if match and path.is_file() and path.resolve().parent == self.directory:
                shard = int(match.group(1))
                # Prefer the active SQLite file when both raw and sealed names exist.
                if shard not in found or path.suffix == ".sqlite":
                    found[shard] = path
        result = []
        for shard, path in sorted(found.items()):
            # Listing must stay cheap with hundreds of cold shards. Inspect a
            # chosen shard only when its timeline is requested.
            result.append({"shard": shard, "compressed": path.name.endswith((".gz", ".zst")),
                           "bytes": path.stat().st_size})
        return result

    def _source(self, shard: int) -> Path:
        if not isinstance(shard, int) or shard < 0:
            raise ReplayError("invalid shard")
        options = [self.directory / f"history-{shard:06}.sqlite",
                   self.directory / f"history-{shard:06}.sqlite.zst",
                   self.directory / f"history-{shard:06}.sqlite.gz"]
        for path in options:
            if path.is_file() and path.resolve().parent == self.directory:
                return path
        raise ReplayError("shard not found")

    def _unpack(self, shard: int, source: Path) -> Path:
        cached = self._cache.get(shard)
        if cached and cached[0].exists():
            self._cache.move_to_end(shard)
            return cached[0]
        target = self.cache / f"history-{shard:06}.sqlite"
        partial = self.cache / f".history-{shard:06}.{os.getpid()}.part"
        if source.name.endswith(".gz"):
            reader = gzip.open(source, "rb")
            process = None
        else:
            try:
                process = subprocess.Popen(["zstd", "-q", "-d", "-c", "--", str(source)], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
            except FileNotFoundError as exc:
                raise ReplayError("zstd CLI is required to read .zst shards") from exc
            reader = process.stdout
        size = 0
        try:
            with reader, partial.open("wb") as output:
                while True:
                    chunk = reader.read(1024 * 1024)
                    if not chunk:
                        break
                    size += len(chunk)
                    if size > MAX_UNPACKED:
                        raise ReplayError("shard exceeds 2 GiB inspection limit")
                    output.write(chunk)
            if process:
                process.wait()
                if process.returncode != 0:
                    raise ReplayError("zstd failed to decode shard")
            os.replace(partial, target)
            self._cache[shard] = (target, size)
            self._cache.move_to_end(shard)
            total = sum(n for _, n in self._cache.values())
            while total > self.max_cache and len(self._cache) > 1 or len(self._cache) > 2:
                old, (path, n) = self._cache.popitem(last=False)
                try:
                    path.unlink()
                except FileNotFoundError:
                    pass
                total -= n
            return target
        except Exception:
            partial.unlink(missing_ok=True)
            if process and process.poll() is None:
                process.kill(); process.wait()
            raise

    @contextmanager
    def connection(self, shard: int):
        with self._lock:
            source = self._source(shard)
            path = source if source.suffix == ".sqlite" else self._unpack(shard, source)
            uri = path.as_uri() + "?mode=ro"
            db = sqlite3.connect(uri, uri=True, timeout=3)
            db.row_factory = sqlite3.Row
            db.execute("PRAGMA query_only=ON")
        try:
            yield db
        finally:
            db.close()


class ReplayApplication:
    def __init__(self, store: ArchiveStore):
        self.store = store

    def shards(self):
        return self.store.shards()

    def timeline(self, shard: int, after: int, limit: int):
        if limit < 1 or limit > 500 or after < 0:
            raise ReplayError("invalid timeline page")
        with self.store.connection(shard) as db:
            rows = db.execute("SELECT id,tick,at_ms,kind,metrics_json FROM records WHERE id>? ORDER BY id LIMIT ?", (after, limit)).fetchall()
        return [{"id": r["id"], "tick": r["tick"], "at_ms": r["at_ms"], "kind": r["kind"],
                 "metrics": json.loads(r["metrics_json"])} for r in rows]

    def record(self, shard: int, record_id: int):
        if record_id < 1:
            raise ReplayError("invalid record id")
        with self.store.connection(shard) as db:
            row = db.execute("SELECT id,tick,at_ms,kind,state_json,metrics_json FROM records WHERE id=?", (record_id,)).fetchone()
            if row is None:
                raise ReplayError("record not found")
            try:
                decisions = [json.loads(r[0]) for r in db.execute(
                    "SELECT trace_json FROM decision_traces WHERE record_id=? ORDER BY entity_id", (record_id,))]
            except sqlite3.OperationalError:
                decisions = []  # pre-V2 shards have no decision_traces table
            events = [json.loads(r[0]) for r in db.execute("SELECT payload FROM events WHERE record_id=? ORDER BY ordinal", (record_id,))]
        return {"id": row["id"], "tick": row["tick"], "at_ms": row["at_ms"], "kind": row["kind"],
                "state": json.loads(row["state_json"]), "metrics": json.loads(row["metrics_json"]),
                "decisions": decisions, "events": events}

    def individual_history(self, shard: int, entity_id: int, after: int = 0, limit: int = 50, before: int | None = None):
        """Return a bounded page of this individual's recorded decisions in one shard."""
        if entity_id < 0 or after < 0 or limit < 1 or limit > 100 or (before is not None and before < 1):
            raise ReplayError("invalid individual history page")
        started = time.monotonic()
        db_deadline = started + MAX_HISTORY_SECONDS
        with self.store.connection(shard) as db:
            db.set_progress_handler(lambda: 1 if time.monotonic() > db_deadline else 0, 1000)
            try:
                if before is None:
                    rows = db.execute(
                        "SELECT r.id,r.tick,r.at_ms,r.kind,length(r.state_json) AS state_bytes, "
                        "CASE WHEN length(r.state_json)<=? THEN r.state_json ELSE NULL END AS state_json, "
                        "length(t.trace_json) AS trace_bytes, "
                        "CASE WHEN length(t.trace_json)<=? THEN t.trace_json ELSE NULL END AS trace_json "
                        "FROM decision_traces t JOIN records r ON r.id=t.record_id "
                        "WHERE t.entity_id=? AND r.id>? ORDER BY r.id LIMIT ?",
                        (MAX_HISTORY_STATE_ROW, 1024 * 1024, entity_id, after, limit),
                    )
                else:
                    rows = db.execute(
                        "SELECT r.id,r.tick,r.at_ms,r.kind,length(r.state_json) AS state_bytes, "
                        "CASE WHEN length(r.state_json)<=? THEN r.state_json ELSE NULL END AS state_json, "
                        "length(t.trace_json) AS trace_bytes, "
                        "CASE WHEN length(t.trace_json)<=? THEN t.trace_json ELSE NULL END AS trace_json "
                        "FROM decision_traces t JOIN records r ON r.id=t.record_id "
                        "WHERE t.entity_id=? AND r.id<? ORDER BY r.id DESC LIMIT ?",
                        (MAX_HISTORY_STATE_ROW, 1024 * 1024, entity_id, before, limit),
                    )
            except sqlite3.OperationalError as exc:
                # Old shards may predate decision_traces entirely.
                if "no such table" in str(exc):
                    return {"items": [], "has_more": False, "next_before": None,
                            "next_after": None, "budget_limited": False}
                raise
            result = []
            bytes_read = 0
            budget_limited = False
            continuation = None
            for row in rows:
                if time.monotonic() - started > MAX_HISTORY_SECONDS:
                    budget_limited = True
                    continuation = row["id"] + 1 if before is not None else row["id"] - 1
                    break
                bytes_read += min(int(row["state_bytes"] or 0), MAX_HISTORY_STATE_ROW) + min(int(row["trace_bytes"] or 0), 1024 * 1024)
                if bytes_read > MAX_HISTORY_READ:
                    budget_limited = True
                    continuation = row["id"] + 1 if before is not None else row["id"] - 1
                    break
                trace = json.loads(row["trace_json"]) if row["trace_json"] else None
                try:
                    event_rows = db.execute(
                        "SELECT CASE WHEN length(payload)<=65536 THEN payload ELSE NULL END FROM events WHERE record_id=? AND ("
                        "json_extract(payload,'$.EntityMoved.id')=? OR "
                        "json_extract(payload,'$.EntityAte.id')=? OR "
                        "json_extract(payload,'$.EntityDrank.id')=? OR "
                        "json_extract(payload,'$.EntityDied.id')=? OR "
                        "json_extract(payload,'$.EntityBorn.id')=? OR "
                        "json_extract(payload,'$.EntitySlept.id')=? OR "
                        "json_extract(payload,'$.HuntEpisodeEnded.id')=? OR "
                        "json_extract(payload,'$.ActionRejected.id')=? OR "
                        "json_extract(payload,'$.EntityAttacked.attacker')=? OR "
                        "json_extract(payload,'$.EntityAttacked.target')=? OR "
                        "json_extract(payload,'$.EntityMated.parent1')=? OR "
                        "json_extract(payload,'$.EntityMated.parent2')=? OR "
                        "json_extract(payload,'$.EntityMated.child')=? OR "
                        "json_extract(payload,'$.CorpseConsumed.id')=? OR "
                        "json_extract(payload,'$.CorpseConsumed.eater')=?) ORDER BY ordinal LIMIT 50",
                        (row["id"], *([entity_id] * 15)),
                    ).fetchall()
                except sqlite3.OperationalError as exc:
                    if "no such table" in str(exc):
                        event_rows = []
                    else:
                        raise
                events = [json.loads(e[0]) for e in event_rows if e[0] is not None]
                state = json.loads(row["state_json"]) if row["state_json"] else {}
                actor = next((e for e in state.get("entities", []) if e.get("id") == entity_id), None)
                result.append({
                    "id": row["id"], "shard": shard, "tick": row["tick"], "at_ms": row["at_ms"],
                    "kind": row["kind"], "trace": trace, "actor": actor,
                    "entity_id": entity_id,
                    "state_available": row["state_json"] is not None,
                    "trace_available": row["trace_json"] is not None,
                    "events_truncated": len(event_rows) >= 50 or any(e[0] is None for e in event_rows),
                    "events": events,
                })
            db.set_progress_handler(None, 0)
        if before is not None:
            result.reverse()
            next_before = continuation if budget_limited else (result[0]["id"] if result else None)
            has_more = budget_limited or len(result) == limit
        else:
            next_after = continuation if budget_limited else (result[-1]["id"] if result else None)
            has_more = budget_limited or len(result) == limit
            next_before = None
        return {"items": result, "has_more": has_more,
                "next_before": next_before, "next_after": None if before is not None else next_after,
                "budget_limited": budget_limited}


class ReplayHandler(BaseHTTPRequestHandler):
    app: ReplayApplication
    web: Path
    routes = {"/", "/replay", "/replay.html", "/replay.js", "/replay.css", "/experience.js", "/documentary.js"}

    def log_message(self, fmt, *args):
        print(f"{self.client_address[0]} {fmt % args}")

    def send_json(self, value, status=200):
        body = json.dumps(value, separators=(",", ":")).encode()
        self.send_response(status); self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body))); self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff"); self.end_headers(); self.wfile.write(body)

    def _write_rejection(self):
        self.send_json({"error": "GET only"}, 405)

    do_POST = _write_rejection
    do_PUT = _write_rejection
    do_PATCH = _write_rejection
    do_DELETE = _write_rejection

    def do_GET(self):
        parsed = urlsplit(self.path)
        if parsed.path in self.routes:
            name = "/replay.html" if parsed.path in {"/", "/replay"} else parsed.path
            path = self.web / name.lstrip("/")
            try:
                body = path.read_bytes()
            except OSError:
                self.send_error(404); return
            content_type = {".html":"text/html; charset=utf-8", ".js":"text/javascript; charset=utf-8", ".css":"text/css; charset=utf-8"}[path.suffix]
            self.send_response(200); self.send_header("Content-Type", content_type); self.send_header("Content-Length", str(len(body)))
            self.send_header("X-Content-Type-Options", "nosniff"); self.end_headers(); self.wfile.write(body); return
        try:
            q = parse_qs(parsed.query, strict_parsing=True)
            def integer(key, default=None):
                raw = q.get(key, [default])[0]
                if raw is None: raise ReplayError(f"missing {key}")
                if not re.fullmatch(r"\d{1,19}", str(raw)): raise ReplayError(f"invalid {key}")
                value = int(raw)
                if value > (1 << 63) - 1: raise ReplayError(f"invalid {key}")
                return value
            if parsed.path == "/api/shards" and not q:
                data = self.app.shards()
            elif parsed.path == "/api/timeline" and set(q) <= {"shard", "after", "limit"}:
                data = self.app.timeline(integer("shard"), integer("after", "0"), integer("limit", "200"))
            elif parsed.path == "/api/shard-meta" and set(q) == {"shard"}:
                shard = integer("shard")
                with self.app.store.connection(shard) as db:
                    row = db.execute("SELECT MIN(tick),MAX(tick),COUNT(*) FROM records").fetchone()
                data = {"shard": shard, "first_tick": row[0], "last_tick": row[1], "records": row[2]}
            elif parsed.path == "/api/record" and set(q) == {"shard", "id"}:
                data = self.app.record(integer("shard"), integer("id"))
            elif parsed.path == "/api/individual-history" and set(q) <= {"shard", "id", "after", "before", "limit"} and {"shard", "id"} <= set(q):
                data = self.app.individual_history(integer("shard"), integer("id"), integer("after", "0"), integer("limit", "50"), integer("before") if "before" in q else None)
            else:
                self.send_json({"error":"unknown read-only endpoint"}, 404); return
            self.send_json(data)
        except (ReplayError, ValueError, sqlite3.Error, OSError) as exc:
            self.send_json({"error": str(exc)}, 400)


def serve(directory: Path, host="127.0.0.1", port=7883, cache_dir=None, web_dir=None):
    store = ArchiveStore(directory, cache_dir)
    handler = type("BoundReplayHandler", (ReplayHandler,), {
        "app": ReplayApplication(store), "web": Path(web_dir or Path(__file__).resolve().parent.parent / "web")})
    server = ThreadingHTTPServer((host, port), handler)
    server.daemon_threads = True
    print(f"Read-only replay: http://{host}:{port}/replay (archives: {store.directory})", flush=True)
    try:
        server.serve_forever()
    finally:
        server.server_close()
        if store._owned_cache:
            store._owned_cache.cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", required=True, type=Path, help="existing Curious archive directory")
    parser.add_argument("--bind", default="127.0.0.1", help="defaults to loopback")
    parser.add_argument("--port", type=int, default=7883)
    parser.add_argument("--cache-dir", type=Path, help="optional cache path outside the source archive")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be between 1 and 65535")
    if args.cache_dir and args.cache_dir.resolve().is_relative_to(args.data_dir.resolve()):
        parser.error("cache directory must be outside the archive directory")
    serve(args.data_dir, args.bind, args.port, args.cache_dir)


if __name__ == "__main__":
    main()
