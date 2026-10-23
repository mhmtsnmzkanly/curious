"""Focused runner safeguards; all process fixtures are non-simulation commands."""
import importlib.util
from copy import deepcopy
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "run_lifelong_water_lab", ROOT / "scripts/run_lifelong_water_lab.py"
)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class RunnerSafeguardTests(unittest.TestCase):
    def fixture_initial_state(self, seed=51099):
        positions = {1: (2, 2), 2: (2, 3)}
        entities = []
        for actor_id, (x, y) in positions.items():
            entities.append({
                "id": actor_id, "species": "Herbivore", "generation": 0,
                "controller": "AI", "phase": "Active", "pos": {"x": x, "y": y},
                "life": {"age": 0, "max_age": 512, "maturity_age": 501,
                         "max_water": 60, "water": 60, "max_energy": 80,
                         "energy": 80, "max_health": 120, "health": 120,
                         "speed": 3, "move_energy_cost": 1, "max_stamina": 16,
                         "stamina": 16, "vision_range": 6},
                "learner": {"enabled": actor_id == 1, "update_enabled": actor_id == 1,
                            "values": [], "episode": None, "last_episode": None},
            })
        sites, resources = [], []
        for pos, replenish, interval in [((0, 2), 4, 2), ((4, 2), 1, 4)]:
            point = {"x": pos[0], "y": pos[1]}
            sites.append({"pos": point, "kind": "Water", "capacity": 12,
                          "replenish": replenish, "interval": interval, "schedule": None})
            resources.append([point, {"Water": {"amount": 12}}])
        for pos in ((0, 0), (4, 0), (0, 4), (4, 4)):
            point = {"x": pos[0], "y": pos[1]}
            sites.append({"pos": point, "kind": "Food", "capacity": 20,
                          "replenish": 2, "interval": 2, "schedule": None})
            resources.append([point, {"Food": {"amount": 20}}])
        return {"tick": 0, "rng": seed, "bounds": [0, 4, 0, 4],
                "entities": entities, "environment": {"sites": sites},
                "resources": resources}

    def test_initial_fixture_is_valid_and_rejects_pinned_fields(self):
        valid = self.fixture_initial_state()
        runner.validate_initial_fixture(valid, 51099, "fixture test")
        mutations = []

        bounds = deepcopy(valid)
        bounds["bounds"] = [0, 5, 0, 4]
        mutations.append((bounds, "bounds"))

        food_rate = deepcopy(valid)
        food_rate["environment"]["sites"][2]["replenish"] = 1
        mutations.append((food_rate, "food"))

        preseeded = deepcopy(valid)
        preseeded["entities"][0]["learner"]["values"].append(
            {"pos": {"x": 0, "y": 2}, "value": 99, "visits": 1}
        )
        mutations.append((preseeded, "pre-seeded"))

        for altered, message in mutations:
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                runner.validate_initial_fixture(altered, 51099, "fixture test")

    def test_nonidentifiable_analysis_is_terminal_completed_capture(self):
        self.assertEqual(
            runner.terminal_manifest_status("completed", "non_identifiable"),
            "completed",
        )
        self.assertEqual(
            runner.terminal_manifest_status("completed", "completed"),
            "completed",
        )
        self.assertEqual(
            runner.terminal_manifest_status("failed", "non_identifiable"),
            "failed",
        )
        self.assertEqual(
            runner.terminal_manifest_status("completed", "malformed"),
            "failed",
        )

    def test_bounded_seed_line_rejects_oversize_before_json_decode(self):
        with tempfile.TemporaryDirectory() as temp:
            raw = Path(temp) / "raw.jsonl"
            raw.write_bytes(b"{" + b"x" * 32 + b"}\n")
            with patch.object(runner, "MAX_SEED_RECORD_BYTES", 16):
                with self.assertRaisesRegex(ValueError, "exceeds"):
                    runner.parse_raw(raw, "calibration", (51099,))

    def test_partial_or_truncated_seed_line_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            raw = Path(temp) / "raw.jsonl"
            raw.write_text('{"incomplete":true}', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "truncated"):
                runner.parse_raw(raw, "calibration", (51099,))

    def test_competitor_disappearance_requires_prior_corpse_and_death_event(self):
        corpse = {"phase": {"Corpse": {"remaining": 2}}, "life": {"health": 0}}
        removed = {"phase": "Removed", "life": {"health": 0}}
        living = {"phase": "Active", "life": {"health": 1}}
        self.assertTrue(runner.competitor_removal_allowed(2, corpse, {2}))
        self.assertTrue(runner.competitor_removal_allowed(2, removed, {2}))
        self.assertFalse(runner.competitor_removal_allowed(2, corpse, set()))
        self.assertFalse(runner.competitor_removal_allowed(2, living, {2}))
        self.assertFalse(runner.competitor_removal_allowed(1, corpse, {1}))

    def test_existing_output_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "existing"
            output.mkdir()
            marker = output / "raw.jsonl"
            marker.write_text("preserve", encoding="utf-8")
            with self.assertRaises(FileExistsError):
                runner.require_new_output(output)
            self.assertEqual(marker.read_text(encoding="utf-8"), "preserve")

    def test_probe_output_cap_is_enforced_and_partial_output_survives(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            work = root / "work"
            work.mkdir()
            raw, err = root / "raw.jsonl", root / "stderr.log"
            with patch.object(runner, "MAX_OUTPUT_BYTES", 16):
                result = runner.run_probe(
                    [sys.executable, "-c", "import os; os.write(1, b'x' * 128)"],
                    work, raw, err,
                )
            self.assertEqual(result["status"], "failed")
            self.assertTrue(raw.exists())
            self.assertLessEqual(raw.stat().st_size, 16)

    def test_seed_scan_operates_on_temporary_registry_fixture(self):
        with tempfile.TemporaryDirectory() as temp:
            registry = Path(temp) / "experiments"
            registry.mkdir()
            (registry / "manifest.json").write_text(
                '{"seeds":[51099]}\n', encoding="utf-8"
            )
            result = runner.scan_seeds(registry, (51099,))
            self.assertEqual(result["unseen"], [])
            self.assertTrue(result["matches"]["51099"])


if __name__ == "__main__":
    unittest.main()
