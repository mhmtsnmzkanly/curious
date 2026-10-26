import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import run_resolver_order_lab as supervisor


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fake_case_result(habitat: str, seed: int, tick: int, input_path: str, input_sha: str) -> dict:
    empty_branch = {
        "pre_state": {"policy": {"water_meal_priority_enabled": False}}, "pre_rng": 1,
        "plan_orders": {}, "pre_actor_bodies": [], "events": [], "event_totals": {},
        "post_state": {"policy": {"water_meal_priority_enabled": False}}, "post_rng": 1,
        "post_actor_bodies": [], "post_last_intents": [], "post_last_decisions": [],
    }
    native_off = {"events": [], "rng": 1, "last_intents": [], "last_decisions": [],
                  "state": {"policy": {"water_meal_priority_enabled": False}}}
    native_on = {**native_off, "state": {"policy": {"water_meal_priority_enabled": True}}}
    return {
        "schema": "curious-resolver-order-v1-case",
        "status": "complete",
        "case": {"habitat": habitat, "seed": seed, "target_tick": tick, "input_path": input_path, "input_sha256": input_sha},
        "prefix_replay_valid": True,
        "prefix_replay": {"from_initial_state_through": tick - 1},
        "pre_target_roundtrip": {"valid": True, "state_equal": True, "rng_equal": True},
        "common_prepared": {},
        "branch_roundtrip_valid": True,
        "candidate_b_equals_c_pre_resolution_audit": {
            "equal": True, "learner_interruptions_equal": True, "decision_trace_equal": True, "plan_content_equal": True,
        },
        "references": {
            "a_matches_native_off": True,
            "c_matches_native_on": True,
            "compared_fields": sorted(supervisor.REQUIRED_REFERENCE_FIELDS),
            "normalizations": sorted(supervisor.REQUIRED_NORMALIZATIONS),
            "native_off": native_off,
            "native_on": native_on,
        },
        "branches": {name: dict(empty_branch) for name in ("A", "B", "C")},
    }


class ResolverOrderSupervisorTest(unittest.TestCase):
    def _fixture(self, root: Path) -> tuple[Path, Path, Path, Path]:
        source = root / "source"
        source.mkdir()
        (source / "docs").mkdir()
        repo = Path(__file__).resolve().parents[1]
        for rel in (supervisor.PROTOCOL_REL, supervisor.ADDENDUM_REL):
            target = source / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((repo / rel).read_bytes())
        subprocess.run(["git", "init", "-q", str(source)], check=True)
        subprocess.run(["git", "-C", str(source), "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "add", "."], check=True)
        subprocess.run(["git", "-C", str(source), "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "fixture"], check=True)

        raw = root / "raw"
        raw.mkdir()
        worlds = []
        for habitat, seed, tick in supervisor.CASES:
            name = f"{habitat}-{seed}-control.jsonl"
            path = raw / name
            rows = [
                {"arm": "control", "habitat": habitat, "seed": seed, "initial_state": {"rng": seed}},
                {"type": "tick", "tick": tick - 1},
                {"type": "tick", "tick": tick},
            ]
            path.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
            worlds.append({"habitat": habitat, "seed": seed, "arm": "control", "raw": name,
                           "raw_bytes": path.stat().st_size, "raw_sha256": digest(path)})
        raw_manifest = raw / "manifest.json"
        raw_manifest.write_text(json.dumps({"schema": "curious-water-meal-priority-v1-manifest", "status": "complete", "worlds": worlds}), encoding="utf-8")

        binary = root / "fake-lab"
        binary.write_text("""#!/usr/bin/env python3
import hashlib, json, os, pathlib, sys
source, tick_text, input_sha, output = pathlib.Path(sys.argv[1]), int(sys.argv[2]), sys.argv[3], pathlib.Path(sys.argv[4])
header = json.loads(source.open().readline())
result = {
  'schema': 'curious-resolver-order-v1-case', 'status': 'complete',
  'case': {'habitat': header['habitat'], 'seed': header['seed'], 'target_tick': tick_text,
           'input_path': str(source.resolve()), 'input_sha256': input_sha},
  'prefix_replay_valid': True,
  'pre_target_roundtrip': {'valid': True, 'state_equal': True, 'rng_equal': True},
  'branch_roundtrip_valid': True,
  'candidate_b_equals_c_pre_resolution_audit': {'equal': True, 'learner_interruptions_equal': True,
      'decision_trace_equal': True, 'plan_content_equal': True},
  'references': {'a_matches_native_off': True, 'c_matches_native_on': True,
      'compared_fields': ['events','rng','last_intents','last_decisions','state'],
      'normalizations': ['policy.water_meal_priority_enabled only'],
      'native_off': {'events': [], 'rng': 1, 'last_intents': [], 'last_decisions': [],
          'state': {'policy': {'water_meal_priority_enabled': False}}},
      'native_on': {'events': [], 'rng': 1, 'last_intents': [], 'last_decisions': [],
          'state': {'policy': {'water_meal_priority_enabled': True}}}},
  'branches': {name: {'pre_state': {'policy': {'water_meal_priority_enabled': False}}, 'pre_rng': 1,
      'plan_orders': {}, 'pre_actor_bodies': [], 'events': [], 'event_totals': {},
      'post_state': {'policy': {'water_meal_priority_enabled': False}}, 'post_rng': 1,
      'post_actor_bodies': [], 'post_last_intents': [], 'post_last_decisions': []} for name in ('A','B','C')}
}
if os.environ.get('FAKE_MISMATCH'):
  result['references']['a_matches_native_off'] = False
output.write_text(json.dumps(result), encoding='utf-8')
""", encoding="utf-8")
        binary.chmod(0o755)
        return source, binary, raw, root / "run"

    def test_fake_child_completes_fixed_cases_and_registers_hashes(self):
        with tempfile.TemporaryDirectory() as temp:
            source, binary, raw, output = self._fixture(Path(temp))
            raw_manifest_sha = digest(raw / "manifest.json")
            with patch.object(supervisor, "RAW_MANIFEST_SHA256", raw_manifest_sha):
                argv = ["runner", "--source-root", str(source), "--binary", str(binary), "--raw-root", str(raw), "--output-dir", str(output)]
                with patch("sys.argv", argv):
                    self.assertEqual(supervisor.main(), 0)
            manifest = json.loads((output / "manifest.json").read_text())
            self.assertEqual(manifest["status"], "complete")
            self.assertEqual(len(manifest["cases"]), 5)
            self.assertEqual(manifest["limits"]["aggregate_run_bytes"], 64 * 1024 * 1024)
            self.assertEqual(len(list(output.glob("*/result.json"))), 5)

    def test_existing_output_directory_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "run"
            output.mkdir()
            (output / "marker").write_text("preserve")
            with patch("sys.argv", ["runner", "--source-root", "/missing", "--binary", "/missing", "--output-dir", str(output)]):
                with self.assertRaises(SystemExit):
                    supervisor.main()
            self.assertEqual((output / "marker").read_text(), "preserve")

    def test_first_reference_failure_is_preserved_without_retry(self):
        with tempfile.TemporaryDirectory() as temp:
            source, binary, raw, output = self._fixture(Path(temp))
            raw_manifest_sha = digest(raw / "manifest.json")
            with patch.object(supervisor, "RAW_MANIFEST_SHA256", raw_manifest_sha):
                argv = ["runner", "--source-root", str(source), "--binary", str(binary), "--raw-root", str(raw), "--output-dir", str(output)]
                with patch("sys.argv", argv), patch.dict(os.environ, {"FAKE_MISMATCH": "1"}):
                    self.assertEqual(supervisor.main(), 1)
            manifest = json.loads((output / "manifest.json").read_text())
            self.assertEqual(manifest["status"], "failed")
            self.assertEqual(len(manifest["cases"]), 1)
            self.assertEqual(manifest["cases"][0]["status"], "failed")
            self.assertTrue((output / manifest["cases"][0]["output"]).exists())
            self.assertIn("native reference equality failed", manifest["failure"]["message"])


if __name__ == "__main__":
    unittest.main()
