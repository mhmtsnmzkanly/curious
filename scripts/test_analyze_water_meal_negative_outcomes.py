import importlib.util
from pathlib import Path
import unittest

SCRIPT = Path(__file__).with_name("analyze_water_meal_negative_outcomes.py")
spec = importlib.util.spec_from_file_location("water_meal_negative_analysis", SCRIPT)
analysis = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(analysis)


class WaterMealNegativeAnalysisTests(unittest.TestCase):
    def test_food_resolution_distinguishes_intake_from_rejected_selection(self):
        tick = {"actor_resolver_events": [
            {"id": 5, "event": {"ActionRejected": {"action": "Eat", "reason": "MissingResource"}}},
            {"id": 6, "event": {"EntityAte": {"amount": 2, "corpse_id": None}}},
        ]}
        self.assertEqual(analysis.eat_outcomes(tick), {
            "plant_meal_energy_by_actor": {6: 2},
            "rejected_eat_counts_by_reason": {"MissingResource": 1},
        })

    def test_alive_exposure_uses_observed_tick_populations(self):
        ticks = [{"alive": {"Herbivore": 3}}, {"alive": {"Herbivore": 1}}]
        self.assertEqual(analysis.alive_exposure(ticks, 1), 3)
        self.assertEqual(analysis.alive_exposure(ticks, 2), 4)

    def test_first_lineage_mismatch_includes_parent_identity(self):
        c = [(20, {"id": 9, "parent_ids": [1, 3]})]
        w = [(20, {"id": 9, "parent_ids": [2, 4]})]
        self.assertEqual(analysis.first_lineage_mismatch(c, w), {
            "control": {"tick": 20, "event": c[0][1]},
            "water_meal_only": {"tick": 20, "event": w[0][1]},
        })


if __name__ == "__main__":
    unittest.main()
