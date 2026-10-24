import gzip
import hashlib
import json
import os
import sqlite3
import tempfile
import time
import unittest
from unittest.mock import patch
from pathlib import Path

import history_retention as retention


class HistoryRetentionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def make_db(self, path, version=2, event_count=3):
        db = sqlite3.connect(path)
        db.executescript("""
            CREATE TABLE records(id INTEGER PRIMARY KEY,tick INTEGER,at_ms TEXT,kind TEXT,state_json TEXT,metrics_json TEXT,trace_json TEXT);
            CREATE TABLE events(record_id INTEGER,ordinal INTEGER,kind TEXT,payload TEXT,PRIMARY KEY(record_id,ordinal));
            PRAGMA user_version=2;
        """)
        for i in (1, 2):
            db.execute("INSERT INTO records VALUES(?,?,?,?,?,?,?)",
                       (i, i * 10, str(i), "tick", json.dumps({"version": 18, "tick": i * 10, "entities": []}), json.dumps({"alive": i}), "{}"))
        for i in range(event_count):
            kind = "EntityBorn" if i % 2 == 0 else "ActionRejected"
            db.execute("INSERT INTO events VALUES(?,?,?,?)", (1 + i % 2, i, kind, json.dumps({kind: {"id": i}})))
        db.execute(f"PRAGMA user_version={int(version)}")
        db.commit()
        db.close()

    def archive(self, number=1, version=2, age_days=10, event_count=3):
        raw = self.root / f"raw-{number}.sqlite"
        self.make_db(raw, version, event_count)
        dest = self.root / f"history-{number:06}.sqlite.gz"
        with raw.open("rb") as inp, dest.open("wb") as out:
            with gzip.GzipFile(fileobj=out, mode="wb", mtime=0) as zipped:
                zipped.write(inp.read())
        raw.unlink()
        old = time.time() - age_days * 86400
        os.utime(dest, (old, old))
        return dest

    def test_dry_run_and_apply_preserve_raw_wal_and_newest(self):
        old = self.archive(1)
        active = self.root / "history-000002.sqlite"
        active.touch()
        (self.root / "history-000002.sqlite-wal").write_bytes(b"wal")
        raw = self.root / "history-000003.sqlite"
        raw.touch()
        before = retention.run(self.root, 7, 10**9, False)
        self.assertEqual([old.name], [row["source"] for row in before])
        self.assertTrue(old.exists())
        self.assertFalse((self.root / "history-retention").exists())
        result = retention.run(self.root, 7, 10**9, True)
        self.assertFalse(old.exists())
        self.assertTrue(active.exists())
        self.assertTrue((self.root / "history-000002.sqlite-wal").exists())
        self.assertTrue(raw.exists())
        self.assertTrue(result[0]["deleted"])
        proof = Path(result[0]["summary"] + ".proof.json")
        metadata = json.loads(proof.read_text())
        self.assertEqual(metadata["source_sha256"], result[0]["source_sha256"])
        self.assertEqual(hashlib.sha256(Path(result[0]["summary"]).read_bytes()).hexdigest(), metadata["summary_sha256"])

    def test_idempotent_after_proof_before_unlink(self):
        source = self.archive(1)
        (self.root / "history-000002.sqlite").touch()
        unlink = Path.unlink

        def fail_source_once(path, *args, **kwargs):
            if path == source:
                raise OSError("simulated interruption before unlink")
            return unlink(path, *args, **kwargs)

        with patch.object(Path, "unlink", fail_source_once):
            with self.assertRaises(OSError):
                retention.run(self.root, 7, 10**9, True)
        self.assertTrue(source.exists())
        proof_files = list((self.root / "history-retention").glob("*.proof.json"))
        self.assertEqual(1, len(proof_files))
        result = retention.run(self.root, 7, 10**9, True)
        self.assertFalse(source.exists())
        self.assertTrue(result[0]["deleted"])

    def test_valid_gzip_without_proof_is_rebuilt_before_delete(self):
        source = self.archive(1)
        (self.root / "history-000002.sqlite").touch()
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        artifact = retention._artifact_path(self.root / "history-retention", source, digest)
        artifact.parent.mkdir()
        tampered = {"format": "curious-history-summary-v1", "source": source.name,
                    "source_sha256": digest, "source_bytes": source.stat().st_size,
                    "selected_events": [{"payload": {"tampered": True}}]}
        with artifact.open("wb") as raw:
            with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as zipped:
                zipped.write(json.dumps(tampered).encode())

        unlink = Path.unlink
        def fail_source_once(path, *args, **kwargs):
            if path == source:
                raise OSError("leave source for artifact inspection")
            return unlink(path, *args, **kwargs)
        with patch.object(Path, "unlink", fail_source_once):
            with self.assertRaises(OSError):
                retention.run(self.root, 7, 10**9, True)
        rebuilt = retention._read_artifact(artifact)
        self.assertNotIn("tampered", json.dumps(rebuilt))
        self.assertEqual(10, rebuilt["first_checkpoint"]["tick"])
        self.assertTrue(source.exists())
        self.assertTrue(artifact.with_suffix(artifact.suffix + ".proof.json").exists())

    def test_symlinked_artifact_and_proof_are_rejected(self):
        source = self.archive(1)
        (self.root / "history-000002.sqlite").touch()
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        evidence = self.root / "history-retention"
        evidence.mkdir()
        artifact = retention._artifact_path(evidence, source, digest)
        outside = self.root / "outside"
        outside.write_bytes(b"outside")
        artifact.symlink_to(outside)
        with self.assertRaisesRegex(retention.RetentionError, "symlink"):
            retention.run(self.root, 7, 10**9, True)
        self.assertTrue(source.exists())
        artifact.unlink()

        with tempfile.TemporaryDirectory() as work:
            db = Path(work) / "history.sqlite"
            retention._inflate(source, db, time.monotonic() + 20)
            summary = retention.summarize(db, source.name, digest, source.stat().st_size,
                                          time.monotonic() + 20)
        retention._write_artifact(artifact, summary)
        proof = artifact.with_suffix(artifact.suffix + ".proof.json")
        proof.symlink_to(outside)
        with self.assertRaisesRegex(retention.RetentionError, "symlink"):
            retention.run(self.root, 7, 10**9, True)
        self.assertTrue(source.exists())

    def test_corrupt_and_unknown_version_are_preserved(self):
        corrupt = self.root / "history-000001.sqlite.gz"
        corrupt.write_bytes(b"not gzip")
        old = time.time() - 10 * 86400
        os.utime(corrupt, (old, old))
        legacy = self.archive(2, version=0)
        unknown = self.archive(3, version=99)
        (self.root / "history-000004.sqlite").touch()
        with self.assertRaises(retention.RetentionError):
            retention.run(self.root, 7, 10**9, True)
        self.assertTrue(corrupt.exists())
        self.assertTrue(legacy.exists())
        self.assertTrue(unknown.exists())
        corrupt.unlink()
        with self.assertRaisesRegex(retention.RetentionError, "unsupported history schema version"):
            retention.run(self.root, 7, 10**9, True)
        self.assertTrue(legacy.exists())
        self.assertTrue(unknown.exists())

    def test_checkpoint_totals_and_deterministic_example_cap(self):
        source = self.archive(1, event_count=30)
        with tempfile.TemporaryDirectory() as work:
            db = Path(work) / "history.sqlite"
            retention._inflate(source, db, time.monotonic() + 20)
            digest = hashlib.sha256(source.read_bytes()).hexdigest()
            a = retention.summarize(db, source.name, digest, source.stat().st_size, time.monotonic() + 20)
            b = retention.summarize(db, source.name, digest, source.stat().st_size, time.monotonic() + 20)
        self.assertEqual(a, b)
        self.assertEqual(a["first_checkpoint"]["tick"], 10)
        self.assertEqual(a["last_checkpoint"]["tick"], 20)
        self.assertEqual(sum(a["event_type_totals"].values()), 30)
        self.assertEqual(len(a["selected_events"]), 20)
        self.assertTrue(a["selected_events_truncated"])
        self.assertEqual(a["selected_events_omitted_count"], 10)

    def test_byte_budget_selects_oldest_archives_with_batch_cap(self):
        paths = [self.archive(i, age_days=i) for i in (1, 2, 3)]
        (self.root / "history-000004.sqlite").touch()
        result = retention.run(self.root, 999, 1, False)
        self.assertEqual(2, len(result))
        self.assertEqual(paths[2].name, result[0]["source"])
        self.assertEqual(paths[1].name, result[1]["source"])

    def test_malformed_configuration_is_rejected(self):
        with self.assertRaises(SystemExit):
            retention.main(["--data-dir", str(self.root), "--keep-days", "0"])
        with self.assertRaises(SystemExit):
            retention.main(["--data-dir", str(self.root), "--byte-budget", "-1"])
        with self.assertRaisesRegex(retention.RetentionError, "keep_days"):
            retention.run(self.root, 0, 100, True)
        with self.assertRaisesRegex(retention.RetentionError, "byte_budget"):
            retention.run(self.root, 1, -1, True)


if __name__ == "__main__":
    unittest.main()
