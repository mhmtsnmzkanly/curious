import importlib.util
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("analyze_water_meal_negative_outcomes.py")
spec = importlib.util.spec_from_file_location("water_meal_negative_analysis", SCRIPT)
analysis = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(analysis)


class WaterMealNegativeAnalysisTests(unittest.TestCase):
    def make_run(self, directory, tick_order=None, mismatch_summary=False):
        name = "meadow-seed62501-control.jsonl"
        header = {"type": "header", "habitat": "meadow", "seed": 62501, "arm": "control",
                  "initial_state": {"entities": [{"id": 1, "generation": 0}]}}
        ticks = [{"type": "tick", "tick": i, "alive": {"Herbivore": 1 if i < 2 else 0},
                  "actor_resolver_events": [], "water_meal_interventions": [],
                  "death_lineage": ([{"id": 1, "generation": 0, "cause": "Starvation", "species": "Herbivore"}] if i == 2 else [])}
                 for i in range(1, 301)]
        if tick_order == "swap":
            ticks[0], ticks[1] = ticks[1], ticks[0]
        summary = {"type": "summary", "living_entity_ticks": 2, "plant_food_gained_energy": 0,
                   "water_gained": 0, "movement_energy_paid": 0, "water_meal_intervention_count": 0,
                   "final_alive": {"Herbivore": 0}, "focal_death_tick": None, "focal_death_cause": None}
        if mismatch_summary:
            summary["water_gained"] = 1
        payload = "\n".join(json.dumps(x) for x in [header, *ticks, summary]) + "\n"
        path = directory / name
        path.write_text(payload)
        raw = path.read_bytes()
        return path, {"raw_bytes": len(raw), "raw_sha256": hashlib.sha256(raw).hexdigest()}

    def test_read_run_rejects_manifest_hash_mismatch(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, entry = self.make_run(root)
            entry["raw_sha256"] = "0" * 64
            with self.assertRaisesRegex(ValueError, "manifest size/hash"):
                analysis.read_run(root, "meadow", 62501, "control", entry)

    def test_read_run_rejects_reordered_ticks_with_matching_hash(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, entry = self.make_run(root, tick_order="swap")
            with self.assertRaisesRegex(ValueError, "tick sequence"):
                analysis.read_run(root, "meadow", 62501, "control", entry)

    def test_read_run_reconciles_summary_to_events(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, entry = self.make_run(root, mismatch_summary=True)
            with self.assertRaisesRegex(ValueError, "accounting does not match summary water_gained"):
                analysis.read_run(root, "meadow", 62501, "control", entry)

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

    def test_lineage_events_ignore_same_tick_list_order(self):
        ticks = [{"tick": 20, "birth_lineage": [{"id": 2}, {"id": 1}]}]
        self.assertEqual(analysis.lineage_events(ticks, "birth_lineage"),
                         [(20, {"id": 1}), (20, {"id": 2})])


if __name__ == "__main__":
    unittest.main()
