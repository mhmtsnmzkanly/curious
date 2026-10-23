import gzip
import hashlib
import json
import os
import sqlite3
import subprocess
import tempfile
import threading
import unittest
from http.client import HTTPConnection
from http.server import ThreadingHTTPServer
from pathlib import Path

import archive_replay as replay
from archive_replay import ArchiveStore, ReplayApplication, ReplayHandler, ReplayError


def make_shard(path: Path, start: int, count: int = 3):
    db = sqlite3.connect(path)
    db.executescript("""
      CREATE TABLE records(id INTEGER PRIMARY KEY,tick INTEGER,at_ms TEXT,kind TEXT,state_json TEXT,metrics_json TEXT,trace_json TEXT);
      CREATE TABLE decision_traces(record_id INTEGER,entity_id INTEGER,trace_json TEXT);
      CREATE TABLE events(record_id INTEGER,ordinal INTEGER,kind TEXT,payload TEXT);
    """)
    for i in range(count):
        tick = start + i
        state = {"tick": tick, "bounds": [0, 2, 0, 2], "entities": [{"id": 1, "species": "Herbivore", "generation": 2, "pos": {"x": 1, "y": 1}, "life": {"health": 10}}], "resources": []}
        metrics = {"alive": 1, "by_species": {"Herbivore": 1}, "max_living_generation": 2}
        db.execute("INSERT INTO records VALUES(?,?,?,?,?,?,?)", (i + 1, tick, str(tick * 1000), "tick", json.dumps(state), json.dumps(metrics), "{}"))
        db.execute("INSERT INTO decision_traces VALUES(?,?,?)", (i + 1, 1, json.dumps({"id": 1, "tick": tick, "decision": {"reason": "Explore"}})))
    db.commit(); db.close()


class ReplayTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "world"
        self.root.mkdir()
        self.cache = Path(self.temp.name) / "cache"
        raw0 = self.root / "history-000000.sqlite"
        make_shard(raw0, 10)
        with raw0.open("rb") as source, gzip.open(self.root / "history-000000.sqlite.gz", "wb") as target:
            target.write(source.read())
        raw0.unlink()
        raw1 = self.root / "history-000001.sqlite"
        make_shard(raw1, 20)
        self.zst = self.root / "history-000001.sqlite.zst"
        subprocess.run(["zstd", "-q", "-f", str(raw1), "-o", str(self.zst)], check=True)
        raw1.unlink()
        raw2 = self.root / "history-000002.sqlite"
        make_shard(raw2, 30)
        self.wal_writer = sqlite3.connect(raw2)
        self.wal_writer.execute("PRAGMA journal_mode=WAL")
        self.wal_writer.execute("UPDATE records SET metrics_json=? WHERE id=3", (json.dumps({"wal_visible": True}),))
        self.wal_writer.commit()

    def tearDown(self):
        self.wal_writer.close()
        self.temp.cleanup()

    def test_gzip_zstd_readonly_and_paging(self):
        original = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.iterdir() if not p.name.endswith("-shm")}
        store = ArchiveStore(self.root, self.cache, max_cache=1024)
        app = ReplayApplication(store)
        self.assertEqual([s["shard"] for s in app.shards()], [0, 1, 2])
        self.assertEqual(app.timeline(0, 0, 2)[0]["tick"], 10)
        self.assertEqual(app.timeline(0, 1, 2)[0]["tick"], 11)
        rec = app.record(1, 2)
        self.assertEqual(rec["state"]["tick"], 21)
        self.assertEqual(rec["decisions"][0]["decision"]["reason"], "Explore")
        life = app.individual_history(1, 1, before=3, limit=2)
        self.assertEqual([r["tick"] for r in life["items"]], [20, 21])
        self.assertEqual(life["items"][0]["trace"]["id"], 1)
        self.assertEqual(life["items"][0]["actor"]["id"], 1)
        self.assertTrue(life["has_more"])
        self.assertEqual(life["next_before"], life["items"][0]["id"])
        self.assertEqual(app.record(2, 3)["metrics"], {"wal_visible": True}, "read-only replay must see the live WAL");
        with store.connection(1) as db:
            self.assertEqual(db.execute("PRAGMA query_only").fetchone()[0], 1)
            with self.assertRaises(sqlite3.OperationalError):
                db.execute("DELETE FROM records")
        self.assertLessEqual(len(store._cache), 2)
        self.assertEqual(original, {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.iterdir() if not p.name.endswith("-shm")})

    def test_history_budget_returns_resumable_cursor(self):
        store = ArchiveStore(self.root, self.cache)
        app = ReplayApplication(store)
        with store.connection(1) as db:
            row = db.execute("SELECT length(r.state_json)+length(t.trace_json) FROM records r JOIN decision_traces t ON t.record_id=r.id WHERE r.id=2 AND t.entity_id=1").fetchone()
        old_budget = replay.MAX_HISTORY_READ
        try:
            replay.MAX_HISTORY_READ = row[0]
            page = app.individual_history(1, 1, before=3, limit=2)
            self.assertTrue(page["budget_limited"])
            self.assertTrue(page["has_more"])
            self.assertEqual(len(page["items"]), 1)
            self.assertEqual(page["items"][0]["id"], 2)
            self.assertEqual(page["next_before"], 2)
            continued = app.individual_history(1, 1, before=page["next_before"], limit=2)
            self.assertEqual([r["id"] for r in continued["items"]], [1])
        finally:
            replay.MAX_HISTORY_READ = old_budget

    def test_history_legacy_and_overflow_cursor_envelopes(self):
        legacy = self.root / "history-000003.sqlite"
        make_shard(legacy, 40)
        with sqlite3.connect(legacy) as db:
            db.execute("DROP TABLE decision_traces")
        page = ReplayApplication(ArchiveStore(self.root, self.cache)).individual_history(3, 1, before=3)
        self.assertEqual(page, {"items": [], "has_more": False, "next_before": None,
                                "next_after": None, "budget_limited": False})

    def test_bounded_fixed_http_api_and_write_methods(self):
        store = ArchiveStore(self.root, self.cache)
        handler = type("TestHandler", (ReplayHandler,), {"app": ReplayApplication(store), "web": Path(__file__).parent.parent / "web"})
        server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        try:
            conn = HTTPConnection("127.0.0.1", server.server_port)
            conn.request("GET", "/api/shards")
            response = conn.getresponse(); self.assertEqual(response.status, 200); self.assertEqual(len(json.loads(response.read())), 3)
            conn.request("GET", "/api/query?sql=DELETE%20FROM%20records")
            response = conn.getresponse(); self.assertEqual(response.status, 404); response.read()
            conn.request("GET", "/api/timeline?shard=0&after=0&limit=501")
            response = conn.getresponse(); self.assertEqual(response.status, 400); response.read()
            conn.request("GET", "/api/individual-history?shard=0&id=1&before=3&limit=101")
            response = conn.getresponse(); self.assertEqual(response.status, 400); response.read()
            conn.request("GET", "/api/individual-history?shard=0&id=9223372036854775808&before=3&limit=2")
            response = conn.getresponse(); self.assertEqual(response.status, 400); response.read()
            conn.request("GET", "/api/individual-history?shard=0&id=1&before=3&limit=2")
            response = conn.getresponse(); self.assertEqual(response.status, 200)
            payload = json.loads(response.read())
            self.assertEqual([r["tick"] for r in payload["items"]], [10, 11])
            self.assertIn("has_more", payload); self.assertIn("next_before", payload); self.assertIn("budget_limited", payload)
            conn.request("POST", "/api/record?shard=0&id=1", body=b'{"action":"step"}')
            response = conn.getresponse(); self.assertEqual(response.status, 405); response.read()
            conn.request("DELETE", "/api/record?shard=0&id=1")
            response = conn.getresponse(); self.assertEqual(response.status, 405); response.read()
            conn.close()
        finally:
            server.shutdown(); server.server_close(); thread.join()

    def test_symlink_shard_cannot_escape_archive_directory(self):
        elsewhere = Path(self.temp.name) / "outside.sqlite"
        make_shard(elsewhere, 99)
        link = self.root / "history-000003.sqlite"
        link.symlink_to(elsewhere)
        store = ArchiveStore(self.root, self.cache)
        self.assertNotIn(3, [s["shard"] for s in store.shards()])
        with self.assertRaises(ReplayError):
            with store.connection(3):
                pass


if __name__ == "__main__":
    unittest.main()
