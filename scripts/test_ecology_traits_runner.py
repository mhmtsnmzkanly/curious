import importlib.util
from pathlib import Path
import sys
import tempfile
import time
import unittest


RUNNER = Path(__file__).resolve().parent / "run_ecology_traits_lab.py"
SPEC = importlib.util.spec_from_file_location("ecology_traits_runner", RUNNER)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class SeedAuditTests(unittest.TestCase):
    def test_ignores_tick_and_energy_values_but_reads_declared_seed_fields(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "events.jsonl").write_text(
                '{"tick":50001,"energy":50002}\n'
                '{"world_seed":50002}\n'
                '"evaluation_seeds": [\n  50001,\n  60000\n]\n',
                encoding="utf-8",
            )
            (root / "trials.csv").write_text(
                "world_seed,score\n50003,3\n150003,4\n", encoding="utf-8"
            )
            (root / "seed-50004.json").write_text("{}\n", encoding="utf-8")
            (root / "snapshot.bin").write_bytes(b"50005")

            result = runner.audit_seeds(root, [50001, 50002, 50003, 50004, 50005])

            self.assertEqual(result["matches"]["50001"], ["events.jsonl:4"])
            self.assertEqual(result["matches"]["50002"], ["events.jsonl:2"])
            self.assertEqual(result["matches"]["50003"], ["trials.csv:2"])
            self.assertEqual(result["matches"]["50004"], ["seed-50004.json"])
            self.assertEqual(result["matches"]["50005"], [])
            self.assertEqual(result["unseen"], [50005])

    def test_process_timeout_does_not_wait_for_a_partial_json_line(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            raw = root / "raw.jsonl"
            stderr = root / "stderr.log"
            started = time.monotonic()
            result = runner.capture_process(
                [sys.executable, "-c", "import os,time; os.write(1,b'{partial'); time.sleep(3)"],
                root,
                raw,
                stderr,
                max_seconds=0.2,
                max_output_bytes=1024,
            )

            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["timed_out"])
            self.assertLess(time.monotonic() - started, 1.5)
            self.assertEqual(raw.read_bytes(), b"{partial")
            self.assertTrue(result["finished_at"])

    def test_process_timeout_still_applies_after_child_closes_stdout(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            raw = root / "raw.jsonl"
            stderr = root / "stderr.log"
            started = time.monotonic()
            result = runner.capture_process(
                [sys.executable, "-c", "import os,time; os.close(1); time.sleep(0.5)"],
                root,
                raw,
                stderr,
                max_seconds=0.05,
                max_output_bytes=1024,
            )

            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["timed_out"])
            self.assertLess(time.monotonic() - started, 0.4)
            self.assertTrue(result["finished_at"])


if __name__ == "__main__":
    unittest.main()
