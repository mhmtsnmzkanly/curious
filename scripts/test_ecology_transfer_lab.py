#!/usr/bin/env python3
"""Unit tests for registration gates and bounded raw-run validation."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("run_ecology_transfer_lab.py")
spec = importlib.util.spec_from_file_location("ecology_transfer_runner_test", SCRIPT)
runner = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(runner)


class TransferRunnerTests(unittest.TestCase):
    def registration(self, identity, scenarios):
        return {
            "status": "registered", "root_reviewed": True, "source": identity,
            "scenario_sha256": scenarios, "seed_roles": runner.expected_seed_roles(),
            "ticks_per_world": 600, "arms": list(runner.ARMS), "habitats": list(runner.HABITATS),
            "world_count": 45, "phase_seconds": 300, "child_seconds": 60,
            "child_address_space_bytes": 1024**3, "aggregate_output_bytes": 1024**3,
            "max_identities": 10000, "max_events_per_tick": 20000,
            "mutation_per_mille": 0, "genetics_enabled": True,
        }

    def test_frozen_design_dimensions_and_roles(self):
        self.assertEqual(len(runner.expected_seed_roles()), 9)
        self.assertEqual(len(runner.ARMS), 5)
        self.assertEqual(len(runner.expected_seed_roles()) * len(runner.ARMS), 45)
        self.assertEqual(runner.TICKS * 45, 27000)
        self.assertEqual(runner.HABITATS["sparse_resource"][0], "scenarios/ecology-traits-lab.json")
        self.assertIn("docs/ECOLOGY_TRANSFER_V1_SCAN_ADDENDUM_20261026.md", runner.SOURCE_PATHS)

    def test_registration_accepts_only_clean_exact_scope(self):
        identity = {"revision": "abc", "clean": True, "file_sha256": {}, "content_sha256": "def"}
        scenarios = {name: "0" * 64 for name in runner.HABITATS}
        reg = self.registration(identity, scenarios)
        runner.validate_registration(reg, identity, scenarios)
        reg["ticks_per_world"] = 601
        with self.assertRaisesRegex(ValueError, "ticks_per_world"):
            runner.validate_registration(reg, identity, scenarios)

    def test_registration_rejects_dirty_source_or_changed_seed_roles(self):
        identity = {"revision": "abc", "clean": False, "file_sha256": {}, "content_sha256": "def"}
        scenarios = {name: "0" * 64 for name in runner.HABITATS}
        reg = self.registration(identity, scenarios)
        with self.assertRaisesRegex(ValueError, "clean committed"):
            runner.validate_registration(reg, identity, scenarios)
        identity["clean"] = True
        reg["seed_roles"] = reg["seed_roles"][:-1]
        with self.assertRaisesRegex(ValueError, "seed roles"):
            runner.validate_registration(reg, identity, scenarios)

    def test_seed_scan_rejects_prior_use_and_accepts_fresh_inventory(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)
            (archive / "old.json").write_text(json.dumps({"seed": 62301}) + "\n")
            roles = [{"habitat": "meadow", "seed": 62301}]
            with self.assertRaisesRegex(ValueError, "already occur"):
                runner.scan_seed_roles(archive, roles)
            (archive / "old.json").unlink()
            audit = runner.scan_seed_roles(archive, roles)
            self.assertEqual(audit["unseen"], [62301])

    def test_seed_scan_explicitly_accepts_lines_over_helper_default(self):
        self.assertEqual(runner.SEED_SCAN_MAX_LINE_BYTES, 64 * 1024 * 1024)
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)
            record = archive / "large-calibration.json"
            record.write_bytes(
                b'{"seed":62301,"padding":"'
                + b"x" * (16 * 1024 * 1024 + 128)
                + b'"}\n'
            )
            with self.assertRaisesRegex(ValueError, "already occur"):
                runner.scan_seed_roles(archive, [{"habitat": "meadow", "seed": 62301}])

    def test_raw_validator_requires_all_ticks_and_endpoint_summary(self):
        with tempfile.TemporaryDirectory() as temp:
            raw = Path(temp) / "run.jsonl"
            rows = [{"type": "header", "initial_state": {"entities": [{}]}}]
            rows.extend({"type": "tick", "tick": tick, "birth_lineage": [], "mating_events": [], "death_lineage": []} for tick in range(1, runner.TICKS + 1))
            rows.append({"type": "summary", "ticks_completed": runner.TICKS, "checkpoint_restore_valid": True, "lineage_issues": 0, "unknown_generation_births": 0, "births_by_species": {"Herbivore": 0}, "deaths_by_species": {"Herbivore": 0}, "final_alive": {"Herbivore": 1}})
            raw.write_text("".join(json.dumps(row) + "\n" for row in rows))
            self.assertEqual(runner._read_run(raw)["ticks_completed"], runner.TICKS)
            malformed = list(rows)
            malformed[1] = {**malformed[1], "birth_lineage": [{"id": 2, "species": "Herbivore", "parent_ids": [0, 1]}], "mating_events": [{}]}
            raw.write_text("".join(json.dumps(row) + "\n" for row in malformed))
            with self.assertRaisesRegex(ValueError, "invalid compact birth lineage"):
                runner._read_run(raw)
            raw.write_text(raw.read_text().replace('"tick": 600', '"tick": 599'))
            with self.assertRaisesRegex(ValueError, "tick sequence"):
                runner._read_run(raw)


if __name__ == "__main__":
    unittest.main()
