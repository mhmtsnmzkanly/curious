"""Focused runner safeguards; all process fixtures are non-simulation commands."""
import importlib.util
from copy import deepcopy
import json
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
SEED_SCANNER_SPEC = importlib.util.spec_from_file_location(
    "run_ecology_traits_lab", ROOT / "scripts/run_ecology_traits_lab.py"
)
seed_scanner = importlib.util.module_from_spec(SEED_SCANNER_SPEC)
SEED_SCANNER_SPEC.loader.exec_module(seed_scanner)


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
        return {"tick": 0, "rng": runner.expected_initial_rng(seed), "bounds": [0, 4, 0, 4],
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

    def test_expected_initial_rng_matches_world_map_population(self):
        self.assertEqual(runner.expected_initial_rng(123), 14463473577699995176)
        self.assertEqual(runner.expected_initial_rng(51099), 15382464887130188488)
        state = self.fixture_initial_state(51099)
        state["rng"] = 51099
        with self.assertRaisesRegex(ValueError, "map-population draws"):
            runner.validate_initial_fixture(state, 51099, "fixture test")

    def test_reanalysis_provenance_requires_unchanged_original_capture(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive = root / "archive"
            source = root / "source"
            original_dir = archive / runner.OUTPUT_DIRS["calibration"]
            analysis_dir = archive / runner.CALIBRATION_REANALYSIS_DIR
            original_dir.mkdir(parents=True)
            analysis_dir.mkdir(parents=True)
            (source / "docs").mkdir(parents=True)
            bio = {
                "Cargo.toml": "cargo-manifest",
                "Cargo.lock": "cargo-lock",
                "src/lib.rs": "rust-library",
                "examples/lab.rs": "rust-lab",
            }
            old_files = {path: runner.hashlib.sha256(value.encode()).hexdigest()
                         for path, value in bio.items()}
            old_files.update({"scripts/run_lifelong_water_lab.py": "old-runner",
                              str(runner.PROTOCOL): "old-protocol"})
            current_files = {**old_files,
                             "scripts/run_lifelong_water_lab.py": "new-runner",
                             str(runner.PROTOCOL): "new-protocol",
                             str(runner.REANALYSIS_PROTOCOL): "reanalysis-doc"}
            protocol = source / runner.REANALYSIS_PROTOCOL
            protocol.parent.mkdir(parents=True, exist_ok=True)
            protocol.write_text("analysis protocol", encoding="utf-8")
            (analysis_dir / "protocol.md").write_text("analysis protocol", encoding="utf-8")
            frozen_protocol = source / runner.PROTOCOL
            frozen_protocol.write_text("frozen protocol", encoding="utf-8")
            (original_dir / "protocol.md").write_text("frozen protocol", encoding="utf-8")
            raw = original_dir / "raw.jsonl"
            raw.write_text("captured raw", encoding="utf-8")
            probe = original_dir / "lab-probe"
            probe.write_bytes(b"immutable probe")
            original = {
                "schema": runner.MANIFEST_SCHEMA, "protocol_version": runner.PROTOCOL_VERSION,
                "role": "calibration", "status": "failed", "revision": "old-rev",
                "exit_code": 0, "timed_out": False, "output_limit_exceeded": False,
                "raw_bytes": raw.stat().st_size, "raw_sha256": runner.sha256_file(raw),
                "binary_sha256": runner.sha256_file(probe),
                "protocol_sha256": runner.sha256_file(frozen_protocol), "source_sha256": "old-source",
                "source_files_sha256": old_files, "runner_sha256": "old-runner",
                "summary_sha256": None,
            }
            manifest_path = original_dir / "manifest.json"
            manifest_path.write_text(json.dumps(original), encoding="utf-8")
            current = {
                "revision": "new-rev", "binary_sha256": original["binary_sha256"],
                "protocol_sha256": original["protocol_sha256"],
                "runner_sha256": "new-runner", "source_files_sha256": current_files,
                "source_sha256": runner.source_map_sha256(current_files),
            }
            reanalysis = {
                "schema": "curious-lifetime-lab-reanalysis-v2", "status": "completed",
                "role": "calibration", "seeds": [51099], "ticks_replayed": 0,
                "zero_new_ticks": True, "probe_launched": False, "revision": "new-rev",
                "original_run_dir": runner.OUTPUT_DIRS["calibration"],
                "original_manifest_sha256": runner.sha256_file(manifest_path),
                "original_raw_sha256": original["raw_sha256"],
                "original_probe_sha256": original["binary_sha256"],
                "protocol_sha256": original["protocol_sha256"],
                "original_source_sha256": original["source_sha256"],
                "original_source_files_sha256": old_files,
                "original_runner_sha256": original["runner_sha256"],
                "current_source_sha256": current["source_sha256"],
                "current_source_files_sha256": current_files,
                "original_summary_present": False, "original_summary_sha256": None,
                "validator_sha256": current["runner_sha256"],
                "reanalysis_protocol_sha256": runner.sha256_file(protocol),
                "expected_initial_rng_by_seed": {"51099": runner.expected_initial_rng(51099)},
            }
            (analysis_dir / "reanalysis.json").write_text(json.dumps(reanalysis), encoding="utf-8")
            valid = runner.validate_calibration_reanalysis(
                archive, current, original, manifest_path, raw, reanalysis, source,
            )
            self.assertEqual(valid["raw_sha256"], original["raw_sha256"])
            for artifact in (raw, probe, original_dir / "protocol.md"):
                saved = artifact.read_bytes()
                artifact.write_bytes(saved + b"tamper")
                with self.subTest(artifact=artifact.name), self.assertRaises(RuntimeError):
                    runner.validate_calibration_reanalysis(
                        archive, current, original, manifest_path, raw, reanalysis, source,
                    )
                artifact.write_bytes(saved)
            for field, value in (("original_probe_sha256", "forged"),
                                 ("ticks_replayed", 1),
                                 ("current_source_sha256", "different-source"),
                                 ("validator_sha256", "different-validator")):
                altered = {**reanalysis, field: value}
                with self.subTest(field=field), self.assertRaises(RuntimeError):
                    runner.validate_calibration_reanalysis(
                        archive, current, original, manifest_path, raw, altered, source,
                    )

    def test_reanalysis_pass_flag_cannot_override_fresh_failed_gate(self):
        reanalysis = {
            "calibration_validity": {"passed": True, "checks": {"fixture": True}},
            "validation_result": {"passed": True, "errors": []},
        }
        fresh_gate = {"passed": False, "checks": {"fixture": False}}
        with self.assertRaisesRegex(RuntimeError, "does not reproduce"):
            runner.require_passing_reanalysis_gate(reanalysis, fresh_gate)

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

    def test_seed_scanner_accepts_explicit_64mib_bound_and_rejects_larger_line(self):
        with tempfile.TemporaryDirectory() as temp:
            registry = Path(temp)
            raw = registry / "captured.jsonl"
            raw.write_bytes(b'{"seed":51099,"payload":"' + b"x" * (17 * 1024 * 1024) + b'"}\n')
            with self.assertRaisesRegex(RuntimeError, "oversized text line"):
                seed_scanner.audit_seeds(registry, [51099])
            result = seed_scanner.audit_seeds(
                registry, [51099], max_line_bytes=64 * 1024 * 1024,
            )
            self.assertTrue(result["matches"]["51099"])
            raw.write_bytes(b'{"seed":51099,"payload":"' + b"x" * (64 * 1024 * 1024) + b'"}\n')
            with self.assertRaisesRegex(RuntimeError, "oversized text line"):
                seed_scanner.audit_seeds(
                    registry, [51099], max_line_bytes=64 * 1024 * 1024,
                )


if __name__ == "__main__":
    unittest.main()
