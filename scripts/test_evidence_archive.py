from __future__ import annotations

import json
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest import mock

try:
    import evidence_archive as archive
    import run_ecology_traits_lab as ecology_runner
except ModuleNotFoundError:
    from scripts import evidence_archive as archive
    from scripts import run_ecology_traits_lab as ecology_runner


class EvidenceArchiveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.experiments = self.root / "data/experiments"
        self.experiments.mkdir(parents=True)
        self.raw = self.experiments / "case" / "raw.jsonl"
        self.raw.parent.mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def run_archive(self, *, apply: bool = True) -> dict:
        return archive.archive_file(self.raw, root=self.root, apply=apply)

    @property
    def index(self) -> Path:
        return self.raw.with_name(self.raw.name + ".archive.json")

    @property
    def compressed(self) -> Path:
        return self.raw.with_name(self.raw.name + ".zst")

    def test_dry_run_does_not_write_or_remove(self) -> None:
        self.raw.write_bytes(b'{"seed":7}\n')
        result = self.run_archive(apply=False)
        self.assertTrue(result["dry_run"])
        self.assertTrue(self.raw.exists())
        self.assertFalse(self.index.exists())
        self.assertFalse(self.compressed.exists())

    def test_exact_roundtrip_and_idempotent_archive(self) -> None:
        original = b'{"kind":"run","seed":41}\r\n{"seeds":[3, 5]}\n\xff\n'
        self.raw.write_bytes(original)
        with self.assertRaises(archive.ArchiveError):
            self.run_archive()
        self.assertEqual(self.raw.read_bytes(), original)

        # JSONL seed auditing requires UTF-8; valid raw JSONL remains byte exact.
        original = b'{"kind":"run","seed":41}\r\n{"seeds":[3, 5]}\n'
        self.raw.write_bytes(original)
        first = self.run_archive()
        self.assertFalse(self.raw.exists())
        self.assertEqual(first["original_sha256"], hashlib.sha256(original).hexdigest())
        index_bytes = self.index.read_bytes()
        second = self.run_archive()
        self.assertTrue(second["already_archived"])
        self.assertEqual(self.index.read_bytes(), index_bytes)

        restored = archive.restore_index(self.index, root=self.root, apply=True)
        self.assertTrue(restored["restored"])
        self.assertEqual(self.raw.read_bytes(), original)
        again = archive.restore_index(self.index, root=self.root, apply=True)
        self.assertTrue(again["already_present"])

    def test_compressed_corruption_during_archive_never_removes_raw(self) -> None:
        self.raw.write_bytes(b'{"seed":81}\n')
        verify = archive._decompress_verified
        changed = False

        def corrupt_then_verify(path: Path, output: Path, size: int, digest: str):
            nonlocal changed
            if not changed:
                changed = True
                with path.open("r+b") as stream:
                    stream.seek(0)
                    stream.write(b"damaged")
            return verify(path, output, size, digest)

        with mock.patch.object(archive, "_decompress_verified", side_effect=corrupt_then_verify):
            with self.assertRaises(archive.ArchiveError):
                self.run_archive()
        self.assertTrue(self.raw.exists())
        self.assertEqual(self.raw.read_bytes(), b'{"seed":81}\n')

    def test_archive_seed_index_remains_visible_to_established_scanner(self) -> None:
        self.raw.write_text(
            '{"kind":"run","seeds":[50041,50042,50043,50044],'
            '"seed":50099,"command":"runner --world-seed 50123"}\n',
            encoding="utf-8",
        )
        wanted = [50041, 50099, 50123]
        before = ecology_runner.audit_seeds(self.experiments, wanted)
        self.assertEqual(before["unseen"], [])
        self.assertTrue(any("raw.jsonl" in path for paths in before["matches"].values() for path in paths))

        self.run_archive()
        metadata = json.loads(self.index.read_text())
        self.assertEqual(metadata["seeds"], [50041, 50042, 50043, 50044, 50099, 50123])
        self.assertEqual(metadata["seed_roles"], {
            "cli_seed": [50123],
            "seed": [50099],
            "seeds": [50041, 50042, 50043, 50044],
        })

        result = ecology_runner.audit_seeds(self.experiments, wanted)
        self.assertEqual(result["unseen"], [])
        self.assertTrue(any("raw.jsonl.archive.json" in path for paths in result["matches"].values() for path in paths))

    def test_path_escape_and_symlink_rejected(self) -> None:
        outside = self.root / "outside.jsonl"
        outside.write_text('{"seed":1}\n')
        with self.assertRaises(archive.ArchiveError):
            archive.archive_file(outside, root=self.root)
        link = self.experiments / "link.jsonl"
        link.symlink_to(outside)
        with self.assertRaises(archive.ArchiveError):
            archive.archive_file(link, root=self.root)

    def test_restore_rejects_traversal_and_symlink_archive(self) -> None:
        self.raw.write_text('{"seed":9}\n')
        self.run_archive()
        payload = json.loads(self.index.read_text())
        payload["originalpath"] = "data/experiments/../../outside.jsonl"
        self.index.write_text(json.dumps(payload))
        with self.assertRaises(archive.ArchiveError):
            archive.restore_index(self.index, root=self.root)

        self.index.unlink()
        self.compressed.unlink()
        self.raw.write_text('{"seed":9}\n')
        self.run_archive()
        archive_copy = self.root / "archive-copy.zst"
        archive_copy.write_bytes(self.compressed.read_bytes())
        self.compressed.unlink()
        self.compressed.symlink_to(archive_copy)
        with self.assertRaises(archive.ArchiveError):
            archive.restore_index(self.index, root=self.root)

    def test_source_change_after_archive_is_published_keeps_raw(self) -> None:
        self.raw.write_bytes(b'{"seed":12}\n')
        verify = archive._decompress_verified
        calls = 0

        def alter_after_published_verify(path: Path, output: Path, size: int, digest: str):
            nonlocal calls
            calls += 1
            result = verify(path, output, size, digest)
            if calls == 2:
                self.raw.write_bytes(b'{"seed":13}\n')
            return result

        with mock.patch.object(archive, "_decompress_verified", side_effect=alter_after_published_verify):
            with self.assertRaisesRegex(archive.ArchiveError, "fingerprint changed|bytes changed"):
                self.run_archive()
        self.assertTrue(self.raw.exists())
        self.assertEqual(self.raw.read_bytes(), b'{"seed":13}\n')

    def test_restore_never_overwrites_different_raw_destination(self) -> None:
        self.raw.write_text('{"seed":18}\n')
        self.run_archive()
        self.raw.write_text('{"seed":19}\n')
        with self.assertRaisesRegex(archive.ArchiveError, "refusing overwrite"):
            archive.restore_index(self.index, root=self.root, apply=True)
        self.assertEqual(self.raw.read_text(), '{"seed":19}\n')

    def test_index_seed_declarations_are_checked_against_archived_bytes(self) -> None:
        self.raw.write_text('{"seed":27}\n')
        self.run_archive()
        payload = json.loads(self.index.read_text())
        payload["seeds"] = [28]
        payload["seed_roles"] = {"seed": [28]}
        self.index.write_text(json.dumps(payload))
        with self.assertRaisesRegex(archive.ArchiveError, "seed declarations"):
            archive.restore_index(self.index, root=self.root, apply=True)
        self.assertFalse(self.raw.exists())

    def test_seed_scan_line_cap_fails_before_any_archive(self) -> None:
        self.raw.write_bytes(b'{"seed":"' + b"1" * (archive.MAX_SCAN_LINE_BYTES + 1) + b'"}\n')
        with self.assertRaisesRegex(archive.ArchiveError, "line exceeds"):
            self.run_archive()
        self.assertTrue(self.raw.exists())
        self.assertFalse(self.compressed.exists())


if __name__ == "__main__":
    unittest.main()
