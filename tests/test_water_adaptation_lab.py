import importlib.util
from pathlib import Path
import unittest

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts/run_water_adaptation_lab.py"
spec = importlib.util.spec_from_file_location("water_adaptation_lab", MODULE_PATH)
lab = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(lab)


class FrozenWaterLabAnalysisTests(unittest.TestCase):
    def test_frozen_table_stability_includes_context_entries(self):
        before = {"entities": [{"id": 1, "learner": {
            "values": [{"value": 8, "visits": 2}], "residuals": [],
            "context_entries": [{"last_seen_tick": 200, "observations": [-3]}]}}]}
        branch = {"state_after_treatment_before_reversal": before,
                  "ticks": [{"state_after": before}]}
        self.assertTrue(lab.frozen_tables_stable(branch))
        changed = {"entities": [{"id": 1, "learner": {
            "values": [{"value": 8, "visits": 2}], "residuals": [], "context_entries": []}}]}
        branch["ticks"] = [{"state_after": changed}]
        self.assertFalse(lab.frozen_tables_stable(branch))

    def test_physical_censor_metrics_and_b_drink_lag(self):
        branch = {"status": "ceiling", "ticks": [
            {"tick": 201, "focal_physical": {"water_gained_resolver": 4,
             "movement_energy_paid_resolver": 2, "thirst_tick": True, "water_before": 0,
             "alive_after": True, "energy_after": 70, "health_after": 100, "water_after": 4},
             "events": [{"EntityDrank": {"id": 1, "pos": {"x": 4, "y": 0}, "amount": 4}}],
             "state_after": {"entities": []}}
        ]}
        m = lab.derive_metrics(branch)
        self.assertEqual(m["water_acquired"], 4)
        self.assertEqual(m["movement_energy"], 2)
        self.assertEqual(m["thirst_ticks"], 1)
        self.assertEqual(m["dehydration_ticks"], 1)
        self.assertEqual(m["first_successful_drink_b_after_fork"]["lag_ticks"], 1)
        self.assertFalse(m["first_successful_drink_b_after_fork"]["censored"])

    def test_missing_drink_is_censored_on_early_death(self):
        m = lab.derive_metrics({"status": "focal_died", "ticks": []})
        self.assertIsNone(m["first_successful_drink_b_after_fork"]["lag_ticks"])
        self.assertTrue(m["first_successful_drink_b_after_fork"]["censored"])
        self.assertEqual(m["first_successful_drink_b_after_fork"]["censor_reason"], "death")

    def test_missing_drink_at_horizon_is_right_censored(self):
        m = lab.derive_metrics({"status": "ceiling", "ticks": [{"tick": 400, "focal_physical": {}}]})
        b = m["first_successful_drink_b_after_fork"]
        self.assertIsNone(b["lag_ticks"])
        self.assertTrue(b["censored"])
        self.assertEqual(b["censor_reason"], "horizon")

    def test_checkpoint_life_is_read_from_direct_life_field_and_unknown_pareto_fails(self):
        learner = {"values": [], "residuals": [], "context_entries": [{"observations": [-2]}]}
        state = {"entities": [{"id": 1, "life": {"health": 100}, "learner": learner}]}
        empty_branch = {"status": "ceiling", "ticks": [], "state_after_treatment_before_reversal": state,
                        "state_after_fork_interruption": state}
        record = {"seed": 1, "role": "calibration", "checkpoint_state": state,
                  "prefix": [{"focal_choice": {"fresh_two_source_choice": True}} for _ in range(5)],
                  "forks": {name: dict(empty_branch) for name in lab.ARMS}}
        result = lab.analyze_record(record)
        self.assertTrue(result["validity"]["focal_alive_at_fork"])
        self.assertTrue(result["validity"]["all_required_branches_present"])
        a = {"alive_ticks": 5, "water_acquired": 2, "movement_energy": 1, "thirst_ticks": 0,
             "dehydration_ticks": 0, "terminal": {"energy": 5, "health": None, "water": 3}}
        b = {"alive_ticks": 4, "water_acquired": 1, "movement_energy": 2, "thirst_ticks": 1,
             "dehydration_ticks": 1, "terminal": {"energy": 4, "health": None, "water": 2}}
        self.assertFalse(lab.arm_comparison(a, b)["pareto_dominates"])

    def test_replay_bundles_all_arms(self):
        branch = {"ticks": []}
        record = {"seed": 62199, "forks": {name: branch for name in lab.ARMS}}
        replay = lab.replay_for(record, "abc123")
        self.assertEqual(len(replay["arms"]), 4)
        self.assertEqual({a["name"] for a in replay["arms"]}, set(lab.ARMS))
        self.assertEqual(replay["source"], "abc123")


if __name__ == "__main__":
    unittest.main()
