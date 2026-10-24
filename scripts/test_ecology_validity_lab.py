import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


MODULE_PATH = Path(__file__).with_name("run_ecology_validity_lab.py")
SPEC = importlib.util.spec_from_file_location("ecology_validity_runner", MODULE_PATH)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class EcologyValidityRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.scenario = Path(self.temp.name) / "scenario.json"
        fixture = {
            "name": "mixed-refuge 4x4 food=1 water=1 age=steady",
            "founders": ([{"species": "Herbivore"}] * 576) + ([{"species": "Carnivore"}] * 32),
            "sites": [{}] * 976,
            "resources": [[], []] * 616,
        }
        self.scenario.write_text(json.dumps(fixture))
        self.hash = runner.sha256(self.scenario)
        self.old_hash = runner.SCENARIO_SHA256
        runner.SCENARIO_SHA256 = self.hash
        self.addCleanup(setattr, runner, "SCENARIO_SHA256", self.old_hash)
        self.old_baseline = runner.ACCEPTED_BASELINE
        runner.ACCEPTED_BASELINE = dict(self.old_baseline, scenario_sha256=self.hash)
        self.addCleanup(setattr, runner, "ACCEPTED_BASELINE", self.old_baseline)
        self.identity = {"revision": "abc", "clean": True}
        self.inventory_sha = runner.sha256(runner.PROJECT_ROOT / "docs/ECOLOGY_FIXTURE_SEED_AUDIT.json")

    def protocol(self, phase="calibration", roles=None):
        if roles is None:
            roles = [{"role": "calibration", "seed": 53099}]
        return {
            "status": "registered",
            "root_reviewed": True,
            "source": self.identity,
            "scenario_sha256": self.hash,
            "accepted_baseline": runner.ACCEPTED_BASELINE,
            "policies": {
                "mate_rendezvous": True,
                "reserve_opportunities": True,
                "water_residual_enabled": False,
            },
            "profile": runner.ACCEPTED_PROFILE,
            "seed_inventory_sha256": self.inventory_sha,
            "seed_inventory_evidence": "docs/ECOLOGY_FIXTURE_SEED_AUDIT.json",
            "seed_inventory_text_sources_scanned": 1093,
            "phase": phase,
            "ticks_per_world": 200 if phase == "calibration" else 500,
            "phase_world_count": 1 if phase == "calibration" else 3,
            "phase_total_ticks": 1500 if phase == "formal" else None,
            "resource_limits": runner.RESOURCE_LIMITS,
            "acceptance": runner.ACCEPTANCE[phase],
            "budget_forecast": ({
                "calibration_ticks": 200,
                "formal_ticks_total": 1500,
                "safety_factor": 2,
                "seed_scan_analysis_reserve_seconds": 60,
                "maximum_seconds": 300,
                "formula": "calibration_probe_seconds * (1500 / calibration_ticks) * 2 + 60",
            } if phase == "formal" else None),
            "seed_roles": roles,
        }

    def test_draft_cannot_start_world_run(self):
        protocol = self.protocol()
        protocol["status"] = "draft"
        with self.assertRaisesRegex(ValueError, "registered"):
            runner.validate_protocol(protocol, self.scenario, self.identity)

    def test_calibration_requires_one_unique_role_and_exact_horizon(self):
        self.assertEqual(runner.validate_protocol(self.protocol(), self.scenario, self.identity)[0], 200)
        duplicate = self.protocol(roles=[{"role": "a", "seed": 53001}, {"role": "b", "seed": 53001}])
        with self.assertRaisesRegex(ValueError, "unique"):
            runner.validate_protocol(duplicate, self.scenario, self.identity)
        too_long = self.protocol()
        too_long["ticks_per_world"] = 199
        with self.assertRaisesRegex(ValueError, "exactly 200"):
            runner.validate_protocol(too_long, self.scenario, self.identity)

    def test_formal_requires_three_roles_and_exact_policy_flags(self):
        roles = [{"role": f"formal-{seed}", "seed": seed} for seed in (53001, 53002, 53003)]
        self.assertEqual(len(runner.validate_protocol(self.protocol("formal", roles), self.scenario, self.identity)[1]), 3)
        wrong = self.protocol()
        wrong["policies"]["water_residual_enabled"] = True
        with self.assertRaisesRegex(ValueError, "policy"):
            runner.validate_protocol(wrong, self.scenario, self.identity)

    def test_dirty_checkout_is_rejected(self):
        dirty = dict(self.identity, clean=False)
        with self.assertRaisesRegex(ValueError, "clean"):
            runner.validate_protocol(self.protocol(), self.scenario, dirty)

    def test_formal_budget_forecast_is_frozen_formula(self):
        self.assertAlmostEqual(runner.estimate_formal_seconds(0.57, 200), 68.55)
        with self.assertRaisesRegex(ValueError, "invalid calibration"):
            runner.estimate_formal_seconds(1.0, 0)

    def test_jsonl_preserves_tick_zero_founders_after_a_founder_is_removed(self):
        founders = {str(i): "Herbivore" for i in range(1, 577)}
        founders.update({str(i): "Carnivore" for i in range(577, 609)})
        initial = {"tick": 0, "entities": [{"id": 1}, {"id": 577}]}
        header = {
            "schema": "curious-ecology-validity-jsonl-v1",
            "seed": 123,
            "requested_ticks": 1,
            "scenario_name": "fixture",
            "founders_at_tick_0": 608,
            "founder_species": founders,
            "initial_state": initial,
            "profile": runner.ACCEPTED_PROFILE,
            "explicit_policy": {"mate_rendezvous": True, "reserve_opportunities": True, "water_residual_enabled": False},
        }
        summary = {
            "schema": "curious-ecology-validity-v1",
            "seed": 123,
            "requested_ticks": 1,
            "ticks_completed": 1,
            "scenario_name": "fixture",
            "founders_at_tick_0": 608,
            "founder_species": founders,
            "initial_state": initial,
            "final_state": {"tick": 1},
            "profile": runner.ACCEPTED_PROFILE,
            "explicit_policy": header["explicit_policy"],
            "capture_limits": {"event_observations_cumulative": 0, "lineage_events_retained": 0},
        }
        tick = {"tick": 1, "alive": {"Herbivore": 4}, "event_counts_by_type": {}, "lineage_events": [], "audit_frame": {"tick": 1, "pre_state": initial, "post_state": {"tick": 1}, "events": []}}
        raw_path = Path(self.temp.name) / "seed.jsonl"
        raw_path.write_text("\n".join(json.dumps(row) for row in ({"header": header}, tick, {"summary": summary})) + "\n")
        raw = runner.read_jsonl(raw_path)
        self.assertEqual(len(raw["founder_species"]), 608)
        self.assertEqual(raw["per_tick"][0]["alive"], {"Herbivore": 4})

    def test_formal_demographic_pass_cannot_hide_resource_or_lineage_audit_failures(self):
        good = {
            "audit": {
                "births_by_generation": {"1": 1},
                "resource_reconciliation": {"food_nonzero_residual_ticks": 0, "water_nonzero_residual_ticks": 0},
                "resource_kind_replacement_summary": {"count": 0},
                "parent_event_issues": [],
                "unattributed_actor_events": [],
                "trait_payment_issues": [],
            },
            "per_tick": [
                {"alive": {"Herbivore": 2, "Carnivore": 2}, "births_by_species_this_tick": {"Herbivore": 1, "Carnivore": 1}}
                for _ in range(500)
            ],
            "living_bred_descendants_by_species": {"Herbivore": 1, "Carnivore": 1},
            "eligible_adjacent_mating_pairs_observed": {},
            "lineage_events": [{"event": {"EntityMated": {}}}],
        }
        outcome = runner.evaluate_record(good, "formal")
        self.assertTrue(outcome["passed"])
        self.assertFalse(outcome["extinct"])
        bad = json.loads(json.dumps(good))
        bad["audit"]["resource_reconciliation"]["water_nonzero_residual_ticks"] = 1
        bad["audit"]["parent_event_issues"].append({"kind": "unknown_parent"})
        outcome = runner.evaluate_record(bad, "formal")
        self.assertFalse(outcome["passed"])
        self.assertFalse(outcome["technical_pass"])

    def test_fresh_phase_seed_scanner_uses_complete_registry_logic(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)
            (archive / "manifest.json").write_text('{"seeds":[53001]}\n')
            audit = runner.scan_seeds(archive, [53001])
            self.assertEqual(audit["unseen"], [])
            (archive / "manifest.json").write_text('{"seeds":[999]}\n')
            audit = runner.scan_seeds(archive, [53001])
            self.assertEqual(audit["unseen"], [53001])

    def test_supervised_seed_scanner_returns_auditable_phase_result(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)
            (archive / "manifest.json").write_text('{"seeds":[7]}\n')
            audit = runner.scan_seeds_supervised(archive, [53001], 30)
            self.assertEqual(audit["unseen"], [53001])
            self.assertEqual(audit["scanned_text_files"], 1)


if __name__ == "__main__":
    unittest.main()
